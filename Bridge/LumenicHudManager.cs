using System;
using System.Collections.Generic;
using System.Reflection;
using ABI_RC.Core.Player;
using ABI_RC.Core.Savior;
using ABI_RC.Core.UI;
using ABI_RC.Core.UI.Hud;
using ABI_RC.Core.UI.UIRework.Managers;
using ABI_RC.Systems.Communications;
using ABI_RC.Systems.PlayerColors;
using HarmonyLib;
using LumenicMenu.Native;
using MelonLoader;
using TMPro;
using UnityEngine;
using UnityEngine.UI;

namespace LumenicMenu.Bridge;

/// <summary>
/// Unified native Unity UGUI HUD replacing CohtmlHud, ChatBoxHud, and PortalHudIndicator.
/// Petite, icon-first dark glass styling, 100% on-GPU with zero texture readback.
/// </summary>
internal sealed class LumenicHudManager : IDisposable
{
    private const float ReferenceWidth = 1280f;
    private const float ReferenceHeight = 720f;
    private const int MaxVisibleToasts = 3;
    private const int MaxFeedRows = 4;
    private const float FeedLifetime = 4.5f;

    // Mirrors native-ui/src/theme.rs tokens so the HUD reads as part of the menu.
    private static Color BgGlass = new Color(0.133f, 0.133f, 0.133f, 0.91f);
    private static Color BgChip = new Color(0.165f, 0.165f, 0.165f, 0.92f);
    private static Color BorderSubtle = new Color(1f, 1f, 1f, 0.13f);
    private static readonly Color TextPrimary = new Color(1f, 1f, 1f, 1f);
    private static readonly Color TextMuted = new Color(0.69f, 0.69f, 0.69f, 1f);
    private static Color AccentSky = new Color(0.894f, 0.212f, 0f, 1f);
    private static readonly Color AccentGreen = new Color(0.216f, 0.741f, 0.235f, 1f);
    private static readonly Color AccentAmber = new Color(0.941f, 0.647f, 0.188f, 1f);
    private static readonly Color AccentRed = new Color(0.937f, 0.267f, 0.267f, 1f);
    private static readonly Color AccentBlue = new Color(0.220f, 0.741f, 0.973f, 1f);
    private static Color AccentPurple => AccentSky;

    private static readonly FieldInfo CohtmlHudViewField = AccessTools.Field(typeof(CohtmlHud), "hudView");
    private static readonly FieldInfo CohtmlHudRendererField = AccessTools.Field(typeof(CohtmlHud), "_hudRenderer");
    private static readonly FieldInfo CohtmlHudCanvasField = AccessTools.Field(typeof(CohtmlHud), "_hudCanvas");
    private static readonly FieldInfo PortalTextField = AccessTools.Field(typeof(PortalHudIndicator), "text");

    private enum HudIcon
    {
        MicOn,
        MicOff,
        Friends,
        Invite,
        Vote,
        Alert,
        Info,
        Flight,
        Lock,
        Comms,
        Net,
        Prop,
        Join,
        Leave
    }

    private sealed class FeedEntry
    {
        public string Id;
        public string Name;
        public bool Joined;
        public float CreatedTime;
        public float ExpireTime;
    }

    private sealed class FeedRowRefs
    {
        public GameObject Root;
        public CanvasGroup Group;
        public Image Icon;
        public TMP_Text NameText;
    }

    private sealed class ToastEntry
    {
        public string Id;
        public string Category;
        public string Headline;
        public string Body;
        public float ExpireTime;
        public float CreatedTime;
    }

    private sealed class ToastCardRefs
    {
        public GameObject Root;
        public CanvasGroup Group;
        public Image Icon;
        public TMP_Text CategoryText;
        public TMP_Text HeadlineText;
        public TMP_Text BodyText;
    }

    private GameObject _rootGo;
    private RectTransform _rootRt;
    private Canvas _canvas;
    private CanvasGroup _rootGroup;
    private Sprite _roundedSprite;
    private Sprite _ringSprite;
    private readonly Dictionary<HudIcon, Sprite> _icons = new Dictionary<HudIcon, Sprite>();
    private TMP_FontAsset _font;
    private PortalHudIndicator _cachedPortal;
    private bool _built;
    private bool _prevEnabled;
    private readonly List<(Image Bg, Image Border, bool IsChip)> _themedCards = new List<(Image, Image, bool)>();
    private readonly List<Image> _accentImages = new List<Image>();
    private TMP_Text _ttsText;
    private string _lastThemeKey = string.Empty;

    // Status & Mic
    private RectTransform _statusClusterRt;
    private RectTransform _statusRowRt;
    private GameObject _micCard;
    private float _lastAppliedAlertsScale = -1f;
    private float _lastAppliedMicScale = -1f;
    private Image _micIcon;
    private Image _micFillIcon;
    private Image _micTile;
    private Image _micTileBorder;
    private float _micDisplay;
    private GameObject _ttsChip;
    private GameObject _commsChip;
    private GameObject _netChip;
    private Image _netIcon;
    private GameObject _flightChip;
    private Image _flightIcon;
    private GameObject _gestureChip;
    private GameObject _skeletalChip;
    private GameObject _moveChip;

    // Notification Badges (petite icon chips)
    private GameObject _badgesRow;
    private GameObject _badgeFriends;
    private GameObject _badgeInvites;
    private GameObject _badgeVotes;
    private GameObject _badgeAlerts;
    private GameObject _badgeInfo;

    // Toasts
    private RectTransform _toastStackRt;
    private readonly List<ToastEntry> _toasts = new List<ToastEntry>();
    private readonly List<ToastCardRefs> _toastCards = new List<ToastCardRefs>();
    private bool _toastsDirty = true;

    // Join / leave feed (lower left)
    private RectTransform _joinFeedRt;
    private readonly List<FeedEntry> _joinFeed = new List<FeedEntry>();
    private readonly List<FeedRowRefs> _feedRows = new List<FeedRowRefs>();
    private bool _feedDirty = true;

    // Prop Spawn
    private GameObject _propCard;
    private TMP_Text _propName;

    // Unified ChatBox + Portal Indicator
    private GameObject _bottomCenterCard;
    private CanvasGroup _bottomCenterGroup;
    private TMP_Text _bottomCenterText;
    private LayoutElement _bottomCenterTextLayout;

    // Live State
    private bool _micVisible = true;
    private bool _micActive;
    private float _micLevel;
    private bool _ttsActive;
    private bool _commsWarning;
    private int _netChainState;
    private bool _isFlying;
    private bool _isNoClip;
    private bool _gestureLocked;
    private bool _skeletalLocked;
    private bool _movementLocked;
    private float _mutedSpeechPulseUntil;
    private float _lastActivityTime;
    private float _hudOpacity = 0.6f;
    private bool _mutedSpeechEnabled = true;
    private bool _smoothMicMeter = true;
    private bool _showForMicUpdates = true;
    private float _nextSettingsPoll;
    private float _builtTime;
    private bool _loggedState;

    // Notification badge states (live + test overrides)
    private bool _hasFriends;
    private bool _hasInvites;
    private bool _hasVotes;
    private bool _hasAlerts;
    private bool _hasInfo;
    private bool _testFriends;
    private bool _testInvites;
    private bool _testVotes;
    private bool _testAlerts;
    private bool _testInfo;

    // ChatBox / Portal state
    private string _chatOrPortalText = string.Empty;
    private bool _chatOrPortalActive;
    private bool _isTestPortalActive;
    private float _portalPulsePhase;
    private float _previewAllUntil;

    public void TriggerLayoutPreview(float seconds = 3.5f)
    {
        float now = Time.unscaledTime;
        _previewAllUntil = now + seconds;
        _lastActivityTime = now;
    }

    public static bool IsVanillaHudView(CohtmlControlledView view)
    {
        if (!CohtmlHud.IsReady || view == null || CohtmlHud.Instance == null || CohtmlHudViewField == null)
            return false;

        try
        {
            return ReferenceEquals(view, CohtmlHudViewField.GetValue(CohtmlHud.Instance));
        }
        catch
        {
            return false;
        }
    }

    public void Tick()
    {
        bool enabled = Settings.EnableHud;
        if (enabled != _prevEnabled)
        {
            _prevEnabled = enabled;
            OnEnableChanged(enabled);
        }

        if (!enabled)
            return;

        if (!_built)
        {
            if (!TryBuildUi())
                return;
        }

        if (Time.unscaledTime >= _nextSettingsPoll)
        {
            _nextSettingsPoll = Time.unscaledTime + 0.5f;
            SyncFromVanillaSettings();
        }

        SuppressVanillaRenderers();
        bool previewAll = Time.unscaledTime < _previewAllUntil;
        UpdateCanvasTransformAndVisibility(previewAll);
        UpdateToastLifecycle(previewAll);
        UpdateMicAndStatusWidgets(previewAll);
        UpdateJoinFeed(previewAll);
        UpdateNotificationBadges(previewAll);
        UpdateBottomCenterCard(previewAll);
    }

    private void OnEnableChanged(bool enabled)
    {
        if (_rootGo != null)
            _rootGo.SetActive(enabled);

        if (enabled)
        {
            SuppressVanillaRenderers();
            SyncFromVanillaSettings();
        }
        else
        {
            RestoreVanillaRenderers();
        }
    }

    private void SuppressVanillaRenderers()
    {
        if (CohtmlHud.IsReady && CohtmlHud.Instance != null)
        {
            if (CohtmlHudRendererField?.GetValue(CohtmlHud.Instance) is Renderer r && !r.forceRenderingOff)
                r.forceRenderingOff = true;
        }

        if (_cachedPortal == null)
            _cachedPortal = UnityEngine.Object.FindAnyObjectByType<PortalHudIndicator>();

        if (_cachedPortal != null && PortalTextField?.GetValue(_cachedPortal) is TMP_Text tmp && tmp.enabled)
            tmp.enabled = false;
    }

