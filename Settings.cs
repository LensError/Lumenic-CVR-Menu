using System;
using MelonLoader;
using UnityEngine;

namespace LumenicMenu;

internal static class Settings
{
    private static MelonPreferences_Category _category;

    private static MelonPreferences_Entry<bool> _enableMainMenu;
    private static MelonPreferences_Entry<bool> _enableQuickMenu;
    private static MelonPreferences_Entry<bool> _enableHud;
    private static MelonPreferences_Entry<int> _maxRefreshHz;
    private static MelonPreferences_Entry<float> _uiScale;
    private static MelonPreferences_Entry<float> _fontScale;
    private static MelonPreferences_Entry<float> _desktopScrollMultiplier;
    private static MelonPreferences_Entry<float> _vrScrollMultiplier;
    private static MelonPreferences_Entry<bool> _linearTextureMode;
    private static MelonPreferences_Entry<bool> _perfLogging;
    private static MelonPreferences_Entry<bool> _uiClickSounds;
    private static MelonPreferences_Entry<string> _uiSoundPack;
    private static MelonPreferences_Entry<bool> _enableRecentHistory;
    private static MelonPreferences_Entry<bool> _disclaimerAccepted;
    private static MelonPreferences_Entry<string> _lastSeenVersion;
    private static MelonPreferences_Entry<string> _dismissedUpdateVersion;
    private static MelonPreferences_Entry<string> _qmQuickActionsOrder;
    private static MelonPreferences_Entry<string> _mmQuickToolsOrder;

    private const float SaveDebounceSeconds = 0.5f;
    private static bool _savePending;
    private static float _lastDirtyTime;

    public static event Action OnSettingsChanged;

    public static bool EnableMainMenu => _enableMainMenu?.Value ?? true;
    public static bool EnableQuickMenu => _enableQuickMenu?.Value ?? true;
    public static bool EnableHud => _enableHud?.Value ?? true;
    public static int MaxRefreshHz => Mathf.Clamp(_maxRefreshHz?.Value ?? 60, 15, 144);
    public static float UiScale => Mathf.Clamp(_uiScale?.Value ?? 1.0f, 0.5f, 2.0f);
    public static float FontScale => Mathf.Clamp(_fontScale?.Value ?? 1.0f, 0.5f, 2.5f);
    public static float DesktopScrollMultiplier => Mathf.Clamp(_desktopScrollMultiplier?.Value ?? 24.0f, 4.0f, 64.0f);
    public static float VrScrollMultiplier => Mathf.Clamp(_vrScrollMultiplier?.Value ?? 3.0f, 0.5f, 12.0f);
    public static bool LinearTextureMode => _linearTextureMode?.Value ?? false;
    public static bool PerfLogging => _perfLogging?.Value ?? false;
    public static bool UiClickSounds => _uiClickSounds?.Value ?? true;
    public static string UiSoundPack => _uiSoundPack?.Value ?? "SoftSub";
    public static bool EnableRecentHistory => _enableRecentHistory?.Value ?? true;
    public static bool DisclaimerAccepted => _disclaimerAccepted?.Value ?? false;
    public static string LastSeenVersion => _lastSeenVersion?.Value ?? string.Empty;
    public static string DismissedUpdateVersion => _dismissedUpdateVersion?.Value ?? string.Empty;
    public static string QmQuickActionsOrder => _qmQuickActionsOrder?.Value ?? string.Empty;
    public static string MmQuickToolsOrder => _mmQuickToolsOrder?.Value ?? string.Empty;

