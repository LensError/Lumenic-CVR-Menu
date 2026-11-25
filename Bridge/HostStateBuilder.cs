using System;
using System.Collections.Generic;
using System.Reflection;
using ABI.CCK.Components;
using ABI_RC.Core.InteractionSystem;
using ABI_RC.Core.Networking;
using ABI_RC.Core.Networking.Crier;
using ABI_RC.Core.Networking.GameServer;
using ABI_RC.Core.Networking.Garrison;
using ABI_RC.Core.Networking.IO.Instancing;
using ABI_RC.Core.Player;
using ABI_RC.Core.Player.PortableMirror;
using ABI_RC.Core.Savior;
using ABI_RC.Core.UI;
using ABI_RC.Core.UI.UIMessage;
using ABI_RC.Systems.Camera;
using ABI_RC.Systems.Communications;
using ABI_RC.Systems.IK;
using ABI_RC.Systems.PlayerColors;
using HarmonyLib;
using LumenicMenu.Native;
using Newtonsoft.Json;
using UnityEngine;

namespace LumenicMenu.Bridge;

/// <summary>
/// Builds and caches the JSON host state snapshot pushed to the Rust UI layer.
/// Preserves last-known Main Menu / Quick Menu domain snapshots so toggling one menu
/// never clobbers or forces unnecessary rebuilds for the other.
/// </summary>
internal sealed class HostStateBuilder
{
    private static readonly FieldInfo BigMenuToggleMirrorField =
        AccessTools.Field(typeof(PortableMirrorInstance), "_bigMenuToggleMirror");

    private object _cachedUiLibSnapshot;
    private object _cachedUixCategories = Array.Empty<object>();
    private List<object> _cachedOnlineFriends = new List<object>();
    private List<object> _cachedActiveWorlds = new List<object>();
    private List<object> _cachedActivityFeed = new List<object>();
    private object _cachedCvrSettings;
    private object _cachedAvatarSettings;
    private object _cachedContentSnapshot;
    private object _cachedSocialSnapshot;

    public void ReadTelemetry(
        float smoothedFps,
        out int fps,
        out int gamePing,
        out int voicePing,
        out string clockText,
        out string dateText)
    {
        var settings = MetaPort.Instance?.settings;
        string clockFmt = settings != null ? settings.GetSettingsString("GeneralClockFormat", "24") : "24";
        DateTime now = DateTime.Now;
        clockText = clockFmt == "12" ? now.ToString("hh:mm tt") : now.ToString("HH:mm");
        dateText = now.ToString("ddd, MMM d");
        fps = Mathf.RoundToInt(smoothedFps);
        gamePing = GetGamePing();
        voicePing = GetVoicePing();
    }

