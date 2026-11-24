using System;
using System.Collections.Generic;
using System.IO;
using System.Linq;
using System.Threading;
using System.Threading.Tasks;
using ABI_RC.Core.Networking;
using ABI_RC.Core.Networking.API;
using ABI_RC.Core.Networking.API.Responses;
using ABI_RC.Core.Networking.API.Responses.DetailsV2;
using ABI_RC.Core.Networking.GameServer;
using ABI_RC.Core.Networking.Garrison;
using ABI_RC.Core.Networking.IO.Instancing;
using ABI_RC.Core.Player;
using ABI_RC.Core.PropManagement;
using ABI_RC.Core.Savior;
using LumenicMenu.Native;
using MelonLoader;
using MelonLoader.Utils;
using Newtonsoft.Json;

namespace LumenicMenu.Bridge;

/// <summary>
/// Tracks and persists client-side recent history (capped at 100 entries per list):
/// - Worlds: Recently Visited
/// - Avatars: Recently Used, Recently Seen
/// - Props: Recently Spawned (by local user), Recently Seen (spawned by other players)
/// - Users: Recently Seen (with timestamp, world, instance name, privacy type, rank, and PFP)
/// </summary>
internal sealed class RecentHistoryService
{
    public const int MaxEntries = 100;

    public const string KeyWorldsVisited = "local_recent_visited";
    public const string KeyAvatarsUsed = "local_recent_used";
    public const string KeyAvatarsSeen = "local_recent_seen";
    public const string KeyPropsSpawned = "local_recent_spawned";
    public const string KeyPropsSeen = "local_recent_props_seen";

    public sealed class RecentContentEntry
    {
        public string Id = string.Empty;
        public string Name = string.Empty;
        public string ImageUrl = string.Empty;
        public long LastSeenUtc;
    }

    public sealed class RecentUserEntry
    {
        public string Id = string.Empty;
        public string Name = string.Empty;
        public string ImageUrl = string.Empty;
        public string Rank = string.Empty;
        public string WorldName = string.Empty;
        public string InstanceName = string.Empty;
        public string InstancePrivacy = string.Empty;
        public long LastSeenUtc;
    }

    private sealed class PersistedHistory
    {
        public int Version = 1;
        public List<RecentContentEntry> WorldsVisited = new List<RecentContentEntry>();
        public List<RecentContentEntry> AvatarsUsed = new List<RecentContentEntry>();
        public List<RecentContentEntry> AvatarsSeen = new List<RecentContentEntry>();
        public List<RecentContentEntry> PropsSpawned = new List<RecentContentEntry>();
        public List<RecentContentEntry> PropsSeen = new List<RecentContentEntry>();
        public List<RecentUserEntry> UsersSeen = new List<RecentUserEntry>();
    }

    private readonly Action _onDirty;
    private readonly object _lock = new object();
    private readonly object _fileLock = new object();
    private readonly string _filePath;
    private readonly HashSet<string> _attemptedHydrations = new HashSet<string>(StringComparer.OrdinalIgnoreCase);

    private PersistedHistory _data = new PersistedHistory();
    private bool _saveScheduled;
    private volatile bool _dirtyDisk;
    private int _hydratingFlag;

    public RecentHistoryService(Action onDirty)
    {
        _onDirty = onDirty;
        string baseDir = string.IsNullOrEmpty(MelonEnvironment.UserDataDirectory)
            ? "UserData"
            : MelonEnvironment.UserDataDirectory;
        _filePath = Path.Combine(baseDir, "LumenicMenu", "recent-history.json");
        LoadFromDisk();
    }

    public static bool IsLocalRecentCategory(string categoryKey)
    {
        return categoryKey == KeyWorldsVisited
            || categoryKey == KeyAvatarsUsed
            || categoryKey == KeyAvatarsSeen
            || categoryKey == KeyPropsSpawned
            || categoryKey == KeyPropsSeen;
    }

    public void ClearAll()
    {
        lock (_lock)
        {
            _data = new PersistedHistory();
            _attemptedHydrations.Clear();
            _dirtyDisk = false;
        }
        try
        {
            lock (_fileLock)
            {
                if (File.Exists(_filePath))
                    File.Delete(_filePath);
            }
        }
        catch (Exception ex)
        {
            MelonLogger.Warning($"[LumenicMenu] Failed to delete recent-history.json: {ex.Message}");
        }
        _onDirty?.Invoke();
    }

