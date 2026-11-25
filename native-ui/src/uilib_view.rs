use lumenic::{Color, ImageFit, TextAlign, TextEditState, UiContext};

use crate::icons;
use crate::state::{
    AppState, UiAction, UiLibCategory, UiLibElement, UiLibModalSnapshot, UiLibPageSnapshot,
};
use crate::theme::{
    push_click, tint_alpha, border_subtle, card_bg, card_inner_bg, EMERALD, RED, SHADOW_CARD,
    TEXT_DIM, TEXT_MUTED, TEXT_PRIMARY, tile_idle_bg, tile_idle_hover,
};

fn parse_hex_rgb(hex: &str) -> Option<Color> {
    let clean = hex.trim().trim_start_matches('#');
    if clean.len() < 6 {
        return None;
    }
    let r = u8::from_str_radix(&clean[0..2], 16).ok()?;
    let g = u8::from_str_radix(&clean[2..4], 16).ok()?;
    let b = u8::from_str_radix(&clean[4..6], 16).ok()?;
    Some(Color::rgb(r, g, b))
}

fn render_icon_or_fallback(
    ui: &mut UiContext,
    state: &AppState,
    icon_key: &str,
    size: f32,
    tint: Color,
    rounded: f32,
    fit: ImageFit,
) {
    if !icon_key.is_empty() {
        if let Some(tex) = state.quick_textures.get(icon_key) {
            ui.img(tex).w(size).h(size).fit(fit).bevel(rounded);
            return;
        }
    }
    icons::icon(ui, icons::PUZZLE, (size * 0.72).max(14.0)).fg(tint);
}

