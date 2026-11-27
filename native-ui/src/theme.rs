use std::cell::Cell;

use lumenic::{Color, UiContext};

use crate::ext::ExtStyleOverride;
use crate::state::{AppState, UiAction};

// Official ChilloutVR brand palette defaults (used when Colorize Menu is OFF).
pub const CVR_ORANGE: Color = Color::rgb(228, 54, 0);    // #E43600
pub const CVR_DARK: Color = Color::rgb(34, 34, 34);      // #222222
pub const COD_GRAY: Color = Color::rgb(26, 26, 26);      // #1A1A1A
pub const WHISKY_SOUR: Color = Color::rgb(240, 165, 48); // #F0A530
pub const APPLE_GREEN: Color = Color::rgb(55, 189, 60);  // #37BD3C

pub const CARD_BG: Color = Color::rgba(34, 34, 34, 232);
pub const CARD_INNER_BG: Color = Color::rgba(26, 26, 26, 226);
pub const TILE_IDLE_BG: Color = Color::rgba(42, 42, 42, 230);
pub const TILE_IDLE_HOVER: Color = Color::rgba(56, 56, 56, 242);
pub const TILE_DISABLED_BG: Color = Color::rgba(28, 28, 28, 175);
pub const BORDER_SUBTLE: Color = Color::rgba(255, 255, 255, 32);
pub const SHADOW_CARD: Color = Color::rgba(0, 0, 0, 135);

pub const TEXT_PRIMARY: Color = Color::rgb(255, 255, 255);
pub const TEXT_MUTED: Color = Color::rgb(176, 176, 176);
pub const TEXT_DIM: Color = Color::rgb(122, 122, 122);

pub const DEFAULT_ACCENT: Color = CVR_ORANGE;
pub const EMERALD: Color = APPLE_GREEN;
pub const AMBER: Color = WHISKY_SOUR;
pub const RED: Color = Color::rgb(239, 68, 68);

#[derive(Clone, Copy)]
pub struct PanelPalette {
    pub card_bg: Color,
    pub card_inner_bg: Color,
    pub tile_idle_bg: Color,
    pub tile_idle_hover: Color,
    pub border_subtle: Color,
    pub text_primary: Color,
    pub text_muted: Color,
    pub text_dim: Color,
    pub shadow_card: Color,
}

impl Default for PanelPalette {
    fn default() -> Self {
        Self {
            card_bg: CARD_BG,
            card_inner_bg: CARD_INNER_BG,
            tile_idle_bg: TILE_IDLE_BG,
            tile_idle_hover: TILE_IDLE_HOVER,
            border_subtle: BORDER_SUBTLE,
            text_primary: TEXT_PRIMARY,
            text_muted: TEXT_MUTED,
            text_dim: TEXT_DIM,
            shadow_card: SHADOW_CARD,
        }
    }
}

#[derive(Clone, Copy)]
pub struct ThemeRuntime {
    pub font_scale: f32,
    pub title_scale: f32,
    pub body_scale: f32,
    pub caption_scale: f32,
    pub icon_scale: f32,
    pub spacing_scale: f32,
    pub padding_scale: f32,
    pub radius_scale: f32,
    pub bevel_scale: f32,
    pub tile_min_height: f32,
    pub qm_action_columns: usize,
    pub glow_intensity: f32,
}

impl Default for ThemeRuntime {
    fn default() -> Self {
        Self {
            font_scale: 1.0,
            title_scale: 1.0,
            body_scale: 1.0,
            caption_scale: 1.0,
            icon_scale: 1.0,
            spacing_scale: 1.0,
            padding_scale: 1.0,
            radius_scale: 1.0,
            bevel_scale: 1.0,
            tile_min_height: 52.0,
            qm_action_columns: 3,
            glow_intensity: 1.0,
        }
    }
}

thread_local! {
    static ACTIVE_PALETTE: Cell<PanelPalette> = Cell::new(PanelPalette::default());
    static ACTIVE_THEME: Cell<ThemeRuntime> = Cell::new(ThemeRuntime::default());
}

