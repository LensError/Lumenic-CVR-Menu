using System;
using System.Collections.Generic;
using System.IO;
using System.Linq;
using System.Net;
using System.Net.Http;
using System.Net.WebSockets;
using System.Text;
using System.Threading;
using System.Threading.Tasks;
using LumenicMenu.Api;
using MelonLoader;
using Newtonsoft.Json;

namespace LumenicEventCalendar;

internal sealed class CalendarApiService : IDisposable
{
    public const string DefaultBaseUrl = "https://cvreventsapi.lenserror.me";
    public const string DefaultWsUrl = "wss://cvreventsapi.lenserror.me";
    private const int MaxApiResponseBytes = 4 * 1024 * 1024;
    private const int MaxRegisteredImageKeys = 256;

    private static readonly JsonSerializerSettings SafeJsonSettings = new JsonSerializerSettings
    {
        MaxDepth = 32,
        TypeNameHandling = TypeNameHandling.None
    };

    private readonly HttpClient _http;
    private readonly object _stateLock = new object();
    private readonly CancellationTokenSource _cts = new CancellationTokenSource();
    private readonly HashSet<string> _registeredImageKeys = new HashSet<string>(StringComparer.Ordinal);

    private List<CalendarEvent> _events = new List<CalendarEvent>();
    private bool _isLoading;
    private bool _wsConnected;
    private string _lastError = string.Empty;
    private DateTimeOffset? _lastUpdatedLocal;
    private int _fetchInFlight;
    private volatile bool _disposed;

    public event Action OnDataChanged;

    public string BaseUrl { get; }
    public string WsUrl { get; }

    public CalendarApiService(string baseUrl = DefaultBaseUrl, string wsUrl = DefaultWsUrl)
    {
        BaseUrl = string.IsNullOrWhiteSpace(baseUrl) ? DefaultBaseUrl : baseUrl.TrimEnd('/');
        WsUrl = string.IsNullOrWhiteSpace(wsUrl) ? DefaultWsUrl : wsUrl.TrimEnd('/');

        var handler = new HttpClientHandler
        {
            AutomaticDecompression = DecompressionMethods.GZip | DecompressionMethods.Deflate
        };
        _http = new HttpClient(handler)
        {
            Timeout = TimeSpan.FromSeconds(15)
        };
        _http.DefaultRequestHeaders.UserAgent.ParseAdd("LumenicEventCalendar/1.0.0");
    }

    public void Start()
    {
        _ = FetchEventsAsync(manual: false);
        _ = Task.Run(() => WebSocketLoopAsync(_cts.Token));
    }

    public void TriggerRefresh(bool manual = true)
    {
        if (_disposed) return;
        _ = FetchEventsAsync(manual);
    }

    public (List<CalendarEvent> Events, bool IsLoading, bool WsConnected, string LastError, DateTimeOffset? LastUpdated) GetSnapshot()
    {
        lock (_stateLock)
        {
            return (new List<CalendarEvent>(_events), _isLoading, _wsConnected, _lastError, _lastUpdatedLocal);
        }
    }

    private async Task FetchEventsAsync(bool manual)
    {
        if (_disposed) return;
        if (Interlocked.Exchange(ref _fetchInFlight, 1) == 1)
            return;

        try
        {
            lock (_stateLock)
            {
                _isLoading = true;
                if (manual) _lastError = string.Empty;
            }
            NotifyChanged();

            using var req = new HttpRequestMessage(HttpMethod.Get, $"{BaseUrl}/api/events");
            using HttpResponseMessage resp = await _http.SendAsync(req, HttpCompletionOption.ResponseHeadersRead, _cts.Token).ConfigureAwait(false);
            resp.EnsureSuccessStatusCode();

            string json = await ReadBoundedStringAsync(resp.Content, MaxApiResponseBytes, _cts.Token).ConfigureAwait(false);
            var parsed = JsonConvert.DeserializeObject<List<CalendarEvent>>(json, SafeJsonSettings) ?? new List<CalendarEvent>();

            var activeKeys = new HashSet<string>(StringComparer.Ordinal);
            foreach (CalendarEvent ev in parsed)
            {
                ev.Hydrate(BaseUrl);
                if (!string.IsNullOrEmpty(ev.LogoImageKey)) activeKeys.Add(ev.LogoImageKey);
                if (!string.IsNullOrEmpty(ev.BannerImageKey)) activeKeys.Add(ev.BannerImageKey);
                RegisterEventImages(ev);
            }
            lock (_registeredImageKeys)
            {
                _registeredImageKeys.IntersectWith(activeKeys);
            }

            SortEvents(parsed);

            lock (_stateLock)
            {
                _events = parsed;
                _isLoading = false;
                _lastError = string.Empty;
                _lastUpdatedLocal = DateTimeOffset.Now;
            }

            if (manual)
                LumenicApi.ShowToast($"Calendar refreshed ({parsed.Count} events)", 2.2f);
        }
        catch (OperationCanceledException)
        {
            // Shutting down.
        }
        catch (Exception ex)
        {
            MelonLogger.Warning($"[EventCalendar] Failed to fetch events: {ex.Message}");
            lock (_stateLock)
            {
                _isLoading = false;
                _lastError = $"Could not reach calendar API: {ex.Message}";
            }
            if (manual)
                LumenicApi.ShowToast("Failed to refresh calendar", 2.5f);
        }
        finally
        {
            Interlocked.Exchange(ref _fetchInFlight, 0);
            NotifyChanged();
        }
    }