/// Renders an active UILib mod page inside the Quick Menu center area.
pub fn render_uilib_page(
    ui: &mut UiContext,
    state: &mut AppState,
    page: &UiLibPageSnapshot,
    accent: Color,
    actions: &mut Vec<UiAction>,
) {
    let is_player_details =
        page.id == "CVRUI-QMUI-PlayerSelectPage" && page.categories.len() >= 2;
    let is_prop_list = page.id == "CVRUI-QMUI-PropList";
    let is_prop_undo_redo = |hb: &UiLibElement| -> bool {
        let tt = hb.tooltip.to_ascii_lowercase();
        tt.contains("undo") || tt.contains("redo")
    };
    if !is_player_details {
        state.quick.studio_cutout_rect = None;
        if state.quick.last_studio_target != "none" {
            state.quick.last_studio_target = String::from("none");
            state.quick.last_studio_bounds = [0.0; 7];
            actions.push(UiAction::SyncStudioStage {
                target: String::from("none"),
                local_x: 0.0,
                local_y: 0.0,
                height_units: 0.0,
                min_x: 0.0,
                max_x: 0.0,
                min_y: 0.0,
                max_y: 0.0,
            });
        }
    }

    ui.col(|ui| {
        // --- PAGE HEADER (Back button, Title/Subtitle, Breadcrumbs, Header Buttons) ---
        ui.row(|ui| {
            if page.can_go_back {
                if ui
                    .btn_with("uilib_back_btn", |ui| {
                        ui.row(|ui| {
                            icons::icon(ui, icons::BACK, 14.0).fg(TEXT_PRIMARY);
                            ui.txt("Back").sz(crate::theme::fs(11.5)).semibold().fg(TEXT_PRIMARY);
                        })
                        .align_center()
                        .gap(4.0);
                    })
                    .px(9.0)
                    .py(5.0)
                    .bg(tile_idle_bg())
                    .hover_bg(tint_alpha(accent, 75))
                    .bd(1.0, border_subtle())
                    .bevel_rounded(6.2, 1.3)
                    .clicked()
                {
                    push_click(actions);
                    actions.push(UiAction::UiLibGoBack);
                }
            }

            ui.col(|ui| {
                let title = if page.title.is_empty() {
                    if page.mod_name.is_empty() {
                        "Mod Page"
                    } else {
                        &page.mod_name
                    }
                } else {
                    &page.title
                };
                ui.txt(title).sz(crate::theme::fs(13.5)).bold().fg(TEXT_PRIMARY);

                if !page.subtitle.is_empty() {
                    ui.txt(&page.subtitle).sz(crate::theme::fs(10.5)).fg(TEXT_MUTED);
                } else if page.breadcrumbs.len() > 1 {
                    let crumb = page.breadcrumbs.join(" / ");
                    ui.txt(&crumb).sz(crate::theme::fs(10.0)).fg(TEXT_DIM);
                }
            })
            .grow(1.0)
            .gap(1.0);

            let visible_header_buttons: Vec<&UiLibElement> = page
                .header_buttons
                .iter()
                .filter(|hb| !(is_prop_list && is_prop_undo_redo(hb)))
                .collect();

            if is_player_details || !visible_header_buttons.is_empty() {
                ui.row(|ui| {
                    if is_player_details {
                        if ui
                            .btn_with("uilib_player_open_mm", |ui| {
                                ui.row(|ui| {
                                    icons::icon(ui, icons::HOME, 13.5).fg(accent);
                                    ui.txt("Main Menu").sz(crate::theme::fs(11.0)).semibold().fg(TEXT_PRIMARY);
                                })
                                .align_center()
                                .gap(5.0);
                            })
                            .px(9.0)
                            .py(5.5)
                            .bg(tint_alpha(accent, 38))
                            .hover_bg(tint_alpha(accent, 78))
                            .bd(1.0, tint_alpha(accent, 150))
                            .bevel_rounded(6.0, 1.3)
                            .clicked()
                        {
                            push_click(actions);
                            actions.push(UiAction::OpenPlayerInMainMenu);
                        }
                    }
                    for hb in &visible_header_buttons {
                        render_compact_header_button(ui, state, hb, accent, actions);
                    }
                })
                .align_center()
                .gap(5.0);
            }
        })
        .align_center()
        .px(10.0)
        .py(6.0)
        .gap(8.0)
        .bg(card_bg())
        .bd(1.0, tint_alpha(accent, 95))
        .bevel_rounded(8.6, 1.3)
        .shadow(10.0, SHADOW_CARD);

        // --- FULL-HEIGHT 2-COLUMN SPLIT ON PLAYER DETAILS (LEFT: INFO + BIO + ACTIONS + MODERATION | RIGHT: FULL-HEIGHT 3D AVATAR STUDIO) ---
        if is_player_details {
            let scroll_id = format!("uilib_page_scroll_{}", page.id);
            ui.row(|ui| {
                // Left column: Scrollable Player Identity + Bio, Quick Actions, and Moderation/Interactions
                ui.scroll(&scroll_id, |ui| {
                    render_player_identity_card(
                        ui,
                        state,
                        page,
                        &page.categories[0],
                        accent,
                        actions,
                    );

                    render_player_quick_actions_card(
                        ui,
                        state,
                        &page.categories[1],
                        accent,
                        actions,
                    );

                    crate::ext_view::render_slot_only(
                        ui,
                        state,
                        "quick",
                        "qm.player_details.actions",
                        accent,
                        actions,
                    );

                    for cat in page.categories.iter().skip(2) {
                        render_uilib_category(ui, state, cat, accent, 2, actions);
                    }
                })
                .grow(1.0)
                .h_frac(1.0)
                .gap(7.0)
                .no_scrollbar();

                // Right column: Full-height GPU-composited 3D Avatar Turntable
                let player_target = if page.selected_player_id.is_empty() {
                    String::from("local")
                } else {
                    format!("player:{}", page.selected_player_id)
                };
                ui.col(|ui| {
                    ui.row(|ui| {
                        icons::icon(ui, icons::AVATAR, 14.0).fg(accent);
                        ui.txt("3D Avatar")
                            .sz(crate::theme::fs(12.0))
                            .semibold()
                            .fg(TEXT_PRIMARY)
                            .grow(1.0);
                    })
                    .align_center()
                    .gap(6.0);

                    crate::avatar_settings_view::render_avatar_turntable_stage(
                        ui,
                        state,
                        1,
                        &player_target,
                        "qm_player_turntable_drag",
                        &page.player_preview_status,
                        accent,
                        actions,
                    );
                })
                .w(246.0)
                .h_frac(1.0)
                .p(10.0)
                .gap(7.0)
                .bg(card_bg())
                .bd(1.0, border_subtle())
                .bevel_rounded(10.9, 2.0)
                .shadow(14.0, SHADOW_CARD);
            })
            .grow(1.0)
            .gap(8.0);
        } else {
            // --- STANDARD FULL-WIDTH SCROLLABLE PAGE BODY ---
            let scroll_id = format!("uilib_page_scroll_{}", page.id);
            ui.scroll(&scroll_id, |ui| {
                // Native colour preview swatch for CVR's built-in Colour Picker page
                if let Some(preview_col) = parse_hex_rgb(&page.colour_preview_hex) {
                    ui.row(|ui| {
                        ui.txt("")
                            .w(44.0)
                            .h(26.0)
                            .bg(preview_col)
                            .bd(1.5, Color::rgba(255, 255, 255, 160))
                            .bevel_rounded(5.5, 1.3);
                        ui.col(|ui| {
                            ui.txt("Colour Preview").sz(crate::theme::fs(11.5)).semibold().fg(TEXT_PRIMARY);
                            let hex_label =
                                format!("#{}", page.colour_preview_hex.trim_start_matches('#'));
                            ui.txt(&hex_label).sz(crate::theme::fs(11.0)).fg(TEXT_MUTED);
                        })
                        .gap(1.0)
                        .grow(1.0);
                    })
                    .align_center()
                    .p(8.0)
                    .gap(8.0)
                    .bg(card_bg())
                    .bd(1.0, border_subtle())
                    .bevel_rounded(7.8, 1.3);
                }

                // Page-level sliders (outside categories)
                if !page.top_sliders.is_empty() {
                    ui.col(|ui| {
                        for el in &page.top_sliders {
                            render_slider_element(ui, state, el, accent, actions);
                        }
                    })
                    .p(9.0)
                    .gap(6.0)
                    .bg(card_bg())
                    .bd(1.0, border_subtle())
                    .bevel_rounded(8.6, 1.3);
                }

                if page.categories.is_empty() && page.top_sliders.is_empty() {
                    ui.col(|ui| {
                        icons::icon(ui, icons::PUZZLE, 24.0)
                            .fg(Color::rgba(148, 163, 184, 75));
                        ui.txt("Empty Page").sz(crate::theme::fs(12.5)).semibold().fg(TEXT_MUTED);
                        ui.txt("This UILib page currently has no visible elements.")
                            .sz(crate::theme::fs(11.0))
                            .fg(TEXT_DIM);
                    })
                    .center()
                    .py(36.0)
                    .gap(5.0)
                    .bg(card_bg())
                    .bd(1.0, border_subtle())
                    .bevel_rounded(8.6, 1.3);
                }

                let prop_undo_redo: Vec<&UiLibElement> = if is_prop_list {
                    page.header_buttons
                        .iter()
                        .filter(|hb| is_prop_undo_redo(hb))
                        .collect()
                } else {
                    Vec::new()
                };

                for (cat_idx, cat) in page.categories.iter().enumerate() {
                    render_uilib_category(ui, state, cat, accent, 4, actions);

                    // On Props page, place Undo / Redo arrows directly under Quick Actions and above the Prop List.
                    if is_prop_list && cat_idx == 0 && !prop_undo_redo.is_empty() {
                        ui.row(|ui| {
                            for hb in &prop_undo_redo {
                                let is_redo = hb.tooltip.to_ascii_lowercase().contains("redo");
                                let lbl = if is_redo { "Redo" } else { "Undo" };
                                let id = format!("uilib_prop_hist_{}", hb.id);
                                let resp = ui
                                    .btn_with(&id, |ui| {
                                        ui.row(|ui| {
                                            if !is_redo {
                                                icons::icon(ui, icons::BACK, 13.5)
                                                    .fg(if hb.disabled { TEXT_DIM } else { accent });
                                            }
                                            ui.txt(lbl)
                                                .sz(crate::theme::fs(11.5))
                                                .semibold()
                                                .fg(if hb.disabled { TEXT_DIM } else { TEXT_PRIMARY });
                                            if is_redo {
                                                icons::icon(ui, icons::CHEVRON_RIGHT, 13.5)
                                                    .fg(if hb.disabled { TEXT_DIM } else { accent });
                                            }
                                        })
                                        .center()
                                        .gap(5.0)
                                        .grow(1.0);
                                    })
                                    .grow(1.0)
                                    .py(6.5)
                                    .px(10.0)
                                    .bg(tile_idle_bg())
                                    .hover_bg(tile_idle_hover())
                                    .bd(1.0, border_subtle())
                                    .bevel_rounded(6.5, 1.3)
                                    .opacity(if hb.disabled { 0.45 } else { 1.0 });

                                if resp.hovered() && !hb.tooltip.is_empty() {
                                    state.quick.hovered_tooltip = hb.tooltip.clone();
                                }
                                if !hb.disabled && resp.clicked() {
                                    push_click(actions);
                                    actions.push(UiAction::UiLibClick {
                                        element_id: hb.id.clone(),
                                    });
                                }
                            }
                        })
                        .align_center()
                        .p(7.0)
                        .gap(6.0)
                        .bg(card_bg())
                        .bd(1.0, border_subtle())
                        .bevel_rounded(8.6, 1.3);
                    }
                }
            })
            .grow(1.0)
            .gap(7.0)
            .no_scrollbar();
        }
    })
    .grow(1.0)
    .gap(7.0);
}