/// Parses `#RRGGBB` or `#RRGGBBAA` hex color strings.
pub fn parse_hex_color(s: &str) -> Option<Color> {
    let clean = s.trim().trim_start_matches('#');
    if clean.len() == 6 {
        let r = u8::from_str_radix(&clean[0..2], 16).ok()?;
        let g = u8::from_str_radix(&clean[2..4], 16).ok()?;
        let b = u8::from_str_radix(&clean[4..6], 16).ok()?;
        Some(Color::rgb(r, g, b))
    } else if clean.len() == 8 {
        let r = u8::from_str_radix(&clean[0..2], 16).ok()?;
        let g = u8::from_str_radix(&clean[2..4], 16).ok()?;
        let b = u8::from_str_radix(&clean[4..6], 16).ok()?;
        let a = u8::from_str_radix(&clean[6..8], 16).ok()?;
        Some(Color::rgba(r, g, b, a))
    } else {
        None
    }
}

/// Resolves a named Lumenic theme color token or `#RRGGBB`/`#RRGGBBAA` hex string.
pub fn resolve_color_token(token: &str, accent: Color) -> Option<Color> {
    let t = token.trim();
    if t.is_empty() {
        return None;
    }
    match t {
        "transparent" => Some(Color::TRANSPARENT),
        "card" | "card_bg" => Some(card_bg()),
        "card_inner" | "card_inner_bg" => Some(card_inner_bg()),
        "tile" | "tile_idle" => Some(tile_idle_bg()),
        "tile_hover" => Some(tile_idle_hover()),
        "border" | "border_subtle" => Some(border_subtle()),
        "accent" => Some(accent),
        "accent_soft" => Some(tint_alpha(accent, 48)),
        "accent_hover" => Some(tint_alpha(accent, 78)),
        "accent_border" => Some(tint_alpha(accent, 185)),
        "primary" | "text" | "text_primary" => Some(text_primary()),
        "muted" | "text_muted" => Some(text_muted()),
        "dim" | "text_dim" => Some(text_dim()),
        "emerald" | "green" | "success" => Some(EMERALD),
        "amber" | "yellow" | "warning" => Some(AMBER),
        "red" | "danger" | "error" => Some(RED),
        _ => parse_hex_color(t),
    }
}

fn scale_alpha(c: Color, factor: f32) -> Color {
    let a = ((c.a as f32) * factor.clamp(0.0, 2.0)).clamp(0.0, 255.0) as u8;
    Color::rgba(c.r, c.g, c.b, a)
}

fn clamp_luma_rgb(r: f32, g: f32, b: f32, max_luma: f32) -> (f32, f32, f32) {
    let luma = 0.2126 * r + 0.7152 * g + 0.0722 * b;
    if luma <= max_luma || luma < 1.0 {
        return (r, g, b);
    }
    let scale = max_luma / luma;
    (r * scale, g * scale, b * scale)
}

fn mix_tinted(base: Color, tint: (f32, f32, f32), amount: f32) -> Color {
    let inv = 1.0 - amount;
    let r = (base.r as f32 * inv + tint.0 * amount).clamp(0.0, 255.0) as u8;
    let g = (base.g as f32 * inv + tint.1 * amount).clamp(0.0, 255.0) as u8;
    let b = (base.b as f32 * inv + tint.2 * amount).clamp(0.0, 255.0) as u8;
    Color::rgba(r, g, b, base.a)
}

