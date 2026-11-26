use lumenic::{Color, TextAlign, TextEditState, UiContext};

use crate::icons;
use crate::state::{AppState, UiAction, UixCategorySnapshot, UixEntrySnapshot};
use crate::theme::{
    border_subtle, card_bg, card_inner_bg, push_click, tile_idle_bg, tile_idle_hover, tint_alpha,
    EMERALD, SHADOW_CARD, TEXT_DIM, TEXT_MUTED, TEXT_PRIMARY,
};

fn format_number(v: f64, is_int: bool) -> String {
    if is_int {
        format!("{}", v.round() as i64)
    } else {
        let s = format!("{v:.3}");
        let trimmed = s.trim_end_matches('0').trim_end_matches('.');
        if trimmed.is_empty() || trimmed == "-0" {
            String::from("0")
        } else {
            String::from(trimmed)
        }
    }
}

fn parse_hex_color(hex: &str) -> Option<Color> {
    let s = hex.trim().strip_prefix('#').unwrap_or(hex.trim());
    if s.len() != 6 {
        return None;
    }
    let r = u8::from_str_radix(&s[0..2], 16).ok()?;
    let g = u8::from_str_radix(&s[2..4], 16).ok()?;
    let b = u8::from_str_radix(&s[4..6], 16).ok()?;
    Some(Color::rgb(r, g, b))
}

fn render_color_swatch_bar(ui: &mut UiContext, colors: &[String], height: f32) {
    let c0 = colors.first().and_then(|s| parse_hex_color(s));
    let c1 = colors.get(1).and_then(|s| parse_hex_color(s));
    let Some(first) = c0 else {
        return;
    };
    ui.col(|ui| {
        if let Some(second) = c1 {
            ui.col(|_| {}).w(4.0).h(height * 0.5).bg(first);
            ui.col(|_| {}).w(4.0).h(height * 0.5).bg(second);
        } else {
            ui.col(|_| {}).w(4.0).h(height).bg(first);
        }
    })
    .w(4.0)
    .h(height)
    .bevel_rounded(2.0, 0.5);
}

fn entry_matches(entry: &UixEntrySnapshot, filter: &str) -> bool {
    if filter.is_empty() {
        return true;
    }
    entry.display_name.to_ascii_lowercase().contains(filter)
        || entry.description.to_ascii_lowercase().contains(filter)
        || entry.section_name.to_ascii_lowercase().contains(filter)
        || entry.id.to_ascii_lowercase().contains(filter)
}

fn category_header_matches(cat: &UixCategorySnapshot, filter: &str) -> bool {
    if filter.is_empty() {
        return true;
    }
    cat.display_name.to_ascii_lowercase().contains(filter)
        || cat.id.to_ascii_lowercase().contains(filter)
        || cat.info.to_ascii_lowercase().contains(filter)
}

fn render_entry_reset_btn(
    ui: &mut UiContext,
    btn_id: &str,
    entry: &UixEntrySnapshot,
    accent: Color,
    actions: &mut Vec<UiAction>,
) {
    if !entry.can_reset || entry.is_default || entry.node_id < 0 {
        return;
    }
    if ui
        .btn_with(btn_id, |ui| {
            icons::icon(ui, icons::RESPAWN, 13.0).fg(accent);
        })
        .px(7.0)
        .py(6.0)
        .bg(tile_idle_bg())
        .hover_bg(tile_idle_hover())
        .bd(1.0, border_subtle())
        .bevel_rounded(5.5, 1.2)
        .clicked()
    {
        push_click(actions);
        actions.push(UiAction::MowodsResetNode {
            node_id: entry.node_id,
        });
    }
}

