using System;
using System.Collections.Generic;
using System.Linq;
using System.Runtime.CompilerServices;
using ABI.CCK.Components;
using ABI_RC.VideoPlayer.Scripts;
using ABI_RC.VideoPlayer.Scripts.Players;
using GameCommon.Models.VideoPlayer;
using LumenicMenu.Api;
using UnityEngine;

namespace VideoPlayerRemote
{
    internal static class LumenicRemoteUI
    {
        private const string ModId = "com.lenserror.videoplayerremote";

        private static readonly VideoPlayerUtils.Resolution[] Resolutions =
            Enum.GetValues(typeof(VideoPlayerUtils.Resolution))
                .Cast<VideoPlayerUtils.Resolution>()
                .ToArray();

        private static readonly VideoPlayerUtils.AudioMode[] AudioModes =
        {
            VideoPlayerUtils.AudioMode.Direct,
            VideoPlayerUtils.AudioMode.AudioSource,
            VideoPlayerUtils.AudioMode.RoomScale
        };

        private static ExtTabHandle _tab;
        private static bool _initialized;

        private static int _mainSection = 0; // 0: Remote, 1: Library, 2: Screens, 3: Settings
        private static int _librarySection = 0; // 0: Favorites, 1: Playlists, 2: World Playlists, 3: Search

        private static string _urlSearchDraft = string.Empty;
        private static float _lastInputEditTime = -10f;
        private static float _savedVolumeBeforeMute = 0.8f;
        private static int _lastTickSecond = -1;
        private static VideoPlayerUtils.PlayerState _lastTickState;
        private static bool _lastTickUntrusted;
        private static CVRVideoPlayer _lastTickPlayer;

        private static LocalScreenInstance _activeScreenDetail;
        private static SavedPlaylist _activePlaylistDetail;
        private static List<SavedVideoItem> _lastSearchResults = new List<SavedVideoItem>();
        private static string _lastSearchTerm = string.Empty;

        [MethodImpl(MethodImplOptions.NoInlining)]
        public static bool TryInit()
        {
            if (_initialized) return true;

            try
            {
                if (LumenicApi.ApiVersion < 1 || !LumenicApi.IsQuickMenuActive)
                {
                    return false;
                }

                _tab = LumenicApi.RegisterQuickMenuTab(ModId, new ExtTabOptions
                {
                    Id = "video_remote",
                    Title = "Video",
                    Subtitle = "World & Prop VideoPlayer Remote",
                    Icon = "smart_display",
                    BadgeText = () =>
                    {
                        int count = VideoPlayerTracker.Players.Count;
                        return count > 0 ? count.ToString() : null;
                    },
                    Priority = 15
                }, BuildMainPage);

                VideoPlayerTracker.OnPlayersListChanged += MarkDirty;
                VideoPlayerTracker.OnSelectedPlayerChanged += OnSelectedPlayerChanged;
                LocalScreenManager.OnScreensChanged += MarkDirty;
                LibraryStore.OnLibraryChanged += MarkDirty;

                _initialized = true;
                VideoPlayerRemoteMod.Log?.Msg("Registered VideoPlayerRemote Quick Menu tab in LumenicMenu.");
                return true;
            }
            catch (Exception ex)
            {
                VideoPlayerRemoteMod.Log?.Warning($"LumenicMenu integration failed, falling back to BTKUILib: {ex.Message}");
                return false;
            }
        }

        [MethodImpl(MethodImplOptions.NoInlining)]
        public static void Shutdown()
        {
            if (!_initialized) return;
            _initialized = false;

            VideoPlayerTracker.OnPlayersListChanged -= MarkDirty;
            VideoPlayerTracker.OnSelectedPlayerChanged -= OnSelectedPlayerChanged;
            LocalScreenManager.OnScreensChanged -= MarkDirty;
            LibraryStore.OnLibraryChanged -= MarkDirty;

            try
            {
                _tab?.Dispose();
                _tab = null;
                LumenicApi.UnregisterMod(ModId);
            }
            catch (Exception ex)
            {
                VideoPlayerRemoteMod.Log?.Warning($"Failed to unregister LumenicMenu UI: {ex.Message}");
            }
        }

        [MethodImpl(MethodImplOptions.NoInlining)]
        public static void Tick()
        {
            if (!_initialized || _mainSection != 0)
                return;

            // Avoid rebuilding the page while the user is actively typing in the search/URL input
            if (Time.unscaledTime - _lastInputEditTime < 2.0f)
                return;

            var vp = VideoPlayerTracker.SelectedPlayer;
            if (vp == null)
            {
                if (_lastTickPlayer != null)
                {
                    _lastTickPlayer = null;
                    _tab?.MarkDirty();
                }
                return;
            }

            var state = VideoPlayerTracker.GetState(vp);
            int curSec = (int)VideoPlayerTracker.GetCurrentTime(vp);
            bool untrusted = vp.UntrustedVideoPending;

            bool changed = vp != _lastTickPlayer ||
                           state != _lastTickState ||
                           untrusted != _lastTickUntrusted ||
                           (curSec != _lastTickSecond && (state == VideoPlayerUtils.PlayerState.Playing ||
                                                          state == VideoPlayerUtils.PlayerState.Buffering ||
                                                          state == VideoPlayerUtils.PlayerState.Seeking));

            if (!changed)
                return;

            _lastTickPlayer = vp;
            _lastTickState = state;
            _lastTickUntrusted = untrusted;
            _lastTickSecond = curSec;
            _tab?.MarkDirty();
        }

        private static void OnSelectedPlayerChanged(CVRVideoPlayer _)
        {
            MarkDirty();
        }

        private static void MarkDirty()
        {
            _tab?.MarkDirty();
        }

        private static void BuildMainPage(UiBuilder ui)
        {
            ui.Tabs(
                id: "vpr_main_tabs",
                options: new[] { "Remote", "Library", "Screens", "Settings" },
                selectedIndex: _mainSection,
                onChange: idx => { _mainSection = idx; MarkDirty(); });

            switch (_mainSection)
            {
                case 0:
                    BuildRemoteSection(ui);
                    break;
                case 1:
                    BuildLibrarySection(ui);
                    break;
                case 2:
                    BuildScreensSection(ui);
                    break;
                case 3:
                    BuildSettingsSection(ui);
                    break;
            }
        }

        // ==================== REMOTE TAB ====================

