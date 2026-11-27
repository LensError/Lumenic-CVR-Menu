use serde::{Deserialize, Serialize};

use super::dtos::*;

/// Snapshot of live ChilloutVR host state pushed from C# only when values change.
#[derive(Clone, Debug, Deserialize)]
#[serde(default)]
pub struct HostStateSnapshot {
    pub is_vr: bool,
    pub user_id: String,
    pub username: String,
    pub user_rank: String,
    pub cvr_version: String,
    pub cvr_branch: String,
    pub world_name: String,
    pub clock_text: String,
    pub date_text: String,
    pub fps: i32,
    pub game_ping: i32,
    pub voice_ping: i32,
    pub local_pfp_key: String,

    // Live player & tool toggle states
    pub is_muted: bool,
    pub is_flying: bool,
    pub flight_allowed: bool,
    pub is_seated: bool,
    pub fbt_available: bool,
    pub fbt_enabled: bool,
    pub camera_open: bool,
    pub mirror_open: bool,
    pub minime_enabled: bool,

    // Quick audio volumes (0..=100)
    pub audio_master: i32,
    pub audio_voice: i32,
    pub audio_video_player: i32,

    // One-shot external navigation sync from C# ViewManager
    pub nav_seq: u32,
    pub nav_tab: String,

    // Live CVR PlayerColorsManager accent & panel colorizing
    pub accent_r: u8,
    pub accent_g: u8,
    pub accent_b: u8,
    pub accent2_r: u8,
    pub accent2_g: u8,
    pub accent2_b: u8,
    pub has_player_accent: bool,
    pub colorize_menu: bool,
    pub use_primary_as_emission: bool,
    pub player_color_presets: Vec<PlayerColorPresetItem>,
    pub player_color_swatches: Vec<PlayerColorSwatchItem>,

    // Active Quick Menu notifications
    pub notifications: Vec<NotificationItem>,

    // Main Menu Home dashboard data
    pub featured_badge: String,
    pub featured_badge_image_key: String,
    pub featured_group: String,
    pub featured_group_image_key: String,
    pub instance_name: String,
    pub instance_privacy: String,
    pub instance_owner: String,
    pub instance_locked: bool,
    pub instance_max_props: i32,
    pub gs_version: String,
    pub instance_uptime_mins: i32,
    pub instance_players: i32,
    pub online_friends: Vec<OnlineFriendItem>,
    pub active_worlds: Vec<ActiveWorldItem>,
    pub activity_feed: Vec<ActivityFeedItem>,

    // Native CVR UILib mod tabs, active page, modals, and toasts
    pub uilib: UiLibSnapshot,

    // Mod Settings bridge (Mowods / UIExpansionKit)
    pub uix_available: bool,
    pub mod_settings_source: String,
    pub mod_settings_both_available: bool,
    pub uix_categories: Vec<UixCategorySnapshot>,

    // Phase 3B Content Browser (Worlds, Avatars, Props, Global Search)
    pub content: ContentBrowserSnapshot,

    // Phase 3C Social, Friends, Current Instance & User Details
    pub social: SocialSnapshot,

    // Phase 3D CVR Settings & LumenicMenu Preferences
    pub cvr_settings: CvrSettingsSnapshot,
    pub qm_quick_actions_order: Vec<String>,
    pub mm_quick_tools_order: Vec<String>,
    pub avatar_settings: AvatarSettingsSnapshot,
    pub ui_click_sounds: bool,
    pub ui_sound_pack: String,
    pub enable_recent_history: bool,
    #[serde(default)]
    pub qm_mod_tabs_icon_only: bool,
    #[serde(default = "default_qm_preset_confirm_mode")]
    pub qm_preset_confirm_mode: String,
    #[serde(default = "default_theme_preset")]
    pub theme_preset: String,
    #[serde(default = "default_one")]
    pub panel_opacity: f32,
    #[serde(default = "default_true")]
    pub cursor_glow: bool,
    #[serde(default = "default_true")]
    pub blend_player_color: bool,
    pub disclaimer_accepted: bool,
    pub show_whats_new: bool,
    pub whats_new_notes: String,
    pub update_status: UpdateStatusDto,
    pub system_alert: Option<SystemAlertDto>,
    pub desktop_scroll_multiplier: f32,
    pub vr_scroll_multiplier: f32,
    pub hud_spread_desktop: f32,
    pub hud_spread_y_desktop: f32,
    pub hud_spread_vr: f32,
    pub hud_spread_y_vr: f32,
    pub hud_alerts_scale_desktop: f32,
    pub hud_mic_scale_desktop: f32,
    pub hud_notifications_scale_desktop: f32,
    pub hud_chatbox_scale_desktop: f32,
    pub hud_alerts_scale_vr: f32,
    pub hud_mic_scale_vr: f32,
    pub hud_notifications_scale_vr: f32,
    pub hud_chatbox_scale_vr: f32,