/// Renders Category 0 ("Player Info") + Bio on Quick Menu Player Details in a cohesive
/// Big Menu style: hexagonal PFP, bold name, rank/pronoun pills, metadata, and integrated bio.
fn render_player_identity_card(
    ui: &mut UiContext,
    state: &mut AppState,
    page: &crate::state::UiLibPageSnapshot,
    cat: &UiLibCategory,
    accent: Color,
    actions: &mut Vec<UiAction>,
) {
    let is_local_player = page.selected_player_id.is_empty()
        || (!state.host.user_id.is_empty()
            && page
                .selected_player_id
                .eq_ignore_ascii_case(&state.host.user_id));
    let details_el = cat
        .elements
        .iter()
        .find(|el| el.kind == "content_details");

    ui.push_id(&cat.id);
    ui.col(|ui| {
        if let Some(el) = details_el {
            ui.row(|ui| {
                // Hexagonal PFP matching Big Menu's user_detail.rs
                let img_id = format!("uilib_cdet_img_{}", el.id);
                let pfp_tex = if !el.icon_key.is_empty() {
                    state.quick_textures.get(&el.icon_key)
                } else {
                    None
                }
                .or_else(|| {
                    if is_local_player {
                        state.quick_textures.get("local_pfp")
                    } else {
                        None
                    }
                });

                let ring = if is_local_player { EMERALD } else { accent };
                let pfp_resp = ui.btn_with(&img_id, |ui| {
                    if let Some(tex) = pfp_tex {
                        ui.img(tex)
                            .w(60.0)
                            .h(60.0)
                            .fit(ImageFit::Cover)
                            .hexagon_pointy()
                            .r(3.5)
                            .bd(2.0, ring);
                    } else {
                        let initial = el
                            .detail_blocks
                            .first()
                            .and_then(|b| b.text.chars().next())
                            .or_else(|| page.title.chars().next())
                            .unwrap_or('?')
                            .to_ascii_uppercase()
                            .to_string();
                        ui.col(|ui| {
                            ui.txt(&initial)
                                .sz(crate::theme::fs(21.0))
                                .bold()
                                .align(TextAlign::Center)
                                .fg(ring);
                        })
                        .center()
                        .w(60.0)
                        .h(60.0)
                        .bg(tint_alpha(ring, 32))
                        .bd(2.0, ring)
                        .hexagon_pointy()
                        .r(3.5);
                    }
                })
                .bg(Color::TRANSPARENT)
                .hover_bg(Color::TRANSPARENT)
                .borderless();

                if !el.disabled && pfp_resp.clicked() {
                    push_click(actions);
                    actions.push(UiAction::UiLibClick {
                        element_id: el.id.clone(),
                    });
                }

                // Identity lines: Name + Rank/Pronouns + remaining detail blocks
                ui.col(|ui| {
                    let mut name_text: Option<&str> = None;
                    let mut badge_blocks: Vec<&crate::state::UiLibDetailBlock> = Vec::new();
                    let mut info_blocks: Vec<&crate::state::UiLibDetailBlock> = Vec::new();

                    for blk in &el.detail_blocks {
                        if blk.header.is_empty() && blk.text.is_empty() {
                            continue;
                        }
                        if name_text.is_none() && !blk.badge && blk.header.is_empty() {
                            name_text = Some(blk.text.as_str());
                        } else if blk.badge {
                            badge_blocks.push(blk);
                        } else {
                            info_blocks.push(blk);
                        }
                    }

                    let display_name = name_text.unwrap_or(if page.title.is_empty() {
                        "Player"
                    } else {
                        page.title.as_str()
                    });
                    ui.txt(display_name)
                        .sz(crate::theme::fs(15.5))
                        .bold()
                        .fg(TEXT_PRIMARY)
                        .ellipsis();

                    if !badge_blocks.is_empty() {
                        ui.row(|ui| {
                            for blk in &badge_blocks {
                                if !blk.text.is_empty() {
                                    ui.row(|ui| {
                                        if !blk.header.is_empty() {
                                            ui.txt(&blk.header)
                                                .sz(crate::theme::fs(9.5))
                                                .fg(TEXT_MUTED);
                                        }
                                        ui.txt(&blk.text)
                                            .sz(crate::theme::fs(9.5))
                                            .semibold()
                                            .fg(TEXT_PRIMARY);
                                    })
                                    .align_center()
                                    .gap(4.0)
                                    .px(6.5)
                                    .py(2.0)
                                    .bg(tint_alpha(accent, 42))
                                    .bd(1.0, tint_alpha(accent, 150))
                                    .bevel_rounded(4.5, 1.2);
                                }
                            }
                        })
                        .align_center()
                        .gap(5.0);
                    }

                    for blk in &info_blocks {
                        ui.row(|ui| {
                            if !blk.header.is_empty() {
                                ui.txt(&blk.header)
                                    .sz(crate::theme::fs(10.5))
                                    .semibold()
                                    .fg(TEXT_MUTED);
                            }
                            if !blk.text.is_empty() {
                                ui.txt(&blk.text)
                                    .sz(crate::theme::fs(10.5))
                                    .fg(TEXT_PRIMARY)
                                    .ellipsis();
                            }
                        })
                        .align_center()
                        .gap(5.0);
                    }
                })
                .gap(3.0)
                .grow(1.0);
            })
            .align_center()
            .gap(11.0);

            if !el.extra_buttons.is_empty() {
                ui.row(|ui| {
                    for eb in &el.extra_buttons {
                        let ebid = format!("uilib_cdet_btn_{}", eb.id);
                        let eb_resp = ui
                            .btn_with(&ebid, |ui| {
                                ui.txt(&eb.label)
                                    .sz(crate::theme::fs(10.5))
                                    .semibold()
                                    .fg(TEXT_PRIMARY);
                            })
                            .px(8.0)
                            .py(5.0)
                            .bg(tile_idle_bg())
                            .hover_bg(tint_alpha(accent, 75))
                            .bd(1.0, border_subtle())
                            .bevel_rounded(5.5, 1.3)
                            .opacity(if eb.disabled { 0.45 } else { 1.0 });
                        if !eb.disabled && eb_resp.clicked() {
                            push_click(actions);
                            actions.push(UiAction::UiLibClick {
                                element_id: eb.id.clone(),
                            });
                        }
                    }
                })
                .gap(5.0);
            }
        }

        // Render any non-content_details elements in category 0 if present.
        for el in cat.elements.iter().filter(|e| e.kind != "content_details") {
            render_full_width_element(ui, state, el, accent, actions);
        }

        // Integrated Bio sub-card (matches Big Menu user_detail.rs)
        ui.col(|ui| {
            ui.row(|ui| {
                ui.txt("Bio")
                    .sz(crate::theme::fs(10.5))
                    .bold()
                    .fg(TEXT_MUTED)
                    .grow(1.0);
                if is_local_player
                    && ui
                        .btn_with("uilib_player_edit_profile", |ui| {
                            ui.txt("Edit")
                                .sz(crate::theme::fs(10.0))
                                .semibold()
                                .fg(TEXT_PRIMARY);
                        })
                        .px(8.0)
                        .py(2.5)
                        .bg(tint_alpha(accent, 38))
                        .hover_bg(tint_alpha(accent, 78))
                        .bd(1.0, tint_alpha(accent, 150))
                        .bevel_rounded(5.0, 1.2)
                        .clicked()
                {
                    push_click(actions);
                    state.main_menu.user_detail_tab = 0;
                    state.main_menu.profile_editing = true;
                    actions.push(UiAction::OpenPlayerInMainMenu);
                }
            })
            .align_center()
            .gap(6.0);

            let bio_trimmed = page.player_bio.trim();
            if bio_trimmed.is_empty() {
                ui.txt("No bio set.")
                    .sz(crate::theme::fs(10.5))
                    .fg(TEXT_DIM);
            } else {
                ui.txt(bio_trimmed)
                    .sz(crate::theme::fs(11.0))
                    .fg(TEXT_PRIMARY)
                    .wrap();
            }
        })
        .p(8.0)
        .gap(4.0)
        .bg(card_inner_bg())
        .bd(1.0, border_subtle())
        .bevel_rounded(7.0, 1.3);
    })
    .p(10.0)
    .gap(8.0)
    .bg(card_bg())
    .bd(1.0, border_subtle())
    .bevel_rounded(10.5, 2.0)
    .shadow(14.0, SHADOW_CARD);
    ui.pop_id();
}

