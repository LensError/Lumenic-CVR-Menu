using System;
using System.Net.Http;
using System.Threading.Tasks;
using MelonLoader;
using Newtonsoft.Json.Linq;

namespace LumenicMenu.Bridge;

internal sealed class UpdateStatusDto
{
    public string status = "idle";
    public string current_version = string.Empty;
    public string latest_version = string.Empty;
    public string release_title = string.Empty;
    public string release_notes = string.Empty;
    public string release_url = string.Empty;
    public string download_url = string.Empty;
    public string error_message = string.Empty;
    public bool show_modal;
}

internal sealed class ModUpdateService
{
    public const string BundledChangelog =
        "What's New in LumenicMenu v" + LumenicMenuMod.ModVersion + ":\n\n" +
        "- Smoother Menus: reworked how menus draw and update behind the scenes so everything feels snappier with less overhead.\n" +
        "- No More Image Stutters: profile pictures, thumbnails, and badges now load in the background without freezing or hitching while scrolling.\n" +
        "- Friendlier Group Pages: group pages now open on an Overview with the description, Owner, Admins & Moderators, and your membership status, with management tools in a separate Manage tab.\n" +
        "- Quick Menu Tweaks & Fixes: scaled Quick Menu down slightly so it doesn't cut off at max UI scale, updated the Quick Menu Player Details card to match the Big Menu, made user group cards clickable as a whole card, and fixed Back navigation from group pages.";

    private const string LatestReleaseApiUrl = "https://api.github.com/repos/LensError/Lumenic-CVR-Menu/releases/latest";
    private const string DefaultReleasePageUrl = "https://github.com/LensError/Lumenic-CVR-Menu/releases/latest";
    private const string DefaultDownloadUrl = "https://github.com/LensError/Lumenic-CVR-Menu/releases/latest/download/LumenicMenu.dll";

    private static readonly HttpClient Http = new()
    {
        Timeout = TimeSpan.FromSeconds(10),
    };

    private readonly object _lock = new();
    private readonly Action _markDirty;
    private readonly string _currentVersion;

    private string _status = "idle";
    private string _latestVersion = string.Empty;
    private string _releaseTitle = string.Empty;
    private string _releaseNotes = string.Empty;
    private string _releaseUrl = DefaultReleasePageUrl;
    private string _downloadUrl = DefaultDownloadUrl;
    private string _errorMessage = string.Empty;
    private bool _sessionDismissed;
    private bool _checkInFlight;

    static ModUpdateService()
    {
        Http.DefaultRequestHeaders.UserAgent.ParseAdd($"LumenicMenu/{LumenicMenuMod.ModVersion}");
        Http.DefaultRequestHeaders.Accept.ParseAdd("application/vnd.github+json");
    }

    public ModUpdateService(string currentVersion, Action markDirty)
    {
        _currentVersion = NormalizeVersion(currentVersion);
        _markDirty = markDirty;
    }

    public void CheckForUpdates(bool manual)
    {
        lock (_lock)
        {
            if (_checkInFlight)
                return;

            _checkInFlight = true;
            _status = "checking";
            _errorMessage = string.Empty;
            if (manual)
                _sessionDismissed = false;
        }
        _markDirty?.Invoke();

        Task.Run(() => RunCheckAsync(manual));
    }

    public void DismissForSession()
    {
        lock (_lock)
        {
            _sessionDismissed = true;
        }
        _markDirty?.Invoke();
    }

    public UpdateStatusDto BuildSnapshot()
    {
        lock (_lock)
        {
            string dismissed = NormalizeVersion(Settings.DismissedUpdateVersion);
            bool updateAvailable = string.Equals(_status, "update_available", StringComparison.Ordinal);
            bool showModal = updateAvailable
                && !_sessionDismissed
                && !string.IsNullOrEmpty(_latestVersion)
                && !string.Equals(_latestVersion, dismissed, StringComparison.OrdinalIgnoreCase);

            return new UpdateStatusDto
            {
                status = _status,
                current_version = _currentVersion,
                latest_version = _latestVersion,
                release_title = _releaseTitle,
                release_notes = _releaseNotes,
                release_url = _releaseUrl,
                download_url = _downloadUrl,
                error_message = _errorMessage,
                show_modal = showModal,
            };
        }
    }