    private void RestoreVanillaRenderers()
    {
        if (CohtmlHud.Instance != null)
        {
            CohtmlHud.Instance.RestoreHud();
            if (CohtmlHudViewField?.GetValue(CohtmlHud.Instance) is CohtmlControlledView view)
                MenuSurfaceHost.RestoreVanillaFrame(view);
        }

        if (_cachedPortal == null)
            _cachedPortal = UnityEngine.Object.FindAnyObjectByType<PortalHudIndicator>();

        if (_cachedPortal != null && PortalTextField?.GetValue(_cachedPortal) is TMP_Text tmp)
            tmp.enabled = true;
    }

    public void SyncFromVanillaSettings()
    {
        _micActive = !Comms_Manager.IsMicMuted;

        if (MetaPort.Instance?.settings == null)
            return;

        var settings = MetaPort.Instance.settings;
        _micVisible = settings.GetSettingsBool("HUDCustomizationMicrophone");
        _hudOpacity = Mathf.Clamp(settings.GetSettingsInt("HUDCustomizationOpacity", 60) / 100f, 0.1f, 1f);
        _mutedSpeechEnabled = settings.GetSettingsBool("HUDCustomizationMutedSpeech");
        _smoothMicMeter = settings.GetSettingsBool("HUDCustomizationSmoothMicMeter", true);

        if (UIMessageManager.Instance != null)
        {
            _hasFriends = settings.GetSettingsBool("HUDCustomizationFriendRequests") && UIMessageManager.Instance.HasFriendRequestMessage;
            _hasInvites = (settings.GetSettingsBool("HUDCustomizationInvites") && UIMessageManager.Instance.HasInviteMessage) || UIMessageManager.Instance.HasInviteRequestMessage;
            _hasVotes = settings.GetSettingsBool("HUDCustomizationVotes") && UIMessageManager.Instance.HasVoteMessage;
            _hasAlerts = settings.GetSettingsBool("HUDCustomizationAlerts") && UIMessageManager.Instance.HasSystemMessage;
            _hasInfo = (settings.GetSettingsBool("HUDCustomizationInfo") && UIMessageManager.Instance.HasOtherMessage) || UIMessageManager.Instance.HasGroupMessage;
        }

        ApplyHudTheme();
    }

    private void ApplyHudTheme()
    {
        string preset = Settings.ThemePreset;
        float opacity = Settings.PanelOpacity;
        bool colorize = MetaPort.Instance?.settings != null
            && MetaPort.Instance.settings.GetSettingsBool(PlayerColorsManager.ColourizeMenuSettingKey);
        Color32 p1 = PlayerColorsManager.CurrentColors.PrimaryColor;
        string key = $"{preset}|{opacity:F2}|{colorize}|{p1.r},{p1.g},{p1.b}";
        if (string.Equals(key, _lastThemeKey, StringComparison.Ordinal))
            return;
        _lastThemeKey = key;

        Color glass = preset switch
        {
            "oled" => new Color(0.039f, 0.039f, 0.047f, 0.96f),
            "frost" => new Color(0.141f, 0.165f, 0.212f, 0.78f),
            "contrast" => new Color(0.055f, 0.055f, 0.063f, 0.98f),
            _ => new Color(0.133f, 0.133f, 0.133f, 0.91f),
        };
        Color chip = preset switch
        {
            "oled" => new Color(0.086f, 0.086f, 0.106f, 0.94f),
            "frost" => new Color(0.188f, 0.220f, 0.282f, 0.77f),
            "contrast" => new Color(0.118f, 0.118f, 0.141f, 0.97f),
            _ => new Color(0.165f, 0.165f, 0.165f, 0.92f),
        };
        Color border = preset switch
        {
            "oled" => new Color(1f, 1f, 1f, 0.11f),
            "frost" => new Color(1f, 1f, 1f, 0.23f),
            "contrast" => new Color(1f, 1f, 1f, 0.44f),
            _ => new Color(1f, 1f, 1f, 0.13f),
        };

        Color accent = new Color(0.894f, 0.212f, 0f, 1f);
        if (colorize && (p1.r | p1.g | p1.b) != 0)
        {
            accent = new Color(p1.r / 255f, p1.g / 255f, p1.b / 255f, 1f);
            float luma = 0.299f * p1.r + 0.587f * p1.g + 0.114f * p1.b;
            float tameScale = luma > 118f ? 118f / luma : 1f;
            Color tame = new Color((p1.r * tameScale) / 255f, (p1.g * tameScale) / 255f, (p1.b * tameScale) / 255f, 1f);
            glass = new Color(Mathf.Lerp(glass.r, tame.r, 0.20f), Mathf.Lerp(glass.g, tame.g, 0.20f), Mathf.Lerp(glass.b, tame.b, 0.20f), glass.a);
            chip = new Color(Mathf.Lerp(chip.r, tame.r, 0.16f), Mathf.Lerp(chip.g, tame.g, 0.16f), Mathf.Lerp(chip.b, tame.b, 0.16f), chip.a);
            float bScale = luma > 185f ? 185f / luma : 1f;
            border = new Color((p1.r * bScale) / 255f, (p1.g * bScale) / 255f, (p1.b * bScale) / 255f, Mathf.Max(border.a, 0.23f));
        }

        glass.a *= opacity;
        chip.a *= opacity;

        BgGlass = glass;
        BgChip = chip;
        BorderSubtle = border;
        AccentSky = accent;

        for (int i = 0; i < _themedCards.Count; i++)
        {
            var (bg, bd, isChip) = _themedCards[i];
            if (bg != null)
                bg.color = isChip ? BgChip : BgGlass;
            if (bd != null)
                bd.color = BorderSubtle;
        }

        for (int i = 0; i < _accentImages.Count; i++)
        {
            if (_accentImages[i] != null)
                _accentImages[i].color = AccentSky;
        }

        if (_ttsText != null)
            _ttsText.color = AccentSky;
    }

    // --- Event Receivers ---

    public void PushToast(string category, string headline, string body, int seconds, bool immediate, string overrideId)
    {
        if (string.IsNullOrWhiteSpace(headline) && string.IsNullOrWhiteSpace(body))
            return;

        if (!string.IsNullOrEmpty(overrideId) && overrideId.StartsWith("userJoinLeft", StringComparison.Ordinal))
        {
            PushJoinLeave(headline, overrideId);
            return;
        }

        float now = Time.unscaledTime;
        float duration = Mathf.Clamp(seconds > 0 ? seconds : 4, 2f, 15f);

        if (!string.IsNullOrEmpty(overrideId))
        {
            for (int i = _toasts.Count - 1; i >= 0; i--)
            {
                if (string.Equals(_toasts[i].Id, overrideId, StringComparison.Ordinal))
                    _toasts.RemoveAt(i);
            }
        }

        var entry = new ToastEntry
        {
            Id = overrideId ?? string.Empty,
            Category = string.IsNullOrWhiteSpace(category) ? "Notice" : category.Trim(),
            Headline = headline?.Trim() ?? string.Empty,
            Body = body?.Trim() ?? string.Empty,
            CreatedTime = now,
            ExpireTime = now + duration
        };

        if (immediate)
            _toasts.Insert(0, entry);
        else
            _toasts.Add(entry);

        while (_toasts.Count > 6)
            _toasts.RemoveAt(_toasts.Count - 1);

        _toastsDirty = true;
        TouchActivity();
    }

    public void ClearToasts()
    {
        _toasts.Clear();
        _toastsDirty = true;
    }

    private void PushJoinLeave(string headline, string id)
    {
        string text = headline?.Trim() ?? string.Empty;
        bool joined = text.EndsWith(" joined", StringComparison.OrdinalIgnoreCase);
        bool left = text.EndsWith(" left", StringComparison.OrdinalIgnoreCase);
        string name = joined ? text.Substring(0, text.Length - 7) : (left ? text.Substring(0, text.Length - 5) : text);
        if (name.Length > 24)
            name = name.Substring(0, 22) + "..";

        for (int i = _joinFeed.Count - 1; i >= 0; i--)
        {
            if (string.Equals(_joinFeed[i].Id, id, StringComparison.Ordinal))
                _joinFeed.RemoveAt(i);
        }

        float now = Time.unscaledTime;
        _joinFeed.Add(new FeedEntry
        {
            Id = id,
            Name = name,
            Joined = !left,
            CreatedTime = now,
            ExpireTime = now + FeedLifetime
        });
        while (_joinFeed.Count > 8)
            _joinFeed.RemoveAt(0);

        _feedDirty = true;
        TouchActivity();
    }

    public void SetMicActive(bool active)
    {
        if (_micActive != active)
        {
            _micActive = active;
            TouchMic();
        }
    }

    public void SetMicLevel(float level)
    {
        _micLevel = Mathf.Clamp01(level);
        if (_micLevel > 0.03f)
            TouchMic();
    }

    public void SetTtsActive(bool active)
    {
        _ttsActive = active;
        if (active) TouchActivity();
    }

    public void SetCommsIndicator(bool shown)
    {
        _commsWarning = shown;
        if (shown) TouchActivity();
    }

    public void OnMutedSpeechDetected()
    {
        if (!_mutedSpeechEnabled)
            return;

        _mutedSpeechPulseUntil = Time.unscaledTime + 1.4f;
        TouchMic();
    }

    public void SetFlightState(bool isFlying, bool noClip)
    {
        _isFlying = isFlying;
        _isNoClip = noClip;
        TouchActivity();
    }

    public void SetGestureLock(bool locked)
    {
        _gestureLocked = locked;
        TouchActivity();
    }

    public void SetSkeletalLock(bool locked)
    {
        _skeletalLocked = locked;
        TouchActivity();
    }

