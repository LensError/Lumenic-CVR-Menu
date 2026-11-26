using System;
using System.Threading;
using ABI_RC.Core;
using ABI_RC.Core.InteractionSystem;
using ABI_RC.Core.Player;
using ABI_RC.Core.Savior;
using ABI_RC.Core.UI;
using ABI_RC.Core.UI.UIRework;
using ABI_RC.Core.UI.UIRework.Managers;
using ABI_RC.Systems.InputManagement;
using LumenicMenu.Bridge;
using LumenicMenu.Native;
using MelonLoader;
using UnityEngine;

[assembly: MelonInfo(typeof(LumenicMenu.LumenicMenuMod), "LumenicMenu", LumenicMenu.LumenicMenuMod.ModVersion, "LensError")]
[assembly: MelonGame(null, "ChilloutVR")]

namespace LumenicMenu;

public sealed class LumenicMenuMod : MelonMod
{
    public const string ModVersion = "0.3.6";
    private const string InputLockReason = "LumenicMenu";

    public static LumenicMenuMod Instance { get; private set; }

    private NativeBridge _bridge;
    private MenuSurfaceHost _mainSurface;
    private MenuSurfaceHost _quickSurface;
    private ImageLoaderService _images;
    private HostStateBuilder _stateBuilder;
    private PreviewRigCoordinator _previewRig;
    private LiveEventWiring _liveEvents;

    internal UiLibBridge UiLib { get; private set; }
    internal UiExpansionKitBridge Uix { get; private set; }
    internal RecentHistoryService RecentHistory { get; private set; }
    internal PinnedContentService PinnedContent { get; private set; }
    internal ContentBrowserService Content { get; private set; }
    internal SocialService Social { get; private set; }
    internal AvatarStageManager AvatarStage { get; private set; }
    internal ModUpdateService Updates { get; private set; }
    internal SystemAlertService SystemAlerts { get; private set; }
    internal LumenicHudManager Hud { get; private set; }

    private bool _keyboardLocked;
    private bool _prevMainEnabled;
    private bool _prevQuickEnabled;
    private bool _prevMainShown;
    private bool _prevQuickShown;
    private bool _bridgeInitAttempted;
    private bool _notifiedApiReady;
    private float _lastTelemetryPollTime = -10f;
    private float _lastDynamicUiPollTime = -10f;
    private float _lastSocialLivePollTime = -10f;
    private int _lastTelFps = int.MinValue;
    private int _lastTelGamePing = int.MinValue;
    private int _lastTelVoicePing = int.MinValue;
    private string _lastTelClock = string.Empty;
    private string _lastTelDate = string.Empty;
    private float _smoothedFps = 90f;
    private int _dirtyDomains = (int)StateDomain.All;
    private bool _loggedTelemetryError;
    private string _pendingVanillaPage;
    private int _pendingVanillaPageFrames;

    internal uint NavSeq { get; private set; }
    internal uint MmOpenSeq { get; private set; }
    internal string NavTab { get; private set; } = string.Empty;
    internal bool SuppressVanillaNavHook { get; set; }
    internal bool IsBridgeHealthy => _bridge != null && _bridge.IsHealthy;
    internal bool IsBridgeHealthyOrPending => !_bridgeInitAttempted || (_bridge != null && _bridge.IsHealthy);

    public override void OnInitializeMelon()
    {
        Instance = this;
        Settings.Initialize();
        Settings.OnSettingsChanged += OnSettingsChanged;

        _bridge = new NativeBridge();
        _images = new ImageLoaderService(_bridge);
        _stateBuilder = new HostStateBuilder();
        _previewRig = new PreviewRigCoordinator();
        _liveEvents = new LiveEventWiring(this);
        UiLib = new UiLibBridge();
        Uix = new UiExpansionKitBridge(() => MarkStateDirty(StateDomain.Uix));
        RecentHistory = new RecentHistoryService(() => MarkStateDirty(StateDomain.Content | StateDomain.Social));
        PinnedContent = new PinnedContentService(() => MarkStateDirty(StateDomain.Content));
        Content = new ContentBrowserService(() => MarkStateDirty(StateDomain.Content), RecentHistory, PinnedContent);
        Social = new SocialService(() => MarkStateDirty(StateDomain.Social | StateDomain.Core), RecentHistory);
        AvatarStage = new AvatarStageManager();
        Updates = new ModUpdateService(ModVersion, () => MarkStateDirty(StateDomain.Core));
        SystemAlerts = new SystemAlertService(() => MarkStateDirty(StateDomain.Core));
        Hud = new LumenicHudManager();
        _mainSurface = new MenuSurfaceHost(NativeBridge.SurfaceMain, "MainMenu", referenceLogicalHeight: 720f);
        _quickSurface = new MenuSurfaceHost(NativeBridge.SurfaceQuick, "QuickMenu", referenceLogicalHeight: 860f);

        _prevMainEnabled = Settings.EnableMainMenu;
        _prevQuickEnabled = Settings.EnableQuickMenu;

        QuickMenuUI.PrepareSharedIcon();
        GameEvents.ApplyPatches(HarmonyInstance);
        GameEvents.SubscribeGameEvents();
        _liveEvents.Subscribe();

        MelonLogger.Msg("[LumenicMenu] Initialized.");
    }