        private static void BuildRemoteSection(UiBuilder ui)
        {
            var vp = VideoPlayerTracker.SelectedPlayer;

            // Target & Now Playing Card
            ui.Card(card =>
            {
                card.Row(top =>
                {
                    top.Col(col =>
                    {
                        if (vp == null)
                        {
                            col.Text($"Target: None ({VideoPlayerTracker.Players.Count} in instance)", size: 12f, weight: "bold");
                            col.Text("Select a world or prop VideoPlayer to control.", size: 10.5f, fg: "muted");
                        }
                        else
                        {
                            string lockInfo = VideoPlayerTracker.GetLockOptionLabel(vp.ControlPermission);
                            bool canCtrl = VideoPlayerTracker.CanControl(vp);
                            col.Row(titleRow =>
                            {
                                titleRow.Text(VideoPlayerTracker.GetFullLabel(vp), size: 12.5f, weight: "bold", ellipsis: true, style: new ExtStyle { Grow = 1f });
                                titleRow.Badge(canCtrl ? lockInfo : $"{lockInfo} (Locked)", fg: canCtrl ? "accent" : "amber");
                            }, new ExtStyle { Align = "center", Gap = 6f });

                            var state = VideoPlayerTracker.GetState(vp);
                            string title = VideoPlayerTracker.GetCurrentTitle(vp);
                            string timeText = VideoPlayerTracker.IsLivestream(vp)
                                ? "LIVE"
                                : $"{FormatTime(VideoPlayerTracker.GetCurrentTime(vp))} / {FormatTime(VideoPlayerTracker.GetDuration(vp))}";
                            string requester = !string.IsNullOrWhiteSpace(vp.lastNetworkVideoUsername)
                                ? $" • By {vp.lastNetworkVideoUsername}"
                                : string.Empty;

                            col.Text(title, size: 11.5f, weight: "semibold", ellipsis: true);
                            col.Text($"[{state}] {timeText}{requester}", size: 10f, fg: "muted", ellipsis: true);

                            if (VideoPlayerTracker.ActivePlaylist != null &&
                                VideoPlayerTracker.ActivePlaylistPlayer == vp &&
                                VideoPlayerTracker.ActivePlaylist.Items.Count > 0)
                            {
                                col.Text(
                                    $"Queue: {VideoPlayerTracker.ActivePlaylist.Name} ({VideoPlayerTracker.ActivePlaylistIndex + 1}/{VideoPlayerTracker.ActivePlaylist.Items.Count})",
                                    size: 10f,
                                    fg: "accent",
                                    ellipsis: true);
                            }
                        }
                    }, new ExtStyle { Grow = 1f, Gap = 2f });

                    top.Button(
                        id: "pick_player",
                        label: "Players",
                        onClick: OpenPlayerSelectSubPage,
                        icon: "tv");
                }, new ExtStyle { Align = "center", Gap = 8f });

                if (vp != null)
                {
                    double duration = VideoPlayerTracker.GetDuration(vp);
                    double curTime = VideoPlayerTracker.GetCurrentTime(vp);
                    bool isLive = VideoPlayerTracker.IsLivestream(vp);
                    bool seekDisabled = isLive || duration <= 0.1 || !VideoPlayerTracker.CanControl(vp);
                    float maxSeek = duration > 1.0 ? (float)duration : 100f;
                    float curSeek = Mathf.Clamp((float)curTime, 0f, maxSeek);

                    card.Slider(
                        id: "seek_slider",
                        label: isLive ? "Timeline (LIVE)" : $"Timeline ({FormatTime(curTime)} / {FormatTime(duration)})",
                        value: curSeek,
                        min: 0f,
                        max: maxSeek,
                        onChange: val => VideoPlayerTracker.RequestSeek(vp, val),
                        disabled: seekDisabled,
                        style: new ExtStyle { WFrac = 1f });
                }
            }, new ExtStyle { Gap = 6f, Px = 10f, Py = 8f });

            // Untrusted URL Prompt
            if (vp != null && vp.UntrustedVideoPending)
            {
                ui.Card(card =>
                {
                    card.Row(r =>
                    {
                        r.Icon("warning", size: 16f, fg: "amber");
                        r.Text("Untrusted Video URL Pending", size: 11.5f, weight: "bold", fg: "amber", style: new ExtStyle { Grow = 1f });
                    }, new ExtStyle { Align = "center", Gap = 6f });

                    card.Row(r =>
                    {
                        r.Button("untrusted_once", "Watch Once", () => HandleUntrusted(VideoPlayerUtils.UntrustedVideoAction.WatchOnce), variant: "accent", style: new ExtStyle { Grow = 1f });
                        r.Button("untrusted_session", "Allow Session", () => HandleUntrusted(VideoPlayerUtils.UntrustedVideoAction.WatchAndAllowForSession), style: new ExtStyle { Grow = 1f });
                        r.Button("untrusted_trust", "Trust Domain", () => HandleUntrusted(VideoPlayerUtils.UntrustedVideoAction.WatchAndTrustDomain), variant: "emerald", style: new ExtStyle { Grow = 1f });
                        r.Button("untrusted_deny", "Deny", () => HandleUntrusted(VideoPlayerUtils.UntrustedVideoAction.Deny), variant: "danger", style: new ExtStyle { Grow = 1f });
                    }, new ExtStyle { Gap = 5f });
                }, new ExtStyle { Gap = 6f, Px = 10f, Py = 7f, BorderColor = "amber", BorderW = 1f });
            }

            // URL / Search Bar
            ui.Row(r =>
            {
                r.TextInput(
                    id: "url_search_input",
                    text: _urlSearchDraft,
                    placeholder: "Paste URL or search YouTube...",
                    onChange: s =>
                    {
                        _urlSearchDraft = s ?? string.Empty;
                        _lastInputEditTime = Time.unscaledTime;
                    },
                    onSubmit: s =>
                    {
                        _urlSearchDraft = string.Empty;
                        _lastInputEditTime = -10f;
                        HandleUrlOrSearchInput(s);
                    },
                    maxLen: 500,
                    style: new ExtStyle { Grow = 1f });

                r.IconButton(
                    id: "url_go",
                    icon: "send",
                    onClick: () =>
                    {
                        string input = _urlSearchDraft;
                        _urlSearchDraft = string.Empty;
                        HandleUrlOrSearchInput(input);
                    },
                    variant: "accent",
                    tooltip: "Play URL or Search YouTube");

                r.IconButton(
                    id: "url_paste",
                    icon: "content_paste",
                    onClick: () => HandleUrlOrSearchInput(GUIUtility.systemCopyBuffer),
                    tooltip: "Paste & Play from Clipboard");
            }, new ExtStyle { Align = "center", Gap = 6f });

            // Transport & Audio
            ui.Scroll("remote_scroll", list =>
            {
                bool hasVp = vp != null;
                var state = hasVp ? VideoPlayerTracker.GetState(vp) : default;
                bool isPlaying = hasVp && (state == VideoPlayerUtils.PlayerState.Playing ||
                                 state == VideoPlayerUtils.PlayerState.Buffering ||
                                 state == VideoPlayerUtils.PlayerState.Seeking);

                list.Card(card =>
                {
                    card.Row(r =>
                    {
                        r.IconButton("prev_track", "skip_previous", VideoPlayerTracker.PreviousTrack, disabled: !hasVp, tooltip: "Previous track");
                        r.IconButton("seek_back", "replay_10", () => { if (vp != null) VideoPlayerTracker.SeekRelative(vp, -Settings.SeekStepSeconds); }, disabled: !hasVp, tooltip: $"Seek -{Settings.SeekStepSeconds:0}s");
                        r.Button("play_pause", isPlaying ? "Pause" : "Play", TogglePlayPause, icon: isPlaying ? "pause" : "play_arrow", variant: "accent", disabled: !hasVp, style: new ExtStyle { Grow = 1f });
                        r.IconButton("stop_btn", "stop", () =>
                        {
                            if (vp == null) return;
                            VideoPlayerTracker.ClearCustomPlaylist();
                            vp.ClearVideoUrl(true, null);
                            MarkDirty();
                        }, variant: "danger", disabled: !hasVp, tooltip: "Stop / Clear URL");
                        r.IconButton("seek_fwd", "forward_10", () => { if (vp != null) VideoPlayerTracker.SeekRelative(vp, Settings.SeekStepSeconds); }, disabled: !hasVp, tooltip: $"Seek +{Settings.SeekStepSeconds:0}s");
                        r.IconButton("next_track", "skip_next", VideoPlayerTracker.NextTrack, disabled: !hasVp, tooltip: "Next track");
                        r.IconButton("resync", "sync", () =>
                        {
                            if (vp == null) return;
                            string url = VideoPlayerTracker.GetCurrentUrl(vp);
                            if (!string.IsNullOrEmpty(url))
                            {
                                vp.SetWatching(false);
                                vp.SetWatching(true);
                                LumenicApi.ShowToast("Resyncing VideoPlayer...", 2.0f);
                            }
                        }, disabled: !hasVp, tooltip: "Resync playback");
                    }, new ExtStyle { Align = "center", Gap = 5f });

                    float vol = hasVp ? Mathf.Clamp01(vp.playbackVolume) : 1f;
                    bool muted = hasVp && vp.playbackVolume <= 0.001f;
                    card.Row(r =>
                    {
                        r.IconButton(
                            id: "mute_btn",
                            icon: muted ? "volume_off" : "volume_up",
                            onClick: () =>
                            {
                                if (vp == null) return;
                                if (!muted)
                                {
                                    if (vp.playbackVolume > 0.001f) _savedVolumeBeforeMute = vp.playbackVolume;
                                    VideoPlayerTracker.SafeSetVolume(vp, 0f);
                                }
                                else
                                {
                                    VideoPlayerTracker.SafeSetVolume(vp, _savedVolumeBeforeMute > 0.01f ? _savedVolumeBeforeMute : 0.8f);
                                }
                                MarkDirty();
                            },
                            active: muted,
                            disabled: !hasVp,
                            tooltip: "Mute / Unmute");

                        r.Slider(
                            id: "vol_slider",
                            label: $"Volume ({Mathf.RoundToInt(vol * 100f)}%)",
                            value: vol,
                            min: 0f,
                            max: 1f,
                            onChange: v =>
                            {
                                if (vp == null) return;
                                if (v > 0.001f) _savedVolumeBeforeMute = v;
                                VideoPlayerTracker.SafeSetVolume(vp, v);
                                MarkDirty();
                            },
                            step: 0.02f,
                            disabled: !hasVp,
                            style: new ExtStyle { Grow = 1f });
                    }, new ExtStyle { Align = "center", Gap = 6f });

                    float speed = hasVp ? Mathf.Clamp(vp.localPlaybackSpeed, 0.5f, 2f) : 1f;
                    card.Slider(
                        id: "speed_slider",
                        label: $"Playback Speed ({speed:0.00}x)",
                        value: speed,
                        min: 0.5f,
                        max: 2.0f,
                        onChange: v =>
                        {
                            if (vp != null) vp.localPlaybackSpeed = Mathf.Clamp(v, 0.5f, 2f);
                            MarkDirty();
                        },
                        step: 0.05f,
                        disabled: !hasVp,
                        style: new ExtStyle { WFrac = 1f });
                }, new ExtStyle { Gap = 8f, Px = 10f, Py = 8f });

                list.Card(card =>
                {
                    card.Row(r =>
                    {
                        r.Toggle(
                            id: "watching_toggle",
                            label: "Watching",
                            value: hasVp && vp.IsWatching,
                            onChange: val => { vp?.SetWatching(val); MarkDirty(); },
                            icon: "visibility",
                            disabled: !hasVp,
                            style: new ExtStyle { Grow = 1f });

                        r.Toggle(
                            id: "hud_toggle",
                            label: "HUD / OSD",
                            value: Settings.HudEnabled,
                            onChange: val => { HudScreenManager.SetEnabled(val); MarkDirty(); },
                            icon: "picture_in_picture_alt",
                            style: new ExtStyle { Grow = 1f });
                    }, new ExtStyle { Gap = 6f });

                    card.Row(r =>
                    {
                        string curUrl = hasVp ? VideoPlayerTracker.GetCurrentUrl(vp) : string.Empty;
                        bool isFav = !string.IsNullOrWhiteSpace(curUrl) && LibraryStore.IsFavorite(curUrl);
                        r.Button(
                            id: "fav_current",
                            label: isFav ? "Unfavorite" : "Favorite Current",
                            onClick: FavoriteCurrentVideo,
                            icon: isFav ? "star" : "star_outline",
                            active: isFav,
                            disabled: !hasVp,
                            style: new ExtStyle { Grow = 1f });

                        r.Button(
                            id: "spawn_screen_quick",
                            label: "Spawn Screen",
                            onClick: () =>
                            {
                                var screen = LocalScreenManager.SpawnScreen();
                                if (screen != null)
                                    LumenicApi.ShowToast($"Spawned Screen #{screen.Id}", 2.0f);
                            },
                            icon: "add_to_queue",
                            style: new ExtStyle { Grow = 1f });
                    }, new ExtStyle { Gap = 6f });
                }, new ExtStyle { Gap = 6f, Px = 10f, Py = 8f });
            }, new ExtStyle { Grow = 1f, Gap = 8f });
        }