    private async Task WebSocketLoopAsync(CancellationToken ct)
    {
        int backoffSeconds = 3;

        while (!ct.IsCancellationRequested && !_disposed)
        {
            ClientWebSocket ws = null;
            try
            {
                ws = new ClientWebSocket();
                ws.Options.KeepAliveInterval = TimeSpan.FromSeconds(25);
                await ws.ConnectAsync(new Uri(WsUrl), ct).ConfigureAwait(false);

                backoffSeconds = 3;
                SetWsConnected(true);

                const int MaxWsMessageBytes = 512 * 1024;
                var buffer = new byte[16 * 1024];
                using var ms = new MemoryStream();

                while (ws.State == WebSocketState.Open && !ct.IsCancellationRequested)
                {
                    ms.SetLength(0);
                    bool oversized = false;
                    WebSocketReceiveResult result;
                    do
                    {
                        result = await ws.ReceiveAsync(new ArraySegment<byte>(buffer), ct).ConfigureAwait(false);
                        if (result.MessageType == WebSocketMessageType.Close)
                            break;
                        if (!oversized)
                        {
                            if (ms.Length + result.Count > MaxWsMessageBytes)
                            {
                                oversized = true;
                                ms.SetLength(0);
                            }
                            else
                            {
                                ms.Write(buffer, 0, result.Count);
                            }
                        }
                    }
                    while (!result.EndOfMessage);

                    if (result.MessageType == WebSocketMessageType.Close)
                        break;

                    if (!oversized && result.MessageType == WebSocketMessageType.Text && ms.Length > 0)
                    {
                        string text = Encoding.UTF8.GetString(ms.GetBuffer(), 0, (int)ms.Length);
                        HandleWebSocketMessage(text);
                    }
                }
            }
            catch (OperationCanceledException)
            {
                break;
            }
            catch (Exception ex)
            {
                MelonLogger.Msg($"[EventCalendar] WebSocket disconnected ({ex.Message}); retrying in {backoffSeconds}s.");
            }
            finally
            {
                SetWsConnected(false);
                try { ws?.Dispose(); } catch { }
            }

            if (ct.IsCancellationRequested || _disposed)
                break;

            try
            {
                await Task.Delay(TimeSpan.FromSeconds(backoffSeconds), ct).ConfigureAwait(false);
            }
            catch (OperationCanceledException)
            {
                break;
            }
            backoffSeconds = Math.Min(backoffSeconds * 2, 60);
        }
    }

    private void HandleWebSocketMessage(string rawJson)
    {
        try
        {
            var msg = JsonConvert.DeserializeObject<WsBroadcastMessage>(rawJson, SafeJsonSettings);
            if (msg == null || string.IsNullOrEmpty(msg.Action))
                return;

            string action = msg.Action.Trim().ToLowerInvariant();
            if (action == "created" || action == "edited" || action == "deleted" || action == "image_updated")
            {
                // Apply instant local patch if event payload is present, then sync cleanly via GET.
                if ((action == "created" || action == "edited") && msg.Event != null && !string.IsNullOrEmpty(msg.Event.Id))
                {
                    msg.Event.Hydrate(BaseUrl);
                    RegisterEventImages(msg.Event);
                    lock (_stateLock)
                    {
                        int idx = _events.FindIndex(e => string.Equals(e.Id, msg.Event.Id, StringComparison.Ordinal));
                        if (idx >= 0) _events[idx] = msg.Event;
                        else _events.Add(msg.Event);
                        SortEvents(_events);
                        _lastUpdatedLocal = DateTimeOffset.Now;
                    }
                    NotifyChanged();
                }
                else if (action == "deleted" && msg.Event != null && !string.IsNullOrEmpty(msg.Event.Id))
                {
                    lock (_stateLock)
                    {
                        _events.RemoveAll(e => string.Equals(e.Id, msg.Event.Id, StringComparison.Ordinal));
                        _lastUpdatedLocal = DateTimeOffset.Now;
                    }
                    NotifyChanged();
                }
                else
                {
                    _ = FetchEventsAsync(manual: false);
                }
            }
        }
        catch (Exception ex)
        {
            MelonLogger.Warning($"[EventCalendar] Failed to process WebSocket message: {ex.Message}");
        }
    }

