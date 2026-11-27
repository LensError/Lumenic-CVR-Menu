using System;
using System.Collections.Generic;
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
using ABI_RC.Systems.XRManagement;
using HarmonyLib;
using LumenicMenu.Native;
using MelonLoader;
using Newtonsoft.Json.Linq;
using UnityEngine;

namespace LumenicMenu.Bridge.Actions;

internal static class CoreActions
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

    public static void Register(Dictionary<string, Action<JObject, LumenicMenuMod>> map)
    {
        map["PlayUiSound"] = (obj, _) =>
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
        };

        map["SwitchToVanilla"] = (obj, mod) => mod.SwitchToVanilla(obj.Value<string>("target") ?? "both");
        map["TogglePerfLogging"] = (_, _) => Settings.SetPerfLogging(!Settings.PerfLogging);
        map["ToggleLinearTexture"] = (_, _) => Settings.SetLinearTextureMode(!Settings.LinearTextureMode);
        map["SetEnableRecentHistory"] = (obj, mod) => { Settings.SetEnableRecentHistory(obj.Value<bool>("enabled")); mod.MarkStateDirty(StateDomain.Core); };
        map["SetQmModTabsIconOnly"] = (obj, mod) => { Settings.SetQmModTabsIconOnly(obj.Value<bool>("enabled")); mod.MarkStateDirty(StateDomain.Core); };
        map["SetQmPresetConfirmMode"] = (obj, mod) => { Settings.SetQmPresetConfirmMode(obj.Value<string>("mode") ?? "sync"); mod.MarkStateDirty(StateDomain.Core); };
        map["SetThemePreset"] = (obj, mod) => { Settings.SetThemePreset(obj.Value<string>("preset") ?? "default"); mod.MarkStateDirty(StateDomain.Core); };
        map["SetPanelOpacity"] = (obj, mod) => { Settings.SetPanelOpacity(obj.Value<float>("opacity")); mod.MarkStateDirty(StateDomain.Core); };
        map["SetCursorGlow"] = (obj, mod) => { Settings.SetCursorGlow(obj.Value<bool>("enabled")); mod.MarkStateDirty(StateDomain.Core); };
        map["SetDesktopScrollMultiplier"] = (obj, mod) => { Settings.SetDesktopScrollMultiplier(obj.Value<float>("value")); mod.MarkStateDirty(StateDomain.Core); };
        map["SetVrScrollMultiplier"] = (obj, mod) => { Settings.SetVrScrollMultiplier(obj.Value<float>("value")); mod.MarkStateDirty(StateDomain.Core); };
        map["SetHudSpread"] = (obj, mod) => { Settings.SetHudSpread(obj.Value<bool>("is_vr"), obj.Value<float>("value")); mod.Hud?.TriggerLayoutPreview(); mod.MarkStateDirty(StateDomain.Core); };
        map["SetHudSpreadY"] = (obj, mod) => { Settings.SetHudSpreadY(obj.Value<bool>("is_vr"), obj.Value<float>("value")); mod.Hud?.TriggerLayoutPreview(); mod.MarkStateDirty(StateDomain.Core); };
        map["SetHudElementScale"] = (obj, mod) => { Settings.SetHudElementScale(obj.Value<bool>("is_vr"), obj.Value<string>("element") ?? string.Empty, obj.Value<float>("value")); mod.Hud?.TriggerLayoutPreview(); mod.MarkStateDirty(StateDomain.Core); };
        map["ResetHudLayout"] = (obj, mod) => { Settings.ResetHudLayout(obj.Value<bool>("is_vr")); mod.Hud?.TriggerLayoutPreview(); mod.MarkStateDirty(StateDomain.Core); };
        map["SetHudSpread"] = (obj, mod) => { Settings.SetHudSpread(obj.Value<bool>("is_vr"), obj.Value<float>("value")); mod.Hud?.TriggerLayoutPreview(); mod.MarkStateDirty(StateDomain.Core); };
        map["SetHudSpreadY"] = (obj, mod) => { Settings.SetHudSpreadY(obj.Value<bool>("is_vr"), obj.Value<float>("value")); mod.Hud?.TriggerLayoutPreview(); mod.MarkStateDirty(StateDomain.Core); };
        map["SetHudElementScale"] = (obj, mod) => { Settings.SetHudElementScale(obj.Value<bool>("is_vr"), obj.Value<string>("element") ?? string.Empty, obj.Value<float>("value")); mod.Hud?.TriggerLayoutPreview(); mod.MarkStateDirty(StateDomain.Core); };
        map["ResetHudLayout"] = (obj, mod) => { Settings.ResetHudLayout(obj.Value<bool>("is_vr")); mod.Hud?.TriggerLayoutPreview(); mod.MarkStateDirty(StateDomain.Core); };
        map["SetFontScale"] = (obj, mod) => { Settings.SetFontScale(obj.Value<float>("scale")); mod.MarkStateDirty(StateDomain.Core); };
        map["ToggleUiClickSounds"] = (_, mod) => { Settings.SetUiClickSounds(!Settings.UiClickSounds); mod.MarkStateDirty(StateDomain.Core); };
        map["SetUiSoundPack"] = (obj, mod) => { Settings.SetUiSoundPack(obj.Value<string>("pack") ?? "Glass"); mod.MarkStateDirty(StateDomain.Core); };
        map["SetMaxRefreshHz"] = (obj, mod) => { Settings.SetMaxRefreshHz(obj.Value<int>("hz")); mod.MarkStateDirty(StateDomain.Core); };
        map["ClearRecentHistory"] = (_, mod) => { mod.RecentHistory?.ClearAll(); mod.MarkStateDirty(StateDomain.Content | StateDomain.Social); };
        map["ClearPinnedItems"] = (_, mod) => { mod.PinnedContent?.ClearAll(); mod.MarkStateDirty(StateDomain.Content); };
        map["ResetTileLayouts"] = (_, mod) =>
        {
            Settings.SetQmQuickActionsOrder(string.Empty);
            Settings.SetMmQuickToolsOrder(string.Empty);
            mod.MarkStateDirty(StateDomain.Core);
        };
        map["SetTileOrder"] = (obj, mod) =>
        {
            string target = obj.Value<string>("target") ?? string.Empty;
            var arr = obj["order"] as JArray;
            var parts = new List<string>();
            if (arr != null)
            {
                foreach (JToken t in arr)
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
            mod.MarkStateDirty(StateDomain.Core);
        };

        map["DummyPing"] = (obj, _) => MelonLogger.Msg(
            $"[LumenicMenu] Rust -> C# DummyPing from {obj.Value<string>("source") ?? "unknown"} (count={obj.Value<int>("count")}).");

        map["ToggleMute"] = (_, mod) => { AudioManagement.ToggleMicrophone(); mod.MarkStateDirty(StateDomain.Core); };
        map["ToggleFlight"] = (_, mod) =>
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
            mod.MarkStateDirty(StateDomain.Core);
        };
        map["ToggleSeatedOrRecalibrate"] = (_, mod) =>
        {
            var body = IKSystem.Instance?.BodySystem;
            bool fbtActive = body != null && body.FullBodyEnabled && (body.IsCalibrated || body.FullBodyAvailable);
            if (fbtActive)
                PlayerSetup.Instance?.ReCalibrateAvatar();
            else
                PlayerSetup.Instance?.SwitchSeatedPlay();
            mod.MarkStateDirty(StateDomain.Core);
        };
        map["ToggleFullBody"] = (_, mod) => { IKSystem.Instance?.BodySystem?.ToggleFullBody(); mod.MarkStateDirty(StateDomain.Core); };
        map["ToggleCamera"] = (_, mod) => { PortableCameraController.Instance?.Toggle(); mod.MarkStateDirty(StateDomain.Core); };
        map["ToggleMirror"] = (_, mod) => { PortableMirrorInstance.TogglePortableMirror(); mod.MarkStateDirty(StateDomain.Core); };
        map["Respawn"] = (_, _) => RootLogic.Instance?.Respawn();
        map["GoHome"] = (_, _) => ViewManager.Instance?.GoHome();
        map["OpenTextChat"] = (_, _) =>
        {
            CVR_MenuManager.Instance?.ToggleQuickMenu(show: false);
            TextComms.OpenKeyboard();
        };

        map["OpenMainMenuPage"] = (obj, mod) =>
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
        };

        map["OpenVanillaPage"] = (obj, mod) => mod.OpenVanillaPageDeferred(obj.Value<string>("page") ?? "home");
        map["RemoveMyProps"] = (_, _) => PropUserActions.RemoveMyProps();
        map["TogglePropDeleteMode"] = (_, _) => PlayerSetup.Instance?.ToggleDeleteMode();
        map["RemoveAllProps"] = (_, _) => PropUserActions.RemoveAllProps();
        map["RemoveAllAvatars"] = (_, _) => CVRPlayerManager.Instance?.ClearAllNetworkPlayerAvatars();
        map["RecalibrateAvatar"] = (_, _) => PlayerSetup.Instance?.ReCalibrateAvatar();
        map["SwitchVrMode"] = (_, _) => XRDeviceManager.Instance?.TogglePreferredVRMode();
        map["ToggleCursorLock"] = (_, _) => CursorLockManager.Instance?.ToggleVRCursorUnlock();
        map["MediaPrev"] = (_, _) => MediaControls.MediaActionPrev();
        map["MediaPlayPause"] = (_, _) => MediaControls.MediaActionPlay();
        map["MediaNext"] = (_, _) => MediaControls.MediaActionFwd();
        map["QuitApplication"] = (_, _) => RootLogic.Instance?.QuitApplication();

        map["OpenHardcodedUrl"] = (obj, _) =>
        {
            string key = obj.Value<string>("key") ?? string.Empty;
            if (!string.IsNullOrWhiteSpace(key))
                CVRWebLinks.OpenHardcodedUrl(key);
        };
        map["OpenExternalUrl"] = (obj, _) =>
        {
            string url = obj.Value<string>("url") ?? string.Empty;
            if (Uri.TryCreate(url, UriKind.Absolute, out Uri uri)
                && uri.Scheme == Uri.UriSchemeHttps
                && !uri.IsFile
                && !uri.IsUnc)
            {
                Application.OpenURL(uri.AbsoluteUri);
            }
        };
        map["CopyText"] = (obj, _) =>
        {
            string text = obj.Value<string>("text") ?? string.Empty;
            if (!string.IsNullOrEmpty(text))
            {
                GUIUtility.systemCopyBuffer = text;
                if (string.Equals(text, "team@chilloutvr.net", StringComparison.OrdinalIgnoreCase))
                    Application.OpenURL("mailto:" + text);
            }
        };

        map["AcceptDisclaimer"] = (_, mod) => { Settings.SetDisclaimerAccepted(true); mod.MarkStateDirty(StateDomain.Core); };
        map["CheckForUpdates"] = (_, mod) => mod.Updates?.CheckForUpdates(manual: true);
        map["RemindUpdateLater"] = (_, mod) => mod.Updates?.DismissForSession();
        map["SkipUpdateVersion"] = (obj, mod) =>
        {
            string version = obj.Value<string>("version") ?? string.Empty;
            if (!string.IsNullOrEmpty(version))
                Settings.SetDismissedUpdateVersion(version);
            mod.Updates?.DismissForSession();
            mod.MarkStateDirty(StateDomain.Core);
        };
        map["DismissWhatsNew"] = (_, mod) => { Settings.SetLastSeenVersion(LumenicMenuMod.ModVersion); mod.MarkStateDirty(StateDomain.Core); };
        map["DismissSystemAlert"] = (obj, mod) => mod.SystemAlerts?.Dismiss(obj.Value<long?>("id") ?? 0);
        map["RespondSystemConfirm"] = (obj, mod) =>
        {
            long id = obj.Value<long?>("id") ?? 0;
            bool confirmed = obj.Value<bool?>("confirmed") ?? false;
            SystemAlertDto alert = mod.SystemAlerts?.Dismiss(id);
            if (alert != null && !string.IsNullOrEmpty(alert.confirm_id) && ViewManager.Instance != null)
                HandleUiConfirmMethod?.Invoke(ViewManager.Instance, new object[] { alert.confirm_id, confirmed ? "true" : "false", string.Empty });
        };

        map["AttemptEatBread"] = (_, _) => { if (ViewManager.Instance != null) AttemptEatBreadMethod?.Invoke(ViewManager.Instance, null); };
        map["OpenLogsFolder"] = (_, _) => { if (ViewManager.Instance != null) OpenLogsFolderMethod?.Invoke(ViewManager.Instance, null); };
        map["OpenLocalPlayerQmPage"] = (_, mod) => { mod.UiLib.OpenLocalPlayerPage(); mod.MarkStateDirty(StateDomain.UiLib); };

        map["OpenPlayerInMainMenu"] = (_, mod) =>
        {
            string uuid = ABI_RC.Systems.UI.UILib.QuickMenuAPI.SelectedPlayerID;
            if (string.IsNullOrEmpty(uuid))
                uuid = MetaPort.Instance?.ownerId ?? ABI_RC.Core.Networking.AuthManager.UserId;
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
        };

        map["SetAudioVolume"] = (obj, mod) =>
        {
            string key = obj.Value<string>("key");
            if (key == "AudioMaster" || key == "AudioPlayerVoice" || key == "AudioVideoPlayer")
            {
                int clamped = Mathf.Clamp(obj.Value<int>("value"), 0, 100);
                MetaPort.Instance?.settings?.SetSetting(key, clamped.ToString());
                mod.MarkStateDirty(StateDomain.Core);
            }
        };

        map["NotificationButtonClick"] = (obj, mod) =>
        {
            string buttonId = obj.Value<string>("button_id");
            if (!string.IsNullOrEmpty(buttonId) && UIMessageManager.Instance != null && OnClickFromCohtmlMethod != null)
            {
                OnClickFromCohtmlMethod.Invoke(UIMessageManager.Instance, new object[] { buttonId });
                mod.MarkStateDirty(StateDomain.Core);
            }
        };

        map["NotificationDismiss"] = (obj, mod) =>
        {
            string messageId = obj.Value<string>("message_id");
            if (!string.IsNullOrEmpty(messageId))
            {
                UIMessageManager.Instance?.ClearMessageByID(messageId);
                mod.MarkStateDirty(StateDomain.Core);
            }
        };

        map["OpenVirtualKeyboard"] = (obj, mod) =>
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
        };
    }
}