    public string BuildJson(
        LumenicMenuMod mod,
        ImageLoaderService images,
        MenuSurfaceHost mainSurface,
        MenuSurfaceHost quickSurface,
        NativeBridge bridge,
        uint mmW,
        uint mmH,
        uint qmW,
        uint qmH)
    {
        var settings = MetaPort.Instance?.settings;

        var cc = PlayerSetup.Instance?.CharacterController;
        bool flightAllowed = (CVRWorld.Instance == null || !CVRWorld.Instance.useAdvancedSettings || CVRWorld.Instance.allowFlying)
            && (cc != null && cc.FlightAllowedInWorld);
        bool isFlying = cc != null && cc.IsFlying();
        bool isSeated = OffsetController.Instance != null && OffsetController.Instance.IsSeatedPlay();

        var body = IKSystem.Instance?.BodySystem;
        bool fbtAvail = body != null && (body.IsCalibrated || body.FullBodyAvailable);
        bool fbtEnabled = body != null && body.FullBodyEnabled;

        int audioMaster = settings != null ? settings.GetSettingsInt("AudioMaster", 100) : 100;
        int audioVoice = settings != null ? settings.GetSettingsInt("AudioPlayerVoice", 100) : 100;
        int audioVideo = settings != null ? settings.GetSettingsInt("AudioVideoPlayer", 100) : 100;

        Color32 p1 = PlayerColorsManager.CurrentColors.PrimaryColor;
        Color32 p2 = PlayerColorsManager.CurrentColors.SecondaryColor;
        bool usePrimaryAsEmission = PlayerColorsManager.CurrentColors.UsePrimaryAsEmission;
        bool colorizeMenu = settings != null && settings.GetSettingsBool(PlayerColorsManager.ColourizeMenuSettingKey);
        bool hasAccent = colorizeMenu && ((p1.r | p1.g | p1.b | p2.r | p2.g | p2.b) != 0);
        EnsurePlayerColorCatalog();

        string worldName = string.Empty;
        try
        {
            worldName = MetaPort.Instance?.CurrentWorldName?.Trim() ?? string.Empty;
        }
        catch
        {
            // Ignore when not yet in an instance.
        }

        string pfpUrl = null;
        try
        {
            pfpUrl = PlayerSetup.Instance?.playerDescriptor?.profileImageUrl;
            if (string.IsNullOrWhiteSpace(pfpUrl))
                pfpUrl = AuthManager.LastAuthenticationData?.Image?.ToString();
            if (string.IsNullOrWhiteSpace(pfpUrl))
                pfpUrl = PlayerSetup.Instance?.AuthInfo?.Image?.ToString();
            if (string.IsNullOrWhiteSpace(pfpUrl))
                pfpUrl = mod.Social?.LocalImageUrl;
            if (string.IsNullOrWhiteSpace(pfpUrl))
                pfpUrl = ABI_RC.Systems.UI.UILib.QuickMenuAPI.LocalPlayerObject?.PlayerIconURL;
        }
        catch
        {
            // PlayerSetup / AuthManager may not be initialized yet during early startup.
        }
        if (!string.IsNullOrWhiteSpace(pfpUrl))
        {
            images?.RequestLocalPfp(NativeBridge.SurfaceQuick, pfpUrl);
            images?.RequestLocalPfp(NativeBridge.SurfaceMain, pfpUrl);
        }

        string userId = MetaPort.Instance?.ownerId ?? AuthManager.UserId ?? string.Empty;
        string userRank = string.Empty;
        string featuredBadge = mod.Social?.LocalFeaturedBadge ?? string.Empty;
        string featuredBadgeImgUrl = mod.Social?.LocalFeaturedBadgeImageUrl ?? string.Empty;
        string featuredGroup = mod.Social?.LocalFeaturedGroup ?? string.Empty;
        string featuredGroupImgUrl = mod.Social?.LocalFeaturedGroupImageUrl ?? string.Empty;
        try
        {
            var auth = AuthManager.LastAuthenticationData ?? PlayerSetup.Instance?.AuthInfo;
            if (!string.IsNullOrWhiteSpace(mod.Social?.LocalRank))
                userRank = mod.Social.LocalRank;
            else if (!string.IsNullOrWhiteSpace(auth?.RankInfo?.ProfileName))
                userRank = auth.RankInfo.ProfileName;
            else if (!string.IsNullOrWhiteSpace(MetaPort.Instance?.playerRank))
                userRank = MetaPort.Instance.playerRank;

            if (string.IsNullOrWhiteSpace(featuredBadge) && !string.IsNullOrWhiteSpace(auth?.FeaturedBadge?.Name))
                featuredBadge = auth.FeaturedBadge.Name;
            if (string.IsNullOrWhiteSpace(featuredBadgeImgUrl) && !string.IsNullOrWhiteSpace(auth?.FeaturedBadge?.Image))
                featuredBadgeImgUrl = auth.FeaturedBadge.Image;

            if (string.IsNullOrWhiteSpace(featuredGroup) && !string.IsNullOrWhiteSpace(auth?.FeaturedGroup?.Name))
                featuredGroup = auth.FeaturedGroup.Name;
            if (string.IsNullOrWhiteSpace(featuredGroupImgUrl) && auth?.FeaturedGroup?.Image != null)
                featuredGroupImgUrl = auth.FeaturedGroup.Image.ToString();
        }
        catch
        {
            // Ignore during early startup.
        }

        bool mainVisible = ViewManager.Instance != null && ViewManager.Instance.IsViewShown;
        bool quickVisible = CVR_MenuManager.Instance != null && CVR_MenuManager.Instance.IsViewShown;

        string featuredBadgeImgKey = (mainVisible && images != null && !string.IsNullOrWhiteSpace(featuredBadgeImgUrl))
            ? images.RequestThumbnail(NativeBridge.SurfaceMain, featuredBadgeImgUrl)
            : string.Empty;
        string featuredGroupImgKey = (mainVisible && images != null && !string.IsNullOrWhiteSpace(featuredGroupImgUrl))
            ? images.RequestThumbnail(NativeBridge.SurfaceMain, featuredGroupImgUrl)
            : string.Empty;

        string instanceName = string.Empty;
        string instancePrivacy = string.Empty;
        string instanceOwner = string.Empty;
        bool instanceLocked = false;
        int instanceMaxProps = 0;
        string gsVersion = string.Empty;
        int instanceUptimeMins = -1;
        int instancePlayers = 1;
        try
        {
            instanceName = ABI_RC.Core.Networking.IO.Instancing.Instances.CurrentInstanceName?.Trim() ?? string.Empty;
            instancePrivacy = ABI_RC.Core.Networking.IO.Instancing.Instances.CurrentInstancePrivacyType.ToString();
            if (CVRPlayerManager.Instance?.NetworkPlayers != null)
                instancePlayers = CVRPlayerManager.Instance.NetworkPlayers.Count + 1;

            if (GarrisonConnection.Instance != null
                && GarrisonConnection.Instance.IsConnected()
                && GSInfoHandler.LastGSInfoUpdate.HasValue)
            {
                var gs = GSInfoHandler.LastGSInfoUpdate.Value;
                if (!string.IsNullOrWhiteSpace(gs.InstanceName))
                    instanceName = gs.InstanceName.Trim();
                string prettyPriv = ABI_RC.Core.Networking.IO.Instancing.Instances.GetPrettyInstanceName(
                    gs.Privacy.ToInstancePrivacyType());
                if (!string.IsNullOrWhiteSpace(prettyPriv))
                    instancePrivacy = prettyPriv;
                instanceOwner = gs.OwnerName ?? string.Empty;
                instanceLocked = gs.Locked;
                instanceMaxProps = gs.MaxPropsPerUser;
                gsVersion = gs.Version ?? string.Empty;
                if (gs.StartedAtUnix > 0)
                {
                    long elapsedSecs = DateTimeOffset.UtcNow.ToUnixTimeSeconds() - gs.StartedAtUnix;
                    instanceUptimeMins = Math.Max(0, (int)(elapsedSecs / 60));
                }
            }
        }
        catch
        {
            // Ignore when not yet in an instance.
        }

        if (quickVisible || _cachedUiLibSnapshot == null)
        {
            _cachedUiLibSnapshot = mod.UiLib?.BuildSnapshot(images);
        }

        bool uixAvailable = mod.Uix != null && mod.Uix.IsAvailable;
        if (quickVisible && !mainVisible)
        {
            mod.Social?.EnsureLocalProfileLoaded();
        }

        if (mainVisible)
        {
            mod.Social?.EnsureHomeFeedsLoaded();
            _cachedUixCategories = uixAvailable ? mod.Uix.BuildCategoriesSnapshot() : Array.Empty<object>();
            if (mod.Social != null)
            {
                _cachedOnlineFriends = mod.Social.BuildOnlineFriendsLeaflet();
                _cachedActiveWorlds = mod.Social.BuildActiveWorldsLeaflet();
                _cachedActivityFeed = mod.Social.BuildActivityFeed();
            }
            _cachedCvrSettings = SettingsBridge.BuildSnapshot();
            _cachedAvatarSettings = AvatarSettingsBridge.BuildSnapshot();
        }
        else
        {
            _cachedCvrSettings ??= new { items = Array.Empty<object>() };
            _cachedAvatarSettings ??= new
            {
                auto_apply = true,
                has_staged = false,
                current_profile = string.Empty,
                staged_profile = string.Empty,
                avatar_height = 1.6f,
                initial_height = 1.6f,
                scale_min = 0.2f,
                scale_max = 5.0f,
                scaling_allowed = true,
                entries = Array.Empty<object>(),
                profiles = Array.Empty<string>()
            };
        }

        bool contentQmDirty = mod.Content != null && mod.Content.ConsumeQmDirty();
        if (mainVisible || _cachedContentSnapshot == null || contentQmDirty)
        {
            _cachedContentSnapshot = mod.Content?.BuildSnapshot(
                mainVisible ? images : null,
                quickVisible ? images : null);
        }

        bool socialQmDirty = mod.Social != null && mod.Social.ConsumeQmDirty();
        if (mainVisible || _cachedSocialSnapshot == null || socialQmDirty)
        {
            _cachedSocialSnapshot = mod.Social?.BuildSocialSnapshot(mainVisible ? images : null);
        }

        object contentSnapshot = _cachedContentSnapshot;
        object socialSnapshot = _cachedSocialSnapshot;

        var snapshot = new
        {
            is_vr = MetaPort.Instance != null && MetaPort.Instance.isUsingVr,
            user_id = userId,
            username = AuthManager.Username ?? string.Empty,
            user_rank = userRank,
            cvr_version = Application.version ?? string.Empty,
            cvr_branch = MetaPort.Instance?.SteamBuildBranchName ?? string.Empty,
            world_name = worldName,
            local_pfp_key = "local_pfp",

            is_muted = Comms_Manager.IsMicMuted,
            is_flying = isFlying,
            flight_allowed = flightAllowed,
            is_seated = isSeated,
            fbt_available = fbtAvail,
            fbt_enabled = fbtEnabled,
            camera_open = PortableCameraController.Instance != null && PortableCameraController.Instance.IsOpen,
            mirror_open = IsPortableMirrorOpen(),
            minime_enabled = AvatarStageManager.IsMiniMeEnabledInQuickMenu(),

            audio_master = audioMaster,
            audio_voice = audioVoice,
            audio_video_player = audioVideo,
            nav_seq = mod.NavSeq,
            mm_open_seq = mod.MmOpenSeq,
            nav_tab = mod.NavTab ?? string.Empty,

            accent_r = p1.r,
            accent_g = p1.g,
            accent_b = p1.b,
            accent2_r = p2.r,
            accent2_g = p2.g,
            accent2_b = p2.b,
            has_player_accent = hasAccent,
            colorize_menu = colorizeMenu,
            use_primary_as_emission = usePrimaryAsEmission,
            player_color_presets = _cachedColorPresets,
            player_color_swatches = _cachedColorSwatches,

            notifications = CollectQmNotifications(),
            featured_badge = featuredBadge,
            featured_badge_image_key = featuredBadgeImgKey,
            featured_group = featuredGroup,
            featured_group_image_key = featuredGroupImgKey,
            instance_name = instanceName,
            instance_privacy = instancePrivacy,
            instance_owner = instanceOwner,
            instance_locked = instanceLocked,
            instance_max_props = instanceMaxProps,
            gs_version = gsVersion,
            instance_uptime_mins = instanceUptimeMins,
            instance_players = instancePlayers,
            online_friends = _cachedOnlineFriends,
            active_worlds = _cachedActiveWorlds,
            activity_feed = _cachedActivityFeed,
            uilib = _cachedUiLibSnapshot,
            uix_available = uixAvailable,
            uix_categories = _cachedUixCategories,
            content = contentSnapshot,
            social = socialSnapshot,
            cvr_settings = _cachedCvrSettings,
            avatar_settings = _cachedAvatarSettings,
            ui_click_sounds = Settings.UiClickSounds,
            ui_sound_pack = Settings.UiSoundPack,
            enable_recent_history = Settings.EnableRecentHistory,
            disclaimer_accepted = Settings.DisclaimerAccepted,
            show_whats_new = !string.Equals(Settings.LastSeenVersion, LumenicMenuMod.ModVersion, StringComparison.OrdinalIgnoreCase),
            whats_new_notes = ModUpdateService.BundledChangelog,
            update_status = mod.Updates?.BuildSnapshot() ?? new UpdateStatusDto { current_version = LumenicMenuMod.ModVersion },
            system_alert = mod.SystemAlerts?.PeekActive(),
            desktop_scroll_multiplier = Settings.DesktopScrollMultiplier,
            vr_scroll_multiplier = Settings.VrScrollMultiplier,
            qm_quick_actions_order = ParseOrderCsv(Settings.QmQuickActionsOrder),
            mm_quick_tools_order = ParseOrderCsv(Settings.MmQuickToolsOrder),

            mm_width = mmW,
            mm_height = mmH,
            qm_width = qmW,
            qm_height = qmH,
            mm_step_ms = Settings.PerfLogging ? (float)Math.Round(mainSurface.LastStepMs, 2) : 0f,
            mm_readback_ms = Settings.PerfLogging ? (float)Math.Round(mainSurface.LastReadbackMs, 2) : 0f,
            mm_upload_ms = Settings.PerfLogging ? (float)Math.Round(mainSurface.LastUploadMs, 2) : 0f,
            mm_blit_ms = Settings.PerfLogging ? (float)Math.Round(mainSurface.LastBlitMs, 2) : 0f,
            qm_step_ms = Settings.PerfLogging ? (float)Math.Round(quickSurface.LastStepMs, 2) : 0f,
            qm_readback_ms = Settings.PerfLogging ? (float)Math.Round(quickSurface.LastReadbackMs, 2) : 0f,
            qm_upload_ms = Settings.PerfLogging ? (float)Math.Round(quickSurface.LastUploadMs, 2) : 0f,
            qm_blit_ms = Settings.PerfLogging ? (float)Math.Round(quickSurface.LastBlitMs, 2) : 0f,
            max_refresh_hz = Settings.MaxRefreshHz,
            font_scale = Settings.FontScale,
            linear_texture = Settings.LinearTextureMode,
            perf_logging = Settings.PerfLogging
        };

        return JsonConvert.SerializeObject(snapshot);
    }

