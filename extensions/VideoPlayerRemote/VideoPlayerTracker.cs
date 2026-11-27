using System;
using System.Collections.Generic;
using System.Linq;
using System.Reflection;
using ABI.CCK.Components;
using ABI_RC.Core.Networking;
using ABI_RC.Core.Player;
using ABI_RC.Core.Savior;
using ABI_RC.VideoPlayer.Scripts;
using ABI_RC.VideoPlayer.Scripts.Players;
using ABI_RC.VideoPlayer.Scripts.YouTubeDl;
using GameCommon.Models.VideoPlayer;
using UnityEngine;

namespace VideoPlayerRemote
{
    internal static class VideoPlayerTracker
    {
        private static readonly List<CVRVideoPlayer> _players = new List<CVRVideoPlayer>();
        private static readonly System.Random _rng = new System.Random();

        private static PropertyInfo _propAvailableLockOptions;
        private static PropertyInfo _propOurLockLevel;

        private static CVRVideoPlayer _selectedPlayer;
        private static bool _searchInFlight;
        private static float _searchStartedAt;

        // Debounced seek state
        private static CVRVideoPlayer _pendingSeekPlayer;
        private static double _pendingSeekTime;
        private static float _pendingSeekSendAt = -1f;

        // Custom playlist playback state
        private static SavedPlaylist _activePlaylist;
        private static int _activePlaylistIndex = -1;
        private static CVRVideoPlayer _activePlaylistPlayer;
        private static readonly Queue<string> _recentModUrls = new Queue<string>();

        public static IReadOnlyList<CVRVideoPlayer> Players => _players;
        public static CVRVideoPlayer SelectedPlayer => IsValid(_selectedPlayer) ? _selectedPlayer : null;
        public static SavedPlaylist ActivePlaylist => _activePlaylist;
        public static int ActivePlaylistIndex => _activePlaylistIndex;
        public static CVRVideoPlayer ActivePlaylistPlayer => IsValid(_activePlaylistPlayer) ? _activePlaylistPlayer : null;
        public static bool HasPendingSeek => _pendingSeekSendAt > 0f;
        public static bool IsSearchInFlight => _searchInFlight && (Time.unscaledTime - _searchStartedAt < 15f);

        public static event Action OnPlayersListChanged;
        public static event Action<CVRVideoPlayer> OnSelectedPlayerChanged;

        public static void Init()
        {
            _propAvailableLockOptions = typeof(CVRVideoPlayer).GetProperty(
                "AvailableLockOptions", BindingFlags.Instance | BindingFlags.NonPublic | BindingFlags.Public);
            _propOurLockLevel = typeof(CVRVideoPlayer).GetProperty(
                "OurLockLevel", BindingFlags.Instance | BindingFlags.NonPublic | BindingFlags.Public);

            GameEvents.OnVideoPlayerRegistered += HandleRegistered;
            GameEvents.OnVideoPlayerUnregistered += HandleUnregistered;
            GameEvents.OnVideoPlayersCleared += HandleCleared;
            GameEvents.OnVideoFinishedPlaying += HandleFinishedPlaying;
            GameEvents.OnVideoUrlChanged += HandleUrlChanged;
            GameEvents.OnVideoUrlCleared += HandleUrlCleared;
            GameEvents.OnWorldLeave += HandleCleared;

            RefreshFromScene();
        }

        public static void Shutdown()
        {
            GameEvents.OnVideoPlayerRegistered -= HandleRegistered;
            GameEvents.OnVideoPlayerUnregistered -= HandleUnregistered;
            GameEvents.OnVideoPlayersCleared -= HandleCleared;
            GameEvents.OnVideoFinishedPlaying -= HandleFinishedPlaying;
            GameEvents.OnVideoUrlChanged -= HandleUrlChanged;
            GameEvents.OnVideoUrlCleared -= HandleUrlCleared;
            GameEvents.OnWorldLeave -= HandleCleared;

            _players.Clear();
            _selectedPlayer = null;
            ClearPendingSeek();
            ClearCustomPlaylist();
            OnPlayersListChanged = null;
            OnSelectedPlayerChanged = null;
        }

