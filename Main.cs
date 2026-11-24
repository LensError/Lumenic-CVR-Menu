using System;
using ABI.CCK.Components;
using ABI_RC.Core.InteractionSystem;
using ABI_RC.Core.Networking.GameServer;
using ABI_RC.Core.Player;
using ABI_RC.Core.PropManagement;
using GameCommon.Packets.Garrison.GameServer;
using ABI_RC.Core.Savior;
using ABI_RC.Core.UI;
using ABI_RC.Core.UI.UIRework;
using ABI_RC.Core.UI.UIRework.Managers;
using ABI_RC.Systems.AvatarScaling;
using ABI_RC.Systems.GameEventSystem;
using ABI_RC.Systems.InputManagement;
using ABI_RC.Systems.PlayerColors;
using LumenicMenu.Bridge;
using LumenicMenu.Native;
using MelonLoader;
using UnityEngine;

[assembly: MelonInfo(typeof(LumenicMenu.LumenicMenuMod), "LumenicMenu", "0.2.3", "LensError")]
[assembly: MelonGame(null, "ChilloutVR")]

namespace LumenicMenu;

public sealed class LumenicMenuMod : MelonMod
{
    private const string InputLockReason = "LumenicMenu";

    public static LumenicMenuMod Instance { get; private set; }

    private NativeBridge _bridge;
    private MenuSurfaceHost _mainSurface;
    private MenuSurfaceHost _quickSurface;
    private ImageLoaderService _images;
    private HostStateBuilder _stateBuilder;

    internal UiLibBridge UiLib { get; private set; }
    internal UiExpansionKitBridge Uix { get; private set; }
    internal RecentHistoryService RecentHistory { get; private set; }
    internal PinnedContentService PinnedContent { get; private set; }
    internal ContentBrowserService Content { get; private set; }
    internal SocialService Social { get; private set; }
    internal AvatarStageManager AvatarStage { get; private set; }

    private bool _keyboardLocked;
    private bool _prevMainEnabled;
    private bool _prevQuickEnabled;
    private string _lastPushedStateJson = string.Empty;
    private float _lastTelemetryPollTime = -10f;
    private float _lastDynamicUiPollTime = -10f;
    private int _lastTelFps = int.MinValue;
    private int _lastTelGamePing = int.MinValue;
    private int _lastTelVoicePing = int.MinValue;
    private string _lastTelClock = string.Empty;
    private string _lastTelDate = string.Empty;
    private float _smoothedFps = 90f;
    private volatile bool _stateDirty = true;
    private bool _liveEventsSubscribed;
    private bool _loggedTelemetryError;
    private string _mainPreviewTarget = "local";
    private string _quickPreviewTarget = "local";
    private string _pendingVanillaPage;
    private int _pendingVanillaPageFrames;

    internal uint NavSeq { get; private set; }
    internal string NavTab { get; private set; } = string.Empty;
    internal bool SuppressVanillaNavHook { get; set; }

    public override void OnInitializeMelon()
    {
        Instance = this;
        Settings.Initialize();
        Settings.OnSettingsChanged += OnSettingsChanged;

        _bridge = new NativeBridge();
        _images = new ImageLoaderService();
        _stateBuilder = new HostStateBuilder();
        UiLib = new UiLibBridge();
        Uix = new UiExpansionKitBridge(MarkStateDirty);
        RecentHistory = new RecentHistoryService(MarkStateDirty);
        PinnedContent = new PinnedContentService(MarkStateDirty);
        Content = new ContentBrowserService(MarkStateDirty, RecentHistory, PinnedContent);
        Social = new SocialService(MarkStateDirty, RecentHistory);
        AvatarStage = new AvatarStageManager();
        _mainSurface = new MenuSurfaceHost(NativeBridge.SurfaceMain, "MainMenu", referenceLogicalHeight: 720f);
        _quickSurface = new MenuSurfaceHost(NativeBridge.SurfaceQuick, "QuickMenu", referenceLogicalHeight: 800f);

        _prevMainEnabled = Settings.EnableMainMenu;
        _prevQuickEnabled = Settings.EnableQuickMenu;

        QuickMenuUI.PrepareSharedIcon();
        GameEvents.ApplyPatches(HarmonyInstance);
        GameEvents.SubscribeGameEvents();
        SubscribeLiveStateEvents();

        MelonLogger.Msg("[LumenicMenu] Initialized.");
    }

