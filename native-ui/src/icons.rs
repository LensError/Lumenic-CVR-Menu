//! Official Google Material Symbols Sharp (Filled) icon font (`https://fonts.google.com/icons`).
//! Embedded directly via `lumenic::register_font_bytes` (ISSUE-054) - zero SVG converters or triangulation.
//! To swap any icon, simply change its string to any icon name from https://fonts.google.com/icons.

use std::sync::Once;

use lumenic::{StyleBuilder, UiContext};

pub const MATERIAL_FONT_FAMILY: &str = "Material Symbols Sharp Filled";
pub const MATERIAL_FONT_BYTES: &[u8] = include_bytes!("../assets/MaterialSymbolsSharp.ttf");

/// Registers the embedded Material Symbols Sharp Filled font once with Lumenic's font system.
pub fn register_material_font() {
    static INIT: Once = Once::new();
    INIT.call_once(|| {
        lumenic::register_font(MATERIAL_FONT_FAMILY, MATERIAL_FONT_BYTES);
    });
}

/// Renders an official Google Material Symbols Sharp (Filled) icon by ligature name.
pub fn icon<'a>(ui: &'a mut UiContext, name: &str, size: f32) -> StyleBuilder<'a> {
    register_material_font();
    ui.txt(name).font(MATERIAL_FONT_FAMILY).sz(size)
}

pub const MIC: &str = "mic";
pub const MIC_OFF: &str = "mic_off";
pub const FLIGHT: &str = "flight";
pub const SEATED: &str = "airline_seat_recline_normal";
pub const CAMERA: &str = "photo_camera";
pub const MIRROR: &str = "flip";
pub const RESPAWN: &str = "restart_alt";
pub const HOME: &str = "home";
pub const CHAT: &str = "chat";
pub const FBT: &str = "accessibility_new";
pub const BELL: &str = "notifications";
pub const VOLUME: &str = "volume_up";
pub const CLOSE: &str = "close";
pub const RELOAD: &str = "refresh";
pub const BACK: &str = "chevron_left";
pub const CHEVRON_DOWN: &str = "expand_more";
pub const CHEVRON_RIGHT: &str = "chevron_right";
pub const PLAYERS: &str = "group";
pub const CUBE: &str = "deployed_code";
pub const PUZZLE: &str = "extension";
pub const CHECK: &str = "check";
pub const WORLD: &str = "public";
pub const AVATAR: &str = "person";
pub const INSTANCE: &str = "location_on";
pub const SETTINGS: &str = "settings";
pub const BOLT: &str = "bolt";
pub const TOOLBOX: &str = "handyman";
pub const SEARCH: &str = "search";
pub const MAIL: &str = "mail";
pub const COMPASS: &str = "explore";
pub const LOGOUT: &str = "logout";
pub const PLAY_PAUSE: &str = "play_pause";
pub const PREV: &str = "skip_previous";
pub const NEXT: &str = "skip_next";
pub const CURSOR: &str = "arrow_selector_tool";
pub const TRASH: &str = "delete";
pub const VR_HEADSET: &str = "vrpano";
pub const STAR: &str = "star";
pub const SLIDERS: &str = "tune";
pub const INFO: &str = "info";
pub const EDIT: &str = "edit";
pub const SAVE: &str = "save";
pub const ADD: &str = "add";
pub const ARROW_UP: &str = "keyboard_arrow_up";
pub const ARROW_DOWN: &str = "keyboard_arrow_down";
pub const PIN: &str = "keep";
pub const APPS: &str = "apps";
