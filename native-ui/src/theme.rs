use lumenic::Color;

use crate::state::{AppState, UiAction};

// Shared Obsidian Slate design tokens across Main Menu, Quick Menu, and all views.
pub const CARD_BG: Color = Color::rgba(15, 19, 28, 224);
pub const CARD_INNER_BG: Color = Color::rgba(22, 28, 41, 215);
pub const TILE_IDLE_BG: Color = Color::rgba(26, 33, 48, 225);
pub const TILE_IDLE_HOVER: Color = Color::rgba(38, 48, 68, 240);
pub const TILE_DISABLED_BG: Color = Color::rgba(20, 24, 34, 170);
pub const BORDER_SUBTLE: Color = Color::rgba(148, 163, 184, 42);
pub const SHADOW_CARD: Color = Color::rgba(0, 0, 0, 125);

pub const TEXT_PRIMARY: Color = Color::rgb(241, 245, 249);
pub const TEXT_MUTED: Color = Color::rgb(148, 163, 184);
pub const TEXT_DIM: Color = Color::rgb(100, 116, 139);

pub const DEFAULT_ACCENT: Color = Color::rgb(56, 189, 248);
pub const EMERALD: Color = Color::rgb(52, 211, 153);
pub const AMBER: Color = Color::rgb(251, 191, 36);
pub const RED: Color = Color::rgb(239, 68, 68);

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
    if !state.host.has_player_accent {
        return DEFAULT_ACCENT;
    }
    let r = state.host.accent_r as f32;
    let g = state.host.accent_g as f32;
    let b = state.host.accent_b as f32;
    if r.max(g).max(b) < 16.0 {
        return DEFAULT_ACCENT;
    }
    let luma = 0.2126 * r + 0.7152 * g + 0.0722 * b;
    if luma >= 110.0 {
        return Color::rgb(state.host.accent_r, state.host.accent_g, state.host.accent_b);
    }
    let lift = (110.0 - luma) / (255.0 - luma).max(1.0);
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
