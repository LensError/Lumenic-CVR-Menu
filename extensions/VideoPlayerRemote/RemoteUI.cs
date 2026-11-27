using System;
using System.Collections.Generic;
using System.Linq;
using ABI.CCK.Components;
using ABI_RC.Core.InteractionSystem;
using ABI_RC.VideoPlayer.Scripts;
using ABI_RC.VideoPlayer.Scripts.Players;
using BTKUILib;
using BTKUILib.UIObjects;
using BTKUILib.UIObjects.Components;
using BTKUILib.UIObjects.Objects;
using GameCommon.Models.VideoPlayer;
using UnityEngine;

namespace VideoPlayerRemote
{
    internal static class RemoteUI
    {
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

        private static Page _rootPage;
        private static Page _playerSelectPage;
        private static Page _screensPage;
        private static Page _screenDetailPage;
        private static Page _hudPage;
        private static Page _searchResultsPage;
        private static Page _favoritesPage;
        private static Page _playlistsPage;
        private static Page _playlistDetailPage;
        private static Page _nativePlaylistsPage;
        private static Page _playerSettingsPage;
        private static Page _videoActionPage;
        private static Page _pickPlaylistPage;

        // Main remote dynamic elements
        private static TextBlock _targetInfoText;
        private static TextBlock _nowPlayingText;
        private static Category _untrustedCategory;
        private static Button _playPauseButton;
        private static SliderFloat _seekSlider;
        private static SliderFloat _volumeSlider;
        private static ToggleButton _muteToggle;
        private static SliderFloat _speedSlider;
        private static ToggleButton _watchingToggle;
        private static ToggleButton _mainHudToggle;
        private static Button _favoriteCurrentButton;

        // HUD setup dynamic elements
        private static ToggleButton _hudPageEnableToggle;
        private static ToggleButton _hudOverdrawToggle;
        private static ToggleButton _hudSmoothToggle;
        private static SliderFloat _hudOpacitySlider;
        private static SliderFloat _hudScaleSlider;
        private static SliderFloat _hudDistanceSlider;
        private static SliderFloat _hudOffsetXSlider;
        private static SliderFloat _hudOffsetYSlider;

        // Player settings dynamic elements
        private static ToggleButton _networkSyncToggle;
        private static ToggleButton _audioOnlyToggle;
        private static Button _resolutionButton;
        private static Button _audioModeButton;
        private static Button _subtitleButton;
        private static Button _lockPermButton;

        private static MultiSelection _resolutionSelect;
        private static MultiSelection _audioModeSelect;
        private static MultiSelection _subtitleSelect;
        private static MultiSelection _lockPermSelect;

        private static readonly List<string> _subtitleIds = new List<string>();
        private static readonly List<VideoPlayerLockOption> _lockOptions = new List<VideoPlayerLockOption>();

        // State for subpages
        private static LocalScreenInstance _activeScreenDetail;
        private static SavedPlaylist _activePlaylistDetail;
        private static SavedVideoItem _activeVideoActionItem;
        private static List<SavedVideoItem> _lastSearchResults = new List<SavedVideoItem>();
        private static string _lastSearchTerm = "";
        private static float _savedVolumeBeforeMute = 0.8f;

        private static bool _suppressControlEvents;
        private static bool _untrustedShown;

        public static void Init()
        {
            BuildRootPage();
            BuildSubPages();

            VideoPlayerTracker.OnPlayersListChanged += RebuildPlayerSelectPage;
            VideoPlayerTracker.OnSelectedPlayerChanged += HandleSelectedPlayerChanged;
            LocalScreenManager.OnScreensChanged += RebuildScreensPage;
            LibraryStore.OnLibraryChanged += HandleLibraryChanged;
            QuickMenuAPI.OnMenuGenerated += HandleMenuGenerated;
        }

        public static void Shutdown()
        {
            VideoPlayerTracker.OnPlayersListChanged -= RebuildPlayerSelectPage;
            VideoPlayerTracker.OnSelectedPlayerChanged -= HandleSelectedPlayerChanged;
            LocalScreenManager.OnScreensChanged -= RebuildScreensPage;
            LibraryStore.OnLibraryChanged -= HandleLibraryChanged;
            QuickMenuAPI.OnMenuGenerated -= HandleMenuGenerated;
        }

        private static void HandleSelectedPlayerChanged(CVRVideoPlayer _)
        {
            RefreshMainRemote();
            RefreshPlayerSettings();
            RebuildNativePlaylistsPage();
        }

        private static void HandleLibraryChanged()
        {
            RebuildFavoritesPage();
            RebuildPlaylistsPage();
            RebuildPlaylistDetailPage();
        }

        private static void HandleMenuGenerated(CVR_MenuManager _)
        {
            RefreshMainRemote();
            RefreshPlayerSettings();
            RebuildPlayerSelectPage();
            RebuildScreensPage();
            RebuildFavoritesPage();
            RebuildPlaylistsPage();
            RebuildNativePlaylistsPage();
        }

        public static void Tick()
        {
            if (_rootPage == null || !_rootPage.IsGenerated || !_rootPage.IsVisible)
                return;

            RefreshMainRemote();
        }

