use lumenic::{Color, DropdownStyle, ImageFit, StyleBuilder, TextAlign, UiContext};

use crate::ext::{
    ExtDropdownStyle, ExtModalSnapshot, ExtNode, ExtNodeStyle, ExtPageSnapshot, ExtSlotEntry,
};
use crate::icons;
use crate::state::{AppState, UiAction};
use crate::theme::{
    bev, border_subtle, card_bg, card_inner_bg, fs, pad, push_click, rad, resolve_color_token,
    shadow_card, sp, text_dim, text_muted, text_primary, tile_idle_bg, tile_idle_hover,
    tint_alpha, EMERALD, RED,
};

fn apply_style<'a>(
    mut sb: StyleBuilder<'a>,
    st: &ExtNodeStyle,
    accent: Color,
) -> StyleBuilder<'a> {
    if let Some(v) = st.w {
        sb = sb.w(v);
    }
    if let Some(v) = st.h {
        sb = sb.h(v);
    }
    if let Some(v) = st.w_frac {
        sb = sb.w_frac(v);
    }
    if let Some(v) = st.h_frac {
        sb = sb.h_frac(v);
    }
    if let Some(v) = st.min_w {
        sb = sb.min_w(v);
    }
    if let Some(v) = st.max_w {
        sb = sb.max_w(v);
    }
    if let Some(v) = st.min_h {
        sb = sb.min_h(v);
    }
    if let Some(v) = st.max_h {
        sb = sb.max_h(v);
    }
    if let Some(v) = st.grow {
        sb = sb.grow(v);
    }

    if let Some(v) = st.p {
        sb = sb.p(pad(v));
    }
    if let Some(v) = st.px {
        sb = sb.px(pad(v));
    }
    if let Some(v) = st.py {
        sb = sb.py(pad(v));
    }
    if let Some(v) = st.pt {
        sb = sb.pt(pad(v));
    }
    if let Some(v) = st.pb {
        sb = sb.pb(pad(v));
    }
    if let Some(v) = st.pl {
        sb = sb.pl(pad(v));
    }
    if let Some(v) = st.pr {
        sb = sb.pr(pad(v));
    }
    if let Some(v) = st.gap {
        sb = sb.gap(sp(v));
    }

    if st.center {
        sb = sb.center();
    } else {
        match st.align.as_str() {
            "start" => sb = sb.align_start(),
            "center" => sb = sb.align_center(),
            "end" => sb = sb.align_end(),
            "stretch" => sb = sb.align_stretch(),
            _ => {}
        }
        match st.justify.as_str() {
            "start" => sb = sb.justify_start(),
            "center" => sb = sb.justify_center(),
            "end" => sb = sb.justify_end(),
            "between" => sb = sb.justify_between(),
            "around" => sb = sb.justify_around(),
            _ => {}
        }
    }

    if let Some(c) = resolve_color_token(&st.bg, accent) {
        sb = sb.bg(c);
    }
    if let Some(c) = resolve_color_token(&st.hover_bg, accent) {
        sb = sb.hover_bg(c);
    }
    if let Some(bw) = st.border_w {
        if bw <= 0.0 {
            sb = sb.borderless();
        } else {
            let bc = resolve_color_token(&st.border_color, accent).unwrap_or_else(border_subtle);
            sb = sb.bd(bw, bc);
        }
    } else if !st.border_color.is_empty() {
        if let Some(bc) = resolve_color_token(&st.border_color, accent) {
            sb = sb.bd(1.0, bc);
        }
    }

    if st.hexagon {
        sb = sb.hexagon_pointy();
        if let Some(r) = st.radius {
            sb = sb.r(rad(r));
        }
    } else if let Some(b) = st.bevel {
        let r = st.radius.unwrap_or(6.0);
        sb = sb.bevel_rounded(rad(r), bev(b));
    } else if let Some(r) = st.radius {
        sb = sb.r(rad(r));
    }

    if let Some(sh) = st.shadow {
        if sh > 0.0 {
            sb = sb.shadow(sh, shadow_card());
        }
    }
    if let Some(op) = st.opacity {
        sb = sb.opacity(op.clamp(0.0, 1.0));
    }
    if st.clip {
        sb = sb.clip();
    }
    if st.passthrough {
        sb = sb.passthrough();
    }
    sb
}

fn push_ext_event(
    actions: &mut Vec<UiAction>,
    ext_id: &str,
    target_id: &str,
    node_id: &str,
    kind: &str,
    string_val: String,
    float_val: f32,
    bool_val: bool,
) {
    actions.push(UiAction::ExtEvent {
        ext_id: ext_id.to_owned(),
        target_id: target_id.to_owned(),
        node_id: node_id.to_owned(),
        kind: kind.to_owned(),
        string_val,
        float_val,
        bool_val,
    });
}

fn set_hovered_tooltip(state: &mut AppState, surface_name: &str, tip: &str) {
    if tip.is_empty() {
        return;
    }
    if surface_name == "main" {
        state.main_menu.hovered_tooltip = tip.to_owned();
    } else {
        state.quick.hovered_tooltip = tip.to_owned();
    }
}