        // ==================== LIBRARY TAB ====================

        private static void BuildLibrarySection(UiBuilder ui)
        {
            ui.Tabs(
                id: "vpr_lib_tabs",
                options: new[] { "Favorites", "Playlists", "World Playlists", "Search" },
                selectedIndex: _librarySection,
                onChange: idx => { _librarySection = idx; MarkDirty(); });

            ui.Scroll("library_scroll", list =>
            {
                switch (_librarySection)
                {
                    case 0:
                        BuildFavoritesList(list);
                        break;
                    case 1:
                        BuildPlaylistsList(list);
                        break;
                    case 2:
                        BuildNativePlaylistsList(list);
                        break;
                    case 3:
                        BuildSearchResultsList(list);
                        break;
                }
            }, new ExtStyle { Grow = 1f, Gap = 6f });
        }

        private static void BuildFavoritesList(UiBuilder list)
        {
            var favs = LibraryStore.Data.Favorites;
            if (favs.Count == 0)
            {
                list.Card(c => c.Text("No favorite videos saved yet.", fg: "muted"));
                return;
            }

            for (int i = 0; i < favs.Count; i++)
            {
                var item = favs[i];
                int idx = i;
                list.Row(row =>
                {
                    row.Col(col =>
                    {
                        col.Text(item.DisplayTitle, size: 11.5f, weight: "semibold", ellipsis: true);
                        col.Text(item.Url, size: 9.5f, fg: "dim", ellipsis: true);
                    }, new ExtStyle { Grow = 1f, Gap = 1f });

                    row.IconButton($"fav_play_{idx}", "play_arrow", () => PlayItem(item), variant: "accent", tooltip: "Play");
                    row.IconButton($"fav_add_{idx}", "playlist_add", () => OpenPickPlaylistModal(item), tooltip: "Add to Playlist");
                    row.IconButton($"fav_del_{idx}", "delete", () =>
                    {
                        LibraryStore.RemoveFavorite(item.Url);
                        LumenicApi.ShowToast("Removed from Favorites.", 1.5f);
                    }, variant: "danger", tooltip: "Remove");
                }, new ExtStyle { Align = "center", Gap = 6f, Px = 9f, Py = 6f, Bg = "card", Radius = 6f });
            }
        }

        private static void BuildPlaylistsList(UiBuilder list)
        {
            list.Row(r =>
            {
                r.Button("create_pl", "New Playlist", () =>
                {
                    LumenicApi.OpenKeyboard(
                        initialText: string.Empty,
                        onSubmit: name =>
                        {
                            if (string.IsNullOrWhiteSpace(name)) return;
                            var pl = LibraryStore.CreatePlaylist(name.Trim());
                            LumenicApi.ShowToast($"Created \"{pl.Name}\"", 2.0f);
                        },
                        title: "Create Playlist",
                        placeholder: "Playlist name...",
                        maxLen: 64,
                        surface: ExtSurface.QuickMenu);
                }, icon: "add", variant: "accent", style: new ExtStyle { Grow = 1f });

                r.Toggle("pl_shuffle", "Shuffle", Settings.PlaylistShuffle, v => { Settings.PlaylistShuffle = v; MarkDirty(); }, icon: "shuffle");
                r.Toggle("pl_loop", "Loop", Settings.PlaylistLoop, v => { Settings.PlaylistLoop = v; MarkDirty(); }, icon: "repeat");
            }, new ExtStyle { Align = "center", Gap = 6f });

            var playlists = LibraryStore.Data.Playlists;
            if (playlists.Count == 0)
            {
                list.Card(c => c.Text("No custom playlists yet.", fg: "muted"));
                return;
            }

            for (int i = 0; i < playlists.Count; i++)
            {
                var pl = playlists[i];
                int idx = i;
                list.Row(row =>
                {
                    row.Col(col =>
                    {
                        col.Text(pl.Name, size: 12f, weight: "semibold", ellipsis: true);
                        col.Text($"{pl.Items.Count} video(s)", size: 10f, fg: "muted");
                    }, new ExtStyle { Grow = 1f, Gap = 1f });

                    row.IconButton($"pl_play_{idx}", "play_arrow", () =>
                    {
                        if (pl.Items.Count == 0)
                        {
                            LumenicApi.ShowToast("Playlist is empty.", 1.5f);
                            return;
                        }
                        VideoPlayerTracker.PlayCustomPlaylist(pl, 0);
                        _mainSection = 0;
                        MarkDirty();
                    }, variant: "accent", tooltip: "Play Playlist");

                    row.Button($"pl_open_{idx}", "Open", () =>
                    {
                        _activePlaylistDetail = pl;
                        _tab?.PushSubPage(new ExtSubPageOptions { Id = $"pl_{pl.Id}", Title = pl.Name }, BuildPlaylistDetailSubPage);
                    }, icon: "chevron_right");
                }, new ExtStyle { Align = "center", Gap = 6f, Px = 9f, Py = 6f, Bg = "card", Radius = 6f });
            }
        }

