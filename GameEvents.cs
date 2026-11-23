using System;
using System.Reflection;
using System.Threading.Tasks;
using ABI.CCK.Components;
using ABI_RC.Core;
using ABI_RC.Core.InteractionSystem;
using ABI_RC.Core.Networking.API.Responses;
using ABI_RC.Core.Player;
using ABI_RC.Core.Savior;
using ABI_RC.Core.UI;
using ABI_RC.Core.UI.UIRework;
using ABI_RC.Systems.ContentClones;
using ABI_RC.Systems.GameEventSystem;
using ABI_RC.Systems.InputManagement;
using ABI_RC.Systems.UI.MenuPreview;
using ABI_RC.Systems.UI.UILib;
using ABI_RC.Systems.UI.UILib.UIObjects;
using ABI_RC.Systems.UI.UILib.UIObjects.Objects;
using cohtml.InputSystem;
using cohtml.Net;
using HarmonyLib;
using LumenicMenu.Native;
using MelonLoader;
using UnityEngine;

namespace LumenicMenu;

/// <summary>
/// Isolated Harmony patches and CVRGameEventSystem subscriptions for the 5 verified CVR menu seams.
/// </summary>
internal static class GameEvents
{
    private static bool _eventsSubscribed;
    private static FieldInfo _joystickScrollField;
    private static FieldInfo _previewCameraField;
    private static FieldInfo _previewIsDraggingField;
    private static FieldInfo _previewIsPageOpenField;
    private static MethodInfo _previewOnRenderingChangedMethod;