    public void RecordWorldVisited(string worldId, string worldName = null)
    {
        if (!Settings.EnableRecentHistory || string.IsNullOrWhiteSpace(worldId))
            return;

        string name = !string.IsNullOrWhiteSpace(worldName)
            ? worldName.Trim()
            : (MetaPort.Instance?.CurrentWorldName?.Trim() ?? string.Empty);

        bool needsHydrate;
        lock (_lock)
        {
            RecentContentEntry entry = UpsertContent(_data.WorldsVisited, worldId.Trim(), name, null);
            needsHydrate = string.IsNullOrWhiteSpace(entry.ImageUrl) || string.IsNullOrWhiteSpace(entry.Name);
        }

        ScheduleSave();
        _onDirty?.Invoke();

        if (needsHydrate)
        {
            Task.Run(() => HydrateSingleAsync("world", worldId.Trim()));
        }
    }

    public void RecordAvatarUsed(string avatarId, string avatarName = null)
    {
        if (!Settings.EnableRecentHistory || string.IsNullOrWhiteSpace(avatarId))
            return;

        bool needsHydrate;
        lock (_lock)
        {
            RecentContentEntry entry = UpsertContent(_data.AvatarsUsed, avatarId.Trim(), avatarName, null);
            needsHydrate = string.IsNullOrWhiteSpace(entry.ImageUrl) || string.IsNullOrWhiteSpace(entry.Name);
        }

        ScheduleSave();
        _onDirty?.Invoke();

        if (needsHydrate)
        {
            Task.Run(() => HydrateSingleAsync("avatar", avatarId.Trim()));
        }
    }

    public void RecordAvatarSeen(string avatarId, string avatarName = null)
    {
        if (!Settings.EnableRecentHistory || string.IsNullOrWhiteSpace(avatarId))
            return;

        string myAvatarId = MetaPort.Instance?.currentAvatarGuid;
        if (!string.IsNullOrEmpty(myAvatarId) && string.Equals(avatarId, myAvatarId, StringComparison.OrdinalIgnoreCase))
            return;

        lock (_lock)
        {
            UpsertContent(_data.AvatarsSeen, avatarId.Trim(), avatarName, null);
        }

        ScheduleSave();
        _onDirty?.Invoke();
    }

    public void RecordPropSpawned(PropSoul prop)
    {
        if (!Settings.EnableRecentHistory || prop == null || string.IsNullOrWhiteSpace(prop.ObjectId))
            return;

        string id = prop.ObjectId.Trim();
        string name = prop.DisplayName;
        if (string.IsNullOrWhiteSpace(name) || name == PropSoul.MatureHiddenName)
            name = prop.ContentMetadata.AssetName ?? string.Empty;

        if (prop.IsSpawnedByMe)
        {
            bool needsHydrate;
            lock (_lock)
            {
                RecentContentEntry entry = UpsertContent(_data.PropsSpawned, id, name, null);
                needsHydrate = string.IsNullOrWhiteSpace(entry.ImageUrl) || string.IsNullOrWhiteSpace(entry.Name);
            }

            ScheduleSave();
            _onDirty?.Invoke();

            if (needsHydrate)
            {
                Task.Run(() => HydrateSingleAsync("prop", id));
            }
        }
        else if (!prop.IsSpawnedBySystem)
        {
            lock (_lock)
            {
                UpsertContent(_data.PropsSeen, id, name, null);
            }

            ScheduleSave();
            _onDirty?.Invoke();
        }
    }

    public void RecordUserSeen(CVRPlayerEntity player)
    {
        if (!Settings.EnableRecentHistory || player == null || string.IsNullOrWhiteSpace(player.Uuid))
            return;

        string myId = MetaPort.Instance?.ownerId ?? AuthManager.UserId;
        if (!string.IsNullOrEmpty(myId) && string.Equals(player.Uuid, myId, StringComparison.OrdinalIgnoreCase))
            return;

        ReadCurrentInstanceContext(out string worldName, out string instanceName, out string privacy);
        string id = player.Uuid.Trim();
        string name = (player.Username ?? string.Empty).Trim();
        string imageUrl = (player.ApiProfileImageUrl ?? string.Empty).Trim();
        string rank = (player.ApiUserRank ?? string.Empty).Trim();

        lock (_lock)
        {
            UpsertUser(id, name, imageUrl, rank, worldName, instanceName, privacy);
        }

        ScheduleSave();
        _onDirty?.Invoke();
    }