/// Renders Category 1 ("Quick Actions": Avatar button + Player Voice Volume slider) on Player Details
/// as a compact horizontal row so it fits neatly beneath Player Info in the left split column.
fn render_player_quick_actions_card(
    ui: &mut UiContext,
    state: &mut AppState,
    cat: &UiLibCategory,
    accent: Color,
    actions: &mut Vec<UiAction>,
) {
    ui.push_id(&cat.id);
    ui.row(|ui| {
        for el in &cat.elements {
            match el.kind.as_str() {
                "button" => {
                    let id = format!("uilib_pqa_btn_{}", el.id);
                    let resp = ui
                        .btn_with(&id, |ui| {
                            ui.row(|ui| {
                                render_icon_or_fallback(
                                    ui,
                                    state,
                                    &el.icon_key,
                                    28.0,
                                    accent,
                                    6.0,
                                    ImageFit::Cover,
                                );
                                ui.col(|ui| {
                                    ui.txt("Avatar").sz(crate::theme::fs(9.5)).semibold().fg(TEXT_MUTED);
                                    let lbl = if el.label.is_empty() {
                                        "Details"
                                    } else {
                                        &el.label
                                    };
                                    ui.txt(lbl)
                                        .sz(crate::theme::fs(11.0))
                                        .semibold()
                                        .fg(TEXT_PRIMARY)
                                        .ellipsis();
                                })
                                .gap(1.0)
                                .grow(1.0);
                            })
                            .align_center()
                            .gap(7.0)
                            .grow(1.0);
                        })
                        .w(165.0)
                        .px(8.0)
                        .py(6.0)
                        .bg(tile_idle_bg())
                        .hover_bg(tile_idle_hover())
                        .bd(1.0, border_subtle())
                        .bevel_rounded(7.0, 1.3)
                        .opacity(if el.disabled { 0.45 } else { 1.0 });

                    if !el.disabled && resp.clicked() {
                        push_click(actions);
                        actions.push(UiAction::UiLibClick {
                            element_id: el.id.clone(),
                        });
                    }
                }
                "slider" => {
                    ui.col(|ui| {
                        render_slider_element(ui, state, el, accent, actions);
                    })
                    .grow(1.0);
                }
                _ => {}
            }
        }
    })
    .align_center()
    .p(8.0)
    .gap(8.0)
    .bg(card_bg())
    .bd(1.0, border_subtle())
    .bevel_rounded(8.6, 1.3);
    ui.pop_id();
}

fn render_compact_header_button(
    ui: &mut UiContext,
    state: &AppState,
    el: &UiLibElement,
    accent: Color,
    actions: &mut Vec<UiAction>,
) {
    let id = format!("uilib_hb_{}", el.id);
    let is_open_main_menu = el.tooltip.to_ascii_lowercase().contains("main menu");
    let resp = if is_open_main_menu {
        ui.btn_with(&id, |ui| {
            ui.row(|ui| {
                icons::icon(ui, icons::HOME, 13.5).fg(accent);
                ui.txt("Main Menu").sz(crate::theme::fs(11.0)).semibold().fg(TEXT_PRIMARY);
            })
            .align_center()
            .gap(5.0);
        })
        .px(9.0)
        .py(5.5)
        .bg(tint_alpha(accent, 38))
        .hover_bg(tint_alpha(accent, 78))
        .bd(1.0, tint_alpha(accent, 150))
        .bevel_rounded(6.0, 1.3)
        .opacity(if el.disabled { 0.45 } else { 1.0 })
    } else {
        ui.btn_with(&id, |ui| {
            ui.row(|ui| {
                if !el.icon_key.is_empty() {
                    render_icon_or_fallback(
                        ui,
                        state,
                        &el.icon_key,
                        16.0,
                        accent,
                        4.0,
                        ImageFit::Contain,
                    );
                }
                if !el.label.is_empty() {
                    ui.txt(&el.label).sz(crate::theme::fs(11.5)).semibold().fg(TEXT_PRIMARY);
                }
            })
            .align_center()
            .gap(5.0);
        })
        .px(8.0)
        .py(6.0)
        .bg(tile_idle_bg())
        .hover_bg(tint_alpha(accent, 75))
        .bd(1.0, border_subtle())
        .bevel_rounded(6.2, 1.3)
        .opacity(if el.disabled { 0.45 } else { 1.0 })
    };

    if !el.disabled && resp.clicked() {
        push_click(actions);
        actions.push(UiAction::UiLibClick {
            element_id: el.id.clone(),
        });
    }
}

fn render_uilib_category(
    ui: &mut UiContext,
    state: &mut AppState,
    cat: &UiLibCategory,
    accent: Color,
    max_cols: i32,
    actions: &mut Vec<UiAction>,
) {
    ui.push_id(&cat.id);
    ui.col(|ui| {
        if cat.show_header && !cat.name.is_empty() {
            ui.row(|ui| {
                if cat.can_collapse {
                    let toggle_id = format!("uilib_cat_hdr_{}", cat.id);
                    if ui
                        .btn_with(&toggle_id, |ui| {
                            ui.row(|ui| {
                                let chev = if cat.collapsed {
                                    icons::CHEVRON_RIGHT
                                } else {
                                    icons::CHEVRON_DOWN
                                };
                                icons::icon(ui, chev, 13.0).fg(accent);
                                ui.txt(&cat.name).sz(crate::theme::fs(12.0)).bold().fg(TEXT_PRIMARY);
                            })
                            .align_center()
                            .gap(5.0);
                        })
                        .px(4.0)
                        .py(2.0)
                        .bg(Color::TRANSPARENT)
                        .hover_bg(Color::rgba(148, 163, 184, 24))
                        .bevel_rounded(4.7, 1.3)
                        .grow(1.0)
                        .clicked()
                    {
                        push_click(actions);
                        actions.push(UiAction::UiLibToggleCategory {
                            category_id: cat.id.clone(),
                        });
                    }
                } else {
                    ui.txt(&cat.name)
                        .sz(crate::theme::fs(12.0))
                        .bold()
                        .fg(TEXT_PRIMARY)
                        .grow(1.0);
                }

                if !cat.header_buttons.is_empty() {
                    ui.row(|ui| {
                        for hb in &cat.header_buttons {
                            render_compact_header_button(ui, state, hb, accent, actions);
                        }
                    })
                    .align_center()
                    .gap(5.0);
                }
            })
            .align_center()
            .gap(5.0);
        }

        if cat.collapsed {
            return;
        }

        // Partition elements into runs of compact grid items (buttons/toggles/cycles)
        // vs full-width items (sliders, text blocks, text inputs, content buttons/details, spacers).
        let mut idx = 0usize;
        while idx < cat.elements.len() {
            let el = &cat.elements[idx];
            if is_grid_tile(&el.kind) {
                let cols_limit = max_cols.max(1);
                let mut row_items: Vec<&UiLibElement> = Vec::with_capacity(cols_limit as usize);
                let mut used_cols = 0i32;
                while idx < cat.elements.len() && is_grid_tile(&cat.elements[idx].kind) {
                    let span = tile_span(&cat.elements[idx]).min(cols_limit);
                    if !row_items.is_empty() && used_cols + span > cols_limit {
                        break;
                    }
                    row_items.push(&cat.elements[idx]);
                    used_cols += span;
                    idx += 1;
                    if used_cols >= cols_limit {
                        break;
                    }
                }

                ui.row(|ui| {
                    for item in &row_items {
                        let weight = tile_span(item).min(cols_limit) as f32;
                        render_grid_tile(ui, state, item, accent, weight, actions);
                    }
                    // Keep partial rows left-aligned instead of stretching 1 button across the full card.
                    if used_cols < cols_limit {
                        ui.txt("").grow((cols_limit - used_cols) as f32);
                    }
                })
                .gap(6.0);
            } else {
                render_full_width_element(ui, state, el, accent, actions);
                idx += 1;
            }
        }
    })
    .p(9.0)
    .gap(6.0)
    .bg(card_bg())
    .bd(1.0, border_subtle())
    .bevel_rounded(8.6, 1.3);
    ui.pop_id();
}

