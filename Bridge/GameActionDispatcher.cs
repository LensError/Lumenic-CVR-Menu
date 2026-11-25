using System;
using System.Reflection;
using ABI.CCK.Components;
using ABI_RC.Core;
using ABI_RC.Core.AudioEffects;
using ABI_RC.Core.Base;
using ABI_RC.Core.InteractionSystem;
using ABI_RC.Core.Player;
using ABI_RC.Core.Player.PortableMirror;
using ABI_RC.Core.PropManagement;
using ABI_RC.Core.Savior;
using ABI_RC.Core.UI;
using ABI_RC.Core.UI.UIRework.Managers;
using ABI_RC.Systems.Camera;
using ABI_RC.Systems.IK;
using ABI_RC.Systems.InputManagement;
using ABI_RC.Systems.PlayerColors;
using ABI_RC.Systems.XRManagement;
using HarmonyLib;
using LumenicMenu.Native;
using MelonLoader;
using Newtonsoft.Json.Linq;
using UnityEngine;

namespace LumenicMenu.Bridge;

/// <summary>
/// Drains and executes typed JSON actions emitted by the Rust Lumenic UI on Unity's main thread.
/// </summary>
internal static class GameActionDispatcher
{
    private static readonly MethodInfo OnClickFromCohtmlMethod =
        AccessTools.Method(typeof(UIMessageManager), "OnClickFromCohtml", new[] { typeof(string) });
    private static readonly MethodInfo AttemptEatBreadMethod =
        AccessTools.Method(typeof(ViewManager), "AttemptEatBread");
    private static readonly MethodInfo OpenLogsFolderMethod =
        AccessTools.Method(typeof(ViewManager), "OpenLogsFolder");
    private static readonly MethodInfo HandleUiConfirmMethod =
        AccessTools.Method(typeof(ViewManager), "HandleUiConfirm", new[] { typeof(string), typeof(string), typeof(string) });
    private static float _lastVrHoverHapticTime = -10f;
    private const float VrHoverHapticCooldownSecs = 0.035f;

    public static void DispatchActionsJson(string actionsJson, LumenicMenuMod mod)
    {
        if (string.IsNullOrEmpty(actionsJson) || mod == null)
            return;

        JArray array;
        try
        {
            array = JArray.Parse(actionsJson);
        }
        catch (Exception ex)
        {
            MelonLogger.Error($"[LumenicMenu] Failed to parse Rust actions JSON: {ex.Message}");
            return;
        }

        foreach (JToken token in array)
        {
            if (token is not JObject obj)
                continue;

            string type = obj.Value<string>("type");
            try
            {
                DispatchOne(type, obj, mod);
            }
            catch (Exception ex)
            {
                MelonLogger.Error($"[LumenicMenu] Action '{type}' failed: {ex.Message}");
            }
        }
    }