    public static void ApplyPatches(HarmonyLib.Harmony harmony)
    {
        _joystickScrollField = AccessTools.Field(typeof(CohtmlViewInputHandler), "_joystickScroll");
        _previewCameraField = AccessTools.Field(typeof(MenuPreviewRig), "Preview");
        _previewIsDraggingField = AccessTools.Field(typeof(MenuPreviewRig), "IsDragging");
        _previewIsPageOpenField = AccessTools.Field(typeof(MenuPreviewRig), "_isPageOpen");
        _previewOnRenderingChangedMethod = AccessTools.Method(typeof(MenuPreviewRig), "OnRenderingChanged", new[] { typeof(bool) });

        PatchSafe(
            harmony,
            AccessTools.PropertyGetter(typeof(CohtmlControlledView), nameof(CohtmlControlledView.ShouldAdvance)),
            prefix: nameof(Prefix_CohtmlControlledView_ShouldAdvance));

        PatchSafe(
            harmony,
            AccessTools.PropertyGetter(typeof(CohtmlControlledView), nameof(CohtmlControlledView.ShouldRender)),
            prefix: nameof(Prefix_CohtmlControlledView_ShouldRender));

        PatchSafe(
            harmony,
            AccessTools.Method(typeof(CohtmlViewInputHandler), nameof(CohtmlViewInputHandler.DoViewInput)),
            prefix: nameof(Prefix_CohtmlViewInputHandler_DoViewInput));

        PatchSafe(
            harmony,
            AccessTools.Method(typeof(CohtmlViewInputHandler), nameof(CohtmlViewInputHandler.ReleaseViewInput)),
            prefix: nameof(Prefix_CohtmlViewInputHandler_ReleaseViewInput));

        PatchSafe(
            harmony,
            AccessTools.Method(typeof(CVR_Menu_Pointer_Reciever), "OnTriggerEnter", new[] { typeof(Collider) }),
            prefix: nameof(Prefix_PointerReceiver_OnTriggerEnter));

        PatchSafe(
            harmony,
            AccessTools.Method(typeof(CVRUIManagerBaseInput), "OnKeyEvent", new[] { typeof(KeyEvent), typeof(InputEventWrapper) }),
            prefix: nameof(Prefix_CVRUIManagerBaseInput_OnKeyEvent));

        PatchSafe(
            harmony,
            AccessTools.Method(typeof(CVRUIManagerBaseInput), "OnCharEvent", new[] { typeof(KeyEvent), typeof(char) }),
            prefix: nameof(Prefix_CVRUIManagerBaseInput_OnCharEvent));

        PatchSafe(
            harmony,
            AccessTools.Method(typeof(CohtmlInputHandler), "SendKeyToFocusedView", new[] { typeof(KeyEvent), typeof(InputEventWrapper) }),
            prefix: nameof(Prefix_CohtmlInputHandler_BlockFocusedView));

        PatchSafe(
            harmony,
            AccessTools.Method(typeof(CohtmlInputHandler), "SendCharToFocusedView", new[] { typeof(KeyEvent), typeof(char) }),
            prefix: nameof(Prefix_CohtmlInputHandler_BlockFocusedView));

        // UILib Mod Compatibility hooks (intercept page navigation, modals, and toasts when Lumenic QM is active)
        PatchSafe(
            harmony,
            AccessTools.Method(typeof(Page), nameof(Page.OpenPage), new[] { typeof(bool), typeof(bool) }),
            prefix: nameof(Prefix_UiLib_Page_OpenPage));

        PatchSafe(
            harmony,
            AccessTools.Method(typeof(QuickMenuAPI), nameof(QuickMenuAPI.GoBack)),
            prefix: nameof(Prefix_UiLib_GoBack));

        PatchSafe(
            harmony,
            AccessTools.Method(typeof(QuickMenuAPI), nameof(QuickMenuAPI.ShowConfirm)),
            prefix: nameof(Prefix_UiLib_ShowConfirm));

        PatchSafe(
            harmony,
            AccessTools.Method(typeof(QuickMenuAPI), nameof(QuickMenuAPI.ShowNotice)),
            prefix: nameof(Prefix_UiLib_ShowNotice));

        PatchSafe(
            harmony,
            AccessTools.Method(typeof(QuickMenuAPI), nameof(QuickMenuAPI.OpenMultiSelect)),
            prefix: nameof(Prefix_UiLib_OpenMultiSelect));

        PatchSafe(
            harmony,
            AccessTools.Method(typeof(QuickMenuAPI), nameof(QuickMenuAPI.OpenNumberInput)),
            prefix: nameof(Prefix_UiLib_OpenNumberInput));

        PatchSafe(
            harmony,
            AccessTools.Method(typeof(QuickMenuAPI), nameof(QuickMenuAPI.ShowAlertToast)),
            prefix: nameof(Prefix_UiLib_ShowAlertToast));

        PatchSafe(
            harmony,
            AccessTools.Method(typeof(MiniMe), "UpdateQuickMenuPosition"),
            postfix: nameof(Postfix_MiniMe_UpdateQuickMenuPosition));

        PatchSafe(
            harmony,
            AccessTools.Method(typeof(MenuPreviewRig), "UpdateRendering"),
            prefix: nameof(Prefix_MenuPreviewRig_UpdateRendering));

        // External / QM -> Main Menu navigation sync (keeps Lumenic Main Menu in sync with ViewManager)
        PatchSafe(
            harmony,
            AccessTools.Method(typeof(ViewManager), nameof(ViewManager.ShowPage), new[] { typeof(string) }),
            postfix: nameof(Postfix_ViewManager_ShowPage));

        PatchSafe(
            harmony,
            AccessTools.Method(typeof(ViewManager), nameof(ViewManager.RequestUserDetailsPage), new[] { typeof(string) }),
            postfix: nameof(Postfix_ViewManager_RequestUserDetailsPage));

        PatchSafe(
            harmony,
            AccessTools.Method(typeof(ViewManager), nameof(ViewManager.RequestWorldDetailsPage), new[] { typeof(string) }),
            postfix: nameof(Postfix_ViewManager_RequestWorldDetailsPage));

        PatchSafe(
            harmony,
            AccessTools.Method(typeof(ViewManager), nameof(ViewManager.RequestAvatarDetailsPage), new[] { typeof(string) }),
            postfix: nameof(Postfix_ViewManager_RequestAvatarDetailsPage));

        PatchSafe(
            harmony,
            AccessTools.Method(typeof(ViewManager), nameof(ViewManager.GetPropDetails), new[] { typeof(string) }),
            postfix: nameof(Postfix_ViewManager_GetPropDetails));

        PatchSafe(
            harmony,
            AccessTools.Method(typeof(ViewManager), nameof(ViewManager.OpenInstanceDetails), new[] { typeof(string) }),
            postfix: nameof(Postfix_ViewManager_OpenInstanceDetails));

        PatchSafe(
            harmony,
            AccessTools.Method(typeof(ViewManager), "ShowGroupModeratedInstanceAlert", new[] { typeof(InstanceDetailsResponse) }),
            prefix: nameof(Prefix_ViewManager_ShowGroupModeratedInstanceAlert));

        PatchSafe(
            harmony,
            AccessTools.Method(typeof(ViewManager), nameof(ViewManager.CancelAndCloseGroupModerationAlertModal)),
            prefix: nameof(Prefix_ViewManager_CancelAndCloseGroupModerationAlertModal));
    }