fn is_grid_tile(kind: &str) -> bool {
    matches!(kind, "button" | "toggle" | "cycle")
}

fn tile_span(el: &UiLibElement) -> i32 {
    // UILib uses a 12-column Bootstrap grid (default 3 cols = 4 tiles per row).
    if el.col_span <= 0 {
        return 1;
    }
    match el.col_span {
        1..=3 => 1,
        4..=6 => 2,
        7..=9 => 3,
        _ => 4,
    }
}

fn render_grid_tile(
    ui: &mut UiContext,
    state: &mut AppState,
    el: &UiLibElement,
    accent: Color,
    grow_weight: f32,
    actions: &mut Vec<UiAction>,
) {
    let id = format!("uilib_el_{}", el.id);
    match el.kind.as_str() {
        "toggle" => {
            let active = el.bool_val;
            let (bg, hover_bg, bd, badge_col, status_txt) = if active {
                (
                    tint_alpha(accent, 52),
                    tint_alpha(accent, 78),
                    tint_alpha(accent, 200),
                    accent,
                    "ON",
                )
            } else {
                (
                    tile_idle_bg(),
                    tile_idle_hover(),
                    border_subtle(),
                    TEXT_DIM,
                    "OFF",
                )
            };

            let resp = ui
                .btn_with(&id, |ui| {
                    ui.col(|ui| {
                        ui.txt(&el.label)
                            .sz(crate::theme::fs(11.0))
                            .semibold()
                            .align(TextAlign::Center)
                            .fg(TEXT_PRIMARY)
                            .wrap();
                        ui.txt(status_txt)
                            .sz(crate::theme::fs(9.5))
                            .bold()
                            .align(TextAlign::Center)
                            .fg(if active { badge_col } else { TEXT_MUTED });
                    })
                    .center()
                    .gap(2.0)
                    .grow(1.0);
                })
                .grow(grow_weight)
                .min_h(54.0)
                .px(7.0)
                .py(5.0)
                .bg(bg)
                .hover_bg(hover_bg)
                .bd(if active { 1.5 } else { 1.0 }, bd)
                .bevel_rounded(7.0, 1.3)
                .opacity(if el.disabled { 0.45 } else { 1.0 });

            if resp.hovered() && !el.tooltip.is_empty() {
                state.quick.hovered_tooltip = el.tooltip.clone();
            }
            if !el.disabled && resp.clicked() {
                push_click(actions);
                actions.push(UiAction::UiLibToggle {
                    element_id: el.id.clone(),
                    value: !active,
                });
            }
        }
        "cycle" => {
            let resp = ui
                .btn_with(&id, |ui| {
                    ui.col(|ui| {
                        if !el.icon_key.is_empty() {
                            render_icon_or_fallback(
                                ui,
                                state,
                                &el.icon_key,
                                18.0,
                                accent,
                                4.0,
                                ImageFit::Contain,
                            );
                        }
                        ui.txt(&el.label)
                            .sz(crate::theme::fs(11.0))
                            .semibold()
                            .align(TextAlign::Center)
                            .fg(TEXT_PRIMARY)
                            .wrap();
                        ui.txt("Cycle")
                            .sz(crate::theme::fs(9.5))
                            .fg(accent);
                    })
                    .center()
                    .gap(2.0)
                    .grow(1.0);
                })
                .grow(grow_weight)
                .min_h(54.0)
                .px(7.0)
                .py(5.0)
                .bg(tile_idle_bg())
                .hover_bg(tile_idle_hover())
                .bd(1.0, tint_alpha(accent, 85))
                .bevel_rounded(7.0, 1.3)
                .opacity(if el.disabled { 0.45 } else { 1.0 });

            if resp.hovered() && !el.tooltip.is_empty() {
                state.quick.hovered_tooltip = el.tooltip.clone();
            }
            if !el.disabled && resp.clicked() {
                push_click(actions);
                actions.push(UiAction::UiLibClick {
                    element_id: el.id.clone(),
                });
            }
        }
        _ => {
            // Standard UILib Button (TextWithIcon, TextOnly, FullSizeImage, ImageOnly)
            let is_full_img = el.style == "FullSizeImage";
            let is_text_only = el.style == "TextOnly";

            let resp = ui
                .btn_with(&id, |ui| {
                    ui.col(|ui| {
                        if !is_text_only && !el.icon_key.is_empty() {
                            let sz = if is_full_img { 28.0 } else { 18.0 };
                            render_icon_or_fallback(
                                ui,
                                state,
                                &el.icon_key,
                                sz,
                                Color::rgb(203, 213, 225),
                                if is_full_img { 6.0 } else { 4.0 },
                                if is_full_img {
                                    ImageFit::Cover
                                } else {
                                    ImageFit::Contain
                                },
                            );
                        }
                        if !el.label.is_empty() {
                            ui.txt(&el.label)
                                .sz(crate::theme::fs(11.0))
                                .semibold()
                                .align(TextAlign::Center)
                                .fg(TEXT_PRIMARY)
                                .wrap();
                        }
                        if el.has_hold {
                            ui.txt("Right-click: Hold").sz(crate::theme::fs(9.0)).fg(TEXT_DIM);
                        }
                    })
                    .center()
                    .gap(3.0)
                    .grow(1.0);
                })
                .grow(grow_weight)
                .min_h(54.0)
                .px(7.0)
                .py(5.0)
                .bg(tile_idle_bg())
                .hover_bg(tile_idle_hover())
                .bd(1.0, border_subtle())
                .bevel_rounded(7.0, 1.3)
                .opacity(if el.disabled { 0.45 } else { 1.0 });

            if resp.hovered() && !el.tooltip.is_empty() {
                state.quick.hovered_tooltip = el.tooltip.clone();
            }
            if !el.disabled {
                if resp.clicked() {
                    push_click(actions);
                    actions.push(UiAction::UiLibClick {
                        element_id: el.id.clone(),
                    });
                } else if el.has_hold && resp.right_clicked() {
                    actions.push(UiAction::PlayUiSound {
                        sound: String::from("Select"),
                    });
                    actions.push(UiAction::UiLibHold {
                        element_id: el.id.clone(),
                    });
                }
            }
        }
    }
}