fn base_palette_for_preset(preset: &str) -> PanelPalette {
    match preset {
        "oled" => PanelPalette {
            card_bg: Color::rgba(10, 10, 12, 246),
            card_inner_bg: Color::rgba(16, 16, 20, 242),
            tile_idle_bg: Color::rgba(22, 22, 27, 238),
            tile_idle_hover: Color::rgba(34, 34, 42, 248),
            border_subtle: Color::rgba(255, 255, 255, 28),
            shadow_card: Color::rgba(0, 0, 0, 190),
            ..PanelPalette::default()
        },
        "frost" => PanelPalette {
            card_bg: Color::rgba(36, 42, 54, 198),
            card_inner_bg: Color::rgba(28, 34, 45, 190),
            tile_idle_bg: Color::rgba(48, 56, 72, 196),
            tile_idle_hover: Color::rgba(64, 74, 94, 220),
            border_subtle: Color::rgba(255, 255, 255, 58),
            shadow_card: Color::rgba(0, 0, 0, 120),
            ..PanelPalette::default()
        },
        "contrast" => PanelPalette {
            card_bg: Color::rgba(14, 14, 16, 250),
            card_inner_bg: Color::rgba(20, 20, 24, 248),
            tile_idle_bg: Color::rgba(30, 30, 36, 248),
            tile_idle_hover: Color::rgba(48, 48, 58, 255),
            border_subtle: Color::rgba(255, 255, 255, 112),
            text_muted: Color::rgb(212, 212, 220),
            text_dim: Color::rgb(168, 168, 180),
            shadow_card: Color::rgba(0, 0, 0, 200),
            ..PanelPalette::default()
        },
        _ => PanelPalette::default(),
    }
}