        private static void BuildRootPage()
        {
            _rootPage = new Page(
                VideoPlayerRemoteMod.IconModName,
                "Main",
                isRootPage: true,
                tabIcon: VideoPlayerRemoteMod.TabIconName);
            _rootPage.MenuTitle = "Video Remote";
            _rootPage.MenuSubtitle = "Control world & prop video players";
            _rootPage.OnPageOpen += RefreshMainRemote;

            // Target & Status
            var statusCat = _rootPage.AddCategory("Now Playing");
            _targetInfoText = statusCat.AddTextBlock("Target: No Video Player Selected");
            _nowPlayingText = statusCat.AddTextBlock("Idle");

            // Untrusted URL quick banner (hidden unless pending)
            _untrustedCategory = _rootPage.AddCategory("Untrusted URL Pending");
            _untrustedCategory.AddButton("Watch Once", "", "Play this untrusted URL once", ButtonStyle.TextOnly)
                .OnPress += () => HandleUntrusted(VideoPlayerUtils.UntrustedVideoAction.WatchOnce);
            _untrustedCategory.AddButton("Allow Session", "", "Trust this domain for this session", ButtonStyle.TextOnly)
                .OnPress += () => HandleUntrusted(VideoPlayerUtils.UntrustedVideoAction.WatchAndAllowForSession);
            _untrustedCategory.AddButton("Trust Domain", "", "Permanently trust this domain", ButtonStyle.TextOnly)
                .OnPress += () => HandleUntrusted(VideoPlayerUtils.UntrustedVideoAction.WatchAndTrustDomain);
            _untrustedCategory.AddButton("Deny", "", "Deny and stop watching", ButtonStyle.TextOnly)
                .OnPress += () => HandleUntrusted(VideoPlayerUtils.UntrustedVideoAction.Deny);
            _untrustedCategory.Hidden = true;

            // Transport (4-wide intuitive rows)
            var transportCat = _rootPage.AddCategory("Transport");
            // Row 1: Prev | Play/Pause | Stop | Next
            transportCat.AddButton("Prev", "", "Previous track in playlist", ButtonStyle.TextOnly)
                .OnPress += VideoPlayerTracker.PreviousTrack;

            _playPauseButton = transportCat.AddButton("Play / Pause", "", "Toggle playback", ButtonStyle.TextOnly);
            _playPauseButton.OnPress += TogglePlayPause;

            transportCat.AddButton("Stop", "", "Clear current video URL", ButtonStyle.TextOnly)
                .OnPress += () =>
                {
                    var vp = VideoPlayerTracker.SelectedPlayer;
                    if (vp == null) return;
                    VideoPlayerTracker.ClearCustomPlaylist();
                    vp.ClearVideoUrl(true, null);
                    RefreshMainRemote();
                };

            transportCat.AddButton("Next", "", "Next track in playlist", ButtonStyle.TextOnly)
                .OnPress += VideoPlayerTracker.NextTrack;

            // Row 2: -15s | +15s | [blank] | Resync
            transportCat.AddButton("-15s", "", "Seek backward", ButtonStyle.TextOnly)
                .OnPress += () =>
                {
                    var vp = VideoPlayerTracker.SelectedPlayer;
                    if (vp != null) VideoPlayerTracker.SeekRelative(vp, -Settings.SeekStepSeconds);
                };

            transportCat.AddButton("+15s", "", "Seek forward", ButtonStyle.TextOnly)
                .OnPress += () =>
                {
                    var vp = VideoPlayerTracker.SelectedPlayer;
                    if (vp != null) VideoPlayerTracker.SeekRelative(vp, Settings.SeekStepSeconds);
                };

            AddBlank(transportCat);

            transportCat.AddButton("Resync", "", "Re-subscribe locally to resync playback", ButtonStyle.TextOnly)
                .OnPress += () =>
                {
                    var vp = VideoPlayerTracker.SelectedPlayer;
                    if (vp == null) return;
                    string url = VideoPlayerTracker.GetCurrentUrl(vp);
                    if (!string.IsNullOrEmpty(url))
                    {
                        vp.SetWatching(false);
                        vp.SetWatching(true);
                        QuickMenuAPI.ShowAlertToast("Resyncing VideoPlayer...", 2);
                    }
                };

            _seekSlider = transportCat.AddSlider("Seek (sec)", "Scrub timeline", 0f, 0f, 100f, decimalPlaces: 0);
            _seekSlider.OnValueUpdated += val =>
            {
                if (_suppressControlEvents) return;
                var vp = VideoPlayerTracker.SelectedPlayer;
                if (vp != null) VideoPlayerTracker.RequestSeek(vp, val);
            };

            // Input & Quick Actions (4-wide row)
            var inputCat = _rootPage.AddCategory("URL, Search & Quick Actions");
            inputCat.AddButton("URL / Search", "", "Enter a video URL or search YouTube", ButtonStyle.TextOnly)
                .OnPress += OpenUrlOrSearchKeyboard;

            inputCat.AddButton("Paste & Play", "", "Play URL or search from clipboard", ButtonStyle.TextOnly)
                .OnPress += () => HandleUrlOrSearchInput(GUIUtility.systemCopyBuffer);

            _favoriteCurrentButton = inputCat.AddButton("Favorite Current", "", "Save currently playing video to Favorites", ButtonStyle.TextOnly);
            _favoriteCurrentButton.OnPress += FavoriteCurrentVideo;

            inputCat.AddButton("Spawn Screen", "", "Spawn a local grabbable screen in front of you", ButtonStyle.TextOnly)
                .OnPress += () =>
                {
                    var screen = LocalScreenManager.SpawnScreen();
                    if (screen != null)
                        QuickMenuAPI.ShowAlertToast($"Spawned Screen #{screen.Id}", 2);
                };

            // Local Audio & Quick Toggles (4-wide row + sliders)
            var localCat = _rootPage.AddCategory("Audio & Quick Toggles");
            _watchingToggle = localCat.AddToggle("Watching", "Subscribe/unsubscribe from selected VideoPlayer", true);
            _watchingToggle.OnValueUpdated += watching =>
            {
                if (_suppressControlEvents) return;
                var vp = VideoPlayerTracker.SelectedPlayer;
                if (vp != null) vp.SetWatching(watching);
            };

            _muteToggle = localCat.AddToggle("Muted", "Mute or unmute selected VideoPlayer", false);
            _muteToggle.OnValueUpdated += muted =>
            {
                if (_suppressControlEvents) return;
                var vp = VideoPlayerTracker.SelectedPlayer;
                if (vp == null) return;
                if (muted)
                {
                    if (vp.playbackVolume > 0.001f)
                        _savedVolumeBeforeMute = vp.playbackVolume;
                    VideoPlayerTracker.SafeSetVolume(vp, 0f);
                }
                else
                {
                    VideoPlayerTracker.SafeSetVolume(vp, _savedVolumeBeforeMute > 0.01f ? _savedVolumeBeforeMute : 0.8f);
                }
                RefreshMainRemote();
            };

            _mainHudToggle = localCat.AddToggle("HUD / OSD", "Toggle 3D on-screen display", Settings.HudEnabled);
            _mainHudToggle.OnValueUpdated += enabled =>
            {
                if (_suppressControlEvents) return;
                HudScreenManager.SetEnabled(enabled);
                RefreshHudPage();
            };

            AddBlank(localCat);

            _volumeSlider = localCat.AddSlider("Volume", "Local VideoPlayer volume", 1f, 0f, 1f, decimalPlaces: 2, defaultValue: 1f, allowReset: true);
            _volumeSlider.OnValueUpdated += val =>
            {
                if (_suppressControlEvents) return;
                var vp = VideoPlayerTracker.SelectedPlayer;
                if (vp == null) return;
                if (val > 0.001f) _savedVolumeBeforeMute = val;
                VideoPlayerTracker.SafeSetVolume(vp, val);
                UpdateMuteToggleVisual(val <= 0.001f);
            };

            _speedSlider = localCat.AddSlider("Speed", "Local playback speed (0.5x - 2.0x)", 1f, 0.5f, 2f, decimalPlaces: 2, defaultValue: 1f, allowReset: true);
            _speedSlider.OnValueUpdated += val =>
            {
                if (_suppressControlEvents) return;
                var vp = VideoPlayerTracker.SelectedPlayer;
                if (vp != null) vp.localPlaybackSpeed = Mathf.Clamp(val, 0.5f, 2f);
            };

            // Navigation (2 clean 4-wide rows)
            var navCat = _rootPage.AddCategory("Menus");
            // Row 1: Select VideoPlayer | Local Screens | HUD / OSD Setup | VideoPlayer Settings
            _playerSelectPage = navCat.AddPage("Select VideoPlayer", "", "Choose world or prop VideoPlayer", VideoPlayerRemoteMod.IconModName);
            _playerSelectPage.MenuTitle = "Select VideoPlayer";
            _playerSelectPage.OnPageOpen += () =>
            {
                VideoPlayerTracker.RefreshFromScene();
                RebuildPlayerSelectPage();
            };

            _screensPage = navCat.AddPage("Local Screens", "", "Spawn and customize local pickup screens", VideoPlayerRemoteMod.IconModName);
            _screensPage.MenuTitle = "Local Pickup Screens";
            _screensPage.OnPageOpen += RebuildScreensPage;

            _hudPage = navCat.AddPage("HUD / OSD Setup", "", "Configure 3D on-screen display, opacity, position, and overdraw", VideoPlayerRemoteMod.IconModName);
            _hudPage.MenuTitle = "HUD / On-Screen Display";
            _hudPage.OnPageOpen += RefreshHudPage;

            _playerSettingsPage = navCat.AddPage("VideoPlayer Settings", "", "Sync, resolution, audio mode, subtitles, and locks", VideoPlayerRemoteMod.IconModName);
            _playerSettingsPage.MenuTitle = "VideoPlayer Settings";
            _playerSettingsPage.OnPageOpen += RefreshPlayerSettings;

            // Row 2: Favorites | Playlists | VideoPlayer Playlists | [blank]
            _favoritesPage = navCat.AddPage("Favorites", "", "Manage your saved favorite videos", VideoPlayerRemoteMod.IconModName);
            _favoritesPage.MenuTitle = "Favorite Videos";
            _favoritesPage.OnPageOpen += RebuildFavoritesPage;

            _playlistsPage = navCat.AddPage("Playlists", "", "Create and play custom video playlists", VideoPlayerRemoteMod.IconModName);
            _playlistsPage.MenuTitle = "Custom Playlists";
            _playlistsPage.OnPageOpen += RebuildPlaylistsPage;

            _nativePlaylistsPage = navCat.AddPage("VideoPlayer Playlists", "", "Playlists built into this world or prop VideoPlayer", VideoPlayerRemoteMod.IconModName);
            _nativePlaylistsPage.MenuTitle = "Built-in VideoPlayer Playlists";
            _nativePlaylistsPage.OnPageOpen += RebuildNativePlaylistsPage;

            AddBlank(navCat);
        }

        private static void BuildSubPages()
        {
            // Non-root helper pages under the same mod namespace
            _screenDetailPage = new Page(VideoPlayerRemoteMod.IconModName, "ScreenDetail", isRootPage: false);
            _screenDetailPage.MenuTitle = "Screen Settings";

            _searchResultsPage = new Page(VideoPlayerRemoteMod.IconModName, "SearchResults", isRootPage: false);
            _searchResultsPage.MenuTitle = "YouTube Search Results";

            _playlistDetailPage = new Page(VideoPlayerRemoteMod.IconModName, "PlaylistDetail", isRootPage: false);
            _playlistDetailPage.MenuTitle = "Playlist";

            _videoActionPage = new Page(VideoPlayerRemoteMod.IconModName, "VideoAction", isRootPage: false);
            _videoActionPage.MenuTitle = "Video Options";

            _pickPlaylistPage = new Page(VideoPlayerRemoteMod.IconModName, "PickPlaylist", isRootPage: false);
            _pickPlaylistPage.MenuTitle = "Add to Playlist";

            BuildPlayerSettingsControls();
            BuildHudPageControls();
        }

