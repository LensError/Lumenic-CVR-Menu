using System;
using System.Collections.Concurrent;
using System.Collections.Generic;
using System.IO;
using System.Net.Http;
using System.Threading;
using System.Threading.Tasks;
using ABI_RC.Core.IO;
using ABI_RC.Core.Savior;
using ABI_RC.Systems.UI.UILib;
using LumenicMenu.Native;
using MelonLoader;

namespace LumenicMenu.Bridge;

/// <summary>
/// Asynchronous non-blocking image loader for local UILib PNG icons, CVR coui:// cached images,
/// and remote CVR URLs (e.g. Player PFP and content thumbnails).
/// Caps concurrent remote HTTP fetches, enforces streaming size limits, and retries transient failures after a TTL.
/// </summary>
internal sealed class ImageLoaderService : IDisposable
{
    private const int MaxSurfaceTextures = 512;
    private const int MaxRemoteBytes = 8 * 1024 * 1024;
    private const int MaxFailedEntries = 256;
    private const double FailedRetryTtlSeconds = 30.0;
    private const string CouiUiResourcesPrefix = "coui://UIResources/";

    private static readonly HttpClient SharedHttp = new HttpClient
    {
        Timeout = TimeSpan.FromSeconds(10)
    };

    private static readonly SemaphoreSlim RemoteFetchGate = new SemaphoreSlim(6, 6);

    private readonly ConcurrentQueue<(uint SurfaceId, string Key, byte[] Bytes, int Token)> _pendingUploads =
        new ConcurrentQueue<(uint, string, byte[], int)>();

    private readonly object _cacheLock = new object();
    private readonly Dictionary<string, byte[]> _bytesCache = new Dictionary<string, byte[]>(StringComparer.Ordinal);
    private readonly LinkedList<string> _bytesOrder = new LinkedList<string>();
    private readonly HashSet<(uint SurfaceId, string Key)> _inFlight = new HashSet<(uint, string)>();
    private readonly Dictionary<string, DateTime> _failedKeys = new Dictionary<string, DateTime>(StringComparer.Ordinal);
    private readonly HashSet<(uint SurfaceId, string Key)> _queuedUploads = new HashSet<(uint, string)>();

    // Main-thread-only upload state matching Rust's per-surface LRU/FIFO texture cap.
    private readonly Dictionary<uint, HashSet<string>> _uploadedPerSurface = new Dictionary<uint, HashSet<string>>();
    private readonly Dictionary<uint, LinkedList<string>> _uploadedOrderPerSurface = new Dictionary<uint, LinkedList<string>>();

    private string _lastLocalPfpUrl = string.Empty;
    private int _localPfpToken;
    private volatile bool _disposed;

    static ImageLoaderService()
    {
        try
        {
            SharedHttp.DefaultRequestHeaders.UserAgent.ParseAdd("LumenicMenu/0.2");
        }
        catch
        {
            // Ignore if header already set.
        }
    }

    public void InvalidateUploaded()
    {
        while (_pendingUploads.TryDequeue(out _)) { }
        lock (_cacheLock)
        {
            _queuedUploads.Clear();
            _failedKeys.Clear();
        }
        _uploadedPerSurface.Clear();
        _uploadedOrderPerSurface.Clear();
    }

    public void RequestLocalPfp(uint surfaceId, string url)
    {
        if (_disposed || string.IsNullOrWhiteSpace(url))
            return;

        int token;
        if (!string.Equals(url, _lastLocalPfpUrl, StringComparison.Ordinal))
        {
            _lastLocalPfpUrl = url;
            RemoveUploadedKey(NativeBridge.SurfaceMain, "local_pfp");
            RemoveUploadedKey(NativeBridge.SurfaceQuick, "local_pfp");
            lock (_cacheLock)
            {
                token = ++_localPfpToken;
                _bytesCache.Remove("local_pfp");
                _failedKeys.Remove("local_pfp");
                _inFlight.Remove((NativeBridge.SurfaceMain, "local_pfp"));
                _inFlight.Remove((NativeBridge.SurfaceQuick, "local_pfp"));
                _queuedUploads.Remove((NativeBridge.SurfaceMain, "local_pfp"));
                _queuedUploads.Remove((NativeBridge.SurfaceQuick, "local_pfp"));
            }
        }
        else
        {
            lock (_cacheLock)
            {
                token = _localPfpToken;
            }
        }

        RequestRemoteUrl(surfaceId, "local_pfp", url, token);
    }