fn render_slider_element(
    ui: &mut UiContext,
    state: &mut AppState,
    el: &UiLibElement,
    accent: Color,
    actions: &mut Vec<UiAction>,
) {
    let min_v = el.min_val;
    let max_v = if el.max_val > min_v {
        el.max_val
    } else {
        min_v + 1.0
    };

    let entry = state
        .quick
        .uilib_sliders
        .entry(el.id.clone())
        .or_insert((el.float_val, el.float_val));

    // Sync from C# whenever host changed externally.
    if (el.float_val - entry.1).abs() > 1e-4 {
        entry.0 = el.float_val.clamp(min_v, max_v);
        entry.1 = el.float_val;
    }

    let mut cur_val = entry.0.clamp(min_v, max_v);
    // Keep slider label static so its hashed WidgetId stays stable across drag frames!
    let static_label = if el.label.is_empty() {
        "Value"
    } else {
        &el.label
    };

    ui.push_id(&el.id);
    ui.row(|ui| {
        let resp = ui
            .slider(static_label, &mut cur_val, min_v..=max_v)
            .fill(accent)
            .bg(card_inner_bg())
            .bd(1.0, border_subtle())
            .bevel_rounded(7.8, 1.3)
            .enabled(!el.disabled)
            .opacity(if el.disabled { 0.48 } else { 1.0 })
            .grow(1.0);

        if !el.disabled && resp.changed() {
            let factor = 10f32.powi(el.decimals.clamp(0, 4));
            let rounded = (cur_val * factor).round() / factor;
            let e = state
                .quick
                .uilib_sliders
                .entry(el.id.clone())
                .or_insert((rounded, rounded));
            if (rounded - e.0).abs() > 1e-5 {
                e.0 = rounded;
                e.1 = rounded;
                actions.push(UiAction::UiLibSlider {
                    element_id: el.id.clone(),
                    value: rounded,
                    reset: false,
                });
            }
        }

        if el.allow_reset {
            let reset_id = format!("uilib_reset_{}", el.id);
            let r_btn = ui
                .btn_with(&reset_id, |ui| {
                    ui.txt("Reset").sz(crate::theme::fs(10.5)).semibold().fg(TEXT_MUTED);
                })
                .px(8.0)
                .py(7.0)
                .bg(tile_idle_bg())
                .hover_bg(tile_idle_hover())
                .bd(1.0, border_subtle())
                .bevel_rounded(6.2, 1.3)
                .opacity(if el.disabled { 0.45 } else { 1.0 });
            if !el.disabled && r_btn.clicked() {
                let def = el.default_val.clamp(min_v, max_v);
                state.quick.uilib_sliders.insert(el.id.clone(), (def, def));
                push_click(actions);
                actions.push(UiAction::UiLibSlider {
                    element_id: el.id.clone(),
                    value: def,
                    reset: true,
                });
            }
        }
    })
    .align_center()
    .gap(6.0);
    ui.pop_id();
}