/// Renders an active custom extension page inside Quick Menu or Main Menu.
pub fn render_ext_page(
    ui: &mut UiContext,
    state: &mut AppState,
    surface_name: &str,
    page: &ExtPageSnapshot,
    accent: Color,
    actions: &mut Vec<UiAction>,
) {
    ui.col(|ui| {
        if !page.hide_header {
            ui.row(|ui| {
                if page.can_go_back {
                    let back_id = format!("ext_back_{}_{}", surface_name, page.page_id);
                    if ui
                        .btn_with(&back_id, |ui| {
                            ui.row(|ui| {
                                icons::icon(ui, icons::BACK, 14.0).fg(text_primary());
                                ui.txt("Back").sz(fs(11.5)).semibold().fg(text_primary());
                            })
                            .align_center()
                            .gap(sp(4.0));
                        })
                        .px(pad(9.0))
                        .py(pad(5.0))
                        .bg(tile_idle_bg())
                        .hover_bg(tint_alpha(accent, 75))
                        .bd(1.0, border_subtle())
                        .bevel_rounded(rad(6.2), bev(1.3))
                        .clicked()
                    {
                        push_click(actions);
                        actions.push(UiAction::ExtGoBack {
                            surface: surface_name.to_owned(),
                        });
                    }
                }

                ui.col(|ui| {
                    let title = if page.title.is_empty() {
                        "Extension"
                    } else {
                        &page.title
                    };
                    ui.txt(title).sz(fs(14.0)).bold().fg(text_primary());
                    if !page.subtitle.is_empty() {
                        ui.txt(&page.subtitle).sz(fs(10.5)).fg(text_muted());
                    }
                })
                .grow(1.0)
                .gap(sp(1.0));

                if !page.header_actions.is_empty() {
                    ui.row(|ui| {
                        for (idx, act) in page.header_actions.iter().enumerate() {
                            render_ext_node(
                                ui,
                                state,
                                surface_name,
                                &page.ext_id,
                                &page.page_id,
                                act,
                                idx,
                                accent,
                                actions,
                            );
                        }
                    })
                    .align_center()
                    .gap(sp(6.0));
                }
            })
            .align_center()
            .px(pad(10.0))
            .py(pad(7.0))
            .gap(sp(8.0))
            .bg(card_bg())
            .bd(1.0, tint_alpha(accent, 95))
            .bevel_rounded(rad(8.6), bev(1.3))
            .shadow(10.0, shadow_card());
        }

        render_ext_node(
            ui,
            state,
            surface_name,
            &page.ext_id,
            &page.page_id,
            &page.root,
            0,
            accent,
            actions,
        );
    })
    .w_frac(1.0)
    .grow(1.0)
    .gap(sp(8.0));
}

/// Renders an active custom extension modal overlay.
pub fn render_ext_modal(
    ui: &mut UiContext,
    state: &mut AppState,
    surface_name: &str,
    modal: &ExtModalSnapshot,
    accent: Color,
    actions: &mut Vec<UiAction>,
) {
    ui.overlay(|ui| {
        ui.zstack(|ui| {
            let fx_id = format!("ext_modal_scrim_fx_{surface_name}");
            let hit_id = format!("ext_modal_scrim_hit_{surface_name}");
            if crate::theme::modal_scrim(ui, &fx_id, &hit_id) && modal.close_on_backdrop {
                push_click(actions);
                actions.push(UiAction::ExtCloseModal {
                    surface: surface_name.to_owned(),
                });
            }

            ui.col(|ui| {
                let modal_w = if modal.width > 120.0 {
                    modal.width.min(960.0)
                } else if surface_name == "main" {
                    520.0
                } else {
                    440.0
                };
                let mut card = ui
                    .col(|ui| {
                        ui.row(|ui| {
                            let title = if modal.title.is_empty() {
                                "Dialog"
                            } else {
                                &modal.title
                            };
                            ui.txt(title)
                                .sz(fs(14.0))
                                .bold()
                                .fg(text_primary())
                                .grow(1.0);
                            if ui
                                .btn_with(&format!("ext_modal_close_{surface_name}"), |ui| {
                                    icons::icon(ui, icons::CLOSE, 14.0).fg(text_muted());
                                })
                                .p(pad(5.0))
                                .bg(Color::TRANSPARENT)
                                .hover_bg(Color::rgba(239, 68, 68, 80))
                                .bevel_rounded(rad(4.5), bev(1.3))
                                .clicked()
                            {
                                push_click(actions);
                                actions.push(UiAction::ExtCloseModal {
                                    surface: surface_name.to_owned(),
                                });
                            }
                        })
                        .align_center()
                        .gap(sp(8.0));

                        render_ext_node(
                            ui,
                            state,
                            surface_name,
                            &modal.ext_id,
                            &modal.modal_id,
                            &modal.root,
                            0,
                            accent,
                            actions,
                        );
                    })
                    .w(modal_w)
                    .p(pad(14.0))
                    .gap(sp(8.0))
                    .bg(Color::rgba(15, 19, 28, 248))
                    .bd(1.5, accent)
                    .bevel_rounded(rad(12.0), bev(2.0))
                    .shadow(22.0, Color::rgba(0, 0, 0, 190));
                let _ = card.hovered();
            })
            .center()
            .w_frac(1.0)
            .h_frac(1.0);
        })
        .w_frac(1.0)
        .h_frac(1.0);
    })
    .grow(1.0);
}