    public override void OnLateInitializeMelon()
    {
        QuickMenuUI.Build();
        QuickMenuUI.SyncToggles();
        if (!_notifiedApiReady)
        {
            _notifiedApiReady = true;
            Api.LumenicApi.NotifyReady();
        }
    }

    public override void OnLateUpdate()
    {
        if (_bridge == null)
            return;

        float dt = Time.unscaledDeltaTime;
        if (dt > 0.0001f && dt < 0.5f)
        {
            float instantFps = 1f / dt;
            _smoothedFps = Mathf.Lerp(_smoothedFps, instantFps, 0.08f);
        }

        if (_pendingVanillaPageFrames > 0)
        {
            _pendingVanillaPageFrames--;
            if (_pendingVanillaPageFrames == 0 && !string.IsNullOrEmpty(_pendingVanillaPage))
            {
                string page = _pendingVanillaPage;
                _pendingVanillaPage = null;
                try
                {
                    if (ViewManager.Instance != null && ViewManager.Instance.IsReady)
                    {
                        if (!ViewManager.Instance.IsViewShown)
                            ViewManager.Instance.UiStateToggle(show: true);
                        ViewManager.Instance.ShowPage(page);
                    }
                }
                catch (Exception ex)
                {
                    MelonLogger.Warning($"[LumenicMenu] Deferred Vanilla ShowPage('{page}') failed: {ex.Message}");
                }
            }
        }

        Hud?.Tick();

        bool anyEnabled = Settings.EnableMainMenu || Settings.EnableQuickMenu;
        if (!anyEnabled)
        {
            SetKeyboardInputLock(false);
            return;
        }

        ViewManager vm = ViewManager.Instance;
        CVR_MenuManager qm = CVR_MenuManager.Instance;

        // Lazily load native DLL only once at least one enabled CVR menu has finished booting.
        if (!_bridge.IsHealthy)
        {
            bool mmReady = Settings.EnableMainMenu && vm != null && vm.IsReady && vm.cohtmlView != null && vm.cohtmlView.FinishedLoading;
            bool qmReady = Settings.EnableQuickMenu && qm != null && qm.IsReady && qm.cohtmlView != null && qm.cohtmlView.FinishedLoading;
            if (!mmReady && !qmReady)
                return;

            TryInitializeNativeHost();
            if (!_bridge.IsHealthy)
            {
                FallbackToVanilla("Native host failed to initialize.");
                return;
            }
        }

        bool mainActive = vm != null && IsLumenicActiveFor(vm.cohtmlView, vm, NativeBridge.SurfaceMain);
        bool quickActive = qm != null && IsLumenicActiveFor(qm.cohtmlView, qm, NativeBridge.SurfaceQuick);
        bool anyVisible = (mainActive && vm.IsViewShown) || (quickActive && qm.IsViewShown);

        // Drain up to 4 decoded local/remote images into Rust textures per frame.
        if (_images != null && _images.DrainUploads(_bridge, out bool uploadedMain, out bool uploadedQuick, maxPerFrame: 4))
        {
            if (uploadedMain) _mainSurface?.RequestForceRedraw();
            if (uploadedQuick) _quickSurface?.RequestForceRedraw();
        }

        bool mainShownNow = mainActive && vm.IsViewShown;
        bool quickShownNow = quickActive && qm.IsViewShown;
        if (mainShownNow != _prevMainShown)
        {
            _prevMainShown = mainShownNow;
            if (mainShownNow) Api.LumenicApi.Host.MarkDirty();
            Api.LumenicApi.NotifyMainMenuToggled(mainShownNow);
        }
        if (quickShownNow != _prevQuickShown)
        {
            _prevQuickShown = quickShownNow;
            if (quickShownNow) Api.LumenicApi.Host.MarkDirty();
            Api.LumenicApi.NotifyQuickMenuToggled(quickShownNow);
        }

        Api.LumenicApi.Host.Tick(_bridge, _images, mainShownNow, quickShownNow);

        // Push zero-JSON lightweight telemetry at 1 Hz (never dirties Main Menu except on minute tick or Info modal).
        if (anyVisible && (Time.unscaledTime - _lastTelemetryPollTime) >= 1.0f)
        {
            PushTelemetryIfNeeded();
        }

        // Poll dynamic BTKUILib mod pages / active toast at 4 Hz so third-party mutations stay live
        // without rebuilding or dirtying Main Menu domains.
        if (quickShownNow
            && (UiLib.IsViewingUiLibPage || UiLib.HasActiveToast)
            && (Time.unscaledTime - _lastDynamicUiPollTime) >= 0.25f)
        {
            _lastDynamicUiPollTime = Time.unscaledTime;
            MarkStateDirty(StateDomain.UiLib);
        }

        if (mainShownNow && (Time.unscaledTime - _lastSocialLivePollTime) >= 2.0f)
        {
            _lastSocialLivePollTime = Time.unscaledTime;
            if (Social != null && Social.HasLiveFriendsChanged())
            {
                MarkStateDirty(StateDomain.Social);
            }
        }

        bool anyActions = false;
        _previewRig.SyncRemoteAndPano(this, _mainSurface, _quickSurface);
        RenderTexture mainPreviewTex = _previewRig.GetOverlayTexture(this, _mainSurface, _quickSurface, NativeBridge.SurfaceMain);
        RenderTexture quickPreviewTex = _previewRig.GetOverlayTexture(this, _mainSurface, _quickSurface, NativeBridge.SurfaceQuick);

        if (anyVisible)
        {
            EnsureUiRayActive(onOpen: false);
            PushStateIfNeeded();
        }

        // Main Menu surface
        if (mainActive)
        {
            if (vm.IsViewShown)
            {
                if (!vm.cohtmlView.AllowInput)
                {
                    if (!_mainSurface.WasGrabAborted)
                    {
                        _mainSurface.ResetTransientInteraction(_bridge);
                        _mainSurface.WasGrabAborted = true;
                    }
                }
                else
                {
                    _mainSurface.WasGrabAborted = false;
                }

                _mainSurface.StepAndBlit(_bridge, vm.cohtmlView, dt, out bool mmActions, out bool mmFatal, mainPreviewTex);
                if (mmFatal)
                {
                    FallbackToVanilla($"MainMenu native error: {_bridge.GetLastError()}");
                    return;
                }
                if (mmActions) anyActions = true;
            }
        }
        else if (_mainSurface.IsTextFocused || _mainSurface.WasHovered)
        {
            _mainSurface.ResetTransientInteraction(_bridge);
        }

        // Quick Menu surface
        if (quickActive)
        {
            if (qm.IsViewShown)
            {
                bool grabAborted = !qm.cohtmlView.AllowInput || qm.IsQuickMenuInGrabMode;
                if (grabAborted)
                {
                    if (!_quickSurface.WasGrabAborted)
                    {
                        _quickSurface.ResetTransientInteraction(_bridge);
                        _quickSurface.WasGrabAborted = true;
                    }
                }
                else
                {
                    _quickSurface.WasGrabAborted = false;
                }

                _quickSurface.StepAndBlit(_bridge, qm.cohtmlView, dt, out bool qmActions, out bool qmFatal, quickPreviewTex);
                if (qmFatal)
                {
                    FallbackToVanilla($"QuickMenu native error: {_bridge.GetLastError()}");
                    return;
                }
                if (qmActions) anyActions = true;
            }
        }
        else if (_quickSurface.IsTextFocused || _quickSurface.WasHovered)
        {
            _quickSurface.ResetTransientInteraction(_bridge);
        }

        bool anyTextFocused =
            (mainActive && vm.IsViewShown && _mainSurface.IsTextFocused)
            || (quickActive && qm.IsViewShown && _quickSurface.IsTextFocused);
        SetKeyboardInputLock(anyTextFocused);

        if (anyActions)
        {
            string actionsJson = _bridge.PollActionsJson();
            GameActionDispatcher.DispatchActionsJson(actionsJson, this);
        }

        Content?.FlushMainThreadActions();
        Uix?.FlushPendingSaves(Time.unscaledTime);
        Settings.FlushPendingSave(Time.unscaledTime);
    }

