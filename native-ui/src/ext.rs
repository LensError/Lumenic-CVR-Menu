use std::collections::HashMap;

use serde::Deserialize;

fn default_one() -> f32 {
    1.0
}

/// Global theme, typography, density, and surface overrides pushed by extensions via `LumenicApi.Theme`.
#[derive(Clone, Debug, Deserialize)]
#[serde(default)]
pub struct ExtThemeOverride {
    /// Optional absolute override for font scale (replaces user's built-in font_scale).
    pub font_scale_override: Option<f32>,
    /// Multiplier applied on top of effective font scale (default 1.0).
    #[serde(default = "default_one")]
    pub font_scale_mul: f32,
    #[serde(default = "default_one")]
    pub title_scale: f32,
    #[serde(default = "default_one")]
    pub body_scale: f32,
    #[serde(default = "default_one")]
    pub caption_scale: f32,
    #[serde(default = "default_one")]
    pub icon_scale: f32,
    #[serde(default = "default_one")]
    pub spacing_scale: f32,
    #[serde(default = "default_one")]
    pub padding_scale: f32,
    #[serde(default = "default_one")]
    pub radius_scale: f32,
    #[serde(default = "default_one")]
    pub bevel_scale: f32,
    pub tile_min_height: Option<f32>,
    pub qm_action_columns: Option<usize>,
    #[serde(default = "default_one")]
    pub glow_intensity: f32,
    #[serde(default = "default_one")]
    pub shadow_strength: f32,
    #[serde(default = "default_one")]
    pub panel_opacity: f32,

    // Optional hex color overrides ("#RRGGBB" or "#RRGGBBAA")
    pub card_bg: Option<String>,
    pub card_inner_bg: Option<String>,
    pub tile_idle_bg: Option<String>,
    pub tile_idle_hover: Option<String>,
    pub border_subtle: Option<String>,
    pub text_primary: Option<String>,
    pub text_muted: Option<String>,
    pub text_dim: Option<String>,
    pub accent_override: Option<String>,
}

impl Default for ExtThemeOverride {
    fn default() -> Self {
        Self {
            font_scale_override: None,
            font_scale_mul: 1.0,
            title_scale: 1.0,
            body_scale: 1.0,
            caption_scale: 1.0,
            icon_scale: 1.0,
            spacing_scale: 1.0,
            padding_scale: 1.0,
            radius_scale: 1.0,
            bevel_scale: 1.0,
            tile_min_height: None,
            qm_action_columns: None,
            glow_intensity: 1.0,
            shadow_strength: 1.0,
            panel_opacity: 1.0,
            card_bg: None,
            card_inner_bg: None,
            tile_idle_bg: None,
            tile_idle_hover: None,
            border_subtle: None,
            text_primary: None,
            text_muted: None,
            text_dim: None,
            accent_override: None,
        }
    }
}

/// Section-scoped style & layout overrides for well-known built-in section IDs.
#[derive(Clone, Debug, Default, Deserialize)]
#[serde(default)]
pub struct ExtStyleOverride {
    pub hidden: bool,
    pub font_scale: Option<f32>,
    pub spacing_scale: Option<f32>,
    pub padding_scale: Option<f32>,
    pub columns: Option<usize>,
    pub min_height: Option<f32>,
    pub card_bg: Option<String>,
    pub border: Option<String>,
}

/// An injected node tree mounted relative to a well-known section ID.
#[derive(Clone, Debug, Default, Deserialize)]
#[serde(default)]
pub struct ExtSlotEntry {
    pub ext_id: String,
    pub slot_id: String,
    pub priority: i32,
    pub root: ExtNode,
}

/// Combined overrides, injections, and optional replacement for a named built-in section.
#[derive(Clone, Debug, Default, Deserialize)]
#[serde(default)]
pub struct ExtSectionPatch {
    pub style: Option<ExtStyleOverride>,
    pub replace: Option<ExtSlotEntry>,
    pub before: Vec<ExtSlotEntry>,
    pub after: Vec<ExtSlotEntry>,
    pub inside_start: Vec<ExtSlotEntry>,
    pub inside_end: Vec<ExtSlotEntry>,
}

/// Extension tab registered in Quick Menu or Main Menu.
#[derive(Clone, Debug, Default, Deserialize)]
#[serde(default)]
pub struct ExtTabSnapshot {
    pub ext_id: String,
    pub id: String,
    pub label: String,
    /// Material Symbols icon ligature name (e.g. "smart_display").
    pub icon: String,
    /// Optional uploaded texture key (takes precedence over Material icon when loaded).
    pub icon_key: String,
    pub badge: String,
    pub priority: i32,
}

/// Active rendered extension page for Quick Menu or Main Menu.
#[derive(Clone, Debug, Default, Deserialize)]
#[serde(default)]
pub struct ExtPageSnapshot {
    pub ext_id: String,
    pub tab_id: String,
    pub page_id: String,
    pub title: String,
    pub subtitle: String,
    pub can_go_back: bool,
    /// When true, omits the default Lumenic page header card so the extension owns the entire viewport.
    pub hide_header: bool,
    pub header_actions: Vec<ExtNode>,
    pub root: ExtNode,
}