        private static void BuildHudPageControls()
        {
            var topCat = _hudPage.AddCategory("HUD / On-Screen Display");
            _hudPageEnableToggle = topCat.AddToggle("Enable HUD", "Show 3D HUD screen in your view", Settings.HudEnabled);
            _hudPageEnableToggle.OnValueUpdated += enabled =>
            {
                if (_suppressControlEvents) return;
                HudScreenManager.SetEnabled(enabled);
                RefreshMainRemote();
            };

            _hudOverdrawToggle = topCat.AddToggle("Overdraw (On Top)", "Draw over world geometry while keeping 3D stereo depth", Settings.HudOverdraw);
            _hudOverdrawToggle.OnValueUpdated += overdraw =>
            {
                if (_suppressControlEvents) return;
                Settings.HudOverdraw = overdraw;
                HudScreenManager.ApplyVisualSettings();
            };

            _hudSmoothToggle = topCat.AddToggle("Smooth Follow", "Gentle head-turn smoothing for VR comfort", Settings.HudSmoothFollow);
            _hudSmoothToggle.OnValueUpdated += smooth =>
            {
                if (_suppressControlEvents) return;
                Settings.HudSmoothFollow = smooth;
            };

            topCat.AddButton("Reset HUD", "", "Reset HUD position, scale, and opacity to defaults", ButtonStyle.TextOnly)
                .OnPress += () =>
                {
                    Settings.HudOpacity = 0.9f;
                    Settings.HudScale = 0.45f;
                    Settings.HudDistance = 1.2f;
                    Settings.HudOffsetX = 0.35f;
                    Settings.HudOffsetY = 0.20f;
                    Settings.HudOverdraw = true;
                    Settings.HudSmoothFollow = true;
                    HudScreenManager.ApplyVisualSettings();
                    RefreshHudPage();
                };

            var posCat = _hudPage.AddCategory("Quick Position Presets");
            // Row 1: Top-Left | Top-Right | Bottom-Left | Bottom-Right
            posCat.AddButton("Top-Left", "", "Place HUD in top-left of view", ButtonStyle.TextOnly)
                .OnPress += () => ApplyHudPreset(-0.35f, 0.20f);
            posCat.AddButton("Top-Right", "", "Place HUD in top-right of view", ButtonStyle.TextOnly)
                .OnPress += () => ApplyHudPreset(0.35f, 0.20f);
            posCat.AddButton("Bottom-Left", "", "Place HUD in bottom-left of view", ButtonStyle.TextOnly)
                .OnPress += () => ApplyHudPreset(-0.35f, -0.20f);
            posCat.AddButton("Bottom-Right", "", "Place HUD in bottom-right of view", ButtonStyle.TextOnly)
                .OnPress += () => ApplyHudPreset(0.35f, -0.20f);

            // Row 2: [blank] | Center | [blank] | [blank]
            AddBlank(posCat);
            posCat.AddButton("Center", "", "Center HUD in front of view", ButtonStyle.TextOnly)
                .OnPress += () => ApplyHudPreset(0.0f, 0.0f);
            AddBlank(posCat);
            AddBlank(posCat);

            var adjCat = _hudPage.AddCategory("Appearance & Position");
            _hudOpacitySlider = adjCat.AddSlider("Opacity", "Adjust HUD transparency", Settings.HudOpacity, 0.1f, 1.0f, decimalPlaces: 2, defaultValue: 0.9f, allowReset: true);
            _hudOpacitySlider.OnValueUpdated += val =>
            {
                if (_suppressControlEvents) return;
                Settings.HudOpacity = val;
                HudScreenManager.ApplyVisualSettings();
            };

            _hudScaleSlider = adjCat.AddSlider("Scale (m)", "HUD screen width", Settings.HudScale, 0.2f, 1.5f, decimalPlaces: 2, defaultValue: 0.45f, allowReset: true);
            _hudScaleSlider.OnValueUpdated += val =>
            {
                if (_suppressControlEvents) return;
                Settings.HudScale = val;
                HudScreenManager.ApplyVisualSettings();
            };

            _hudDistanceSlider = adjCat.AddSlider("Distance (m)", "3D depth distance from eyes", Settings.HudDistance, 0.5f, 3.0f, decimalPlaces: 2, defaultValue: 1.2f, allowReset: true);
            _hudDistanceSlider.OnValueUpdated += val =>
            {
                if (_suppressControlEvents) return;
                Settings.HudDistance = val;
            };

            _hudOffsetXSlider = adjCat.AddSlider("Offset X", "Horizontal position (-Left / +Right)", Settings.HudOffsetX, -1.0f, 1.0f, decimalPlaces: 2, defaultValue: 0.35f, allowReset: true);
            _hudOffsetXSlider.OnValueUpdated += val =>
            {
                if (_suppressControlEvents) return;
                Settings.HudOffsetX = val;
            };

            _hudOffsetYSlider = adjCat.AddSlider("Offset Y", "Vertical position (-Down / +Up)", Settings.HudOffsetY, -0.8f, 0.8f, decimalPlaces: 2, defaultValue: 0.20f, allowReset: true);
            _hudOffsetYSlider.OnValueUpdated += val =>
            {
                if (_suppressControlEvents) return;
                Settings.HudOffsetY = val;
            };
        }

        private static void ApplyHudPreset(float x, float y)
        {
            Settings.HudOffsetX = x;
            Settings.HudOffsetY = y;
            RefreshHudPage();
        }

        private static void RefreshHudPage()
        {
            if (_hudPageEnableToggle == null)
                return;

            _suppressControlEvents = true;
            try
            {
                _hudPageEnableToggle.ToggleValue = Settings.HudEnabled;
                _hudOverdrawToggle.ToggleValue = Settings.HudOverdraw;
                _hudSmoothToggle.ToggleValue = Settings.HudSmoothFollow;
                _hudOpacitySlider.SetSliderValue(Settings.HudOpacity);
                _hudScaleSlider.SetSliderValue(Settings.HudScale);
                _hudDistanceSlider.SetSliderValue(Settings.HudDistance);
                _hudOffsetXSlider.SetSliderValue(Settings.HudOffsetX);
                _hudOffsetYSlider.SetSliderValue(Settings.HudOffsetY);
            }
            finally
            {
                _suppressControlEvents = false;
            }
        }

        private static void BuildPlayerSettingsControls()
        {
            var cat = _playerSettingsPage.AddCategory("Selected VideoPlayer Settings");

            _networkSyncToggle = cat.AddToggle("Network Sync", "Sync playback with other players", true);
            _networkSyncToggle.OnValueUpdated += sync =>
            {
                if (_suppressControlEvents) return;
                VideoPlayerTracker.SelectedPlayer?.SetNetworkSync(sync);
            };

            _audioOnlyToggle = cat.AddToggle("YT Audio Only", "Stream audio only from YouTube", false);
            _audioOnlyToggle.OnValueUpdated += audioOnly =>
            {
                if (_suppressControlEvents) return;
                VideoPlayerTracker.SelectedPlayer?.SetYTAudioOnly(audioOnly);
            };

            _resolutionSelect = new MultiSelection(
                "Max Resolution",
                Resolutions.Select(r => $"{(int)r}p").ToArray(),
                selectedOption: Array.IndexOf(Resolutions, VideoPlayerUtils.Resolution.P720));
            _resolutionSelect.OnOptionUpdated += idx =>
            {
                if (_suppressControlEvents || idx < 0 || idx >= Resolutions.Length) return;
                var vp = VideoPlayerTracker.SelectedPlayer;
                if (vp != null)
                {
                    vp.SetMaxResolution(Resolutions[idx]);
                    RefreshPlayerSettings();
                }
            };
            _resolutionButton = cat.AddButton("Max Res: 720p", "", "Change maximum video resolution", ButtonStyle.TextOnly);
            _resolutionButton.OnPress += () => QuickMenuAPI.OpenMultiSelect(_resolutionSelect);

            _audioModeSelect = new MultiSelection(
                "Audio Mode",
                new[] { "Direct", "AudioSource", "RoomScale" },
                selectedOption: 0);
            _audioModeSelect.OnOptionUpdated += idx =>
            {
                if (_suppressControlEvents || idx < 0 || idx >= AudioModes.Length) return;
                var vp = VideoPlayerTracker.SelectedPlayer;
                if (vp != null)
                {
                    VideoPlayerTracker.SafeSetAudioMode(vp, AudioModes[idx]);
                    RefreshPlayerSettings();
                }
            };
            _audioModeButton = cat.AddButton("Audio Mode: Direct", "", "Change audio playback mode", ButtonStyle.TextOnly);
            _audioModeButton.OnPress += () => QuickMenuAPI.OpenMultiSelect(_audioModeSelect);

            _subtitleSelect = new MultiSelection("Subtitles", new[] { "None" }, selectedOption: 0);
            _subtitleSelect.OnOptionUpdated += idx =>
            {
                if (_suppressControlEvents || idx < 0 || idx >= _subtitleIds.Count) return;
                var vp = VideoPlayerTracker.SelectedPlayer;
                if (vp?.VideoPlayer?.Subtitles != null)
                {
                    string subId = _subtitleIds[idx];
                    vp.VideoPlayer.Subtitles.SelectedSubtitle = subId == "-1" ? string.Empty : subId;
                    RefreshPlayerSettings();
                }
            };
            _subtitleButton = cat.AddButton("Subtitles: None", "", "Select subtitle track", ButtonStyle.TextOnly);
            _subtitleButton.OnPress += () =>
            {
                RefreshSubtitleOptions();
                QuickMenuAPI.OpenMultiSelect(_subtitleSelect);
            };

            _lockPermSelect = new MultiSelection("Control Permission", new[] { "Everyone" }, selectedOption: 0);
            _lockPermSelect.OnOptionUpdated += idx =>
            {
                if (_suppressControlEvents || idx < 0 || idx >= _lockOptions.Count) return;
                var vp = VideoPlayerTracker.SelectedPlayer;
                if (vp != null && VideoPlayerTracker.CanControl(vp))
                {
                    vp.SetControlPermission(_lockOptions[idx]);
                    RefreshPlayerSettings();
                }
            };
            _lockPermButton = cat.AddButton("Control: Everyone", "", "Change who can control this video player", ButtonStyle.TextOnly);
            _lockPermButton.OnPress += () =>
            {
                RefreshLockOptions();
                if (_lockOptions.Count > 1)
                    QuickMenuAPI.OpenMultiSelect(_lockPermSelect);
                else
                    QuickMenuAPI.ShowAlertToast("You cannot change lock permissions on this VideoPlayer.", 2);
            };

            var modCat = _playerSettingsPage.AddCategory("Remote Mod Settings");
            var autoSelectToggle = modCat.AddToggle("Auto-Select Nearest", "Automatically pick the nearest video player", Settings.AutoSelectNearest);
            autoSelectToggle.OnValueUpdated += val => Settings.AutoSelectNearest = val;

            var seekStepSlider = modCat.AddSlider("Seek Step (sec)", "Step size for -/+ skip buttons", Settings.SeekStepSeconds, 5f, 120f, decimalPlaces: 0, defaultValue: 15f, allowReset: true);
            seekStepSlider.OnValueUpdated += val => Settings.SeekStepSeconds = val;
        }

