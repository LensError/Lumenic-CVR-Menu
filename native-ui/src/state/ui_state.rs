use std::collections::{HashMap, HashSet};

use lumenic::TextEditState;

use super::actions::UiAction;
use super::host::HostStateSnapshot;
use super::textures::SurfaceTextureCache;

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

#[derive(Debug, Clone)]
pub struct DropdownOption {
    pub label: String,
    pub action: UiAction,
    pub selected: bool,
}

#[derive(Debug, Clone)]
pub struct ActiveDropdown {
    pub title: String,
    pub options: Vec<DropdownOption>,
}

/// Local UI state for the Phase 3A Main Menu Shell & Home Dashboard.
pub struct MainMenuState {
    pub active_tab: String,
    pub last_open_seq: u32,
    pub nav_history: Vec<MainNavEntry>,
    pub info_tab: usize,    // 0 = Game Info, 1 = Instance Info
    pub info_modal_open: bool,
    pub leaflet_tab: usize, // 0 = Online Friends, 1 = Active Worlds
    pub quit_confirm: bool,
    pub go_home_confirm: bool,
    pub safety_confirm_action: String,
    pub group_edit_id: String,
    pub group_detail_tab: usize, // 0 = Overview, 1 = Members, 2 = Manage (staff only)
    pub group_manage_members_mode: bool,
    pub group_name_input: TextEditState,
    pub group_desc_input: TextEditState,
    pub group_member_search: TextEditState,
    pub group_member_role_filter: String,
    pub group_selected_members: HashSet<String>,
    pub group_mass_confirm: bool,
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
    pub active_dropdown: Option<ActiveDropdown>,
}

impl Default for MainMenuState {
    fn default() -> Self {
        Self {
            active_tab: String::from("home"),
            last_open_seq: 0,
            nav_history: Vec::new(),
            info_tab: 0,
            info_modal_open: false,
            leaflet_tab: 0,
            quit_confirm: false,
            go_home_confirm: false,
            safety_confirm_action: String::new(),
            group_edit_id: String::new(),
            group_detail_tab: 0,
            group_manage_members_mode: false,
            group_name_input: TextEditState::new(),
            group_desc_input: TextEditState::new(),
            group_member_search: TextEditState::new(),
            group_member_role_filter: String::from("all"),
            group_selected_members: HashSet::new(),
            group_mass_confirm: false,
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
            active_dropdown: None,
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

pub struct ExtTextInputSlot {
    pub edit: TextEditState,
    pub last_host: String,
    pub last_sent: String,
    pub was_focused: bool,
}

impl ExtTextInputSlot {
    pub fn new(initial: &str) -> Self {
        Self {
            edit: TextEditState::with_text(initial),
            last_host: initial.to_owned(),
            last_sent: initial.to_owned(),
            was_focused: false,
        }
    }
}

/// Complete pure-Rust UI state shared between Main Menu and Quick Menu renderers.
#[derive(Default)]
pub struct AppState {
    pub host: HostStateSnapshot,
    pub extensions: crate::ext::ExtensionsSnapshot,
    pub main_menu: MainMenuState,
    pub quick: QuickMenuState,
    pub main_textures: SurfaceTextureCache,
    pub quick_textures: SurfaceTextureCache,
    pub ext_sliders: HashMap<String, (f32, f32, f32)>,
    pub ext_text_inputs: HashMap<String, ExtTextInputSlot>,
}