    public void RefreshCurrentInstanceUsers()
    {
        try
        {
            var players = CVRPlayerManager.Instance?.NetworkPlayers;
            if (players == null || players.Count == 0)
                return;

            ReadCurrentInstanceContext(out string worldName, out string instanceName, out string privacy);
            string myId = MetaPort.Instance?.ownerId ?? AuthManager.UserId;
            bool changed = false;

            lock (_lock)
            {
                foreach (CVRPlayerEntity p in players)
                {
                    if (p == null || string.IsNullOrWhiteSpace(p.Uuid))
                        continue;
                    if (!string.IsNullOrEmpty(myId) && string.Equals(p.Uuid, myId, StringComparison.OrdinalIgnoreCase))
                        continue;

                    UpsertUser(
                        p.Uuid.Trim(),
                        (p.Username ?? string.Empty).Trim(),
                        (p.ApiProfileImageUrl ?? string.Empty).Trim(),
                        (p.ApiUserRank ?? string.Empty).Trim(),
                        worldName,
                        instanceName,
                        privacy);
                    changed = true;
                }
            }

            if (changed)
            {
                ScheduleSave();
                _onDirty?.Invoke();
            }
        }
        catch
        {
            // Ignore transient CVRPlayerManager mutations.
        }
    }

    /// <summary>
    /// Updates cached Name/ImageUrl across matching recent lists whenever API cards or details load.
    /// </summary>
    public void UpdateKnownMetadata(string kind, string id, string name, string imageUrl)
    {
        if (string.IsNullOrWhiteSpace(id))
            return;

        bool changed = false;
        lock (_lock)
        {
            if (kind == "world" || kind == "worlds")
            {
                changed |= PatchMetadata(_data.WorldsVisited, id, name, imageUrl);
            }
            else if (kind == "avatar" || kind == "avatars")
            {
                changed |= PatchMetadata(_data.AvatarsUsed, id, name, imageUrl);
                changed |= PatchMetadata(_data.AvatarsSeen, id, name, imageUrl);
            }
            else if (kind == "prop" || kind == "props")
            {
                changed |= PatchMetadata(_data.PropsSpawned, id, name, imageUrl);
                changed |= PatchMetadata(_data.PropsSeen, id, name, imageUrl);
            }
        }

        if (changed)
        {
            ScheduleSave();
        }
    }

    public List<RecentContentEntry> GetContentEntries(string categoryKey)
    {
        lock (_lock)
        {
            List<RecentContentEntry> src = GetListForCategoryLocked(categoryKey);
            if (src == null)
                return new List<RecentContentEntry>();

            return src.Select(e => new RecentContentEntry
            {
                Id = e.Id,
                Name = string.IsNullOrWhiteSpace(e.Name) ? ShortIdFallback(e.Id) : e.Name,
                ImageUrl = e.ImageUrl ?? string.Empty,
                LastSeenUtc = e.LastSeenUtc
            }).ToList();
        }
    }

    public List<object> BuildRecentUsersSnapshot(ImageLoaderService images)
    {
        List<RecentUserEntry> copy;
        lock (_lock)
        {
            copy = _data.UsersSeen.Select(u => new RecentUserEntry
            {
                Id = u.Id,
                Name = u.Name,
                ImageUrl = u.ImageUrl,
                Rank = u.Rank,
                WorldName = u.WorldName,
                InstanceName = u.InstanceName,
                InstancePrivacy = u.InstancePrivacy,
                LastSeenUtc = u.LastSeenUtc
            }).ToList();
        }

        long nowUnix = DateTimeOffset.UtcNow.ToUnixTimeSeconds();
        var list = new List<object>(copy.Count);
        foreach (RecentUserEntry u in copy)
        {
            list.Add(new
            {
                id = u.Id,
                name = string.IsNullOrWhiteSpace(u.Name) ? "Player" : u.Name,
                image_key = images != null ? images.RequestThumbnail(NativeBridge.SurfaceMain, u.ImageUrl) : string.Empty,
                rank = u.Rank ?? string.Empty,
                world_name = u.WorldName ?? string.Empty,
                instance_name = u.InstanceName ?? string.Empty,
                instance_privacy = u.InstancePrivacy ?? string.Empty,
                last_seen_unix = u.LastSeenUtc,
                last_seen_text = FormatRelativeTimestamp(u.LastSeenUtc, nowUnix)
            });
        }
        return list;
    }