        private static void BuildPlaylistDetailSubPage(UiBuilder ui)
        {
            var pl = _activePlaylistDetail;
            if (pl == null || !LibraryStore.Data.Playlists.Contains(pl))
            {
                ui.Card(c => c.Text("Playlist no longer exists.", fg: "muted"));
                return;
            }

            ui.Row(r =>
            {
                r.Button("pl_play_all", "Play All", () =>
                {
                    if (pl.Items.Count == 0)
                    {
                        LumenicApi.ShowToast("Playlist is empty.", 1.5f);
                        return;
                    }
                    VideoPlayerTracker.PlayCustomPlaylist(pl, 0);
                    _tab?.PopSubPage();
                    _mainSection = 0;
                    MarkDirty();
                }, icon: "play_arrow", variant: "accent", style: new ExtStyle { Grow = 1f });

                r.Button("pl_rename", "Rename", () =>
                {
                    LumenicApi.OpenKeyboard(
                        initialText: pl.Name,
                        onSubmit: name =>
                        {
                            if (!string.IsNullOrWhiteSpace(name))
                                LibraryStore.RenamePlaylist(pl, name.Trim());
                        },
                        title: "Rename Playlist",
                        placeholder: "Playlist name...",
                        maxLen: 64,
                        surface: ExtSurface.QuickMenu);
                }, icon: "edit");

                r.Button("pl_delete", "Delete", () =>
                {
                    LumenicApi.OpenModal(ExtSurface.QuickMenu, ModId, new ExtModalOptions
                    {
                        Id = "confirm_del_pl",
                        Title = $"Delete \"{pl.Name}\"?",
                        Width = 380f
                    }, m =>
                    {
                        m.Text("This will permanently remove this custom playlist.", fg: "muted", wrap: true);
                        m.Row(mr =>
                        {
                            mr.Button("cancel_del_pl", "Cancel", () => LumenicApi.CloseModal(ExtSurface.QuickMenu));
                            mr.Button("ok_del_pl", "Delete", () =>
                            {
                                LibraryStore.DeletePlaylist(pl);
                                _activePlaylistDetail = null;
                                LumenicApi.CloseModal(ExtSurface.QuickMenu);
                                _tab?.PopSubPage();
                            }, variant: "danger");
                        }, new ExtStyle { Gap = 8f, Justify = "end" });
                    });
                }, icon: "delete", variant: "danger");
            }, new ExtStyle { Gap = 6f });

            ui.Scroll("pl_items_scroll", list =>
            {
                if (pl.Items.Count == 0)
                {
                    list.Card(c => c.Text("No tracks in this playlist yet.", fg: "muted"));
                    return;
                }

                for (int i = 0; i < pl.Items.Count; i++)
                {
                    int idx = i;
                    var item = pl.Items[idx];
                    list.Row(row =>
                    {
                        row.Col(col =>
                        {
                            col.Text($"{idx + 1}. {item.DisplayTitle}", size: 11.5f, weight: "semibold", ellipsis: true);
                            col.Text(item.Url, size: 9.5f, fg: "dim", ellipsis: true);
                        }, new ExtStyle { Grow = 1f, Gap = 1f });

                        row.IconButton($"pli_play_{idx}", "play_arrow", () =>
                        {
                            VideoPlayerTracker.PlayCustomPlaylist(pl, idx);
                            _tab?.PopSubPage();
                            _mainSection = 0;
                            MarkDirty();
                        }, variant: "accent", tooltip: "Play from here");

                        row.IconButton($"pli_up_{idx}", "arrow_upward", () =>
                        {
                            if (idx > 0)
                                LibraryStore.MoveInPlaylist(pl, idx, -1);
                        }, disabled: idx == 0, tooltip: "Move up");

                        row.IconButton($"pli_dn_{idx}", "arrow_downward", () =>
                        {
                            if (idx < pl.Items.Count - 1)
                                LibraryStore.MoveInPlaylist(pl, idx, 1);
                        }, disabled: idx >= pl.Items.Count - 1, tooltip: "Move down");

                        row.IconButton($"pli_rm_{idx}", "delete", () =>
                        {
                            LibraryStore.RemoveFromPlaylist(pl, idx);
                        }, variant: "danger", tooltip: "Remove");
                    }, new ExtStyle { Align = "center", Gap = 4f, Px = 8f, Py = 6f, Bg = "card", Radius = 6f });
                }
            }, new ExtStyle { Grow = 1f, Gap = 5f });
        }

        private static void BuildNativePlaylistsList(UiBuilder list)
        {
            var vp = VideoPlayerTracker.SelectedPlayer;
            if (vp == null)
            {
                list.Card(c => c.Text("No video player selected.", fg: "muted"));
                return;
            }

            if (vp.entities == null || vp.entities.Count == 0)
            {
                list.Card(c => c.Text("This video player has no built-in playlists.", fg: "muted"));
                return;
            }

            for (int p = 0; p < vp.entities.Count; p++)
            {
                var playlist = vp.entities[p];
                if (playlist == null || playlist.playlistVideos == null) continue;

                string title = string.IsNullOrWhiteSpace(playlist.playlistTitle) ? "Playlist" : playlist.playlistTitle;
                list.Card(card =>
                {
                    card.Text($"{title} ({playlist.playlistVideos.Count})", size: 12f, weight: "bold");
                    for (int v = 0; v < playlist.playlistVideos.Count; v++)
                    {
                        var vid = playlist.playlistVideos[v];
                        if (vid == null) continue;
                        string vTitle = string.IsNullOrWhiteSpace(vid.videoTitle) ? vid.videoUrl : vid.videoTitle;

                        card.Row(row =>
                        {
                            row.Text(vTitle, size: 11f, weight: "medium", ellipsis: true, style: new ExtStyle { Grow = 1f });
                            row.IconButton($"npl_{p}_{v}", "play_arrow", () =>
                            {
                                if (!VideoPlayerTracker.IsValid(vp))
                                {
                                    LumenicApi.ShowToast("Video player no longer exists.", 2.0f);
                                    return;
                                }
                                VideoPlayerTracker.ClearCustomPlaylist();
                                vp.SelectVideo(playlist, vid);
                                LumenicApi.ShowToast($"Playing: {vTitle}", 2.0f);
                                _mainSection = 0;
                                MarkDirty();
                            }, variant: "accent");
                        }, new ExtStyle { Align = "center", Gap = 6f, Px = 8f, Py = 5f, Bg = "card_inner", Radius = 6f });
                    }
                }, new ExtStyle { Gap = 5f, Px = 10f, Py = 8f });
            }
        }