    public static void SubscribeGameEvents()
    {
        if (_eventsSubscribed)
            return;

        CVRGameEventSystem.MainMenu.OnOpen.AddListener(OnMainMenuOpen);
        CVRGameEventSystem.MainMenu.OnClose.AddListener(OnMainMenuClose);
        CVRGameEventSystem.MainMenu.OnReload.AddListener(OnMainMenuReload);

        CVRGameEventSystem.QuickMenu.OnOpen.AddListener(OnQuickMenuOpen);
        CVRGameEventSystem.QuickMenu.OnClose.AddListener(OnQuickMenuClose);
        CVRGameEventSystem.QuickMenu.OnReload.AddListener(OnQuickMenuReload);

        CVRGameEventSystem.VRModeSwitch.OnPostSwitch.AddListener(OnVrModePostSwitch);

        _eventsSubscribed = true;
    }

    public static void UnsubscribeGameEvents()
    {
        if (!_eventsSubscribed)
            return;

        CVRGameEventSystem.MainMenu.OnOpen.RemoveListener(OnMainMenuOpen);
        CVRGameEventSystem.MainMenu.OnClose.RemoveListener(OnMainMenuClose);
        CVRGameEventSystem.MainMenu.OnReload.RemoveListener(OnMainMenuReload);

        CVRGameEventSystem.QuickMenu.OnOpen.RemoveListener(OnQuickMenuOpen);
        CVRGameEventSystem.QuickMenu.OnClose.RemoveListener(OnQuickMenuClose);
        CVRGameEventSystem.QuickMenu.OnReload.RemoveListener(OnQuickMenuReload);

        CVRGameEventSystem.VRModeSwitch.OnPostSwitch.RemoveListener(OnVrModePostSwitch);

        _eventsSubscribed = false;
    }

    private static void PatchSafe(HarmonyLib.Harmony harmony, MethodBase target, string prefix = null, string postfix = null)
    {
        if (target == null)
        {
            MelonLogger.Warning($"[LumenicMenu] Harmony target missing for {prefix ?? postfix}; skipping.");
            return;
        }

        HarmonyMethod prefixHarmony = null;
        if (!string.IsNullOrEmpty(prefix))
        {
            MethodInfo prefixMethod = AccessTools.Method(typeof(GameEvents), prefix);
            if (prefixMethod == null)
            {
                MelonLogger.Error($"[LumenicMenu] Patch method {prefix} not found.");
                return;
            }
            prefixHarmony = new HarmonyMethod(prefixMethod);
        }

        HarmonyMethod postfixHarmony = null;
        if (!string.IsNullOrEmpty(postfix))
        {
            MethodInfo postfixMethod = AccessTools.Method(typeof(GameEvents), postfix);
            if (postfixMethod == null)
            {
                MelonLogger.Error($"[LumenicMenu] Patch method {postfix} not found.");
                return;
            }
            postfixHarmony = new HarmonyMethod(postfixMethod);
        }

        try
        {
            harmony.Patch(target, prefix: prefixHarmony, postfix: postfixHarmony);
        }
        catch (Exception ex)
        {
            MelonLogger.Error($"[LumenicMenu] Failed to patch {target.DeclaringType?.Name}.{target.Name}: {ex.Message}");
        }
    }