    /// <summary>
    /// Requests a remote content thumbnail URL (Worlds, Avatars, Props, Search) on the target surface
    /// and returns a deterministic texture key matching Rust's `main_textures` cache.
    /// </summary>
    public string RequestThumbnail(uint surfaceId, string url)
    {
        if (_disposed || string.IsNullOrWhiteSpace(url))
            return string.Empty;

        string trimmed = url.Trim();
        if (!trimmed.StartsWith("http://", StringComparison.OrdinalIgnoreCase)
            && !trimmed.StartsWith("https://", StringComparison.OrdinalIgnoreCase))
        {
            return string.Empty;
        }

        string key = "thumb:" + trimmed;
        RequestRemoteUrl(surfaceId, key, trimmed, token: 0);
        return key;
    }

    public void RequestUrl(uint surfaceId, string key, string url)
    {
        if (_disposed || string.IsNullOrWhiteSpace(key) || string.IsNullOrWhiteSpace(url))
            return;

        string trimmed = url.Trim();
        if (!trimmed.StartsWith("http://", StringComparison.OrdinalIgnoreCase)
            && !trimmed.StartsWith("https://", StringComparison.OrdinalIgnoreCase))
        {
            return;
        }

        RequestRemoteUrl(surfaceId, key, trimmed, token: 0);
    }

    /// <summary>
    /// Resolves a UILib icon reference (local mod PNG, built-in UILib PNG, coui://UIResources/ cache path,
    /// or remote HTTPS URL) into a deterministic texture key and schedules async loading if not yet uploaded.
    /// </summary>
    public string RequestUiLibIcon(uint surfaceId, string modName, string iconName)
    {
        if (_disposed || string.IsNullOrWhiteSpace(iconName))
            return string.Empty;

        string trimmed = iconName.Trim();
        if (trimmed.StartsWith("http://", StringComparison.OrdinalIgnoreCase)
            || trimmed.StartsWith("https://", StringComparison.OrdinalIgnoreCase))
        {
            string urlKey = "url:" + trimmed;
            RequestRemoteUrl(surfaceId, urlKey, trimmed, token: 0);
            return urlKey;
        }

        if (trimmed.StartsWith(CouiUiResourcesPrefix, StringComparison.OrdinalIgnoreCase))
        {
            string rel = trimmed.Substring(CouiUiResourcesPrefix.Length);
            string couiKey = "coui:" + rel;
            ScheduleLocalTask(surfaceId, couiKey, () => LoadCouiResourceAsync(surfaceId, rel, couiKey));
            return couiKey;
        }

        string cleanMod = string.IsNullOrWhiteSpace(modName) ? string.Empty : UIUtils.GetCleanString(modName);
        string localKey = string.IsNullOrEmpty(cleanMod)
            ? $"uilib:{trimmed}"
            : $"uilib:{cleanMod}/{trimmed}";

        ScheduleLocalTask(surfaceId, localKey, () => LoadLocalUiLibIconAsync(surfaceId, cleanMod, trimmed, localKey));
        return localKey;
    }

    private void ScheduleLocalTask(uint surfaceId, string key, Action loader)
    {
        if (_disposed || !TryAcquireFetchSlot(surfaceId, key, token: 0))
            return;

        Task.Run(loader);
    }

    private void RequestRemoteUrl(uint surfaceId, string key, string url, int token)
    {
        if (_disposed || !IsAllowedRemoteUrl(url) || !TryAcquireFetchSlot(surfaceId, key, token))
            return;

        Task.Run(() => FetchRemoteImageAsync(surfaceId, key, url, token));
    }

    private bool TryAcquireFetchSlot(uint surfaceId, string key, int token)
    {
        if (IsUploaded(surfaceId, key))
            return false;

        var uploadId = (surfaceId, key);
        lock (_cacheLock)
        {
            if (_queuedUploads.Contains(uploadId))
                return false;

            if (_bytesCache.TryGetValue(key, out byte[] cached))
            {
                _queuedUploads.Add(uploadId);
                _pendingUploads.Enqueue((surfaceId, key, cached, token));
                return false;
            }
            if (IsRecentlyFailedLocked(key) || _inFlight.Contains(uploadId))
                return false;
            _inFlight.Add(uploadId);
            return true;
        }
    }

    private bool IsRecentlyFailedLocked(string key)
    {
        if (!_failedKeys.TryGetValue(key, out DateTime failedAt))
            return false;
        if ((DateTime.UtcNow - failedAt).TotalSeconds < FailedRetryTtlSeconds)
            return true;
        _failedKeys.Remove(key);
        return false;
    }

    private static bool IsAllowedRemoteUrl(string url)
    {
        if (string.IsNullOrWhiteSpace(url))
            return false;

        foreach (string prefix in ImageCache.ExpectedUrlPrefixes)
        {
            if (url.StartsWith(prefix, StringComparison.OrdinalIgnoreCase))
                return true;
        }
        return false;
    }