    public override void OnApplicationQuit()
    {
        CleanupAll();
    }

    public override void OnDeinitializeMelon()
    {
        CleanupAll();
    }

    internal void MarkStateDirty()
    {
        MarkStateDirty(StateDomain.All);
    }

    internal void MarkStateDirty(StateDomain domains)
    {
        int mask = (int)domains;
        int initial, computed;
        do
        {
            initial = Volatile.Read(ref _dirtyDomains);
            computed = initial | mask;
            if (computed == initial)
                return;
        }
        while (Interlocked.CompareExchange(ref _dirtyDomains, computed, initial) != initial);
    }

    internal void CommitVirtualKeyboard(uint surfaceId, string fieldId, string text)
    {
        if (_bridge == null || !_bridge.IsHealthy)
            return;

        _bridge.CommitVirtualKeyboard(surfaceId, fieldId, text);
        GetSurface(surfaceId)?.RequestForceRedraw();
    }

    /// <summary>
    /// Mandatory ownership guard preventing Cohtml startup and F5 reload deadlocks.
    /// </summary>
    internal bool IsLumenicActiveFor(CohtmlControlledView view, out uint surfaceId)
    {
        surfaceId = 0;
        if (view == null)
            return false;

        ViewManager vm = ViewManager.Instance;
        if (vm != null && ReferenceEquals(vm.cohtmlView, view))
        {
            surfaceId = NativeBridge.SurfaceMain;
            return IsLumenicActiveFor(view, vm, NativeBridge.SurfaceMain);
        }

        CVR_MenuManager qm = CVR_MenuManager.Instance;
        if (qm != null && ReferenceEquals(qm.cohtmlView, view))
        {
            surfaceId = NativeBridge.SurfaceQuick;
            return IsLumenicActiveFor(view, qm, NativeBridge.SurfaceQuick);
        }

        return false;
    }