    private static void OnMainMenuOpen()
    {
        LumenicMenuMod.Instance?.OnMenuOpened(NativeBridge.SurfaceMain);
    }

    private static void OnMainMenuClose()
    {
        LumenicMenuMod.Instance?.OnMenuClosedOrReloaded(NativeBridge.SurfaceMain);
    }

    private static void OnMainMenuReload()
    {
        LumenicMenuMod.Instance?.OnMenuClosedOrReloaded(NativeBridge.SurfaceMain);
    }

    private static void OnQuickMenuOpen()
    {
        LumenicMenuMod.Instance?.OnMenuOpened(NativeBridge.SurfaceQuick);
    }

    private static void OnQuickMenuClose()
    {
        LumenicMenuMod.Instance?.OnMenuClosedOrReloaded(NativeBridge.SurfaceQuick);
    }

    private static void OnQuickMenuReload()
    {
        LumenicMenuMod.Instance?.OnMenuClosedOrReloaded(NativeBridge.SurfaceQuick);
    }

    private static void OnVrModePostSwitch(bool isVr)
    {
        LumenicMenuMod.Instance?.OnVrModeSwitched(isVr);
    }

    private static bool IsJoystickScrollEnabled()
    {
        try
        {
            if (_joystickScrollField != null)
                return (bool)_joystickScrollField.GetValue(null);
        }
        catch
        {
            // Fall back to true if private field is unavailable.
        }
        return true;
    }

    // --- Seam 1: Pause Cohtml Advance & Render when Lumenic owns the view ---

    private static bool Prefix_CohtmlControlledView_ShouldAdvance(CohtmlControlledView __instance, ref bool __result)
    {
        try
        {
            var mod = LumenicMenuMod.Instance;
            if (mod != null && mod.IsLumenicActiveFor(__instance, out _))
            {
                __result = false;
                return false;
            }
        }
        catch (Exception ex)
        {
            MelonLogger.Error($"[LumenicMenu] ShouldAdvance patch error: {ex.Message}");
        }
        return true;
    }

    private static bool Prefix_CohtmlControlledView_ShouldRender(CohtmlControlledView __instance, ref bool __result)
    {
        try
        {
            var mod = LumenicMenuMod.Instance;
            if (mod != null && mod.IsLumenicActiveFor(__instance, out _))
            {
                __result = false;
                return false;
            }
        }
        catch (Exception ex)
        {
            MelonLogger.Error($"[LumenicMenu] ShouldRender patch error: {ex.Message}");
        }
        return true;
    }

    // --- Seam 2: Pointer Input (Desktop, VR Laser, Release outside quad, Index-Finger Poke) ---

    private static bool Prefix_CohtmlViewInputHandler_DoViewInput(
        CohtmlControlledView view,
        Vector2 viewCoords,
        bool isMouseDown,
        bool isMouseUp,
        float scrollValue)
    {
        try
        {
            var mod = LumenicMenuMod.Instance;
            if (mod == null || !mod.IsLumenicActiveFor(view, out uint surfaceId))
                return true;

            bool allowScroll = scrollValue != 0f
                && (MetaPort.Instance == null || !MetaPort.Instance.isUsingVr || IsJoystickScrollEnabled());

            mod.HandlePointerInput(
                surfaceId,
                viewCoords.x,
                viewCoords.y,
                isMouseDown,
                isMouseUp,
                allowScroll ? scrollValue : 0f);
            return false;
        }
        catch (Exception ex)
        {
            MelonLogger.Error($"[LumenicMenu] DoViewInput patch error: {ex.Message}");
            return true;
        }
    }