        public static void Tick()
        {
            bool pruned = PruneDeadPlayers();
            if (!IsValid(_selectedPlayer) && _players.Count > 0 && Settings.AutoSelectNearest)
            {
                SelectNearest();
            }
            else if (pruned)
            {
                OnPlayersListChanged?.Invoke();
            }

            if (_pendingSeekSendAt > 0f && Time.unscaledTime >= _pendingSeekSendAt)
            {
                _pendingSeekSendAt = -1f;
                if (IsValid(_pendingSeekPlayer) && _pendingSeekPlayer.syncEnabled)
                {
                    _pendingSeekPlayer.SetVideoTimestampToNetwork(_pendingSeekTime);
                }
                _pendingSeekPlayer = null;
            }
        }

        public static void RefreshFromScene()
        {
            _players.Clear();
            try
            {
                var found = UnityEngine.Object.FindObjectsByType<CVRVideoPlayer>(FindObjectsInactive.Include, FindObjectsSortMode.None);
                foreach (var vp in found)
                {
                    if (vp != null && !_players.Contains(vp))
                        _players.Add(vp);
                }
            }
            catch (Exception ex)
            {
                VideoPlayerRemoteMod.Log?.Warning($"Failed scanning scene video players: {ex.Message}");
            }

            if (!IsValid(_selectedPlayer) && _players.Count > 0 && Settings.AutoSelectNearest)
            {
                SelectNearest();
            }

            OnPlayersListChanged?.Invoke();
        }

        public static void SelectPlayer(CVRVideoPlayer player)
        {
            if (player == _selectedPlayer)
                return;

            _selectedPlayer = IsValid(player) ? player : null;
            OnSelectedPlayerChanged?.Invoke(_selectedPlayer);
        }

        public static CVRVideoPlayer SelectNearest()
        {
            PruneDeadPlayers();
            if (_players.Count == 0)
            {
                SelectPlayer(null);
                return null;
            }

            CVRVideoPlayer nearest = _players
                .OrderBy(GetDistanceToLocalPlayer)
                .FirstOrDefault();

            SelectPlayer(nearest);
            return nearest;
        }

        public static List<CVRVideoPlayer> GetWorldPlayers()
        {
            PruneDeadPlayers();
            return _players.Where(vp => !IsPropPlayer(vp))
                .OrderBy(GetDistanceToLocalPlayer)
                .ToList();
        }

        public static List<CVRVideoPlayer> GetPropPlayers()
        {
            PruneDeadPlayers();
            return _players.Where(IsPropPlayer)
                .OrderBy(GetDistanceToLocalPlayer)
                .ToList();
        }

        public static bool IsValid(CVRVideoPlayer vp)
        {
            return vp != null && vp.gameObject != null;
        }

        public static bool IsPropPlayer(CVRVideoPlayer vp)
        {
            if (!IsValid(vp))
                return false;
            return vp.GetComponentInParent<CVRSpawnable>() != null;
        }

        public static float GetDistanceToLocalPlayer(CVRVideoPlayer vp)
        {
            if (!IsValid(vp))
                return float.MaxValue;

            Vector3 origin = Vector3.zero;
            if (PlayerSetup.Instance != null)
            {
                if (PlayerSetup.Instance.activeCam != null)
                    origin = PlayerSetup.Instance.activeCam.transform.position;
                else
                    origin = PlayerSetup.Instance.transform.position;
            }

            return Vector3.Distance(origin, vp.transform.position);
        }

        public static string GetShortName(CVRVideoPlayer vp)
        {
            if (!IsValid(vp))
                return "None";

            var spawnable = vp.GetComponentInParent<CVRSpawnable>();
            if (spawnable != null)
            {
                string propName = CleanObjectName(spawnable.gameObject.name);
                string owner = ResolveOwnerName(spawnable.ownerId);
                return string.IsNullOrEmpty(owner) ? propName : $"{propName} ({owner})";
            }

            string id = !string.IsNullOrWhiteSpace(vp.playerId) ? vp.playerId : vp.gameObject.name;
            if (id.Contains("/"))
            {
                string[] parts = id.Split('/');
                id = parts[parts.Length - 1];
            }
            return CleanObjectName(id);
        }