    /// <summary>
    /// Lazily resolves missing thumbnails/names for entries in the currently opened recent category.
    /// Throttled to avoid spamming CVR's API.
    /// </summary>
    public void RequestHydrateCategory(string categoryKey, Action onItemHydrated = null)
    {
        if (!AuthManager.IsAuthenticated || !IsLocalRecentCategory(categoryKey))
            return;

        if (Interlocked.CompareExchange(ref _hydratingFlag, 1, 0) != 0)
            return;

        Task.Run(async () =>
        {
            try
            {
                string detailKind = categoryKey switch
                {
                    KeyWorldsVisited => "world",
                    KeyAvatarsUsed or KeyAvatarsSeen => "avatar",
                    _ => "prop"
                };

                List<string> missingIds;
                lock (_lock)
                {
                    List<RecentContentEntry> list = GetListForCategoryLocked(categoryKey);
                    missingIds = list == null
                        ? new List<string>()
                        : list
                            .Where(e => (string.IsNullOrWhiteSpace(e.ImageUrl) || string.IsNullOrWhiteSpace(e.Name))
                                        && !_attemptedHydrations.Contains($"{detailKind}:{e.Id}"))
                            .Take(24)
                            .Select(e => e.Id)
                            .ToList();
                }

                int resolved = 0;
                foreach (string id in missingIds)
                {
                    bool ok = await HydrateSingleAsync(detailKind, id).ConfigureAwait(false);
                    if (ok)
                    {
                        resolved++;
                        if (resolved % 3 == 0)
                        {
                            onItemHydrated?.Invoke();
                            _onDirty?.Invoke();
                        }
                    }
                    await Task.Delay(140).ConfigureAwait(false);
                }

                if (resolved > 0)
                {
                    onItemHydrated?.Invoke();
                    _onDirty?.Invoke();
                }
            }
            catch (Exception ex)
            {
                MelonLogger.Warning($"[LumenicMenu] RecentHistory hydrate failed: {ex.Message}");
            }
            finally
            {
                Interlocked.Exchange(ref _hydratingFlag, 0);
            }
        });
    }

    public void FlushSave()
    {
        if (!_dirtyDisk)
            return;
        SaveToDiskSync();
    }

    private async Task<bool> HydrateSingleAsync(string detailKind, string id)
    {
        if (!AuthManager.IsAuthenticated || string.IsNullOrWhiteSpace(id))
            return false;

        string attemptKey = $"{detailKind}:{id}";
        lock (_lock)
        {
            if (!_attemptedHydrations.Add(attemptKey))
                return false;
        }

        try
        {
            string name = null;
            string imageUrl = null;

            if (detailKind == "world")
            {
                BaseResponse<ContentWorldResponse> resp =
                    await ApiConnection.MakeRequest<ContentWorldResponse>(
                        ApiConnection.ApiOperation.WorldDetail,
                        new { worldID = id },
                        "2").ConfigureAwait(false);
                if (resp?.Data != null)
                {
                    name = resp.Data.Name;
                    imageUrl = resp.Data.Image?.ToString();
                }
            }
            else if (detailKind == "avatar")
            {
                BaseResponse<ContentAvatarResponse> resp =
                    await ApiConnection.MakeRequest<ContentAvatarResponse>(
                        ApiConnection.ApiOperation.AvatarDetail,
                        new { avatarID = id },
                        "2").ConfigureAwait(false);
                if (resp?.Data != null)
                {
                    name = resp.Data.Name;
                    imageUrl = resp.Data.Image?.ToString();
                }
            }
            else if (detailKind == "prop")
            {
                BaseResponse<ContentSpawnableResponse> resp =
                    await ApiConnection.MakeRequest<ContentSpawnableResponse>(
                        ApiConnection.ApiOperation.PropDetail,
                        new { id },
                        "2").ConfigureAwait(false);
                if (resp?.Data != null)
                {
                    name = resp.Data.Name;
                    imageUrl = resp.Data.Image?.ToString();
                }
            }

            if (!string.IsNullOrWhiteSpace(name) || !string.IsNullOrWhiteSpace(imageUrl))
            {
                UpdateKnownMetadata(detailKind, id, name, imageUrl);
                _onDirty?.Invoke();
                return true;
            }
        }
        catch
        {
            // Private/deleted content or transient API errors are ignored cleanly.
        }
        return false;
    }

