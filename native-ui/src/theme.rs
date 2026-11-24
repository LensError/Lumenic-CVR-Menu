use std::cell::Cell;

use lumenic::{Color, UiContext};

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
}

impl Default for PanelPalette {
    fn default() -> Self {
        Self {
            card_bg: CARD_BG,
            card_inner_bg: CARD_INNER_BG,
            tile_idle_bg: TILE_IDLE_BG,
            tile_idle_hover: TILE_IDLE_HOVER,
            border_subtle: BORDER_SUBTLE,
        }
    }
}

thread_local! {
    static ACTIVE_PALETTE: Cell<PanelPalette> = Cell::new(PanelPalette::default());
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

/// Computes the frame's panel palette from CVR's Colorize Menu setting and Primary player color.
/// Keeps panel tinting dark and tame (never bright/neon) while matching OG menu semantics:
/// Primary = subtle panel colorizing, Secondary = accent highlights.
pub fn begin_frame(state: &AppState) {
    let palette = if !state.host.colorize_menu {
        PanelPalette::default()
    } else {
        let pr = state.host.accent_r as f32;
        let pg = state.host.accent_g as f32;
        let pb = state.host.accent_b as f32;
        if pr.max(pg).max(pb) < 12.0 {
            PanelPalette::default()
        } else {
            // Cap primary luminance so bright/white primaries remain dark and stylish on panels.
            let tame = clamp_luma_rgb(pr, pg, pb, 118.0);
            let border_tint = clamp_luma_rgb(pr, pg, pb, 185.0);
            PanelPalette {
                card_bg: mix_tinted(CARD_BG, tame, 0.20),
                card_inner_bg: mix_tinted(CARD_INNER_BG, tame, 0.14),
                tile_idle_bg: mix_tinted(TILE_IDLE_BG, tame, 0.16),
                tile_idle_hover: mix_tinted(TILE_IDLE_HOVER, tame, 0.22),
                border_subtle: Color::rgba(
                    border_tint.0.clamp(0.0, 255.0) as u8,
                    border_tint.1.clamp(0.0, 255.0) as u8,
                    border_tint.2.clamp(0.0, 255.0) as u8,
                    58,
                ),
            }
        }
    };
    ACTIVE_PALETTE.with(|p| p.set(palette));
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