    internal bool IsLumenicActiveFor(CohtmlControlledView view, CVRUIManagerBase manager, uint surfaceId)
    {
        bool enabled = surfaceId == NativeBridge.SurfaceMain
            ? Settings.EnableMainMenu
            : Settings.EnableQuickMenu;

        return enabled
            && _bridge != null
            && _bridge.IsHealthy
            && view != null
            && manager != null
            && view.FinishedLoading
            && manager.IsReady
            && view.ViewTextureMips != null;
    }

    internal void HandlePointerInput(uint surfaceId, float x, float y, bool isDown, bool isUp, float scrollValue)
    {
        MenuSurfaceHost surf = GetSurface(surfaceId);
        if (surf == null || _bridge == null || !_bridge.IsHealthy)
            return;

        surf.HitThisFrame = true;
        _bridge.PointerMove(surfaceId, x, y);
        if (isDown)
        {
            surf.HasDiscreteInput = true;
            _bridge.PointerDown(surfaceId, 0);
        }
        if (isUp)
        {
            surf.HasDiscreteInput = true;
            _bridge.PointerUp(surfaceId, 0);
        }
        if (scrollValue != 0f)
        {
            bool isVr = MetaPort.Instance != null && MetaPort.Instance.isUsingVr;
            float mult = isVr ? Settings.VrScrollMultiplier : Settings.DesktopScrollMultiplier;
            _bridge.PointerScroll(surfaceId, 0f, scrollValue * mult);
        }
    }

    internal void HandlePointerRelease(uint surfaceId, float x, float y)
    {
        MenuSurfaceHost surf = GetSurface(surfaceId);
        if (surf == null || _bridge == null || !_bridge.IsHealthy)
            return;

        // Leave BEFORE PointerUp so pointer_pos is cleared (preventing phantom button clicks when
        // dragged off the quad) while active slider drags still release/commit cleanly.
        _bridge.PointerLeave(surfaceId);
        _bridge.PointerUp(surfaceId, 0);
        surf.HitThisFrame = false;
        surf.HasDiscreteInput = true;
    }