    private async Task RunCheckAsync(bool manual)
    {
        try
        {
            using HttpResponseMessage resp = await Http.GetAsync(LatestReleaseApiUrl).ConfigureAwait(false);
            if (!resp.IsSuccessStatusCode)
            {
                SetError($"GitHub API returned HTTP {(int)resp.StatusCode}.");
                return;
            }

            string json = await resp.Content.ReadAsStringAsync().ConfigureAwait(false);
            JObject root = JObject.Parse(json);

            string tag = root.Value<string>("tag_name") ?? string.Empty;
            string latest = NormalizeVersion(tag);
            if (string.IsNullOrEmpty(latest))
            {
                SetError("Latest release tag is empty.");
                return;
            }

            string title = (root.Value<string>("name") ?? tag).Trim();
            string notes = (root.Value<string>("body") ?? string.Empty).Trim();
            if (notes.Length > 1800)
                notes = notes.Substring(0, 1797) + "...";

            string htmlUrl = root.Value<string>("html_url") ?? DefaultReleasePageUrl;
            string dllUrl = DefaultDownloadUrl;
            if (root["assets"] is JArray assets)
            {
                foreach (JToken token in assets)
                {
                    string assetName = token.Value<string>("name") ?? string.Empty;
                    string browserUrl = token.Value<string>("browser_download_url") ?? string.Empty;
                    if (assetName.EndsWith(".dll", StringComparison.OrdinalIgnoreCase) && !string.IsNullOrEmpty(browserUrl))
                    {
                        dllUrl = browserUrl;
                        break;
                    }
                }
            }

            bool isNewer = IsRemoteNewer(_currentVersion, latest);
            lock (_lock)
            {
                _latestVersion = latest;
                _releaseTitle = title;
                _releaseNotes = notes;
                _releaseUrl = htmlUrl;
                _downloadUrl = dllUrl;
                _errorMessage = string.Empty;
                _status = isNewer ? "update_available" : "up_to_date";
                if (manual && isNewer)
                {
                    _sessionDismissed = false;
                    if (string.Equals(NormalizeVersion(Settings.DismissedUpdateVersion), latest, StringComparison.OrdinalIgnoreCase))
                        Settings.SetDismissedUpdateVersion(string.Empty);
                }
                _checkInFlight = false;
            }

            if (isNewer)
                MelonLogger.Msg($"[LumenicMenu] Update available: v{latest} (current v{_currentVersion})");
        }
        catch (Exception ex)
        {
            SetError(ex.Message);
        }
        finally
        {
            _markDirty?.Invoke();
        }
    }

    private void SetError(string message)
    {
        lock (_lock)
        {
            _status = "error";
            _errorMessage = string.IsNullOrWhiteSpace(message) ? "Update check failed." : message;
            _checkInFlight = false;
        }
        MelonLogger.Warning($"[LumenicMenu] Update check failed: {_errorMessage}");
    }

    private static string NormalizeVersion(string raw)
    {
        if (string.IsNullOrWhiteSpace(raw))
            return string.Empty;

        string trimmed = raw.Trim();
        if (trimmed.StartsWith("v", StringComparison.OrdinalIgnoreCase))
            trimmed = trimmed.Substring(1);
        int dash = trimmed.IndexOf('-');
        if (dash >= 0)
            trimmed = trimmed.Substring(0, dash);
        return trimmed.Trim();
    }

    private static bool IsRemoteNewer(string current, string remote)
    {
        if (Version.TryParse(current, out Version curVer) && Version.TryParse(remote, out Version remVer))
            return remVer > curVer;
        return !string.Equals(current, remote, StringComparison.OrdinalIgnoreCase) && !string.IsNullOrEmpty(remote);
    }
}