        private static void BuildSearchResultsList(UiBuilder list)
        {
            if (_lastSearchResults == null || _lastSearchResults.Count == 0)
            {
                list.Card(c => c.Text(
                    string.IsNullOrWhiteSpace(_lastSearchTerm)
                        ? "Type a YouTube search in the Remote tab's input box."
                        : $"No results found for \"{_lastSearchTerm}\".",
                    fg: "muted",
                    wrap: true));
                return;
            }

            list.Text($"Results for \"{_lastSearchTerm}\" ({_lastSearchResults.Count})", size: 11.5f, weight: "semibold", fg: "muted");

            for (int i = 0; i < _lastSearchResults.Count; i++)
            {
                var item = _lastSearchResults[i];
                int idx = i;
                list.Row(row =>
                {
                    row.Col(col =>
                    {
                        col.Text(item.DisplayTitle, size: 11.5f, weight: "semibold", ellipsis: true);
                        col.Text(item.Url, size: 9.5f, fg: "dim", ellipsis: true);
                    }, new ExtStyle { Grow = 1f, Gap = 1f });

                    row.IconButton($"sr_play_{idx}", "play_arrow", () => PlayItem(item), variant: "accent", tooltip: "Play");
                    row.IconButton($"sr_fav_{idx}", "star", () =>
                    {
                        LibraryStore.AddFavorite(item.DisplayTitle, item.Url);
                        LumenicApi.ShowToast("Added to Favorites!", 1.5f);
                    }, tooltip: "Favorite");
                    row.IconButton($"sr_add_{idx}", "playlist_add", () => OpenPickPlaylistModal(item), tooltip: "Add to Playlist");
                }, new ExtStyle { Align = "center", Gap = 5f, Px = 9f, Py = 6f, Bg = "card", Radius = 6f });
            }
        }

        // ==================== SCREENS & HUD TAB ====================

        private static void BuildScreensSection(UiBuilder ui)
        {
            ui.Card(card =>
            {
                card.Row(r =>
                {
                    r.Text("Spawn Local Screen", size: 12f, weight: "semibold", style: new ExtStyle { Grow = 1f });
                    r.Button("hud_setup_btn", "HUD / OSD Setup", OpenHudSubPage, icon: "picture_in_picture_alt", variant: "accent");
                }, new ExtStyle { Align = "center", Gap = 6f });

                card.Row(r =>
                {
                    r.Button("spawn_sm", "Small (0.6m)", () => SpawnWithPreset(0.6f), style: new ExtStyle { Grow = 1f });
                    r.Button("spawn_md", "Medium (1.2m)", () => SpawnWithPreset(1.2f), style: new ExtStyle { Grow = 1f });
                    r.Button("spawn_lg", "Large (2.2m)", () => SpawnWithPreset(2.2f), style: new ExtStyle { Grow = 1f });
                    r.Button("spawn_xl", "Cinema (3.5m)", () => SpawnWithPreset(3.5f), style: new ExtStyle { Grow = 1f });
                }, new ExtStyle { Gap = 5f });
            }, new ExtStyle { Gap = 6f, Px = 10f, Py = 8f });

            ui.Scroll("screens_scroll", list =>
            {
                var screens = LocalScreenManager.Screens;
                if (screens.Count == 0)
                {
                    list.Card(c => c.Text("No local pickup screens spawned.", fg: "muted"));
                    return;
                }

                for (int i = 0; i < screens.Count; i++)
                {
                    var scr = screens[i];
                    if (scr == null || !scr.IsAlive) continue;

                    string targetName = scr.FollowSelectedPlayer
                        ? "Follows Selected"
                        : (scr.PinnedPlayer != null ? VideoPlayerTracker.GetShortName(scr.PinnedPlayer) : "Unbound");

                    list.Row(row =>
                    {
                        row.Col(col =>
                        {
                            col.Text($"Screen #{scr.Id} ({scr.WidthScale:0.0}m)", size: 12f, weight: "semibold");
                            col.Text($"{targetName}{(scr.GrabLocked ? " • Locked" : "")}", size: 10f, fg: "muted");
                        }, new ExtStyle { Grow = 1f, Gap = 1f });

                        row.Button($"scr_cfg_{scr.Id}", "Configure", () =>
                        {
                            _activeScreenDetail = scr;
                            _tab?.PushSubPage(new ExtSubPageOptions { Id = $"scr_{scr.Id}", Title = $"Screen #{scr.Id}" }, BuildScreenDetailSubPage);
                        }, icon: "tune");

                        row.IconButton($"scr_del_{scr.Id}", "delete", () =>
                        {
                            LocalScreenManager.DespawnScreen(scr);
                        }, variant: "danger", tooltip: "Despawn");
                    }, new ExtStyle { Align = "center", Gap = 6f, Px = 9f, Py = 6f, Bg = "card", Radius = 6f });
                }
            }, new ExtStyle { Grow = 1f, Gap = 6f });
        }

        private static void BuildScreenDetailSubPage(UiBuilder ui)
        {
            var screen = _activeScreenDetail;
            if (screen == null || !screen.IsAlive)
            {
                ui.Card(c => c.Text("This screen no longer exists.", fg: "muted"));
                return;
            }

            ui.Scroll("screen_detail_scroll", list =>
            {
                list.Card(card =>
                {
                    card.Text("Position & Pinning", size: 12f, weight: "semibold", fg: "muted");

                    card.Row(r =>
                    {
                        r.Button("scr_bring", "Bring to Me", () =>
                        {
                            if (!screen.IsAlive) return;
                            screen.BringToPlayer();
                            LumenicApi.ShowToast($"Brought Screen #{screen.Id} to you.", 1.5f);
                        }, icon: "near_me", variant: "accent", style: new ExtStyle { Grow = 1f });

                        r.Button("scr_pin_cur", "Pin to Selected Player", () =>
                        {
                            if (!screen.IsAlive) return;
                            var vp = VideoPlayerTracker.SelectedPlayer;
                            if (vp == null)
                            {
                                LumenicApi.ShowToast("No video player selected.", 1.5f);
                                return;
                            }
                            screen.FollowSelectedPlayer = false;
                            screen.PinnedPlayer = vp;
                            screen.RefreshFeed();
                            MarkDirty();
                            LumenicApi.ShowToast($"Pinned to {VideoPlayerTracker.GetShortName(vp)}", 1.5f);
                        }, icon: "push_pin", style: new ExtStyle { Grow = 1f });
                    }, new ExtStyle { Gap = 6f });

                    card.Toggle("scr_lock", "Lock Grab", screen.GrabLocked, v => { if (screen.IsAlive) { screen.SetGrabLocked(v); MarkDirty(); } }, icon: "lock");
                    card.Toggle("scr_local_pin", "Pin to Local Space", screen.PinToLocalSpace, v => { if (screen.IsAlive) { screen.SetPinToLocalSpace(v); MarkDirty(); } }, icon: "anchor");
                    card.Toggle("scr_playspace", "Follow Playspace", screen.FollowPlayspace, v => { if (screen.IsAlive) { screen.SetFollowPlayspace(v); MarkDirty(); } }, icon: "open_with");
                    card.Toggle("scr_follow_sel", "Follow Selected VideoPlayer", screen.FollowSelectedPlayer, v =>
                    {
                        if (!screen.IsAlive) return;
                        screen.FollowSelectedPlayer = v;
                        if (!v && VideoPlayerTracker.SelectedPlayer != null)
                            screen.PinnedPlayer = VideoPlayerTracker.SelectedPlayer;
                        screen.RefreshFeed();
                        MarkDirty();
                    }, icon: "sync");
                }, new ExtStyle { Gap = 6f, Px = 10f, Py = 8f });

                list.Card(card =>
                {
                    card.Text("Scale & Size Presets", size: 12f, weight: "semibold", fg: "muted");
                    card.Slider("scr_width", $"Width ({screen.WidthScale:0.00}m)", screen.WidthScale, 0.3f, 4.0f,
                        v => { if (screen.IsAlive) { screen.SetScale(v); MarkDirty(); } }, step: 0.05f, style: new ExtStyle { WFrac = 1f });

                    card.Row(r =>
                    {
                        r.Button("scr_06", "0.6m", () => { if (screen.IsAlive) { screen.SetScale(0.6f); MarkDirty(); } }, style: new ExtStyle { Grow = 1f });
                        r.Button("scr_12", "1.2m", () => { if (screen.IsAlive) { screen.SetScale(1.2f); MarkDirty(); } }, style: new ExtStyle { Grow = 1f });
                        r.Button("scr_22", "2.2m", () => { if (screen.IsAlive) { screen.SetScale(2.2f); MarkDirty(); } }, style: new ExtStyle { Grow = 1f });
                        r.Button("scr_35", "3.5m", () => { if (screen.IsAlive) { screen.SetScale(3.5f); MarkDirty(); } }, style: new ExtStyle { Grow = 1f });
                    }, new ExtStyle { Gap = 5f });
                }, new ExtStyle { Gap = 6f, Px = 10f, Py = 8f });

                list.Button("scr_despawn", "Despawn Screen", () =>
                {
                    LocalScreenManager.DespawnScreen(screen);
                    _activeScreenDetail = null;
                    _tab?.PopSubPage();
                }, icon: "delete", variant: "danger");
            }, new ExtStyle { Grow = 1f, Gap = 8f });
        }