    public void SetMovementLock(bool locked)
    {
        _movementLocked = locked;
        TouchActivity();
    }

    public void SetNetworkHealth(int state)
    {
        _netChainState = state;
        if (state > 0) TouchActivity();
    }

    public void UpdateNotificationCategory(UIMessageCategory category)
    {
        if (MetaPort.Instance?.settings == null || UIMessageManager.Instance == null)
            return;

        var settings = MetaPort.Instance.settings;
        switch (category)
        {
            case UIMessageCategory.FriendRequests:
                _hasFriends = settings.GetSettingsBool("HUDCustomizationFriendRequests") && UIMessageManager.Instance.HasFriendRequestMessage;
                break;
            case UIMessageCategory.Invite:
            case UIMessageCategory.InviteRequests:
                _hasInvites = (settings.GetSettingsBool("HUDCustomizationInvites") && UIMessageManager.Instance.HasInviteMessage) || UIMessageManager.Instance.HasInviteRequestMessage;
                break;
            case UIMessageCategory.Votes:
                _hasVotes = settings.GetSettingsBool("HUDCustomizationVotes") && UIMessageManager.Instance.HasVoteMessage;
                break;
            case UIMessageCategory.SystemNotif:
                _hasAlerts = settings.GetSettingsBool("HUDCustomizationAlerts") && UIMessageManager.Instance.HasSystemMessage;
                break;
            case UIMessageCategory.Other:
            case UIMessageCategory.Group:
                _hasInfo = (settings.GetSettingsBool("HUDCustomizationInfo") && UIMessageManager.Instance.HasOtherMessage) || UIMessageManager.Instance.HasGroupMessage;
                break;
        }
        TouchActivity();
    }

    public void SetPropPreview(bool active, string headline, string propName)
    {
        if (_propCard == null)
            return;

        _propCard.SetActive(active);
        if (active)
        {
            if (_propName != null)
                _propName.text = string.IsNullOrWhiteSpace(propName) ? (headline ?? "Prop") : propName;
            TouchActivity();
        }
    }

    public void SetChatOrPortalText(string text, PortalHudIndicator source = null)
    {
        if (source != null)
            SuppressPortalInstance(source);

        _chatOrPortalText = text ?? string.Empty;
        if (!string.IsNullOrEmpty(_chatOrPortalText))
            TouchActivity();
    }

    public void SetChatOrPortalActive(bool active, PortalHudIndicator source = null)
    {
        if (source != null)
            SuppressPortalInstance(source);

        _chatOrPortalActive = active;
        if (active)
        {
            _portalPulsePhase = 0f;
            TouchActivity();
        }
    }

    private void SuppressPortalInstance(PortalHudIndicator source)
    {
        if (!Settings.EnableHud || source == null)
            return;

        _cachedPortal = source;
        if (PortalTextField?.GetValue(source) is TMP_Text tmp && tmp.enabled)
            tmp.enabled = false;
    }

    // --- QuickMenu Test Helpers ---

    public void TriggerTestBadge(string kind)
    {
        switch (kind)
        {
            case "friends": _testFriends = !_testFriends; break;
            case "invites": _testInvites = !_testInvites; break;
            case "votes": _testVotes = !_testVotes; break;
            case "alerts": _testAlerts = !_testAlerts; break;
            case "info": _testInfo = !_testInfo; break;
        }
        TouchActivity();
    }

    public void ToggleTestPortal()
    {
        _isTestPortalActive = !_isTestPortalActive;
        if (_isTestPortalActive)
        {
            SetChatOrPortalText("<b><color=#38BDF8>Portal Ready</color></b>  <size=85%><color=#99A7C3>Look at portal to enter</color></size>");
            SetChatOrPortalActive(true);
        }
        else
        {
            SetChatOrPortalActive(false);
        }
    }

    public void TriggerTestChatBox()
    {
        SetChatOrPortalText("<b><color=#38BDF8>LensError</color></b>\n<size=90%>Hello from the unified Lumenic HUD!</size>");
        SetChatOrPortalActive(true);
        _isTestPortalActive = false;
    }

    public void ClearAllTests()
    {
        _testFriends = false;
        _testInvites = false;
        _testVotes = false;
        _testAlerts = false;
        _testInfo = false;
        _isTestPortalActive = false;
        _chatOrPortalActive = false;
        _chatOrPortalText = string.Empty;
        ClearToasts();
        _joinFeed.Clear();
        _feedDirty = true;
        SetPropPreview(false, string.Empty, string.Empty);
    }

    private void TouchActivity()
    {
        _lastActivityTime = Time.unscaledTime;
    }

    // "Only updates" display mode must not wake the HUD for voice activity.
    private void TouchMic()
    {
        if (_showForMicUpdates)
            TouchActivity();
    }

    // --- UI Construction ---

    private bool TryBuildUi()
    {
        Camera uiCam = PlayerSetup.Instance?.activeUiCam;
        if (uiCam == null)
            return false;

        _font = ResolveFontAsset();
        _roundedSprite = CreateChamferSprite(24, 5f, false);
        _ringSprite = CreateChamferSprite(24, 5f, true);
        BuildIconSprites();

        _rootGo = new GameObject("LumenicHudCanvas");
        _rootGo.layer = uiCam.gameObject.layer != 0 ? uiCam.gameObject.layer : 5;
        _rootGo.transform.SetParent(uiCam.transform, false);

        _canvas = _rootGo.AddComponent<Canvas>();
        _canvas.renderMode = RenderMode.WorldSpace;
        _canvas.worldCamera = uiCam;
        _canvas.sortingOrder = 50;

        var scaler = _rootGo.AddComponent<CanvasScaler>();
        scaler.dynamicPixelsPerUnit = 2f;

        _rootGroup = _rootGo.AddComponent<CanvasGroup>();
        _rootGroup.interactable = false;
        _rootGroup.blocksRaycasts = false;

        var rootRt = _rootGo.GetComponent<RectTransform>();
        _rootRt = rootRt;
        rootRt.sizeDelta = new Vector2(ReferenceWidth, ReferenceHeight);
        rootRt.pivot = new Vector2(0.5f, 0.5f);

        BuildToastContainer(rootRt);
        BuildNotificationBadges(rootRt);
        BuildStatusCluster(rootRt);
        BuildPropCard(rootRt);
        BuildBottomCenterCard(rootRt);

        _built = true;
        _builtTime = Time.unscaledTime;
        MelonLogger.Msg("[LumenicMenu][HUD] Canvas built.");
        SyncFromVanillaSettings();
        return true;
    }

    private void BuildToastContainer(RectTransform parent)
    {
        var container = CreateRect("ToastStack", parent, new Vector2(0.5f, 1f), new Vector2(0.5f, 1f), new Vector2(0.5f, 1f), new Vector2(0f, -18f), new Vector2(340f, 240f));
        _toastStackRt = container;
        var layout = container.gameObject.AddComponent<VerticalLayoutGroup>();
        layout.spacing = 6f;
        layout.childAlignment = TextAnchor.UpperCenter;
        layout.childControlWidth = true;
        layout.childControlHeight = false;
        layout.childForceExpandWidth = true;
        layout.childForceExpandHeight = false;

        for (int i = 0; i < MaxVisibleToasts; i++)
        {
            var cardRt = CreateCard($"ToastCard_{i}", container, BgGlass, 340f, 50f);
            var cg = cardRt.gameObject.AddComponent<CanvasGroup>();

            var hlg = cardRt.gameObject.AddComponent<HorizontalLayoutGroup>();
            hlg.padding = new RectOffset(10, 12, 7, 7);
            hlg.spacing = 9f;
            hlg.childAlignment = TextAnchor.MiddleLeft;
            hlg.childControlWidth = true;
            hlg.childControlHeight = true;
            hlg.childForceExpandWidth = false;
            hlg.childForceExpandHeight = false;

            var csf = cardRt.gameObject.AddComponent<ContentSizeFitter>();
            csf.horizontalFit = ContentSizeFitter.FitMode.Unconstrained;
            csf.verticalFit = ContentSizeFitter.FitMode.PreferredSize;

            var iconImg = CreateIcon("Icon", cardRt, HudIcon.Info, 16f, AccentSky);

            var textCol = CreateRect("TextCol", cardRt, new Vector2(0.5f, 0.5f), new Vector2(0.5f, 0.5f), new Vector2(0.5f, 0.5f), Vector2.zero, new Vector2(290f, 36f));
            var textLe = textCol.gameObject.AddComponent<LayoutElement>();
            textLe.flexibleWidth = 1f;
            var vlg = textCol.gameObject.AddComponent<VerticalLayoutGroup>();
            vlg.spacing = 1f;
            vlg.childControlWidth = true;
            vlg.childControlHeight = true;
            vlg.childForceExpandWidth = true;
            vlg.childForceExpandHeight = false;

            var headerRow = CreateRect("HeaderRow", textCol, new Vector2(0.5f, 0.5f), new Vector2(0.5f, 0.5f), new Vector2(0.5f, 0.5f), Vector2.zero, new Vector2(290f, 16f));
            var headerHlg = headerRow.gameObject.AddComponent<HorizontalLayoutGroup>();
            headerHlg.spacing = 6f;
            headerHlg.childAlignment = TextAnchor.MiddleLeft;
            headerHlg.childControlWidth = true;
            headerHlg.childControlHeight = true;
            headerHlg.childForceExpandWidth = false;

            var headTmp = CreateText("Headline", headerRow, 12.5f, TextPrimary, FontStyles.Bold, TextAlignmentOptions.Left);
            var headLe = headTmp.gameObject.AddComponent<LayoutElement>();
            headLe.flexibleWidth = 1f;

            var catTmp = CreateText("Category", headerRow, 10f, TextMuted, FontStyles.Normal, TextAlignmentOptions.Right);

            var bodyTmp = CreateText("Body", textCol, 11.5f, TextMuted, FontStyles.Normal, TextAlignmentOptions.Left);
            bodyTmp.textWrappingMode = TextWrappingModes.Normal;

            cardRt.gameObject.SetActive(false);
            _toastCards.Add(new ToastCardRefs
            {
                Root = cardRt.gameObject,
                Group = cg,
                Icon = iconImg,
                CategoryText = catTmp,
                HeadlineText = headTmp,
                BodyText = bodyTmp
            });
        }
    }