fn render_full_width_element(
    ui: &mut UiContext,
    state: &mut AppState,
    el: &UiLibElement,
    accent: Color,
    actions: &mut Vec<UiAction>,
) {
    match el.kind.as_str() {
        "slider" => render_slider_element(ui, state, el, accent, actions),
        "text_block" => {
            if !el.label.is_empty() {
                ui.txt(&el.label)
                    .sz(crate::theme::fs(11.0))
                    .fg(TEXT_MUTED)
                    .wrap()
                    .px(4.0)
                    .py(2.0);
            }
        }
        "text_input" => {
            let id = format!("uilib_txtin_{}", el.id);
            let display = if el.text_val.is_empty() {
                if el.placeholder.is_empty() {
                    "Click to edit text..."
                } else {
                    &el.placeholder
                }
            } else {
                &el.text_val
            };
            let is_placeholder = el.text_val.is_empty();
            let ti_resp = ui
                .btn_with(&id, |ui| {
                    ui.row(|ui| {
                        ui.txt(display)
                            .sz(crate::theme::fs(11.5))
                            .fg(if is_placeholder { TEXT_DIM } else { TEXT_PRIMARY })
                            .grow(1.0);
                        ui.txt("Edit").sz(crate::theme::fs(10.5)).semibold().fg(accent);
                    })
                    .align_center()
                    .gap(6.0)
                    .grow(1.0);
                })
                .px(10.0)
                .py(7.0)
                .bg(card_inner_bg())
                .hover_bg(tile_idle_hover())
                .bd(1.0, border_subtle())
                .bevel_rounded(6.2, 1.3)
                .opacity(if el.disabled { 0.45 } else { 1.0 });
            if !el.disabled && ti_resp.clicked() {
                push_click(actions);
                actions.push(UiAction::UiLibTextInput {
                    element_id: el.id.clone(),
                });
            }
        }
        "content_button" => {
            let id = format!("uilib_cbtn_{}", el.id);
            let resp = ui
                .btn_with(&id, |ui| {
                    ui.row(|ui| {
                        render_icon_or_fallback(
                            ui,
                            state,
                            &el.icon_key,
                            36.0,
                            accent,
                            7.0,
                            ImageFit::Cover,
                        );
                        ui.col(|ui| {
                            ui.txt(&el.label).sz(crate::theme::fs(12.0)).bold().fg(TEXT_PRIMARY);
                            for line in &el.lines {
                                if !line.is_empty() {
                                    ui.txt(line).sz(crate::theme::fs(10.5)).fg(TEXT_MUTED).wrap();
                                }
                            }
                        })
                        .grow(1.0)
                        .gap(1.0);
                    })
                    .align_center()
                    .gap(8.0)
                    .grow(1.0);
                })
                .p(7.0)
                .bg(tile_idle_bg())
                .hover_bg(tile_idle_hover())
                .bd(1.0, border_subtle())
                .bevel_rounded(7.0, 1.3)
                .opacity(if el.disabled { 0.45 } else { 1.0 });

            if resp.hovered() && !el.tooltip.is_empty() {
                state.quick.hovered_tooltip = el.tooltip.clone();
            }
            if !el.disabled && resp.clicked() {
                push_click(actions);
                actions.push(UiAction::UiLibClick {
                    element_id: el.id.clone(),
                });
            }
        }
        "content_details" => {
            ui.col(|ui| {
                ui.row(|ui| {
                    let img_id = format!("uilib_cdet_img_{}", el.id);
                    if !el.icon_key.is_empty() {
                        let resp = ui
                            .btn_with(&img_id, |ui| {
                                render_icon_or_fallback(
                                    ui,
                                    state,
                                    &el.icon_key,
                                    44.0,
                                    accent,
                                    8.0,
                                    ImageFit::Cover,
                                );
                            })
                            .p(2.0)
                            .bg(Color::TRANSPARENT)
                            .hover_bg(tint_alpha(accent, 45))
                            .bevel_rounded(6.2, 1.3);
                        if !el.disabled && resp.clicked() {
                            push_click(actions);
                            actions.push(UiAction::UiLibClick {
                                element_id: el.id.clone(),
                            });
                        }
                    }

                    ui.col(|ui| {
                        for blk in &el.detail_blocks {
                            ui.row(|ui| {
                                if !blk.header.is_empty() {
                                    ui.txt(&blk.header).sz(crate::theme::fs(11.0)).semibold().fg(TEXT_MUTED);
                                }
                                if !blk.text.is_empty() {
                                    if blk.badge {
                                        ui.txt(&blk.text)
                                            .sz(crate::theme::fs(10.0))
                                            .bold()
                                            .fg(accent)
                                            .bg(tint_alpha(accent, 34))
                                            .px(6.0)
                                            .py(1.5)
                                            .bevel_rounded(3.9, 1.3);
                                    } else {
                                        ui.txt(&blk.text).sz(crate::theme::fs(11.5)).fg(TEXT_PRIMARY).wrap();
                                    }
                                }
                            })
                            .align_center()
                            .gap(5.0);
                        }
                    })
                    .grow(1.0)
                    .gap(3.0);
                })
                .align_center()
                .gap(9.0);

                if !el.extra_buttons.is_empty() {
                    ui.row(|ui| {
                        for eb in &el.extra_buttons {
                            let ebid = format!("uilib_cdet_btn_{}", eb.id);
                            let eb_resp = ui
                                .btn_with(&ebid, |ui| {
                                    ui.txt(&eb.label).sz(crate::theme::fs(11.0)).semibold().fg(TEXT_PRIMARY);
                                })
                                .px(8.0)
                                .py(5.0)
                                .bg(tile_idle_bg())
                                .hover_bg(tint_alpha(accent, 75))
                                .bd(1.0, border_subtle())
                                .bevel_rounded(5.5, 1.3)
                                .opacity(if eb.disabled { 0.45 } else { 1.0 });
                            if !eb.disabled && eb_resp.clicked() {
                                push_click(actions);
                                actions.push(UiAction::UiLibClick {
                                    element_id: eb.id.clone(),
                                });
                            }
                        }
                    })
                    .gap(5.0);
                }
            })
            .p(8.0)
            .gap(6.0)
            .bg(card_inner_bg())
            .bd(1.0, border_subtle())
            .bevel_rounded(7.0, 1.3);
        }
        "spacer" => {
            ui.txt("").h(1.0).bg(border_subtle());
        }
        "custom" => {
            ui.push_id(&el.id);
            ui.row(|ui| {
                ui.txt("Custom HTML element (not supported natively)")
                    .sz(crate::theme::fs(11.0))
                    .fg(TEXT_DIM)
                    .grow(1.0);
            })
            .align_center()
            .p(8.0)
            .bg(Color::rgba(15, 23, 42, 160))
            .bd(1.0, Color::rgba(251, 191, 36, 55))
            .bevel_rounded(6.2, 1.3);
            ui.pop_id();
        }
        _ => {}
    }
}

/// Renders active UILib alert toast banner.
pub fn render_uilib_toast(ui: &mut UiContext, toast: &str, accent: Color) {
    if toast.is_empty() {
        return;
    }
    ui.overlay(|ui| {
        ui.col(|ui| {
            ui.txt(toast)
                .sz(crate::theme::fs(12.5))
                .semibold()
                .align(TextAlign::Center)
                .fg(TEXT_PRIMARY)
                .bg(Color::rgba(15, 19, 28, 245))
                .bd(1.5, accent)
                .px(16.0)
                .py(9.0)
                .bevel_rounded(7.8, 1.3)
                .shadow(16.0, Color::rgba(0, 0, 0, 160));
        })
        .center()
        .w_frac(1.0)
        .pt(72.0);
    })
    .grow(1.0)
    .passthrough();
}