#[allow(clippy::too_many_arguments)]
fn render_entry_row(
    ui: &mut UiContext,
    state: &mut AppState,
    cat: &UixCategorySnapshot,
    entry: &UixEntrySnapshot,
    accent: Color,
    actions: &mut Vec<UiAction>,
) {
    let full_key = format!("{}::{}", cat.id, entry.id);
    let reset_id = format!("mm_uix_rst_{full_key}");
    match entry.kind.as_str() {
        "bool" => {
            ui.row(|ui| {
                ui.col(|ui| {
                    ui.txt(&entry.display_name)
                        .sz(crate::theme::fs(12.0))
                        .semibold()
                        .fg(TEXT_PRIMARY);
                    if !entry.description.is_empty() {
                        ui.txt(&entry.description)
                            .sz(crate::theme::fs(10.5))
                            .fg(TEXT_MUTED)
                            .wrap();
                    }
                })
                .gap(2.0)
                .grow(1.0);

                render_entry_reset_btn(ui, &reset_id, entry, accent, actions);

                let active = entry.bool_val;
                let btn_id = format!("mm_uix_b_{full_key}");
                if ui
                    .btn_with(&btn_id, |ui| {
                        ui.txt(if active { "ON" } else { "OFF" })
                            .sz(crate::theme::fs(11.0))
                            .bold()
                            .align(TextAlign::Center)
                            .fg(if active { EMERALD } else { TEXT_MUTED })
                            .grow(1.0);
                    })
                    .w(66.0)
                    .py(6.0)
                    .bg(if active {
                        tint_alpha(EMERALD, 45)
                    } else {
                        tile_idle_bg()
                    })
                    .hover_bg(tile_idle_hover())
                    .bd(
                        1.0,
                        if active {
                            tint_alpha(EMERALD, 190)
                        } else {
                            border_subtle()
                        },
                    )
                    .bevel_rounded(6.0, 1.3)
                    .clicked()
                {
                    push_click(actions);
                    actions.push(UiAction::UixSetBool {
                        category: cat.id.clone(),
                        entry: entry.id.clone(),
                        value: !active,
                    });
                }
            })
            .align_center()
            .p(10.0)
            .gap(8.0)
            .bg(card_inner_bg())
            .bd(1.0, border_subtle())
            .bevel_rounded(7.0, 1.3);
        }

        "slider" => {
            let (cur_val, last_host) = state
                .main_menu
                .uix_sliders
                .entry(full_key.clone())
                .or_insert((entry.num_val as f32, entry.num_val as f32));
            if (*last_host - entry.num_val as f32).abs() > 0.0001 {
                *cur_val = entry.num_val as f32;
                *last_host = entry.num_val as f32;
            }
            let mut slider_v = *cur_val;
            let min_f = entry.min_val as f32;
            let max_f = entry.max_val.max(entry.min_val + 0.001) as f32;

            let val_txt = format_number(slider_v as f64, entry.is_int);
            let show_reset = entry.can_reset && !entry.is_default && entry.node_id >= 0;
            let mut changed = false;
            let mut hovered = false;

            if show_reset {
                ui.row(|ui| {
                    let res = crate::widgets::styled_slider(
                        ui,
                        &full_key,
                        &entry.display_name,
                        &val_txt,
                        &mut slider_v,
                        min_f..=max_f,
                        accent,
                        true,
                        false,
                    );
                    changed = res.changed;
                    hovered = res.hovered;
                    render_entry_reset_btn(ui, &reset_id, entry, accent, actions);
                })
                .align_center()
                .gap(6.0);
            } else {
                let res = crate::widgets::styled_slider(
                    ui,
                    &full_key,
                    &entry.display_name,
                    &val_txt,
                    &mut slider_v,
                    min_f..=max_f,
                    accent,
                    true,
                    false,
                );
                changed = res.changed;
                hovered = res.hovered;
            }

            if changed {
                let mut snapped = slider_v as f64;
                if entry.snap_val > 0.0 {
                    snapped = (snapped / entry.snap_val).round() * entry.snap_val;
                }
                if entry.is_int {
                    snapped = snapped.round();
                }
                snapped = snapped.clamp(entry.min_val, entry.max_val);
                *cur_val = snapped as f32;
                actions.push(UiAction::UixSetNumber {
                    category: cat.id.clone(),
                    entry: entry.id.clone(),
                    value: snapped,
                    save: false,
                });
            }
            if !entry.description.is_empty() && hovered {
                state.main_menu.hovered_tooltip = entry.description.clone();
            }
        }

        "number" => {
            let host_str = format_number(entry.num_val, entry.is_int);
            let (buf, last_synced) = state
                .main_menu
                .uix_text_inputs
                .entry(full_key.clone())
                .or_insert_with(|| {
                    let mut te = TextEditState::new();
                    te.set_text(&host_str);
                    (te, host_str.clone())
                });
            if *last_synced != host_str {
                buf.set_text(&host_str);
                *last_synced = host_str.clone();
            }

            ui.row(|ui| {
                ui.col(|ui| {
                    ui.txt(&entry.display_name)
                        .sz(crate::theme::fs(12.0))
                        .semibold()
                        .fg(TEXT_PRIMARY);
                    if !entry.description.is_empty() {
                        ui.txt(&entry.description)
                            .sz(crate::theme::fs(10.5))
                            .fg(TEXT_MUTED)
                            .wrap();
                    }
                })
                .gap(2.0)
                .grow(1.0);

                render_entry_reset_btn(ui, &reset_id, entry, accent, actions);

                let inp_id = format!("mm_uix_n_{full_key}");
                let submitted = ui
                    .input(&inp_id, buf)
                    .w(116.0)
                    .bg(tile_idle_bg())
                    .fg(TEXT_PRIMARY)
                    .submitted();

                let apply_id = format!("mm_uix_na_{full_key}");
                let clicked = ui
                    .btn_with(&apply_id, |ui| {
                        ui.txt("Apply")
                            .sz(crate::theme::fs(10.5))
                            .semibold()
                            .fg(TEXT_PRIMARY);
                    })
                    .px(10.0)
                    .py(6.0)
                    .bg(tint_alpha(accent, 48))
                    .hover_bg(tint_alpha(accent, 80))
                    .bd(1.0, tint_alpha(accent, 175))
                    .bevel_rounded(5.5, 1.3)
                    .clicked();

                if submitted || clicked {
                    if let Ok(parsed) = buf.text().trim().parse::<f64>() {
                        push_click(actions);
                        actions.push(UiAction::UixSetNumber {
                            category: cat.id.clone(),
                            entry: entry.id.clone(),
                            value: parsed,
                            save: true,
                        });
                    }
                }
            })
            .align_center()
            .p(10.0)
            .gap(8.0)
            .bg(card_inner_bg())
            .bd(1.0, border_subtle())
            .bevel_rounded(7.0, 1.3);
        }

        "string" => {
            let host_str = entry.string_val.clone();
            let (buf, last_synced) = state
                .main_menu
                .uix_text_inputs
                .entry(full_key.clone())
                .or_insert_with(|| {
                    let mut te = TextEditState::new();
                    te.set_text(&host_str);
                    (te, host_str.clone())
                });
            if *last_synced != host_str {
                buf.set_text(&host_str);
                *last_synced = host_str.clone();
            }

            ui.col(|ui| {
                ui.row(|ui| {
                    ui.col(|ui| {
                        ui.txt(&entry.display_name)
                            .sz(crate::theme::fs(12.0))
                            .semibold()
                            .fg(TEXT_PRIMARY);
                        if !entry.description.is_empty() {
                            ui.txt(&entry.description)
                                .sz(crate::theme::fs(10.5))
                                .fg(TEXT_MUTED)
                                .wrap();
                        }
                    })
                    .gap(2.0)
                    .grow(1.0);
                    render_entry_reset_btn(ui, &reset_id, entry, accent, actions);
                })
                .align_center()
                .gap(8.0);

                ui.row(|ui| {
                    let inp_id = format!("mm_uix_t_{full_key}");
                    let submitted = ui
                        .input(&inp_id, buf)
                        .bg(tile_idle_bg())
                        .fg(TEXT_PRIMARY)
                        .grow(1.0)
                        .submitted();

                    let apply_id = format!("mm_uix_ta_{full_key}");
                    let clicked = ui
                        .btn_with(&apply_id, |ui| {
                            ui.txt("Save")
                                .sz(crate::theme::fs(10.5))
                                .semibold()
                                .fg(TEXT_PRIMARY);
                        })
                        .px(11.0)
                        .py(6.0)
                        .bg(tint_alpha(accent, 48))
                        .hover_bg(tint_alpha(accent, 80))
                        .bd(1.0, tint_alpha(accent, 175))
                        .bevel_rounded(5.5, 1.3)
                        .clicked();

                    if submitted || clicked {
                        push_click(actions);
                        actions.push(UiAction::UixSetString {
                            category: cat.id.clone(),
                            entry: entry.id.clone(),
                            value: buf.text(),
                        });
                    }
                })
                .align_center()
                .gap(7.0);
            })
            .p(10.0)
            .gap(5.0)
            .bg(card_inner_bg())
            .bd(1.0, border_subtle())
            .bevel_rounded(7.0, 1.3);
        }

        "enum" => {
            ui.col(|ui| {
                ui.row(|ui| {
                    ui.col(|ui| {
                        ui.txt(&entry.display_name)
                            .sz(crate::theme::fs(12.0))
                            .semibold()
                            .fg(TEXT_PRIMARY);
                        if !entry.description.is_empty() {
                            ui.txt(&entry.description)
                                .sz(crate::theme::fs(10.5))
                                .fg(TEXT_MUTED)
                                .wrap();
                        }
                    })
                    .gap(2.0)
                    .grow(1.0);
                    render_entry_reset_btn(ui, &reset_id, entry, accent, actions);
                })
                .align_center()
                .gap(8.0);

                if entry.use_buttons && entry.enum_options.len() <= 4 {
                    ui.row(|ui| {
                        for (idx, opt) in entry.enum_options.iter().enumerate() {
                            let active = idx as i32 == entry.enum_index;
                            let opt_id = format!("mm_uix_e_{full_key}_{idx}");
                            if ui
                                .btn_with(&opt_id, |ui| {
                                    ui.txt(opt)
                                        .sz(crate::theme::fs(11.0))
                                        .semibold()
                                        .align(TextAlign::Center)
                                        .fg(if active { TEXT_PRIMARY } else { TEXT_MUTED })
                                        .grow(1.0);
                                })
                                .grow(1.0)
                                .px(8.0)
                                .py(5.5)
                                .bg(if active {
                                    tint_alpha(accent, 52)
                                } else {
                                    tile_idle_bg()
                                })
                                .hover_bg(tile_idle_hover())
                                .bd(
                                    1.0,
                                    if active {
                                        tint_alpha(accent, 195)
                                    } else {
                                        border_subtle()
                                    },
                                )
                                .bevel_rounded(5.5, 1.3)
                                .clicked()
                            {
                                push_click(actions);
                                actions.push(UiAction::UixSetEnum {
                                    category: cat.id.clone(),
                                    entry: entry.id.clone(),
                                    index: idx as i32,
                                });
                            }
                        }
                    })
                    .gap(5.0);
                } else if !entry.enum_options.is_empty() {
                    let len = entry.enum_options.len() as i32;
                    let cur_idx = entry.enum_index.clamp(0, len - 1);
                    let cur_lbl = &entry.enum_options[cur_idx as usize];
                    let prev_id = format!("mm_uix_ep_{full_key}");
                    let next_id = format!("mm_uix_en_{full_key}");
                    ui.row(|ui| {
                        if ui
                            .btn_with(&prev_id, |ui| {
                                icons::icon(ui, icons::BACK, 13.0).fg(TEXT_PRIMARY);
                            })
                            .px(10.0)
                            .py(5.5)
                            .bg(tile_idle_bg())
                            .hover_bg(tile_idle_hover())
                            .bd(1.0, border_subtle())
                            .bevel_rounded(5.5, 1.3)
                            .clicked()
                        {
                            push_click(actions);
                            let next = (cur_idx - 1).rem_euclid(len);
                            actions.push(UiAction::UixSetEnum {
                                category: cat.id.clone(),
                                entry: entry.id.clone(),
                                index: next,
                            });
                        }

                        let idx_badge = format!("{} ({}/{})", cur_lbl, cur_idx + 1, len);
                        let dd_id = format!("mm_uix_edd_{full_key}");
                        if ui
                            .btn_with(&dd_id, |ui| {
                                ui.row(|ui| {
                                    ui.txt(&idx_badge)
                                        .sz(crate::theme::fs(11.5))
                                        .semibold()
                                        .align(TextAlign::Center)
                                        .fg(TEXT_PRIMARY)
                                        .ellipsis()
                                        .grow(1.0);
                                    icons::icon(ui, icons::CHEVRON_DOWN, 14.0).fg(accent);
                                })
                                .align_center()
                                .gap(6.0)
                                .grow(1.0);
                            })
                            .grow(1.0)
                            .px(10.0)
                            .py(5.5)
                            .bg(tile_idle_bg())
                            .hover_bg(tile_idle_hover())
                            .bd(1.0, tint_alpha(accent, 140))
                            .bevel_rounded(5.5, 1.3)
                            .clicked()
                        {
                            push_click(actions);
                            let options = entry
                                .enum_options
                                .iter()
                                .enumerate()
                                .map(|(idx, opt)| crate::state::DropdownOption {
                                    label: opt.clone(),
                                    action: UiAction::UixSetEnum {
                                        category: cat.id.clone(),
                                        entry: entry.id.clone(),
                                        index: idx as i32,
                                    },
                                    selected: idx as i32 == cur_idx,
                                })
                                .collect();
                            state.main_menu.active_dropdown = Some(crate::state::ActiveDropdown {
                                title: entry.display_name.clone(),
                                options,
                            });
                        }

                        if ui
                            .btn_with(&next_id, |ui| {
                                icons::icon(ui, icons::CHEVRON_RIGHT, 13.0).fg(TEXT_PRIMARY);
                            })
                            .px(10.0)
                            .py(5.5)
                            .bg(tile_idle_bg())
                            .hover_bg(tile_idle_hover())
                            .bd(1.0, border_subtle())
                            .bevel_rounded(5.5, 1.3)
                            .clicked()
                        {
                            push_click(actions);
                            let next = (cur_idx + 1).rem_euclid(len);
                            actions.push(UiAction::UixSetEnum {
                                category: cat.id.clone(),
                                entry: entry.id.clone(),
                                index: next,
                            });
                        }
                    })
                    .align_center()
                    .gap(6.0);
                }
            })
            .p(10.0)
            .gap(6.0)
            .bg(card_inner_bg())
            .bd(1.0, border_subtle())
            .bevel_rounded(7.0, 1.3);
        }

        "button" => {
            ui.row(|ui| {
                ui.col(|ui| {
                    ui.txt(&entry.display_name)
                        .sz(crate::theme::fs(12.0))
                        .semibold()
                        .fg(TEXT_PRIMARY);
                    if !entry.description.is_empty() {
                        ui.txt(&entry.description)
                            .sz(crate::theme::fs(10.5))
                            .fg(TEXT_MUTED)
                            .wrap();
                    }
                })
                .gap(2.0)
                .grow(1.0);

                let lbl = if entry.button_label.is_empty() {
                    entry.display_name.as_str()
                } else {
                    entry.button_label.as_str()
                };
                let btn_id = format!("mm_uix_act_{full_key}");
                if ui
                    .btn_with(&btn_id, |ui| {
                        ui.txt(lbl)
                            .sz(crate::theme::fs(11.0))
                            .semibold()
                            .fg(TEXT_PRIMARY);
                    })
                    .px(12.0)
                    .py(6.5)
                    .bg(tint_alpha(accent, 42))
                    .hover_bg(tint_alpha(accent, 78))
                    .bd(1.0, tint_alpha(accent, 175))
                    .bevel_rounded(6.0, 1.3)
                    .clicked()
                {
                    push_click(actions);
                    actions.push(UiAction::UixCustomButton {
                        category: cat.id.clone(),
                        global_id: entry.node_id,
                    });
                }
            })
            .align_center()
            .p(10.0)
            .gap(10.0)
            .bg(card_inner_bg())
            .bd(1.0, border_subtle())
            .bevel_rounded(7.0, 1.3);
        }

        _ => {}
    }
}

