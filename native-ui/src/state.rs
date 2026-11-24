use std::collections::{HashMap, HashSet, VecDeque};

use lumenic::{App, TextEditState, TextureId};
use serde::{Deserialize, Serialize};

pub const MAX_SURFACE_TEXTURES: usize = 512;

#[derive(Default)]
pub struct SurfaceTextureCache {
    pub map: HashMap<String, TextureId>,
    order: VecDeque<String>,
}

impl SurfaceTextureCache {
    pub fn get(&self, key: &str) -> Option<TextureId> {
        self.map.get(key).copied()
    }

    pub fn insert(&mut self, app: &mut App, key: String, tex: TextureId) {
        if let Some(old) = self.map.insert(key.clone(), tex) {
            app.destroy_texture(old);
            self.order.retain(|k| k != &key);
        }
        // Keep local PFP and UILib mod icons pinned so browsing content thumbnails never evicts them.
        if key != "local_pfp" && !key.starts_with("uilib:") {
            self.order.push_back(key);
        }

        while self.order.len() > MAX_SURFACE_TEXTURES {
            if let Some(oldest) = self.order.pop_front() {
                if let Some(old_tex) = self.map.remove(&oldest) {
                    app.destroy_texture(old_tex);
                }
            }
        }
    }
}

#[derive(Clone, Debug, Default, Deserialize)]
#[serde(default)]
pub struct NotificationButton {
    pub id: String,
    pub text: String,
    pub is_accept: bool,
    pub is_decline: bool,
}

#[derive(Clone, Debug, Default, Deserialize)]
#[serde(default)]
pub struct NotificationItem {
    pub id: String,
    pub title: String,
    pub body: String,
    pub category: String,
    pub buttons: Vec<NotificationButton>,
}

#[derive(Clone, Debug, Default, Deserialize)]
#[serde(default)]
pub struct OnlineFriendItem {
    pub id: String,
    pub name: String,
    pub world_name: String,
    pub status: String,
}

#[derive(Clone, Debug, Default, Deserialize)]
#[serde(default)]
pub struct ActiveWorldItem {
    pub id: String,
    pub name: String,
    pub player_count: i32,
    pub region: String,
}

#[derive(Clone, Debug, Default, Deserialize)]
#[serde(default)]
pub struct ActivityFeedItem {
    pub id: String,
    pub title: String,
    pub detail: String,
    pub time_text: String,
    pub kind: String, // "friend" | "instance" | "invite" | "system"
}

// ============================================================================
// UIExpansionKit (Mod Settings) Snapshot Types
// ============================================================================

#[derive(Clone, Debug, Default, Deserialize)]
#[serde(default)]
pub struct UixEntrySnapshot {
    pub id: String,
    pub display_name: String,
    pub description: String,
    pub kind: String, // "bool" | "slider" | "number" | "string" | "enum"
    pub bool_val: bool,
    pub num_val: f64,
    pub min_val: f64,
    pub max_val: f64,
    pub snap_val: f64,
    pub is_int: bool,
    pub string_val: String,
    pub enum_options: Vec<String>,
    pub enum_index: i32,
}

#[derive(Clone, Debug, Default, Deserialize)]
#[serde(default)]
pub struct UixCustomControlSnapshot {
    pub global_id: i32,
    pub kind: String, // "button" | "toggle" | "label" | "spacer"
    pub text: String,
    pub bool_val: bool,
}

#[derive(Clone, Debug, Default, Deserialize)]
#[serde(default)]
pub struct UixCategorySnapshot {
    pub id: String,
    pub display_name: String,
    pub entries: Vec<UixEntrySnapshot>,
    pub custom_controls: Vec<UixCustomControlSnapshot>,
}

// ============================================================================
// Phase 3B Content Browser Snapshot Types (Worlds, Avatars, Props, Search)
// ============================================================================

pub fn resolve_tile_order(saved: &[String], defaults: &[&str]) -> Vec<String> {
    let mut out = Vec::with_capacity(defaults.len());
    for id in saved {
        if defaults.contains(&id.as_str()) && !out.iter().any(|x| x == id) {
            out.push(id.clone());
        }
    }
    for &def in defaults {
        if !out.iter().any(|x| x == def) {
            out.push(String::from(def));
        }
    }
    out
}

#[derive(Clone, Debug, Default, Deserialize)]
#[serde(default)]
pub struct ContentCategoryItem {
    pub key: String,
    pub name: String,
    pub is_system: bool,
    pub is_recent: bool,
}

#[derive(Clone, Debug, Default, Deserialize)]
#[serde(default)]
pub struct ContentCardItem {
    pub id: String,
    pub name: String,
    pub image_key: String,
    pub player_count: i32,
}