    private static bool Prefix_CohtmlViewInputHandler_ReleaseViewInput(
        CohtmlControlledView view,
        Vector2 viewCoords)
    {
        try
        {
            var mod = LumenicMenuMod.Instance;
            if (mod == null || !mod.IsLumenicActiveFor(view, out uint surfaceId))
                return true;

            // Send pointer_move + pointer_up + pointer_leave (NOT pointer_cancel) so drags commit cleanly.
            mod.HandlePointerRelease(surfaceId, viewCoords.x, viewCoords.y);
            return false;
        }
        catch (Exception ex)
        {
            MelonLogger.Error($"[LumenicMenu] ReleaseViewInput patch error: {ex.Message}");
            return true;
        }
    }

    private static bool Prefix_PointerReceiver_OnTriggerEnter(CVR_Menu_Pointer_Reciever __instance, Collider other)
    {
        try
        {
            if (__instance == null || __instance.view == null || other == null)
                return true;

            var mod = LumenicMenuMod.Instance;
            if (mod == null || !mod.IsLumenicActiveFor(__instance.view, out uint surfaceId))
                return true;

            if (!__instance.view.AllowInput || !__instance.view.Enabled)
                return false;
            if (surfaceId == NativeBridge.SurfaceQuick
                && CVR_MenuManager.Instance != null
                && CVR_MenuManager.Instance.IsQuickMenuInGrabMode)
                return false;

            CVRPointer pointer = other.GetComponent<CVRPointer>();
            if (pointer != null && pointer.type == "index" && pointer.gameObject.layer == 8)
            {
                Collider targetCollider = __instance.GetComponent<Collider>();
                Collider otherCollider = other.gameObject.GetComponent<Collider>();
                if (targetCollider != null && otherCollider != null)
                {
                    Vector3 closest = otherCollider.ClosestPointOnBounds(__instance.transform.position);
                    Ray ray = new Ray(pointer.transform.position, pointer.transform.position - closest);
                    if (targetCollider.Raycast(ray, out RaycastHit hitInfo, 1000f))
                    {
                        float x = hitInfo.textureCoord.x * __instance.view.Width;
                        float y = (1f - hitInfo.textureCoord.y) * __instance.view.Height;
                        mod.HandleFingerPoke(surfaceId, x, y);
                    }
                }
            }
            return false;
        }
        catch (Exception ex)
        {
            MelonLogger.Error($"[LumenicMenu] OnTriggerEnter poke patch error: {ex.Message}");
            return true;
        }
    }

    // --- Seam 3: Keyboard Routing (Prevent Double-Typing & KeyUp Duplication) ---

    private static bool Prefix_CVRUIManagerBaseInput_OnKeyEvent(
        CVRUIManagerBaseInput __instance,
        KeyEvent eventData,
        InputEventWrapper unityEvent)
    {
        try
        {
            var mod = LumenicMenuMod.Instance;
            if (mod == null || __instance == null || !__instance.IsReady || !__instance.IsViewShown)
                return true;

            if (!mod.IsLumenicActiveFor(__instance.cohtmlView, out uint surfaceId))
                return true;

            if (eventData != null && eventData.Type == KeyEventData.EventType.KeyDown)
            {
                uint mods = VkKeyMapper.PackModifiers(eventData.Modifiers);
                mod.HandleKeyDown(surfaceId, eventData.KeyCode, mods);
            }
            return false;
        }
        catch (Exception ex)
        {
            MelonLogger.Error($"[LumenicMenu] OnKeyEvent patch error: {ex.Message}");
            return true;
        }
    }

    private static bool Prefix_CVRUIManagerBaseInput_OnCharEvent(
        CVRUIManagerBaseInput __instance,
        KeyEvent eventData,
        char character)
    {
        try
        {
            var mod = LumenicMenuMod.Instance;
            if (mod == null || __instance == null || !__instance.IsReady || !__instance.IsViewShown)
                return true;

            if (!mod.IsLumenicActiveFor(__instance.cohtmlView, out uint surfaceId))
                return true;

            // Route printable characters only via OnCharEvent -> Lumenic on_ime_commit.
            if (character >= 0x20 && character != 0x7F)
            {
                mod.HandleCharInput(surfaceId, character);
            }
            return false;
        }
        catch (Exception ex)
        {
            MelonLogger.Error($"[LumenicMenu] OnCharEvent patch error: {ex.Message}");
            return true;
        }
    }