    internal void HandleFingerPoke(uint surfaceId, float x, float y)
    {
        MenuSurfaceHost surf = GetSurface(surfaceId);
        if (surf == null || _bridge == null || !_bridge.IsHealthy)
            return;

        surf.HitThisFrame = true;
        surf.HasDiscreteInput = true;
        _bridge.PointerMove(surfaceId, x, y);
        _bridge.PointerDown(surfaceId, 0);
        _bridge.PointerUp(surfaceId, 0);
    }

    internal void ClearSurfaceFocus(uint surfaceId)
    {
        if (_bridge != null && _bridge.IsHealthy)
        {
            _bridge.ClearFocus(surfaceId);
        }
    }

    internal void HandleKeyDown(uint surfaceId, int vkCode, uint modFlags)
    {
        if (KeyboardManager.Instance != null && KeyboardManager.Instance.IsViewShown)
            return;

        MenuSurfaceHost surf = GetSurface(surfaceId);
        if (surf == null || _bridge == null || !_bridge.IsHealthy)
            return;

        if (VkKeyMapper.TryMapKeyDown(vkCode, modFlags, surf.IsTextFocused, out uint lumenicKey))
        {
            surf.HasDiscreteInput = true;
            _bridge.KeyDown(surfaceId, lumenicKey, modFlags);
        }
    }

    internal void HandleCharInput(uint surfaceId, char character)
    {
        if (KeyboardManager.Instance != null && KeyboardManager.Instance.IsViewShown)
            return;

        MenuSurfaceHost surf = GetSurface(surfaceId);
        if (surf == null || _bridge == null || !_bridge.IsHealthy)
            return;

        surf.HasDiscreteInput = true;
        _bridge.TextInput(surfaceId, character.ToString());
    }

    internal void OnMenuOpened(uint surfaceId)
    {
        if (surfaceId == NativeBridge.SurfaceMain)
        {
            MmOpenSeq++;
            SettingsBridge.InvalidateCache();
            MarkStateDirty(StateDomain.All);
        }
        else
        {
            MarkStateDirty(StateDomain.Core | StateDomain.UiLib);
        }
        EnsureUiRayActive(onOpen: true);
        GetSurface(surfaceId)?.RequestForceRedraw();
        AvatarSettingsBridge.RefreshPreviewRendering();
    }

    private static void EnsureUiRayActive(bool onOpen)
    {
        PlayerSetup ps = PlayerSetup.Instance;
        if (ps == null)
            return;

        bool isVr = MetaPort.Instance != null && MetaPort.Instance.isUsingVr;
        if (!isVr)
        {
            if (ps.desktopRay != null && !ps.desktopRay.uiActive)
            {
                ps.desktopRay.uiActive = true;
            }
            return;
        }

        if (!onOpen)
            return;

        bool leftActive = ps.vrRayLeft != null && ps.vrRayLeft.uiActive;
        bool rightActive = ps.vrRayRight != null && ps.vrRayRight.uiActive;
        if (leftActive || rightActive)
            return;

        CVRHand preferred = CVR_MenuManager.Instance != null
            ? CVR_MenuManager.Instance.lastInteractedHand
            : CVRHand.Right;
        ControllerRay target = preferred == CVRHand.Left
            ? (ps.vrRayLeft ?? ps.vrRayRight)
            : (ps.vrRayRight ?? ps.vrRayLeft);
        if (target != null)
        {
            target.uiActive = true;
        }
    }

    internal void OnMenuClosedOrReloaded(uint surfaceId)
    {
        GetSurface(surfaceId)?.ResetTransientInteraction(_bridge);
        AvatarSettingsBridge.RefreshPreviewRendering();
        _previewRig?.SyncRemoteAndPano(this, _mainSurface, _quickSurface);
        SetKeyboardInputLock(false);
    }

    internal bool ShouldRenderAvatarPreview()
    {
        return _previewRig != null && _previewRig.ShouldRenderLocalAvatarPreview(this, _mainSurface, _quickSurface);
    }

    internal void RotateActiveAvatarPreview(float delta, bool isDragging)
    {
        _previewRig?.RotateActive(this, _mainSurface, _quickSurface, delta, isDragging);
    }