    private void BuildNotificationBadges(RectTransform parent)
    {
        var row = CreateRect("BadgesRow", parent, new Vector2(1f, 1f), new Vector2(1f, 1f), new Vector2(1f, 1f), new Vector2(-18f, -18f), new Vector2(200f, 28f));
        var hlg = row.gameObject.AddComponent<HorizontalLayoutGroup>();
        hlg.spacing = 6f;
        hlg.childAlignment = TextAnchor.MiddleRight;
        hlg.childControlWidth = false;
        hlg.childControlHeight = false;
        hlg.childForceExpandWidth = false;
        hlg.childForceExpandHeight = false;

        _badgesRow = row.gameObject;
        _badgeFriends = CreateIconChip("Friends", row, HudIcon.Friends, AccentSky, 24f, 13f, out var friendsIcon);
        _accentImages.Add(friendsIcon);
        _badgeInvites = CreateIconChip("Invites", row, HudIcon.Invite, AccentGreen, 24f, 13f);
        _badgeVotes = CreateIconChip("Votes", row, HudIcon.Vote, AccentAmber, 24f, 13f);
        _badgeAlerts = CreateIconChip("Alerts", row, HudIcon.Alert, AccentRed, 24f, 13f);
        _badgeInfo = CreateIconChip("Info", row, HudIcon.Info, TextMuted, 24f, 13f);
    }

    private static void AddLayout(GameObject go, float w, float h)
    {
        var le = go.GetComponent<LayoutElement>() ?? go.AddComponent<LayoutElement>();
        le.preferredWidth = w;
        le.preferredHeight = h;
        le.minWidth = w;
        le.minHeight = h;
    }

    private void BuildStatusCluster(RectTransform parent)
    {
        float feedH = MaxFeedRows * 20f + (MaxFeedRows - 1) * 3f;
        var col = CreateRect("StatusCluster", parent, Vector2.zero, Vector2.zero, Vector2.zero, new Vector2(16f, 16f), new Vector2(320f, feedH + 6f + 34f));
        _statusClusterRt = col;
        var vlg = col.gameObject.AddComponent<VerticalLayoutGroup>();
        vlg.spacing = 6f;
        vlg.childAlignment = TextAnchor.LowerLeft;
        vlg.childControlWidth = false;
        vlg.childControlHeight = false;
        vlg.childForceExpandWidth = false;
        vlg.childForceExpandHeight = false;

        var feed = CreateRect("JoinFeed", col, Vector2.zero, Vector2.zero, Vector2.zero, Vector2.zero, new Vector2(240f, feedH));
        _joinFeedRt = feed;
        var feedVlg = feed.gameObject.AddComponent<VerticalLayoutGroup>();
        feedVlg.spacing = 3f;
        feedVlg.childAlignment = TextAnchor.LowerLeft;
        feedVlg.childControlWidth = false;
        feedVlg.childControlHeight = false;
        feedVlg.childForceExpandWidth = false;
        feedVlg.childForceExpandHeight = false;
        for (int i = 0; i < MaxFeedRows; i++)
            BuildFeedRow(feed, i);

        var row = CreateRect("StatusRow", col, Vector2.zero, Vector2.zero, Vector2.zero, Vector2.zero, new Vector2(320f, 34f));
        _statusRowRt = row;
        var hlg = row.gameObject.AddComponent<HorizontalLayoutGroup>();
        hlg.spacing = 6f;
        hlg.childAlignment = TextAnchor.MiddleLeft;
        hlg.childControlWidth = false;
        hlg.childControlHeight = false;
        hlg.childForceExpandWidth = false;
        hlg.childForceExpandHeight = false;

        BuildMicPill(row);

        _flightChip = CreateIconChip("Flight", row, HudIcon.Flight, AccentSky, 24f, 13f, out _flightIcon);
        _gestureChip = CreateIconChip("GestureLock", row, HudIcon.Lock, AccentAmber, 24f, 13f);
        _skeletalChip = CreateIconChip("SkeletalLock", row, HudIcon.Lock, AccentPurple, 24f, 13f);
        _moveChip = CreateIconChip("MoveLock", row, HudIcon.Lock, AccentRed, 24f, 13f);
        _commsChip = CreateIconChip("CommsWarn", row, HudIcon.Comms, AccentRed, 24f, 13f);
        _netChip = CreateIconChip("NetHealth", row, HudIcon.Net, AccentAmber, 24f, 13f, out _netIcon);
    }

    private void BuildMicPill(RectTransform parent)
    {
        var pill = CreateRect("MicPill", parent, Vector2.zero, Vector2.zero, Vector2.zero, Vector2.zero, new Vector2(34f, 34f));
        _micCard = pill.gameObject;
        var hlg = pill.gameObject.AddComponent<HorizontalLayoutGroup>();
        hlg.spacing = 6f;
        hlg.childAlignment = TextAnchor.MiddleLeft;
        hlg.childControlWidth = true;
        hlg.childControlHeight = true;
        hlg.childForceExpandWidth = false;
        hlg.childForceExpandHeight = false;
        var fit = pill.gameObject.AddComponent<ContentSizeFitter>();
        fit.horizontalFit = ContentSizeFitter.FitMode.PreferredSize;
        fit.verticalFit = ContentSizeFitter.FitMode.Unconstrained;

        var tile = CreateCard("MicTile", pill, BgChip, 34f, 34f);
        AddLayout(tile.gameObject, 34f, 34f);
        _micTile = tile.GetComponent<Image>();
        _micTileBorder = tile.Find("Border").GetComponent<Image>();

        _micIcon = CreateIcon("MicIcon", tile, HudIcon.MicOff, 20f, AccentRed);
        _micFillIcon = CreateIcon("MicFillIcon", tile, HudIcon.MicOn, 20f, AccentGreen);
        _micFillIcon.type = Image.Type.Filled;
        _micFillIcon.fillMethod = Image.FillMethod.Vertical;
        _micFillIcon.fillOrigin = (int)Image.OriginVertical.Bottom;
        _micFillIcon.fillAmount = 0f;
        _micFillIcon.enabled = false;

        var tts = CreateCard("TtsChip", pill, BgChip, 26f, 18f);
        AddLayout(tts.gameObject, 26f, 18f);
        _ttsText = CreateText("TtsText", tts, 8.5f, AccentSky, FontStyles.Bold, TextAlignmentOptions.Center);
        var ttsRt = _ttsText.rectTransform;
        ttsRt.anchorMin = Vector2.zero;
        ttsRt.anchorMax = Vector2.one;
        ttsRt.offsetMin = Vector2.zero;
        ttsRt.offsetMax = Vector2.zero;
        _ttsText.text = "TTS";
        _ttsChip = tts.gameObject;
        _ttsChip.SetActive(false);
    }

    private void BuildFeedRow(RectTransform parent, int index)
    {
        var row = CreateCard($"FeedRow_{index}", parent, BgGlass, 120f, 20f);
        var cg = row.gameObject.AddComponent<CanvasGroup>();
        var hlg = row.gameObject.AddComponent<HorizontalLayoutGroup>();
        hlg.padding = new RectOffset(7, 10, 3, 3);
        hlg.spacing = 5f;
        hlg.childAlignment = TextAnchor.MiddleLeft;
        hlg.childControlWidth = true;
        hlg.childControlHeight = true;
        hlg.childForceExpandWidth = false;
        hlg.childForceExpandHeight = false;
        var fit = row.gameObject.AddComponent<ContentSizeFitter>();
        fit.horizontalFit = ContentSizeFitter.FitMode.PreferredSize;
        fit.verticalFit = ContentSizeFitter.FitMode.Unconstrained;

        var icon = CreateIcon("Icon", row, HudIcon.Join, 10f, AccentGreen);
        var text = CreateText("Name", row, 11f, TextPrimary, FontStyles.Normal, TextAlignmentOptions.Left);
        row.gameObject.SetActive(false);
        _feedRows.Add(new FeedRowRefs { Root = row.gameObject, Group = cg, Icon = icon, NameText = text });
    }

    private void BuildPropCard(RectTransform parent)
    {
        var cardRt = CreateCard("PropCard", parent, BgGlass, 220f, 34f);
        cardRt.anchorMin = new Vector2(0.5f, 0f);
        cardRt.anchorMax = new Vector2(0.5f, 0f);
        cardRt.pivot = new Vector2(0.5f, 0f);
        cardRt.anchoredPosition = new Vector2(0f, 84f);

        var hlg = cardRt.gameObject.AddComponent<HorizontalLayoutGroup>();
        hlg.padding = new RectOffset(10, 12, 6, 6);
        hlg.spacing = 7f;
        hlg.childAlignment = TextAnchor.MiddleCenter;
        hlg.childControlWidth = true;
        hlg.childControlHeight = true;
        hlg.childForceExpandWidth = false;

        _accentImages.Add(CreateIcon("PropIcon", cardRt, HudIcon.Prop, 14f, AccentSky));
        _propName = CreateText("PropName", cardRt, 12f, TextPrimary, FontStyles.Bold, TextAlignmentOptions.Left);

        _propCard = cardRt.gameObject;
        _propCard.SetActive(false);
    }