        public static string GetFullLabel(CVRVideoPlayer vp)
        {
            if (!IsValid(vp))
                return "No Video Player Selected";

            string kind = IsPropPlayer(vp) ? "[Prop]" : "[World]";
            string name = GetShortName(vp);
            float dist = GetDistanceToLocalPlayer(vp);
            string distText = dist < 999f ? $"{dist:F1}m" : "";
            return string.IsNullOrEmpty(distText) ? $"{kind} {name}" : $"{kind} {name} ({distText})";
        }

        public static string GetCurrentTitle(CVRVideoPlayer vp)
        {
            if (!IsValid(vp))
                return "Nothing Playing";

            try
            {
                string metaTitle = vp.VideoPlayer?.Info?.VideoMetaData?.GetTitle();
                if (!string.IsNullOrWhiteSpace(metaTitle))
                    return metaTitle;
            }
            catch { }

            if (vp.currentlySelectedVideo != null && !string.IsNullOrWhiteSpace(vp.currentlySelectedVideo.videoTitle))
                return vp.currentlySelectedVideo.videoTitle;

            if (!string.IsNullOrWhiteSpace(vp.lastNetworkVideoUrl))
                return vp.lastNetworkVideoUrl;

            return "Nothing Playing";
        }

        public static string GetCurrentUrl(CVRVideoPlayer vp)
        {
            if (!IsValid(vp))
                return null;

            if (!string.IsNullOrWhiteSpace(vp.lastNetworkVideoUrl))
                return vp.lastNetworkVideoUrl;

            if (vp.currentlySelectedVideo != null && !string.IsNullOrWhiteSpace(vp.currentlySelectedVideo.videoUrl))
                return vp.currentlySelectedVideo.videoUrl;

            try
            {
                return vp.VideoPlayer?.Info?.VideoMetaData?.GetUrl();
            }
            catch
            {
                return null;
            }
        }

        public static VideoPlayerUtils.PlayerState GetState(CVRVideoPlayer vp)
        {
            if (!IsValid(vp) || vp.VideoPlayer == null || vp.VideoPlayer.Info == null)
                return VideoPlayerUtils.PlayerState.NotPlaying;

            try
            {
                return vp.VideoPlayer.Info.GetState();
            }
            catch
            {
                return VideoPlayerUtils.PlayerState.NotPlaying;
            }
        }

        public static double GetCurrentTime(CVRVideoPlayer vp)
        {
            if (!IsValid(vp) || vp.VideoPlayer?.Info == null)
                return 0.0;
            try { return Math.Max(0.0, vp.VideoPlayer.Info.Time); }
            catch { return 0.0; }
        }

        public static double GetDuration(CVRVideoPlayer vp)
        {
            if (!IsValid(vp) || vp.VideoPlayer?.Info?.VideoMetaData == null)
                return 0.0;
            try { return Math.Max(0.0, vp.VideoPlayer.Info.VideoMetaData.GetDuration()); }
            catch { return 0.0; }
        }

        public static bool IsLivestream(CVRVideoPlayer vp)
        {
            if (!IsValid(vp) || vp.VideoPlayer?.Info?.VideoMetaData == null)
                return false;
            try { return vp.VideoPlayer.Info.VideoMetaData.IsLivestream; }
            catch { return false; }
        }

        public static VideoPlayerLockOption GetAvailableLockOptions(CVRVideoPlayer vp)
        {
            if (!IsValid(vp) || _propAvailableLockOptions == null)
                return VideoPlayerLockOption.Anyone;
            try
            {
                return (VideoPlayerLockOption)_propAvailableLockOptions.GetValue(vp);
            }
            catch
            {
                return VideoPlayerLockOption.Anyone;
            }
        }

        public static VideoPlayerLockOption GetOurLockLevel(CVRVideoPlayer vp)
        {
            if (!IsValid(vp) || _propOurLockLevel == null)
                return VideoPlayerLockOption.Anyone;
            try
            {
                return (VideoPlayerLockOption)_propOurLockLevel.GetValue(vp);
            }
            catch
            {
                return VideoPlayerLockOption.Anyone;
            }
        }