    // Diagnostics & surface info
    pub mm_width: u32,
    pub mm_height: u32,
    pub qm_width: u32,
    pub qm_height: u32,
    #[serde(default)]
    pub mm_open_seq: u32,
    pub mm_step_ms: f32,
    pub mm_readback_ms: f32,
    pub mm_upload_ms: f32,
    pub mm_blit_ms: f32,
    pub qm_step_ms: f32,
    pub qm_readback_ms: f32,
    pub qm_upload_ms: f32,
    pub qm_blit_ms: f32,
    pub max_refresh_hz: u32,
    pub font_scale: f32,
    pub linear_texture: bool,
    pub perf_logging: bool,
}

#[derive(Clone, Debug, Default, Deserialize)]
#[serde(default)]
pub struct StaticCatalogSnapshot {
    pub player_color_presets: Vec<PlayerColorPresetItem>,
    pub player_color_swatches: Vec<PlayerColorSwatchItem>,
    pub whats_new_notes: String,
}

#[derive(Clone, Debug, Default, Deserialize)]
#[serde(default)]
pub struct SocialDomainSnapshot {
    pub online_friends: Vec<OnlineFriendItem>,
    pub active_worlds: Vec<ActiveWorldItem>,
    pub activity_feed: Vec<ActivityFeedItem>,
    pub social: SocialSnapshot,
}

#[derive(Clone, Debug, Default, Deserialize)]
#[serde(default)]
pub struct UixDomainSnapshot {
    pub uix_available: bool,
    pub mod_settings_source: String,
    pub mod_settings_both_available: bool,
    pub uix_categories: Vec<UixCategorySnapshot>,
}

/// Core lightweight state pushed independently of heavy Main Menu / UILib domains.
#[derive(Clone, Debug, Default, Deserialize)]
#[serde(default)]
pub struct CoreStatePatch {
    pub is_vr: bool,
    pub user_id: String,
    pub username: String,
    pub user_rank: String,
    pub cvr_version: String,
    pub cvr_branch: String,
    pub world_name: String,
    pub local_pfp_key: String,

    pub is_muted: bool,
    pub is_flying: bool,
    pub flight_allowed: bool,
    pub is_seated: bool,
    pub fbt_available: bool,
    pub fbt_enabled: bool,
    pub camera_open: bool,
    pub mirror_open: bool,
    pub minime_enabled: bool,

    pub audio_master: i32,
    pub audio_voice: i32,
    pub audio_video_player: i32,
    pub nav_seq: u32,
    pub mm_open_seq: u32,
    pub nav_tab: String,

    pub accent_r: u8,
    pub accent_g: u8,
    pub accent_b: u8,
    pub accent2_r: u8,
    pub accent2_g: u8,
    pub accent2_b: u8,
    pub has_player_accent: bool,
    pub colorize_menu: bool,
    pub use_primary_as_emission: bool,

    pub notifications: Vec<NotificationItem>,

    pub featured_badge: String,
    pub featured_badge_image_key: String,
    pub featured_group: String,
    pub featured_group_image_key: String,
    pub instance_name: String,
    pub instance_privacy: String,
    pub instance_owner: String,
    pub instance_locked: bool,
    pub instance_max_props: i32,
    pub gs_version: String,
    pub instance_uptime_mins: i32,
    pub instance_players: i32,

