use lumenic::{Color, TextAlign, TextEditState, UiContext};

use crate::icons;
use crate::state::{AppState, UiAction};
use crate::theme::{
    push_click, tint_alpha, BORDER_SUBTLE, CARD_BG, CARD_INNER_BG, EMERALD, SHADOW_CARD, TEXT_DIM,
    TEXT_MUTED, TEXT_PRIMARY, TILE_IDLE_BG, TILE_IDLE_HOVER,
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

/// Renders the native UIExpansionKit Mod Settings screen inside the Main Menu.
pub fn render_uix_mod_settings(
    ui: &mut UiContext,
    state: &mut AppState,
    accent: Color,
    actions: &mut Vec<UiAction>,
) {
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

    ui.col(|ui| {
        // =====================================================================
        // HEADER BAR
        // =====================================================================
        ui.row(|ui| {
            if ui
                .btn_with("mm_uix_back", |ui| {
                    ui.row(|ui| {
                        icons::icon(ui, icons::BACK, 14.0).fg(TEXT_PRIMARY);
                        ui.txt("Home").sz(11.5).semibold().fg(TEXT_PRIMARY);
                    })
                    .center()
                    .gap(5.0);
                })
                .px(11.0)
                .py(6.0)
                .bg(TILE_IDLE_BG)
                .hover_bg(TILE_IDLE_HOVER)
                .bd(1.0, BORDER_SUBTLE)
                .bevel_rounded(6.0, 1.3)
                .clicked()
            {
                push_click(actions);
                state.main_menu.active_tab = String::from("home");
            }

            icons::icon(ui, icons::PUZZLE, 16.0).fg(accent);
            ui.txt("Mod Settings (UIExpansionKit)")
                .sz(14.5)
                .bold()
                .fg(TEXT_PRIMARY)
                .grow(1.0);

            let count_txt = format!("{} Mods", state.host.uix_categories.len());
            ui.txt(&count_txt)
                .sz(11.0)
                .mono()
                .fg(accent)
                .bg(tint_alpha(accent, 30))
                .px(8.0)
                .py(3.5)
                .bevel_rounded(5.0, 1.3);

            if ui
                .btn_with("mm_uix_refresh", |ui| {
                    ui.row(|ui| {
                        icons::icon(ui, icons::RELOAD, 13.0).fg(TEXT_PRIMARY);
                        ui.txt("Refresh").sz(11.0).semibold().fg(TEXT_PRIMARY);
                    })
                    .center()
                    .gap(5.0);
                })
                .px(10.0)
                .py(6.0)
                .bg(TILE_IDLE_BG)
                .hover_bg(TILE_IDLE_HOVER)
                .bd(1.0, BORDER_SUBTLE)
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
        .bg(CARD_BG)
        .bd(1.0, BORDER_SUBTLE)
        .bevel_rounded(10.0, 2.0)
        .shadow(14.0, SHADOW_CARD);

        // =====================================================================
        // BODY: LEFT CATEGORY LIST | RIGHT SETTINGS ENTRIES
        // =====================================================================
        if !state.host.uix_available && state.host.uix_categories.is_empty() {
            ui.col(|ui| {
                icons::icon(ui, icons::PUZZLE, 34.0).fg(TEXT_DIM);
                ui.txt("UIExpansionKit Not Detected")
                    .sz(16.0)
                    .bold()
                    .fg(TEXT_PRIMARY);
                ui.txt("Install UI Expansion Kit.dll in ChilloutVR/Mods to configure mod preferences here.")
                    .sz(12.0)
                    .fg(TEXT_MUTED);
            })
            .center()
            .grow(1.0)
            .gap(8.0)
            .bg(CARD_BG)
            .bd(1.0, BORDER_SUBTLE)
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
                            .placeholder("Filter mods...")
                            .bg(CARD_INNER_BG)
                            .fg(TEXT_PRIMARY)
                            .grow(1.0);
                        if state.host.is_vr && u_inp.clicked() {
                            actions.push(UiAction::OpenVirtualKeyboard {
                                surface_id: 0,
                                field_id: String::from("mm_uix_cat_search"),
                                initial_text: state.main_menu.uix_search_input.text(),
                                title: String::from("Filter Mods"),
                                placeholder: String::from("Filter mods..."),
                                max_len: 96,
                                hidden: false,
                            });
                        }
                    })
                    .align_center()
                    .gap(6.0);

                    let filter = state
                        .main_menu
                        .uix_search_input
                        .text()
                        .trim()
                        .to_ascii_lowercase();
                    let categories: Vec<(String, String, usize)> = state
                        .host
                        .uix_categories
                        .iter()
                        .filter(|c| {
                            filter.is_empty()
                                || c.display_name.to_ascii_lowercase().contains(&filter)
                                || c.id.to_ascii_lowercase().contains(&filter)
                        })
                        .map(|c| {
                            (
                                c.id.clone(),
                                c.display_name.clone(),
                                c.entries.len() + c.custom_controls.len(),
                            )
                        })
                        .collect();

                    ui.scroll("mm_uix_cat_scroll", |ui| {
                        if categories.is_empty() {
                            ui.txt("No matching mods")
                                .sz(11.5)
                                .fg(TEXT_DIM)
                                .p(10.0);
                        } else {
                            for (cat_id, display, count) in &categories {
                                let active = state.main_menu.uix_selected_category == *cat_id;
                                let btn_id = format!("mm_uix_cat_{cat_id}");
                                if ui
                                    .btn_with(&btn_id, |ui| {
                                        ui.row(|ui| {
                                            ui.txt(display)
                                                .sz(11.5)
                                                .semibold()
                                                .fg(if active { TEXT_PRIMARY } else { TEXT_MUTED })
                                                .ellipsis()
                                                .grow(1.0);
                                            let cnt = format!("{count}");
                                            ui.txt(&cnt)
                                                .sz(10.0)
                                                .mono()
                                                .fg(if active { accent } else { TEXT_DIM });
                                        })
                                        .align_center()
                                        .gap(6.0)
                                        .grow(1.0);
                                    })
                                    .px(9.0)
                                    .py(7.0)
                                    .bg(if active {
                                        tint_alpha(accent, 48)
                                    } else {
                                        CARD_INNER_BG
                                    })
                                    .hover_bg(TILE_IDLE_HOVER)
                                    .bd(
                                        1.0,
                                        if active {
                                            tint_alpha(accent, 195)
                                        } else {
                                            BORDER_SUBTLE
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
                .w(270.0)
                .h_frac(1.0)
                .p(10.0)
                .gap(8.0)
                .bg(CARD_BG)
                .bd(1.0, BORDER_SUBTLE)
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
                        ui.row(|ui| {
                            ui.txt(&cat.display_name)
                                .sz(15.0)
                                .bold()
                                .fg(TEXT_PRIMARY)
                                .grow(1.0);
                            if cat.display_name != cat.id {
                                ui.txt(&cat.id).sz(10.5).mono().fg(TEXT_DIM);
                            }
                        })
                        .align_center()
                        .gap(8.0);

                        ui.scroll("mm_uix_entries_scroll", |ui| {
                            for entry in &cat.entries {
                                let full_key = format!("{}::{}", cat.id, entry.id);
                                match entry.kind.as_str() {
                                    "bool" => {
                                        ui.row(|ui| {
                                            ui.col(|ui| {
                                                ui.txt(&entry.display_name)
                                                    .sz(12.0)
                                                    .semibold()
                                                    .fg(TEXT_PRIMARY);
                                                if !entry.description.is_empty() {
                                                    ui.txt(&entry.description)
                                                        .sz(10.5)
                                                        .fg(TEXT_MUTED)
                                                        .wrap();
                                                }
                                            })
                                            .gap(2.0)
                                            .grow(1.0);

                                            let active = entry.bool_val;
                                            let btn_id = format!("mm_uix_b_{full_key}");
                                            if ui
                                                .btn_with(&btn_id, |ui| {
                                                    ui.txt(if active { "ON" } else { "OFF" })
                                                        .sz(11.0)
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
                                                    TILE_IDLE_BG
                                                })
                                                .hover_bg(TILE_IDLE_HOVER)
                                                .bd(
                                                    1.0,
                                                    if active {
                                                        tint_alpha(EMERALD, 190)
                                                    } else {
                                                        BORDER_SUBTLE
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
                                        .gap(10.0)
                                        .bg(CARD_INNER_BG)
                                        .bd(1.0, BORDER_SUBTLE)
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

                                        ui.push_id(&full_key);
                                        let mut resp = ui
                                            .slider(&entry.display_name, &mut slider_v, min_f..=max_f)
                                            .fill(accent)
                                            .bg(CARD_INNER_BG)
                                            .bd(1.0, BORDER_SUBTLE)
                                            .bevel_rounded(7.0, 1.3)
                                            .w_frac(1.0);
                                        if resp.changed() {
                                            let mut snapped = slider_v as f64;
                                            if entry.snap_val > 0.0 {
                                                snapped = (snapped / entry.snap_val).round()
                                                    * entry.snap_val;
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
                                        if !entry.description.is_empty() && resp.hovered() {
                                            state.main_menu.hovered_tooltip = entry.description.clone();
                                        }
                                        ui.pop_id();
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
                                                    .sz(12.0)
                                                    .semibold()
                                                    .fg(TEXT_PRIMARY);
                                                if !entry.description.is_empty() {
                                                    ui.txt(&entry.description)
                                                        .sz(10.5)
                                                        .fg(TEXT_MUTED)
                                                        .wrap();
                                                }
                                            })
                                            .gap(2.0)
                                            .grow(1.0);

                                            let inp_id = format!("mm_uix_n_{full_key}");
                                            let submitted = ui
                                                .input(&inp_id, buf)
                                                .w(120.0)
                                                .bg(TILE_IDLE_BG)
                                                .fg(TEXT_PRIMARY)
                                                .submitted();

                                            let apply_id = format!("mm_uix_na_{full_key}");
                                            let clicked = ui
                                                .btn_with(&apply_id, |ui| {
                                                    ui.txt("Apply")
                                                        .sz(10.5)
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
                                        .bg(CARD_INNER_BG)
                                        .bd(1.0, BORDER_SUBTLE)
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
                                            ui.txt(&entry.display_name)
                                                .sz(12.0)
                                                .semibold()
                                                .fg(TEXT_PRIMARY);
                                            if !entry.description.is_empty() {
                                                ui.txt(&entry.description)
                                                    .sz(10.5)
                                                    .fg(TEXT_MUTED)
                                                    .wrap();
                                            }
                                            ui.row(|ui| {
                                                let inp_id = format!("mm_uix_t_{full_key}");
                                                let submitted = ui
                                                    .input(&inp_id, buf)
                                                    .bg(TILE_IDLE_BG)
                                                    .fg(TEXT_PRIMARY)
                                                    .grow(1.0)
                                                    .submitted();

                                                let apply_id = format!("mm_uix_ta_{full_key}");
                                                let clicked = ui
                                                    .btn_with(&apply_id, |ui| {
                                                        ui.txt("Save")
                                                            .sz(10.5)
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
                                        .bg(CARD_INNER_BG)
                                        .bd(1.0, BORDER_SUBTLE)
                                        .bevel_rounded(7.0, 1.3);
                                    }

                                    "enum" => {
                                        ui.col(|ui| {
                                            ui.txt(&entry.display_name)
                                                .sz(12.0)
                                                .semibold()
                                                .fg(TEXT_PRIMARY);
                                            if !entry.description.is_empty() {
                                                ui.txt(&entry.description)
                                                    .sz(10.5)
                                                    .fg(TEXT_MUTED)
                                                    .wrap();
                                            }

                                            if entry.enum_options.len() <= 4 {
                                                ui.row(|ui| {
                                                    for (idx, opt) in
                                                        entry.enum_options.iter().enumerate()
                                                    {
                                                        let active = idx as i32 == entry.enum_index;
                                                        let opt_id =
                                                            format!("mm_uix_e_{full_key}_{idx}");
                                                        if ui
                                                            .btn_with(&opt_id, |ui| {
                                                                ui.txt(opt)
                                                                    .sz(11.0)
                                                                    .semibold()
                                                                    .align(TextAlign::Center)
                                                                    .fg(if active {
                                                                        TEXT_PRIMARY
                                                                    } else {
                                                                        TEXT_MUTED
                                                                    })
                                                                    .grow(1.0);
                                                            })
                                                            .grow(1.0)
                                                            .px(8.0)
                                                            .py(5.5)
                                                            .bg(if active {
                                                                tint_alpha(accent, 52)
                                                            } else {
                                                                TILE_IDLE_BG
                                                            })
                                                            .hover_bg(TILE_IDLE_HOVER)
                                                            .bd(
                                                                1.0,
                                                                if active {
                                                                    tint_alpha(accent, 195)
                                                                } else {
                                                                    BORDER_SUBTLE
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
                                                            icons::icon(ui, icons::BACK, 13.0)
                                                                .fg(TEXT_PRIMARY);
                                                        })
                                                        .px(10.0)
                                                        .py(5.5)
                                                        .bg(TILE_IDLE_BG)
                                                        .hover_bg(TILE_IDLE_HOVER)
                                                        .bd(1.0, BORDER_SUBTLE)
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

                                                    let idx_badge =
                                                        format!("{} ({}/{})", cur_lbl, cur_idx + 1, len);
                                                    ui.txt(&idx_badge)
                                                        .sz(11.5)
                                                        .semibold()
                                                        .align(TextAlign::Center)
                                                        .fg(TEXT_PRIMARY)
                                                        .bg(TILE_IDLE_BG)
                                                        .bd(1.0, tint_alpha(accent, 140))
                                                        .px(10.0)
                                                        .py(5.5)
                                                        .bevel_rounded(5.5, 1.3)
                                                        .grow(1.0);

                                                    if ui
                                                        .btn_with(&next_id, |ui| {
                                                            icons::icon(ui, icons::CHEVRON_RIGHT, 13.0)
                                                                .fg(TEXT_PRIMARY);
                                                        })
                                                        .px(10.0)
                                                        .py(5.5)
                                                        .bg(TILE_IDLE_BG)
                                                        .hover_bg(TILE_IDLE_HOVER)
                                                        .bd(1.0, BORDER_SUBTLE)
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
                                        .bg(CARD_INNER_BG)
                                        .bd(1.0, BORDER_SUBTLE)
                                        .bevel_rounded(7.0, 1.3);
                                    }

                                    _ => {}
                                }
                            }

                            // Custom controls injected via ExpansionKitApi.GetSettingsCategory(...)
                            if !cat.custom_controls.is_empty() {
                                ui.txt("Mod Actions")
                                    .sz(11.0)
                                    .semibold()
                                    .fg(TEXT_MUTED)
                                    .pt(6.0);

                                for ctrl in &cat.custom_controls {
                                    match ctrl.kind.as_str() {
                                        "button" => {
                                            let btn_id = format!(
                                                "mm_uix_cb_{}_{}",
                                                cat.id, ctrl.global_id
                                            );
                                            if ui
                                                .btn_with(&btn_id, |ui| {
                                                    ui.txt(&ctrl.text)
                                                        .sz(11.5)
                                                        .semibold()
                                                        .align(TextAlign::Center)
                                                        .fg(TEXT_PRIMARY)
                                                        .grow(1.0);
                                                })
                                                .px(12.0)
                                                .py(8.0)
                                                .bg(TILE_IDLE_BG)
                                                .hover_bg(TILE_IDLE_HOVER)
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
                                            let btn_id = format!(
                                                "mm_uix_ct_{}_{}",
                                                cat.id, ctrl.global_id
                                            );
                                            ui.row(|ui| {
                                                ui.txt(&ctrl.text)
                                                    .sz(12.0)
                                                    .semibold()
                                                    .fg(TEXT_PRIMARY)
                                                    .grow(1.0);
                                                if ui
                                                    .btn_with(&btn_id, |ui| {
                                                        ui.txt(if active { "ON" } else { "OFF" })
                                                            .sz(11.0)
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
                                                        TILE_IDLE_BG
                                                    })
                                                    .hover_bg(TILE_IDLE_HOVER)
                                                    .bd(
                                                        1.0,
                                                        if active {
                                                            tint_alpha(EMERALD, 190)
                                                        } else {
                                                            BORDER_SUBTLE
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
                                            .bg(CARD_INNER_BG)
                                            .bd(1.0, BORDER_SUBTLE)
                                            .bevel_rounded(7.0, 1.3);
                                        }
                                        "label" => {
                                            ui.txt(&ctrl.text)
                                                .sz(11.5)
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
                        .gap(7.0)
                        .no_scrollbar();
                    } else {
                        ui.col(|ui| {
                            ui.txt("Select a mod category from the left.")
                                .sz(12.5)
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
                .bg(CARD_BG)
                .bd(1.0, BORDER_SUBTLE)
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