        private static void OpenHudSubPage()
        {
            _tab?.PushSubPage(new ExtSubPageOptions { Id = "hud_setup", Title = "HUD / OSD Setup" }, ui =>
            {
                ui.Scroll("hud_scroll", list =>
                {
                    list.Card(card =>
                    {
                        card.Toggle("hud_en", "Enable HUD / OSD", Settings.HudEnabled, v => { HudScreenManager.SetEnabled(v); MarkDirty(); }, icon: "picture_in_picture_alt");
                        card.Toggle("hud_over", "Always On Top (Overdraw)", Settings.HudOverdraw, v => { Settings.HudOverdraw = v; HudScreenManager.ApplyVisualSettings(); MarkDirty(); }, icon: "layers");
                        card.Toggle("hud_smooth", "Smooth Head Follow", Settings.HudSmoothFollow, v => { Settings.HudSmoothFollow = v; MarkDirty(); }, icon: "motion_photos_on");
                    }, new ExtStyle { Gap = 6f, Px = 10f, Py = 8f });

                    list.Card(card =>
                    {
                        card.Text("Position Presets", size: 12f, weight: "semibold", fg: "muted");
                        card.Row(r =>
                        {
                            r.Button("hud_tl", "Top-Left", () => { Settings.HudOffsetX = -0.35f; Settings.HudOffsetY = 0.20f; MarkDirty(); }, style: new ExtStyle { Grow = 1f });
                            r.Button("hud_tr", "Top-Right", () => { Settings.HudOffsetX = 0.35f; Settings.HudOffsetY = 0.20f; MarkDirty(); }, style: new ExtStyle { Grow = 1f });
                            r.Button("hud_bl", "Bot-Left", () => { Settings.HudOffsetX = -0.35f; Settings.HudOffsetY = -0.20f; MarkDirty(); }, style: new ExtStyle { Grow = 1f });
                            r.Button("hud_br", "Bot-Right", () => { Settings.HudOffsetX = 0.35f; Settings.HudOffsetY = -0.20f; MarkDirty(); }, style: new ExtStyle { Grow = 1f });
                        }, new ExtStyle { Gap = 5f });

                        card.Row(r =>
                        {
                            r.Button("hud_center", "Center", () => { Settings.HudOffsetX = 0f; Settings.HudOffsetY = 0f; MarkDirty(); }, style: new ExtStyle { Grow = 1f });
                            r.Button("hud_reset", "Reset HUD", () =>
                            {
                                Settings.HudOpacity = 0.9f;
                                Settings.HudScale = 0.45f;
                                Settings.HudDistance = 1.2f;
                                Settings.HudOffsetX = 0.35f;
                                Settings.HudOffsetY = 0.20f;
                                Settings.HudOverdraw = true;
                                Settings.HudSmoothFollow = true;
                                HudScreenManager.ApplyVisualSettings();
                                MarkDirty();
                            }, variant: "danger", style: new ExtStyle { Grow = 1f });
                        }, new ExtStyle { Gap = 5f });
                    }, new ExtStyle { Gap = 6f, Px = 10f, Py = 8f });

                    list.Card(card =>
                    {
                        card.Text("Appearance & Placement", size: 12f, weight: "semibold", fg: "muted");
                        card.Slider("hud_op", $"Opacity ({Settings.HudOpacity:0.00})", Settings.HudOpacity, 0.1f, 1.0f, v => { Settings.HudOpacity = v; HudScreenManager.ApplyVisualSettings(); MarkDirty(); }, step: 0.05f, style: new ExtStyle { WFrac = 1f });
                        card.Slider("hud_sc", $"Scale ({Settings.HudScale:0.00}m)", Settings.HudScale, 0.2f, 1.5f, v => { Settings.HudScale = v; HudScreenManager.ApplyVisualSettings(); MarkDirty(); }, step: 0.05f, style: new ExtStyle { WFrac = 1f });
                        card.Slider("hud_dist", $"Distance ({Settings.HudDistance:0.00}m)", Settings.HudDistance, 0.5f, 3.0f, v => { Settings.HudDistance = v; MarkDirty(); }, step: 0.05f, style: new ExtStyle { WFrac = 1f });
                        card.Slider("hud_ox", $"Offset X ({Settings.HudOffsetX:0.00})", Settings.HudOffsetX, -1.0f, 1.0f, v => { Settings.HudOffsetX = v; MarkDirty(); }, step: 0.05f, style: new ExtStyle { WFrac = 1f });
                        card.Slider("hud_oy", $"Offset Y ({Settings.HudOffsetY:0.00})", Settings.HudOffsetY, -0.8f, 0.8f, v => { Settings.HudOffsetY = v; MarkDirty(); }, step: 0.05f, style: new ExtStyle { WFrac = 1f });
                    }, new ExtStyle { Gap = 6f, Px = 10f, Py = 8f });
                }, new ExtStyle { Grow = 1f, Gap = 8f });
            });
        }

        // ==================== SETTINGS TAB ====================