    pub qm_quick_actions_order: Vec<String>,
    pub mm_quick_tools_order: Vec<String>,
    pub ui_click_sounds: bool,
    pub ui_sound_pack: String,
    pub enable_recent_history: bool,
    #[serde(default)]
    pub qm_mod_tabs_icon_only: bool,
    #[serde(default = "default_qm_preset_confirm_mode")]
    pub qm_preset_confirm_mode: String,
    #[serde(default = "default_theme_preset")]
    pub theme_preset: String,
    #[serde(default = "default_one")]
    pub panel_opacity: f32,
    #[serde(default = "default_true")]
    pub cursor_glow: bool,
    #[serde(default = "default_true")]
    pub blend_player_color: bool,
    pub disclaimer_accepted: bool,
    pub show_whats_new: bool,
    pub update_status: UpdateStatusDto,
    pub system_alert: Option<SystemAlertDto>,
    pub desktop_scroll_multiplier: f32,
    pub vr_scroll_multiplier: f32,
    pub hud_spread_desktop: f32,
    pub hud_spread_y_desktop: f32,
    pub hud_spread_vr: f32,
    pub hud_spread_y_vr: f32,
    pub hud_alerts_scale_desktop: f32,
    pub hud_mic_scale_desktop: f32,
    pub hud_notifications_scale_desktop: f32,
    pub hud_chatbox_scale_desktop: f32,
    pub hud_alerts_scale_vr: f32,
    pub hud_mic_scale_vr: f32,
    pub hud_notifications_scale_vr: f32,
    pub hud_chatbox_scale_vr: f32,

    pub mm_width: u32,
    pub mm_height: u32,
    pub qm_width: u32,
    pub qm_height: u32,
    pub mm_step_ms: f32,
    pub mm_readback_ms: f32,
    pub mm_upload_ms: f32,
    pub mm_blit_ms: f32,
    pub qm_step_ms: f32,
    pub qm_readback_ms: f32,
    pub qm_upload_ms: f32,
    pub qm_blit_ms: f32,
    pub max_refresh_hz: u32,
    pub font_scale: f32,
    pub linear_texture: bool,
    pub perf_logging: bool,
}