    private static void DispatchOne(string type, JObject obj, LumenicMenuMod mod)
    {
        switch (type)
        {
            case "PlayUiSound":
            {
                string sound = obj.Value<string>("sound") ?? "Click";
                if (string.Equals(sound, "Hover", StringComparison.OrdinalIgnoreCase)
                    && MetaPort.Instance != null
                    && MetaPort.Instance.isUsingVr)
                {
                    float now = Time.unscaledTime;
                    if ((now - _lastVrHoverHapticTime) >= VrHoverHapticCooldownSecs)
                    {
                        _lastVrHoverHapticTime = now;
                        CVRHand hand = CVR_MenuManager.Instance != null
                            ? CVR_MenuManager.Instance.lastInteractedHand
                            : CVRHand.Right;
                        CVRInputManager.Instance?.Vibrate(0f, 0.04f, 10f, 0.15f, hand);
                    }
                }

                if (Settings.UiClickSounds)
                {
                    if (!UiSoundPlayer.TryPlay(sound)
                        && Enum.TryParse(sound, ignoreCase: true, out AudioClipField clip))
                    {
                        InterfaceAudio.Play(clip);
                    }
                }
                break;
            }

            case "SwitchToVanilla":
                mod.SwitchToVanilla(obj.Value<string>("target") ?? "both");
                break;

            case "TogglePerfLogging":
                Settings.SetPerfLogging(!Settings.PerfLogging);
                break;

            case "ToggleLinearTexture":
                Settings.SetLinearTextureMode(!Settings.LinearTextureMode);
                break;

            case "SetEnableRecentHistory":
                Settings.SetEnableRecentHistory(obj.Value<bool>("enabled"));
                mod.MarkStateDirty();
                break;

            case "SetDesktopScrollMultiplier":
                Settings.SetDesktopScrollMultiplier(obj.Value<float>("value"));
                mod.MarkStateDirty();
                break;

            case "SetVrScrollMultiplier":
                Settings.SetVrScrollMultiplier(obj.Value<float>("value"));
                mod.MarkStateDirty();
                break;

            case "SetFontScale":
                Settings.SetFontScale(obj.Value<float>("scale"));
                mod.MarkStateDirty();
                break;

            case "ClearRecentHistory":
                mod.RecentHistory?.ClearAll();
                mod.MarkStateDirty();
                break;

            case "ClearPinnedItems":
                mod.PinnedContent?.ClearAll();
                mod.MarkStateDirty();
                break;

            case "ResetTileLayouts":
                Settings.SetQmQuickActionsOrder(string.Empty);
                Settings.SetMmQuickToolsOrder(string.Empty);
                mod.MarkStateDirty();
                break;

            case "DummyPing":
                MelonLogger.Msg(
                    $"[LumenicMenu] Rust -> C# DummyPing from {obj.Value<string>("source") ?? "unknown"} (count={obj.Value<int>("count")}).");
                break;

            case "ToggleMute":
                AudioManagement.ToggleMicrophone();
                mod.MarkStateDirty();
                break;

            case "ToggleFlight":
            {
                var cc = PlayerSetup.Instance?.CharacterController;
                bool worldAllows = (CVRWorld.Instance == null || !CVRWorld.Instance.useAdvancedSettings || CVRWorld.Instance.allowFlying)
                    && (cc != null && cc.FlightAllowedInWorld);
                if (cc != null && worldAllows)
                {
                    cc.ToggleFlight();
                }
                else if (Settings.UiClickSounds)
                {
                    InterfaceAudio.Play(AudioClipField.Warning);
                }
                mod.MarkStateDirty();
                break;
            }

            case "ToggleSeatedOrRecalibrate":
            {
                var body = IKSystem.Instance?.BodySystem;
                bool fbtActive = body != null && body.FullBodyEnabled && (body.IsCalibrated || body.FullBodyAvailable);
                if (fbtActive)
                {
                    PlayerSetup.Instance?.ReCalibrateAvatar();
                }
                else
                {
                    PlayerSetup.Instance?.SwitchSeatedPlay();
                }
                mod.MarkStateDirty();
                break;
            }

            case "ToggleFullBody":
                IKSystem.Instance?.BodySystem?.ToggleFullBody();
                mod.MarkStateDirty();
                break;

            case "ToggleCamera":
                PortableCameraController.Instance?.Toggle();
                mod.MarkStateDirty();
                break;

            case "ToggleMirror":
                PortableMirrorInstance.TogglePortableMirror();
                mod.MarkStateDirty();
                break;

            case "Respawn":
                RootLogic.Instance?.Respawn();
                break;

            case "GoHome":
                ViewManager.Instance?.GoHome();
                break;

            case "OpenTextChat":
                CVR_MenuManager.Instance?.ToggleQuickMenu(show: false);
                TextComms.OpenKeyboard();
                break;

            case "OpenMainMenuPage":
            {
                string page = obj.Value<string>("page") ?? "home";
                CVR_MenuManager.Instance?.ToggleQuickMenu(show: false);
                mod.Content?.CloseDetail();
                mod.Content?.CloseInstanceDetail();
                mod.Social?.CloseUserDetail();
                mod.Social?.CloseGroupDetail();
                if (page == "currentInstance")
                {
                    string curInstId = ABI_RC.Core.Networking.IO.Instancing.Instances.CurrentInstanceId ?? string.Empty;
                    if (!string.IsNullOrEmpty(curInstId))
                    {
                        mod.Content?.LoadInstanceDetail(curInstId);
                    }
                    page = "worlds";
                }
                else if (page == "worlds" || page == "avatars" || page == "props")
                {
                    mod.Content?.EnsureCategoriesAndLoad(page);
                }
                mod.NavigateMainMenuTab(page);
                if (ViewManager.Instance != null)
                {
                    mod.SuppressVanillaNavHook = true;
                    try
                    {
                        ViewManager.Instance.UiStateToggle(show: true);
                        ViewManager.Instance.ShowPage(page);
                    }
                    finally
                    {
                        mod.SuppressVanillaNavHook = false;
                    }
                }
                break;
            }

            case "OpenVanillaPage":
            {
                string page = obj.Value<string>("page") ?? "home";
                mod.OpenVanillaPageDeferred(page);
                break;
            }

            case "RemoveMyProps":
                PropUserActions.RemoveMyProps();
                break;

            case "TogglePropDeleteMode":
                PlayerSetup.Instance?.ToggleDeleteMode();
                break;

            case "RemoveAllProps":
                PropUserActions.RemoveAllProps();
                break;

            case "RemoveAllAvatars":
                CVRPlayerManager.Instance?.ClearAllNetworkPlayerAvatars();
                break;

            case "RecalibrateAvatar":
                PlayerSetup.Instance?.ReCalibrateAvatar();
                break;

            case "SwitchVrMode":
                XRDeviceManager.Instance?.TogglePreferredVRMode();
                break;

            case "ToggleCursorLock":
                CursorLockManager.Instance?.ToggleVRCursorUnlock();
                break;

            case "MediaPrev":
                MediaControls.MediaActionPrev();
                break;

            case "MediaPlayPause":
                MediaControls.MediaActionPlay();
                break;

            case "MediaNext":
                MediaControls.MediaActionFwd();
                break;

            case "QuitApplication":
                RootLogic.Instance?.QuitApplication();
                break;

            case "OpenHardcodedUrl":
            {
                string key = obj.Value<string>("key") ?? string.Empty;
                if (!string.IsNullOrWhiteSpace(key))
                    ABI_RC.Core.CVRWebLinks.OpenHardcodedUrl(key);
                break;
            }

            case "OpenExternalUrl":
            {
                string url = obj.Value<string>("url") ?? string.Empty;
                if (url.StartsWith("https://", StringComparison.OrdinalIgnoreCase))
                    Application.OpenURL(url);
                break;
            }

            case "CopyText":
            {
                string text = obj.Value<string>("text") ?? string.Empty;
                if (!string.IsNullOrEmpty(text) && text.Contains("@"))
                    Application.OpenURL("mailto:" + text);
                break;
            }

            case "AcceptDisclaimer":
                Settings.SetDisclaimerAccepted(true);
                mod.MarkStateDirty();
                break;

            case "CheckForUpdates":
                mod.Updates?.CheckForUpdates(manual: true);
                break;

            case "RemindUpdateLater":
                mod.Updates?.DismissForSession();
                break;

            case "SkipUpdateVersion":
            {
                string version = obj.Value<string>("version") ?? string.Empty;
                if (!string.IsNullOrEmpty(version))
                    Settings.SetDismissedUpdateVersion(version);
                mod.Updates?.DismissForSession();
                mod.MarkStateDirty();
                break;
            }

            case "DismissWhatsNew":
                Settings.SetLastSeenVersion(LumenicMenuMod.ModVersion);
                mod.MarkStateDirty();
                break;

            case "DismissSystemAlert":
            {
                long id = obj.Value<long?>("id") ?? 0;
                mod.SystemAlerts?.Dismiss(id);
                break;
            }

            case "RespondSystemConfirm":
            {
                long id = obj.Value<long?>("id") ?? 0;
                bool confirmed = obj.Value<bool?>("confirmed") ?? false;
                SystemAlertDto alert = mod.SystemAlerts?.Dismiss(id);
                if (alert != null && !string.IsNullOrEmpty(alert.confirm_id) && ViewManager.Instance != null)
                    HandleUiConfirmMethod?.Invoke(ViewManager.Instance, new object[] { alert.confirm_id, confirmed ? "true" : "false", string.Empty });
                break;
            }

            case "AttemptEatBread":
                if (ViewManager.Instance != null)
                    AttemptEatBreadMethod?.Invoke(ViewManager.Instance, null);
                break;

            case "OpenLogsFolder":
                if (ViewManager.Instance != null)
                    OpenLogsFolderMethod?.Invoke(ViewManager.Instance, null);
                break;

            case "OpenLocalPlayerQmPage":
                mod.UiLib.OpenLocalPlayerPage();
                mod.MarkStateDirty();
                break;

            case "OpenPlayerInMainMenu":
            {
                string uuid = ABI_RC.Systems.UI.UILib.QuickMenuAPI.SelectedPlayerID;
                if (string.IsNullOrEmpty(uuid))
                {
                    uuid = MetaPort.Instance?.ownerId ?? ABI_RC.Core.Networking.AuthManager.UserId;
                }
                CVR_MenuManager.Instance?.ToggleQuickMenu(show: false);
                if (!string.IsNullOrEmpty(uuid))
                {
                    mod.Social?.LoadUserDetail(uuid);
                    mod.NavigateMainMenuTab("friends");
                }
                else
                {
                    mod.NavigateMainMenuTab("home");
                }
                if (ViewManager.Instance != null)
                {
                    mod.SuppressVanillaNavHook = true;
                    try
                    {
                        ViewManager.Instance.UiStateToggle(show: true);
                        if (!string.IsNullOrEmpty(uuid))
                            ViewManager.Instance.RequestUserDetailsPage(uuid);
                        else
                            ViewManager.Instance.ShowPage("home");
                    }
                    finally
                    {
                        mod.SuppressVanillaNavHook = false;
                    }
                }
                break;
            }

            case "SetAudioVolume":
            {
                string key = obj.Value<string>("key");
                if (key == "AudioMaster" || key == "AudioPlayerVoice" || key == "AudioVideoPlayer")
                {
                    int clamped = Mathf.Clamp(obj.Value<int>("value"), 0, 100);
                    MetaPort.Instance?.settings?.SetSetting(key, clamped.ToString());
                    mod.MarkStateDirty();
                }
                break;
            }

            case "NotificationButtonClick":
            {
                string buttonId = obj.Value<string>("button_id");
                if (!string.IsNullOrEmpty(buttonId) && UIMessageManager.Instance != null && OnClickFromCohtmlMethod != null)
                {
                    OnClickFromCohtmlMethod.Invoke(UIMessageManager.Instance, new object[] { buttonId });
                    mod.MarkStateDirty();
                }
                break;
            }

            case "NotificationDismiss":
            {
                string messageId = obj.Value<string>("message_id");
                if (!string.IsNullOrEmpty(messageId))
                {
                    UIMessageManager.Instance?.ClearMessageByID(messageId);
                    mod.MarkStateDirty();
                }
                break;
            }

            case "OpenVirtualKeyboard":
            {
                string fieldId = obj.Value<string>("field_id") ?? string.Empty;
                string initialText = obj.Value<string>("initial_text") ?? string.Empty;
                string title = obj.Value<string>("title") ?? "Input";
                string placeholder = obj.Value<string>("placeholder") ?? string.Empty;
                int maxLen = Math.Max(0, obj.Value<int>("max_len"));
                bool hidden = obj.Value<bool>("hidden");
                bool multiLine = obj.Value<bool>("multi_line") || fieldId == "mm_prof_bio";

                if (KeyboardManager.Instance != null && KeyboardManager.Instance.IsReady && !string.IsNullOrEmpty(fieldId))
                {
                    JToken surfToken = obj["surface_id"];
                    uint surfaceId = surfToken != null && surfToken.Type != JTokenType.Null
                        ? surfToken.Value<uint>()
                        : ((CVR_MenuManager.Instance != null && CVR_MenuManager.Instance.IsViewShown)
                            ? NativeBridge.SurfaceQuick
                            : NativeBridge.SurfaceMain);
                    var source = surfaceId == NativeBridge.SurfaceQuick
                        ? KeyboardManager.OpenSource.QuickMenu
                        : KeyboardManager.OpenSource.MainMenu;

                    mod.ClearSurfaceFocus(surfaceId);
                    KeyboardManager.Instance.ShowKeyboard(
                        initialText,
                        submitted => mod.CommitVirtualKeyboard(surfaceId, fieldId, submitted ?? string.Empty),
                        placeholder,
                        "Confirm",
                        maxLen,
                        hidden,
                        multiLine: multiLine,
                        title,
                        source);
                }
                break;
            }

            case "UiLibSelectTab":
                Api.LumenicApi.Host.ClearActiveTab(Api.ExtSurface.QuickMenu);
                mod.UiLib.SelectTab(obj.Value<string>("tab_id"));
                mod.MarkStateDirty();
                break;

            case "ExtSelectTab":
            {
                string surf = obj.Value<string>("surface") ?? "quick";
                string tabId = obj.Value<string>("tab_id") ?? string.Empty;
                if (surf == "main")
                {
                    Api.LumenicApi.Host.SelectTab(Api.ExtSurface.MainMenu, tabId);
                }
                else
                {
                    Api.LumenicApi.Host.SelectTab(Api.ExtSurface.QuickMenu, tabId);
                }
                break;
            }

            case "ExtGoBack":
            {
                string surf = obj.Value<string>("surface") ?? "quick";
                Api.LumenicApi.Host.PopSubPage(surf == "main" ? Api.ExtSurface.MainMenu : Api.ExtSurface.QuickMenu);
                break;
            }

            case "ExtCloseModal":
            {
                string surf = obj.Value<string>("surface") ?? "quick";
                Api.LumenicApi.Host.CloseModal(surf == "main" ? Api.ExtSurface.MainMenu : Api.ExtSurface.QuickMenu);
                break;
            }

            case "ExtEvent":
                Api.LumenicApi.Host.DispatchEvent(
                    obj.Value<string>("ext_id") ?? string.Empty,
                    obj.Value<string>("target_id") ?? string.Empty,
                    obj.Value<string>("node_id") ?? string.Empty,
                    obj.Value<string>("kind") ?? string.Empty,
                    obj.Value<string>("string_val") ?? string.Empty,
                    obj.Value<float>("float_val"),
                    obj.Value<bool>("bool_val"));
                break;

            case "UiLibGoBack":
                mod.UiLib.HandleGoBack();
                mod.MarkStateDirty();
                break;

            case "UiLibToggleCategory":
                mod.UiLib.ToggleCategory(obj.Value<string>("category_id"));
                mod.MarkStateDirty();
                break;

            case "UiLibClick":
                mod.UiLib.ClickElement(obj.Value<string>("element_id"));
                mod.MarkStateDirty();
                break;

            case "UiLibHold":
                mod.UiLib.HoldElement(obj.Value<string>("element_id"));
                mod.MarkStateDirty();
                break;

            case "UiLibToggle":
                mod.UiLib.ToggleElement(obj.Value<string>("element_id"), obj.Value<bool>("value"));
                mod.MarkStateDirty();
                break;

            case "UiLibSlider":
                mod.UiLib.SetSlider(
                    obj.Value<string>("element_id"),
                    obj.Value<float>("value"),
                    obj.Value<bool>("reset"));
                mod.MarkStateDirty();
                break;

            case "UiLibTextInput":
                mod.UiLib.OpenTextInput(obj.Value<string>("element_id"));
                mod.MarkStateDirty();
                break;

            case "UiLibModalConfirm":
                mod.UiLib.ModalConfirm(obj.Value<bool>("accepted"));
                mod.MarkStateDirty();
                break;

            case "UiLibModalNoticeOk":
                mod.UiLib.ModalNoticeOk();
                mod.MarkStateDirty();
                break;

            case "UiLibModalSelectOption":
                mod.UiLib.ModalSelectOption(obj.Value<int>("index"));
                mod.MarkStateDirty();
                break;

            case "UiLibModalNumberSubmit":
                mod.UiLib.ModalNumberSubmit(obj.Value<float>("value"));
                mod.MarkStateDirty();
                break;

            case "UiLibModalDismiss":
                mod.UiLib.ModalDismiss();
                mod.MarkStateDirty();
                break;

            case "RefreshUixSettings":
                mod.Uix?.EnsureProbed();
                mod.MarkStateDirty();
                break;

            case "UixSetBool":
                mod.Uix?.SetBool(
                    obj.Value<string>("category"),
                    obj.Value<string>("entry"),
                    obj.Value<bool>("value"));
                mod.MarkStateDirty();
                break;

            case "UixSetNumber":
                mod.Uix?.SetNumber(
                    obj.Value<string>("category"),
                    obj.Value<string>("entry"),
                    obj.Value<double>("value"),
                    obj.Value<bool>("save"));
                mod.MarkStateDirty();
                break;

            case "UixSetString":
                mod.Uix?.SetString(
                    obj.Value<string>("category"),
                    obj.Value<string>("entry"),
                    obj.Value<string>("value"));
                mod.MarkStateDirty();
                break;

            case "UixSetEnum":
                mod.Uix?.SetEnum(
                    obj.Value<string>("category"),
                    obj.Value<string>("entry"),
                    obj.Value<int>("index"));
                mod.MarkStateDirty();
                break;

            case "UixCustomButton":
                mod.Uix?.ClickCustomButton(
                    obj.Value<string>("category"),
                    obj.Value<int>("global_id"));
                mod.MarkStateDirty();
                break;

            case "UixCustomToggle":
                mod.Uix?.ToggleCustomToggle(
                    obj.Value<string>("category"),
                    obj.Value<int>("global_id"),
                    obj.Value<bool>("value"));
                mod.MarkStateDirty();
                break;

            case "ContentLoadList":
                mod.Content?.LoadList(
                    obj.Value<string>("kind") ?? "worlds",
                    obj.Value<string>("category_key") ?? string.Empty,
                    obj.Value<bool>("is_system"),
                    obj.Value<int>("page"),
                    obj.Value<int>("sort_order"),
                    obj.Value<bool>("ascending"));
                break;

            case "ContentLoadMore":
                mod.Content?.LoadMore(obj.Value<string>("kind") ?? "worlds");
                break;

            case "ContentLoadDetail":
                mod.Content?.LoadDetail(
                    obj.Value<string>("kind") ?? "world",
                    obj.Value<string>("id") ?? string.Empty);
                break;

            case "ContentCloseDetail":
                mod.Content?.CloseDetail();
                break;

            case "ContentGlobalSearch":
                mod.Content?.RunGlobalSearch(obj.Value<string>("query") ?? string.Empty);
                break;

            case "ContentWearAvatar":
                mod.Content?.WearAvatar(obj.Value<string>("avatar_id") ?? string.Empty);
                break;

            case "ContentSpawnProp":
                mod.Content?.SpawnProp(obj.Value<string>("prop_id") ?? string.Empty);
                break;

            case "ContentJoinInstance":
                mod.Content?.JoinInstance(obj.Value<string>("instance_id") ?? string.Empty);
                break;

            case "ContentDropPortal":
                mod.Content?.DropPortal(obj.Value<string>("instance_id") ?? string.Empty);
                break;

            case "ContentDropCurrentInstancePortal":
                mod.Content?.DropPortal(ABI_RC.Core.Networking.IO.Instancing.Instances.CurrentInstanceId ?? string.Empty);
                break;

            case "ContentLoadInstanceDetail":
            {
                string instId = obj.Value<string>("instance_id") ?? string.Empty;
                if (instId == "__current__")
                    instId = ABI_RC.Core.Networking.IO.Instancing.Instances.CurrentInstanceId ?? string.Empty;
                mod.Content?.LoadInstanceDetail(instId);
                break;
            }

            case "ContentCloseInstanceDetail":
                mod.Content?.CloseInstanceDetail();
                break;

            case "ContentToggleCategory":
                mod.Content?.ToggleContentCategory(
                    obj.Value<string>("kind") ?? string.Empty,
                    obj.Value<string>("id") ?? string.Empty,
                    obj.Value<string>("category_key") ?? string.Empty);
                break;

            case "ContentCreateInstance":
                mod.Content?.CreateInstance(
                    obj.Value<string>("world_id") ?? string.Empty,
                    obj.Value<string>("privacy") ?? "Friends",
                    obj.Value<string>("region") ?? "Eu",
                    obj.Value<string>("group_id") ?? string.Empty);
                break;

            case "ContentResolveGroupModPrompt":
                mod.Content?.ResolveGroupModPrompt(
                    obj.Value<bool?>("accept") ?? false,
                    obj.Value<bool?>("never_show") ?? false);
                break;

            case "ContentSetHomeWorld":
                mod.Content?.SetHomeWorld(obj.Value<string>("world_id") ?? string.Empty);
                break;

            case "ContentTogglePin":
                mod.Content?.TogglePin(
                    obj.Value<string>("kind") ?? string.Empty,
                    obj.Value<string>("id") ?? string.Empty);
                break;

            case "ContentUnpin":
                mod.Content?.Unpin(
                    obj.Value<string>("kind") ?? string.Empty,
                    obj.Value<string>("id") ?? string.Empty);
                break;

            case "QmOpenWorldDetails":
            {
                string worldId = obj.Value<string>("world_id") ?? string.Empty;
                CVR_MenuManager.Instance?.ToggleQuickMenu(show: false);
                if (!string.IsNullOrEmpty(worldId))
                {
                    mod.Content?.LoadDetail("world", worldId);
                    mod.NavigateMainMenuTab("worlds");
                    if (ViewManager.Instance != null)
                    {
                        mod.SuppressVanillaNavHook = true;
                        try
                        {
                            ViewManager.Instance.UiStateToggle(show: true);
                        }
                        finally
                        {
                            mod.SuppressVanillaNavHook = false;
                        }
                    }
                }
                break;
            }

            case "SocialRefreshFriends":
                mod.Social?.RefreshFriends();
                break;

            case "SocialLoadUserDetail":
                mod.Social?.LoadUserDetail(obj.Value<string>("user_id") ?? string.Empty);
                break;

            case "SocialSaveUserNote":
                mod.Social?.SaveUserNote(
                    obj.Value<string>("user_id") ?? string.Empty,
                    obj.Value<string>("note") ?? string.Empty);
                break;

            case "SocialUpdateOwnProfile":
                mod.Social?.UpdateOwnProfile(
                    obj.Value<string>("pronouns") ?? string.Empty,
                    obj.Value<string>("bio") ?? string.Empty,
                    obj.Value<bool>("update_pronouns"),
                    obj.Value<bool>("update_bio"),
                    uid => mod.UiLib?.InvalidateBioCache(uid));
                break;

            case "SetColorizeMenu":
            {
                bool enabled = obj.Value<bool>("enabled");
                CVRSettings s = MetaPort.Instance?.settings;
                if (s != null)
                {
                    s.SetSettingsBool(PlayerColorsManager.ColourizeMenuSettingKey, enabled);
                    s.settingsHaveChanged = true;
                }
                SettingsBridge.InvalidateCache();
                mod.MarkStateDirty();
                break;
            }

            case "SetPlayerColorPreset":
            {
                int presetId = obj.Value<int>("preset_id");
                var preset = (PlayerColorPresets)presetId;
                if (PlayerColorsUtility.ColorPresetToColor.TryGetValue(preset, out var pair))
                {
                    Color c1 = preset == PlayerColorPresets.ChilloutVR ? pair.Item2 : pair.Item1;
                    Color c2 = preset == PlayerColorPresets.ChilloutVR ? pair.Item1 : pair.Item2;
                    PlayerColors cur = PlayerColorsManager.CurrentColors;
                    PlayerColorsManager.ChangePlayerColor(new PlayerColors(c1, c2, cur.UsePrimaryAsEmission));
                    mod.MarkStateDirty();
                }
                break;
            }

            case "SetPlayerColorPrimary":
            {
                int swatchId = obj.Value<int>("swatch_id");
                var swatch = (PlayerColorSwatches)swatchId;
                if (PlayerColorsUtility.ColorSwatchToColor.TryGetValue(swatch, out Color c))
                {
                    PlayerColors cur = PlayerColorsManager.CurrentColors;
                    PlayerColorsManager.ChangePlayerColor(new PlayerColors(c, cur.SecondaryColor, cur.UsePrimaryAsEmission));
                    mod.MarkStateDirty();
                }
                break;
            }

            case "SetPlayerColorSecondary":
            {
                int swatchId = obj.Value<int>("swatch_id");
                var swatch = (PlayerColorSwatches)swatchId;
                if (PlayerColorsUtility.ColorSwatchToColor.TryGetValue(swatch, out Color c))
                {
                    PlayerColors cur = PlayerColorsManager.CurrentColors;
                    PlayerColorsManager.ChangePlayerColor(new PlayerColors(cur.PrimaryColor, c, cur.UsePrimaryAsEmission));
                    mod.MarkStateDirty();
                }
                break;
            }

            case "SetPlayerColorEmission":
            {
                bool enabled = obj.Value<bool>("enabled");
                PlayerColors cur = PlayerColorsManager.CurrentColors;
                PlayerColorsManager.ChangePlayerColor(new PlayerColors(cur.PrimaryColor, cur.SecondaryColor, enabled));
                mod.MarkStateDirty();
                break;
            }

            case "SocialCloseUserDetail":
                mod.Social?.CloseUserDetail();
                break;

            case "SocialInvitePlayer":
                mod.Social?.InvitePlayer(obj.Value<string>("user_id") ?? string.Empty);
                break;

            case "SocialRequestInvite":
                mod.Social?.RequestInvite(obj.Value<string>("user_id") ?? string.Empty);
                break;

            case "SocialSetPlayerMute":
                mod.Social?.SetPlayerMute(obj.Value<string>("user_id") ?? string.Empty, obj.Value<bool>("muted"));
                break;

            case "SocialSetPlayerVolume":
                mod.Social?.SetPlayerVoiceVolume(obj.Value<string>("user_id") ?? string.Empty, obj.Value<int>("volume"));
                break;

            case "SocialSetAvatarVisibility":
                mod.Social?.SetPlayerAvatarVisibility(obj.Value<string>("user_id") ?? string.Empty, obj.Value<int>("mode"));
                break;

            case "SocialSetPropVisibility":
                mod.Social?.SetPlayerPropVisibility(obj.Value<string>("user_id") ?? string.Empty, obj.Value<int>("mode"));
                break;

            case "SocialReloadPlayerAvatar":
                mod.Social?.ReloadPlayerAvatar(obj.Value<string>("user_id") ?? string.Empty);
                break;

            case "SocialKickUser":
                mod.Social?.KickUser(obj.Value<string>("user_id") ?? string.Empty);
                break;

            case "SocialSendFriendRequest":
                mod.Social?.SendFriendRequest(obj.Value<string>("user_id") ?? string.Empty);
                break;

            case "SocialUnfriend":
                mod.Social?.Unfriend(obj.Value<string>("user_id") ?? string.Empty);
                break;

            case "SocialSetBlocked":
                mod.Social?.SetBlocked(
                    obj.Value<string>("user_id") ?? string.Empty,
                    obj.Value<bool>("blocked"));
                break;

            case "SocialSetFeaturedBadge":
                mod.Social?.SetFeaturedBadge(
                    obj.Value<string>("badge_id") ?? string.Empty,
                    obj.Value<bool>("featured"));
                break;

            case "SocialSetFeaturedGroup":
                mod.Social?.SetFeaturedGroup(
                    obj.Value<string>("group_id") ?? string.Empty);
                break;

            case "SocialLoadGroupDetail":
                mod.Social?.LoadGroupDetail(obj.Value<string>("group_id") ?? string.Empty);
                break;

            case "SocialCloseGroupDetail":
                mod.Social?.CloseGroupDetail();
                break;

            case "SocialUpdateGroupName":
                mod.Social?.UpdateGroupName(
                    obj.Value<string>("group_id") ?? string.Empty,
                    obj.Value<string>("name") ?? string.Empty);
                break;

            case "SocialUpdateGroupDesc":
                mod.Social?.UpdateGroupDescription(
                    obj.Value<string>("group_id") ?? string.Empty,
                    obj.Value<string>("description") ?? string.Empty);
                break;

            case "SocialKickGroupMember":
                mod.Social?.KickGroupMember(
                    obj.Value<string>("group_id") ?? string.Empty,
                    obj.Value<string>("target_id") ?? string.Empty);
                break;

            case "SocialLeaveGroup":
                mod.Social?.LeaveGroup(obj.Value<string>("group_id") ?? string.Empty);
                break;

            case "SocialDeleteGroup":
                mod.Social?.DeleteGroup(obj.Value<string>("group_id") ?? string.Empty);
                break;

            case "SocialLoadMoreGroupMembers":
                mod.Social?.LoadMoreGroupMembers(obj.Value<string>("group_id") ?? string.Empty);
                break;

            case "SocialSetGroupMemberRole":
                mod.Social?.SetGroupMemberRole(
                    obj.Value<string>("group_id") ?? string.Empty,
                    obj.Value<string>("user_id") ?? string.Empty,
                    obj.Value<string>("role") ?? string.Empty);
                break;

            case "SocialMassSetGroupMembersToMember":
            {
                string groupId = obj.Value<string>("group_id") ?? string.Empty;
                var userIds = new System.Collections.Generic.List<string>();
                if (obj["user_ids"] is JArray arr)
                {
                    foreach (JToken tok in arr)
                    {
                        string uid = tok?.Value<string>();
                        if (!string.IsNullOrWhiteSpace(uid))
                            userIds.Add(uid);
                    }
                }
                mod.Social?.MassSetGroupMembersToMember(groupId, userIds);
                break;
            }

            case "SocialCancelMassGroupRole":
                mod.Social?.CancelMassGroupRole();
                break;

            case "SocialUpdateGroupSettings":
                mod.Social?.UpdateGroupSettings(
                    obj.Value<string>("group_id") ?? string.Empty,
                    obj.Value<bool>("listed"),
                    obj.Value<int>("member_publicity"),
                    obj.Value<int>("event_publicity"),
                    obj.Value<int>("privacy_join"));
                break;

            case "ContentLoadDiscover":
                mod.Content?.LoadDiscover();
                break;

            case "CvrSetBool":
                SettingsBridge.SetBool(
                    obj.Value<string>("key"),
                    obj.Value<bool>("value"));
                mod.MarkStateDirty();
                break;

            case "CvrSetInt":
                SettingsBridge.SetInt(
                    obj.Value<string>("key"),
                    obj.Value<int>("value"));
                mod.MarkStateDirty();
                break;

            case "CvrSetFloat":
                SettingsBridge.SetFloat(
                    obj.Value<string>("key"),
                    obj.Value<float>("value"));
                mod.MarkStateDirty();
                break;

            case "CvrSetString":
                SettingsBridge.SetString(
                    obj.Value<string>("key"),
                    obj.Value<string>("value"));
                mod.MarkStateDirty();
                break;

            case "ToggleUiClickSounds":
                Settings.SetUiClickSounds(!Settings.UiClickSounds);
                mod.MarkStateDirty();
                break;

            case "SetUiSoundPack":
                Settings.SetUiSoundPack(obj.Value<string>("pack") ?? "Glass");
                mod.MarkStateDirty();
                break;

            case "SetMaxRefreshHz":
                Settings.SetMaxRefreshHz(obj.Value<int>("hz"));
                mod.MarkStateDirty();
                break;

            case "SetTileOrder":
            {
                string target = obj.Value<string>("target") ?? string.Empty;
                var arr = obj["order"] as Newtonsoft.Json.Linq.JArray;
                var parts = new System.Collections.Generic.List<string>();
                if (arr != null)
                {
                    foreach (var t in arr)
                    {
                        string s = t?.ToString();
                        if (!string.IsNullOrWhiteSpace(s))
                            parts.Add(s.Trim());
                    }
                }
                string csv = string.Join(",", parts);
                if (target == "qm_quick_actions")
                    Settings.SetQmQuickActionsOrder(csv);
                else if (target == "mm_quick_tools")
                    Settings.SetMmQuickToolsOrder(csv);
                mod.MarkStateDirty();
                break;
            }

            case "AvatarSetParam":
                AvatarSettingsBridge.SetParam(obj.Value<string>("param") ?? string.Empty, obj.Value<float>("value"));
                mod.MarkStateDirty();
                break;

            case "AvatarSetAutoApply":
                AvatarSettingsBridge.SetAutoApply(obj.Value<bool>("enabled"));
                mod.MarkStateDirty();
                break;

            case "AvatarApplyStaged":
                AvatarSettingsBridge.ApplyStaged();
                mod.MarkStateDirty();
                break;

            case "AvatarDiscardStaged":
                AvatarSettingsBridge.DiscardStaged();
                mod.MarkStateDirty();
                break;

            case "AvatarLoadProfile":
                AvatarSettingsBridge.LoadProfile(obj.Value<string>("profile") ?? string.Empty);
                mod.MarkStateDirty();
                break;

            case "AvatarSaveProfile":
                AvatarSettingsBridge.SaveProfile(obj.Value<string>("profile") ?? string.Empty);
                mod.MarkStateDirty();
                break;

            case "AvatarDeleteProfile":
                AvatarSettingsBridge.DeleteProfile(obj.Value<string>("profile") ?? string.Empty);
                mod.MarkStateDirty();
                break;

            case "AvatarReorderProfiles":
            {
                string[] ordered = obj["profiles"]?.ToObject<string[]>();
                AvatarSettingsBridge.ReorderProfiles(ordered);
                mod.MarkStateDirty();
                break;
            }

            case "AvatarResetDefaults":
                AvatarSettingsBridge.ResetDefaults();
                mod.MarkStateDirty();
                break;

            case "AvatarSetHeight":
                AvatarSettingsBridge.SetAvatarHeight(obj.Value<float>("height"));
                mod.MarkStateDirty();
                break;

            case "AvatarResetHeight":
                AvatarSettingsBridge.ResetAvatarHeight();
                mod.MarkStateDirty();
                break;

            case "AvatarPreviewRotate":
                mod.RotateActiveAvatarPreview(obj.Value<float>("delta"), obj.Value<bool>("is_dragging"));
                break;

            case "WorldPanoLook":
                AvatarSettingsBridge.LookWorldPano(
                    obj.Value<float>("delta_x"),
                    obj.Value<float>("delta_y"),
                    obj.Value<bool>("is_dragging"));
                break;

            case "SyncAvatarPreview":
                mod.SyncAvatarPreview(
                    obj.Value<uint>("surface_id"),
                    obj.Value<bool>("active"),
                    obj.Value<string>("target") ?? "local",
                    obj.Value<float>("x"),
                    obj.Value<float>("y"),
                    obj.Value<float>("w"),
                    obj.Value<float>("h"));
                break;

            case "SyncStudioStage":
                break;

            default:
                MelonLogger.Warning($"[LumenicMenu] Unhandled UI action type: '{type}'.");
                break;
        }
    }
}