        public static string GetLockOptionLabel(VideoPlayerLockOption option)
        {
            return option switch
            {
                VideoPlayerLockOption.GroupMembers => "Group Members",
                VideoPlayerLockOption.InstanceCreator => "Instance Creator",
                VideoPlayerLockOption.Spawner => "Prop Spawner",
                VideoPlayerLockOption.GroupModerators => "Group Moderators",
                VideoPlayerLockOption.InstanceModerator => "Instance Moderators",
                VideoPlayerLockOption.Staff => "CVR Staff",
                _ => "Everyone"
            };
        }

        public static bool CanControl(CVRVideoPlayer vp)
        {
            if (!IsValid(vp))
                return false;
            if (!vp.syncEnabled)
                return true;
            return (int)GetOurLockLevel(vp) >= (int)vp.ControlPermission;
        }

        public static void SafeSetVolume(CVRVideoPlayer vp, float volume)
        {
            if (!IsValid(vp))
                return;
            if (vp.VideoPlayer == null)
            {
                vp.playbackVolume = volume;
                return;
            }
            vp.SetVolume(volume);
        }

        public static void SafeSetAudioMode(CVRVideoPlayer vp, VideoPlayerUtils.AudioMode mode)
        {
            if (!IsValid(vp))
                return;
            if (vp.VideoPlayer == null)
            {
                vp.audioPlaybackMode = mode;
                return;
            }
            vp.SetAudioMode(mode);
        }

        public static void RequestSeek(CVRVideoPlayer vp, double targetTime)
        {
            if (!IsValid(vp) || vp.VideoPlayer == null || IsLivestream(vp) || !CanControl(vp))
                return;

            double duration = GetDuration(vp);
            if (duration > 0.0)
                targetTime = Math.Max(0.0, Math.Min(duration, targetTime));
            else
                targetTime = Math.Max(0.0, targetTime);

            vp.SetVideoTimestamp(targetTime, broadcast: false);
            if (vp.syncEnabled)
            {
                _pendingSeekPlayer = vp;
                _pendingSeekTime = targetTime;
                _pendingSeekSendAt = Time.unscaledTime + 0.15f;
            }
        }

        public static void SeekRelative(CVRVideoPlayer vp, double deltaSeconds)
        {
            if (!IsValid(vp) || IsLivestream(vp))
                return;
            RequestSeek(vp, GetCurrentTime(vp) + deltaSeconds);
        }

        public static void PlayUrl(CVRVideoPlayer vp, string url, bool keepCustomPlaylist = false)
        {
            if (!IsValid(vp) || string.IsNullOrWhiteSpace(url))
                return;

            string cleaned = NormalizeUrl(url.Trim());
            if (!keepCustomPlaylist)
                ClearCustomPlaylist();

            RecordModUrl(cleaned);
            vp.SetVideoUrl(cleaned);
        }

        public static void PlayCustomPlaylist(SavedPlaylist playlist, int startIndex = 0)
        {
            var vp = SelectedPlayer;
            if (!IsValid(vp) || playlist == null || playlist.Items.Count == 0)
                return;

            _activePlaylist = playlist;
            _activePlaylistPlayer = vp;
            _activePlaylistIndex = Mathf.Clamp(startIndex, 0, playlist.Items.Count - 1);

            PlayUrl(vp, playlist.Items[_activePlaylistIndex].Url, keepCustomPlaylist: true);
        }

        public static void ClearCustomPlaylist()
        {
            _activePlaylist = null;
            _activePlaylistIndex = -1;
            _activePlaylistPlayer = null;
            _recentModUrls.Clear();
        }

        public static void HandlePlaylistDeleted(SavedPlaylist playlist)
        {
            if (playlist != null && _activePlaylist == playlist)
                ClearCustomPlaylist();
        }

        public static void HandlePlaylistItemRemoved(SavedPlaylist playlist, int removedIndex)
        {
            if (playlist == null || _activePlaylist != playlist)
                return;

            if (playlist.Items.Count == 0)
            {
                ClearCustomPlaylist();
                return;
            }

            if (removedIndex < _activePlaylistIndex)
                _activePlaylistIndex--;
            else if (_activePlaylistIndex >= playlist.Items.Count)
                _activePlaylistIndex = playlist.Items.Count - 1;
        }