    public static void Initialize()
    {
        _category = MelonPreferences.CreateCategory("LumenicMenu", "LumenicMenu");

        _enableMainMenu = _category.CreateEntry(
            "EnableMainMenu",
            true,
            "Enable Lumenic Main Menu",
            "Replace CVR's Main Menu with Lumenic UI.");

        _enableQuickMenu = _category.CreateEntry(
            "EnableQuickMenu",
            true,
            "Enable Lumenic Quick Menu",
            "Replace CVR's Quick Menu with Lumenic UI.");

        _enableHud = _category.CreateEntry(
            "EnableHud",
            true,
            "Enable Lumenic HUD",
            "Replace CVR's HUD, ChatBox HUD, and Portal indicator with unified Lumenic UGUI HUD.");

        _maxRefreshHz = _category.CreateEntry(
            "MaxRefreshHz",
            60,
            "Max Redraw Refresh Rate (Hz)",
            "Caps dirty-frame pixel readback frequency (15-144 Hz) to preserve VR frametime.");

        _uiScale = _category.CreateEntry(
            "UiScale",
            1.0f,
            "UI Scale Factor",
            "DPI scale factor passed to Lumenic surfaces (0.5-2.0).");

        _fontScale = _category.CreateEntry(
            "FontScale",
            1.0f,
            "Text Size",
            "Global text size multiplier across Lumenic menus (presets: 75%, 100%, 125%, 150%; range 0.5-2.5).");

        _desktopScrollMultiplier = _category.CreateEntry(
            "DesktopScrollMultiplier",
            24.0f,
            "Desktop Scroll Multiplier",
            "Multiplier converting CVR desktop mouse scrollValue (+/-0.12/notch) into Lumenic scroll lines.");

        _vrScrollMultiplier = _category.CreateEntry(
            "VrScrollMultiplier",
            3.0f,
            "VR Stick Scroll Multiplier",
            "Multiplier converting CVR VR thumbstick per-frame scrollValue into Lumenic scroll lines.");

        _linearTextureMode = _category.CreateEntry(
            "LinearTextureMode",
            false,
            "Texture2D Linear Color Space",
            "Milestone 1 calibration toggle: false = sRGB Texture2D, true = Linear Texture2D.");

        _perfLogging = _category.CreateEntry(
            "PerfLogging",
            false,
            "Stopwatch Perf Logging",
            "Logs step, readback, upload, and blit timings in ms on active frames.");

        _uiClickSounds = _category.CreateEntry(
            "UiClickSounds",
            true,
            "CVR UI Click & Hover Sounds",
            "Plays UI feedback sounds on Lumenic hover and button clicks.");

        _uiSoundPack = _category.CreateEntry(
            "UiSoundPack",
            "SoftSub",
            "UI Sound Pack (SoftSub | Glass | Vanilla)",
            "Selects the Lumenic UI sound pack: 'SoftSub' (default warm click), 'Glass', or 'Vanilla' (CVR InterfaceAudio).");

        _enableRecentHistory = _category.CreateEntry(
            "EnableRecentHistory",
            true,
            "Track Local Recent History",
            "Records and shows local Recent categories (Worlds, Avatars, Props, Users) in UserData/LumenicMenu/recent-history.json.");

        _disclaimerAccepted = _category.CreateEntry(
            "DisclaimerAccepted",
            false,
            "First-Use Disclaimer Accepted",
            "Tracks whether the LumenicMenu first-use support and bug-reporting disclaimer modal has been acknowledged.");

        _lastSeenVersion = _category.CreateEntry(
            "LastSeenVersion",
            string.Empty,
            "Last Seen Mod Version",
            "Tracks the last LumenicMenu version whose bundled changelog modal was shown.");

        _dismissedUpdateVersion = _category.CreateEntry(
            "DismissedUpdateVersion",
            string.Empty,
            "Dismissed Update Version",
            "Version string skipped by the user so the update modal does not reopen for that release.");

        _qmQuickActionsOrder = _category.CreateEntry(
            "QmQuickActionsOrder",
            string.Empty,
            "Quick Menu Quick Actions Tile Order",
            "Comma-separated custom tile order for the Quick Menu Quick Actions 3x3 grid.");

        _mmQuickToolsOrder = _category.CreateEntry(
            "MmQuickToolsOrder",
            string.Empty,
            "Main Menu Quick Tools Tile Order",
            "Comma-separated custom tile order for the Main Menu Quick Tools grid.");

        _enableMainMenu.OnEntryValueChanged.Subscribe((_, _) => NotifyChanged());
        _enableQuickMenu.OnEntryValueChanged.Subscribe((_, _) => NotifyChanged());
        _enableHud.OnEntryValueChanged.Subscribe((_, _) => NotifyChanged());
        _maxRefreshHz.OnEntryValueChanged.Subscribe((_, _) => NotifyChanged());
        _uiScale.OnEntryValueChanged.Subscribe((_, _) => NotifyChanged());
        _fontScale.OnEntryValueChanged.Subscribe((_, _) => NotifyChanged());
        _desktopScrollMultiplier.OnEntryValueChanged.Subscribe((_, _) => NotifyChanged());
        _vrScrollMultiplier.OnEntryValueChanged.Subscribe((_, _) => NotifyChanged());
        _linearTextureMode.OnEntryValueChanged.Subscribe((_, _) => NotifyChanged());
        _perfLogging.OnEntryValueChanged.Subscribe((_, _) => NotifyChanged());
        _uiClickSounds.OnEntryValueChanged.Subscribe((_, _) => NotifyChanged());
        _uiSoundPack.OnEntryValueChanged.Subscribe((_, _) => NotifyChanged());
        _enableRecentHistory.OnEntryValueChanged.Subscribe((_, _) => NotifyChanged());
        _disclaimerAccepted.OnEntryValueChanged.Subscribe((_, _) => NotifyChanged());
        _lastSeenVersion.OnEntryValueChanged.Subscribe((_, _) => NotifyChanged());
        _dismissedUpdateVersion.OnEntryValueChanged.Subscribe((_, _) => NotifyChanged());
        _qmQuickActionsOrder.OnEntryValueChanged.Subscribe((_, _) => NotifyChanged());
        _mmQuickToolsOrder.OnEntryValueChanged.Subscribe((_, _) => NotifyChanged());
    }