/// Computes the frame's panel palette and typography/density tokens from CVR's Colorize Menu setting,
/// built-in user FontScale, theme preset, and any active `LumenicApi.Theme` overrides.
pub fn begin_frame(state: &AppState) {
    let base_preset = base_palette_for_preset(state.host.theme_preset.as_str());
    let mut palette = if !state.host.colorize_menu {
        base_preset
    } else {
        let pr = state.host.accent_r as f32;
        let pg = state.host.accent_g as f32;
        let pb = state.host.accent_b as f32;
        if pr.max(pg).max(pb) < 12.0 {
            base_preset
        } else {
            // Cap primary luminance so bright/white primaries remain dark and stylish on panels.
            let tame = clamp_luma_rgb(pr, pg, pb, 118.0);
            let border_tint = clamp_luma_rgb(pr, pg, pb, 185.0);
            PanelPalette {
                card_bg: mix_tinted(base_preset.card_bg, tame, 0.20),
                card_inner_bg: mix_tinted(base_preset.card_inner_bg, tame, 0.14),
                tile_idle_bg: mix_tinted(base_preset.tile_idle_bg, tame, 0.16),
                tile_idle_hover: mix_tinted(base_preset.tile_idle_hover, tame, 0.22),
                border_subtle: Color::rgba(
                    border_tint.0.clamp(0.0, 255.0) as u8,
                    border_tint.1.clamp(0.0, 255.0) as u8,
                    border_tint.2.clamp(0.0, 255.0) as u8,
                    base_preset.border_subtle.a.max(58),
                ),
                ..base_preset
            }
        }
    };

    let ext = &state.extensions.theme;
    let host_opacity = if state.host.panel_opacity.is_finite() {
        state.host.panel_opacity.clamp(0.5, 1.0)
    } else {
        1.0
    };
    let ext_opacity = if ext.panel_opacity.is_finite() {
        ext.panel_opacity.clamp(0.15, 1.0)
    } else {
        1.0
    };
    let opacity = (host_opacity * ext_opacity).clamp(0.15, 1.0);
    if (opacity - 1.0).abs() > 0.001 {
        palette.card_bg = scale_alpha(palette.card_bg, opacity);
        palette.card_inner_bg = scale_alpha(palette.card_inner_bg, opacity);
        palette.tile_idle_bg = scale_alpha(palette.tile_idle_bg, opacity);
    }

    let shadow_str = if ext.shadow_strength.is_finite() {
        ext.shadow_strength.clamp(0.0, 2.0)
    } else {
        1.0
    };
    if (shadow_str - 1.0).abs() > 0.001 {
        palette.shadow_card = scale_alpha(palette.shadow_card, shadow_str);
    }

    if let Some(c) = ext.card_bg.as_deref().and_then(parse_hex_color) {
        palette.card_bg = c;
    }
    if let Some(c) = ext.card_inner_bg.as_deref().and_then(parse_hex_color) {
        palette.card_inner_bg = c;
    }
    if let Some(c) = ext.tile_idle_bg.as_deref().and_then(parse_hex_color) {
        palette.tile_idle_bg = c;
    }
    if let Some(c) = ext.tile_idle_hover.as_deref().and_then(parse_hex_color) {
        palette.tile_idle_hover = c;
    }
    if let Some(c) = ext.border_subtle.as_deref().and_then(parse_hex_color) {
        palette.border_subtle = c;
    }
    if let Some(c) = ext.text_primary.as_deref().and_then(parse_hex_color) {
        palette.text_primary = c;
    }
    if let Some(c) = ext.text_muted.as_deref().and_then(parse_hex_color) {
        palette.text_muted = c;
    }
    if let Some(c) = ext.text_dim.as_deref().and_then(parse_hex_color) {
        palette.text_dim = c;
    }

    let host_fs = if state.host.font_scale.is_finite() && state.host.font_scale > 0.0 {
        state.host.font_scale.clamp(0.5, 2.5)
    } else {
        1.0
    };
    let base_fs = ext
        .font_scale_override
        .filter(|v| v.is_finite() && *v > 0.0)
        .unwrap_or(host_fs);
    let mul_fs = if ext.font_scale_mul.is_finite() && ext.font_scale_mul > 0.0 {
        ext.font_scale_mul
    } else {
        1.0
    };

    let clamp_scale = |v: f32, lo: f32, hi: f32| {
        if v.is_finite() && v > 0.0 {
            v.clamp(lo, hi)
        } else {
            1.0
        }
    };

    let runtime = ThemeRuntime {
        font_scale: (base_fs * mul_fs).clamp(0.5, 2.5),
        title_scale: clamp_scale(ext.title_scale, 0.5, 2.5),
        body_scale: clamp_scale(ext.body_scale, 0.5, 2.5),
        caption_scale: clamp_scale(ext.caption_scale, 0.5, 2.5),
        icon_scale: clamp_scale(ext.icon_scale, 0.5, 2.5),
        spacing_scale: clamp_scale(ext.spacing_scale, 0.4, 2.5),
        padding_scale: clamp_scale(ext.padding_scale, 0.4, 2.5),
        radius_scale: clamp_scale(ext.radius_scale, 0.0, 2.5),
        bevel_scale: clamp_scale(ext.bevel_scale, 0.0, 2.5),
        tile_min_height: ext
            .tile_min_height
            .filter(|v| v.is_finite())
            .map(|v| v.clamp(28.0, 140.0))
            .unwrap_or_else(|| (52.0 * (base_fs * mul_fs).clamp(0.85, 1.85)).clamp(44.0, 120.0)),
        qm_action_columns: ext
            .qm_action_columns
            .map(|c| c.clamp(2, 6))
            .unwrap_or(3),
        glow_intensity: if !state.host.cursor_glow {
            0.0
        } else if ext.glow_intensity.is_finite() {
            ext.glow_intensity.clamp(0.0, 2.0)
        } else {
            1.0
        },
    };

    ACTIVE_PALETTE.with(|p| p.set(palette));
    ACTIVE_THEME.with(|t| t.set(runtime));
}

struct StyleOverrideGuard {
    prev_theme: ThemeRuntime,
    prev_palette: PanelPalette,
}

impl Drop for StyleOverrideGuard {
    fn drop(&mut self) {
        ACTIVE_THEME.with(|t| t.set(self.prev_theme));
        ACTIVE_PALETTE.with(|p| p.set(self.prev_palette));
    }
}