#[derive(Clone, Debug, Default, Deserialize)]
#[serde(default)]
pub struct ContentListSnapshot {
    pub category_key: String,
    pub is_system: bool,
    pub page: i32,
    pub total_pages: i32,
    pub sort_order: i32,
    pub ascending: bool,
    pub loading: bool,
    pub loading_more: bool,
    pub has_more: bool,
    pub list_revision: i32,
    pub loaded_once: bool,
    pub error: String,
    pub items: Vec<ContentCardItem>,
}

#[derive(Clone, Debug, Default, Deserialize)]
#[serde(default)]
pub struct SearchResultItem {
    pub id: String,
    pub name: String,
    pub kind: String, // "world" | "avatar" | "prop" | "user"
    pub image_key: String,
}

#[derive(Clone, Debug, Default, Deserialize)]
#[serde(default)]
pub struct WorldInstanceItem {
    pub id: String,
    pub name: String,
    pub player_count: i32,
    pub max_players: i32,
    pub region: String,
}

#[derive(Clone, Debug, Default, Deserialize)]
#[serde(default)]
pub struct ContentDetailSnapshot {
    pub kind: String, // "world" | "avatar" | "prop"
    pub id: String,
    pub name: String,
    pub description: String,
    pub author_id: String,
    pub author_name: String,
    pub author_image_key: String,
    pub image_key: String,
    pub pano_url: String,
    pub uploaded_at: String,
    pub updated_at: String,
    pub file_size_text: String,
    pub platforms: Vec<String>,
    pub has_3d_preview: bool,
    pub is_current_avatar: bool,
    pub is_public: bool,
    pub is_shared: bool,
    #[serde(default = "default_true")]
    pub can_use: bool,
    pub loading: bool,
    pub error: String,
    pub categories: Vec<String>,
    pub instances: Vec<WorldInstanceItem>,
}

#[derive(Clone, Debug, Default, Deserialize)]
#[serde(default)]
pub struct InstanceMemberItem {
    pub id: String,
    pub name: String,
    pub image_key: String,
}

fn default_true() -> bool {
    true
}

#[derive(Clone, Debug, Default, Deserialize)]
#[serde(default)]
pub struct InstanceDetailSnapshot {
    pub id: String,
    pub name: String,
    pub world_id: String,
    pub world_name: String,
    pub world_image_key: String,
    pub world_author_id: String,
    pub world_author_name: String,
    pub privacy: String,
    pub region: String,
    pub game_mode_name: String,
    pub owner_id: String,
    pub owner_name: String,
    pub owner_rank: String,
    pub owner_image_key: String,
    pub group_id: String,
    pub group_name: String,
    pub group_image_key: String,
    pub group_verified: bool,
    #[serde(default = "default_true")]
    pub can_join: bool,
    pub cannot_join_reason: String,
    pub player_count: i32,
    pub max_players: i32,
    pub loading: bool,
    pub error: String,
    pub members: Vec<InstanceMemberItem>,
}

#[derive(Clone, Debug, Default, Deserialize)]
#[serde(default)]
pub struct PinnedContentItem {
    pub kind: String, // "avatar" | "world" | "prop"
    pub id: String,
    pub name: String,
    pub image_key: String,
}

#[derive(Clone, Debug, Default, Deserialize)]
#[serde(default)]
pub struct GroupModPromptSnapshot {
    pub instance_id: String,
    pub instance_name: String,
    pub world_name: String,
    pub group_id: String,
    pub group_name: String,
    pub privacy: String,
    pub is_verified: bool,
    pub moderator_count: usize,
}

#[derive(Clone, Debug, Default, Deserialize)]
#[serde(default)]
pub struct ContentBrowserSnapshot {
    pub categories_loaded: bool,
    pub pinned_items: Vec<PinnedContentItem>,
    pub world_categories: Vec<ContentCategoryItem>,
    pub avatar_categories: Vec<ContentCategoryItem>,
    pub prop_categories: Vec<ContentCategoryItem>,
    pub worlds: ContentListSnapshot,
    pub avatars: ContentListSnapshot,
    pub props: ContentListSnapshot,
    pub search_query: String,
    pub search_loading: bool,
    pub search_error: String,
    pub search_results: Vec<SearchResultItem>,
    pub discover_worlds: Vec<ContentCardItem>,
    pub discover_avatars: Vec<ContentCardItem>,
    pub discover_props: Vec<ContentCardItem>,
    pub discover_loading: bool,
    pub discover_loaded: bool,
    pub discover_error: String,
    pub creating_instance: bool,
    pub create_instance_error: String,
    pub group_mod_prompt: Option<GroupModPromptSnapshot>,
    pub instance_detail: Option<InstanceDetailSnapshot>,
    pub detail: Option<ContentDetailSnapshot>,
}