    private static string[] ParseOrderCsv(string csv)
    {
        if (string.IsNullOrWhiteSpace(csv))
            return Array.Empty<string>();
        return csv.Split(new[] { ',' }, StringSplitOptions.RemoveEmptyEntries);
    }

    private static int GetGamePing()
    {
        try
        {
            GarrisonConnection conn = GarrisonConnection.Instance;
            if (conn != null && conn.IsConnected() && conn.Ping.HasValue)
                return (int)conn.Ping.Value;
        }
        catch
        {
            // Ignore when disconnected or before Garrison initializes.
        }
        return 0;
    }

    private static int GetVoicePing()
    {
        try
        {
            CrierConnection conn = CrierConnection.Instance;
            if (conn != null && conn.IsConnected())
                return conn.Ping;
        }
        catch
        {
            // Ignore when disconnected or before Crier initializes.
        }
        return 0;
    }

    private static bool IsPortableMirrorOpen()
    {
        try
        {
            if (BigMenuToggleMirrorField != null)
            {
                return (UnityEngine.Object)BigMenuToggleMirrorField.GetValue(null) != null;
            }
        }
        catch
        {
            // Ignore reflection errors if game internals change.
        }
        return false;
    }

    private static List<object> CollectQmNotifications()
    {
        var list = new List<object>();
        if (UIMessageManager.Instance == null)
            return list;

        try
        {
            CVRUIMessage[] messages = UIMessageManager.Instance.GetAllMessagesForQM();
            if (messages == null)
                return list;

            int cap = Math.Min(messages.Length, 20);
            for (int i = 0; i < cap; i++)
            {
                CVRUIMessage msg = messages[i];
                if (msg == null || msg.MessageCleared)
                    continue;

                var btns = new List<object>();
                if (msg.Buttons != null)
                {
                    foreach (UIMessageButton b in msg.Buttons)
                    {
                        if (b == null) continue;
                        btns.Add(new
                        {
                            id = b.ButtonElementID ?? string.Empty,
                            text = b.ButtonText ?? "Action",
                            is_accept = b.QuickMenuAcceptButton,
                            is_decline = b.QuickMenuDeclineButton
                        });
                    }
                }

                string body = !string.IsNullOrWhiteSpace(msg.QuickMenuText)
                    ? msg.QuickMenuText
                    : (msg.MessageText ?? string.Empty);

                list.Add(new
                {
                    id = msg.MessageElementID ?? string.Empty,
                    title = msg.MessageName ?? "Notification",
                    body,
                    category = msg.MessageCategory.ToString(),
                    buttons = btns
                });
            }
        }
        catch
        {
            // Keep UI alive even if message list mutates mid-read.
        }
        return list;
    }