/// Temporarily applies a named section's style/layout overrides while rendering that section.
pub fn with_style_override<R>(ov: Option<&ExtStyleOverride>, f: impl FnOnce() -> R) -> R {
    let Some(ov) = ov else {
        return f();
    };
    let prev_theme = ACTIVE_THEME.with(|t| t.get());
    let prev_palette = ACTIVE_PALETTE.with(|p| p.get());
    let _guard = StyleOverrideGuard {
        prev_theme,
        prev_palette,
    };

    let mut next_theme = prev_theme;
    if let Some(fs) = ov.font_scale.filter(|v| v.is_finite() && *v > 0.0) {
        next_theme.font_scale = fs.clamp(0.5, 2.5);
    }
    if let Some(sp) = ov.spacing_scale.filter(|v| v.is_finite() && *v > 0.0) {
        next_theme.spacing_scale = sp.clamp(0.4, 2.5);
    }
    if let Some(pd) = ov.padding_scale.filter(|v| v.is_finite() && *v > 0.0) {
        next_theme.padding_scale = pd.clamp(0.4, 2.5);
    }
    if let Some(cols) = ov.columns {
        next_theme.qm_action_columns = cols.clamp(1, 8);
    }
    if let Some(mh) = ov.min_height.filter(|v| v.is_finite() && *v > 0.0) {
        next_theme.tile_min_height = mh.clamp(24.0, 200.0);
    }

    let mut next_palette = prev_palette;
    if let Some(c) = ov.card_bg.as_deref().and_then(parse_hex_color) {
        next_palette.card_bg = c;
    }
    if let Some(c) = ov.border.as_deref().and_then(parse_hex_color) {
        next_palette.border_subtle = c;
    }

    ACTIVE_THEME.with(|t| t.set(next_theme));
    ACTIVE_PALETTE.with(|p| p.set(next_palette));
    f()
}

#[inline]
pub fn fs(base: f32) -> f32 {
    ACTIVE_THEME.with(|t| {
        let th = t.get();
        (base * th.font_scale * th.body_scale).clamp(6.0, 64.0)
    })
}

#[inline]
pub fn fs_title(base: f32) -> f32 {
    ACTIVE_THEME.with(|t| {
        let th = t.get();
        (base * th.font_scale * th.title_scale).clamp(6.0, 72.0)
    })
}

#[inline]
pub fn fs_caption(base: f32) -> f32 {
    ACTIVE_THEME.with(|t| {
        let th = t.get();
        (base * th.font_scale * th.caption_scale).clamp(6.0, 48.0)
    })
}

/// Scales fixed container heights/widths (notifications, clock, tabs, pills) proportionally
/// with `font_scale` so larger text never clips or overflows.
#[inline]
pub fn text_box(base: f32) -> f32 {
    ACTIVE_THEME.with(|t| base * t.get().font_scale.clamp(0.85, 1.85))
}

#[inline]
pub fn ic(base: f32) -> f32 {
    ACTIVE_THEME.with(|t| (base * t.get().icon_scale).clamp(8.0, 96.0))
}

#[inline]
pub fn sp(base: f32) -> f32 {
    ACTIVE_THEME.with(|t| (base * t.get().spacing_scale).clamp(0.0, 96.0))
}

#[inline]
pub fn pad(base: f32) -> f32 {
    ACTIVE_THEME.with(|t| (base * t.get().padding_scale).clamp(0.0, 96.0))
}

#[inline]
pub fn rad(base: f32) -> f32 {
    ACTIVE_THEME.with(|t| (base * t.get().radius_scale).clamp(0.0, 64.0))
}

#[inline]
pub fn bev(base: f32) -> f32 {
    ACTIVE_THEME.with(|t| (base * t.get().bevel_scale).clamp(0.0, 24.0))
}

#[inline]
pub fn tile_min_h() -> f32 {
    ACTIVE_THEME.with(|t| t.get().tile_min_height)
}

#[inline]
pub fn qm_action_cols() -> usize {
    ACTIVE_THEME.with(|t| t.get().qm_action_columns)
}

#[inline]
pub fn glow_intensity() -> f32 {
    ACTIVE_THEME.with(|t| t.get().glow_intensity)
}

#[inline]
pub fn card_bg() -> Color {
    ACTIVE_PALETTE.with(|p| p.get().card_bg)
}

#[inline]
pub fn card_inner_bg() -> Color {
    ACTIVE_PALETTE.with(|p| p.get().card_inner_bg)
}

#[inline]
pub fn tile_idle_bg() -> Color {
    ACTIVE_PALETTE.with(|p| p.get().tile_idle_bg)
}