// ============================================================================
// Phase 3C Social, Instance Roster & User Details Snapshot Types
// ============================================================================

#[derive(Clone, Debug, Default, Deserialize)]
#[serde(default)]
pub struct FriendEntryItem {
    pub id: String,
    pub name: String,
    pub image_key: String,
    pub is_online: bool,
    pub is_connected: bool,
    pub instance_id: String,
    pub instance_name: String,
}

#[derive(Clone, Debug, Default, Deserialize)]
#[serde(default)]
pub struct InstancePlayerItem {
    pub id: String,
    pub name: String,
    pub rank: String,
    pub image_key: String,
    pub is_friend: bool,
    pub is_local: bool,
}

#[derive(Clone, Debug, Default, Deserialize)]
#[serde(default)]
pub struct UserBadgeItem {
    pub id: String,
    pub name: String,
    pub description: String,
    pub image_key: String,
    pub featured: bool,
    pub hidden: bool,
}

#[derive(Clone, Debug, Default, Deserialize)]
#[serde(default)]
pub struct UserGroupItem {
    pub id: String,
    pub name: String,
    pub tag: String,
    pub image_key: String,
    pub member_count: i32,
    pub is_owner: bool,
    pub is_featured: bool,
}

#[derive(Clone, Debug, Default, Deserialize)]
#[serde(default)]
pub struct UserDetailSnapshot {
    pub id: String,
    pub name: String,
    pub rank: String,
    pub pronouns: String,
    pub bio: String,
    pub note: String,
    pub note_saving: bool,
    pub note_status: String,
    pub profile_saving: bool,
    pub profile_status: String,
    pub featured_badge: String,
    pub featured_badge_image_key: String,
    pub featured_group: String,
    pub featured_group_image_key: String,
    pub image_key: String,
    pub avatar_id: String,
    pub avatar_name: String,
    pub avatar_image_key: String,
    pub has_3d_preview: bool,
    pub online_state: bool,
    pub is_connected: bool,
    pub is_friend: bool,
    pub is_blocked: bool,
    pub is_local: bool,
    pub is_muted: bool,
    pub voice_volume: i32,
    pub avatar_visibility: i32,
    pub prop_visibility: i32,
    pub instance_id: String,
    pub instance_name: String,
    pub world_name: String,
    pub loading: bool,
    pub error: String,
    pub badges: Vec<UserBadgeItem>,
    pub groups: Vec<UserGroupItem>,
}

#[derive(Clone, Debug, Default, Deserialize)]
#[serde(default)]
pub struct GroupMemberItem {
    pub id: String,
    pub name: String,
    pub role: String,
    pub image_key: String,
}

#[derive(Clone, Debug, Default, Deserialize)]
#[serde(default)]
pub struct GroupDetailSnapshot {
    pub id: String,
    pub name: String,
    pub tag: String,
    pub description: String,
    pub image_key: String,
    pub member_count: i32,
    pub owner_id: String,
    pub owner_name: String,
    pub is_owner: bool,
    pub is_member: bool,
    pub is_featured: bool,
    pub loading: bool,
    pub saving: bool,
    pub status_msg: String,
    pub error: String,
    pub members: Vec<GroupMemberItem>,
}

#[derive(Clone, Debug, Default, Deserialize)]
#[serde(default)]
pub struct RecentUserEntryItem {
    pub id: String,
    pub name: String,
    pub image_key: String,
    pub rank: String,
    pub world_name: String,
    pub instance_name: String,
    pub instance_privacy: String,
    pub last_seen_unix: i64,
    pub last_seen_text: String,
}

#[derive(Clone, Debug, Default, Deserialize)]
#[serde(default)]
pub struct SocialSnapshot {
    pub friends: Vec<FriendEntryItem>,
    pub instance_players: Vec<InstancePlayerItem>,
    pub recent_users: Vec<RecentUserEntryItem>,
    pub my_groups: Vec<UserGroupItem>,
    pub user_detail: Option<UserDetailSnapshot>,
    pub group_detail: Option<GroupDetailSnapshot>,
}

// ============================================================================
// Phase 3D CVR Settings Snapshot
// ============================================================================

#[derive(Clone, Debug, Default, Deserialize)]
#[serde(default)]
pub struct CvrSettingItem {
    pub key: String,
    pub label: String,
    pub tooltip: String,
    pub category: String,
    pub group: String,
    pub kind: String, // "bool" | "int_slider" | "int_choice" | "int_step" | "float_step" | "str_choice" | "string_ro"
    pub bool_val: bool,
    pub int_val: i32,
    pub float_val: f32,
    pub str_val: String,
    pub min_val: i32,
    pub max_val: i32,
    pub unit: String,
    pub choice_vals: Vec<String>,
    pub choice_labels: Vec<String>,
}