    private void BuildBottomCenterCard(RectTransform parent)
    {
        var cardRt = CreateCard("ChatPortalCard", parent, BgGlass, 460f, 44f);
        cardRt.anchorMin = new Vector2(0.5f, 0f);
        cardRt.anchorMax = new Vector2(0.5f, 0f);
        cardRt.pivot = new Vector2(0.5f, 0f);
        cardRt.anchoredPosition = new Vector2(0f, 18f);

        _bottomCenterGroup = cardRt.gameObject.AddComponent<CanvasGroup>();

        var vlg = cardRt.gameObject.AddComponent<VerticalLayoutGroup>();
        vlg.padding = new RectOffset(12, 12, 7, 7);
        vlg.childControlWidth = true;
        vlg.childControlHeight = true;
        vlg.childForceExpandWidth = false;
        vlg.childForceExpandHeight = false;

        var csf = cardRt.gameObject.AddComponent<ContentSizeFitter>();
        csf.horizontalFit = ContentSizeFitter.FitMode.PreferredSize;
        csf.verticalFit = ContentSizeFitter.FitMode.PreferredSize;

        _bottomCenterText = CreateText("ChatPortalText", cardRt, 13f, TextPrimary, FontStyles.Normal, TextAlignmentOptions.Center);
        _bottomCenterText.textWrappingMode = TextWrappingModes.Normal;
        _bottomCenterText.richText = true;
        _bottomCenterTextLayout = _bottomCenterText.gameObject.AddComponent<LayoutElement>();

        _bottomCenterCard = cardRt.gameObject;
        _bottomCenterCard.SetActive(false);
    }

    // --- Per-Frame Updates ---

    private void UpdateCanvasTransformAndVisibility(bool previewAll)
    {
        if (_rootGo == null || _rootGroup == null)
            return;

        Camera uiCam = PlayerSetup.Instance?.activeUiCam;
        if (uiCam == null)
            return;

        Transform hudParent = uiCam.transform;
        if (_rootGo.transform.parent != hudParent)
        {
            _rootGo.transform.SetParent(hudParent, false);
            _canvas.worldCamera = uiCam;
        }

        // Metres in world space; divide by the parent's scale so avatar scaling never resizes the HUD.
        float parentScale = Mathf.Max(0.0001f, hudParent.lossyScale.x);
        bool isVr = MetaPort.Instance != null && MetaPort.Instance.isUsingVr;
        float dist = isVr ? 1.25f : 1.30f;
        float fovRad = uiCam.fieldOfView * 0.5f * Mathf.Deg2Rad;
        float frustumH = 2f * dist * Mathf.Tan(fovRad);
        float baseFill = isVr ? 0.54f : 0.90f;
        float hudSpreadX = isVr ? Settings.HudSpreadVr : Settings.HudSpreadDesktop;
        float hudSpreadY = isVr ? Settings.HudSpreadYVr : Settings.HudSpreadYDesktop;
        float scale = frustumH * baseFill / ReferenceHeight / parentScale;

        if (_rootRt != null)
        {
            float targetW = ReferenceWidth * hudSpreadX;
            float targetH = ReferenceHeight * hudSpreadY;
            if (Mathf.Abs(_rootRt.sizeDelta.x - targetW) > 0.5f || Mathf.Abs(_rootRt.sizeDelta.y - targetH) > 0.5f)
            {
                _rootRt.sizeDelta = new Vector2(targetW, targetH);
            }
        }

        float offsetY = isVr ? -0.08f * frustumH : 0f;
        _rootGo.transform.localPosition = new Vector3(0f, offsetY, dist) / parentScale;
        _rootGo.transform.localRotation = Quaternion.identity;
        _rootGo.transform.localScale = new Vector3(scale, scale, scale);

        float alertsScale = isVr ? Settings.HudAlertsScaleVr : Settings.HudAlertsScaleDesktop;
        float micScale = isVr ? Settings.HudMicScaleVr : Settings.HudMicScaleDesktop;
        bool statusLayoutDirty = false;
        if (Mathf.Abs(_lastAppliedAlertsScale - alertsScale) > 0.002f)
        {
            _lastAppliedAlertsScale = alertsScale;
            ApplySizedChip(_badgeFriends, 24f, 13f, alertsScale);
            ApplySizedChip(_badgeInvites, 24f, 13f, alertsScale);
            ApplySizedChip(_badgeVotes, 24f, 13f, alertsScale);
            ApplySizedChip(_badgeAlerts, 24f, 13f, alertsScale);
            ApplySizedChip(_badgeInfo, 24f, 13f, alertsScale);
            if (_badgesRow != null)
            {
                var brt = _badgesRow.GetComponent<RectTransform>();
                if (brt != null)
                {
                    brt.sizeDelta = new Vector2(Mathf.Max(200f, 5f * (24f * alertsScale + 6f)), Mathf.Max(28f, 24f * alertsScale + 4f));
                    LayoutRebuilder.MarkLayoutForRebuild(brt);
                }
            }
            ApplySizedChip(_flightChip, 24f, 13f, alertsScale);
            ApplySizedChip(_gestureChip, 24f, 13f, alertsScale);
            ApplySizedChip(_skeletalChip, 24f, 13f, alertsScale);
            ApplySizedChip(_moveChip, 24f, 13f, alertsScale);
            ApplySizedChip(_commsChip, 24f, 13f, alertsScale);
            ApplySizedChip(_netChip, 24f, 13f, alertsScale);
            statusLayoutDirty = true;
        }
        if (Mathf.Abs(_lastAppliedMicScale - micScale) > 0.002f)
        {
            _lastAppliedMicScale = micScale;
            ApplyMicScale(micScale);
            statusLayoutDirty = true;
        }
        if (statusLayoutDirty && _statusRowRt != null)
        {
            float rowH = Mathf.Max(34f * micScale, 24f * alertsScale);
            float rowW = Mathf.Max(320f, 34f * micScale + 6f * (24f * alertsScale + 6f) + 40f);
            _statusRowRt.sizeDelta = new Vector2(rowW, rowH);
            if (_statusClusterRt != null)
            {
                float feedH = MaxFeedRows * 20f + (MaxFeedRows - 1) * 3f;
                _statusClusterRt.sizeDelta = new Vector2(rowW, feedH + 6f + rowH);
                LayoutRebuilder.MarkLayoutForRebuild(_statusClusterRt);
            }
            LayoutRebuilder.MarkLayoutForRebuild(_statusRowRt);
        }

        float notifScale = isVr ? Settings.HudNotificationsScaleVr : Settings.HudNotificationsScaleDesktop;
        ApplyElementScale(_toastStackRt, notifScale);
        ApplyElementScale(_joinFeedRt, notifScale);
        ApplyElementScale(_propCard != null ? _propCard.transform : null, notifScale);
        ApplyElementScale(_bottomCenterCard != null ? _bottomCenterCard.transform : null, isVr ? Settings.HudChatboxScaleVr : Settings.HudChatboxScaleDesktop);

        if (!_loggedState && Time.unscaledTime - _builtTime > 2f)
        {
            _loggedState = true;
            Vector3 rel = uiCam.transform.InverseTransformPoint(_rootGo.transform.position);
            MelonLogger.Msg(
                $"[LumenicMenu][HUD] parent={hudParent.name} layer={_rootGo.layer} camMask={uiCam.cullingMask} " +
                $"camOn={uiCam.enabled && uiCam.gameObject.activeInHierarchy} rel={rel} lossy={_rootGo.transform.lossyScale.x:F5} " +
                $"active={_rootGo.activeInHierarchy} vr={isVr} showHud={MetaPort.Instance?.settings?.GetSettingsBool("GeneralShowHud")} " +
                $"behaviour={CohtmlHud.GetCurrentDisplayBehaviour()} mic={_micVisible} opacity={_hudOpacity:F2}");
        }

        if (previewAll)
        {
            _rootGroup.alpha = Mathf.Max(_hudOpacity, 0.75f);
            return;
        }

        bool showGeneral = MetaPort.Instance?.settings == null || MetaPort.Instance.settings.GetSettingsBool("GeneralShowHud");
        var behaviour = CohtmlHud.GetCurrentDisplayBehaviour();
        _showForMicUpdates = behaviour == CohtmlHud.DisplayBehaviourType.Always
            || behaviour == CohtmlHud.DisplayBehaviourType.VoiceActivityAndUpdates;
        if (!showGeneral || behaviour == CohtmlHud.DisplayBehaviourType.Never)
        {
            _rootGroup.alpha = 0f;
            return;
        }

        if (behaviour == CohtmlHud.DisplayBehaviourType.Always)
        {
            _rootGroup.alpha = _hudOpacity;
            return;
        }

        float idleFor = Time.unscaledTime - _lastActivityTime;
        bool hasActiveToastOrCard = _toasts.Count > 0 || _joinFeed.Count > 0 || _chatOrPortalActive || (_propCard != null && _propCard.activeSelf);
        float targetAlpha = (hasActiveToastOrCard || idleFor < 4.0f) ? _hudOpacity : 0f;
        _rootGroup.alpha = Mathf.MoveTowards(_rootGroup.alpha, targetAlpha, Time.unscaledDeltaTime * 3.5f);
    }

    private static void ApplyElementScale(Transform target, float scale)
    {
        if (target == null)
            return;
        if (Mathf.Abs(target.localScale.x - scale) > 0.002f)
            target.localScale = new Vector3(scale, scale, 1f);
    }

    private static void ApplySizedChip(GameObject chip, float baseBox, float baseIcon, float scale)
    {
        if (chip == null)
            return;
        float box = baseBox * scale;
        var rt = chip.GetComponent<RectTransform>();
        if (rt != null)
            rt.sizeDelta = new Vector2(box, box);
        var glyph = chip.transform.Find("Glyph") as RectTransform;
        if (glyph != null)
        {
            float ic = baseIcon * scale;
            glyph.sizeDelta = new Vector2(ic, ic);
            var le = glyph.GetComponent<LayoutElement>();
            if (le != null)
            {
                le.preferredWidth = ic;
                le.preferredHeight = ic;
            }
        }
    }

