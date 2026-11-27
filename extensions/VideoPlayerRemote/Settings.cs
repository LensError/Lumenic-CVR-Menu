using MelonLoader;
using UnityEngine;

namespace VideoPlayerRemote
{
    internal static class Settings
    {
        private static MelonPreferences_Category _category;
        private static MelonPreferences_Entry<float> _defaultScreenScale;
        private static MelonPreferences_Entry<bool> _defaultFollowSelected;
        private static MelonPreferences_Entry<float> _seekStepSeconds;
        private static MelonPreferences_Entry<bool> _autoSelectNearest;
        private static MelonPreferences_Entry<bool> _playlistShuffle;
        private static MelonPreferences_Entry<bool> _playlistLoop;

        // HUD / On-Screen Display settings
        private static MelonPreferences_Entry<bool> _hudEnabled;
        private static MelonPreferences_Entry<bool> _hudOverdraw;
        private static MelonPreferences_Entry<bool> _hudSmoothFollow;
        private static MelonPreferences_Entry<float> _hudOpacity;
        private static MelonPreferences_Entry<float> _hudScale;
        private static MelonPreferences_Entry<float> _hudDistance;
        private static MelonPreferences_Entry<float> _hudOffsetX;
        private static MelonPreferences_Entry<float> _hudOffsetY;

        public static float DefaultScreenScale
        {
            get => _defaultScreenScale != null ? Mathf.Clamp(_defaultScreenScale.Value, 0.3f, 4.0f) : 1.2f;
            set { if (_defaultScreenScale != null) _defaultScreenScale.Value = Mathf.Clamp(value, 0.3f, 4.0f); }
        }

        public static bool DefaultFollowSelected
        {
            get => _defaultFollowSelected == null || _defaultFollowSelected.Value;
            set { if (_defaultFollowSelected != null) _defaultFollowSelected.Value = value; }
        }

        public static float SeekStepSeconds
        {
            get => _seekStepSeconds != null ? Mathf.Clamp(_seekStepSeconds.Value, 5f, 120f) : 15f;
            set { if (_seekStepSeconds != null) _seekStepSeconds.Value = Mathf.Clamp(value, 5f, 120f); }
        }

        public static bool AutoSelectNearest
        {
            get => _autoSelectNearest == null || _autoSelectNearest.Value;
            set { if (_autoSelectNearest != null) _autoSelectNearest.Value = value; }
        }

        public static bool PlaylistShuffle
        {
            get => _playlistShuffle != null && _playlistShuffle.Value;
            set { if (_playlistShuffle != null) _playlistShuffle.Value = value; }
        }

        public static bool PlaylistLoop
        {
            get => _playlistLoop == null || _playlistLoop.Value;
            set { if (_playlistLoop != null) _playlistLoop.Value = value; }
        }

        public static bool HudEnabled
        {
            get => _hudEnabled != null && _hudEnabled.Value;
            set { if (_hudEnabled != null) _hudEnabled.Value = value; }
        }

        public static bool HudOverdraw
        {
            get => _hudOverdraw == null || _hudOverdraw.Value;
            set { if (_hudOverdraw != null) _hudOverdraw.Value = value; }
        }

        public static bool HudSmoothFollow
        {
            get => _hudSmoothFollow == null || _hudSmoothFollow.Value;
            set { if (_hudSmoothFollow != null) _hudSmoothFollow.Value = value; }
        }

        public static float HudOpacity
        {
            get => _hudOpacity != null ? Mathf.Clamp(_hudOpacity.Value, 0.1f, 1.0f) : 0.9f;
            set { if (_hudOpacity != null) _hudOpacity.Value = Mathf.Clamp(value, 0.1f, 1.0f); }
        }

        public static float HudScale
        {
            get => _hudScale != null ? Mathf.Clamp(_hudScale.Value, 0.2f, 1.5f) : 0.45f;
            set { if (_hudScale != null) _hudScale.Value = Mathf.Clamp(value, 0.2f, 1.5f); }
        }

        public static float HudDistance
        {
            get => _hudDistance != null ? Mathf.Clamp(_hudDistance.Value, 0.5f, 3.0f) : 1.2f;
            set { if (_hudDistance != null) _hudDistance.Value = Mathf.Clamp(value, 0.5f, 3.0f); }
        }

        public static float HudOffsetX
        {
            get => _hudOffsetX != null ? Mathf.Clamp(_hudOffsetX.Value, -1.0f, 1.0f) : 0.35f;
            set { if (_hudOffsetX != null) _hudOffsetX.Value = Mathf.Clamp(value, -1.0f, 1.0f); }
        }

        public static float HudOffsetY
        {
            get => _hudOffsetY != null ? Mathf.Clamp(_hudOffsetY.Value, -0.8f, 0.8f) : 0.20f;
            set { if (_hudOffsetY != null) _hudOffsetY.Value = Mathf.Clamp(value, -0.8f, 0.8f); }
        }

        private static MelonPreferences_Entry<T> CreateHiddenEntry<T>(string identifier, T defaultValue, string displayName)
            => _category.CreateEntry(identifier, defaultValue, displayName, description: null, is_hidden: true, dont_save_default: false);

        public static void Init()
        {
            _category = MelonPreferences.CreateCategory("VideoPlayerRemote", "Video Player Remote", is_hidden: true, should_save: true);
            _defaultScreenScale = CreateHiddenEntry("DefaultScreenScale", 1.2f, "Default Local Screen Scale (m)");
            _defaultFollowSelected = CreateHiddenEntry("DefaultFollowSelected", true, "New Screens Follow Selected VideoPlayer");
            _seekStepSeconds = CreateHiddenEntry("SeekStepSeconds", 15f, "Seek Step (Seconds)");
            _autoSelectNearest = CreateHiddenEntry("AutoSelectNearest", true, "Auto-Select Nearest Video Player");
            _playlistShuffle = CreateHiddenEntry("PlaylistShuffle", false, "Playlist Shuffle");
            _playlistLoop = CreateHiddenEntry("PlaylistLoop", true, "Playlist Loop");

            _hudEnabled = CreateHiddenEntry("HudEnabled", false, "Enable HUD / OSD Screen");
            _hudOverdraw = CreateHiddenEntry("HudOverdraw", true, "HUD Always On Top (Overdraw)");
            _hudSmoothFollow = CreateHiddenEntry("HudSmoothFollow", true, "HUD Smooth Follow");
            _hudOpacity = CreateHiddenEntry("HudOpacity", 0.9f, "HUD Opacity");
            _hudScale = CreateHiddenEntry("HudScale", 0.45f, "HUD Width Scale (m)");
            _hudDistance = CreateHiddenEntry("HudDistance", 1.2f, "HUD Distance (m)");
            _hudOffsetX = CreateHiddenEntry("HudOffsetX", 0.35f, "HUD Horizontal Offset");
            _hudOffsetY = CreateHiddenEntry("HudOffsetY", 0.20f, "HUD Vertical Offset");
        }
    }
}