    private static List<object> _cachedColorPresets;
    private static List<object> _cachedColorSwatches;

    private static void EnsurePlayerColorCatalog()
    {
        if (_cachedColorPresets != null && _cachedColorSwatches != null)
            return;

        var presets = new List<object>();
        try
        {
            foreach (PlayerColorPresets p in Enum.GetValues(typeof(PlayerColorPresets)))
            {
                if (!PlayerColorsUtility.ColorPresetToColor.TryGetValue(p, out var pair))
                    continue;
                Color32 c1 = p == PlayerColorPresets.ChilloutVR ? (Color32)pair.Item2 : (Color32)pair.Item1;
                Color32 c2 = p == PlayerColorPresets.ChilloutVR ? (Color32)pair.Item1 : (Color32)pair.Item2;
                presets.Add(new
                {
                    id = (int)p,
                    name = PrettyEnumName(p.ToString()),
                    r1 = c1.r,
                    g1 = c1.g,
                    b1 = c1.b,
                    r2 = c2.r,
                    g2 = c2.g,
                    b2 = c2.b
                });
            }
        }
        catch
        {
            // Keep empty list if enum unavailable.
        }

        var swatches = new List<object>();
        try
        {
            foreach (KeyValuePair<PlayerColorSwatches, Color> kv in PlayerColorsUtility.ColorSwatchToColor)
            {
                Color32 c = kv.Value;
                swatches.Add(new
                {
                    id = (int)kv.Key,
                    name = PrettyEnumName(kv.Key.ToString()),
                    r = c.r,
                    g = c.g,
                    b = c.b
                });
            }
        }
        catch
        {
            // Keep empty list if enum unavailable.
        }

        _cachedColorPresets = presets;
        _cachedColorSwatches = swatches;
    }

    private static string PrettyEnumName(string raw)
    {
        if (string.IsNullOrEmpty(raw))
            return string.Empty;
        var sb = new System.Text.StringBuilder(raw.Length + 4);
        for (int i = 0; i < raw.Length; i++)
        {
            char ch = raw[i];
            if (i > 0 && char.IsUpper(ch) && !char.IsUpper(raw[i - 1]))
                sb.Append(' ');
            sb.Append(ch);
        }
        return sb.ToString();
    }
}