    internal void SyncAvatarPreview(uint surfaceId, bool active, string target, float normX, float normY, float normW, float normH)
    {
        _previewRig?.SyncSurfacePreview(this, _mainSurface, _quickSurface, surfaceId, active, target, normX, normY, normW, normH);
    }

    internal void OnVrModeSwitched(bool isVr)
    {
        _mainSurface?.RequestForceRedraw();
        _quickSurface?.RequestForceRedraw();
        MarkStateDirty(StateDomain.Core);
    }

    internal void NavigateMainMenuTab(string tab)
    {
        if (string.IsNullOrWhiteSpace(tab))
            return;

        NavTab = tab;
        NavSeq++;
        _mainSurface?.RequestForceRedraw();
        MarkStateDirty(StateDomain.Core);
    }

    internal void OpenVanillaPageDeferred(string page)
    {
        SwitchToVanilla("main");
        if (!string.IsNullOrWhiteSpace(page))
        {
            _pendingVanillaPage = page;
            _pendingVanillaPageFrames = 3;
        }
    }

    internal void SwitchToVanilla(string target)
    {
        if (target == "main")
        {
            Settings.SetEnableMainMenu(false);
        }
        else if (target == "quick")
        {
            Settings.SetEnableQuickMenu(false);
        }
        else
        {
            Settings.SetEnableMainMenu(false);
            Settings.SetEnableQuickMenu(false);
        }
    }

    private void OnSettingsChanged()
    {
        QuickMenuUI.SyncToggles();

        bool mainNow = Settings.EnableMainMenu;
        bool quickNow = Settings.EnableQuickMenu;

        if (_prevMainEnabled && !mainNow)
        {
            _mainSurface?.ResetTransientInteraction(_bridge);
            MenuSurfaceHost.RestoreVanillaFrame(ViewManager.Instance?.cohtmlView);
        }
        else if (!_prevMainEnabled && mainNow)
        {
            _mainSurface?.RequestForceRedraw();
        }

        if (_prevQuickEnabled && !quickNow)
        {
            _quickSurface?.ResetTransientInteraction(_bridge);
            AvatarStage?.RestoreMiniMeScale();
            MenuSurfaceHost.RestoreVanillaFrame(CVR_MenuManager.Instance?.cohtmlView);
        }
        else if (!_prevQuickEnabled && quickNow)
        {
            _quickSurface?.RequestForceRedraw();
        }

        if (!mainNow && !quickNow)
        {
            SetKeyboardInputLock(false);
        }

        _prevMainEnabled = mainNow;
        _prevQuickEnabled = quickNow;
        AvatarSettingsBridge.RefreshPreviewRendering();
        _previewRig?.SyncRemoteAndPano(this, _mainSurface, _quickSurface);
        MarkStateDirty(StateDomain.Core | StateDomain.Settings);
    }

    private void TryInitializeNativeHost()
    {
        _bridgeInitAttempted = true;
        GetCurrentSurfaceDimensions(out uint mmW, out uint mmH, out uint qmW, out uint qmH);
        if (_bridge.Load(mmW, mmH, qmW, qmH, Settings.UiScale))
        {
            _stateBuilder?.InvalidateCache();
            _lastTelFps = int.MinValue;
            MarkStateDirty(StateDomain.All);
            Api.LumenicApi.Host.InvalidateNativeCache();
            _images?.InvalidateUploaded();
            _mainSurface?.ClearLiveOverlay();
            _quickSurface?.ClearLiveOverlay();
            _mainSurface?.InvalidateNativeSurface();
            _quickSurface?.InvalidateNativeSurface();
            AvatarSettingsBridge.RefreshPreviewRendering();
            if (!_notifiedApiReady)
            {
                _notifiedApiReady = true;
                Api.LumenicApi.NotifyReady();
            }
        }
    }

    private void GetCurrentSurfaceDimensions(out uint mmW, out uint mmH, out uint qmW, out uint qmH)
    {
        CohtmlControlledView mmView = ViewManager.Instance?.cohtmlView;
        CohtmlControlledView qmView = CVR_MenuManager.Instance?.cohtmlView;

        mmW = (uint)Math.Max(1, mmView != null && mmView.Width > 0 ? mmView.Width : 2560);
        mmH = (uint)Math.Max(1, mmView != null && mmView.Height > 0 ? mmView.Height : 1440);
        qmW = (uint)Math.Max(1, qmView != null && qmView.Width > 0 ? qmView.Width : 2048);
        qmH = (uint)Math.Max(1, qmView != null && qmView.Height > 0 ? qmView.Height : 2048);
    }