    private void ApplyMicScale(float scale)
    {
        float tileSz = 34f * scale;
        float iconSz = 20f * scale;
        if (_micTile != null)
        {
            _micTile.rectTransform.sizeDelta = new Vector2(tileSz, tileSz);
            AddLayout(_micTile.gameObject, tileSz, tileSz);
        }
        if (_micIcon != null)
            _micIcon.rectTransform.sizeDelta = new Vector2(iconSz, iconSz);
        if (_micFillIcon != null)
            _micFillIcon.rectTransform.sizeDelta = new Vector2(iconSz, iconSz);
        if (_ttsChip != null)
        {
            var ttsRt = _ttsChip.GetComponent<RectTransform>();
            if (ttsRt != null)
                ttsRt.sizeDelta = new Vector2(26f * scale, 18f * scale);
            AddLayout(_ttsChip, 26f * scale, 18f * scale);
        }
        if (_micCard != null)
        {
            var pillRt = _micCard.GetComponent<RectTransform>();
            if (pillRt != null)
            {
                pillRt.sizeDelta = new Vector2(pillRt.sizeDelta.x, tileSz);
                LayoutRebuilder.MarkLayoutForRebuild(pillRt);
            }
        }
    }

    private void UpdateToastLifecycle(bool previewAll)
    {
        float now = Time.unscaledTime;
        for (int i = _toasts.Count - 1; i >= 0; i--)
        {
            if (now >= _toasts[i].ExpireTime)
            {
                _toasts.RemoveAt(i);
                _toastsDirty = true;
            }
        }

        if (previewAll && _toasts.Count == 0 && _toastCards.Count > 0)
        {
            ToastCardRefs card = _toastCards[0];
            card.Root.SetActive(true);
            card.CategoryText.text = "Preview";
            card.HeadlineText.text = "Notification Toast";
            card.BodyText.text = "Adjusting HUD layout & scale";
            card.BodyText.gameObject.SetActive(true);
            card.Icon.sprite = _icons[HudIcon.Info];
            card.Icon.color = AccentSky;
            card.Group.alpha = 1f;
            for (int i = 1; i < _toastCards.Count; i++)
            {
                if (_toastCards[i].Root.activeSelf)
                    _toastCards[i].Root.SetActive(false);
            }
            _toastsDirty = true;
            return;
        }

        for (int i = 0; i < _toastCards.Count; i++)
        {
            ToastCardRefs card = _toastCards[i];
            if (i < _toasts.Count)
            {
                ToastEntry entry = _toasts[i];
                if (!card.Root.activeSelf || _toastsDirty)
                {
                    card.Root.SetActive(true);
                    card.CategoryText.text = entry.Category;
                    card.HeadlineText.text = entry.Headline;
                    card.BodyText.text = entry.Body;
                    card.BodyText.gameObject.SetActive(!string.IsNullOrEmpty(entry.Body));

                    bool isServer = entry.Category.IndexOf("Server", StringComparison.OrdinalIgnoreCase) >= 0;
                    card.Icon.sprite = _icons[isServer ? HudIcon.Alert : HudIcon.Info];
                    card.Icon.color = isServer ? AccentAmber : AccentSky;
                }

                float age = now - entry.CreatedTime;
                float remaining = entry.ExpireTime - now;
                float fadeIn = Mathf.Clamp01(age / 0.16f);
                float fadeOut = Mathf.Clamp01(remaining / 0.30f);
                card.Group.alpha = Mathf.Min(fadeIn, fadeOut);
            }
            else if (card.Root.activeSelf)
            {
                card.Root.SetActive(false);
            }
        }

        _toastsDirty = false;
    }

    private void UpdateMicAndStatusWidgets(bool previewAll)
    {
        bool liveMicActive = !Comms_Manager.IsMicMuted;
        if (_micActive != liveMicActive)
        {
            _micActive = liveMicActive;
            TouchActivity();
        }

        if (_micCard != null)
        {
            bool showMic = _micVisible || previewAll;
            if (_micCard.activeSelf != showMic)
                _micCard.SetActive(showMic);
            if (showMic)
                UpdateMicPill();
        }

        if (_commsChip != null && _commsChip.activeSelf != _commsWarning)
            _commsChip.SetActive(_commsWarning);

        if (_netChip != null)
        {
            bool showNet = _netChainState > 0 || previewAll;
            if (_netChip.activeSelf != showNet)
                _netChip.SetActive(showNet);
            if (showNet && _netIcon != null)
                _netIcon.color = _netChainState >= 2 ? AccentRed : AccentAmber;
        }

        if (_flightChip != null)
        {
            bool showFlight = _isFlying || previewAll;
            if (_flightChip.activeSelf != showFlight)
                _flightChip.SetActive(showFlight);
            if (showFlight && _flightIcon != null)
                _flightIcon.color = _isNoClip ? AccentAmber : AccentSky;
        }

        bool showGesture = _gestureLocked || previewAll;
        if (_gestureChip != null && _gestureChip.activeSelf != showGesture)
            _gestureChip.SetActive(showGesture);
        if (_skeletalChip != null && _skeletalChip.activeSelf != _skeletalLocked)
            _skeletalChip.SetActive(_skeletalLocked);
        if (_moveChip != null && _moveChip.activeSelf != _movementLocked)
            _moveChip.SetActive(_movementLocked);
    }

    private void UpdateMicPill()
    {
        float now = Time.unscaledTime;
        bool mutedPulse = !_micActive && _mutedSpeechEnabled && now < _mutedSpeechPulseUntil;
        Color accent = _ttsActive
            ? AccentBlue
            : (_micActive
                ? AccentGreen
                : (mutedPulse ? Color.Lerp(AccentRed, AccentAmber, 0.5f + 0.5f * Mathf.Sin(now * 16f)) : AccentRed));

        _micTile.color = WithAlpha(BgGlass, 0.90f);
        _micTileBorder.color = WithAlpha(accent, (_micActive && !_ttsActive) ? 0.50f : 0.70f);

        if (_micActive)
        {
            float target = Mathf.Pow(_micLevel, 0.6f);
            if (!_smoothMicMeter)
                _micDisplay = target;
            else
                _micDisplay = target > _micDisplay
                    ? Mathf.Lerp(_micDisplay, target, 0.6f)
                    : Mathf.MoveTowards(_micDisplay, target, Time.unscaledDeltaTime * 2.5f);

            _micIcon.sprite = _icons[HudIcon.MicOn];
            _micIcon.color = WithAlpha(_ttsActive ? AccentBlue : TextPrimary, _ttsActive ? 0.40f : 0.28f);

            _micFillIcon.enabled = _micDisplay > 0.01f;
            _micFillIcon.fillAmount = _micDisplay;
            _micFillIcon.color = _ttsActive ? AccentBlue : (_micDisplay > 0.85f ? AccentAmber : AccentGreen);
        }
        else
        {
            _micDisplay = 0f;
            _micIcon.sprite = _icons[_ttsActive ? HudIcon.MicOn : HudIcon.MicOff];
            _micIcon.color = accent;
            _micFillIcon.enabled = false;
            _micFillIcon.fillAmount = 0f;
        }

        if (_ttsChip.activeSelf != _ttsActive)
            _ttsChip.SetActive(_ttsActive);
    }

    private void UpdateJoinFeed(bool previewAll)
    {
        float now = Time.unscaledTime;
        for (int i = _joinFeed.Count - 1; i >= 0; i--)
        {
            if (now >= _joinFeed[i].ExpireTime)
            {
                _joinFeed.RemoveAt(i);
                _feedDirty = true;
            }
        }

        if (previewAll && _joinFeed.Count == 0 && _feedRows.Count > 0)
        {
            FeedRowRefs row = _feedRows[0];
            row.Root.SetActive(true);
            row.NameText.text = "PlayerJoined";
            row.Icon.sprite = _icons[HudIcon.Join];
            row.Icon.color = AccentGreen;
            row.NameText.color = TextPrimary;
            row.Group.alpha = 1f;
            for (int i = 1; i < _feedRows.Count; i++)
            {
                if (_feedRows[i].Root.activeSelf)
                    _feedRows[i].Root.SetActive(false);
            }
            _feedDirty = true;
            return;
        }

        int start = Mathf.Max(0, _joinFeed.Count - _feedRows.Count);
        for (int i = 0; i < _feedRows.Count; i++)
        {
            FeedRowRefs row = _feedRows[i];
            int idx = start + i;
            if (idx >= _joinFeed.Count)
            {
                if (row.Root.activeSelf)
                    row.Root.SetActive(false);
                continue;
            }

            FeedEntry entry = _joinFeed[idx];
            if (!row.Root.activeSelf || _feedDirty)
            {
                row.Root.SetActive(true);
                row.NameText.text = entry.Name;
                row.Icon.sprite = _icons[entry.Joined ? HudIcon.Join : HudIcon.Leave];
                row.Icon.color = entry.Joined ? AccentGreen : AccentRed;
                row.NameText.color = entry.Joined ? TextPrimary : TextMuted;
            }

            float age = now - entry.CreatedTime;
            float remaining = entry.ExpireTime - now;
            row.Group.alpha = Mathf.Min(Mathf.Clamp01(age / 0.16f), Mathf.Clamp01(remaining / 0.4f));
        }

        _feedDirty = false;
    }