impl HostStateSnapshot {
    pub fn apply_core_patch(&mut self, patch: CoreStatePatch) {
        self.is_vr = patch.is_vr;
        self.user_id = patch.user_id;
        self.username = patch.username;
        self.user_rank = patch.user_rank;
        self.cvr_version = patch.cvr_version;
        self.cvr_branch = patch.cvr_branch;
        self.world_name = patch.world_name;
        self.local_pfp_key = patch.local_pfp_key;

        self.is_muted = patch.is_muted;
        self.is_flying = patch.is_flying;
        self.flight_allowed = patch.flight_allowed;
        self.is_seated = patch.is_seated;
        self.fbt_available = patch.fbt_available;
        self.fbt_enabled = patch.fbt_enabled;
        self.camera_open = patch.camera_open;
        self.mirror_open = patch.mirror_open;
        self.minime_enabled = patch.minime_enabled;

        self.audio_master = patch.audio_master;
        self.audio_voice = patch.audio_voice;
        self.audio_video_player = patch.audio_video_player;
        self.nav_seq = patch.nav_seq;
        self.mm_open_seq = patch.mm_open_seq;
        self.nav_tab = patch.nav_tab;

        self.accent_r = patch.accent_r;
        self.accent_g = patch.accent_g;
        self.accent_b = patch.accent_b;
        self.accent2_r = patch.accent2_r;
        self.accent2_g = patch.accent2_g;
        self.accent2_b = patch.accent2_b;
        self.has_player_accent = patch.has_player_accent;
        self.colorize_menu = patch.colorize_menu;
        self.use_primary_as_emission = patch.use_primary_as_emission;

        self.notifications = patch.notifications;

        self.featured_badge = patch.featured_badge;
        self.featured_badge_image_key = patch.featured_badge_image_key;
        self.featured_group = patch.featured_group;
        self.featured_group_image_key = patch.featured_group_image_key;
        self.instance_name = patch.instance_name;
        self.instance_privacy = patch.instance_privacy;
        self.instance_owner = patch.instance_owner;
        self.instance_locked = patch.instance_locked;
        self.instance_max_props = patch.instance_max_props;
        self.gs_version = patch.gs_version;
        self.instance_uptime_mins = patch.instance_uptime_mins;
        self.instance_players = patch.instance_players;

        self.qm_quick_actions_order = patch.qm_quick_actions_order;
        self.mm_quick_tools_order = patch.mm_quick_tools_order;
        self.ui_click_sounds = patch.ui_click_sounds;
        self.ui_sound_pack = patch.ui_sound_pack;
        self.enable_recent_history = patch.enable_recent_history;
        self.qm_mod_tabs_icon_only = patch.qm_mod_tabs_icon_only;
        self.qm_preset_confirm_mode = patch.qm_preset_confirm_mode;
        self.theme_preset = patch.theme_preset;
        self.panel_opacity = patch.panel_opacity;
        self.cursor_glow = patch.cursor_glow;
        self.blend_player_color = patch.blend_player_color;
        self.disclaimer_accepted = patch.disclaimer_accepted;
        self.show_whats_new = patch.show_whats_new;
        self.update_status = patch.update_status;
        self.system_alert = patch.system_alert;
        self.desktop_scroll_multiplier = patch.desktop_scroll_multiplier;
        self.vr_scroll_multiplier = patch.vr_scroll_multiplier;
        self.hud_spread_desktop = patch.hud_spread_desktop;
        self.hud_spread_y_desktop = patch.hud_spread_y_desktop;
        self.hud_spread_vr = patch.hud_spread_vr;
        self.hud_spread_y_vr = patch.hud_spread_y_vr;
        self.hud_alerts_scale_desktop = patch.hud_alerts_scale_desktop;
        self.hud_mic_scale_desktop = patch.hud_mic_scale_desktop;
        self.hud_notifications_scale_desktop = patch.hud_notifications_scale_desktop;
        self.hud_chatbox_scale_desktop = patch.hud_chatbox_scale_desktop;
        self.hud_alerts_scale_vr = patch.hud_alerts_scale_vr;
        self.hud_mic_scale_vr = patch.hud_mic_scale_vr;
        self.hud_notifications_scale_vr = patch.hud_notifications_scale_vr;
        self.hud_chatbox_scale_vr = patch.hud_chatbox_scale_vr;

        self.mm_width = patch.mm_width;
        self.mm_height = patch.mm_height;
        self.qm_width = patch.qm_width;
        self.qm_height = patch.qm_height;
        self.mm_step_ms = patch.mm_step_ms;
        self.mm_readback_ms = patch.mm_readback_ms;
        self.mm_upload_ms = patch.mm_upload_ms;
        self.mm_blit_ms = patch.mm_blit_ms;
        self.qm_step_ms = patch.qm_step_ms;
        self.qm_readback_ms = patch.qm_readback_ms;
        self.qm_upload_ms = patch.qm_upload_ms;
        self.qm_blit_ms = patch.qm_blit_ms;
        self.max_refresh_hz = patch.max_refresh_hz;
        self.font_scale = patch.font_scale;
        self.linear_texture = patch.linear_texture;
        self.perf_logging = patch.perf_logging;
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct UpdateStatusDto {
    pub status: String,
    pub current_version: String,
    pub latest_version: String,
    pub release_title: String,
    pub release_notes: String,
    pub release_url: String,
    pub download_url: String,
    pub error_message: String,
    pub show_modal: bool,
}

impl Default for UpdateStatusDto {
    fn default() -> Self {
        Self {
            status: String::from("idle"),
            current_version: String::from(env!("CARGO_PKG_VERSION")),
            latest_version: String::new(),
            release_title: String::new(),
            release_notes: String::new(),
            release_url: String::from("https://github.com/LensError/Lumenic-CVR-Menu/releases/latest"),
            download_url: String::from("https://github.com/LensError/Lumenic-CVR-Menu/releases/latest/download/LumenicMenu.dll"),
            error_message: String::new(),
            show_modal: false,
        }
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct SystemAlertDto {
    pub id: i64,
    pub kind: String,
    pub title: String,
    pub message: String,
    pub confirm_id: String,
}

impl Default for HostStateSnapshot {
    fn default() -> Self {
        Self {
            is_vr: false,
            user_id: String::new(),
            username: String::new(),
            user_rank: String::new(),
            cvr_version: String::new(),
            cvr_branch: String::new(),
            world_name: String::new(),
            clock_text: String::new(),
            date_text: String::new(),
            fps: 0,
            game_ping: 0,
            voice_ping: 0,
            local_pfp_key: String::from("local_pfp"),
            is_muted: false,
            is_flying: false,
            flight_allowed: false,
            is_seated: false,
            fbt_available: false,
            fbt_enabled: false,
            camera_open: false,
            mirror_open: false,
            minime_enabled: true,
            audio_master: 100,
            audio_voice: 100,
            audio_video_player: 100,
            nav_seq: 0,
            mm_open_seq: 0,
            nav_tab: String::new(),
            accent_r: 34,
            accent_g: 34,
            accent_b: 34,
            accent2_r: 228,
            accent2_g: 54,
            accent2_b: 0,
            has_player_accent: false,
            colorize_menu: false,
            use_primary_as_emission: false,
            player_color_presets: Vec::new(),
            player_color_swatches: Vec::new(),
            notifications: Vec::new(),
            featured_badge: String::new(),
            featured_badge_image_key: String::new(),
            featured_group: String::new(),
            featured_group_image_key: String::new(),
            instance_name: String::new(),
            instance_privacy: String::new(),
            instance_owner: String::new(),
            instance_locked: false,
            instance_max_props: 0,
            gs_version: String::new(),
            instance_uptime_mins: -1,
            instance_players: 1,
            online_friends: Vec::new(),
            active_worlds: Vec::new(),
            activity_feed: Vec::new(),
            uilib: UiLibSnapshot::default(),
            uix_available: false,
            mod_settings_source: String::new(),
            mod_settings_both_available: false,
            uix_categories: Vec::new(),
            content: ContentBrowserSnapshot::default(),
            social: SocialSnapshot::default(),
            cvr_settings: CvrSettingsSnapshot::default(),
            qm_quick_actions_order: Vec::new(),
            mm_quick_tools_order: Vec::new(),
            avatar_settings: AvatarSettingsSnapshot::default(),
            ui_click_sounds: true,
            ui_sound_pack: String::from("SoftSub"),
            enable_recent_history: true,
            qm_mod_tabs_icon_only: false,
            qm_preset_confirm_mode: default_qm_preset_confirm_mode(),
            theme_preset: default_theme_preset(),
            panel_opacity: 1.0,
            cursor_glow: true,
            blend_player_color: true,
            disclaimer_accepted: true,
            show_whats_new: false,
            whats_new_notes: String::new(),
            update_status: UpdateStatusDto::default(),
            system_alert: None,
            desktop_scroll_multiplier: 24.0,
            vr_scroll_multiplier: 3.0,
            hud_spread_desktop: 1.0,
            hud_spread_y_desktop: 1.0,
            hud_spread_vr: 0.85,
            hud_spread_y_vr: 0.85,
            hud_alerts_scale_desktop: 1.0,
            hud_mic_scale_desktop: 1.0,
            hud_notifications_scale_desktop: 1.0,
            hud_chatbox_scale_desktop: 1.0,
            hud_alerts_scale_vr: 1.0,
            hud_mic_scale_vr: 1.0,
            hud_notifications_scale_vr: 1.0,
            hud_chatbox_scale_vr: 1.0,
            mm_width: 2560,
            mm_height: 1440,
            qm_width: 2048,
            qm_height: 2048,
            mm_step_ms: 0.0,
            mm_readback_ms: 0.0,
            mm_upload_ms: 0.0,
            mm_blit_ms: 0.0,
            qm_step_ms: 0.0,
            qm_readback_ms: 0.0,
            qm_upload_ms: 0.0,
            qm_blit_ms: 0.0,
            max_refresh_hz: 60,
            font_scale: 1.0,
            linear_texture: false,
            perf_logging: false,
        }
    }
}

fn default_qm_preset_confirm_mode() -> String {
    String::from("sync")
}

fn default_theme_preset() -> String {
    String::from("default")
}

fn default_one() -> f32 {
    1.0
}

fn default_true() -> bool {
    true
}