        public static void HandlePlaylistItemMoved(SavedPlaylist playlist, int oldIndex, int newIndex)
        {
            if (playlist == null || _activePlaylist != playlist)
                return;

            if (_activePlaylistIndex == oldIndex)
            {
                _activePlaylistIndex = newIndex;
            }
            else if (oldIndex < _activePlaylistIndex && newIndex >= _activePlaylistIndex)
            {
                _activePlaylistIndex--;
            }
            else if (oldIndex > _activePlaylistIndex && newIndex <= _activePlaylistIndex)
            {
                _activePlaylistIndex++;
            }
        }

        private static void RecordModUrl(string url)
        {
            _recentModUrls.Enqueue(url);
            while (_recentModUrls.Count > 4)
                _recentModUrls.Dequeue();
        }

        private static void ClearPendingSeek()
        {
            _pendingSeekPlayer = null;
            _pendingSeekTime = 0.0;
            _pendingSeekSendAt = -1f;
        }

        public static void NextTrack()
        {
            var vp = SelectedPlayer;
            if (!IsValid(vp))
                return;

            if (_activePlaylist != null && _activePlaylistPlayer == vp && _activePlaylist.Items.Count > 0)
            {
                AdvanceCustomPlaylist(1);
                return;
            }

            vp.Next();
        }

        public static void PreviousTrack()
        {
            var vp = SelectedPlayer;
            if (!IsValid(vp))
                return;

            if (_activePlaylist != null && _activePlaylistPlayer == vp && _activePlaylist.Items.Count > 0)
            {
                AdvanceCustomPlaylist(-1);
                return;
            }

            vp.Previous();
        }

        private static void AdvanceCustomPlaylist(int direction)
        {
            if (_activePlaylist == null || !IsValid(_activePlaylistPlayer) || _activePlaylist.Items.Count == 0)
                return;

            int count = _activePlaylist.Items.Count;
            int nextIndex;

            if (Settings.PlaylistShuffle && count > 1)
            {
                nextIndex = _rng.Next(count);
                if (nextIndex == _activePlaylistIndex)
                    nextIndex = (nextIndex + 1) % count;
            }
            else
            {
                nextIndex = _activePlaylistIndex + direction;
                if (nextIndex >= count)
                {
                    if (!Settings.PlaylistLoop)
                    {
                        ClearCustomPlaylist();
                        return;
                    }
                    nextIndex = 0;
                }
                else if (nextIndex < 0)
                {
                    nextIndex = Settings.PlaylistLoop ? count - 1 : 0;
                }
            }

            _activePlaylistIndex = nextIndex;
            PlayUrl(_activePlaylistPlayer, _activePlaylist.Items[_activePlaylistIndex].Url, keepCustomPlaylist: true);
        }

        public static bool IsHttpUrl(string input)
        {
            if (string.IsNullOrWhiteSpace(input))
                return false;
            return Uri.TryCreate(input.Trim(), UriKind.Absolute, out var uri) &&
                   (uri.Scheme == Uri.UriSchemeHttp || uri.Scheme == Uri.UriSchemeHttps);
        }

        public static string NormalizeUrl(string url)
        {
            if (Uri.TryCreate(url, UriKind.Absolute, out var uri) && !Uri.IsWellFormedUriString(url, UriKind.Absolute))
            {
                try
                {
                    string[] segments = uri.Segments;
                    for (int i = 0; i < segments.Length; i++)
                    {
                        string seg = segments[i].TrimEnd('/');
                        seg = Uri.UnescapeDataString(seg);
                        seg = Uri.EscapeDataString(seg);
                        segments[i] = seg;
                    }
                    return new UriBuilder(uri) { Path = string.Join("/", segments) }.Uri.AbsoluteUri;
                }
                catch
                {
                    return url;
                }
            }
            return url;
        }