#[inline]
pub fn tile_idle_hover() -> Color {
    ACTIVE_PALETTE.with(|p| p.get().tile_idle_hover)
}

#[inline]
pub fn border_subtle() -> Color {
    ACTIVE_PALETTE.with(|p| p.get().border_subtle)
}

#[inline]
pub fn text_primary() -> Color {
    ACTIVE_PALETTE.with(|p| p.get().text_primary)
}

#[inline]
pub fn text_muted() -> Color {
    ACTIVE_PALETTE.with(|p| p.get().text_muted)
}

#[inline]
pub fn text_dim() -> Color {
    ACTIVE_PALETTE.with(|p| p.get().text_dim)
}

#[inline]
pub fn shadow_card() -> Color {
    ACTIVE_PALETTE.with(|p| p.get().shadow_card)
}

pub const BG_GLOW_FRAG: &str = r#"
in vec2 v_uv;
out vec4 frag_color;
uniform vec2 u_resolution;
uniform float u_uniforms[16];

float hash21(vec2 p) {
    p = fract(p * vec2(123.34, 456.21));
    p += dot(p, p + 45.32);
    return fract(p.x * p.y);
}

vec4 hexCoords(vec2 p) {
    const vec2 r = vec2(1.0, 1.7320508);
    const vec2 h = r * 0.5;
    vec2 a = mod(p, r) - h;
    vec2 b = mod(p - h, r) - h;
    vec2 gv = dot(a, a) < dot(b, b) ? a : b;
    vec2 id = p - gv;
    return vec4(gv, id);
}

float hexEdgeDist(vec2 gv) {
    vec2 p = abs(gv);
    float d = max(dot(p, normalize(vec2(1.0, 1.7320508))), p.x);
    return 0.5 - d;
}

void main() {
    float ptrActive = u_uniforms[2];
    if (ptrActive < 0.01) {
        frag_color = vec4(0.0);
        return;
    }

    float hasCutout = u_uniforms[6];
    vec4 cutRect = vec4(u_uniforms[7], u_uniforms[8], u_uniforms[9], u_uniforms[10]);
    if (hasCutout > 0.5) {
        if (v_uv.x >= cutRect.x && v_uv.x <= cutRect.z && v_uv.y >= cutRect.y && v_uv.y <= cutRect.w) {
            frag_color = vec4(0.0);
            return;
        }
    }

    vec2 res = max(u_resolution, vec2(1.0));
    float dpiScale = max(res.y / 720.0, 0.5);
    vec2 px = v_uv * res;
    vec2 ptrPx = vec2(u_uniforms[0], u_uniforms[1]) * res;
    vec3 accent = vec3(u_uniforms[3], u_uniforms[4], u_uniforms[5]);

    float radius = 210.0 * dpiScale;
    float pxDist = length(px - ptrPx);
    if (pxDist > radius * 1.15) {
        frag_color = vec4(0.0);
        return;
    }

    float hexScale = 34.0 * dpiScale;
    vec4 hc = hexCoords(px / hexScale);
    vec2 gv = hc.xy;
    vec2 cellId = hc.zw;
    vec2 cellCenterPx = cellId * hexScale;

    float cellDist = length(cellCenterPx - ptrPx);
    float cellRand = hash21(cellId);

    float radialMask = smoothstep(radius, 0.0, pxDist);
    float radialPow = radialMask * radialMask;
    float cellPulse = smoothstep(radius * (0.85 + 0.25 * cellRand), 0.0, cellDist);
    float cellEnergy = mix(radialPow, cellPulse * (0.65 + 0.35 * cellRand), 0.55);

    float edge = hexEdgeDist(gv);
    float borderLine = 1.0 - smoothstep(0.012, 0.062, edge);
    float innerGlow = smoothstep(0.0, 0.14, edge) * (1.0 - smoothstep(0.14, 0.45, edge));
    float cellFill = smoothstep(0.02, 0.07, edge) * (0.12 + 0.16 * cellRand);

    float hotCore = exp(-(pxDist / dpiScale) * 0.014);
    vec3 brightAccent = mix(accent, vec3(1.0), 0.32 * hotCore);
    vec3 col = brightAccent * (borderLine * (0.75 + 0.65 * hotCore) + innerGlow * 0.28 + cellFill);
    float alpha = (borderLine * 0.78 + innerGlow * 0.24 + cellFill * 0.45) * cellEnergy * ptrActive;

    float edgeFade = smoothstep(0.0, 14.0 * dpiScale, min(min(px.x, res.x - px.x), min(px.y, res.y - px.y)));
    alpha *= edgeFade;

    frag_color = vec4(col, clamp(alpha, 0.0, 0.72));
}
"#;