/// Renders active UILib modal overlays (Confirm, Notice, MultiSelect, NumberInput).
pub fn render_uilib_modal(
    ui: &mut UiContext,
    state: &mut AppState,
    modal: &UiLibModalSnapshot,
    accent: Color,
    actions: &mut Vec<UiAction>,
) {
    if modal.kind == "number_input" && state.quick.modal_number_revision != modal.revision {
        state.quick.modal_number_revision = modal.revision;
        state.quick.modal_number_edit = TextEditState::with_text(&format!("{}", modal.number_value));
        state.quick.modal_number_error.clear();
    }

    ui.overlay(|ui| {
        ui.zstack(|ui| {
            if crate::theme::modal_scrim(ui, "uilib_modal_backdrop_fx", "uilib_modal_backdrop") {
                push_click(actions);
                actions.push(UiAction::UiLibModalDismiss);
            }

            ui.col(|ui| {
                let mut card = ui.col(|ui| {
                    // Modal Header
                    ui.row(|ui| {
                        ui.txt(&modal.title)
                            .sz(crate::theme::fs(16.0))
                            .bold()
                            .fg(TEXT_PRIMARY)
                            .grow(1.0);
                        if ui
                            .btn_with("uilib_modal_close", |ui| {
                                icons::icon(ui, icons::CLOSE, 14.0).fg(TEXT_MUTED);
                            })
                            .p(5.0)
                            .bg(Color::TRANSPARENT)
                            .hover_bg(Color::rgba(239, 68, 68, 80))
                            .bevel_rounded(4.7, 1.3)
                            .clicked()
                        {
                            push_click(actions);
                            actions.push(UiAction::UiLibModalDismiss);
                        }
                    })
                    .align_center()
                    .gap(8.0);

                    match modal.kind.as_str() {
                        "confirm" => {
                            if !modal.body.is_empty() {
                                ui.txt(&modal.body).sz(crate::theme::fs(13.0)).fg(TEXT_MUTED).wrap();
                            }
                            ui.row(|ui| {
                                let no_lbl = if modal.no_text.is_empty() {
                                    "No"
                                } else {
                                    &modal.no_text
                                };
                                if ui
                                    .btn_with("uilib_modal_no", |ui| {
                                        ui.txt(no_lbl)
                                            .sz(crate::theme::fs(13.0))
                                            .semibold()
                                            .align(TextAlign::Center)
                                            .fg(TEXT_PRIMARY);
                                    })
                                    .bg(tile_idle_bg())
                                    .hover_bg(tile_idle_hover())
                                    .bd(1.0, border_subtle())
                                    .py(10.0)
                                    .bevel_rounded(7.0, 1.3)
                                    .grow(1.0)
                                    .clicked()
                                {
                                    push_click(actions);
                                    actions.push(UiAction::UiLibModalConfirm { accepted: false });
                                }

                                let yes_lbl = if modal.yes_text.is_empty() {
                                    "Yes"
                                } else {
                                    &modal.yes_text
                                };
                                if ui
                                    .btn_with("uilib_modal_yes", |ui| {
                                        ui.txt(yes_lbl)
                                            .sz(crate::theme::fs(13.0))
                                            .bold()
                                            .align(TextAlign::Center)
                                            .fg(Color::rgb(12, 18, 28));
                                    })
                                    .bg(accent)
                                    .hover_bg(tint_alpha(accent, 225))
                                    .py(10.0)
                                    .bevel_rounded(7.0, 1.3)
                                    .grow(1.0)
                                    .clicked()
                                {
                                    push_click(actions);
                                    actions.push(UiAction::UiLibModalConfirm { accepted: true });
                                }
                            })
                            .gap(10.0);
                        }
                        "notice" => {
                            if !modal.body.is_empty() {
                                ui.txt(&modal.body).sz(crate::theme::fs(13.0)).fg(TEXT_MUTED).wrap();
                            }
                            let ok_lbl = if modal.ok_text.is_empty() {
                                "OK"
                            } else {
                                &modal.ok_text
                            };
                            if ui
                                .btn_with("uilib_modal_ok", |ui| {
                                    ui.txt(ok_lbl)
                                        .sz(crate::theme::fs(13.0))
                                        .bold()
                                        .align(TextAlign::Center)
                                        .fg(Color::rgb(12, 18, 28));
                                })
                                .bg(accent)
                                .py(10.0)
                                .bevel_rounded(7.0, 1.3)
                                .clicked()
                            {
                                push_click(actions);
                                actions.push(UiAction::UiLibModalNoticeOk);
                            }
                        }
                        "multi_select" => {
                            ui.scroll("uilib_multiselect_scroll", |ui| {
                                for (i, opt) in modal.options.iter().enumerate() {
                                    let is_sel = i as i32 == modal.selected_index;
                                    let opt_id = format!("uilib_mopt_{i}");
                                    if ui
                                        .btn_with(&opt_id, |ui| {
                                            ui.row(|ui| {
                                                ui.txt(opt)
                                                    .sz(crate::theme::fs(13.0))
                                                    .semibold()
                                                    .fg(if is_sel { accent } else { TEXT_PRIMARY })
                                                    .grow(1.0);
                                                if is_sel {
                                                    icons::icon(ui, icons::CHECK, 15.0).fg(accent);
                                                }
                                            })
                                            .align_center()
                                            .gap(8.0)
                                            .grow(1.0);
                                        })
                                        .px(12.0)
                                        .py(10.0)
                                        .bg(if is_sel {
                                            tint_alpha(accent, 45)
                                        } else {
                                            tile_idle_bg()
                                        })
                                        .hover_bg(tint_alpha(accent, 75))
                                        .bd(1.0, if is_sel { accent } else { border_subtle() })
                                        .bevel_rounded(7.0, 1.3)
                                        .clicked()
                                    {
                                        push_click(actions);
                                        actions.push(UiAction::UiLibModalSelectOption {
                                            index: i as i32,
                                        });
                                    }
                                }
                            })
                            .max_h(340.0)
                            .gap(6.0);
                        }
                        "number_input" => {
                            let inp = ui
                                .input("uilib_num_input", &mut state.quick.modal_number_edit)
                                .placeholder("Enter number")
                                .bg(card_inner_bg())
                                .fg(TEXT_PRIMARY);
                            let mut try_submit = inp.submitted();

                            for row in [
                                ["7", "8", "9", "-"],
                                ["4", "5", "6", "."],
                                ["1", "2", "3", "Del"],
                                ["0", "Clear", "Cancel", "OK"],
                            ] {
                                ui.row(|ui| {
                                    for key in row {
                                        let is_ok = key == "OK";
                                        let is_cancel = key == "Cancel";
                                        let btn_id = format!("uilib_numkey_{key}");
                                        if ui
                                            .btn_with(&btn_id, |ui| {
                                                ui.txt(key)
                                                    .sz(crate::theme::fs(13.0))
                                                    .bold()
                                                    .align(TextAlign::Center)
                                                    .fg(if is_ok {
                                                        Color::rgb(12, 18, 28)
                                                    } else {
                                                        TEXT_PRIMARY
                                                    });
                                            })
                                            .grow(1.0)
                                            .py(9.0)
                                            .bg(if is_ok {
                                                accent
                                            } else if is_cancel {
                                                Color::rgba(185, 28, 28, 195)
                                            } else {
                                                tile_idle_bg()
                                            })
                                            .hover_bg(if is_ok {
                                                tint_alpha(accent, 225)
                                            } else {
                                                tile_idle_hover()
                                            })
                                            .bd(1.0, border_subtle())
                                            .bevel_rounded(6.2, 1.3)
                                            .clicked()
                                        {
                                            push_click(actions);
                                            match key {
                                                "OK" => try_submit = true,
                                                "Cancel" => {
                                                    actions.push(UiAction::UiLibModalDismiss)
                                                }
                                                "Clear" => {
                                                    state.quick.modal_number_edit =
                                                        TextEditState::new();
                                                    state.quick.modal_number_error.clear();
                                                }
                                                "Del" => {
                                                    state.quick.modal_number_edit.backspace();
                                                }
                                                "-" => {
                                                    let cur = state.quick.modal_number_edit.text();
                                                    if let Some(rest) = cur.strip_prefix('-') {
                                                        state.quick.modal_number_edit.set_text(rest);
                                                    } else {
                                                        state.quick.modal_number_edit.set_text(&format!("-{cur}"));
                                                    }
                                                }
                                                "." => {
                                                    let cur = state.quick.modal_number_edit.text();
                                                    if !cur.contains('.') {
                                                        state.quick.modal_number_edit.insert_str(".");
                                                    }
                                                }
                                                digit => {
                                                    state.quick.modal_number_edit.insert_str(digit);
                                                }
                                            }
                                        }
                                    }
                                })
                                .gap(6.0);
                            }

                            if !state.quick.modal_number_error.is_empty() {
                                ui.txt(&state.quick.modal_number_error).sz(crate::theme::fs(11.5)).fg(RED);
                            }

                            if try_submit {
                                let raw = state.quick.modal_number_edit.text();
                                match raw.trim().parse::<f32>() {
                                    Ok(v) if v.is_finite() => {
                                        state.quick.modal_number_error.clear();
                                        actions.push(UiAction::UiLibModalNumberSubmit { value: v });
                                    }
                                    _ => {
                                        state.quick.modal_number_error = String::from(
                                            "Enter a valid finite number.",
                                        );
                                    }
                                }
                            }
                        }
                        _ => {}
                    }
                })
                .w(440.0)
                .p(18.0)
                .gap(12.0)
                .bg(Color::rgba(15, 19, 28, 248))
                .bd(1.5, accent)
                .bevel_rounded(12.5, 2.0)
                .shadow(24.0, Color::rgba(0, 0, 0, 190));
                // Make modal card padding hit-testable so clicks inside the card never hit backdrop.
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