        private static void RefreshMainRemote()
        {
            if (_targetInfoText == null || !CVR_MenuManager.IsReadyStatic)
                return;

            var vp = VideoPlayerTracker.SelectedPlayer;
            _suppressControlEvents = true;
            try
            {
                if (_mainHudToggle != null)
                    _mainHudToggle.ToggleValue = Settings.HudEnabled;

                if (vp == null)
                {
                    _targetInfoText.Text = $"Target: None ({VideoPlayerTracker.Players.Count} found in instance)";
                    _nowPlayingText.Text = "Open 'Select VideoPlayer' to pick a world or prop VideoPlayer.";
                    _playPauseButton.ButtonText = "Play / Pause";
                    SetUntrustedVisible(false);
                    return;
                }

                string lockInfo = VideoPlayerTracker.GetLockOptionLabel(vp.ControlPermission);
                string canCtrl = VideoPlayerTracker.CanControl(vp) ? "" : " [Locked]";
                _targetInfoText.Text = $"Target: {VideoPlayerTracker.GetFullLabel(vp)} | Lock: {lockInfo}{canCtrl}";

                var state = VideoPlayerTracker.GetState(vp);
                string title = Truncate(VideoPlayerTracker.GetCurrentTitle(vp), 64);
                string timeText = VideoPlayerTracker.IsLivestream(vp)
                    ? "LIVE"
                    : $"{FormatTime(VideoPlayerTracker.GetCurrentTime(vp))} / {FormatTime(VideoPlayerTracker.GetDuration(vp))}";
                string requester = !string.IsNullOrWhiteSpace(vp.lastNetworkVideoUsername)
                    ? $" | By: {vp.lastNetworkVideoUsername}"
                    : "";
                string playlistTag = VideoPlayerTracker.ActivePlaylist != null &&
                                     VideoPlayerTracker.ActivePlaylistPlayer == vp &&
                                     VideoPlayerTracker.ActivePlaylist.Items.Count > 0
                    ? $"\nQueue: {VideoPlayerTracker.ActivePlaylist.Name} ({VideoPlayerTracker.ActivePlaylistIndex + 1}/{VideoPlayerTracker.ActivePlaylist.Items.Count})"
                    : "";

                _nowPlayingText.Text = $"[{state}] {title}\n{timeText}{requester}{playlistTag}";

                bool isPlaying = state == VideoPlayerUtils.PlayerState.Playing ||
                                 state == VideoPlayerUtils.PlayerState.Buffering ||
                                 state == VideoPlayerUtils.PlayerState.Seeking;
                _playPauseButton.ButtonText = isPlaying ? "Pause" : "Play";

                if (!VideoPlayerTracker.HasPendingSeek)
                {
                    double duration = VideoPlayerTracker.GetDuration(vp);
                    double curTime = VideoPlayerTracker.GetCurrentTime(vp);
                    _seekSlider.Disabled = VideoPlayerTracker.IsLivestream(vp) || duration <= 0.1 || !VideoPlayerTracker.CanControl(vp);
                    _seekSlider.MaxValue = duration > 1.0 ? (float)duration : 100f;
                    _seekSlider.SetSliderValue(Mathf.Clamp((float)curTime, 0f, _seekSlider.MaxValue));
                }

                _volumeSlider.SetSliderValue(Mathf.Clamp01(vp.playbackVolume));
                _muteToggle.ToggleValue = vp.playbackVolume <= 0.001f;
                _speedSlider.SetSliderValue(Mathf.Clamp(vp.localPlaybackSpeed, 0.5f, 2f));
                _watchingToggle.ToggleValue = vp.IsWatching;
                if (_mainHudToggle != null)
                    _mainHudToggle.ToggleValue = Settings.HudEnabled;

                string curUrl = VideoPlayerTracker.GetCurrentUrl(vp);
                bool isFav = LibraryStore.IsFavorite(curUrl);
                _favoriteCurrentButton.ButtonText = isFav ? "Unfavorite Current" : "Favorite Current";

                SetUntrustedVisible(vp.UntrustedVideoPending);
            }
            finally
            {
                _suppressControlEvents = false;
            }
        }

        private static void SetUntrustedVisible(bool visible)
        {
            if (_untrustedCategory == null || _untrustedShown == visible)
                return;
            _untrustedShown = visible;
            _untrustedCategory.Hidden = !visible;
        }

        private static void UpdateMuteToggleVisual(bool muted)
        {
            _suppressControlEvents = true;
            _muteToggle.ToggleValue = muted;
            _suppressControlEvents = false;
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

            RefreshMainRemote();
        }

        private static void HandleUntrusted(VideoPlayerUtils.UntrustedVideoAction action)
        {
            var vp = VideoPlayerTracker.SelectedPlayer;
            if (vp == null) return;
            vp.HandleUntrustedVideo(action);
            RefreshMainRemote();
        }

        private static void OpenUrlOrSearchKeyboard()
        {
            QuickMenuAPI.OpenKeyboard(
                currentText: "",
                callback: HandleUrlOrSearchInput,
                placeholder: "Paste URL or type YouTube search...",
                maxCharacterCount: 500,
                multiLine: false,
                title: "Play URL or Search YouTube");
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
                    QuickMenuAPI.ShowAlertToast("No video player selected.", 2);
                    return;
                }
                VideoPlayerTracker.PlayUrl(vp, trimmed);
                QuickMenuAPI.ShowAlertToast("Sent URL to video player.", 2);
                RefreshMainRemote();
                return;
            }

            if (VideoPlayerTracker.IsSearchInFlight)
            {
                QuickMenuAPI.ShowAlertToast("Search already in progress...", 2);
                return;
            }

