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
using MelonLoader;
using Newtonsoft.Json;
using UnityEngine;

namespace LumenicMenu.Bridge;

[Flags]
internal enum StateDomain
{
    None = 0,
    Core = 1 << 0,
    StaticCatalog = 1 << 1,
    UiLib = 1 << 2,
    Content = 1 << 3,
    Social = 1 << 4,
    Settings = 1 << 5,
    AvatarSettings = 1 << 6,
    Uix = 1 << 7,
    All = Core | StaticCatalog | UiLib | Content | Social | Settings | AvatarSettings | Uix
}

/// <summary>
/// Builds and pushes domain-scoped JSON state snapshots to the Rust UI layer.
/// Each domain diffs independently and only dirties the surface(s) that consume it,
/// avoiding monolithic JSON serialization and cross-surface redraw storms.
/// </summary>
internal sealed class HostStateBuilder
{
    private static readonly FieldInfo BigMenuToggleMirrorField =
        AccessTools.Field(typeof(PortableMirrorInstance), "_bigMenuToggleMirror");

    private sealed class NotificationButtonDto
    {
        public string id;
        public string text;
        public bool is_accept;
        public bool is_decline;
    }

    private sealed class NotificationDto
    {
        public string id;
        public string title;
        public string body;
        public string category;
        public List<NotificationButtonDto> buttons;
    }

    private sealed class ColorPresetDto
    {
        public int id;
        public string name;
        public byte r1;
        public byte g1;
        public byte b1;
        public byte r2;
        public byte g2;
        public byte b2;
    }

    private sealed class ColorSwatchDto
    {
        public int id;
        public string name;
        public byte r;
        public byte g;
        public byte b;
    }

    private sealed class StaticCatalogDto
    {
        public List<ColorPresetDto> player_color_presets;
        public List<ColorSwatchDto> player_color_swatches;
        public string whats_new_notes;
    }

    private sealed class SocialDomainDto
    {
        public List<object> online_friends;
        public List<object> active_worlds;
        public List<object> activity_feed;
        public object social;
    }

    private sealed class UixDomainDto
    {
        public bool uix_available;
        public object uix_categories;
    }

    private sealed class CoreStateDto
    {
        public bool is_vr;
        public string user_id;
        public string username;
        public string user_rank;
        public string cvr_version;
        public string cvr_branch;
        public string world_name;
        public string local_pfp_key;

        public bool is_muted;
        public bool is_flying;
        public bool flight_allowed;
        public bool is_seated;
        public bool fbt_available;
        public bool fbt_enabled;
        public bool camera_open;
        public bool mirror_open;
        public bool minime_enabled;

        public int audio_master;
        public int audio_voice;
        public int audio_video_player;
        public uint nav_seq;
        public uint mm_open_seq;
        public string nav_tab;

        public byte accent_r;
        public byte accent_g;
        public byte accent_b;
        public byte accent2_r;
        public byte accent2_g;
        public byte accent2_b;
        public bool has_player_accent;
        public bool colorize_menu;
        public bool use_primary_as_emission;

        public List<NotificationDto> notifications;
        public string featured_badge;
        public string featured_badge_image_key;
        public string featured_group;
        public string featured_group_image_key;
        public string instance_name;
        public string instance_privacy;
        public string instance_owner;
        public bool instance_locked;
        public int instance_max_props;
        public string gs_version;
        public int instance_uptime_mins;
        public int instance_players;

        public string[] qm_quick_actions_order;
        public string[] mm_quick_tools_order;
        public bool ui_click_sounds;
        public string ui_sound_pack;
        public bool enable_recent_history;
        public bool qm_mod_tabs_icon_only;
        public bool disclaimer_accepted;
        public bool show_whats_new;
        public UpdateStatusDto update_status;
        public SystemAlertDto system_alert;
        public float desktop_scroll_multiplier;
        public float vr_scroll_multiplier;
        public float hud_spread_desktop;
        public float hud_spread_y_desktop;
        public float hud_spread_vr;
        public float hud_spread_y_vr;
        public float hud_alerts_scale_desktop;
        public float hud_mic_scale_desktop;
        public float hud_notifications_scale_desktop;
        public float hud_chatbox_scale_desktop;
        public float hud_alerts_scale_vr;
        public float hud_mic_scale_vr;
        public float hud_notifications_scale_vr;
        public float hud_chatbox_scale_vr;