/// Renders a list of injected slot entries (Before / After / InsideStart / InsideEnd / Replace).
pub fn render_ext_slots(
    ui: &mut UiContext,
    state: &mut AppState,
    surface_name: &str,
    entries: &[ExtSlotEntry],
    accent: Color,
    actions: &mut Vec<UiAction>,
) {
    for (idx, entry) in entries.iter().enumerate() {
        render_ext_node(
            ui,
            state,
            surface_name,
            &entry.ext_id,
            &entry.slot_id,
            &entry.root,
            idx,
            accent,
            actions,
        );
    }
}

/// Wraps a well-known built-in section with extension Before/After/Inside injections,
/// section-scoped style overrides, Hide, and full Replace support.
pub fn render_section(
    ui: &mut UiContext,
    state: &mut AppState,
    surface_name: &str,
    section_id: &str,
    accent: Color,
    actions: &mut Vec<UiAction>,
    builtin: impl FnOnce(&mut UiContext, &mut AppState, &mut Vec<UiAction>),
) {
    let patch = state.extensions.sections.get(section_id).cloned();
    let Some(patch) = patch else {
        builtin(ui, state, actions);
        return;
    };

    if !patch.before.is_empty() {
        render_ext_slots(ui, state, surface_name, &patch.before, accent, actions);
    }

    let hidden = patch.style.as_ref().is_some_and(|s| s.hidden);
    if !hidden {
        crate::theme::with_style_override(patch.style.as_ref(), || {
            if let Some(ref repl) = patch.replace {
                render_ext_node(
                    ui,
                    state,
                    surface_name,
                    &repl.ext_id,
                    &repl.slot_id,
                    &repl.root,
                    0,
                    accent,
                    actions,
                );
            } else {
                if !patch.inside_start.is_empty() {
                    render_ext_slots(ui, state, surface_name, &patch.inside_start, accent, actions);
                }
                builtin(ui, state, actions);
                if !patch.inside_end.is_empty() {
                    render_ext_slots(ui, state, surface_name, &patch.inside_end, accent, actions);
                }
            }
        });
    }

    if !patch.after.is_empty() {
        render_ext_slots(ui, state, surface_name, &patch.after, accent, actions);
    }
}

/// Renders an empty slot-only mount point (e.g. `qm.home.banner`, `mm.home.banner`) if any extension mounted there.
pub fn render_slot_only(
    ui: &mut UiContext,
    state: &mut AppState,
    surface_name: &str,
    section_id: &str,
    accent: Color,
    actions: &mut Vec<UiAction>,
) {
    let Some(patch) = state.extensions.sections.get(section_id).cloned() else {
        return;
    };
    if patch.style.as_ref().is_some_and(|s| s.hidden) {
        return;
    }
    crate::theme::with_style_override(patch.style.as_ref(), || {
        if !patch.before.is_empty() {
            render_ext_slots(ui, state, surface_name, &patch.before, accent, actions);
        }
        if let Some(ref repl) = patch.replace {
            render_ext_node(
                ui,
                state,
                surface_name,
                &repl.ext_id,
                &repl.slot_id,
                &repl.root,
                0,
                accent,
                actions,
            );
        } else {
            if !patch.inside_start.is_empty() {
                render_ext_slots(ui, state, surface_name, &patch.inside_start, accent, actions);
            }
            if !patch.inside_end.is_empty() {
                render_ext_slots(ui, state, surface_name, &patch.inside_end, accent, actions);
            }
        }
        if !patch.after.is_empty() {
            render_ext_slots(ui, state, surface_name, &patch.after, accent, actions);
        }
    });
}