            QuickMenuAPI.ShowAlertToast($"Searching YouTube for \"{Truncate(trimmed, 24)}\"...", 2);
            VideoPlayerTracker.SearchYouTube(trimmed, (results, term) =>
            {
                _lastSearchResults = results ?? new List<SavedVideoItem>();
                _lastSearchTerm = term;
                RebuildSearchResultsPage();
                _searchResultsPage.OpenPage();
            });
        }

        private static void FavoriteCurrentVideo()
        {
            var vp = VideoPlayerTracker.SelectedPlayer;
            if (vp == null)
            {
                QuickMenuAPI.ShowAlertToast("No video player selected.", 2);
                return;
            }

            string url = VideoPlayerTracker.GetCurrentUrl(vp);
            if (string.IsNullOrWhiteSpace(url))
            {
                QuickMenuAPI.ShowAlertToast("No video is currently playing.", 2);
                return;
            }

            if (LibraryStore.IsFavorite(url))
            {
                LibraryStore.RemoveFavorite(url);
                QuickMenuAPI.ShowAlertToast("Removed from Favorites.", 2);
            }
            else
            {
                string title = VideoPlayerTracker.GetCurrentTitle(vp);
                LibraryStore.AddFavorite(title, url);
                QuickMenuAPI.ShowAlertToast("Added to Favorites!", 2);
            }
            RefreshMainRemote();
        }

        // ==================== PLAYER SELECTOR ====================

        private static void RebuildPlayerSelectPage()
        {
            if (_playerSelectPage == null || !CVR_MenuManager.IsReadyStatic)
                return;

            _playerSelectPage.ClearChildren();

            var actionsCat = _playerSelectPage.AddCategory("Quick Selection");
            actionsCat.AddButton("Select Nearest", "", "Pick the closest video player", ButtonStyle.TextOnly)
                .OnPress += () =>
                {
                    var vp = VideoPlayerTracker.SelectNearest();
                    QuickMenuAPI.ShowAlertToast(vp != null
                        ? $"Selected: {VideoPlayerTracker.GetShortName(vp)}"
                        : "No video players found.", 2);
                    RebuildPlayerSelectPage();
                };

            actionsCat.AddButton("Refresh List", "", "Rescan world and prop video players", ButtonStyle.TextOnly)
                .OnPress += () =>
                {
                    VideoPlayerTracker.RefreshFromScene();
                    RebuildPlayerSelectPage();
                };

            var worldPlayers = VideoPlayerTracker.GetWorldPlayers();
            var worldCat = _playerSelectPage.AddCategory($"World VideoPlayers ({worldPlayers.Count})");
            if (worldPlayers.Count == 0)
            {
                worldCat.AddTextBlock("No world VideoPlayers found.");
            }
            else
            {
                foreach (var vp in worldPlayers)
                {
                    var captured = vp;
                    bool isSelected = captured == VideoPlayerTracker.SelectedPlayer;
                    string prefix = isSelected ? "* " : "";
                    string state = VideoPlayerTracker.GetState(captured).ToString();
                    string dist = $"{VideoPlayerTracker.GetDistanceToLocalPlayer(captured):F0}m";
                    worldCat.AddButton(
                            $"{prefix}{VideoPlayerTracker.GetShortName(captured)}",
                            "",
                            $"World VideoPlayer | {state} | {dist}",
                            ButtonStyle.TextOnly)
                        .OnPress += () =>
                        {
                            VideoPlayerTracker.SelectPlayer(captured);
                            QuickMenuAPI.ShowAlertToast($"Selected: {VideoPlayerTracker.GetShortName(captured)}", 2);
                            _rootPage.OpenPage();
                        };
                }
            }

            var propPlayers = VideoPlayerTracker.GetPropPlayers();
            var propCat = _playerSelectPage.AddCategory($"Prop VideoPlayers ({propPlayers.Count})");
            if (propPlayers.Count == 0)
            {
                propCat.AddTextBlock("No spawned prop VideoPlayers found.");
            }
            else
            {
                foreach (var vp in propPlayers)
                {
                    var captured = vp;
                    bool isSelected = captured == VideoPlayerTracker.SelectedPlayer;
                    string prefix = isSelected ? "* " : "";
                    string state = VideoPlayerTracker.GetState(captured).ToString();
                    string dist = $"{VideoPlayerTracker.GetDistanceToLocalPlayer(captured):F0}m";
                    propCat.AddButton(
                            $"{prefix}{VideoPlayerTracker.GetShortName(captured)}",
                            "",
                            $"Prop VideoPlayer | {state} | {dist}",
                            ButtonStyle.TextOnly)
                        .OnPress += () =>
                        {
                            VideoPlayerTracker.SelectPlayer(captured);
                            QuickMenuAPI.ShowAlertToast($"Selected: {VideoPlayerTracker.GetShortName(captured)}", 2);
                            _rootPage.OpenPage();
                        };
                }
            }
        }

        // ==================== LOCAL PICKUP SCREENS ====================

        private static void RebuildScreensPage()
        {
            if (_screensPage == null || !CVR_MenuManager.IsReadyStatic)
                return;

            _screensPage.ClearChildren();

            var spawnCat = _screensPage.AddCategory("Spawn & Defaults");
            spawnCat.AddButton("Spawn Screen", "", "Spawn a grabbable local screen in front of you", ButtonStyle.TextOnly)
                .OnPress += () =>
                {
                    var screen = LocalScreenManager.SpawnScreen();
                    if (screen != null)
                        QuickMenuAPI.ShowAlertToast($"Spawned Screen #{screen.Id}", 2);
                };

            spawnCat.AddButton("Despawn All", "", "Remove all spawned local screens", ButtonStyle.TextOnly)
                .OnPress += () =>
                {
                    LocalScreenManager.CleanupAll();
                    QuickMenuAPI.ShowAlertToast("Removed all local screens.", 2);
                };

            var defaultScaleSlider = spawnCat.AddSlider(
                "Default Scale (m)",
                "Width in meters for newly spawned screens",
                Settings.DefaultScreenScale,
                0.3f,
                4.0f,
                decimalPlaces: 2,
                defaultValue: 1.2f,
                allowReset: true);
            defaultScaleSlider.OnValueUpdated += val => Settings.DefaultScreenScale = val;

            var followToggle = spawnCat.AddToggle(
                "Follow Selected VideoPlayer",
                "New screens automatically switch when you pick another VideoPlayer",
                Settings.DefaultFollowSelected);
            followToggle.OnValueUpdated += val => Settings.DefaultFollowSelected = val;

            var presetCat = _screensPage.AddCategory("Quick Spawn Presets");
            presetCat.AddButton("Small (0.6m)", "", "Spawn a 0.6m portable screen", ButtonStyle.TextOnly)
                .OnPress += () => SpawnWithPreset(0.6f);
            presetCat.AddButton("Medium (1.2m)", "", "Spawn a 1.2m desk screen", ButtonStyle.TextOnly)
                .OnPress += () => SpawnWithPreset(1.2f);
            presetCat.AddButton("Large (2.2m)", "", "Spawn a 2.2m living room screen", ButtonStyle.TextOnly)
                .OnPress += () => SpawnWithPreset(2.2f);
            presetCat.AddButton("Cinema (3.5m)", "", "Spawn a 3.5m cinema screen", ButtonStyle.TextOnly)
                .OnPress += () => SpawnWithPreset(3.5f);

            var activeCat = _screensPage.AddCategory($"Active Screens ({LocalScreenManager.Screens.Count})");
            if (LocalScreenManager.Screens.Count == 0)
            {
                activeCat.AddTextBlock("No local screens spawned yet.");
            }
            else
            {
                foreach (var screen in LocalScreenManager.Screens)
                {
                    var captured = screen;
                    activeCat.AddButton(captured.DisplayLabel, "", "Configure scale, position, lock, or despawn", ButtonStyle.TextOnly)
                        .OnPress += () =>
                        {
                            _activeScreenDetail = captured;
                            RebuildScreenDetailPage();
                            _screenDetailPage.OpenPage();
                        };
                }
            }
        }

        private static void SpawnWithPreset(float scale)
        {
            var screen = LocalScreenManager.SpawnScreen(scale: scale);
            if (screen != null)
                QuickMenuAPI.ShowAlertToast($"Spawned Screen #{screen.Id} ({scale:F1}m)", 2);
        }

        private static void RebuildScreenDetailPage()
        {
            if (_screenDetailPage == null)
                return;

            _screenDetailPage.ClearChildren();

            if (_activeScreenDetail == null || !_activeScreenDetail.IsAlive)
            {
                _screenDetailPage.AddCategory("Screen").AddTextBlock("This screen no longer exists.");
                return;
            }

            var screen = _activeScreenDetail;
            _screenDetailPage.MenuTitle = $"Screen #{screen.Id}";

            var ctrlCat = _screenDetailPage.AddCategory("Position & Pinning");
            // Row 1: Bring to Me | Lock Grab | Pin to Local Space | Follow Playspace
            ctrlCat.AddButton("Bring to Me", "", "Move this screen directly in front of you", ButtonStyle.TextOnly)
                .OnPress += () =>
                {
                    if (!screen.IsAlive) return;
                    screen.BringToPlayer();
                    QuickMenuAPI.ShowAlertToast($"Brought Screen #{screen.Id} to you.", 2);
                };

            var lockToggle = ctrlCat.AddToggle("Lock Grab", "Prevent accidentally grabbing this screen", screen.GrabLocked);
            lockToggle.OnValueUpdated += locked =>
            {
                if (!screen.IsAlive) return;
                screen.SetGrabLocked(locked);
            };

            var localPinToggle = ctrlCat.AddToggle("Pin to Local Space", "Keep screen anchored to your player movement & rotation", screen.PinToLocalSpace);
            localPinToggle.OnValueUpdated += pin =>
            {
                if (!screen.IsAlive) return;
                screen.SetPinToLocalSpace(pin);
            };

            var playspaceToggle = ctrlCat.AddToggle("Follow Playspace", "Move screen when you playspace-drag up/down/around in VR", screen.FollowPlayspace);
            playspaceToggle.OnValueUpdated += follow =>
            {
                if (!screen.IsAlive) return;
                screen.SetFollowPlayspace(follow);
            };

            // Row 2: Follow Selected VideoPlayer | Pin to Current VideoPlayer | [blank] | Despawn Screen
            var followToggle = ctrlCat.AddToggle("Follow Selected VideoPlayer", "Auto-switch feed when you select another VideoPlayer", screen.FollowSelectedPlayer);
            followToggle.OnValueUpdated += follow =>
            {
                if (!screen.IsAlive) return;
                screen.FollowSelectedPlayer = follow;
                if (!follow && VideoPlayerTracker.SelectedPlayer != null)
                    screen.PinnedPlayer = VideoPlayerTracker.SelectedPlayer;
                screen.RefreshFeed();
            };

            ctrlCat.AddButton("Pin to Current VideoPlayer", "", "Bind this screen to the currently selected VideoPlayer", ButtonStyle.TextOnly)
                .OnPress += () =>
                {
                    if (!screen.IsAlive) return;
                    var vp = VideoPlayerTracker.SelectedPlayer;
                    if (vp == null)
                    {
                        QuickMenuAPI.ShowAlertToast("No video player selected.", 2);
                        return;
                    }
                    screen.FollowSelectedPlayer = false;
                    screen.PinnedPlayer = vp;
                    screen.RefreshFeed();
                    followToggle.ToggleValue = false;
                    QuickMenuAPI.ShowAlertToast($"Pinned to {VideoPlayerTracker.GetShortName(vp)}", 2);
                };

            AddBlank(ctrlCat);

            ctrlCat.AddButton("Despawn Screen", "", "Delete this local screen", ButtonStyle.TextOnly)
                .OnPress += () =>
                {
                    LocalScreenManager.DespawnScreen(screen);
                    _activeScreenDetail = null;
                    _screensPage.OpenPage();
                };

            var sizeCat = _screenDetailPage.AddCategory("Scale & Size Presets");
            var scaleSlider = sizeCat.AddSlider("Width (m)", "Adjust screen width (16:9)", screen.WidthScale, 0.3f, 4.0f, decimalPlaces: 2, defaultValue: 1.2f, allowReset: true);
            scaleSlider.OnValueUpdated += val =>
            {
                if (!screen.IsAlive) return;
                screen.SetScale(val);
            };

            sizeCat.AddButton("Small (0.6m)", "", "Set width to 0.6m", ButtonStyle.TextOnly)
                .OnPress += () => ApplyScreenScale(screen, scaleSlider, 0.6f);
            sizeCat.AddButton("Medium (1.2m)", "", "Set width to 1.2m", ButtonStyle.TextOnly)
                .OnPress += () => ApplyScreenScale(screen, scaleSlider, 1.2f);
            sizeCat.AddButton("Large (2.2m)", "", "Set width to 2.2m", ButtonStyle.TextOnly)
                .OnPress += () => ApplyScreenScale(screen, scaleSlider, 2.2f);
            sizeCat.AddButton("Cinema (3.5m)", "", "Set width to 3.5m", ButtonStyle.TextOnly)
                .OnPress += () => ApplyScreenScale(screen, scaleSlider, 3.5f);
        }

        private static void ApplyScreenScale(LocalScreenInstance screen, SliderFloat slider, float scale)
        {
            if (screen == null || !screen.IsAlive)
                return;
            screen.SetScale(scale);
            slider.SetSliderValue(scale);
        }

        // ==================== SEARCH RESULTS ====================

        private static void RebuildSearchResultsPage()
        {
            if (_searchResultsPage == null)
                return;

            _searchResultsPage.ClearChildren();
            _searchResultsPage.MenuTitle = string.IsNullOrEmpty(_lastSearchTerm)
                ? "YouTube Search"
                : $"Search: {Truncate(_lastSearchTerm, 26)}";

            var topCat = _searchResultsPage.AddCategory("Actions");
            topCat.AddButton("New Search", "", "Search YouTube again", ButtonStyle.TextOnly)
                .OnPress += OpenUrlOrSearchKeyboard;

            var resCat = _searchResultsPage.AddCategory($"Results ({_lastSearchResults.Count})");
            if (_lastSearchResults.Count == 0)
            {
                resCat.AddTextBlock("No results found.");
                return;
            }

            foreach (var item in _lastSearchResults)
            {
                var captured = item;
                resCat.AddButton(Truncate(captured.DisplayTitle, 38), "", captured.Url, ButtonStyle.TextOnly)
                    .OnPress += () => OpenVideoActionPage(captured, allowRemoveFavorite: false);
            }
        }

        // ==================== FAVORITES ====================

        private static void RebuildFavoritesPage()
        {
            if (_favoritesPage == null || !CVR_MenuManager.IsReadyStatic)
                return;

            _favoritesPage.ClearChildren();

            var topCat = _favoritesPage.AddCategory("Actions");
            topCat.AddButton("Favorite Current", "", "Add the currently playing video to Favorites", ButtonStyle.TextOnly)
                .OnPress += FavoriteCurrentVideo;

            topCat.AddButton("Add URL", "", "Add a video URL manually to Favorites", ButtonStyle.TextOnly)
                .OnPress += () =>
                {
                    OpenTextPrompt("", "Add Favorite URL", "https://...", url =>
                    {
                        if (!VideoPlayerTracker.IsHttpUrl(url))
                        {
                            QuickMenuAPI.ShowAlertToast("Invalid HTTP/HTTPS URL.", 2);
                            return;
                        }
                        OpenTextPrompt("", "Favorite Title", "Optional video title...", title =>
                        {
                            LibraryStore.AddFavorite(title, url);
                            QuickMenuAPI.ShowAlertToast("Saved to Favorites.", 2);
                        }, 120);
                    });
                };

            var listCat = _favoritesPage.AddCategory($"Saved Favorites ({LibraryStore.Data.Favorites.Count})");
            if (LibraryStore.Data.Favorites.Count == 0)
            {
                listCat.AddTextBlock("No favorite videos yet.");
                return;
            }

            foreach (var fav in LibraryStore.Data.Favorites)
            {
                var captured = fav;
                listCat.AddButton(Truncate(captured.DisplayTitle, 38), "", captured.Url, ButtonStyle.TextOnly)
                    .OnPress += () => OpenVideoActionPage(captured, allowRemoveFavorite: true);
            }
        }

        // ==================== PLAYLISTS ====================

        private static void RebuildPlaylistsPage()
        {
            if (_playlistsPage == null || !CVR_MenuManager.IsReadyStatic)
                return;

            _playlistsPage.ClearChildren();

            var ctrlCat = _playlistsPage.AddCategory("Playlist Controls");
            ctrlCat.AddButton("New Playlist", "", "Create a new custom playlist", ButtonStyle.TextOnly)
                .OnPress += () =>
                {
                    OpenTextPrompt("", "Create Playlist", "Playlist name...", name =>
                    {
                        if (string.IsNullOrWhiteSpace(name)) return;
                        var created = LibraryStore.CreatePlaylist(name);
                        _activePlaylistDetail = created;
                        RebuildPlaylistDetailPage();
                        _playlistDetailPage.OpenPage();
                    }, 64);
                };

            var shuffleToggle = ctrlCat.AddToggle("Shuffle", "Shuffle custom playlist playback", Settings.PlaylistShuffle);
            shuffleToggle.OnValueUpdated += val => Settings.PlaylistShuffle = val;

            var loopToggle = ctrlCat.AddToggle("Loop", "Loop custom playlist when finished", Settings.PlaylistLoop);
            loopToggle.OnValueUpdated += val => Settings.PlaylistLoop = val;

            if (VideoPlayerTracker.ActivePlaylist != null)
            {
                ctrlCat.AddButton("Stop Queue", "", "Stop auto-advancing the active custom playlist", ButtonStyle.TextOnly)
                    .OnPress += () =>
                    {
                        VideoPlayerTracker.ClearCustomPlaylist();
                        RebuildPlaylistsPage();
                        RefreshMainRemote();
                    };
            }

            var listCat = _playlistsPage.AddCategory($"Your Playlists ({LibraryStore.Data.Playlists.Count})");
            if (LibraryStore.Data.Playlists.Count == 0)
            {
                listCat.AddTextBlock("No custom playlists yet. Press 'New Playlist' to create one.");
                return;
            }

            foreach (var pl in LibraryStore.Data.Playlists)
            {
                var captured = pl;
                string activePrefix = VideoPlayerTracker.ActivePlaylist == captured ? "* " : "";
                listCat.AddButton($"{activePrefix}{captured.Name} ({captured.Items.Count})", "", "Open playlist", ButtonStyle.TextOnly)
                    .OnPress += () =>
                    {
                        _activePlaylistDetail = captured;
                        RebuildPlaylistDetailPage();
                        _playlistDetailPage.OpenPage();
                    };
            }
        }

        private static void RebuildPlaylistDetailPage()
        {
            if (_playlistDetailPage == null || !CVR_MenuManager.IsReadyStatic)
                return;

            _playlistDetailPage.ClearChildren();

            var pl = _activePlaylistDetail;
            if (pl == null || !LibraryStore.Data.Playlists.Contains(pl))
            {
                _playlistDetailPage.AddCategory("Playlist").AddTextBlock("Playlist not found.");
                return;
            }

            _playlistDetailPage.MenuTitle = pl.Name;

            var actionsCat = _playlistDetailPage.AddCategory("Playlist Actions");
            actionsCat.AddButton("Play Playlist", "", "Start playing from track 1", ButtonStyle.TextOnly)
                .OnPress += () =>
                {
                    if (pl.Items.Count == 0)
                    {
                        QuickMenuAPI.ShowAlertToast("Playlist is empty.", 2);
                        return;
                    }
                    if (VideoPlayerTracker.SelectedPlayer == null)
                    {
                        QuickMenuAPI.ShowAlertToast("No video player selected.", 2);
                        return;
                    }
                    VideoPlayerTracker.PlayCustomPlaylist(pl, 0);
                    QuickMenuAPI.ShowAlertToast($"Playing: {pl.Name}", 2);
                    _rootPage.OpenPage();
                };

            actionsCat.AddButton("Add Current Video", "", "Append the currently playing video", ButtonStyle.TextOnly)
                .OnPress += () =>
                {
                    var vp = VideoPlayerTracker.SelectedPlayer;
                    string url = VideoPlayerTracker.GetCurrentUrl(vp);
                    if (string.IsNullOrWhiteSpace(url))
                    {
                        QuickMenuAPI.ShowAlertToast("No video is currently playing.", 2);
                        return;
                    }
                    LibraryStore.AddToPlaylist(pl, VideoPlayerTracker.GetCurrentTitle(vp), url);
                    QuickMenuAPI.ShowAlertToast("Added current video to playlist.", 2);
                };

            actionsCat.AddButton("Add URL", "", "Append a URL manually", ButtonStyle.TextOnly)
                .OnPress += () =>
                {
                    OpenTextPrompt("", "Add Track URL", "https://...", url =>
                    {
                        if (!VideoPlayerTracker.IsHttpUrl(url))
                        {
                            QuickMenuAPI.ShowAlertToast("Invalid HTTP/HTTPS URL.", 2);
                            return;
                        }
                        OpenTextPrompt("", "Track Title", "Optional track title...", title =>
                        {
                            LibraryStore.AddToPlaylist(pl, title, url);
                            QuickMenuAPI.ShowAlertToast("Added track to playlist.", 2);
                        }, 120);
                    });
                };

            actionsCat.AddButton("Rename", "", "Rename this playlist", ButtonStyle.TextOnly)
                .OnPress += () =>
                {
                    OpenTextPrompt(pl.Name, "Rename Playlist", "New playlist name...", newName =>
                    {
                        LibraryStore.RenamePlaylist(pl, newName);
                    }, 64);
                };

            actionsCat.AddButton("Delete Playlist", "", "Permanently delete this playlist", ButtonStyle.TextOnly)
                .OnPress += () =>
                {
                    QuickMenuAPI.ShowConfirm(
                        "Delete Playlist?",
                        $"Delete '{pl.Name}' ({pl.Items.Count} videos)?",
                        () =>
                        {
                            if (VideoPlayerTracker.ActivePlaylist == pl)
                                VideoPlayerTracker.ClearCustomPlaylist();
                            LibraryStore.DeletePlaylist(pl);
                            _activePlaylistDetail = null;
                            _playlistsPage.OpenPage();
                        });
                };

            var tracksCat = _playlistDetailPage.AddCategory($"Tracks ({pl.Items.Count})");
            if (pl.Items.Count == 0)
            {
                tracksCat.AddTextBlock("No videos in this playlist yet.");
                return;
            }

            for (int i = 0; i < pl.Items.Count; i++)
            {
                int index = i;
                var item = pl.Items[index];
                bool isCurrent = VideoPlayerTracker.ActivePlaylist == pl && VideoPlayerTracker.ActivePlaylistIndex == index;
                string prefix = isCurrent ? "* " : $"{index + 1}. ";

                var itemCat = _playlistDetailPage.AddCategory($"{prefix}{Truncate(item.DisplayTitle, 38)}");
                itemCat.AddButton("Play", "", "Play from this track", ButtonStyle.TextOnly)
                    .OnPress += () =>
                    {
                        if (VideoPlayerTracker.SelectedPlayer == null)
                        {
                            QuickMenuAPI.ShowAlertToast("No video player selected.", 2);
                            return;
                        }
                        VideoPlayerTracker.PlayCustomPlaylist(pl, index);
                        _rootPage.OpenPage();
                    };

                if (index > 0)
                {
                    itemCat.AddButton("Move Up", "", "Move track earlier", ButtonStyle.TextOnly)
                        .OnPress += () => LibraryStore.MoveInPlaylist(pl, index, -1);
                }

                if (index < pl.Items.Count - 1)
                {
                    itemCat.AddButton("Move Down", "", "Move track later", ButtonStyle.TextOnly)
                        .OnPress += () => LibraryStore.MoveInPlaylist(pl, index, 1);
                }

                itemCat.AddButton("Remove", "", "Remove track from playlist", ButtonStyle.TextOnly)
                    .OnPress += () => LibraryStore.RemoveFromPlaylist(pl, index);
            }
        }

        // ==================== VIDEO ACTION & PLAYLIST PICKER ====================

        private static void OpenVideoActionPage(SavedVideoItem item, bool allowRemoveFavorite)
        {
            _activeVideoActionItem = item;
            _videoActionPage.ClearChildren();
            _videoActionPage.MenuTitle = Truncate(item.DisplayTitle, 28);

            var infoCat = _videoActionPage.AddCategory("Video");
            infoCat.AddTextBlock($"{item.DisplayTitle}\n{Truncate(item.Url, 72)}");

            var actCat = _videoActionPage.AddCategory("Actions");
            actCat.AddButton("Play Now", "", "Play on selected video player", ButtonStyle.TextOnly)
                .OnPress += () =>
                {
                    var vp = VideoPlayerTracker.SelectedPlayer;
                    if (vp == null)
                    {
                        QuickMenuAPI.ShowAlertToast("No video player selected.", 2);
                        return;
                    }
                    VideoPlayerTracker.PlayUrl(vp, item.Url);
                    QuickMenuAPI.ShowAlertToast($"Playing: {Truncate(item.DisplayTitle, 28)}", 2);
                    _rootPage.OpenPage();
                };

            bool isFav = LibraryStore.IsFavorite(item.Url);
            actCat.AddButton(isFav ? "Remove Favorite" : "Add to Favorites", "", "Toggle favorite status", ButtonStyle.TextOnly)
                .OnPress += () =>
                {
                    if (LibraryStore.IsFavorite(item.Url))
                    {
                        LibraryStore.RemoveFavorite(item.Url);
                        QuickMenuAPI.ShowAlertToast("Removed from Favorites.", 2);
                    }
                    else
                    {
                        LibraryStore.AddFavorite(item.DisplayTitle, item.Url);
                        QuickMenuAPI.ShowAlertToast("Added to Favorites!", 2);
                    }
                    if (allowRemoveFavorite)
                        _favoritesPage.OpenPage();
                    else
                        OpenVideoActionPage(item, allowRemoveFavorite);
                };

            actCat.AddButton("Add to Playlist", "", "Choose a custom playlist to add this video to", ButtonStyle.TextOnly)
                .OnPress += () => OpenPickPlaylistPage(item);

            _videoActionPage.OpenPage();
        }

        private static void OpenPickPlaylistPage(SavedVideoItem item)
        {
            _pickPlaylistPage.ClearChildren();
            var cat = _pickPlaylistPage.AddCategory($"Add '{Truncate(item.DisplayTitle, 24)}' to:");

            cat.AddButton("+ Create New Playlist", "", "Create a playlist and add this video", ButtonStyle.TextOnly)
                .OnPress += () =>
                {
                    OpenTextPrompt("", "New Playlist", "Playlist name...", name =>
                    {
                        if (string.IsNullOrWhiteSpace(name)) return;
                        var pl = LibraryStore.CreatePlaylist(name);
                        LibraryStore.AddToPlaylist(pl, item.DisplayTitle, item.Url);
                        QuickMenuAPI.ShowAlertToast($"Added to '{pl.Name}'", 2);
                        _videoActionPage.OpenPage();
                    }, 64);
                };

            foreach (var pl in LibraryStore.Data.Playlists)
            {
                var captured = pl;
                cat.AddButton($"{captured.Name} ({captured.Items.Count})", "", "Add video to this playlist", ButtonStyle.TextOnly)
                    .OnPress += () =>
                    {
                        LibraryStore.AddToPlaylist(captured, item.DisplayTitle, item.Url);
                        QuickMenuAPI.ShowAlertToast($"Added to '{captured.Name}'", 2);
                        _videoActionPage.OpenPage();
                    };
            }

            _pickPlaylistPage.OpenPage();
        }

        // ==================== BUILT-IN PLAYER PLAYLISTS ====================

        private static void RebuildNativePlaylistsPage()
        {
            if (_nativePlaylistsPage == null || !CVR_MenuManager.IsReadyStatic)
                return;

            _nativePlaylistsPage.ClearChildren();
            var vp = VideoPlayerTracker.SelectedPlayer;
            if (vp == null)
            {
                _nativePlaylistsPage.AddCategory("Built-in Playlists").AddTextBlock("No video player selected.");
                return;
            }

            if (vp.entities == null || vp.entities.Count == 0)
            {
                _nativePlaylistsPage.AddCategory("Built-in Playlists").AddTextBlock("This video player has no built-in playlists.");
                return;
            }

            foreach (var playlist in vp.entities)
            {
                if (playlist == null || playlist.playlistVideos == null)
                    continue;

                var capturedPl = playlist;
                string title = string.IsNullOrWhiteSpace(capturedPl.playlistTitle) ? "Playlist" : capturedPl.playlistTitle;
                var cat = _nativePlaylistsPage.AddCategory($"{title} ({capturedPl.playlistVideos.Count})");

                foreach (var vid in capturedPl.playlistVideos)
                {
                    if (vid == null) continue;
                    var capturedVid = vid;
                    string vTitle = string.IsNullOrWhiteSpace(capturedVid.videoTitle) ? capturedVid.videoUrl : capturedVid.videoTitle;
                    cat.AddButton(Truncate(vTitle, 38), "", capturedVid.videoUrl ?? "", ButtonStyle.TextOnly)
                        .OnPress += () =>
                        {
                            if (!VideoPlayerTracker.IsValid(vp))
                            {
                                QuickMenuAPI.ShowAlertToast("Video player no longer exists.", 2);
                                return;
                            }
                            VideoPlayerTracker.ClearCustomPlaylist();
                            vp.SelectVideo(capturedPl, capturedVid);
                            QuickMenuAPI.ShowAlertToast($"Playing: {Truncate(vTitle, 28)}", 2);
                            _rootPage.OpenPage();
                        };
                }
            }
        }

        // ==================== PLAYER SETTINGS REFRESH ====================

        private static void RefreshPlayerSettings()
        {
            if (!CVR_MenuManager.IsReadyStatic)
                return;

            var vp = VideoPlayerTracker.SelectedPlayer;
            _suppressControlEvents = true;
            try
            {
                bool hasPlayer = vp != null;
                _networkSyncToggle.Disabled = !hasPlayer;
                _audioOnlyToggle.Disabled = !hasPlayer;
                _resolutionButton.Disabled = !hasPlayer;
                _audioModeButton.Disabled = !hasPlayer;
                _subtitleButton.Disabled = !hasPlayer;
                _lockPermButton.Disabled = !hasPlayer;

                if (!hasPlayer)
                    return;

                _networkSyncToggle.ToggleValue = vp.syncEnabled;
                _audioOnlyToggle.ToggleValue = vp.ytAudioOnly;

                int resIdx = Array.IndexOf(Resolutions, vp.maxResolution);
                if (resIdx >= 0) _resolutionSelect.SetSelectedOptionWithoutAction(resIdx);
                _resolutionButton.ButtonText = $"Max Res: {(int)vp.maxResolution}p";

                int audioIdx = Array.IndexOf(AudioModes, vp.audioPlaybackMode);
                if (audioIdx >= 0) _audioModeSelect.SetSelectedOptionWithoutAction(audioIdx);
                _audioModeButton.ButtonText = $"Audio: {vp.audioPlaybackMode}";

                RefreshSubtitleOptions();
                RefreshLockOptions();
            }
            finally
            {
                _suppressControlEvents = false;
            }
        }

        private static void RefreshSubtitleOptions()
        {
            _subtitleIds.Clear();
            var names = new List<string> { "None" };
            _subtitleIds.Add("-1");

            var vp = VideoPlayerTracker.SelectedPlayer;
            int selectedIdx = 0;
            string currentSub = string.Empty;

            if (vp?.VideoPlayer?.Subtitles != null)
            {
                try
                {
                    currentSub = vp.VideoPlayer.Subtitles.SelectedSubtitle ?? string.Empty;
                    var subs = vp.VideoPlayer.Subtitles.GetAvailableSubtitles();
                    if (subs != null)
                    {
                        foreach (var s in subs)
                        {
                            names.Add(string.IsNullOrWhiteSpace(s.SubtitleName) ? s.SubtitleId : s.SubtitleName);
                            _subtitleIds.Add(s.SubtitleId ?? "-1");
                            if (!string.IsNullOrEmpty(currentSub) && s.SubtitleId == currentSub)
                                selectedIdx = _subtitleIds.Count - 1;
                        }
                    }
                }
                catch { }
            }

            _subtitleSelect.Options = names.ToArray();
            _subtitleSelect.SetSelectedOptionWithoutAction(selectedIdx);
            _subtitleButton.ButtonText = $"Subtitles: {Truncate(names[selectedIdx], 18)}";
        }

        private static void RefreshLockOptions()
        {
            _lockOptions.Clear();
            var labels = new List<string>();

            var vp = VideoPlayerTracker.SelectedPlayer;
            if (vp == null)
            {
                _lockPermButton.ButtonText = "Control: None";
                return;
            }

            var current = vp.ControlPermission;
            var available = VideoPlayerTracker.GetAvailableLockOptions(vp);
            var ourLevel = VideoPlayerTracker.GetOurLockLevel(vp);

            foreach (VideoPlayerLockOption opt in Enum.GetValues(typeof(VideoPlayerLockOption)))
            {
                if (opt == VideoPlayerLockOption.None)
                    continue;
                if ((((available & opt) != 0) && (int)opt <= (int)ourLevel) || opt == current)
                {
                    _lockOptions.Add(opt);
                    labels.Add(VideoPlayerTracker.GetLockOptionLabel(opt));
                }
            }

            if (_lockOptions.Count == 0)
            {
                _lockOptions.Add(current);
                labels.Add(VideoPlayerTracker.GetLockOptionLabel(current));
            }

            int selIdx = Math.Max(0, _lockOptions.IndexOf(current));
            _lockPermSelect.Options = labels.ToArray();
            _lockPermSelect.SetSelectedOptionWithoutAction(selIdx);
            _lockPermButton.ButtonText = $"Control: {VideoPlayerTracker.GetLockOptionLabel(current)}";
        }

        // ==================== FORMATTING HELPERS ====================

        private static void AddBlank(Category cat)
        {
            var blank = cat.AddButton(" ", "", "", ButtonStyle.TextOnly);
            blank.Disabled = true;
        }

        private static void OpenTextPrompt(string current, string title, string placeholder, Action<string> onDone, int maxChars = 500)
        {
            QuickMenuAPI.OpenKeyboard(current ?? "", onDone, placeholder ?? "", maxChars, false, title ?? "");
        }

        private static string FormatTime(double seconds)
        {
            if (seconds < 0 || double.IsNaN(seconds) || double.IsInfinity(seconds))
                return "00:00";
            var ts = TimeSpan.FromSeconds(seconds);
            return ts.TotalHours >= 1.0
                ? $"{(int)ts.TotalHours:D2}:{ts.Minutes:D2}:{ts.Seconds:D2}"
                : $"{ts.Minutes:D2}:{ts.Seconds:D2}";
        }

        private static string Truncate(string text, int maxLen)
        {
            if (string.IsNullOrEmpty(text) || text.Length <= maxLen)
                return text ?? "";
            return text.Substring(0, maxLen - 3) + "...";
        }
    }
}