    private static bool TryResolveContainedPath(string rootDir, string relativePath, out string fullPath)
    {
        fullPath = null;
        if (string.IsNullOrWhiteSpace(relativePath) || Path.IsPathRooted(relativePath))
            return false;

        try
        {
            string normalizedRoot = Path.GetFullPath(rootDir);
            if (!normalizedRoot.EndsWith(Path.DirectorySeparatorChar.ToString(), StringComparison.Ordinal))
                normalizedRoot += Path.DirectorySeparatorChar;

            string candidate = Path.GetFullPath(Path.Combine(normalizedRoot, relativePath.Replace('/', Path.DirectorySeparatorChar)));
            if (!candidate.StartsWith(normalizedRoot, StringComparison.OrdinalIgnoreCase))
                return false;

            fullPath = candidate;
            return true;
        }
        catch
        {
            return false;
        }
    }

    private void LoadCouiResourceAsync(uint surfaceId, string relativePath, string key)
    {
        try
        {
            string rootDir = Path.Combine(MetaPort.APPLICATION_STREAMINGASSETPATH, "Cohtml", "UIResources");
            if (!TryResolveContainedPath(rootDir, relativePath, out string fullPath) || !File.Exists(fullPath))
            {
                MarkFailed(surfaceId, key);
                return;
            }

            byte[] bytes = File.ReadAllBytes(fullPath);
            if (bytes.Length == 0 || bytes.Length > MaxRemoteBytes)
            {
                MarkFailed(surfaceId, key);
                return;
            }

            StoreAndEnqueue(surfaceId, key, bytes, token: 0);
        }
        catch
        {
            MarkFailed(surfaceId, key);
        }
    }

    private void LoadLocalUiLibIconAsync(uint surfaceId, string cleanMod, string iconName, string key)
    {
        try
        {
            string baseDir = Path.Combine(
                MetaPort.APPLICATION_STREAMINGASSETPATH,
                "Cohtml", "UIResources", "GameUI", "UILib", "images");

            string safeName = iconName.EndsWith(".png", StringComparison.OrdinalIgnoreCase)
                ? iconName
                : iconName + ".png";

            string path = null;
            if (!string.IsNullOrEmpty(cleanMod)
                && TryResolveContainedPath(baseDir, Path.Combine(cleanMod, safeName), out string modCandidate)
                && File.Exists(modCandidate))
            {
                path = modCandidate;
            }
            if (path == null
                && TryResolveContainedPath(baseDir, safeName, out string rootCandidate)
                && File.Exists(rootCandidate))
            {
                path = rootCandidate;
            }

            if (path == null)
            {
                MarkFailed(surfaceId, key);
                return;
            }

            byte[] bytes = File.ReadAllBytes(path);
            if (bytes.Length == 0 || bytes.Length > MaxRemoteBytes)
            {
                MarkFailed(surfaceId, key);
                return;
            }

            StoreAndEnqueue(surfaceId, key, bytes, token: 0);
        }
        catch
        {
            MarkFailed(surfaceId, key);
        }
    }

    private async Task FetchRemoteImageAsync(uint surfaceId, string key, string url, int token)
    {
        bool acquired = false;
        try
        {
            await RemoteFetchGate.WaitAsync().ConfigureAwait(false);
            acquired = true;
            if (_disposed)
                return;

            using HttpResponseMessage resp = await SharedHttp.GetAsync(url, HttpCompletionOption.ResponseHeadersRead).ConfigureAwait(false);
            if (!resp.IsSuccessStatusCode)
            {
                MarkFailed(surfaceId, key, token);
                return;
            }

            long? contentLen = resp.Content.Headers.ContentLength;
            if (contentLen.HasValue && contentLen.Value > MaxRemoteBytes)
            {
                MarkFailed(surfaceId, key, token);
                return;
            }

            byte[] bytes = await ReadBoundedBytesAsync(resp.Content, MaxRemoteBytes).ConfigureAwait(false);
            if (bytes == null || bytes.Length == 0)
            {
                MarkFailed(surfaceId, key, token);
                return;
            }

            StoreAndEnqueue(surfaceId, key, bytes, token);
        }
        catch
        {
            MarkFailed(surfaceId, key, token);
        }
        finally
        {
            if (acquired)
                RemoteFetchGate.Release();
        }
    }

    private static async Task<byte[]> ReadBoundedBytesAsync(HttpContent content, int maxBytes)
    {
        using Stream stream = await content.ReadAsStreamAsync().ConfigureAwait(false);
        using var ms = new MemoryStream();
        byte[] buffer = new byte[16384];
        int total = 0;
        while (true)
        {
            int read = await stream.ReadAsync(buffer, 0, buffer.Length).ConfigureAwait(false);
            if (read <= 0)
                break;
            total += read;
            if (total > maxBytes)
                return null;
            ms.Write(buffer, 0, read);
        }
        return ms.ToArray();
    }