#[derive(Clone, Debug, Default, Deserialize)]
#[serde(default)]
pub struct AvatarSettingEntry {
    pub name: String,
    pub param: String,
    pub kind: String,
    pub val_x: f32,
    pub val_y: f32,
    pub val_z: f32,
    pub min_x: f32,
    pub max_x: f32,
    pub min_y: f32,
    pub max_y: f32,
    pub min_z: f32,
    pub max_z: f32,
    pub options: Vec<String>,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(default)]
pub struct AvatarSettingsSnapshot {
    pub auto_apply: bool,
    pub has_staged: bool,
    pub current_profile: String,
    pub staged_profile: String,
    pub avatar_height: f32,
    pub initial_height: f32,
    pub scale_min: f32,
    pub scale_max: f32,
    pub scaling_allowed: bool,
    pub entries: Vec<AvatarSettingEntry>,
    pub profiles: Vec<String>,
}

impl Default for AvatarSettingsSnapshot {
    fn default() -> Self {
        Self {
            auto_apply: true,
            has_staged: false,
            current_profile: String::new(),
            staged_profile: String::new(),
            avatar_height: 1.6,
            initial_height: 1.6,
            scale_min: 0.2,
            scale_max: 5.0,
            scaling_allowed: true,
            entries: Vec::new(),
            profiles: Vec::new(),
        }
    }
}

#[derive(Clone, Debug, Default, Deserialize)]
#[serde(default)]
pub struct CvrSettingsSnapshot {
    pub items: Vec<CvrSettingItem>,
}

// ============================================================================
// UILib Mod Compatibility Snapshot Types
// ============================================================================

#[derive(Clone, Debug, Default, Deserialize)]
#[serde(default)]
pub struct UiLibTab {
    pub id: String,
    pub label: String,
    pub mod_name: String,
    pub icon_key: String,
    pub active: bool,
    pub kind: String, // "home" | "players" | "props" | "mod"
}

#[derive(Clone, Debug, Default, Deserialize)]
#[serde(default)]
pub struct UiLibDetailBlock {
    pub header: String,
    pub text: String,
    pub badge: bool,
}

#[derive(Clone, Debug, Default, Deserialize)]
#[serde(default)]
pub struct UiLibExtraButton {
    pub id: String,
    pub label: String,
    pub icon_key: String,
    pub tooltip: String,
    pub disabled: bool,
}

#[derive(Clone, Debug, Default, Deserialize)]
#[serde(default)]
pub struct UiLibElement {
    pub id: String,
    pub kind: String, // "button" | "toggle" | "slider" | "cycle" | "text_block" | "text_input" | "content_button" | "content_details" | "spacer" | "custom"
    pub label: String,
    pub tooltip: String,
    pub icon_key: String,
    pub sub_icon_key: String,
    pub disabled: bool,
    pub bool_val: bool,
    pub float_val: f32,
    pub min_val: f32,
    pub max_val: f32,
    pub default_val: f32,
    pub allow_reset: bool,
    pub decimals: i32,
    pub has_hold: bool,
    pub col_span: i32,
    pub style: String,
    pub text_val: String,
    pub placeholder: String,
    pub lines: Vec<String>,
    pub detail_blocks: Vec<UiLibDetailBlock>,
    pub extra_buttons: Vec<UiLibExtraButton>,
}

#[derive(Clone, Debug, Default, Deserialize)]
#[serde(default)]
pub struct UiLibCategory {
    pub id: String,
    pub name: String,
    pub show_header: bool,
    pub can_collapse: bool,
    pub collapsed: bool,
    pub header_buttons: Vec<UiLibElement>,
    pub elements: Vec<UiLibElement>,
}

#[derive(Clone, Debug, Default, Deserialize)]
#[serde(default)]
pub struct UiLibPageSnapshot {
    pub id: String,
    pub title: String,
    pub subtitle: String,
    pub mod_name: String,
    pub can_go_back: bool,
    pub breadcrumbs: Vec<String>,
    pub colour_preview_hex: String,
    pub player_preview_status: String,
    pub selected_player_id: String,
    pub player_bio: String,
    pub header_buttons: Vec<UiLibElement>,
    pub top_sliders: Vec<UiLibElement>,
    pub categories: Vec<UiLibCategory>,
}

#[derive(Clone, Debug, Default, Deserialize)]
#[serde(default)]
pub struct UiLibModalSnapshot {
    pub kind: String, // "confirm" | "notice" | "multi_select" | "number_input"
    pub title: String,
    pub body: String,
    pub yes_text: String,
    pub no_text: String,
    pub ok_text: String,
    pub options: Vec<String>,
    pub selected_index: i32,
    pub number_value: f32,
    pub revision: u64,
}