    private List<RecentContentEntry> GetListForCategoryLocked(string categoryKey)
    {
        return categoryKey switch
        {
            KeyWorldsVisited => _data.WorldsVisited,
            KeyAvatarsUsed => _data.AvatarsUsed,
            KeyAvatarsSeen => _data.AvatarsSeen,
            KeyPropsSpawned => _data.PropsSpawned,
            KeyPropsSeen => _data.PropsSeen,
            _ => null
        };
    }

    private static RecentContentEntry UpsertContent(
        List<RecentContentEntry> list,
        string id,
        string name,
        string imageUrl)
    {
        long now = DateTimeOffset.UtcNow.ToUnixTimeSeconds();
        int existingIdx = list.FindIndex(e => string.Equals(e.Id, id, StringComparison.OrdinalIgnoreCase));
        RecentContentEntry entry;
        if (existingIdx >= 0)
        {
            entry = list[existingIdx];
            list.RemoveAt(existingIdx);
        }
        else
        {
            entry = new RecentContentEntry { Id = id };
        }

        if (!string.IsNullOrWhiteSpace(name))
            entry.Name = name.Trim();
        if (!string.IsNullOrWhiteSpace(imageUrl))
            entry.ImageUrl = imageUrl.Trim();
        entry.LastSeenUtc = now;

        list.Insert(0, entry);
        if (list.Count > MaxEntries)
            list.RemoveRange(MaxEntries, list.Count - MaxEntries);

        return entry;
    }

    private void UpsertUser(
        string id,
        string name,
        string imageUrl,
        string rank,
        string worldName,
        string instanceName,
        string instancePrivacy)
    {
        long now = DateTimeOffset.UtcNow.ToUnixTimeSeconds();
        List<RecentUserEntry> list = _data.UsersSeen;
        int existingIdx = list.FindIndex(u => string.Equals(u.Id, id, StringComparison.OrdinalIgnoreCase));
        RecentUserEntry entry;
        if (existingIdx >= 0)
        {
            entry = list[existingIdx];
            list.RemoveAt(existingIdx);
        }
        else
        {
            entry = new RecentUserEntry { Id = id };
        }

        if (!string.IsNullOrWhiteSpace(name))
            entry.Name = name;
        if (!string.IsNullOrWhiteSpace(imageUrl))
            entry.ImageUrl = imageUrl;
        if (!string.IsNullOrWhiteSpace(rank))
            entry.Rank = rank;
        if (!string.IsNullOrWhiteSpace(worldName))
            entry.WorldName = worldName;
        if (!string.IsNullOrWhiteSpace(instanceName))
            entry.InstanceName = instanceName;
        if (!string.IsNullOrWhiteSpace(instancePrivacy))
            entry.InstancePrivacy = instancePrivacy;
        entry.LastSeenUtc = now;

        list.Insert(0, entry);
        if (list.Count > MaxEntries)
            list.RemoveRange(MaxEntries, list.Count - MaxEntries);
    }

    private static bool PatchMetadata(List<RecentContentEntry> list, string id, string name, string imageUrl)
    {
        bool changed = false;
        for (int i = 0; i < list.Count; i++)
        {
            RecentContentEntry e = list[i];
            if (!string.Equals(e.Id, id, StringComparison.OrdinalIgnoreCase))
                continue;
            if (!string.IsNullOrWhiteSpace(name) && e.Name != name.Trim())
            {
                e.Name = name.Trim();
                changed = true;
            }
            if (!string.IsNullOrWhiteSpace(imageUrl) && e.ImageUrl != imageUrl.Trim())
            {
                e.ImageUrl = imageUrl.Trim();
                changed = true;
            }
        }
        return changed;
    }