#[allow(clippy::too_many_arguments)]
pub fn render_ext_node(
    ui: &mut UiContext,
    state: &mut AppState,
    surface_name: &str,
    ext_id: &str,
    target_id: &str,
    node: &ExtNode,
    sibling_idx: usize,
    accent: Color,
    actions: &mut Vec<UiAction>,
) {
    let local_id = if node.id.is_empty() {
        format!("n{sibling_idx}")
    } else {
        node.id.clone()
    };
    let widget_id = format!("ext_{surface_name}_{ext_id}_{target_id}_{local_id}");

    match node.kind.as_str() {
        "col" => {
            let sb = ui.col(|ui| {
                for (i, ch) in node.children.iter().enumerate() {
                    render_ext_node(
                        ui,
                        state,
                        surface_name,
                        ext_id,
                        target_id,
                        ch,
                        i,
                        accent,
                        actions,
                    );
                }
            });
            let mut resp = apply_style(sb, &node.style, accent);
            if !node.tooltip.is_empty() && resp.hovered() {
                set_hovered_tooltip(state, surface_name, &node.tooltip);
            }
        }

        "row" => {
            let sb = ui.row(|ui| {
                for (i, ch) in node.children.iter().enumerate() {
                    render_ext_node(
                        ui,
                        state,
                        surface_name,
                        ext_id,
                        target_id,
                        ch,
                        i,
                        accent,
                        actions,
                    );
                }
            });
            let mut resp = apply_style(sb, &node.style, accent);
            if !node.tooltip.is_empty() && resp.hovered() {
                set_hovered_tooltip(state, surface_name, &node.tooltip);
            }
        }

        "zstack" => {
            let sb = ui.zstack(|ui| {
                for (i, ch) in node.children.iter().enumerate() {
                    render_ext_node(
                        ui,
                        state,
                        surface_name,
                        ext_id,
                        target_id,
                        ch,
                        i,
                        accent,
                        actions,
                    );
                }
            });
            let _ = apply_style(sb, &node.style, accent);
        }

        "card" => {
            let sb = ui
                .col(|ui| {
                    for (i, ch) in node.children.iter().enumerate() {
                        render_ext_node(
                            ui,
                            state,
                            surface_name,
                            ext_id,
                            target_id,
                            ch,
                            i,
                            accent,
                            actions,
                        );
                    }
                })
                .p(pad(10.0))
                .gap(sp(6.0))
                .bg(card_bg())
                .bd(1.0, border_subtle())
                .bevel_rounded(rad(9.0), bev(1.5))
                .shadow(10.0, shadow_card());
            let mut resp = apply_style(sb, &node.style, accent);
            if !node.tooltip.is_empty() && resp.hovered() {
                set_hovered_tooltip(state, surface_name, &node.tooltip);
            }
        }

        "grid" => {
            let cols = node.columns.max(1);
            let row_gap = sp(node.style.gap.unwrap_or(6.0));
            let sb = ui
                .col(|ui| {
                    for (row_idx, chunk) in node.children.chunks(cols).enumerate() {
                        ui.row(|ui| {
                            for (c_idx, ch) in chunk.iter().enumerate() {
                                render_ext_node(
                                    ui,
                                    state,
                                    surface_name,
                                    ext_id,
                                    target_id,
                                    ch,
                                    row_idx * cols + c_idx,
                                    accent,
                                    actions,
                                );
                            }
                            for _ in chunk.len()..cols {
                                ui.col(|_| {}).grow(1.0);
                            }
                        })
                        .gap(row_gap);
                    }
                })
                .gap(row_gap);
            let _ = apply_style(sb, &node.style, accent);
        }

        "scroll" => {
            let mut sb = ui.scroll(&widget_id, |ui| {
                for (i, ch) in node.children.iter().enumerate() {
                    render_ext_node(
                        ui,
                        state,
                        surface_name,
                        ext_id,
                        target_id,
                        ch,
                        i,
                        accent,
                        actions,
                    );
                }
            });
            if node.hide_scrollbar {
                sb = sb.no_scrollbar();
            }
            let _ = apply_style(sb, &node.style, accent);
        }

        "hscroll" => {
            let mut sb = ui.hscroll(&widget_id, |ui| {
                for (i, ch) in node.children.iter().enumerate() {
                    render_ext_node(
                        ui,
                        state,
                        surface_name,
                        ext_id,
                        target_id,
                        ch,
                        i,
                        accent,
                        actions,
                    );
                }
            });
            if node.hide_scrollbar {
                sb = sb.no_scrollbar();
            }
            let _ = apply_style(sb, &node.style, accent);
        }

        "spacer" => {
            let mut sb = ui.col(|_| {});
            if node.style.w.is_none() && node.style.h.is_none() && node.style.grow.is_none() {
                sb = sb.grow(1.0);
            }
            let _ = apply_style(sb, &node.style, accent);
        }

        "divider" => {
            let col = resolve_color_token(&node.fg, accent).unwrap_or_else(border_subtle);
            let mut sb = ui.col(|_| {}).bg(col);
            if node.style.w.is_none() && node.style.h.is_none() {
                sb = sb.w_frac(1.0).h(1.0);
            }
            let _ = apply_style(sb, &node.style, accent);
        }

        "text" => {
            let sz = fs(node.size.unwrap_or(11.5));
            let col = resolve_color_token(&node.fg, accent).unwrap_or_else(text_primary);
            let mut sb = ui.txt(&node.text).sz(sz).fg(col);
            if !node.font.is_empty() {
                sb = sb.font(&node.font);
            }
            match node.weight.as_str() {
                "medium" => sb = sb.medium(),
                "semibold" => sb = sb.semibold(),
                "bold" => sb = sb.bold(),
                _ => {}
            }
            match node.text_align.as_str() {
                "center" => sb = sb.align(TextAlign::Center),
                "right" => sb = sb.align(TextAlign::Right),
                "left" => sb = sb.align(TextAlign::Left),
                _ => {}
            }
            if node.wrap {
                sb = sb.wrap();
            }
            if node.ellipsis {
                sb = sb.ellipsis();
            }
            let mut resp = apply_style(sb, &node.style, accent);
            if !node.tooltip.is_empty() && resp.hovered() {
                set_hovered_tooltip(state, surface_name, &node.tooltip);
            }
        }

        "icon" => {
            let sz = node.size.unwrap_or(16.0);
            let col = resolve_color_token(&node.fg, accent).unwrap_or_else(text_primary);
            let name = if node.icon.is_empty() {
                icons::PUZZLE
            } else {
                &node.icon
            };
            let sb = icons::icon(ui, name, sz).fg(col);
            let mut resp = apply_style(sb, &node.style, accent);
            if !node.tooltip.is_empty() && resp.hovered() {
                set_hovered_tooltip(state, surface_name, &node.tooltip);
            }
        }

        "image" => {
            let fit = match node.fit.as_str() {
                "contain" => ImageFit::Contain,
                "fill" => ImageFit::Fill,
                _ => ImageFit::Cover,
            };
            let tex = if surface_name == "main" {
                state.main_textures.get(&node.image_key)
            } else {
                state.quick_textures.get(&node.image_key)
            };
            if let Some(tex_id) = tex {
                let sb = ui.img(tex_id).fit(fit);
                let mut resp = apply_style(sb, &node.style, accent);
                if !node.tooltip.is_empty() && resp.hovered() {
                    set_hovered_tooltip(state, surface_name, &node.tooltip);
                }
            } else {
                let fallback_icon = if node.icon.is_empty() {
                    icons::PUZZLE
                } else {
                    &node.icon
                };
                let sb = ui
                    .col(|ui| {
                        icons::icon(ui, fallback_icon, node.size.unwrap_or(18.0))
                            .fg(text_dim());
                    })
                    .center()
                    .bg(card_inner_bg())
                    .bd(1.0, border_subtle())
                    .r(rad(6.0));
                let _ = apply_style(sb, &node.style, accent);
            }
        }

        "badge" => {
            let tint = resolve_color_token(&node.fg, accent).unwrap_or(accent);
            let sb = ui
                .row(|ui| {
                    if !node.icon.is_empty() {
                        icons::icon(ui, &node.icon, node.size.unwrap_or(11.0)).fg(tint);
                    }
                    ui.txt(&node.text)
                        .sz(fs(node.size.unwrap_or(10.0)))
                        .semibold()
                        .fg(text_primary());
                })
                .align_center()
                .gap(sp(4.0))
                .px(pad(7.0))
                .py(pad(2.5))
                .bg(tint_alpha(tint, 44))
                .bd(1.0, tint_alpha(tint, 155))
                .bevel_rounded(rad(5.0), bev(1.2));
            let _ = apply_style(sb, &node.style, accent);
        }

        "progress" => {
            let min_v = node.min;
            let max_v = if node.max > node.min { node.max } else { 1.0 };
            let frac = ((node.value - min_v) / (max_v - min_v)).clamp(0.0, 1.0);
            let fill_col = resolve_color_token(&node.fg, accent).unwrap_or(accent);
            let track_h = node.style.h.unwrap_or(7.0);
            let r = rad(node.style.radius.unwrap_or(track_h * 0.5));
            let sb = ui
                .row(|ui| {
                    if frac > 0.001 {
                        ui.col(|_| {})
                            .w_frac(frac)
                            .h_frac(1.0)
                            .bg(fill_col)
                            .r(r);
                    }
                })
                .w_frac(1.0)
                .h(track_h)
                .bg(card_inner_bg())
                .bd(1.0, border_subtle())
                .r(r)
                .clip();
            let _ = apply_style(sb, &node.style, accent);
        }

        "button" => {
            let (def_bg, def_hover, def_bd, def_fg) = if node.disabled {
                (
                    crate::theme::TILE_DISABLED_BG,
                    crate::theme::TILE_DISABLED_BG,
                    Color::rgba(100, 116, 139, 32),
                    text_dim(),
                )
            } else if node.active || node.variant == "accent" {
                (
                    tint_alpha(accent, 52),
                    tint_alpha(accent, 82),
                    tint_alpha(accent, 195),
                    text_primary(),
                )
            } else if node.variant == "danger" {
                (
                    tint_alpha(RED, 48),
                    tint_alpha(RED, 78),
                    tint_alpha(RED, 185),
                    text_primary(),
                )
            } else if node.variant == "emerald" {
                (
                    tint_alpha(EMERALD, 48),
                    tint_alpha(EMERALD, 78),
                    tint_alpha(EMERALD, 185),
                    text_primary(),
                )
            } else if node.variant == "ghost" {
                (
                    Color::TRANSPARENT,
                    tile_idle_hover(),
                    Color::TRANSPARENT,
                    text_primary(),
                )
            } else {
                (
                    tile_idle_bg(),
                    tile_idle_hover(),
                    border_subtle(),
                    text_primary(),
                )
            };

            let text_col = resolve_color_token(&node.fg, accent).unwrap_or(def_fg);
            let mut sb = ui
                .btn_with(&widget_id, |ui| {
                    if !node.children.is_empty() {
                        for (i, ch) in node.children.iter().enumerate() {
                            render_ext_node(
                                ui,
                                state,
                                surface_name,
                                ext_id,
                                target_id,
                                ch,
                                i,
                                accent,
                                actions,
                            );
                        }
                    } else {
                        ui.row(|ui| {
                            if !node.icon.is_empty() {
                                icons::icon(ui, &node.icon, node.size.unwrap_or(15.0))
                                    .fg(if node.active { accent } else { text_col });
                            }
                            if !node.text.is_empty() {
                                ui.txt(&node.text)
                                    .sz(fs(node.size.unwrap_or(11.5)))
                                    .semibold()
                                    .align(TextAlign::Center)
                                    .fg(text_col);
                            }
                        })
                        .center()
                        .gap(sp(5.0));
                    }
                })
                .px(pad(10.0))
                .py(pad(6.0))
                .bg(def_bg)
                .hover_bg(def_hover)
                .bd(if node.active { 1.5 } else { 1.0 }, def_bd)
                .bevel_rounded(rad(6.5), bev(1.3))
                .opacity(if node.disabled { 0.52 } else { 1.0 });

            let st = &node.style;
            if let Some(v) = st.w { sb = sb.w(v); }
            if let Some(v) = st.h { sb = sb.h(v); }
            if let Some(v) = st.w_frac { sb = sb.w_frac(v); }
            if let Some(v) = st.h_frac { sb = sb.h_frac(v); }
            if let Some(v) = st.min_w { sb = sb.min_w(v); }
            if let Some(v) = st.max_w { sb = sb.max_w(v); }
            if let Some(v) = st.min_h { sb = sb.min_h(v); }
            if let Some(v) = st.max_h { sb = sb.max_h(v); }
            if let Some(v) = st.grow { sb = sb.grow(v); }
            if let Some(v) = st.p { sb = sb.p(pad(v)); }
            if let Some(v) = st.px { sb = sb.px(pad(v)); }
            if let Some(v) = st.py { sb = sb.py(pad(v)); }
            if let Some(c) = resolve_color_token(&st.bg, accent) { sb = sb.bg(c); }
            if let Some(c) = resolve_color_token(&st.hover_bg, accent) { sb = sb.hover_bg(c); }
            if let Some(bw) = st.border_w {
                if bw <= 0.0 { sb = sb.borderless(); }
                else {
                    let bc = resolve_color_token(&st.border_color, accent).unwrap_or_else(border_subtle);
                    sb = sb.bd(bw, bc);
                }
            } else if let Some(bc) = resolve_color_token(&st.border_color, accent) {
                sb = sb.bd(1.0, bc);
            }
            if let Some(b) = st.bevel {
                sb = sb.bevel_rounded(rad(st.radius.unwrap_or(6.5)), bev(b));
            } else if let Some(r) = st.radius {
                sb = sb.r(rad(r));
            }
            if let Some(op) = st.opacity { sb = sb.opacity(op.clamp(0.0, 1.0)); }

            if sb.hovered() && !node.tooltip.is_empty() {
                set_hovered_tooltip(state, surface_name, &node.tooltip);
            }
            if !node.disabled && sb.clicked() {
                push_click(actions);
                push_ext_event(
                    actions,
                    ext_id,
                    target_id,
                    &local_id,
                    "click",
                    String::new(),
                    0.0,
                    node.active,
                );
            }
        }

        "toggle" => {
            let mut clicked = false;
            let sb = ui
                .row(|ui| {
                    if !node.icon.is_empty() {
                        icons::icon(ui, &node.icon, 14.0)
                            .fg(if node.active { accent } else { text_muted() });
                    }
                    ui.txt(&node.text)
                        .sz(fs(node.size.unwrap_or(11.0)))
                        .semibold()
                        .fg(text_primary())
                        .ellipsis()
                        .grow(1.0);
                    if ui
                        .btn_with(&format!("{widget_id}_btn"), |ui| {
                            ui.txt(if node.active { "ON" } else { "OFF" })
                                .sz(fs(10.0))
                                .bold()
                                .align(TextAlign::Center)
                                .fg(if node.active { EMERALD } else { text_muted() })
                                .grow(1.0);
                        })
                        .w(50.0)
                        .h(22.0)
                        .bg(if node.active {
                            tint_alpha(EMERALD, 45)
                        } else {
                            tile_idle_bg()
                        })
                        .hover_bg(tile_idle_hover())
                        .bd(
                            1.0,
                            if node.active {
                                tint_alpha(EMERALD, 190)
                            } else {
                                border_subtle()
                            },
                        )
                        .bevel_rounded(rad(5.0), bev(1.3))
                        .clicked()
                    {
                        clicked = true;
                    }
                })
                .align_center()
                .px(pad(8.0))
                .py(pad(5.0))
                .gap(sp(6.0))
                .bg(card_inner_bg())
                .bd(1.0, border_subtle())
                .bevel_rounded(rad(6.0), bev(1.3));
            let mut resp = apply_style(sb, &node.style, accent);
            if !node.tooltip.is_empty() && resp.hovered() {
                set_hovered_tooltip(state, surface_name, &node.tooltip);
            }
            if !node.disabled && clicked {
                push_click(actions);
                push_ext_event(
                    actions,
                    ext_id,
                    target_id,
                    &local_id,
                    "change",
                    String::new(),
                    0.0,
                    !node.active,
                );
            }
        }

        "slider" => {
            let min_v = node.min;
            let max_v = if node.max > node.min {
                node.max
            } else {
                min_v + 1.0
            };
            let host_v = node.value.clamp(min_v, max_v);
            let (cur_v, last_host, last_sent) = state
                .ext_sliders
                .entry(widget_id.clone())
                .or_insert((host_v, host_v, host_v));
            if (*last_host - host_v).abs() > 0.0001 {
                *last_host = host_v;
                if (*last_sent - host_v).abs() > 0.0001 {
                    *cur_v = host_v;
                    *last_sent = host_v;
                }
            }
            let mut v = *cur_v;
            let fill_col = resolve_color_token(&node.fg, accent).unwrap_or(accent);

            ui.push_id(&widget_id);
            let sb = ui
                .slider(&node.text, &mut v, min_v..=max_v)
                .fill(fill_col)
                .no_focus_ring()
                .track_h(14.0)
                .track_bg(tile_idle_bg())
                .track_bd(1.0, border_subtle())
                .bg(card_inner_bg())
                .bd(1.0, border_subtle())
                .bevel(rad(8.0));
            let mut resp = apply_style(sb, &node.style, accent);
            let hovered = resp.hovered();
            if !node.disabled && resp.changed() {
                let snapped = if node.step > 0.0 {
                    (((v - min_v) / node.step).round() * node.step + min_v).clamp(min_v, max_v)
                } else {
                    v.clamp(min_v, max_v)
                };
                *cur_v = snapped;
                if (snapped - *last_sent).abs() > 0.0001 {
                    *last_sent = snapped;
                    push_ext_event(
                        actions,
                        ext_id,
                        target_id,
                        &local_id,
                        "change",
                        String::new(),
                        snapped,
                        false,
                    );
                }
            }
            ui.pop_id();
            if hovered && !node.tooltip.is_empty() {
                set_hovered_tooltip(state, surface_name, &node.tooltip);
            }
        }

        "input" => {
            let slot = state
                .ext_text_inputs
                .entry(widget_id.clone())
                .or_insert_with(|| crate::state::ExtTextInputSlot::new(&node.text));
            if slot.last_host != node.text {
                slot.last_host = node.text.clone();
                if (!slot.was_focused || node.text.is_empty()) && slot.edit.text() != node.text {
                    slot.edit.set_text(&node.text);
                    slot.last_sent = node.text.clone();
                }
            }

            let mut sb = if node.password {
                ui.password_input(&widget_id, &mut slot.edit)
            } else {
                ui.input(&widget_id, &mut slot.edit)
            }
            .bg(card_inner_bg())
            .fg(text_primary())
            .bd(1.0, border_subtle())
            .bevel_rounded(rad(6.0), bev(1.3))
            .submit_on_enter();
            if !node.placeholder.is_empty() {
                sb = sb.placeholder(&node.placeholder);
            }
            if let Some(v) = node.style.w { sb = sb.w(v); }
            if let Some(v) = node.style.h { sb = sb.h(v); }
            if let Some(v) = node.style.w_frac { sb = sb.w_frac(v); }
            if let Some(v) = node.style.h_frac { sb = sb.h_frac(v); }
            if let Some(v) = node.style.min_w { sb = sb.min_w(v); }
            if let Some(v) = node.style.max_w { sb = sb.max_w(v); }
            if let Some(v) = node.style.grow { sb = sb.grow(v); }
            if let Some(c) = resolve_color_token(&node.style.bg, accent) { sb = sb.bg(c); }
            if let Some(op) = node.style.opacity { sb = sb.opacity(op.clamp(0.0, 1.0)); }
            let focused = sb.focused();
            let hovered = sb.hovered();
            let clicked = sb.clicked();
            let changed = sb.changed();
            let submitted = sb.submitted();
            slot.was_focused = focused;
            let current_text = slot.edit.text();

            if !node.disabled && submitted {
                slot.last_sent = current_text.clone();
                push_ext_event(
                    actions,
                    ext_id,
                    target_id,
                    &local_id,
                    "submit",
                    current_text.clone(),
                    0.0,
                    false,
                );
            } else if !node.disabled && changed && current_text != slot.last_sent {
                slot.last_sent = current_text.clone();
                push_ext_event(
                    actions,
                    ext_id,
                    target_id,
                    &local_id,
                    "change",
                    current_text.clone(),
                    0.0,
                    false,
                );
            }
            if hovered && !node.tooltip.is_empty() {
                set_hovered_tooltip(state, surface_name, &node.tooltip);
            }
            if state.host.is_vr && clicked && !node.disabled {
                let field_id = format!("ext|{surface_name}|{ext_id}|{target_id}|{local_id}");
                actions.push(UiAction::OpenVirtualKeyboard {
                    surface_id: if surface_name == "main" { 0 } else { 1 },
                    field_id,
                    initial_text: current_text,
                    title: if node.placeholder.is_empty() {
                        String::from("Input")
                    } else {
                        node.placeholder.clone()
                    },
                    placeholder: node.placeholder.clone(),
                    max_len: if node.max_len > 0 { node.max_len as i32 } else { 256 },
                    hidden: node.password,
                });
            }
        }

        "dropdown" => {
            let opt_refs: Vec<&str> = node.options.iter().map(|s| s.as_str()).collect();
            let mut sel = if opt_refs.is_empty() {
                0
            } else {
                node.selected_index.min(opt_refs.len() - 1)
            };

            let dd_style = build_dropdown_style(node.dropdown_style.as_ref(), accent);

            let (changed, hovered) = if node.searchable {
                let query_key = format!("{widget_id}_q");
                let slot = state
                    .ext_text_inputs
                    .entry(query_key)
                    .or_insert_with(|| crate::state::ExtTextInputSlot::new(""));

                let sb = ui
                    .combobox_styled(&widget_id, &mut sel, &mut slot.edit, &opt_refs, &dd_style)
                    .bg(tile_idle_bg())
                    .hover_bg(tile_idle_hover())
                    .fg(text_primary())
                    .bd(1.0, tint_alpha(accent, 140))
                    .bevel_rounded(rad(6.0), bev(1.3))
                    .enabled(!node.disabled);
                let mut resp = apply_style(sb, &node.style, accent);
                (!node.disabled && resp.changed(), resp.hovered())
            } else {
                let sb = ui
                    .dropdown_styled(&widget_id, &mut sel, &opt_refs, &dd_style)
                    .bg(tile_idle_bg())
                    .hover_bg(tile_idle_hover())
                    .fg(text_primary())
                    .bd(1.0, tint_alpha(accent, 140))
                    .bevel_rounded(rad(6.0), bev(1.3))
                    .enabled(!node.disabled);
                let mut resp = apply_style(sb, &node.style, accent);
                (!node.disabled && resp.changed(), resp.hovered())
            };

            if hovered && !node.tooltip.is_empty() {
                set_hovered_tooltip(state, surface_name, &node.tooltip);
            }
            if changed {
                push_click(actions);
                let chosen = node.options.get(sel).cloned().unwrap_or_default();
                push_ext_event(
                    actions,
                    ext_id,
                    target_id,
                    &local_id,
                    "change",
                    chosen,
                    sel as f32,
                    false,
                );
            }
        }

        "tabs" => {
            let stretch = node.style.w.is_some()
                || node.style.w_frac.is_some()
                || node.style.grow.is_some();
            let sb = ui
                .row(|ui| {
                    for (idx, opt) in node.options.iter().enumerate() {
                        let is_sel = idx == node.selected_index;
                        let mut tab_btn = ui
                            .btn_with(&format!("{widget_id}_t{idx}"), |ui| {
                                let mut t = ui
                                    .txt(opt)
                                    .sz(fs(node.size.unwrap_or(11.0)))
                                    .semibold()
                                    .align(TextAlign::Center)
                                    .fg(if is_sel {
                                        text_primary()
                                    } else {
                                        text_muted()
                                    });
                                if stretch {
                                    t = t.grow(1.0);
                                }
                                let _ = t;
                            })
                            .py(pad(5.5))
                            .px(pad(11.0))
                            .bg(if is_sel {
                                tint_alpha(accent, 52)
                            } else {
                                tile_idle_bg()
                            })
                            .hover_bg(tile_idle_hover())
                            .bd(
                                1.0,
                                if is_sel {
                                    tint_alpha(accent, 195)
                                } else {
                                    border_subtle()
                                },
                            )
                            .bevel_rounded(rad(5.5), bev(1.3));
                        if stretch {
                            tab_btn = tab_btn.grow(1.0);
                        }
                        if tab_btn.clicked() && !node.disabled {
                            push_click(actions);
                            push_ext_event(
                                actions,
                                ext_id,
                                target_id,
                                &local_id,
                                "change",
                                opt.clone(),
                                idx as f32,
                                false,
                            );
                        }
                    }
                })
                .gap(sp(5.0));
            let _ = apply_style(sb, &node.style, accent);
        }

        _ => {}
    }
}