#[derive(Clone, Debug, Default, Deserialize)]
#[serde(default)]
pub struct UiLibSnapshot {
    pub tabs: Vec<UiLibTab>,
    pub active_page: Option<UiLibPageSnapshot>,
    pub modal: Option<UiLibModalSnapshot>,
    pub toast: String,
}

#[derive(Clone, Debug, Default, Deserialize)]
#[serde(default)]
pub struct PlayerColorPresetItem {
    pub id: i32,
    pub name: String,
    pub r1: u8,
    pub g1: u8,
    pub b1: u8,
    pub r2: u8,
    pub g2: u8,
    pub b2: u8,
}

#[derive(Clone, Debug, Default, Deserialize)]
#[serde(default)]
pub struct PlayerColorSwatchItem {
    pub id: i32,
    pub name: String,
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

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

    // UIExpansionKit Mod Settings bridge
    pub uix_available: bool,
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

    // Diagnostics & surface info
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
    pub linear_texture: bool,
    pub perf_logging: bool,
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
            uix_categories: Vec::new(),
            content: ContentBrowserSnapshot::default(),
            social: SocialSnapshot::default(),
            cvr_settings: CvrSettingsSnapshot::default(),
            qm_quick_actions_order: Vec::new(),
            mm_quick_tools_order: Vec::new(),
            avatar_settings: AvatarSettingsSnapshot::default(),
            ui_click_sounds: true,
            ui_sound_pack: String::from("Glass"),
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
            linear_texture: false,
            perf_logging: false,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum MainNavEntry {
    Tab(String),
    ContentDetail {
        tab: String,
        kind: String,
        id: String,
    },
    InstanceDetail {
        tab: String,
        instance_id: String,
    },
    UserDetail {
        tab: String,
        user_id: String,
        detail_tab: usize,
    },
    GroupDetail {
        tab: String,
        group_id: String,
    },
}

/// Local UI state for the Phase 3A Main Menu Shell & Home Dashboard.
pub struct MainMenuState {
    pub active_tab: String,
    pub nav_history: Vec<MainNavEntry>,
    pub info_tab: usize,    // 0 = Game Info, 1 = Instance Info
    pub info_modal_open: bool,
    pub leaflet_tab: usize, // 0 = Online Friends, 1 = Active Worlds
    pub quit_confirm: bool,
    pub go_home_confirm: bool,
    pub safety_confirm_action: String,
    pub group_edit_id: String,
    pub group_name_input: TextEditState,
    pub group_desc_input: TextEditState,
    pub search_input: TextEditState,
    pub last_search: String,
    pub mm_root_rect: Option<[f32; 4]>,
    pub uix_selected_category: String,
    pub uix_search_input: TextEditState,
    pub uix_sliders: HashMap<String, (f32, f32)>,
    pub uix_text_inputs: HashMap<String, (TextEditState, String)>,
    pub browser_local_filter: TextEditState,
    pub cb_last_revisions: HashMap<String, i32>,
    pub cb_load_more_requested_page: HashMap<String, (i32, i32)>,
    pub cb_auto_loaded: HashSet<String>,
    pub search_filter_kind: String,
    pub create_inst_privacy: String,
    pub create_inst_region: String,
    pub create_inst_group_id: String,
    pub friends_filter: usize, // 0 = Online, 1 = All, 2 = Offline
    pub friends_search_input: TextEditState,
    pub user_detail_tab: usize, // 0 = Overview, 1 = Badges, 2 = Groups
    pub user_note_user_id: String,
    pub user_note_input: TextEditState,
    pub profile_edit_user_id: String,
    pub profile_pronouns_input: TextEditState,
    pub profile_bio_input: TextEditState,
    pub profile_editing: bool,
    pub discover_tab: usize,    // 0 = Random Worlds, 1 = Random Avatars, 2 = Random Props
    pub messages_filter: String,
    pub settings_tab: usize,
    pub settings_search_input: TextEditState,
    pub settings_sliders: HashMap<String, (f32, i32)>,
    pub avatar_sliders: HashMap<String, (f32, f32)>,
    pub aas_profile_edit_mode: bool,
    pub aas_new_profile_input: TextEditState,
    pub aas_preview_rect: Option<[f32; 4]>,
    pub aas_preview_dragging: bool,
    pub aas_last_preview_sync: Option<[f32; 4]>,
    pub aas_preview_target: String,
    pub aas_preview_rendered: bool,
    pub world_pano_enabled: bool,
    pub last_nav_seq: u32,
    pub hovered_tooltip: String,
    pub mm_tools_reorder_mode: bool,
    pub mm_tools_reorder_selected: Option<usize>,
}

impl Default for MainMenuState {
    fn default() -> Self {
        Self {
            active_tab: String::from("home"),
            nav_history: Vec::new(),
            info_tab: 0,
            info_modal_open: false,
            leaflet_tab: 0,
            quit_confirm: false,
            go_home_confirm: false,
            safety_confirm_action: String::new(),
            group_edit_id: String::new(),
            group_name_input: TextEditState::new(),
            group_desc_input: TextEditState::new(),
            search_input: TextEditState::new(),
            last_search: String::new(),
            mm_root_rect: None,
            uix_selected_category: String::new(),
            uix_search_input: TextEditState::new(),
            uix_sliders: HashMap::new(),
            uix_text_inputs: HashMap::new(),
            browser_local_filter: TextEditState::new(),
            cb_last_revisions: HashMap::new(),
            cb_load_more_requested_page: HashMap::new(),
            cb_auto_loaded: HashSet::new(),
            search_filter_kind: String::from("all"),
            create_inst_privacy: String::from("Friends"),
            create_inst_region: String::from("Eu"),
            create_inst_group_id: String::new(),
            friends_filter: 0,
            friends_search_input: TextEditState::new(),
            user_detail_tab: 0,
            user_note_user_id: String::new(),
            user_note_input: TextEditState::new(),
            profile_edit_user_id: String::new(),
            profile_pronouns_input: TextEditState::new(),
            profile_bio_input: TextEditState::new(),
            profile_editing: false,
            discover_tab: 0,
            messages_filter: String::from("all"),
            settings_tab: 0,
            settings_search_input: TextEditState::new(),
            settings_sliders: HashMap::new(),
            avatar_sliders: HashMap::new(),
            aas_profile_edit_mode: false,
            aas_new_profile_input: TextEditState::new(),
            aas_preview_rect: None,
            aas_preview_dragging: false,
            aas_last_preview_sync: None,
            aas_preview_target: String::new(),
            aas_preview_rendered: false,
            world_pano_enabled: true,
            last_nav_seq: 0,
            hovered_tooltip: String::new(),
            mm_tools_reorder_mode: false,
            mm_tools_reorder_selected: None,
        }
    }
}

/// Local UI state for the Quick Menu & UILib bridge.
pub struct QuickMenuState {
    pub go_home_confirm: bool,
    pub master_slider: f32,
    pub voice_slider: f32,
    pub video_slider: f32,
    pub last_host_master: i32,
    pub last_host_voice: i32,
    pub last_host_video: i32,
    /// Per-element UILib slider shadow values (element_id -> (current, last_synced_host)).
    pub uilib_sliders: HashMap<String, (f32, f32)>,
    /// Number input modal state.
    pub modal_number_edit: TextEditState,
    pub modal_number_revision: u64,
    pub modal_number_error: String,
    /// Last synced 3D Studio Box target ("none" | "home_minime" | "player_details") and quad bounds.
    pub last_studio_target: String,
    pub last_studio_bounds: [f32; 7],
    /// Logical [x, y, w, h] of the transparent 3D studio chamber so the menu-wide background shader
    /// punches a clean transparent cutout behind MiniMe / Player 3D Avatar.
    pub studio_cutout_rect: Option<[f32; 4]>,
    /// Logical [x, y, w, h] of the Quick Menu root container for framing the menu-wide glow shader.
    pub qm_root_rect: Option<[f32; 4]>,
    /// Whether the clickable FPS / System Info telemetry modal popout is open.
    pub show_info_modal: bool,
    /// Logical [x, y, w, h] of the bottom UILib tab strip hscroll for touch-drag panning.
    pub tab_strip_rect: Option<[f32; 4]>,
    /// Accumulated horizontal drag distance on the tab strip (used in preview/WASM mode).
    pub tab_strip_drag_dist: f32,
    pub avatar_preview_rect: Option<[f32; 4]>,
    pub avatar_preview_dragging: bool,
    pub last_avatar_preview_sync: Option<[f32; 4]>,
    pub avatar_preview_target: String,
    pub avatar_preview_rendered: bool,
    pub hovered_tooltip: String,
    pub qm_reorder_mode: bool,
    pub qm_reorder_selected: Option<usize>,
    pub qm_pinned_filter: usize, // 0 = All, 1 = Avatars, 2 = Worlds, 3 = Props
    pub qm_pinned_edit: bool,
    pub qm_world_modal: Option<(String, String)>, // (world_id, world_name)
    pub qm_world_privacy: String,
    pub qm_world_region: String,
    pub qm_world_group_id: String,
}

impl Default for QuickMenuState {
    fn default() -> Self {
        Self {
            go_home_confirm: false,
            master_slider: 100.0,
            voice_slider: 100.0,
            video_slider: 100.0,
            last_host_master: 100,
            last_host_voice: 100,
            last_host_video: 100,
            uilib_sliders: HashMap::new(),
            modal_number_edit: TextEditState::new(),
            modal_number_revision: 0,
            modal_number_error: String::new(),
            last_studio_target: String::new(),
            last_studio_bounds: [0.0; 7],
            studio_cutout_rect: None,
            qm_root_rect: None,
            show_info_modal: false,
            tab_strip_rect: None,
            tab_strip_drag_dist: 0.0,
            avatar_preview_rect: None,
            avatar_preview_dragging: false,
            last_avatar_preview_sync: None,
            avatar_preview_target: String::new(),
            avatar_preview_rendered: false,
            hovered_tooltip: String::new(),
            qm_reorder_mode: false,
            qm_reorder_selected: None,
            qm_pinned_filter: 0,
            qm_pinned_edit: false,
            qm_world_modal: None,
            qm_world_privacy: String::from("Friends"),
            qm_world_region: String::from("Eu"),
            qm_world_group_id: String::new(),
        }
    }
}

/// Complete pure-Rust UI state shared between Main Menu and Quick Menu renderers.
#[derive(Default)]
pub struct AppState {
    pub host: HostStateSnapshot,
    pub main_menu: MainMenuState,
    pub quick: QuickMenuState,
    pub main_textures: SurfaceTextureCache,
    pub quick_textures: SurfaceTextureCache,
}

/// Typed actions emitted by Rust UI and executed by C# on Unity's main thread.
#[derive(Clone, Debug, Serialize)]
#[serde(tag = "type")]
pub enum UiAction {
    SwitchToVanilla { target: String },
    TogglePerfLogging,
    ToggleLinearTexture,
    DummyPing { source: String, count: u32 },
    PlayUiSound { sound: String },
    ToggleMute,
    ToggleFlight,
    ToggleSeatedOrRecalibrate,
    ToggleFullBody,
    ToggleCamera,
    ToggleMirror,
    Respawn,
    GoHome,
    OpenTextChat,
    OpenMainMenuPage { page: String },
    OpenVanillaPage { page: String },
    OpenLocalPlayerQmPage,
    OpenPlayerInMainMenu,
    RemoveMyProps,
    TogglePropDeleteMode,
    RemoveAllProps,
    RemoveAllAvatars,
    RecalibrateAvatar,
    SwitchVrMode,
    ToggleCursorLock,
    MediaPrev,
    MediaPlayPause,
    MediaNext,
    QuitApplication,
    SetAudioVolume { key: String, value: i32 },
    NotificationButtonClick { button_id: String },
    NotificationDismiss { message_id: String },
    OpenVirtualKeyboard {
        surface_id: u32,
        field_id: String,
        initial_text: String,
        title: String,
        placeholder: String,
        max_len: i32,
        hidden: bool,
    },