    private void StoreAndEnqueue(uint surfaceId, string key, byte[] bytes, int token)
    {
        var uploadId = (surfaceId, key);
        lock (_cacheLock)
        {
            if (_disposed)
                return;
            if (key == "local_pfp" && token != _localPfpToken)
            {
                _inFlight.Remove(uploadId);
                return;
            }

            _inFlight.Remove(uploadId);
            if (key != "local_pfp")
            {
                _bytesOrder.Remove(key);
                _bytesOrder.AddLast(key);
            }
            _bytesCache[key] = bytes;

            while (_bytesOrder.Count > MaxSurfaceTextures)
            {
                string oldest = _bytesOrder.First.Value;
                _bytesOrder.RemoveFirst();
                _bytesCache.Remove(oldest);
            }

            if (_queuedUploads.Add(uploadId))
            {
                _pendingUploads.Enqueue((surfaceId, key, bytes, token));
            }
        }
    }

    private void MarkFailed(uint surfaceId, string key, int token = 0)
    {
        var uploadId = (surfaceId, key);
        lock (_cacheLock)
        {
            _inFlight.Remove(uploadId);
            if (key == "local_pfp" && token != _localPfpToken)
                return;
            if (_failedKeys.Count >= MaxFailedEntries)
                _failedKeys.Clear();
            _failedKeys[key] = DateTime.UtcNow;
        }
    }

    /// <summary>
    /// Must be called on Unity's main thread. Uploads up to maxPerFrame decoded images to Rust.
    /// </summary>
    public bool DrainUploads(NativeBridge bridge, out bool uploadedMain, out bool uploadedQuick, int maxPerFrame = 4)
    {
        uploadedMain = false;
        uploadedQuick = false;
        if (_disposed || bridge == null || !bridge.IsHealthy)
            return false;

        bool uploadedAny = false;
        int count = 0;
        while (count < maxPerFrame && _pendingUploads.TryDequeue(out var item))
        {
            int curPfpToken;
            lock (_cacheLock)
            {
                _queuedUploads.Remove((item.SurfaceId, item.Key));
                curPfpToken = _localPfpToken;
            }

            if (item.Key == "local_pfp" && item.Token != curPfpToken)
                continue;

            if (bridge.UploadImageBytes(item.SurfaceId, item.Key, item.Bytes))
            {
                RecordUploadedKey(item.SurfaceId, item.Key);
                uploadedAny = true;
                if (item.SurfaceId == NativeBridge.SurfaceMain)
                    uploadedMain = true;
                else if (item.SurfaceId == NativeBridge.SurfaceQuick)
                    uploadedQuick = true;
            }
            else
            {
                MelonLogger.Warning($"[LumenicMenu] Failed to upload image '{item.Key}': {bridge.GetLastError()}");
                MarkFailed(item.SurfaceId, item.Key, item.Token);
            }
            count++;
        }
        return uploadedAny;
    }

    private bool IsUploaded(uint surfaceId, string key)
    {
        return _uploadedPerSurface.TryGetValue(surfaceId, out var set) && set.Contains(key);
    }

    private void RecordUploadedKey(uint surfaceId, string key)
    {
        if (!_uploadedPerSurface.TryGetValue(surfaceId, out var set))
        {
            set = new HashSet<string>(StringComparer.Ordinal);
            _uploadedPerSurface[surfaceId] = set;
        }
        if (!_uploadedOrderPerSurface.TryGetValue(surfaceId, out var order))
        {
            order = new LinkedList<string>();
            _uploadedOrderPerSurface[surfaceId] = order;
        }

        set.Add(key);
        if (key != "local_pfp")
        {
            order.Remove(key);
            order.AddLast(key);
            while (order.Count > MaxSurfaceTextures)
            {
                string oldest = order.First.Value;
                order.RemoveFirst();
                set.Remove(oldest);
            }
        }
    }

    private void RemoveUploadedKey(uint surfaceId, string key)
    {
        if (_uploadedPerSurface.TryGetValue(surfaceId, out var set))
            set.Remove(key);
        if (_uploadedOrderPerSurface.TryGetValue(surfaceId, out var order))
            order.Remove(key);
    }

    public void Dispose()
    {
        _disposed = true;
        lock (_cacheLock)
        {
            while (_pendingUploads.TryDequeue(out _)) { }
            _bytesCache.Clear();
            _bytesOrder.Clear();
            _inFlight.Clear();
            _failedKeys.Clear();
            _queuedUploads.Clear();
        }
        _uploadedPerSurface.Clear();
        _uploadedOrderPerSurface.Clear();
    }
}
