use serde::Deserialize;

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
    pub members: Vec<InstanceMemberItem>,
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

#[derive(Clone, Debug, Default, PartialEq, Eq, Deserialize)]
#[serde(default)]
pub struct PinnedContentItem {
    pub kind: String, // "avatar" | "world" | "prop"
    pub id: String,
    pub name: String,
    pub image_key: String,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Deserialize)]
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
    pub instance_region: String,
    pub instance_player_count: i32,
    pub instance_max_players: i32,
    pub world_id: String,
    pub world_name: String,
    pub world_image_key: String,
    pub loading: bool,
    pub error: String,
    pub badges: Vec<UserBadgeItem>,
    pub groups: Vec<UserGroupItem>,
    pub instance_members: Vec<InstanceMemberItem>,
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
    pub local_user_id: String,
    pub local_role: String,
    pub is_owner: bool,
    pub is_member: bool,
    pub is_featured: bool,
    pub can_manage_roles: bool,
    pub can_kick: bool,
    pub can_edit: bool,
    pub setting_listed: bool,
    pub setting_member_publicity: i32,
    pub setting_event_publicity: i32,
    pub setting_privacy_join: i32,
    pub members_page: i32,
    pub members_total_pages: i32,
    pub loading_more_members: bool,
    pub mass_role_running: bool,
    pub mass_role_done: i32,
    pub mass_role_total: i32,
    pub mass_role_failed: i32,
    pub loading: bool,
    pub saving: bool,
    pub status_msg: String,
    pub error: String,
    pub staff_scanning: bool,
    pub staff: Vec<GroupMemberItem>,
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