/// Renders the native Mod Settings screen (Mowods or UIExpansionKit) inside the Main Menu.
pub fn render_uix_mod_settings(
    ui: &mut UiContext,
    state: &mut AppState,
    accent: Color,
    actions: &mut Vec<UiAction>,
) {
    let filter = state
        .main_menu
        .uix_search_input
        .text()
        .trim()
        .to_ascii_lowercase();

    // Ensure a valid category is selected when categories exist.
    if !state.host.uix_categories.is_empty() {
        let sel_valid = state
            .host
            .uix_categories
            .iter()
            .any(|c| c.id == state.main_menu.uix_selected_category);
        if !sel_valid {
            state.main_menu.uix_selected_category = state.host.uix_categories[0].id.clone();
        }
    }

    let source_title = match state.host.mod_settings_source.as_str() {
        "mowods" => "Mod Settings (Mowods)",
        "uix" => "Mod Settings (UIExpansionKit)",
        _ => "Mod Settings",
    };

    ui.col(|ui| {
        // =====================================================================
        // HEADER BAR
        // =====================================================================
        ui.row(|ui| {
            if ui
                .btn_with("mm_uix_back", |ui| {
                    ui.row(|ui| {
                        icons::icon(ui, icons::BACK, 14.0).fg(TEXT_PRIMARY);
                        ui.txt(crate::main_menu::back_button_label(state, "Home"))
                            .sz(crate::theme::fs(11.5))
                            .semibold()
                            .fg(TEXT_PRIMARY);
                    })
                    .center()
                    .gap(5.0);
                })
                .px(11.0)
                .py(6.0)
                .bg(tile_idle_bg())
                .hover_bg(tile_idle_hover())
                .bd(1.0, border_subtle())
                .bevel_rounded(6.0, 1.3)
                .clicked()
            {
                push_click(actions);
                crate::main_menu::nav_go_back(state, "home", actions);
            }

            icons::icon(ui, icons::PUZZLE, 16.0).fg(accent);
            ui.txt(source_title)
                .sz(crate::theme::fs(14.5))
                .bold()
                .fg(TEXT_PRIMARY)
                .grow(1.0);

            if state.host.mod_settings_both_available {
                ui.row(|ui| {
                    for (src_id, lbl) in [("mowods", "Mowods"), ("uix", "UIX")] {
                        let active = state.host.mod_settings_source == src_id;
                        let btn_id = format!("mm_uix_src_{src_id}");
                        if ui
                            .btn_with(&btn_id, |ui| {
                                ui.txt(lbl)
                                    .sz(crate::theme::fs(10.5))
                                    .semibold()
                                    .fg(if active { TEXT_PRIMARY } else { TEXT_MUTED });
                            })
                            .px(9.0)
                            .py(4.5)
                            .bg(if active {
                                tint_alpha(accent, 52)
                            } else {
                                tile_idle_bg()
                            })
                            .hover_bg(tile_idle_hover())
                            .bd(
                                1.0,
                                if active {
                                    tint_alpha(accent, 195)
                                } else {
                                    border_subtle()
                                },
                            )
                            .bevel_rounded(5.0, 1.2)
                            .clicked()
                            && !active
                        {
                            push_click(actions);
                            state.main_menu.uix_selected_category.clear();
                            actions.push(UiAction::SetModSettingsSource {
                                source: String::from(src_id),
                            });
                        }
                    }
                })
                .align_center()
                .gap(4.0);
            }

            let count_txt = format!("{} Mods", state.host.uix_categories.len());
            ui.txt(&count_txt)
                .sz(crate::theme::fs(11.0))
                .fg(accent)
                .bg(tint_alpha(accent, 30))
                .px(8.0)
                .py(3.5)
                .bevel_rounded(5.0, 1.3);

            if ui
                .btn_with("mm_uix_refresh", |ui| {
                    ui.row(|ui| {
                        icons::icon(ui, icons::RELOAD, 13.0).fg(TEXT_PRIMARY);
                        ui.txt("Refresh")
                            .sz(crate::theme::fs(11.0))
                            .semibold()
                            .fg(TEXT_PRIMARY);
                    })
                    .center()
                    .gap(5.0);
                })
                .px(10.0)
                .py(6.0)
                .bg(tile_idle_bg())
                .hover_bg(tile_idle_hover())
                .bd(1.0, border_subtle())
                .bevel_rounded(6.0, 1.3)
                .clicked()
            {
                push_click(actions);
                actions.push(UiAction::RefreshUixSettings);
            }
        })
        .align_center()
        .px(12.0)
        .py(8.0)
        .gap(9.0)
        .bg(card_bg())
        .bd(1.0, border_subtle())
        .bevel_rounded(10.0, 2.0)
        .shadow(14.0, SHADOW_CARD);

        // =====================================================================
        // BODY: LEFT CATEGORY LIST | RIGHT SETTINGS ENTRIES
        // =====================================================================
        if !state.host.uix_available && state.host.uix_categories.is_empty() {
            ui.col(|ui| {
                icons::icon(ui, icons::PUZZLE, 34.0).fg(TEXT_DIM);
                ui.txt("No Mod Settings Provider Detected")
                    .sz(crate::theme::fs(16.0))
                    .bold()
                    .fg(TEXT_PRIMARY);
                ui.txt("Install Mowods.dll or UI Expansion Kit.dll in ChilloutVR/Mods to configure mod preferences here.")
                    .sz(crate::theme::fs(12.0))
                    .fg(TEXT_MUTED);
            })
            .center()
            .grow(1.0)
            .gap(8.0)
            .bg(card_bg())
            .bd(1.0, border_subtle())
            .bevel_rounded(10.5, 2.0)
            .shadow(14.0, SHADOW_CARD);
        } else {
            ui.row(|ui| {
                // --- LEFT SIDEBAR: SEARCH + CATEGORY LIST ---
                ui.col(|ui| {
                    ui.row(|ui| {
                        icons::icon(ui, icons::SEARCH, 13.0).fg(accent);
                        let mut u_inp = ui
                            .input("mm_uix_cat_search", &mut state.main_menu.uix_search_input)
                            .placeholder("Search mods & settings...")
                            .bg(card_inner_bg())
                            .fg(TEXT_PRIMARY)
                            .grow(1.0);
                        if state.host.is_vr && u_inp.clicked() {
                            actions.push(UiAction::OpenVirtualKeyboard {
                                surface_id: 0,
                                field_id: String::from("mm_uix_cat_search"),
                                initial_text: state.main_menu.uix_search_input.text(),
                                title: String::from("Search Mods & Settings"),
                                placeholder: String::from("Search mods & settings..."),
                                max_len: 96,
                                hidden: false,
                            });
                        }
                    })
                    .align_center()
                    .gap(6.0);

                    let categories: Vec<(String, String, usize, Vec<String>)> = state
                        .host
                        .uix_categories
                        .iter()
                        .filter_map(|c| {
                            if filter.is_empty() {
                                return Some((
                                    c.id.clone(),
                                    c.display_name.clone(),
                                    c.entries.len() + c.custom_controls.len(),
                                    c.colors.clone(),
                                ));
                            }
                            if category_header_matches(c, &filter) {
                                return Some((
                                    c.id.clone(),
                                    c.display_name.clone(),
                                    c.entries.len() + c.custom_controls.len(),
                                    c.colors.clone(),
                                ));
                            }
                            let matching_entries = c
                                .entries
                                .iter()
                                .filter(|e| entry_matches(e, &filter))
                                .count();
                            let matching_custom = c
                                .custom_controls
                                .iter()
                                .filter(|ctrl| ctrl.text.to_ascii_lowercase().contains(&filter))
                                .count();
                            let total = matching_entries + matching_custom;
                            if total > 0 {
                                Some((c.id.clone(), c.display_name.clone(), total, c.colors.clone()))
                            } else {
                                None
                            }
                        })
                        .collect();

                    // Auto-select first matching category when current selection is filtered out.
                    if !categories.is_empty()
                        && !categories
                            .iter()
                            .any(|(id, _, _, _)| *id == state.main_menu.uix_selected_category)
                    {
                        state.main_menu.uix_selected_category = categories[0].0.clone();
                    }

                    ui.scroll("mm_uix_cat_scroll", |ui| {
                        if categories.is_empty() {
                            ui.txt("No matching mods or settings")
                                .sz(crate::theme::fs(11.5))
                                .fg(TEXT_DIM)
                                .p(10.0);
                        } else {
                            for (cat_id, display, count, colors) in &categories {
                                let active = state.main_menu.uix_selected_category == *cat_id;
                                let btn_id = format!("mm_uix_cat_{cat_id}");
                                let cat_accent = colors
                                    .first()
                                    .and_then(|s| parse_hex_color(s))
                                    .unwrap_or(accent);
                                if ui
                                    .btn_with(&btn_id, |ui| {
                                        ui.row(|ui| {
                                            render_color_swatch_bar(ui, colors, 20.0);
                                            ui.txt(display)
                                                .sz(crate::theme::fs(11.5))
                                                .semibold()
                                                .fg(if active { TEXT_PRIMARY } else { TEXT_MUTED })
                                                .ellipsis()
                                                .grow(1.0);
                                            let cnt = format!("{count}");
                                            ui.txt(&cnt)
                                                .sz(crate::theme::fs(10.0))
                                                .fg(if active { cat_accent } else { TEXT_DIM });
                                        })
                                        .align_center()
                                        .gap(7.0)
                                        .grow(1.0);
                                    })
                                    .px(8.0)
                                    .py(6.5)
                                    .bg(if active {
                                        tint_alpha(cat_accent, 42)
                                    } else {
                                        card_inner_bg()
                                    })
                                    .hover_bg(tile_idle_hover())
                                    .bd(
                                        1.0,
                                        if active {
                                            tint_alpha(cat_accent, 195)
                                        } else {
                                            border_subtle()
                                        },
                                    )
                                    .bevel_rounded(6.0, 1.3)
                                    .clicked()
                                {
                                    push_click(actions);
                                    state.main_menu.uix_selected_category = cat_id.clone();
                                }
                            }
                        }
                    })
                    .grow(1.0)
                    .gap(5.0)
                    .no_scrollbar();
                })
                .w(276.0)
                .h_frac(1.0)
                .p(10.0)
                .gap(8.0)
                .bg(card_bg())
                .bd(1.0, border_subtle())
                .bevel_rounded(10.5, 2.0)
                .shadow(14.0, SHADOW_CARD);

                // --- RIGHT PANEL: SELECTED CATEGORY ENTRIES + CUSTOM CONTROLS ---
                let selected_cat = state
                    .host
                    .uix_categories
                    .iter()
                    .find(|c| c.id == state.main_menu.uix_selected_category)
                    .cloned();

                ui.col(|ui| {
                    if let Some(cat) = selected_cat {
                        let cat_matches_all = category_header_matches(&cat, &filter);
                        let visible_entries: Vec<&UixEntrySnapshot> = cat
                            .entries
                            .iter()
                            .filter(|e| cat_matches_all || entry_matches(e, &filter))
                            .collect();
                        let any_non_default = cat
                            .entries
                            .iter()
                            .any(|e| e.can_reset && !e.is_default && e.node_id >= 0);

                        // Mod Header (color swatch + name + info + source link + reset all)
                        ui.row(|ui| {
                            render_color_swatch_bar(ui, &cat.colors, 28.0);
                            ui.col(|ui| {
                                ui.txt(&cat.display_name)
                                    .sz(crate::theme::fs(15.0))
                                    .bold()
                                    .fg(TEXT_PRIMARY)
                                    .ellipsis();
                                if !cat.info.is_empty() {
                                    ui.txt(&cat.info)
                                        .sz(crate::theme::fs(10.5))
                                        .fg(TEXT_MUTED)
                                        .ellipsis();
                                } else if cat.display_name != cat.id {
                                    ui.txt(&cat.id)
                                        .sz(crate::theme::fs(10.5))
                                        .fg(TEXT_DIM)
                                        .ellipsis();
                                }
                            })
                            .gap(1.5)
                            .grow(1.0);

                            if cat.has_link && cat.node_id >= 0 {
                                let link_id = format!("mm_uix_link_{}", cat.node_id);
                                if ui
                                    .btn_with(&link_id, |ui| {
                                        ui.row(|ui| {
                                            icons::icon(ui, icons::WORLD, 13.0).fg(accent);
                                            ui.txt("Source")
                                                .sz(crate::theme::fs(10.5))
                                                .semibold()
                                                .fg(TEXT_PRIMARY);
                                        })
                                        .center()
                                        .gap(5.0);
                                    })
                                    .px(9.0)
                                    .py(5.5)
                                    .bg(tile_idle_bg())
                                    .hover_bg(tile_idle_hover())
                                    .bd(1.0, border_subtle())
                                    .bevel_rounded(6.0, 1.2)
                                    .clicked()
                                {
                                    push_click(actions);
                                    actions.push(UiAction::MowodsOpenLink {
                                        page_id: cat.node_id,
                                    });
                                }
                            }

                            if cat.node_id >= 0 && any_non_default {
                                let rst_page_id = format!("mm_uix_rst_page_{}", cat.node_id);
                                if ui
                                    .btn_with(&rst_page_id, |ui| {
                                        ui.row(|ui| {
                                            icons::icon(ui, icons::RESPAWN, 13.0).fg(accent);
                                            ui.txt("Reset Mod")
                                                .sz(crate::theme::fs(10.5))
                                                .semibold()
                                                .fg(TEXT_PRIMARY);
                                        })
                                        .center()
                                        .gap(5.0);
                                    })
                                    .px(9.0)
                                    .py(5.5)
                                    .bg(tile_idle_bg())
                                    .hover_bg(tile_idle_hover())
                                    .bd(1.0, tint_alpha(accent, 140))
                                    .bevel_rounded(6.0, 1.2)
                                    .clicked()
                                {
                                    push_click(actions);
                                    actions.push(UiAction::MowodsResetNode {
                                        node_id: cat.node_id,
                                    });
                                }
                            }
                        })
                        .align_center()
                        .gap(9.0);

                        // Group visible entries by section (preserving order)
                        let has_sections = visible_entries
                            .iter()
                            .any(|e| !e.section_name.is_empty());

                        ui.scroll("mm_uix_entries_scroll", |ui| {
                            if has_sections {
                                let mut groups: Vec<(i32, String, String, Vec<&UixEntrySnapshot>)> =
                                    Vec::new();
                                for entry in &visible_entries {
                                    let sec_label = if entry.section_name.is_empty() {
                                        String::from("General")
                                    } else {
                                        entry.section_name.clone()
                                    };
                                    if let Some(last) = groups.last_mut() {
                                        if last.0 == entry.section_id && last.1 == sec_label {
                                            last.3.push(*entry);
                                            continue;
                                        }
                                    }
                                    groups.push((
                                        entry.section_id,
                                        sec_label,
                                        entry.section_desc.clone(),
                                        vec![*entry],
                                    ));
                                }

                                for (g_idx, (sec_id, sec_name, sec_desc, items)) in
                                    groups.into_iter().enumerate()
                                {
                                    let sec_has_reset = sec_id >= 0
                                        && items
                                            .iter()
                                            .any(|e| e.can_reset && !e.is_default && e.node_id >= 0);

                                    ui.col(|ui| {
                                        ui.row(|ui| {
                                            ui.col(|ui| {
                                                ui.txt(&sec_name)
                                                    .sz(crate::theme::fs(13.0))
                                                    .bold()
                                                    .fg(TEXT_PRIMARY);
                                                if !sec_desc.is_empty() {
                                                    ui.txt(&sec_desc)
                                                        .sz(crate::theme::fs(10.5))
                                                        .fg(TEXT_MUTED)
                                                        .wrap();
                                                }
                                            })
                                            .gap(2.0)
                                            .grow(1.0);

                                            if sec_has_reset {
                                                let sec_rst_id =
                                                    format!("mm_uix_rst_sec_{}_{}", sec_id, g_idx);
                                                if ui
                                                    .btn_with(&sec_rst_id, |ui| {
                                                        ui.row(|ui| {
                                                            icons::icon(ui, icons::RESPAWN, 12.0)
                                                                .fg(accent);
                                                            ui.txt("Reset")
                                                                .sz(crate::theme::fs(10.0))
                                                                .semibold()
                                                                .fg(TEXT_MUTED);
                                                        })
                                                        .center()
                                                        .gap(4.0);
                                                    })
                                                    .px(8.0)
                                                    .py(4.0)
                                                    .bg(tile_idle_bg())
                                                    .hover_bg(tile_idle_hover())
                                                    .bd(1.0, border_subtle())
                                                    .bevel_rounded(5.0, 1.0)
                                                    .clicked()
                                                {
                                                    push_click(actions);
                                                    actions.push(UiAction::MowodsResetNode {
                                                        node_id: sec_id,
                                                    });
                                                }
                                            }
                                        })
                                        .align_center()
                                        .gap(8.0)
                                        .pb(2.0);

                                        for entry in items {
                                            render_entry_row(ui, state, &cat, entry, accent, actions);
                                        }
                                    })
                                    .p(10.0)
                                    .gap(7.0)
                                    .bg(Color::rgba(255, 255, 255, 7))
                                    .bd(1.0, border_subtle())
                                    .bevel_rounded(9.0, 1.5);
                                }
                            } else {
                                for entry in &visible_entries {
                                    render_entry_row(ui, state, &cat, entry, accent, actions);
                                }
                            }

                            // Custom controls injected via ExpansionKitApi.GetSettingsCategory(...)
                            if !cat.custom_controls.is_empty() {
                                ui.txt("Mod Actions")
                                    .sz(crate::theme::fs(11.0))
                                    .semibold()
                                    .fg(TEXT_MUTED)
                                    .pt(6.0);

                                for ctrl in &cat.custom_controls {
                                    if !cat_matches_all
                                        && !filter.is_empty()
                                        && !ctrl.text.to_ascii_lowercase().contains(&filter)
                                    {
                                        continue;
                                    }
                                    match ctrl.kind.as_str() {
                                        "button" => {
                                            let btn_id =
                                                format!("mm_uix_cb_{}_{}", cat.id, ctrl.global_id);
                                            if ui
                                                .btn_with(&btn_id, |ui| {
                                                    ui.txt(&ctrl.text)
                                                        .sz(crate::theme::fs(11.5))
                                                        .semibold()
                                                        .align(TextAlign::Center)
                                                        .fg(TEXT_PRIMARY)
                                                        .grow(1.0);
                                                })
                                                .px(12.0)
                                                .py(8.0)
                                                .bg(tile_idle_bg())
                                                .hover_bg(tile_idle_hover())
                                                .bd(1.0, tint_alpha(accent, 120))
                                                .bevel_rounded(6.5, 1.3)
                                                .clicked()
                                            {
                                                push_click(actions);
                                                actions.push(UiAction::UixCustomButton {
                                                    category: cat.id.clone(),
                                                    global_id: ctrl.global_id,
                                                });
                                            }
                                        }
                                        "toggle" => {
                                            let active = ctrl.bool_val;
                                            let btn_id =
                                                format!("mm_uix_ct_{}_{}", cat.id, ctrl.global_id);
                                            ui.row(|ui| {
                                                ui.txt(&ctrl.text)
                                                    .sz(crate::theme::fs(12.0))
                                                    .semibold()
                                                    .fg(TEXT_PRIMARY)
                                                    .grow(1.0);
                                                if ui
                                                    .btn_with(&btn_id, |ui| {
                                                        ui.txt(if active { "ON" } else { "OFF" })
                                                            .sz(crate::theme::fs(11.0))
                                                            .bold()
                                                            .align(TextAlign::Center)
                                                            .fg(if active {
                                                                EMERALD
                                                            } else {
                                                                TEXT_MUTED
                                                            })
                                                            .grow(1.0);
                                                    })
                                                    .w(66.0)
                                                    .py(6.0)
                                                    .bg(if active {
                                                        tint_alpha(EMERALD, 45)
                                                    } else {
                                                        tile_idle_bg()
                                                    })
                                                    .hover_bg(tile_idle_hover())
                                                    .bd(
                                                        1.0,
                                                        if active {
                                                            tint_alpha(EMERALD, 190)
                                                        } else {
                                                            border_subtle()
                                                        },
                                                    )
                                                    .bevel_rounded(6.0, 1.3)
                                                    .clicked()
                                                {
                                                    push_click(actions);
                                                    actions.push(UiAction::UixCustomToggle {
                                                        category: cat.id.clone(),
                                                        global_id: ctrl.global_id,
                                                        value: !active,
                                                    });
                                                }
                                            })
                                            .align_center()
                                            .p(10.0)
                                            .gap(10.0)
                                            .bg(card_inner_bg())
                                            .bd(1.0, border_subtle())
                                            .bevel_rounded(7.0, 1.3);
                                        }
                                        "label" => {
                                            ui.txt(&ctrl.text)
                                                .sz(crate::theme::fs(11.5))
                                                .fg(TEXT_MUTED)
                                                .wrap()
                                                .px(6.0)
                                                .py(4.0);
                                        }
                                        _ => {
                                            ui.txt("").h(8.0);
                                        }
                                    }
                                }
                            }
                        })
                        .grow(1.0)
                        .gap(8.0)
                        .no_scrollbar();
                    } else {
                        ui.col(|ui| {
                            ui.txt("Select a mod category from the left.")
                                .sz(crate::theme::fs(12.5))
                                .fg(TEXT_MUTED);
                        })
                        .center()
                        .grow(1.0);
                    }
                })
                .grow(1.0)
                .h_frac(1.0)
                .p(12.0)
                .gap(9.0)
                .bg(card_bg())
                .bd(1.0, border_subtle())
                .bevel_rounded(10.5, 2.0)
                .shadow(14.0, SHADOW_CARD);
            })
            .grow(1.0)
            .gap(10.0);
        }
    })
    .grow(1.0)
    .gap(9.0);
}