        private static void BuildSettingsSection(UiBuilder ui)
        {
            var vp = VideoPlayerTracker.SelectedPlayer;
            bool hasVp = vp != null;

            ui.Scroll("settings_scroll", list =>
            {
                list.Card(card =>
                {
                    card.Text("Selected VideoPlayer Settings", size: 12f, weight: "semibold", fg: "muted");

                    card.Row(r =>
                    {
                        r.Toggle("vp_sync", "Network Sync", hasVp && vp.syncEnabled, v => { vp?.SetNetworkSync(v); MarkDirty(); }, icon: "sync", disabled: !hasVp, style: new ExtStyle { Grow = 1f });
                        r.Toggle("vp_audio_only", "YT Audio Only", hasVp && vp.ytAudioOnly, v => { vp?.SetYTAudioOnly(v); MarkDirty(); }, icon: "headphones", disabled: !hasVp, style: new ExtStyle { Grow = 1f });
                    }, new ExtStyle { Gap = 6f });

                    // Max Resolution
                    string[] resLabels = Resolutions.Select(r => $"{(int)r}p").ToArray();
                    int resIdx = hasVp ? Math.Max(0, Array.IndexOf(Resolutions, vp.maxResolution)) : 0;
                    card.Row(r =>
                    {
                        r.Text("Max Resolution", size: 11f, weight: "medium", style: new ExtStyle { Grow = 1f });
                        r.Dropdown("vp_res", resLabels, resIdx, idx =>
                        {
                            if (vp != null && idx >= 0 && idx < Resolutions.Length)
                            {
                                vp.SetMaxResolution(Resolutions[idx]);
                                MarkDirty();
                            }
                        }, disabled: !hasVp, style: new ExtStyle { W = 180f });
                    }, new ExtStyle { Align = "center", Gap = 8f });

                    // Audio Mode
                    string[] audioLabels = { "Direct", "AudioSource", "RoomScale" };
                    int audioIdx = hasVp ? Math.Max(0, Array.IndexOf(AudioModes, vp.audioPlaybackMode)) : 0;
                    card.Row(r =>
                    {
                        r.Text("Audio Mode", size: 11f, weight: "medium", style: new ExtStyle { Grow = 1f });
                        r.Dropdown("vp_audio_mode", audioLabels, audioIdx, idx =>
                        {
                            if (vp != null && idx >= 0 && idx < AudioModes.Length)
                            {
                                VideoPlayerTracker.SafeSetAudioMode(vp, AudioModes[idx]);
                                MarkDirty();
                            }
                        }, disabled: !hasVp, style: new ExtStyle { W = 180f });
                    }, new ExtStyle { Align = "center", Gap = 8f });

                    // Subtitles
                    GetSubtitleChoices(vp, out var subNames, out var subIds, out int subIdx);
                    card.Row(r =>
                    {
                        r.Text("Subtitles", size: 11f, weight: "medium", style: new ExtStyle { Grow = 1f });
                        r.Dropdown("vp_subs", subNames, subIdx, idx =>
                        {
                            if (vp?.VideoPlayer?.Subtitles != null && idx >= 0 && idx < subIds.Count)
                            {
                                string subId = subIds[idx];
                                vp.VideoPlayer.Subtitles.SelectedSubtitle = subId == "-1" ? string.Empty : subId;
                                MarkDirty();
                            }
                        }, disabled: !hasVp, style: new ExtStyle { W = 180f });
                    }, new ExtStyle { Align = "center", Gap = 8f });

                    // Control Permission
                    GetLockChoices(vp, out var lockLabels, out var lockOpts, out int lockIdx);
                    card.Row(r =>
                    {
                        r.Text("Control Permission", size: 11f, weight: "medium", style: new ExtStyle { Grow = 1f });
                        r.Dropdown("vp_lock", lockLabels, lockIdx, idx =>
                        {
                            if (vp != null && VideoPlayerTracker.CanControl(vp) && idx >= 0 && idx < lockOpts.Count)
                            {
                                vp.SetControlPermission(lockOpts[idx]);
                                MarkDirty();
                            }
                        }, disabled: !hasVp || lockOpts.Count <= 1, style: new ExtStyle { W = 180f });
                    }, new ExtStyle { Align = "center", Gap = 8f });
                }, new ExtStyle { Gap = 6f, Px = 10f, Py = 8f });

                list.Card(card =>
                {
                    card.Text("Remote Mod Settings", size: 12f, weight: "semibold", fg: "muted");
                    card.Toggle("mod_autoselect", "Auto-Select Nearest VideoPlayer", Settings.AutoSelectNearest, v => { Settings.AutoSelectNearest = v; MarkDirty(); }, icon: "my_location");
                    card.Slider("mod_seekstep", $"Seek Step ({Settings.SeekStepSeconds:0}s)", Settings.SeekStepSeconds, 5f, 120f, v => { Settings.SeekStepSeconds = v; MarkDirty(); }, step: 5f, style: new ExtStyle { WFrac = 1f });
                    card.Slider("mod_defscale", $"Default Screen Width ({Settings.DefaultScreenScale:0.0}m)", Settings.DefaultScreenScale, 0.3f, 4.0f, v => { Settings.DefaultScreenScale = v; MarkDirty(); }, step: 0.1f, style: new ExtStyle { WFrac = 1f });
                    card.Toggle("mod_follow", "New Screens Follow Selected Player", Settings.DefaultFollowSelected, v => { Settings.DefaultFollowSelected = v; MarkDirty(); }, icon: "sync");
                }, new ExtStyle { Gap = 6f, Px = 10f, Py = 8f });
            }, new ExtStyle { Grow = 1f, Gap = 8f });
        }

        // ==================== PLAYER SELECT SUBPAGE ====================

        private static void OpenPlayerSelectSubPage()
        {
            VideoPlayerTracker.RefreshFromScene();
            _tab?.PushSubPage(new ExtSubPageOptions { Id = "player_select", Title = "Select VideoPlayer" }, ui =>
            {
                ui.Row(r =>
                {
                    r.Button("ps_nearest", "Select Nearest", () =>
                    {
                        var nearest = VideoPlayerTracker.SelectNearest();
                        if (nearest != null)
                        {
                            LumenicApi.ShowToast($"Selected: {VideoPlayerTracker.GetShortName(nearest)}", 1.5f);
                            _tab?.PopSubPage();
                        }
                        else
                        {
                            LumenicApi.ShowToast("No video players found.", 1.5f);
                        }
                    }, icon: "my_location", variant: "accent", style: new ExtStyle { Grow = 1f });

                    r.Button("ps_refresh", "Refresh", () =>
                    {
                        VideoPlayerTracker.RefreshFromScene();
                        MarkDirty();
                    }, icon: "refresh");
                }, new ExtStyle { Gap = 6f });

                ui.Scroll("ps_scroll", list =>
                {
                    var players = VideoPlayerTracker.Players;
                    if (players.Count == 0)
                    {
                        list.Card(c => c.Text("No VideoPlayers found in this instance.", fg: "muted"));
                        return;
                    }

                    for (int i = 0; i < players.Count; i++)
                    {
                        var vp = players[i];
                        if (!VideoPlayerTracker.IsValid(vp)) continue;

                        bool isSelected = vp == VideoPlayerTracker.SelectedPlayer;
                        string typeTag = VideoPlayerTracker.IsPropPlayer(vp) ? "Prop" : "World";
                        float dist = VideoPlayerTracker.GetDistanceToLocalPlayer(vp);
                        string distStr = dist < 9999f ? $"{dist:0.0}m" : "?";
                        string nowTitle = VideoPlayerTracker.GetCurrentTitle(vp);

                        list.Row(row =>
                        {
                            row.Col(col =>
                            {
                                col.Row(tr =>
                                {
                                    tr.Text(VideoPlayerTracker.GetShortName(vp), size: 12f, weight: "bold", ellipsis: true, style: new ExtStyle { Grow = 1f });
                                    tr.Badge($"{typeTag} • {distStr}", fg: isSelected ? "emerald" : "muted");
                                }, new ExtStyle { Align = "center", Gap = 6f });
                                col.Text(nowTitle, size: 10f, fg: "muted", ellipsis: true);
                            }, new ExtStyle { Grow = 1f, Gap = 2f });

                            row.Button($"ps_pick_{i}", isSelected ? "Selected" : "Select", () =>
                            {
                                VideoPlayerTracker.SelectPlayer(vp);
                                _tab?.PopSubPage();
                            }, variant: isSelected ? "emerald" : "accent");
                        }, new ExtStyle { Align = "center", Gap = 6f, Px = 9f, Py = 6f, Bg = "card", Radius = 6f });
                    }
                }, new ExtStyle { Grow = 1f, Gap = 6f });
            });
        }

        // ==================== MODALS & HELPERS ====================