    // UILib Mod Compatibility actions
    UiLibSelectTab { tab_id: String },
    UiLibGoBack,
    UiLibToggleCategory { category_id: String },
    UiLibClick { element_id: String },
    UiLibHold { element_id: String },
    UiLibToggle { element_id: String, value: bool },
    UiLibSlider { element_id: String, value: f32, reset: bool },
    UiLibTextInput { element_id: String },
    UiLibModalConfirm { accepted: bool },
    UiLibModalNoticeOk,
    UiLibModalSelectOption { index: i32 },
    UiLibModalNumberSubmit { value: f32 },
    UiLibModalDismiss,

    // UIExpansionKit Mod Settings actions
    RefreshUixSettings,
    UixSetBool {
        category: String,
        entry: String,
        value: bool,
    },
    UixSetNumber {
        category: String,
        entry: String,
        value: f64,
        save: bool,
    },
    UixSetString {
        category: String,
        entry: String,
        value: String,
    },
    UixSetEnum {
        category: String,
        entry: String,
        index: i32,
    },
    UixCustomButton {
        category: String,
        global_id: i32,
    },
    UixCustomToggle {
        category: String,
        global_id: i32,
        value: bool,
    },

    // Phase 3B Content Browser actions
    ContentLoadList {
        kind: String,
        category_key: String,
        is_system: bool,
        page: i32,
        sort_order: i32,
        ascending: bool,
    },
    ContentLoadMore {
        kind: String,
    },
    ContentLoadDetail {
        kind: String,
        id: String,
    },
    ContentCloseDetail,
    ContentGlobalSearch {
        query: String,
    },
    ContentWearAvatar {
        avatar_id: String,
    },
    ContentSpawnProp {
        prop_id: String,
    },
    ContentJoinInstance {
        instance_id: String,
    },
    ContentDropPortal {
        instance_id: String,
    },
    ContentDropCurrentInstancePortal,
    ContentLoadInstanceDetail {
        instance_id: String,
    },
    ContentCloseInstanceDetail,
    ContentToggleCategory {
        kind: String,
        id: String,
        category_key: String,
    },
    ContentCreateInstance {
        world_id: String,
        privacy: String,
        region: String,
        group_id: String,
    },
    ContentResolveGroupModPrompt {
        accept: bool,
        never_show: bool,
    },
    ContentSetHomeWorld {
        world_id: String,
    },
    ContentTogglePin {
        kind: String,
        id: String,
    },
    ContentUnpin {
        kind: String,
        id: String,
    },
    QmOpenWorldDetails {
        world_id: String,
    },