    private static void ReadCurrentInstanceContext(
        out string worldName,
        out string instanceName,
        out string privacy)
    {
        worldName = string.Empty;
        instanceName = string.Empty;
        privacy = string.Empty;
        try
        {
            worldName = MetaPort.Instance?.CurrentWorldName?.Trim() ?? string.Empty;
            instanceName = Instances.CurrentInstanceName?.Trim() ?? string.Empty;
            privacy = Instances.GetPrettyInstanceName(Instances.CurrentInstancePrivacyType) ?? string.Empty;
            if (GarrisonConnection.Instance != null
                && GarrisonConnection.Instance.IsConnected()
                && GSInfoHandler.LastGSInfoUpdate.HasValue)
            {
                var gs = GSInfoHandler.LastGSInfoUpdate.Value;
                if (!string.IsNullOrWhiteSpace(gs.InstanceName))
                    instanceName = gs.InstanceName.Trim();
                string prettyPriv = Instances.GetPrettyInstanceName(gs.Privacy.ToInstancePrivacyType());
                if (!string.IsNullOrWhiteSpace(prettyPriv))
                    privacy = prettyPriv;
            }
        }
        catch
        {
            // Ignore when instance context is not yet initialized.
        }
    }

    private static string FormatRelativeTimestamp(long unixSeconds, long nowUnix)
    {
        if (unixSeconds <= 0)
            return string.Empty;

        long diff = Math.Max(0, nowUnix - unixSeconds);
        DateTime localTime = DateTimeOffset.FromUnixTimeSeconds(unixSeconds).LocalDateTime;
        if (diff < 60)
            return "Just now";
        if (diff < 3600)
            return $"{diff / 60}m ago";
        if (diff < 86400)
            return $"{diff / 3600}h ago · {localTime:HH:mm}";
        if (diff < 172800)
            return $"Yesterday · {localTime:HH:mm}";
        return localTime.ToString("MMM d · HH:mm");
    }

    private static string ShortIdFallback(string id)
    {
        if (string.IsNullOrEmpty(id))
            return "Unknown";
        return id.Length > 12 ? id.Substring(0, 12) + "..." : id;
    }

    private void LoadFromDisk()
    {
        try
        {
            if (!File.Exists(_filePath))
                return;

            string json = File.ReadAllText(_filePath);
            PersistedHistory loaded = JsonConvert.DeserializeObject<PersistedHistory>(json);
            if (loaded != null)
            {
                loaded.WorldsVisited ??= new List<RecentContentEntry>();
                loaded.AvatarsUsed ??= new List<RecentContentEntry>();
                loaded.AvatarsSeen ??= new List<RecentContentEntry>();
                loaded.PropsSpawned ??= new List<RecentContentEntry>();
                loaded.PropsSeen ??= new List<RecentContentEntry>();
                loaded.UsersSeen ??= new List<RecentUserEntry>();
                TrimToCap(loaded.WorldsVisited);
                TrimToCap(loaded.AvatarsUsed);
                TrimToCap(loaded.AvatarsSeen);
                TrimToCap(loaded.PropsSpawned);
                TrimToCap(loaded.PropsSeen);
                TrimToCap(loaded.UsersSeen);
                lock (_lock)
                {
                    _data = loaded;
                }
            }
        }
        catch (Exception ex)
        {
            MelonLogger.Warning($"[LumenicMenu] Failed to load recent history: {ex.Message}");
        }
    }

    private static void TrimToCap<T>(List<T> list)
    {
        if (list.Count > MaxEntries)
            list.RemoveRange(MaxEntries, list.Count - MaxEntries);
    }

    private void ScheduleSave()
    {
        _dirtyDisk = true;
        lock (_lock)
        {
            if (_saveScheduled)
                return;
            _saveScheduled = true;
        }

        Task.Run(async () =>
        {
            try
            {
                await Task.Delay(1500).ConfigureAwait(false);
                SaveToDiskSync();
            }
            finally
            {
                lock (_lock)
                {
                    _saveScheduled = false;
                }
                if (_dirtyDisk)
                {
                    ScheduleSave();
                }
            }
        });
    }

    private void SaveToDiskSync()
    {
        string json;
        lock (_lock)
        {
            _dirtyDisk = false;
            json = JsonConvert.SerializeObject(_data, Formatting.Indented);
        }

        lock (_fileLock)
        {
            try
            {
                string dir = Path.GetDirectoryName(_filePath);
                if (!string.IsNullOrEmpty(dir) && !Directory.Exists(dir))
                    Directory.CreateDirectory(dir);

                string tmp = _filePath + ".tmp";
                File.WriteAllText(tmp, json);
                if (File.Exists(_filePath))
                    File.Delete(_filePath);
                File.Move(tmp, _filePath);
            }
            catch (Exception ex)
            {
                MelonLogger.Warning($"[LumenicMenu] Failed to save recent history: {ex.Message}");
            }
        }
    }
}