        public static bool SearchYouTube(string query, Action<List<SavedVideoItem>, string> callback)
        {
            if (string.IsNullOrWhiteSpace(query) || IsSearchInFlight || MetaPort.Instance == null)
                return false;

            _searchInFlight = true;
            _searchStartedAt = Time.unscaledTime;
            MetaPort.Instance.StartCoroutine(
                YouTubeSearchUtils.YoutubeSearch(query.Trim(), 25, YoutubeSearchOrderFilter.Relevance, data =>
                {
                    _searchInFlight = false;
                    var results = new List<SavedVideoItem>();
                    if (data?.Results != null)
                    {
                        foreach (var item in data.Results)
                        {
                            if (item == null || string.IsNullOrWhiteSpace(item.Id))
                                continue;
                            results.Add(new SavedVideoItem
                            {
                                Title = string.IsNullOrWhiteSpace(item.Name) ? item.Id : item.Name,
                                Url = "https://youtube.com/watch?v=" + item.Id
                            });
                        }
                    }
                    callback?.Invoke(results, data?.Term ?? query.Trim());
                }));
            return true;
        }

        private static void HandleRegistered(CVRVideoPlayer vp)
        {
            if (!IsValid(vp))
                return;
            if (!_players.Contains(vp))
                _players.Add(vp);

            if (!IsValid(_selectedPlayer) && Settings.AutoSelectNearest)
                SelectNearest();

            OnPlayersListChanged?.Invoke();
        }

        private static void HandleUnregistered(CVRVideoPlayer vp)
        {
            _players.Remove(vp);
            if (_activePlaylistPlayer == vp)
                ClearCustomPlaylist();

            if (_selectedPlayer == vp)
            {
                _selectedPlayer = null;
                if (Settings.AutoSelectNearest && _players.Count > 0)
                    SelectNearest();
                else
                    OnSelectedPlayerChanged?.Invoke(null);
            }

            OnPlayersListChanged?.Invoke();
        }

        private static void HandleCleared()
        {
            _players.Clear();
            _selectedPlayer = null;
            ClearPendingSeek();
            ClearCustomPlaylist();
            OnSelectedPlayerChanged?.Invoke(null);
            OnPlayersListChanged?.Invoke();
        }

        private static void HandleFinishedPlaying(CVRVideoPlayer vp)
        {
            if (_activePlaylist != null && _activePlaylistPlayer == vp && _activePlaylist.Items.Count > 0)
            {
                AdvanceCustomPlaylist(1);
            }
        }

        private static void HandleUrlChanged(CVRVideoPlayer vp, string url, bool fromNetwork)
        {
            if (_activePlaylist != null && _activePlaylistPlayer == vp && fromNetwork)
            {
                bool matchesRecent = _recentModUrls.Any(u =>
                    string.Equals(u, url, StringComparison.OrdinalIgnoreCase));
                if (!matchesRecent)
                {
                    ClearCustomPlaylist();
                }
            }
        }

        private static void HandleUrlCleared(CVRVideoPlayer vp)
        {
            if (_activePlaylist != null && _activePlaylistPlayer == vp)
            {
                ClearCustomPlaylist();
            }
        }

        private static bool PruneDeadPlayers()
        {
            int removed = _players.RemoveAll(vp => !IsValid(vp));
            if (_selectedPlayer != null && !IsValid(_selectedPlayer))
            {
                _selectedPlayer = null;
                OnSelectedPlayerChanged?.Invoke(null);
            }
            if (_activePlaylistPlayer != null && !IsValid(_activePlaylistPlayer))
            {
                ClearCustomPlaylist();
            }
            return removed > 0;
        }

        private static string ResolveOwnerName(string ownerId)
        {
            if (string.IsNullOrWhiteSpace(ownerId))
                return string.Empty;
            if (!string.IsNullOrEmpty(AuthManager.UserId) && ownerId == AuthManager.UserId)
                return "You";
            try
            {
                string name = CVRPlayerManager.Instance?.TryGetPlayerName(ownerId);
                return string.IsNullOrWhiteSpace(name) ? string.Empty : name;
            }
            catch
            {
                return string.Empty;
            }
        }

        private static string CleanObjectName(string raw)
        {
            if (string.IsNullOrWhiteSpace(raw))
                return "Video Player";
            string cleaned = raw.Replace("(Clone)", "").Trim();
            if (cleaned.Length > 28)
                cleaned = cleaned.Substring(0, 25) + "...";
            return cleaned;
        }
    }
}