    // Phase 3C Social & User Details actions
    SocialRefreshFriends,
    SocialLoadUserDetail {
        user_id: String,
    },
    SocialSaveUserNote {
        user_id: String,
        note: String,
    },
    SocialUpdateOwnProfile {
        pronouns: String,
        bio: String,
        update_pronouns: bool,
        update_bio: bool,
    },
    SetColorizeMenu {
        enabled: bool,
    },
    SetPlayerColorPreset {
        preset_id: i32,
    },
    SetPlayerColorPrimary {
        swatch_id: i32,
    },
    SetPlayerColorSecondary {
        swatch_id: i32,
    },
    SetPlayerColorEmission {
        enabled: bool,
    },
    SocialCloseUserDetail,
    SocialInvitePlayer {
        user_id: String,
    },
    SocialRequestInvite {
        user_id: String,
    },
    SocialSendFriendRequest {
        user_id: String,
    },
    SocialUnfriend {
        user_id: String,
    },
    SocialSetBlocked {
        user_id: String,
        blocked: bool,
    },
    SocialSetPlayerMute {
        user_id: String,
        muted: bool,
    },
    SocialSetPlayerVolume {
        user_id: String,
        volume: i32,
    },
    SocialSetAvatarVisibility {
        user_id: String,
        mode: i32,
    },
    SocialSetPropVisibility {
        user_id: String,
        mode: i32,
    },
    SocialReloadPlayerAvatar {
        user_id: String,
    },
    SocialKickUser {
        user_id: String,
    },
    SocialSetFeaturedBadge {
        badge_id: String,
        featured: bool,
    },
    SocialSetFeaturedGroup {
        group_id: String,
    },
    SocialLoadGroupDetail {
        group_id: String,
    },
    SocialCloseGroupDetail,
    SocialUpdateGroupName {
        group_id: String,
        name: String,
    },
    SocialUpdateGroupDesc {
        group_id: String,
        description: String,
    },
    SocialKickGroupMember {
        group_id: String,
        target_id: String,
    },
    SocialLeaveGroup {
        group_id: String,
    },
    SocialDeleteGroup {
        group_id: String,
    },