    public override void OnLateInitializeMelon()
    {
        QuickMenuUI.Build();
        QuickMenuUI.SyncToggles();
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

        // Push zero-JSON lightweight telemetry at 1 Hz (never dirties Main Menu except on minute tick or Info modal).
        if (anyVisible && (Time.unscaledTime - _lastTelemetryPollTime) >= 1.0f)
        {
            PushTelemetryIfNeeded();
        }

        // Poll dynamic BTKUILib mod pages / active toast at 4 Hz so third-party mutations stay live.
        if (anyVisible
            && (UiLib.IsViewingUiLibPage || UiLib.HasActiveToast)
            && (Time.unscaledTime - _lastDynamicUiPollTime) >= 0.25f)
        {
            _lastDynamicUiPollTime = Time.unscaledTime;
            _stateDirty = true;
        }

        bool anyActions = false;
        SyncRemotePlayerPreviewRig();
        RenderTexture mainPreviewTex = GetOverlayTextureForSurface(NativeBridge.SurfaceMain);
        RenderTexture quickPreviewTex = GetOverlayTextureForSurface(NativeBridge.SurfaceQuick);

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

                PushStateIfNeeded();

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

                PushStateIfNeeded();

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
        _stateDirty = true;
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
            SettingsBridge.InvalidateCache();
        }
        GetSurface(surfaceId)?.RequestForceRedraw();
        AvatarSettingsBridge.RefreshPreviewRendering();
        _stateDirty = true;
    }

    internal void OnMenuClosedOrReloaded(uint surfaceId)
    {
        GetSurface(surfaceId)?.ResetTransientInteraction(_bridge);
        AvatarSettingsBridge.RefreshPreviewRendering();
        SyncRemotePlayerPreviewRig();
        SetKeyboardInputLock(false);
    }

    internal bool ShouldRenderAvatarPreview()
    {
        ViewManager vm = ViewManager.Instance;
        bool mainWants = vm != null
            && vm.IsViewShown
            && IsLumenicActiveFor(vm.cohtmlView, vm, NativeBridge.SurfaceMain)
            && _mainSurface != null
            && _mainSurface.IsLiveOverlayActive
            && PlayerPreviewRig.IsLocalTarget(_mainPreviewTarget);

        CVR_MenuManager qm = CVR_MenuManager.Instance;
        bool quickWants = qm != null
            && qm.IsViewShown
            && IsLumenicActiveFor(qm.cohtmlView, qm, NativeBridge.SurfaceQuick)
            && _quickSurface != null
            && _quickSurface.IsLiveOverlayActive
            && PlayerPreviewRig.IsLocalTarget(_quickPreviewTarget);

        return mainWants || quickWants;
    }

    private static bool IsWorldPanoTarget(string target)
    {
        return !string.IsNullOrEmpty(target) && target.StartsWith("worldpano:", StringComparison.Ordinal);
    }

    private string GetActiveRemotePreviewTarget()
    {
        ViewManager vm = ViewManager.Instance;
        if (vm != null
            && vm.IsViewShown
            && IsLumenicActiveFor(vm.cohtmlView, vm, NativeBridge.SurfaceMain)
            && _mainSurface != null
            && _mainSurface.IsLiveOverlayActive
            && !PlayerPreviewRig.IsLocalTarget(_mainPreviewTarget)
            && !IsWorldPanoTarget(_mainPreviewTarget))
        {
            return _mainPreviewTarget;
        }

        CVR_MenuManager qm = CVR_MenuManager.Instance;
        if (qm != null
            && qm.IsViewShown
            && IsLumenicActiveFor(qm.cohtmlView, qm, NativeBridge.SurfaceQuick)
            && _quickSurface != null
            && _quickSurface.IsLiveOverlayActive
            && !PlayerPreviewRig.IsLocalTarget(_quickPreviewTarget)
            && !IsWorldPanoTarget(_quickPreviewTarget))
        {
            return _quickPreviewTarget;
        }

        return null;
    }

    private string GetActiveWorldPanoUrl()
    {
        ViewManager vm = ViewManager.Instance;
        if (vm != null
            && vm.IsViewShown
            && IsLumenicActiveFor(vm.cohtmlView, vm, NativeBridge.SurfaceMain)
            && _mainSurface != null
            && _mainSurface.IsLiveOverlayActive
            && IsWorldPanoTarget(_mainPreviewTarget))
        {
            return _mainPreviewTarget.Substring("worldpano:".Length);
        }
        return null;
    }

    private void SyncRemotePlayerPreviewRig()
    {
        string remoteTarget = GetActiveRemotePreviewTarget();
        if (!string.IsNullOrEmpty(remoteTarget))
        {
            PlayerPreviewRig.Instance.SyncTarget(remoteTarget, active: true);
        }
        else if (PlayerPreviewRig.HasInstance)
        {
            PlayerPreviewRig.Instance.SyncTarget(string.Empty, active: false);
        }

        string panoUrl = GetActiveWorldPanoUrl();
        AvatarSettingsBridge.SyncWorldPano(panoUrl, !string.IsNullOrEmpty(panoUrl));
    }

    private RenderTexture GetOverlayTextureForSurface(uint surfaceId)
    {
        MenuSurfaceHost surf = GetSurface(surfaceId);
        if (surf == null || !surf.IsLiveOverlayActive)
            return null;

        string target = surfaceId == NativeBridge.SurfaceMain ? _mainPreviewTarget : _quickPreviewTarget;
        if (IsWorldPanoTarget(target))
            return AvatarSettingsBridge.GetWorldPanoTexture();
        if (PlayerPreviewRig.IsLocalTarget(target))
            return ShouldRenderAvatarPreview() ? AvatarSettingsBridge.GetPreviewTexture() : null;

        return PlayerPreviewRig.HasInstance ? PlayerPreviewRig.Instance.Texture : null;
    }

    internal void RotateActiveAvatarPreview(float delta, bool isDragging)
    {
        string remoteTarget = GetActiveRemotePreviewTarget();
        if (!string.IsNullOrEmpty(remoteTarget))
        {
            PlayerPreviewRig.Instance.Rotate(delta, isDragging);
        }
        else
        {
            AvatarSettingsBridge.RotatePreview(delta, isDragging);
        }
    }

    internal void SyncAvatarPreview(uint surfaceId, bool active, string target, float normX, float normY, float normW, float normH)
    {
        if (surfaceId == NativeBridge.SurfaceMain)
            _mainPreviewTarget = string.IsNullOrEmpty(target) ? "local" : target;
        else
            _quickPreviewTarget = string.IsNullOrEmpty(target) ? "local" : target;

        MenuSurfaceHost surf = GetSurface(surfaceId);
        if (surf != null)
        {
            surf.SetLiveOverlay(active, normX, normY, normW, normH);
        }
        AvatarSettingsBridge.RefreshPreviewRendering();
        SyncRemotePlayerPreviewRig();
    }

    internal void OnVrModeSwitched(bool isVr)
    {
        _mainSurface?.RequestForceRedraw();
        _quickSurface?.RequestForceRedraw();
        _stateDirty = true;
    }

    internal void NavigateMainMenuTab(string tab)
    {
        if (string.IsNullOrWhiteSpace(tab))
            return;

        NavTab = tab;
        NavSeq++;
        _mainSurface?.RequestForceRedraw();
        _stateDirty = true;
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

    private void SubscribeLiveStateEvents()
    {
        if (_liveEventsSubscribed)
            return;

        CVRGameEventSystem.Microphone.OnMute.AddListener(OnExternalStateEvent);
        CVRGameEventSystem.Microphone.OnUnmute.AddListener(OnExternalStateEvent);
        CVRGameEventSystem.World.OnLoad.AddListener(OnWorldLoadForHistory);
        CVRGameEventSystem.World.OnReady.AddListener(OnWorldReadyForHistory);
        CVRGameEventSystem.Instance.OnConnected.AddListener(OnExternalWorldEvent);
        CVRGameEventSystem.Instance.OnDisconnected.AddListener(OnExternalWorldEvent);
        CVRGameEventSystem.Player.OnJoinEntity.AddListener(OnPlayerJoinedForHistory);
        CVRGameEventSystem.Player.OnLeaveEntity.AddListener(OnExternalPlayerEvent);
        CVRGameEventSystem.Avatar.OnLocalAvatarLoad.AddListener(OnLocalAvatarLoadForHistory);
        CVRGameEventSystem.Avatar.OnLocalAvatarClear.AddListener(OnExternalAvatarEvent);
        CVRGameEventSystem.Avatar.OnRemoteAvatarLoad.AddListener(OnRemoteAvatarLoadForHistory);
        CVRGameEventSystem.Spawnable.OnPropSpawned.AddListener(OnPropSpawnedForHistory);
        GSInfoHandler.OnGSInfoUpdate += OnGsInfoUpdated;
        UniversalScaling.RulesChanged += OnExternalStateEvent;
        PlayerColorsManager.OnLocalPlayerColorChanged += OnPlayerColorsChanged;
        KeyboardManager.KeyboardClosed += OnVirtualKeyboardClosed;
        _liveEventsSubscribed = true;
    }

    private void UnsubscribeLiveStateEvents()
    {
        if (!_liveEventsSubscribed)
            return;

        CVRGameEventSystem.Microphone.OnMute.RemoveListener(OnExternalStateEvent);
        CVRGameEventSystem.Microphone.OnUnmute.RemoveListener(OnExternalStateEvent);
        CVRGameEventSystem.World.OnLoad.RemoveListener(OnWorldLoadForHistory);
        CVRGameEventSystem.World.OnReady.RemoveListener(OnWorldReadyForHistory);
        CVRGameEventSystem.Instance.OnConnected.RemoveListener(OnExternalWorldEvent);
        CVRGameEventSystem.Instance.OnDisconnected.RemoveListener(OnExternalWorldEvent);
        CVRGameEventSystem.Player.OnJoinEntity.RemoveListener(OnPlayerJoinedForHistory);
        CVRGameEventSystem.Player.OnLeaveEntity.RemoveListener(OnExternalPlayerEvent);
        CVRGameEventSystem.Avatar.OnLocalAvatarLoad.RemoveListener(OnLocalAvatarLoadForHistory);
        CVRGameEventSystem.Avatar.OnLocalAvatarClear.RemoveListener(OnExternalAvatarEvent);
        CVRGameEventSystem.Avatar.OnRemoteAvatarLoad.RemoveListener(OnRemoteAvatarLoadForHistory);
        CVRGameEventSystem.Spawnable.OnPropSpawned.RemoveListener(OnPropSpawnedForHistory);
        GSInfoHandler.OnGSInfoUpdate -= OnGsInfoUpdated;
        UniversalScaling.RulesChanged -= OnExternalStateEvent;
        PlayerColorsManager.OnLocalPlayerColorChanged -= OnPlayerColorsChanged;
        KeyboardManager.KeyboardClosed -= OnVirtualKeyboardClosed;
        _liveEventsSubscribed = false;
    }

    private void OnExternalStateEvent() => _stateDirty = true;
    private void OnExternalWorldEvent(string _) => _stateDirty = true;
    private void OnExternalPlayerEvent(CVRPlayerEntity _) => _stateDirty = true;
    private void OnExternalAvatarEvent(CVRAvatar _) => _stateDirty = true;

    private void OnWorldLoadForHistory(string worldId)
    {
        RecentHistory?.RecordWorldVisited(worldId);
        Content?.RefreshLocalRecentIfActive("worlds");
        _stateDirty = true;
    }

    private void OnWorldReadyForHistory(string worldId)
    {
        RecentHistory?.RecordWorldVisited(worldId, MetaPort.Instance?.CurrentWorldName);
        RecentHistory?.RefreshCurrentInstanceUsers();
        Content?.RefreshLocalRecentIfActive("worlds");
        _stateDirty = true;
    }

    private void OnPlayerJoinedForHistory(CVRPlayerEntity player)
    {
        RecentHistory?.RecordUserSeen(player);
        _stateDirty = true;
    }

    private void OnLocalAvatarLoadForHistory(CVRAvatar _)
    {
        string avatarId = MetaPort.Instance?.currentAvatarGuid;
        RecentHistory?.RecordAvatarUsed(avatarId);
        Content?.RefreshLocalRecentIfActive("avatars");
        _stateDirty = true;
    }

    private void OnRemoteAvatarLoadForHistory(CVRPlayerEntity player, CVRAvatar _)
    {
        if (player != null)
        {
            RecentHistory?.RecordAvatarSeen(player.ContentMetadata.AssetId, player.ContentMetadata.AssetName);
            Content?.RefreshLocalRecentIfActive("avatars");
        }
        _stateDirty = true;
    }

    private void OnPropSpawnedForHistory(string _, PropSoul prop)
    {
        RecentHistory?.RecordPropSpawned(prop);
        Content?.RefreshLocalRecentIfActive("props");
        _stateDirty = true;
    }

    private void OnGsInfoUpdated(ServerInfoPacket _)
    {
        RecentHistory?.RefreshCurrentInstanceUsers();
        _stateDirty = true;
    }
    private void OnPlayerColorsChanged(PlayerColors _) => _stateDirty = true;

    private void OnVirtualKeyboardClosed()
    {
        if (_bridge != null && _bridge.IsHealthy)
        {
            _bridge.ClearFocus(NativeBridge.SurfaceMain);
            _bridge.ClearFocus(NativeBridge.SurfaceQuick);
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
        SyncRemotePlayerPreviewRig();
        _stateDirty = true;
    }

    private void TryInitializeNativeHost()
    {
        GetCurrentSurfaceDimensions(out uint mmW, out uint mmH, out uint qmW, out uint qmH);
        if (_bridge.Load(mmW, mmH, qmW, qmH, Settings.UiScale))
        {
            _lastPushedStateJson = string.Empty;
            _lastTelFps = int.MinValue;
            _stateDirty = true;
            _images?.InvalidateUploaded();
            _mainSurface?.ClearLiveOverlay();
            _quickSurface?.ClearLiveOverlay();
            _mainSurface?.InvalidateNativeSurface();
            _quickSurface?.InvalidateNativeSurface();
            AvatarSettingsBridge.RefreshPreviewRendering();
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
        if (!_stateDirty || !_bridge.IsHealthy || _stateBuilder == null)
            return;

        _stateDirty = false;

        GetCurrentSurfaceDimensions(out uint mmW, out uint mmH, out uint qmW, out uint qmH);
        string json = _stateBuilder.BuildJson(
            this,
            _images,
            _mainSurface,
            _quickSurface,
            _bridge,
            mmW,
            mmH,
            qmW,
            qmH);

        if (json == _lastPushedStateJson)
            return;

        _lastPushedStateJson = json;
        if (!_bridge.PushStateJson(json))
        {
            MelonLogger.Warning($"[LumenicMenu] PushStateJson failed: {_bridge.GetLastError()}");
        }
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
        Uix?.FlushPendingSaves(0f, force: true);
        Uix?.Dispose();
        Uix = null;
        RecentHistory?.FlushSave();
        PinnedContent?.FlushSave();
        Settings.OnSettingsChanged -= OnSettingsChanged;
        UnsubscribeLiveStateEvents();
        GameEvents.UnsubscribeGameEvents();
        try { HarmonyInstance.UnpatchSelf(); } catch { }
        SetKeyboardInputLock(false);

        _mainSurface?.Dispose();
        _mainSurface = null;
        _quickSurface?.Dispose();
        _quickSurface = null;

        AvatarStage?.Dispose();
        AvatarStage = null;

        PlayerPreviewRig.DisposeInstance();

        UiSoundPlayer.Dispose();

        _images?.Dispose();
        _images = null;

        _bridge?.Dispose();
        _bridge = null;
    }
}