    private void UpdateNotificationBadges(bool previewAll)
    {
        bool f = _hasFriends || _testFriends || previewAll;
        bool inv = _hasInvites || _testInvites || previewAll;
        bool v = _hasVotes || _testVotes || previewAll;
        bool a = _hasAlerts || _testAlerts || previewAll;
        bool info = _hasInfo || _testInfo || previewAll;

        if (_badgeFriends != null && _badgeFriends.activeSelf != f) _badgeFriends.SetActive(f);
        if (_badgeInvites != null && _badgeInvites.activeSelf != inv) _badgeInvites.SetActive(inv);
        if (_badgeVotes != null && _badgeVotes.activeSelf != v) _badgeVotes.SetActive(v);
        if (_badgeAlerts != null && _badgeAlerts.activeSelf != a) _badgeAlerts.SetActive(a);
        if (_badgeInfo != null && _badgeInfo.activeSelf != info) _badgeInfo.SetActive(info);

        if (_badgesRow != null)
        {
            bool any = f || inv || v || a || info;
            if (_badgesRow.activeSelf != any)
                _badgesRow.SetActive(any);
        }
    }

    private void UpdateBottomCenterCard(bool previewAll)
    {
        if (_bottomCenterCard == null)
            return;

        bool hasLive = _chatOrPortalActive && !string.IsNullOrEmpty(_chatOrPortalText);
        bool shouldShow = hasLive || previewAll;
        if (_bottomCenterCard.activeSelf != shouldShow)
            _bottomCenterCard.SetActive(shouldShow);

        if (!shouldShow)
            return;

        string displayText = hasLive ? _chatOrPortalText : "ChatBox / Portal Preview";
        if (!string.Equals(_bottomCenterText.text, displayText, StringComparison.Ordinal))
        {
            _bottomCenterText.text = displayText;
            if (_bottomCenterTextLayout != null)
            {
                float natW = _bottomCenterText.GetPreferredValues(displayText).x;
                _bottomCenterTextLayout.preferredWidth = Mathf.Clamp(natW, 40f, 400f);
            }
            LayoutRebuilder.ForceRebuildLayoutImmediate(_bottomCenterCard.GetComponent<RectTransform>());
        }

        if (_isTestPortalActive)
        {
            _portalPulsePhase += Time.unscaledDeltaTime;
            _bottomCenterGroup.alpha = Mathf.Lerp(0.72f, 1.0f, Mathf.PingPong(_portalPulsePhase / 1.1f, 1f));
        }
        else
        {
            _bottomCenterGroup.alpha = 1f;
        }
    }

    // --- UGUI Builders ---

    private RectTransform CreateRect(
        string name,
        RectTransform parent,
        Vector2 anchorMin,
        Vector2 anchorMax,
        Vector2 pivot,
        Vector2 anchoredPos,
        Vector2 sizeDelta)
    {
        var go = new GameObject(name);
        go.layer = parent.gameObject.layer;
        var rt = go.AddComponent<RectTransform>();
        rt.SetParent(parent, false);
        rt.anchorMin = anchorMin;
        rt.anchorMax = anchorMax;
        rt.pivot = pivot;
        rt.anchoredPosition = anchoredPos;
        rt.sizeDelta = sizeDelta;
        return rt;
    }

    private RectTransform CreateCard(string name, RectTransform parent, Color color, float width, float height)
    {
        var rt = CreateRect(name, parent, new Vector2(0.5f, 0.5f), new Vector2(0.5f, 0.5f), new Vector2(0.5f, 0.5f), Vector2.zero, new Vector2(width, height));
        var img = rt.gameObject.AddComponent<Image>();
        img.sprite = _roundedSprite;
        img.type = Image.Type.Sliced;
        img.color = color;

        var border = CreateRect("Border", rt, Vector2.zero, Vector2.one, new Vector2(0.5f, 0.5f), Vector2.zero, Vector2.zero);
        border.gameObject.AddComponent<LayoutElement>().ignoreLayout = true;
        var borderImg = border.gameObject.AddComponent<Image>();
        borderImg.sprite = _ringSprite;
        borderImg.type = Image.Type.Sliced;
        borderImg.color = BorderSubtle;
        borderImg.raycastTarget = false;
        _themedCards.Add((img, borderImg, color == BgChip));
        return rt;
    }

    private Image CreateIcon(string name, RectTransform parent, HudIcon icon, float size, Color color)
    {
        var rt = CreateRect(name, parent, new Vector2(0.5f, 0.5f), new Vector2(0.5f, 0.5f), new Vector2(0.5f, 0.5f), Vector2.zero, new Vector2(size, size));
        var le = rt.gameObject.AddComponent<LayoutElement>();
        le.preferredWidth = size;
        le.preferredHeight = size;
        var img = rt.gameObject.AddComponent<Image>();
        img.sprite = _icons[icon];
        img.color = color;
        img.raycastTarget = false;
        return img;
    }

    private GameObject CreateIconChip(string name, RectTransform parent, HudIcon icon, Color color, float boxSize, float iconSize)
    {
        return CreateIconChip(name, parent, icon, color, boxSize, iconSize, out _);
    }

    private GameObject CreateIconChip(string name, RectTransform parent, HudIcon icon, Color color, float boxSize, float iconSize, out Image iconOut)
    {
        var rt = CreateCard(name, parent, BgChip, boxSize, boxSize);
        iconOut = CreateIcon("Glyph", rt, icon, iconSize, color);
        rt.gameObject.SetActive(false);
        return rt.gameObject;
    }

    private TMP_Text CreateText(string name, RectTransform parent, float size, Color color, FontStyles style, TextAlignmentOptions align)
    {
        var go = new GameObject(name);
        go.layer = parent.gameObject.layer;
        var rt = go.AddComponent<RectTransform>();
        rt.SetParent(parent, false);
        var tmp = go.AddComponent<TextMeshProUGUI>();
        ConfigureTmp(tmp, size, color, style, align);
        return tmp;
    }

    private void ConfigureTmp(TMP_Text tmp, float size, Color color, FontStyles style, TextAlignmentOptions align)
    {
        if (_font != null)
            tmp.font = _font;
        tmp.fontSize = size;
        tmp.color = color;
        tmp.fontStyle = style;
        tmp.alignment = align;
        tmp.raycastTarget = false;
    }

    private static TMP_FontAsset ResolveFontAsset()
    {
        if (TMP_Settings.defaultFontAsset != null)
            return TMP_Settings.defaultFontAsset;

        var existing = UnityEngine.Object.FindAnyObjectByType<TMP_Text>();
        if (existing != null && existing.font != null)
            return existing.font;

        return Resources.Load<TMP_FontAsset>("Fonts & Materials/LiberationSans SDF");
    }

    // --- Procedural Anti-Aliased SDF Icon & 9-Slice Sprites ---