fn build_dropdown_style(ov: Option<&ExtDropdownStyle>, accent: Color) -> DropdownStyle {
    let mut ds = DropdownStyle::new()
        .bg(Color::rgba(15, 19, 28, 248))
        .bd(1.2, tint_alpha(accent, 175))
        .bevel_rounded(rad(8.0), bev(1.5))
        .p(pad(6.0))
        .gap(sp(3.0))
        .max_h(260.0)
        .sz(fs(11.0))
        .item_h(crate::theme::text_box(30.0))
        .item_bg(Color::TRANSPARENT)
        .item_hover_bg(tile_idle_hover())
        .item_fg(text_primary())
        .item_bevel_rounded(rad(5.0), bev(1.2))
        .selected_bg(tint_alpha(accent, 52))
        .selected_hover_bg(tint_alpha(accent, 78))
        .selected_fg(text_primary())
        .selected_bd(1.0, tint_alpha(accent, 195))
        .shadow(16.0, Color::rgba(0, 0, 0, 185));

    if let Some(o) = ov {
        if let Some(c) = resolve_color_token(&o.bg, accent) {
            ds = ds.bg(c);
        }
        if let Some(bw) = o.border_w {
            let bc = resolve_color_token(&o.border_color, accent)
                .unwrap_or_else(|| tint_alpha(accent, 175));
            ds = ds.bd(bw, bc);
        } else if let Some(bc) = resolve_color_token(&o.border_color, accent) {
            ds = ds.bd(1.2, bc);
        }
        if let Some(b) = o.bevel {
            ds = ds.bevel_rounded(rad(o.radius.unwrap_or(8.0)), bev(b));
        } else if let Some(r) = o.radius {
            ds = ds.r(rad(r));
        }
        if let Some(v) = o.max_h {
            ds = ds.max_h(v);
        }
        if let Some(v) = o.min_w {
            ds = ds.min_w(v);
        }
        if let Some(v) = o.padding {
            ds = ds.p(pad(v));
        }
        if let Some(v) = o.gap {
            ds = ds.gap(sp(v));
        }
        if let Some(v) = o.font_size {
            ds = ds.sz(fs(v));
        }
        if let Some(v) = o.item_h {
            ds = ds.item_h(v);
        }
        if let Some(c) = resolve_color_token(&o.item_bg, accent) {
            ds = ds.item_bg(c);
        }
        if let Some(c) = resolve_color_token(&o.item_hover_bg, accent) {
            ds = ds.item_hover_bg(c);
        }
        if let Some(c) = resolve_color_token(&o.item_fg, accent) {
            ds = ds.item_fg(c);
        }
        if let Some(c) = resolve_color_token(&o.selected_bg, accent) {
            ds = ds.selected_bg(c);
        }
        if let Some(c) = resolve_color_token(&o.selected_hover_bg, accent) {
            ds = ds.selected_hover_bg(c);
        }
        if let Some(c) = resolve_color_token(&o.selected_fg, accent) {
            ds = ds.selected_fg(c);
        }
        if let Some(c) = resolve_color_token(&o.selected_border, accent) {
            ds = ds.selected_bd(1.0, c);
        }
    }
    ds
}