        public uint mm_width;
        public uint mm_height;
        public uint qm_width;
        public uint qm_height;
        public float mm_step_ms;
        public float mm_readback_ms;
        public float mm_upload_ms;
        public float mm_blit_ms;
        public float qm_step_ms;
        public float qm_readback_ms;
        public float qm_upload_ms;
        public float qm_blit_ms;
        public int max_refresh_hz;
        public float font_scale;
        public bool linear_texture;
        public bool perf_logging;
    }

    private string _lastCoreJson = string.Empty;
    private string _lastUiLibJson = string.Empty;
    private string _lastContentJson = string.Empty;
    private string _lastSocialJson = string.Empty;
    private string _lastSettingsJson = string.Empty;
    private string _lastAvatarSettingsJson = string.Empty;
    private string _lastUixJson = string.Empty;
    private bool _staticCatalogPushed;

    public void InvalidateCache()
    {
        _lastCoreJson = string.Empty;
        _lastUiLibJson = string.Empty;
        _lastContentJson = string.Empty;
        _lastSocialJson = string.Empty;
        _lastSettingsJson = string.Empty;
        _lastAvatarSettingsJson = string.Empty;
        _lastUixJson = string.Empty;
        _staticCatalogPushed = false;
    }

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

    public void PushDirtyDomains(
        StateDomain dirty,
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
        if (bridge == null || !bridge.IsHealthy)
            return;

        bool mainVisible = ViewManager.Instance != null && ViewManager.Instance.IsViewShown;
        bool quickVisible = CVR_MenuManager.Instance != null && CVR_MenuManager.Instance.IsViewShown;

        if (!_staticCatalogPushed || (dirty & StateDomain.StaticCatalog) != 0)
        {
            EnsurePlayerColorCatalog();
            string catalogJson = JsonConvert.SerializeObject(new StaticCatalogDto
            {
                player_color_presets = _cachedColorPresets,
                player_color_swatches = _cachedColorSwatches,
                whats_new_notes = ModUpdateService.BundledChangelog
            });
            if (bridge.PushDomainJson(NativeBridge.DomainStaticCatalog, catalogJson))
            {
                _staticCatalogPushed = true;
            }
            else
            {
                MelonLogger.Warning($"[LumenicMenu] PushDomainJson(StaticCatalog) failed: {bridge.GetLastError()}");
            }
        }

        if ((dirty & StateDomain.Core) != 0 || string.IsNullOrEmpty(_lastCoreJson))
        {
            string coreJson = BuildCoreJson(mod, images, mainSurface, quickSurface, mainVisible, mmW, mmH, qmW, qmH);
            PushIfChanged(bridge, NativeBridge.DomainCore, "Core", coreJson, ref _lastCoreJson);
        }

        if (quickVisible && ((dirty & StateDomain.UiLib) != 0 || string.IsNullOrEmpty(_lastUiLibJson)))
        {
            object uiLibSnap = mod.UiLib?.BuildSnapshot(images);
            if (uiLibSnap != null)
            {
                string uiLibJson = JsonConvert.SerializeObject(uiLibSnap);
                PushIfChanged(bridge, NativeBridge.DomainUiLib, "UiLib", uiLibJson, ref _lastUiLibJson);
            }
        }

        if (quickVisible && !mainVisible)
        {
            mod.Social?.EnsureLocalProfileLoaded();
        }

        bool contentQmDirty = mod.Content != null && mod.Content.ConsumeQmDirty();
        if (contentQmDirty || (mainVisible && ((dirty & StateDomain.Content) != 0 || string.IsNullOrEmpty(_lastContentJson))))
        {
            object contentSnap = mod.Content?.BuildSnapshot(
                mainVisible ? images : null,
                quickVisible ? images : null);
            if (contentSnap != null)
            {
                string contentJson = JsonConvert.SerializeObject(contentSnap);
                PushIfChanged(bridge, NativeBridge.DomainContent, "Content", contentJson, ref _lastContentJson);
            }
        }

        bool socialQmDirty = mod.Social != null && mod.Social.ConsumeQmDirty();
        if (socialQmDirty || (mainVisible && ((dirty & StateDomain.Social) != 0 || string.IsNullOrEmpty(_lastSocialJson))))
        {
            if (mainVisible)
            {
                mod.Social?.EnsureHomeFeedsLoaded();
            }
            var socialDom = new SocialDomainDto
            {
                online_friends = mod.Social?.BuildOnlineFriendsLeaflet() ?? new List<object>(),
                active_worlds = mod.Social?.BuildActiveWorldsLeaflet() ?? new List<object>(),
                activity_feed = mod.Social?.BuildActivityFeed() ?? new List<object>(),
                social = mod.Social?.BuildSocialSnapshot(mainVisible ? images : null)
            };
            string socialJson = JsonConvert.SerializeObject(socialDom);
            PushIfChanged(bridge, NativeBridge.DomainSocial, "Social", socialJson, ref _lastSocialJson);
        }

        if (mainVisible && ((dirty & StateDomain.Settings) != 0 || string.IsNullOrEmpty(_lastSettingsJson)))
        {
            string settingsJson = JsonConvert.SerializeObject(SettingsBridge.BuildSnapshot());
            PushIfChanged(bridge, NativeBridge.DomainSettings, "Settings", settingsJson, ref _lastSettingsJson);
        }

        if (mainVisible && ((dirty & StateDomain.AvatarSettings) != 0 || string.IsNullOrEmpty(_lastAvatarSettingsJson)))
        {
            string avatarJson = JsonConvert.SerializeObject(AvatarSettingsBridge.BuildSnapshot());
            PushIfChanged(bridge, NativeBridge.DomainAvatarSettings, "AvatarSettings", avatarJson, ref _lastAvatarSettingsJson);
        }

        if (mainVisible && ((dirty & StateDomain.Uix) != 0 || string.IsNullOrEmpty(_lastUixJson)))
        {
            bool uixAvailable = mod.Uix != null && mod.Uix.IsAvailable;
            var uixDom = new UixDomainDto
            {
                uix_available = uixAvailable,
                uix_categories = uixAvailable ? mod.Uix.BuildCategoriesSnapshot() : Array.Empty<object>()
            };
            string uixJson = JsonConvert.SerializeObject(uixDom);
            PushIfChanged(bridge, NativeBridge.DomainUix, "Uix", uixJson, ref _lastUixJson);
        }
    }