    private static bool Prefix_CohtmlInputHandler_BlockFocusedView()
    {
        try
        {
            var mod = LumenicMenuMod.Instance;
            if (mod != null && CohtmlInputHandler.FocusedView is CohtmlControlledView focusedView)
            {
                if (mod.IsLumenicActiveFor(focusedView, out _))
                    return false;
            }
        }
        catch
        {
            // Never disrupt Cohtml input fallback on error.
        }
        return true;
    }

    // --- UILib Mod Compatibility Hooks ---

    private static bool IsQuickMenuOwnedByLumenic(out LumenicMenuMod mod)
    {
        mod = LumenicMenuMod.Instance;
        if (mod == null || CVR_MenuManager.Instance == null)
            return false;
        return mod.IsLumenicActiveFor(CVR_MenuManager.Instance.cohtmlView, CVR_MenuManager.Instance, NativeBridge.SurfaceQuick);
    }

    private static bool Prefix_UiLib_Page_OpenPage(Page __instance, bool resetBreadcrumbs, bool forceBreadcrumbAdd)
    {
        try
        {
            if (IsQuickMenuOwnedByLumenic(out var mod))
            {
                mod.UiLib.HandleOpenPage(__instance, resetBreadcrumbs, forceBreadcrumbAdd);
                mod.MarkStateDirty();
                return false;
            }
        }
        catch (Exception ex)
        {
            MelonLogger.Error($"[LumenicMenu] UiLib OpenPage hook error: {ex.Message}");
        }
        return true;
    }

    private static bool Prefix_UiLib_GoBack()
    {
        try
        {
            if (IsQuickMenuOwnedByLumenic(out var mod))
            {
                mod.UiLib.HandleGoBack();
                mod.MarkStateDirty();
                return false;
            }
        }
        catch (Exception ex)
        {
            MelonLogger.Error($"[LumenicMenu] UiLib GoBack hook error: {ex.Message}");
        }
        return true;
    }

    private static bool Prefix_UiLib_ShowConfirm(string title, string content, Action onYes, Action onNo, string yesText, string noText)
    {
        try
        {
            if (IsQuickMenuOwnedByLumenic(out var mod))
            {
                mod.UiLib.HandleShowConfirm(title, content, onYes, onNo, yesText, noText);
                mod.MarkStateDirty();
                return false;
            }
        }
        catch (Exception ex)
        {
            MelonLogger.Error($"[LumenicMenu] UiLib ShowConfirm hook error: {ex.Message}");
        }
        return true;
    }

    private static bool Prefix_UiLib_ShowNotice(string title, string content, Action onOK, string okText)
    {
        try
        {
            if (IsQuickMenuOwnedByLumenic(out var mod))
            {
                mod.UiLib.HandleShowNotice(title, content, onOK, okText);
                mod.MarkStateDirty();
                return false;
            }
        }
        catch (Exception ex)
        {
            MelonLogger.Error($"[LumenicMenu] UiLib ShowNotice hook error: {ex.Message}");
        }
        return true;
    }

    private static bool Prefix_UiLib_OpenMultiSelect(MultiSelection multiSelection)
    {
        try
        {
            if (IsQuickMenuOwnedByLumenic(out var mod))
            {
                mod.UiLib.HandleOpenMultiSelect(multiSelection);
                mod.MarkStateDirty();
                return false;
            }
        }
        catch (Exception ex)
        {
            MelonLogger.Error($"[LumenicMenu] UiLib OpenMultiSelect hook error: {ex.Message}");
        }
        return true;
    }