    private void BuildIconSprites()
    {
        _icons[HudIcon.MicOn] = RasterizeIcon(32, p =>
        {
            float d = SdfCapsule(p, new Vector2(0.5f, 0.38f), new Vector2(0.5f, 0.68f), 0.14f);
            d = Mathf.Min(d, Mathf.Abs(SdfArcBottom(p, new Vector2(0.5f, 0.50f), 0.25f)) - 0.045f);
            d = Mathf.Min(d, SdfSegment(p, new Vector2(0.5f, 0.14f), new Vector2(0.5f, 0.25f)) - 0.045f);
            d = Mathf.Min(d, SdfSegment(p, new Vector2(0.36f, 0.14f), new Vector2(0.64f, 0.14f)) - 0.045f);
            return d;
        });

        _icons[HudIcon.MicOff] = RasterizeIcon(32, p =>
        {
            float d = SdfCapsule(p, new Vector2(0.5f, 0.38f), new Vector2(0.5f, 0.68f), 0.14f);
            d = Mathf.Min(d, Mathf.Abs(SdfArcBottom(p, new Vector2(0.5f, 0.50f), 0.25f)) - 0.045f);
            d = Mathf.Min(d, SdfSegment(p, new Vector2(0.5f, 0.14f), new Vector2(0.5f, 0.25f)) - 0.045f);
            float slash = SdfSegment(p, new Vector2(0.18f, 0.18f), new Vector2(0.82f, 0.82f)) - 0.055f;
            return Mathf.Min(d, slash);
        });

        _icons[HudIcon.Friends] = RasterizeIcon(32, p =>
        {
            float head = SdfCircle(p, new Vector2(0.5f, 0.67f), 0.15f);
            float shoulders = SdfCapsule(p, new Vector2(0.30f, 0.24f), new Vector2(0.70f, 0.24f), 0.13f);
            return Mathf.Min(head, shoulders);
        });

        _icons[HudIcon.Invite] = RasterizeIcon(32, p =>
        {
            float box = Mathf.Abs(SdfBox(p - new Vector2(0.5f, 0.5f), new Vector2(0.34f, 0.24f))) - 0.045f;
            float flap = Mathf.Min(
                SdfSegment(p, new Vector2(0.18f, 0.70f), new Vector2(0.5f, 0.45f)),
                SdfSegment(p, new Vector2(0.82f, 0.70f), new Vector2(0.5f, 0.45f))) - 0.045f;
            return Mathf.Min(box, flap);
        });

        _icons[HudIcon.Vote] = RasterizeIcon(32, p =>
        {
            float ring = Mathf.Abs(SdfCircle(p, new Vector2(0.5f, 0.5f), 0.34f)) - 0.045f;
            float check = Mathf.Min(
                SdfSegment(p, new Vector2(0.32f, 0.50f), new Vector2(0.45f, 0.36f)),
                SdfSegment(p, new Vector2(0.45f, 0.36f), new Vector2(0.70f, 0.64f))) - 0.055f;
            return Mathf.Min(ring, check);
        });

        _icons[HudIcon.Alert] = RasterizeIcon(32, p =>
        {
            float tri = Mathf.Abs(SdfTriangle(p, new Vector2(0.5f, 0.84f), new Vector2(0.16f, 0.20f), new Vector2(0.84f, 0.20f))) - 0.045f;
            float stem = SdfSegment(p, new Vector2(0.5f, 0.62f), new Vector2(0.5f, 0.42f)) - 0.045f;
            float dot = SdfCircle(p, new Vector2(0.5f, 0.30f), 0.048f);
            return Mathf.Min(tri, Mathf.Min(stem, dot));
        });

        _icons[HudIcon.Info] = RasterizeIcon(32, p =>
        {
            float ring = Mathf.Abs(SdfCircle(p, new Vector2(0.5f, 0.5f), 0.34f)) - 0.045f;
            float dot = SdfCircle(p, new Vector2(0.5f, 0.68f), 0.048f);
            float stem = SdfSegment(p, new Vector2(0.5f, 0.53f), new Vector2(0.5f, 0.30f)) - 0.048f;
            return Mathf.Min(ring, Mathf.Min(dot, stem));
        });

        _icons[HudIcon.Flight] = RasterizeIcon(32, p =>
        {
            float l = SdfSegment(p, new Vector2(0.16f, 0.42f), new Vector2(0.5f, 0.68f)) - 0.06f;
            float r = SdfSegment(p, new Vector2(0.84f, 0.42f), new Vector2(0.5f, 0.68f)) - 0.06f;
            float l2 = SdfSegment(p, new Vector2(0.24f, 0.26f), new Vector2(0.5f, 0.48f)) - 0.05f;
            float r2 = SdfSegment(p, new Vector2(0.76f, 0.26f), new Vector2(0.5f, 0.48f)) - 0.05f;
            return Mathf.Min(Mathf.Min(l, r), Mathf.Min(l2, r2));
        });

        _icons[HudIcon.Lock] = RasterizeIcon(32, p =>
        {
            float body = SdfBox(p - new Vector2(0.5f, 0.36f), new Vector2(0.24f, 0.18f));
            float shackle = Mathf.Abs(SdfArcTop(p, new Vector2(0.5f, 0.54f), 0.15f)) - 0.045f;
            return Mathf.Min(body, shackle);
        });

        _icons[HudIcon.Comms] = RasterizeIcon(32, p =>
        {
            float a1 = Mathf.Abs(SdfCircle(p, new Vector2(0.5f, 0.36f), 0.10f));
            float a2 = Mathf.Abs(SdfArcTop(p, new Vector2(0.5f, 0.36f), 0.24f)) - 0.045f;
            float a3 = Mathf.Abs(SdfArcTop(p, new Vector2(0.5f, 0.36f), 0.38f)) - 0.045f;
            return Mathf.Min(a1, Mathf.Min(a2, a3));
        });

        _icons[HudIcon.Net] = RasterizeIcon(32, p =>
        {
            float b1 = SdfBox(p - new Vector2(0.26f, 0.30f), new Vector2(0.05f, 0.12f));
            float b2 = SdfBox(p - new Vector2(0.50f, 0.42f), new Vector2(0.05f, 0.24f));
            float b3 = SdfBox(p - new Vector2(0.74f, 0.54f), new Vector2(0.05f, 0.36f));
            return Mathf.Min(b1, Mathf.Min(b2, b3));
        });

        _icons[HudIcon.Prop] = RasterizeIcon(32, p =>
        {
            float box = Mathf.Abs(SdfBox(p - new Vector2(0.5f, 0.5f), new Vector2(0.28f, 0.28f))) - 0.045f;
            float cross = SdfSegment(p, new Vector2(0.22f, 0.5f), new Vector2(0.78f, 0.5f)) - 0.04f;
            return Mathf.Min(box, cross);
        });

        _icons[HudIcon.Join] = RasterizeIcon(32, p =>
        {
            float h = SdfSegment(p, new Vector2(0.22f, 0.5f), new Vector2(0.78f, 0.5f)) - 0.07f;
            float v = SdfSegment(p, new Vector2(0.5f, 0.22f), new Vector2(0.5f, 0.78f)) - 0.07f;
            return Mathf.Min(h, v);
        });

        _icons[HudIcon.Leave] = RasterizeIcon(32, p =>
            SdfSegment(p, new Vector2(0.22f, 0.5f), new Vector2(0.78f, 0.5f)) - 0.07f);
    }

    private static Sprite RasterizeIcon(int size, Func<Vector2, float> sdf)
    {
        var tex = new Texture2D(size, size, TextureFormat.RGBA32, false)
        {
            wrapMode = TextureWrapMode.Clamp,
            filterMode = FilterMode.Bilinear
        };

        Color32[] pixels = new Color32[size * size];
        float aa = 1.3f / size;
        for (int y = 0; y < size; y++)
        {
            for (int x = 0; x < size; x++)
            {
                Vector2 p = new Vector2((x + 0.5f) / size, (y + 0.5f) / size);
                float d = sdf(p);
                float alpha = Mathf.Clamp01(0.5f - d / aa);
                pixels[y * size + x] = new Color32(255, 255, 255, (byte)(alpha * 255f));
            }
        }
        tex.SetPixels32(pixels);
        tex.Apply(false, true);
        return Sprite.Create(tex, new Rect(0f, 0f, size, size), new Vector2(0.5f, 0.5f), 100f);
    }

    private static float SdfCircle(Vector2 p, Vector2 c, float r) => Vector2.Distance(p, c) - r;

    private static float SdfBox(Vector2 p, Vector2 b)
    {
        Vector2 d = new Vector2(Mathf.Abs(p.x) - b.x, Mathf.Abs(p.y) - b.y);
        return Mathf.Sqrt(Mathf.Max(d.x, 0f) * Mathf.Max(d.x, 0f) + Mathf.Max(d.y, 0f) * Mathf.Max(d.y, 0f))
            + Mathf.Min(Mathf.Max(d.x, d.y), 0f);
    }

    private static float SdfSegment(Vector2 p, Vector2 a, Vector2 b)
    {
        Vector2 pa = p - a;
        Vector2 ba = b - a;
        float h = Mathf.Clamp01(Vector2.Dot(pa, ba) / Mathf.Max(0.0001f, Vector2.Dot(ba, ba)));
        return (pa - ba * h).magnitude;
    }

    private static float SdfCapsule(Vector2 p, Vector2 a, Vector2 b, float r) => SdfSegment(p, a, b) - r;

    private static float SdfArcBottom(Vector2 p, Vector2 c, float r)
    {
        Vector2 q = p - c;
        if (q.y > 0f) return Vector2.Distance(q, new Vector2(q.x >= 0f ? r : -r, 0f));
        return Mathf.Abs(q.magnitude - r);
    }

    private static float SdfArcTop(Vector2 p, Vector2 c, float r)
    {
        Vector2 q = p - c;
        if (q.y < 0f) return Vector2.Distance(q, new Vector2(q.x >= 0f ? r : -r, 0f));
        return Mathf.Abs(q.magnitude - r);
    }

    private static float SdfTriangle(Vector2 p, Vector2 a, Vector2 b, Vector2 c)
    {
        return Mathf.Min(SdfSegment(p, a, b), Mathf.Min(SdfSegment(p, b, c), SdfSegment(p, c, a)));
    }

    private static Color WithAlpha(Color c, float a) => new Color(c.r, c.g, c.b, a);

    // Inward distance from the chamfered-rect edge, in pixels (positive inside).
    private static float ChamferDist(float px, float py, float s, float c)
    {
        const float k = 0.70710678f;
        float d = Mathf.Min(Mathf.Min(px, s - px), Mathf.Min(py, s - py));
        d = Mathf.Min(d, (px + py - c) * k);
        d = Mathf.Min(d, (s - px + py - c) * k);
        d = Mathf.Min(d, (px + s - py - c) * k);
        d = Mathf.Min(d, (s - px + s - py - c) * k);
        return d;
    }

    private static Sprite CreateChamferSprite(int size, float chamfer, bool ring)
    {
        var tex = new Texture2D(size, size, TextureFormat.RGBA32, false)
        {
            name = ring ? "LumenicHud_ChamferRing" : "LumenicHud_Chamfer",
            wrapMode = TextureWrapMode.Clamp,
            filterMode = FilterMode.Bilinear
        };

        Color32[] pixels = new Color32[size * size];
        for (int y = 0; y < size; y++)
        {
            for (int x = 0; x < size; x++)
            {
                float d = ChamferDist(x + 0.5f, y + 0.5f, size, chamfer);
                float a = Mathf.Clamp01(d + 0.5f);
                if (ring)
                    a = Mathf.Clamp01(a - Mathf.Clamp01(d - 0.5f));
                pixels[y * size + x] = new Color32(255, 255, 255, (byte)(a * 255f));
            }
        }

        tex.SetPixels32(pixels);
        tex.Apply(false, true);

        float b = chamfer + 1f;
        return Sprite.Create(
            tex,
            new Rect(0f, 0f, size, size),
            new Vector2(0.5f, 0.5f),
            100f,
            0,
            SpriteMeshType.FullRect,
            new Vector4(b, b, b, b));
    }

    public void Dispose()
    {
        RestoreVanillaRenderers();
        if (_rootGo != null)
        {
            UnityEngine.Object.Destroy(_rootGo);
            _rootGo = null;
        }
        if (_roundedSprite != null)
        {
            if (_roundedSprite.texture != null)
                UnityEngine.Object.Destroy(_roundedSprite.texture);
            UnityEngine.Object.Destroy(_roundedSprite);
            _roundedSprite = null;
        }
        if (_ringSprite != null)
        {
            if (_ringSprite.texture != null)
                UnityEngine.Object.Destroy(_ringSprite.texture);
            UnityEngine.Object.Destroy(_ringSprite);
            _ringSprite = null;
        }
        foreach (var kv in _icons)
        {
            if (kv.Value != null)
            {
                if (kv.Value.texture != null)
                    UnityEngine.Object.Destroy(kv.Value.texture);
                UnityEngine.Object.Destroy(kv.Value);
            }
        }
        _icons.Clear();
        _built = false;
    }
}