    private static void PushIfChanged(NativeBridge bridge, uint domainId, string label, string json, ref string lastJson)
    {
        if (string.Equals(json, lastJson, StringComparison.Ordinal))
            return;

        if (bridge.PushDomainJson(domainId, json))
        {
            lastJson = json;
        }
        else
        {
            MelonLogger.Warning($"[LumenicMenu] PushDomainJson({label}) failed: {bridge.GetLastError()}");
        }
    }

    private static string BuildCoreJson(
        LumenicMenuMod mod,
        ImageLoaderService images,
        MenuSurfaceHost mainSurface,
        MenuSurfaceHost quickSurface,
        bool mainVisible,
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

        string worldName = string.Empty;
        try
        {
            worldName = MetaPort.Instance?.CurrentWorldName?.Trim() ?? string.Empty;
        }
        catch (Exception ex)
        {
            DiagLog.WarnOnce("HostState.WorldName", ex);
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
        catch (Exception ex)
        {
            DiagLog.WarnOnce("HostState.PfpUrl", ex);
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
        catch (Exception ex)
        {
            DiagLog.WarnOnce("HostState.WorldName", ex);
        }

        var dto = new CoreStateDto
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

            qm_quick_actions_order = ParseOrderCsv(Settings.QmQuickActionsOrder),
            mm_quick_tools_order = ParseOrderCsv(Settings.MmQuickToolsOrder),
            ui_click_sounds = Settings.UiClickSounds,
            ui_sound_pack = Settings.UiSoundPack,
            enable_recent_history = Settings.EnableRecentHistory,
            qm_mod_tabs_icon_only = Settings.QmModTabsIconOnly,
            disclaimer_accepted = Settings.DisclaimerAccepted,
            show_whats_new = !string.Equals(Settings.LastSeenVersion, LumenicMenuMod.ModVersion, StringComparison.OrdinalIgnoreCase),
            update_status = mod.Updates?.BuildSnapshot() ?? new UpdateStatusDto { current_version = LumenicMenuMod.ModVersion },
            system_alert = mod.SystemAlerts?.PeekActive(),
            desktop_scroll_multiplier = Settings.DesktopScrollMultiplier,
            vr_scroll_multiplier = Settings.VrScrollMultiplier,
            hud_spread_desktop = Settings.HudSpreadDesktop,
            hud_spread_y_desktop = Settings.HudSpreadYDesktop,
            hud_spread_vr = Settings.HudSpreadVr,
            hud_spread_y_vr = Settings.HudSpreadYVr,
            hud_alerts_scale_desktop = Settings.HudAlertsScaleDesktop,
            hud_mic_scale_desktop = Settings.HudMicScaleDesktop,
            hud_notifications_scale_desktop = Settings.HudNotificationsScaleDesktop,
            hud_chatbox_scale_desktop = Settings.HudChatboxScaleDesktop,
            hud_alerts_scale_vr = Settings.HudAlertsScaleVr,
            hud_mic_scale_vr = Settings.HudMicScaleVr,
            hud_notifications_scale_vr = Settings.HudNotificationsScaleVr,
            hud_chatbox_scale_vr = Settings.HudChatboxScaleVr,

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

        return JsonConvert.SerializeObject(dto);
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

    private static List<NotificationDto> CollectQmNotifications()
    {
        var list = new List<NotificationDto>();
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

                var btns = new List<NotificationButtonDto>();
                if (msg.Buttons != null)
                {
                    foreach (UIMessageButton b in msg.Buttons)
                    {
                        if (b == null) continue;
                        btns.Add(new NotificationButtonDto
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

                list.Add(new NotificationDto
                {
                    id = msg.MessageElementID ?? string.Empty,
                    title = msg.MessageName ?? "Notification",
                    body = body,
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

    private static List<ColorPresetDto> _cachedColorPresets;
    private static List<ColorSwatchDto> _cachedColorSwatches;

    private static void EnsurePlayerColorCatalog()
    {
        if (_cachedColorPresets != null && _cachedColorSwatches != null)
            return;

        var presets = new List<ColorPresetDto>();
        try
        {
            foreach (PlayerColorPresets p in Enum.GetValues(typeof(PlayerColorPresets)))
            {
                if (!PlayerColorsUtility.ColorPresetToColor.TryGetValue(p, out var pair))
                    continue;
                Color32 c1 = p == PlayerColorPresets.ChilloutVR ? (Color32)pair.Item2 : (Color32)pair.Item1;
                Color32 c2 = p == PlayerColorPresets.ChilloutVR ? (Color32)pair.Item1 : (Color32)pair.Item2;
                presets.Add(new ColorPresetDto
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

        var swatches = new List<ColorSwatchDto>();
        try
        {
            foreach (KeyValuePair<PlayerColorSwatches, Color> kv in PlayerColorsUtility.ColorSwatchToColor)
            {
                Color32 c = kv.Value;
                swatches.Add(new ColorSwatchDto
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