pub fn resolve_player_accent(state: &AppState) -> Color {
    if let Some(c) = state
        .extensions
        .theme
        .accent_override
        .as_deref()
        .and_then(parse_hex_color)
    {
        return c;
    }
    if !state.host.colorize_menu {
        return DEFAULT_ACCENT;
    }
    // Match OG ChilloutVR menu: SecondaryColor drives accent highlights, falling back to Primary if Secondary is black.
    let (mut r, mut g, mut b) = (
        state.host.accent2_r as f32,
        state.host.accent2_g as f32,
        state.host.accent2_b as f32,
    );
    if r.max(g).max(b) < 16.0 {
        r = state.host.accent_r as f32;
        g = state.host.accent_g as f32;
        b = state.host.accent_b as f32;
    }
    if r.max(g).max(b) < 16.0 {
        return DEFAULT_ACCENT;
    }
    let luma = 0.2126 * r + 0.7152 * g + 0.0722 * b;
    if luma >= 100.0 {
        return Color::rgb(r as u8, g as u8, b as u8);
    }
    let lift = (100.0 - luma) / (255.0 - luma).max(1.0);
    let lr = (r + (255.0 - r) * lift * 0.65).clamp(0.0, 255.0) as u8;
    let lg = (g + (255.0 - g) * lift * 0.65).clamp(0.0, 255.0) as u8;
    let lb = (b + (255.0 - b) * lift * 0.65).clamp(0.0, 255.0) as u8;
    Color::rgb(lr, lg, lb)
}

#[inline]
pub fn is_special_rank(rank: &str) -> bool {
    let trimmed = rank.trim();
    !trimmed.is_empty() && !trimmed.eq_ignore_ascii_case("user")
}

#[inline]
pub fn tint_alpha(c: Color, alpha: u8) -> Color {
    Color::rgba(c.r, c.g, c.b, alpha)
}

#[inline]
pub fn push_click(actions: &mut Vec<UiAction>) {
    actions.push(UiAction::PlayUiSound {
        sound: String::from("Click"),
    });
}

const MODAL_SCRIM_FRAG: &str = r#"
#ifdef GL_ES
precision mediump float;
#endif
varying vec2 v_uv;
uniform vec4 u_bounds;
uniform vec4 u_params[4];
void main() {
    float edge_x = smoothstep(0.0, 0.24, min(v_uv.x, 1.0 - v_uv.x));
    float edge_y = smoothstep(0.0, 0.24, min(v_uv.y, 1.0 - v_uv.y));
    float alpha = 0.78 * edge_x * edge_y;
    gl_FragColor = vec4(0.016, 0.027, 0.051, alpha);
}
"#;

/// Renders a full-viewport edge-faded dark modal scrim and returns true if clicked.
pub fn modal_scrim(ui: &mut UiContext, fx_id: &str, hit_id: &str) -> bool {
    ui.viewport(fx_id, |c2d, _s3d| {
        c2d.shader(0x4C55_4D02, MODAL_SCRIM_FRAG, &[]);
    })
    .w_frac(1.0)
    .h_frac(1.0)
    .no_bg()
    .borderless()
    .r(0.0)
    .passthrough();

    ui.btn_with(hit_id, |_| {})
        .w_frac(1.0)
        .h_frac(1.0)
        .bg(Color::TRANSPARENT)
        .hover_bg(Color::TRANSPARENT)
        .borderless()
        .r(0.0)
        .clicked()
}