/// Active custom modal opened by an extension on Quick Menu or Main Menu.
#[derive(Clone, Debug, Default, Deserialize)]
#[serde(default)]
pub struct ExtModalSnapshot {
    pub ext_id: String,
    pub modal_id: String,
    pub title: String,
    pub width: f32,
    pub close_on_backdrop: bool,
    pub root: ExtNode,
}

/// Complete extension payload pushed via `lumenic_cvr_push_extensions_json`.
#[derive(Clone, Debug, Default, Deserialize)]
#[serde(default)]
pub struct ExtensionsSnapshot {
    pub theme: ExtThemeOverride,
    pub qm_tabs: Vec<ExtTabSnapshot>,
    pub mm_tabs: Vec<ExtTabSnapshot>,
    pub qm_active_page: Option<ExtPageSnapshot>,
    pub mm_active_page: Option<ExtPageSnapshot>,
    pub qm_modal: Option<ExtModalSnapshot>,
    pub mm_modal: Option<ExtModalSnapshot>,
    pub hidden_tiles: Vec<String>,
    pub sections: HashMap<String, ExtSectionPatch>,
}

/// Full styling properties for any declarative `ExtNode`.
#[derive(Clone, Debug, Default, Deserialize)]
#[serde(default)]
pub struct ExtNodeStyle {
    pub w: Option<f32>,
    pub h: Option<f32>,
    pub w_frac: Option<f32>,
    pub h_frac: Option<f32>,
    pub min_w: Option<f32>,
    pub max_w: Option<f32>,
    pub min_h: Option<f32>,
    pub max_h: Option<f32>,
    pub grow: Option<f32>,

    pub p: Option<f32>,
    pub px: Option<f32>,
    pub py: Option<f32>,
    pub pt: Option<f32>,
    pub pb: Option<f32>,
    pub pl: Option<f32>,
    pub pr: Option<f32>,
    pub gap: Option<f32>,

    /// "start" | "center" | "end" | "stretch"
    pub align: String,
    /// "start" | "center" | "end" | "between" | "around"
    pub justify: String,
    pub center: bool,

    /// Color tokens ("card", "card_inner", "tile", "tile_hover", "accent", "accent_soft", "emerald", "amber", "red", "transparent")
    /// or hex ("#RRGGBB" / "#RRGGBBAA").
    pub bg: String,
    pub hover_bg: String,
    pub border_color: String,
    pub border_w: Option<f32>,
    pub radius: Option<f32>,
    pub bevel: Option<f32>,
    pub hexagon: bool,
    pub shadow: Option<f32>,
    pub opacity: Option<f32>,
    pub clip: bool,
    pub passthrough: bool,
}

/// Declarative UI tree node rendered natively by `ext_view.rs`.
#[derive(Clone, Debug, Default, Deserialize)]
#[serde(default)]
pub struct ExtNode {
    /// Node kind:
    /// Layout: "col" | "row" | "grid" | "scroll" | "hscroll" | "zstack" | "card" | "spacer" | "divider"
    /// Display: "text" | "icon" | "image" | "badge" | "progress"
    /// Interactive: "button" | "toggle" | "slider" | "input" | "tabs"
    pub kind: String,
    pub id: String,
    pub style: ExtNodeStyle,

    // Text / Icon / Badge / Input / Button props
    pub text: String,
    pub placeholder: String,
    pub icon: String,
    pub image_key: String,
    /// "contain" | "cover" | "fill"
    pub fit: String,
    pub font: String,
    pub size: Option<f32>,
    /// "normal" | "medium" | "semibold" | "bold"
    pub weight: String,
    /// "left" | "center" | "right"
    pub text_align: String,
    /// Color token or hex string
    pub fg: String,
    pub wrap: bool,
    pub ellipsis: bool,

    // Interactive / Layout props
    pub active: bool,
    pub disabled: bool,
    /// Button variant: "" | "default" | "accent" | "danger" | "ghost" | "emerald"
    pub variant: String,
    pub tooltip: String,
    pub value: f32,
    pub min: f32,
    pub max: f32,
    pub step: f32,
    pub columns: usize,
    pub options: Vec<String>,
    pub selected_index: usize,
    pub hide_scrollbar: bool,
    pub max_len: usize,
    pub password: bool,
    pub searchable: bool,
    pub dropdown_style: Option<ExtDropdownStyle>,

    pub children: Vec<ExtNode>,
}

/// Optional popup styling overrides for `"dropdown"` / searchable combobox nodes.
#[derive(Clone, Debug, Default, Deserialize)]
#[serde(default)]
pub struct ExtDropdownStyle {
    pub bg: String,
    pub border_color: String,
    pub border_w: Option<f32>,
    pub radius: Option<f32>,
    pub bevel: Option<f32>,
    pub max_h: Option<f32>,
    pub min_w: Option<f32>,
    pub padding: Option<f32>,
    pub gap: Option<f32>,
    pub font_size: Option<f32>,
    pub item_h: Option<f32>,
    pub item_bg: String,
    pub item_hover_bg: String,
    pub item_fg: String,
    pub selected_bg: String,
    pub selected_hover_bg: String,
    pub selected_fg: String,
    pub selected_border: String,
    pub shadow: Option<f32>,
}