    private static bool Prefix_UiLib_OpenNumberInput(string name, float input, Action<float> onCompleted)
    {
        try
        {
            if (IsQuickMenuOwnedByLumenic(out var mod))
            {
                mod.UiLib.HandleOpenNumberInput(name, input, onCompleted);
                mod.MarkStateDirty();
                return false;
            }
        }
        catch (Exception ex)
        {
            MelonLogger.Error($"[LumenicMenu] UiLib OpenNumberInput hook error: {ex.Message}");
        }
        return true;
    }

    private static bool Prefix_UiLib_ShowAlertToast(string message, int delay)
    {
        try
        {
            if (IsQuickMenuOwnedByLumenic(out var mod))
            {
                mod.UiLib.HandleShowAlertToast(message, delay);
                mod.MarkStateDirty();
                return false;
            }
        }
        catch (Exception ex)
        {
            MelonLogger.Error($"[LumenicMenu] UiLib ShowAlertToast hook error: {ex.Message}");
        }
        return true;
    }

    // --- MiniMe Studio Box Placement & Click-and-Drag Turntable Spin ---

    private static void Postfix_MiniMe_UpdateQuickMenuPosition()
    {
        try
        {
            if (IsQuickMenuOwnedByLumenic(out var mod))
            {
                mod.AvatarStage?.ApplyMiniMePostfix();
            }
        }
        catch (Exception ex)
        {
            MelonLogger.Error($"[LumenicMenu] MiniMe position postfix error: {ex.Message}");
        }
    }

    private static bool Prefix_MenuPreviewRig_UpdateRendering(MenuPreviewRig __instance)
    {
        try
        {
            if (__instance is not AvatarSettingsPreview)
                return true;

            var mod = LumenicMenuMod.Instance;
            if (mod == null || (!Settings.EnableMainMenu && !Settings.EnableQuickMenu))
                return true;

            if (_previewCameraField?.GetValue(__instance) is not MenuPreviewCamera preview)
                return true;

            bool vanillaWants = !Settings.EnableMainMenu
                && _previewIsPageOpenField != null
                && (bool)_previewIsPageOpenField.GetValue(__instance)
                && ViewManager.Instance != null
                && ViewManager.Instance.IsViewShown;

            bool want = mod.ShouldRenderAvatarPreview() || vanillaWants;
            if (want != preview.Rendering)
            {
                preview.Rendering = want;
                _previewIsDraggingField?.SetValue(__instance, false);
                _previewOnRenderingChangedMethod?.Invoke(__instance, new object[] { want });
            }
            return false;
        }
        catch (Exception ex)
        {
            MelonLogger.Error($"[LumenicMenu] MenuPreviewRig.UpdateRendering patch error: {ex.Message}");
            return true;
        }
    }

    // --- External / QM -> Main Menu Navigation Sync ---

    private static string MapVanillaPageToLumenicTab(string category)
    {
        return (category ?? string.Empty).ToLowerInvariant() switch
        {
            "home" => "home",
            "friends" => "friends",
            "avatars" => "avatars",
            "worlds" => "worlds",
            "props" => "props",
            "search" => "search",
            "messages" => "messages",
            "settings" => "settings",
            "currentinstance" => "currentInstance",
            "avatarsettings" => "avatarSettings",
            "modsettings" => "modSettings",
            "discover" => "discover",
            _ => null
        };
    }

    private static bool TryGetNavMod(out LumenicMenuMod mod)
    {
        mod = LumenicMenuMod.Instance;
        return mod != null && Settings.EnableMainMenu && !mod.SuppressVanillaNavHook;
    }

    private static void Postfix_ViewManager_ShowPage(string category)
    {
        try
        {
            if (!TryGetNavMod(out var mod))
                return;
            string tab = MapVanillaPageToLumenicTab(category);
            if (!string.IsNullOrEmpty(tab))
            {
                if (tab == "friends") mod.Social?.CloseUserDetail();
                else if (tab == "worlds" || tab == "avatars" || tab == "props") mod.Content?.CloseDetail();
                mod.NavigateMainMenuTab(tab);
            }
        }
        catch (Exception ex)
        {
            MelonLogger.Warning($"[LumenicMenu] ShowPage nav sync error: {ex.Message}");
        }
    }