    private void RegisterEventImages(CalendarEvent ev)
    {
        if (!string.IsNullOrEmpty(ev.LogoImageKey) && !string.IsNullOrEmpty(ev.ResolvedLogoUrl))
        {
            bool shouldRegister;
            lock (_registeredImageKeys)
            {
                if (_registeredImageKeys.Count >= MaxRegisteredImageKeys)
                    _registeredImageKeys.Clear();
                shouldRegister = _registeredImageKeys.Add(ev.LogoImageKey);
            }
            if (shouldRegister)
                LumenicApi.RegisterImageUrl(ev.LogoImageKey, ev.ResolvedLogoUrl, ExtSurface.MainMenu);
        }

        if (!string.IsNullOrEmpty(ev.BannerImageKey) && !string.IsNullOrEmpty(ev.ResolvedBannerUrl))
        {
            bool shouldRegister;
            lock (_registeredImageKeys)
            {
                if (_registeredImageKeys.Count >= MaxRegisteredImageKeys)
                    _registeredImageKeys.Clear();
                shouldRegister = _registeredImageKeys.Add(ev.BannerImageKey);
            }
            if (shouldRegister)
                LumenicApi.RegisterImageUrl(ev.BannerImageKey, ev.ResolvedBannerUrl, ExtSurface.MainMenu);
        }
    }

    private static async Task<string> ReadBoundedStringAsync(HttpContent content, int maxBytes, CancellationToken ct)
    {
        using Stream stream = await content.ReadAsStreamAsync().ConfigureAwait(false);
        using var ms = new MemoryStream();
        byte[] buffer = new byte[16384];
        int total = 0;
        while (true)
        {
            int read = await stream.ReadAsync(buffer, 0, buffer.Length, ct).ConfigureAwait(false);
            if (read <= 0)
                break;
            total += read;
            if (total > maxBytes)
                throw new InvalidOperationException("Calendar API response exceeded maximum allowed size.");
            ms.Write(buffer, 0, read);
        }
        return Encoding.UTF8.GetString(ms.GetBuffer(), 0, (int)ms.Length);
    }

    private static void SortEvents(List<CalendarEvent> list)
    {
        DateTimeOffset nowUtc = DateTimeOffset.UtcNow;
        list.Sort((a, b) =>
        {
            // Live active events float to the top, then chronological by next occurrence, then cancelled/postponed.
            int aRank = a.IsLiveNow(nowUtc) ? 0 : (a.IsActive ? 1 : 2);
            int bRank = b.IsLiveNow(nowUtc) ? 0 : (b.IsActive ? 1 : 2);
            int cmp = aRank.CompareTo(bRank);
            if (cmp != 0) return cmp;

            DateTimeOffset aTime = a.NextStartUtc ?? DateTimeOffset.MaxValue;
            DateTimeOffset bTime = b.NextStartUtc ?? DateTimeOffset.MaxValue;
            cmp = aTime.CompareTo(bTime);
            if (cmp != 0) return cmp;

            return string.Compare(a.Title, b.Title, StringComparison.OrdinalIgnoreCase);
        });
    }

    private void SetWsConnected(bool connected)
    {
        bool changed;
        lock (_stateLock)
        {
            changed = _wsConnected != connected;
            _wsConnected = connected;
        }
        if (changed) NotifyChanged();
    }

    private void NotifyChanged()
    {
        try { OnDataChanged?.Invoke(); }
        catch (Exception ex) { MelonLogger.Error($"[EventCalendar] OnDataChanged error: {ex}"); }
    }

    public void Dispose()
    {
        if (_disposed) return;
        _disposed = true;
        try { _cts.Cancel(); } catch { }
        try { _cts.Dispose(); } catch { }
        try { _http.Dispose(); } catch { }
    }
}