    private void PushTelemetryIfNeeded()
    {
        if (_bridge == null || !_bridge.IsHealthy || _stateBuilder == null)
            return;

        _lastTelemetryPollTime = Time.unscaledTime;
        try
        {
            _stateBuilder.ReadTelemetry(_smoothedFps, out int fps, out int gamePing, out int voicePing, out string clockText, out string dateText);

            if (fps == _lastTelFps
                && gamePing == _lastTelGamePing
                && voicePing == _lastTelVoicePing
                && clockText == _lastTelClock
                && dateText == _lastTelDate)
            {
                return;
            }

            _lastTelFps = fps;
            _lastTelGamePing = gamePing;
            _lastTelVoicePing = voicePing;
            _lastTelClock = clockText;
            _lastTelDate = dateText;
            _bridge.PushTelemetry(fps, gamePing, voicePing, clockText, dateText);
            _loggedTelemetryError = false;
        }
        catch (Exception ex)
        {
            if (!_loggedTelemetryError)
            {
                _loggedTelemetryError = true;
                MelonLogger.Warning($"[LumenicMenu] Telemetry poll failed: {ex.Message}");
            }
        }
    }

    private void PushStateIfNeeded()
    {
        if (!_bridge.IsHealthy || _stateBuilder == null)
            return;

        StateDomain dirty = (StateDomain)Interlocked.Exchange(ref _dirtyDomains, (int)StateDomain.None);
        if (dirty == StateDomain.None)
            return;

        GetCurrentSurfaceDimensions(out uint mmW, out uint mmH, out uint qmW, out uint qmH);
        _stateBuilder.PushDirtyDomains(
            dirty,
            this,
            _images,
            _mainSurface,
            _quickSurface,
            _bridge,
            mmW,
            mmH,
            qmW,
            qmH);
    }

    private void FallbackToVanilla(string reason)
    {
        MelonLogger.Error($"[LumenicMenu] Reverting to Vanilla UI: {reason}");
        SetKeyboardInputLock(false);
        _mainSurface?.ClearLiveOverlay();
        _quickSurface?.ClearLiveOverlay();
        Settings.SetEnableMainMenu(false);
        Settings.SetEnableQuickMenu(false);
        AvatarSettingsBridge.RefreshPreviewRendering();
        MenuSurfaceHost.RestoreVanillaFrame(ViewManager.Instance?.cohtmlView);
        MenuSurfaceHost.RestoreVanillaFrame(CVR_MenuManager.Instance?.cohtmlView);
        QuickMenuUI.SyncToggles();
    }

    private void SetKeyboardInputLock(bool locked)
    {
        if (_keyboardLocked == locked)
            return;

        _keyboardLocked = locked;
        try
        {
            CVRInputManager._moduleKeyboard?.InputLock?.SetLock(InputLockReason, locked);
        }
        catch (Exception ex)
        {
            MelonLogger.Warning($"[LumenicMenu] Failed to update keyboard InputLock({locked}): {ex.Message}");
        }
    }

    private MenuSurfaceHost GetSurface(uint surfaceId)
    {
        return surfaceId == NativeBridge.SurfaceMain ? _mainSurface : _quickSurface;
    }

    private void CleanupAll()
    {
        Settings.FlushPendingSave(0f, force: true);
        Uix?.FlushPendingSaves(0f, force: true);
        Uix?.Dispose();
        Uix = null;
        RecentHistory?.FlushSave();
        PinnedContent?.FlushSave();
        Settings.OnSettingsChanged -= OnSettingsChanged;
        _liveEvents?.Unsubscribe();
        GameEvents.UnsubscribeGameEvents();
        try { HarmonyInstance.UnpatchSelf(); } catch { }
        SetKeyboardInputLock(false);

        _mainSurface?.DetachAndDispose(_bridge);
        _mainSurface = null;
        _quickSurface?.DetachAndDispose(_bridge);
        _quickSurface = null;

        AvatarStage?.Dispose();
        AvatarStage = null;

        PlayerPreviewRig.DisposeInstance();

        UiSoundPlayer.Dispose();

        _images?.Dispose();
        _images = null;

        _bridge?.Dispose();
        _bridge = null;

        Hud?.Dispose();
        Hud = null;
    }
}