    private static void Postfix_ViewManager_RequestUserDetailsPage(string userID)
    {
        try
        {
            if (!TryGetNavMod(out var mod) || string.IsNullOrWhiteSpace(userID))
                return;
            mod.Social?.LoadUserDetail(userID);
            mod.NavigateMainMenuTab("friends");
        }
        catch (Exception ex)
        {
            MelonLogger.Warning($"[LumenicMenu] RequestUserDetailsPage nav sync error: {ex.Message}");
        }
    }

    private static void Postfix_ViewManager_RequestWorldDetailsPage(string worldId)
    {
        try
        {
            if (!TryGetNavMod(out var mod) || string.IsNullOrWhiteSpace(worldId))
                return;
            mod.Content?.LoadDetail("world", worldId);
            mod.NavigateMainMenuTab("worlds");
        }
        catch (Exception ex)
        {
            MelonLogger.Warning($"[LumenicMenu] RequestWorldDetailsPage nav sync error: {ex.Message}");
        }
    }

    private static void Postfix_ViewManager_RequestAvatarDetailsPage(string avatarID)
    {
        try
        {
            if (!TryGetNavMod(out var mod) || string.IsNullOrWhiteSpace(avatarID))
                return;
            mod.Content?.LoadDetail("avatar", avatarID);
            mod.NavigateMainMenuTab("avatars");
        }
        catch (Exception ex)
        {
            MelonLogger.Warning($"[LumenicMenu] RequestAvatarDetailsPage nav sync error: {ex.Message}");
        }
    }

    private static void Postfix_ViewManager_GetPropDetails(string propId)
    {
        try
        {
            if (!TryGetNavMod(out var mod) || string.IsNullOrWhiteSpace(propId))
                return;
            mod.Content?.LoadDetail("prop", propId);
            mod.NavigateMainMenuTab("props");
        }
        catch (Exception ex)
        {
            MelonLogger.Warning($"[LumenicMenu] GetPropDetails nav sync error: {ex.Message}");
        }
    }

    private static void Postfix_ViewManager_OpenInstanceDetails(string instanceId)
    {
        try
        {
            if (!TryGetNavMod(out var mod) || string.IsNullOrWhiteSpace(instanceId))
                return;
            mod.Content?.LoadInstanceDetail(instanceId);
            mod.NavigateMainMenuTab("worlds");
        }
        catch (Exception ex)
        {
            MelonLogger.Warning($"[LumenicMenu] OpenInstanceDetails nav sync error: {ex.Message}");
        }
    }

    private static bool Prefix_ViewManager_ShowGroupModeratedInstanceAlert(
        InstanceDetailsResponse groupInstanceDetails,
        ref Task<bool> __result)
    {
        try
        {
            LumenicMenuMod mod = LumenicMenuMod.Instance;
            if (mod != null
                && mod.Content != null
                && (Settings.EnableMainMenu || Settings.EnableQuickMenu))
            {
                __result = mod.Content.ShowGroupModPrompt(groupInstanceDetails);
                mod.MarkStateDirty();
                return false;
            }
        }
        catch (Exception ex)
        {
            MelonLogger.Warning($"[LumenicMenu] ShowGroupModeratedInstanceAlert hook error: {ex.Message}");
        }
        return true;
    }

    private static bool Prefix_ViewManager_CancelAndCloseGroupModerationAlertModal()
    {
        try
        {
            LumenicMenuMod mod = LumenicMenuMod.Instance;
            if (mod != null
                && mod.Content != null
                && (Settings.EnableMainMenu || Settings.EnableQuickMenu))
            {
                mod.Content.CancelGroupModPrompt();
                mod.MarkStateDirty();
                return false;
            }
        }
        catch (Exception ex)
        {
            MelonLogger.Warning($"[LumenicMenu] CancelGroupModerationAlert hook error: {ex.Message}");
        }
        return true;
    }
}