    public static void SetQmQuickActionsOrder(string csv)
    {
        string safe = csv ?? string.Empty;
        if (_qmQuickActionsOrder != null && _qmQuickActionsOrder.Value != safe)
        {
            _qmQuickActionsOrder.Value = safe;
            ScheduleSave();
        }
    }

    public static void SetMmQuickToolsOrder(string csv)
    {
        string safe = csv ?? string.Empty;
        if (_mmQuickToolsOrder != null && _mmQuickToolsOrder.Value != safe)
        {
            _mmQuickToolsOrder.Value = safe;
            ScheduleSave();
        }
    }

    public static void SetUiClickSounds(bool value)
    {
        if (_uiClickSounds != null && _uiClickSounds.Value != value)
        {
            _uiClickSounds.Value = value;
            ScheduleSave();
        }
    }

    public static void SetUiSoundPack(string pack)
    {
        if (_uiSoundPack != null && !string.IsNullOrWhiteSpace(pack) && _uiSoundPack.Value != pack)
        {
            _uiSoundPack.Value = pack;
            ScheduleSave();
        }
    }

    public static void SetMaxRefreshHz(int hz)
    {
        int clamped = Mathf.Clamp(hz, 15, 144);
        if (_maxRefreshHz != null && _maxRefreshHz.Value != clamped)
        {
            _maxRefreshHz.Value = clamped;
            ScheduleSave();
        }
    }

    public static void SetFontScale(float scale)
    {
        float clamped = Mathf.Clamp(scale, 0.5f, 2.5f);
        if (_fontScale != null && Math.Abs(_fontScale.Value - clamped) > 0.001f)
        {
            _fontScale.Value = clamped;
            ScheduleSave();
        }
    }

    public static void SetEnableMainMenu(bool value)
    {
        if (_enableMainMenu != null && _enableMainMenu.Value != value)
        {
            _enableMainMenu.Value = value;
            ScheduleSave();
        }
    }

    public static void SetEnableQuickMenu(bool value)
    {
        if (_enableQuickMenu != null && _enableQuickMenu.Value != value)
        {
            _enableQuickMenu.Value = value;
            ScheduleSave();
        }
    }

    public static void SetEnableHud(bool value)
    {
        if (_enableHud != null && _enableHud.Value != value)
        {
            _enableHud.Value = value;
            ScheduleSave();
        }
    }

    public static void SetLinearTextureMode(bool value)
    {
        if (_linearTextureMode != null && _linearTextureMode.Value != value)
        {
            _linearTextureMode.Value = value;
            ScheduleSave();
        }
    }

    public static void SetEnableRecentHistory(bool value)
    {
        if (_enableRecentHistory != null && _enableRecentHistory.Value != value)
        {
            _enableRecentHistory.Value = value;
            ScheduleSave();
        }
    }

    public static void SetDisclaimerAccepted(bool value)
    {
        if (_disclaimerAccepted != null && _disclaimerAccepted.Value != value)
        {
            _disclaimerAccepted.Value = value;
            ScheduleSave();
        }
    }

    public static void SetLastSeenVersion(string version)
    {
        string safe = version ?? string.Empty;
        if (_lastSeenVersion != null && _lastSeenVersion.Value != safe)
        {
            _lastSeenVersion.Value = safe;
            ScheduleSave();
        }
    }

    public static void SetDismissedUpdateVersion(string version)
    {
        string safe = version ?? string.Empty;
        if (_dismissedUpdateVersion != null && _dismissedUpdateVersion.Value != safe)
        {
            _dismissedUpdateVersion.Value = safe;
            ScheduleSave();
        }
    }

    public static void SetDesktopScrollMultiplier(float value)
    {
        float clamped = Mathf.Clamp(value, 4.0f, 64.0f);
        if (_desktopScrollMultiplier != null && Math.Abs(_desktopScrollMultiplier.Value - clamped) > 0.01f)
        {
            _desktopScrollMultiplier.Value = clamped;
            ScheduleSave();
        }
    }

    public static void SetVrScrollMultiplier(float value)
    {
        float clamped = Mathf.Clamp(value, 0.5f, 12.0f);
        if (_vrScrollMultiplier != null && Math.Abs(_vrScrollMultiplier.Value - clamped) > 0.01f)
        {
            _vrScrollMultiplier.Value = clamped;
            ScheduleSave();
        }
    }

    public static void SetPerfLogging(bool value)
    {
        if (_perfLogging != null && _perfLogging.Value != value)
        {
            _perfLogging.Value = value;
            ScheduleSave();
        }
    }

    private static void ScheduleSave()
    {
        _savePending = true;
        _lastDirtyTime = Time.unscaledTime;
    }

    public static void FlushPendingSave(float now, bool force = false)
    {
        if (!_savePending)
            return;

        if (!force && (now - _lastDirtyTime) < SaveDebounceSeconds)
            return;

        _savePending = false;
        MelonPreferences.Save();
    }

    private static void NotifyChanged()
    {
        OnSettingsChanged?.Invoke();
    }
}