        private static void OpenPickPlaylistModal(SavedVideoItem item)
        {
            if (item == null) return;
            LumenicApi.OpenModal(ExtSurface.QuickMenu, ModId, new ExtModalOptions
            {
                Id = "pick_playlist",
                Title = "Add to Playlist",
                Width = 420f
            }, ui =>
            {
                ui.Text(item.DisplayTitle, size: 11.5f, weight: "semibold", ellipsis: true);
                var playlists = LibraryStore.Data.Playlists;
                if (playlists.Count == 0)
                {
                    ui.Text("No custom playlists created yet.", fg: "muted");
                }
                else
                {
                    ui.Scroll("pick_pl_scroll", list =>
                    {
                        for (int i = 0; i < playlists.Count; i++)
                        {
                            var pl = playlists[i];
                            list.Button($"pick_pl_{i}", $"{pl.Name} ({pl.Items.Count})", () =>
                            {
                                LibraryStore.AddToPlaylist(pl, item.DisplayTitle, item.Url);
                                LumenicApi.CloseModal(ExtSurface.QuickMenu);
                                LumenicApi.ShowToast($"Added to \"{pl.Name}\"", 1.5f);
                            }, icon: "playlist_add");
                        }
                    }, new ExtStyle { MaxH = 240f, Gap = 5f });
                }

                ui.Row(r =>
                {
                    r.Button("pick_pl_close", "Close", () => LumenicApi.CloseModal(ExtSurface.QuickMenu));
                }, new ExtStyle { Justify = "end" });
            });
        }

        private static void HandleUntrusted(VideoPlayerUtils.UntrustedVideoAction action)
        {
            var vp = VideoPlayerTracker.SelectedPlayer;
            if (vp == null) return;
            vp.HandleUntrustedVideo(action);
            MarkDirty();
        }

        private static void TogglePlayPause()
        {
            var vp = VideoPlayerTracker.SelectedPlayer;
            if (vp == null) return;

            var state = VideoPlayerTracker.GetState(vp);
            bool isPlaying = state == VideoPlayerUtils.PlayerState.Playing ||
                             state == VideoPlayerUtils.PlayerState.Buffering ||
                             state == VideoPlayerUtils.PlayerState.Seeking;

            if (isPlaying)
                vp.Pause();
            else
                vp.Play();

            MarkDirty();
        }

        private static void HandleUrlOrSearchInput(string text)
        {
            if (string.IsNullOrWhiteSpace(text))
                return;

            string trimmed = text.Trim();
            if (VideoPlayerTracker.IsHttpUrl(trimmed))
            {
                var vp = VideoPlayerTracker.SelectedPlayer;
                if (vp == null)
                {
                    LumenicApi.ShowToast("No video player selected.", 2.0f);
                    return;
                }
                VideoPlayerTracker.PlayUrl(vp, trimmed);
                LumenicApi.ShowToast("Sent URL to video player.", 2.0f);
                MarkDirty();
                return;
            }

            if (VideoPlayerTracker.IsSearchInFlight)
            {
                LumenicApi.ShowToast("Search already in progress...", 2.0f);
                return;
            }

            string displayQuery = trimmed.Length > 28 ? trimmed.Substring(0, 25) + "..." : trimmed;
            LumenicApi.ShowToast($"Searching YouTube for \"{displayQuery}\"...", 2.0f);
            VideoPlayerTracker.SearchYouTube(trimmed, (results, term) =>
            {
                _lastSearchResults = results ?? new List<SavedVideoItem>();
                _lastSearchTerm = term;
                _mainSection = 1;
                _librarySection = 3;
                MarkDirty();
            });
        }

        private static void PlayItem(SavedVideoItem item)
        {
            if (item == null || string.IsNullOrWhiteSpace(item.Url)) return;
            var vp = VideoPlayerTracker.SelectedPlayer;
            if (vp == null)
            {
                LumenicApi.ShowToast("No video player selected.", 2.0f);
                return;
            }
            VideoPlayerTracker.PlayUrl(vp, item.Url);
            LumenicApi.ShowToast($"Playing: {item.DisplayTitle}", 2.0f);
            _mainSection = 0;
            MarkDirty();
        }

        private static void FavoriteCurrentVideo()
        {
            var vp = VideoPlayerTracker.SelectedPlayer;
            if (vp == null)
            {
                LumenicApi.ShowToast("No video player selected.", 2.0f);
                return;
            }

            string url = VideoPlayerTracker.GetCurrentUrl(vp);
            if (string.IsNullOrWhiteSpace(url))
            {
                LumenicApi.ShowToast("No video is currently playing.", 2.0f);
                return;
            }

            if (LibraryStore.IsFavorite(url))
            {
                LibraryStore.RemoveFavorite(url);
                LumenicApi.ShowToast("Removed from Favorites.", 1.5f);
            }
            else
            {
                string title = VideoPlayerTracker.GetCurrentTitle(vp);
                LibraryStore.AddFavorite(title, url);
                LumenicApi.ShowToast("Added to Favorites!", 1.5f);
            }
            MarkDirty();
        }

        private static void SpawnWithPreset(float scale)
        {
            var screen = LocalScreenManager.SpawnScreen(scale: scale);
            if (screen != null)
            {
                LumenicApi.ShowToast($"Spawned Screen #{screen.Id} ({scale:0.0}m)", 2.0f);
            }
        }

        private static void GetSubtitleChoices(CVRVideoPlayer vp, out List<string> names, out List<string> ids, out int selectedIdx)
        {
            names = new List<string> { "None" };
            ids = new List<string> { "-1" };
            selectedIdx = 0;

            if (vp?.VideoPlayer?.Subtitles != null)
            {
                try
                {
                    string currentSub = vp.VideoPlayer.Subtitles.SelectedSubtitle ?? string.Empty;
                    var subs = vp.VideoPlayer.Subtitles.GetAvailableSubtitles();
                    if (subs != null)
                    {
                        foreach (var s in subs)
                        {
                            names.Add(string.IsNullOrWhiteSpace(s.SubtitleName) ? s.SubtitleId : s.SubtitleName);
                            ids.Add(s.SubtitleId ?? "-1");
                            if (!string.IsNullOrEmpty(currentSub) && s.SubtitleId == currentSub)
                                selectedIdx = ids.Count - 1;
                        }
                    }
                }
                catch { }
            }
        }

        private static void GetLockChoices(CVRVideoPlayer vp, out List<string> labels, out List<VideoPlayerLockOption> opts, out int selectedIdx)
        {
            labels = new List<string>();
            opts = new List<VideoPlayerLockOption>();
            selectedIdx = 0;

            if (vp == null)
            {
                labels.Add("None");
                opts.Add(VideoPlayerLockOption.None);
                return;
            }

            var current = vp.ControlPermission;
            var available = VideoPlayerTracker.GetAvailableLockOptions(vp);
            var ourLevel = VideoPlayerTracker.GetOurLockLevel(vp);

            foreach (VideoPlayerLockOption opt in Enum.GetValues(typeof(VideoPlayerLockOption)))
            {
                if (opt == VideoPlayerLockOption.None) continue;
                if ((((available & opt) != 0) && (int)opt <= (int)ourLevel) || opt == current)
                {
                    opts.Add(opt);
                    labels.Add(VideoPlayerTracker.GetLockOptionLabel(opt));
                }
            }

            if (opts.Count == 0)
            {
                opts.Add(current);
                labels.Add(VideoPlayerTracker.GetLockOptionLabel(current));
            }

            selectedIdx = Math.Max(0, opts.IndexOf(current));
        }

        private static string FormatTime(double seconds)
        {
            if (double.IsNaN(seconds) || double.IsInfinity(seconds) || seconds < 0)
                return "0:00";

            int total = (int)seconds;
            int hrs = total / 3600;
            int mins = (total % 3600) / 60;
            int secs = total % 60;
            return hrs > 0 ? $"{hrs}:{mins:D2}:{secs:D2}" : $"{mins}:{secs:D2}";
        }
    }
}
