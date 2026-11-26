use lumenic::{Color, Rect, UiContext};

use crate::theme::{
    border_subtle, card_inner_bg, glow_intensity, tile_idle_bg, tile_idle_hover, tint_alpha,
    BG_GLOW_FRAG, TEXT_DIM, TEXT_MUTED, TEXT_PRIMARY, TILE_DISABLED_BG,
};

pub struct SliderResult {
    pub changed: bool,
    pub hovered: bool,
}

/// Unified LumenicMenu slider: static WidgetId (never drops drag mid-slide),
/// taller 14px track, stretched-hexagon chamfered container, and no focus outline on click.
#[allow(clippy::too_many_arguments)]
pub fn styled_slider(
    ui: &mut UiContext,
    id: &str,
    label: &str,
    value_text: &str,
    value: &mut f32,
    range: std::ops::RangeInclusive<f32>,
    accent: Color,
    enabled: bool,
    flat: bool,
) -> SliderResult {
    let mut changed = false;
    let mut hovered = false;
    ui.push_id(id);
    let col = ui.col(|ui| {
        ui.row(|ui| {
            ui.txt(label)
                .sz(crate::theme::fs(11.0))
                .semibold()
                .fg(if enabled { TEXT_PRIMARY } else { TEXT_DIM })
                .ellipsis()
                .grow(1.0);
            if !value_text.is_empty() {
                ui.txt(value_text)
                    .sz(crate::theme::fs(10.5))
                    .medium()
                    .fg(if enabled { accent } else { TEXT_DIM });
            }
        })
        .align_center()
        .gap(6.0);

        let mut resp = ui
            .slider("__track", value, range)
            .fill(accent)
            .fg(accent)
            .no_bg()
            .borderless()
            .no_focus_ring()
            .track_h(14.0)
            .track_bg(tile_idle_bg())
            .track_bd(1.0, border_subtle())
            .enabled(enabled)
            .w_frac(1.0);
        changed = resp.changed();
        hovered = resp.hovered();
    });
    let mut col_resp = if flat {
        col.px(2.0).py(3.0).gap(4.0).w_frac(1.0)
    } else {
        col.px(10.0)
            .py(7.0)
            .gap(5.0)
            .bg(card_inner_bg())
            .hover_bg(tile_idle_hover())
            .bd(1.0, border_subtle())
            .bevel(8.0)
            .opacity(if enabled { 1.0 } else { 0.48 })
            .grow(1.0)
    };
    hovered |= col_resp.hovered();
    ui.pop_id();
    SliderResult { changed, hovered }
}

pub struct TileColors {
    pub bg: Color,
    pub hover_bg: Color,
    pub border: Color,
    pub icon: Color,
    pub title: Color,
    pub status: Color,
}

/// Shared color resolution for Quick Menu action tiles and Main Menu utility tiles.
pub fn resolve_tile_colors(active: bool, enabled: bool, active_color: Color) -> TileColors {
    if !enabled {
        TileColors {
            bg: TILE_DISABLED_BG,
            hover_bg: TILE_DISABLED_BG,
            border: Color::rgba(100, 116, 139, 32),
            icon: TEXT_DIM,
            title: TEXT_DIM,
            status: TEXT_DIM,
        }
    } else if active {
        TileColors {
            bg: tint_alpha(active_color, 54),
            hover_bg: tint_alpha(active_color, 78),
            border: tint_alpha(active_color, 195),
            icon: active_color,
            title: TEXT_PRIMARY,
            status: active_color,
        }
    } else {
        TileColors {
            bg: tile_idle_bg(),
            hover_bg: tile_idle_hover(),
            border: border_subtle(),
            icon: Color::rgb(203, 213, 225),
            title: TEXT_PRIMARY,
            status: TEXT_MUTED,
        }
    }
}

/// Computes the 11-float uniform buffer for the cursor-follow hex-grid background shader.
pub fn compute_bg_glow_uniforms(
    ui: &UiContext,
    root_rect_opt: Option<[f32; 4]>,
    accent: Color,
    cutout_rect_opt: Option<[f32; 4]>,
) -> [f32; 11] {
    let [vw, vh] = ui.viewport_size();
    let root_rect = match root_rect_opt {
        Some([x, y, w, h]) if w > 1.0 && h > 1.0 => Rect::new(x, y, w, h),
        _ => Rect::new(0.0, 0.0, vw.max(1.0), vh.max(1.0)),
    };
    let (ptr_u, ptr_v, ptr_active) = if let Some([px, py]) = ui.pointer_pos() {
        let u = (px - root_rect.x) / root_rect.w.max(1.0);
        let v = (py - root_rect.y) / root_rect.h.max(1.0);
        let inside = (-0.25..=1.25).contains(&u) && (-0.25..=1.25).contains(&v);
        (
            u.clamp(0.0, 1.0),
            v.clamp(0.0, 1.0),
            if inside { 1.0 } else { 0.0 },
        )
    } else {
        (0.5, 0.5, 0.0)
    };
    let ac = accent.to_normalized_f32();
    let (has_cut, cut_x0, cut_y0, cut_x1, cut_y1) =
        if let Some([cx, cy, cw, ch]) = cutout_rect_opt {
            (
                1.0,
                ((cx - root_rect.x) / root_rect.w.max(1.0)).clamp(0.0, 1.0),
                ((cy - root_rect.y) / root_rect.h.max(1.0)).clamp(0.0, 1.0),
                (((cx + cw) - root_rect.x) / root_rect.w.max(1.0)).clamp(0.0, 1.0),
                (((cy + ch) - root_rect.y) / root_rect.h.max(1.0)).clamp(0.0, 1.0),
            )
        } else {
            (0.0, 0.0, 0.0, 0.0, 0.0)
        };
    let glow_active = ptr_active * glow_intensity();
    [
        ptr_u, ptr_v, glow_active, ac[0], ac[1], ac[2], has_cut, cut_x0, cut_y0, cut_x1, cut_y1,
    ]
}

/// Renders the passthrough background hex-glow shader viewport.
pub fn render_bg_glow(ui: &mut UiContext, id: &str, shader_key: u64, uniforms: [f32; 11]) {
    ui.viewport(id, move |c2d, _s3d| {
        c2d.shader(shader_key, BG_GLOW_FRAG, &uniforms);
    })
    .w_frac(1.0)
    .h_frac(1.0)
    .no_bg()
    .borderless()
    .r(0.0)
    .passthrough();
}