    // Phase 3D Settings & Discover actions
    ContentLoadDiscover,
    CvrSetBool {
        key: String,
        value: bool,
    },
    CvrSetInt {
        key: String,
        value: i32,
    },
    CvrSetFloat {
        key: String,
        value: f32,
    },
    CvrSetString {
        key: String,
        value: String,
    },
    ToggleUiClickSounds,
    SetUiSoundPack {
        pack: String,
    },
    SetMaxRefreshHz {
        hz: i32,
    },
    SetTileOrder {
        target: String,
        order: Vec<String>,
    },

    // Advanced Avatar Settings & Preview actions
    AvatarSetParam {
        param: String,
        value: f32,
    },
    AvatarSetAutoApply {
        enabled: bool,
    },
    AvatarApplyStaged,
    AvatarDiscardStaged,
    AvatarLoadProfile {
        profile: String,
    },
    AvatarSaveProfile {
        profile: String,
    },
    AvatarDeleteProfile {
        profile: String,
    },
    AvatarReorderProfiles {
        profiles: Vec<String>,
    },
    AvatarResetDefaults,
    AvatarSetHeight {
        height: f32,
    },
    AvatarResetHeight,
    AvatarPreviewRotate {
        delta: f32,
        is_dragging: bool,
    },
    WorldPanoLook {
        delta_x: f32,
        delta_y: f32,
        is_dragging: bool,
    },
    SyncAvatarPreview {
        surface_id: u32,
        active: bool,
        target: String,
        x: f32,
        y: f32,
        w: f32,
        h: f32,
    },

    SyncStudioStage {
        target: String,
        local_x: f32,
        local_y: f32,
        height_units: f32,
        min_x: f32,
        max_x: f32,
        min_y: f32,
        max_y: f32,
    },
}
