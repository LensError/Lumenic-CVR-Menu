use lumenic::{Color, TextAlign, UiContext};

use crate::icons;
use crate::state::{AppState, CvrSettingItem, UiAction};
use crate::theme::{
    push_click, tint_alpha, border_subtle, card_bg, card_inner_bg, EMERALD, SHADOW_CARD, TEXT_DIM,
    TEXT_MUTED, TEXT_PRIMARY, tile_idle_bg, tile_idle_hover,
};

const SETTINGS_TABS: [(&str, &str, &str); 16] = [
    ("lumenic", icons::BOLT, "LumenicMenu"),
    ("about_lumenic", icons::INFO, "About LumenicMenu"),
    ("colors", icons::STAR, "Player Colors"),
    ("general", icons::HOME, "General"),
    ("audio", icons::VOLUME, "Audio"),
    ("graphics", icons::CAMERA, "Graphics"),
    ("controls", icons::VR_HEADSET, "Controls & VR"),
    ("ik", icons::FBT, "IK & Full Body"),
    ("hud", icons::BELL, "HUD & Plates"),
    ("interactions", icons::CURSOR, "Interactions"),
    ("implementation", icons::PUZZLE, "Implementation"),
    ("safety", icons::CHECK, "Safety & Filter"),
    ("cache", icons::TRASH, "Cache"),
    ("experimental", icons::TOOLBOX, "Experimental"),
    ("support", icons::MAIL, "Support"),
    ("legal", icons::SHIELD, "Legal"),
];

pub fn tab_index(cat_id: &str) -> usize {
    SETTINGS_TABS
        .iter()
        .position(|(id, _, _)| *id == cat_id)
        .unwrap_or(0)
}

fn render_subcat_card(
    ui: &mut UiContext,
    title: &str,
    count: Option<usize>,
    accent: Color,
    header_extra: impl FnOnce(&mut UiContext),
    body: impl FnOnce(&mut UiContext),
) {
    ui.col(|ui| {
        if !title.is_empty() {
            ui.row(|ui| {
                ui.txt("")
                    .w(3.5)
                    .h(crate::theme::text_box(16.0))
                    .bg(accent)
                    .r(2.0);
                ui.txt(title)
                    .sz(crate::theme::fs(13.5))
                    .bold()
                    .fg(TEXT_PRIMARY)
                    .ellipsis()
                    .grow(1.0);
                if let Some(c) = count {
                    ui.txt(&format!("{c}"))
                        .sz(crate::theme::fs(10.0))
                        .semibold()
                        .fg(TEXT_DIM)
                        .px(6.0)
                        .py(2.0)
                        .bg(Color::rgba(10, 14, 22, 130))
                        .r(4.0);
                }
                header_extra(ui);
            })
            .align_center()
            .gap(8.0)
            .pb(2.0);
        }
        body(ui);
    })
    .w_frac(1.0)
    .p(10.0)
    .gap(7.0)
    .bg(Color::rgba(18, 22, 32, 145))
    .bd(1.0, tint_alpha(accent, 55))
    .bevel_rounded(9.0, 1.5);
}

fn compact_toggle_cell(
    ui: &mut UiContext,
    id: &str,
    label: &str,
    active: bool,
) -> (bool, bool) {
    let mut clicked = false;
    let mut resp = ui
        .row(|ui| {
            ui.txt(label)
                .sz(crate::theme::fs(11.0))
                .semibold()
                .fg(TEXT_PRIMARY)
                .ellipsis()
                .grow(1.0);

            if ui
                .btn_with(id, |ui| {
                    ui.txt(if active { "ON" } else { "OFF" })
                        .sz(crate::theme::fs(10.0))
                        .bold()
                        .align(TextAlign::Center)
                        .fg(if active { EMERALD } else { TEXT_MUTED })
                        .grow(1.0);
                })
                .w(crate::theme::text_box(50.0))
                .h(crate::theme::text_box(22.0))
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
                .bevel_rounded(5.0, 1.3)
                .clicked()
            {
                clicked = true;
            }
        })
        .align_center()
        .grow(1.0)
        .px(8.0)
        .py(5.0)
        .gap(6.0)
        .bg(card_inner_bg())
        .bd(1.0, border_subtle())
        .bevel_rounded(6.0, 1.3);
    (clicked, resp.hovered())
}

fn render_cvr_item(
    ui: &mut UiContext,
    state: &mut AppState,
    item: &CvrSettingItem,
    accent: Color,
    actions: &mut Vec<UiAction>,
) {
    let is_hovered;
    match item.kind.as_str() {
        "bool" => {
            let (clicked, hov) = compact_toggle_cell(
                ui,
                &format!("mm_sb_{}", item.key),
                &item.label,
                item.bool_val,
            );
            is_hovered = hov;
            if clicked {
                push_click(actions);
                actions.push(UiAction::CvrSetBool {
                    key: item.key.clone(),
                    value: !item.bool_val,
                });
            }
        }
        "int_slider" => {
            let min_v = item.min_val;
            let max_v = item.max_val.max(min_v + 1);
            let host_val = item.int_val.clamp(min_v, max_v);
            let (cur_f, last_host) = state
                .main_menu
                .settings_sliders
                .entry(item.key.clone())
                .or_insert((host_val as f32, host_val));
            if *last_host != host_val {
                *cur_f = host_val as f32;
                *last_host = host_val;
            }
            let mut v = *cur_f;
            let val_txt = if item.unit.is_empty() {
                format!("{}", v.round() as i32)
            } else {
                format!("{} {}", v.round() as i32, item.unit)
            };
            let res = crate::widgets::styled_slider(
                ui,
                &item.key,
                &item.label,
                &val_txt,
                &mut v,
                (min_v as f32)..=(max_v as f32),
                accent,
                true,
                false,
            );
            if res.changed {
                let next_i = (v.round() as i32).clamp(min_v, max_v);
                *cur_f = next_i as f32;
                if next_i != *last_host {
                    *last_host = next_i;
                    actions.push(UiAction::CvrSetInt {
                        key: item.key.clone(),
                        value: next_i,
                    });
                }
            }
            is_hovered = res.hovered;
        }
        "int_choice" | "str_choice" => {
            let is_int = item.kind == "int_choice";
            let total_label_chars: usize = item.choice_labels.iter().map(|s| s.len()).sum();
            let use_dropdown = item.choice_vals.len() <= 1
                || item.choice_vals.len() > 3
                || total_label_chars > 30;
            let mut resp = ui
                .col(|ui| {
                    ui.txt(&item.label)
                        .sz(crate::theme::fs(11.0))
                        .semibold()
                        .fg(TEXT_PRIMARY)
                        .ellipsis();
                    if use_dropdown {
                        let cur_label = item
                            .choice_vals
                            .iter()
                            .zip(item.choice_labels.iter())
                            .find(|(v, _)| *v == &item.str_val)
                            .map(|(_, l)| l.as_str())
                            .or_else(|| item.choice_labels.first().map(|s| s.as_str()))
                            .unwrap_or("Select...");
                        if ui
                            .btn_with(&format!("mm_dd_{}", item.key), |ui| {
                                ui.row(|ui| {
                                    ui.txt(cur_label)
                                        .sz(crate::theme::fs(10.5))
                                        .semibold()
                                        .fg(TEXT_PRIMARY)
                                        .ellipsis()
                                        .grow(1.0);
                                    icons::icon(ui, icons::CHEVRON_DOWN, 14.0).fg(accent);
                                })
                                .align_center()
                                .gap(6.0)
                                .grow(1.0);
                            })
                            .w_frac(1.0)
                            .px(9.0)
                            .py(5.0)
                            .bg(tile_idle_bg())
                            .hover_bg(tile_idle_hover())
                            .bd(1.0, tint_alpha(accent, 140))
                            .bevel_rounded(5.0, 1.3)
                            .clicked()
                        {
                            push_click(actions);
                            let mut options = Vec::with_capacity(item.choice_vals.len());
                            for (val, lbl) in item.choice_vals.iter().zip(item.choice_labels.iter()) {
                                let act = if is_int {
                                    if let Ok(parsed) = val.parse::<i32>() {
                                        UiAction::CvrSetInt {
                                            key: item.key.clone(),
                                            value: parsed,
                                        }
                                    } else {
                                        UiAction::CvrSetString {
                                            key: item.key.clone(),
                                            value: val.clone(),
                                        }
                                    }
                                } else {
                                    UiAction::CvrSetString {
                                        key: item.key.clone(),
                                        value: val.clone(),
                                    }
                                };
                                options.push(crate::state::DropdownOption {
                                    label: lbl.clone(),
                                    action: act,
                                    selected: item.str_val == *val,
                                });
                            }
                            state.main_menu.active_dropdown = Some(crate::state::ActiveDropdown {
                                title: item.label.clone(),
                                options,
                            });
                        }
                    } else {
                        ui.row(|ui| {
                            for (idx, (val, lbl)) in item
                                .choice_vals
                                .iter()
                                .zip(item.choice_labels.iter())
                                .enumerate()
                            {
                                let active = item.str_val == *val;
                                if ui
                                    .btn_with(&format!("mm_sc_{}_{idx}", item.key), |ui| {
                                        ui.txt(lbl)
                                            .sz(crate::theme::fs(10.0))
                                            .semibold()
                                            .align(TextAlign::Center)
                                            .fg(if active { TEXT_PRIMARY } else { TEXT_MUTED })
                                            .grow(1.0);
                                    })
                                    .grow(1.0)
                                    .py(3.5)
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
                                    .bevel_rounded(4.5, 1.3)
                                    .clicked()
                                {
                                    push_click(actions);
                                    if is_int {
                                        if let Ok(parsed) = val.parse::<i32>() {
                                            actions.push(UiAction::CvrSetInt {
                                                key: item.key.clone(),
                                                value: parsed,
                                            });
                                        }
                                    } else {
                                        actions.push(UiAction::CvrSetString {
                                            key: item.key.clone(),
                                            value: val.clone(),
                                        });
                                    }
                                }
                            }
                        })
                        .gap(4.0);
                    }
                })
                .grow(1.0)
                .px(8.0)
                .py(5.0)
                .gap(4.0)
                .bg(card_inner_bg())
                .bd(1.0, border_subtle())
                .bevel_rounded(6.0, 1.3);
            is_hovered = resp.hovered();
        }
        "int_step" => {
            let step = if item.int_val.abs() >= 1000 {
                100
            } else if item.int_val.abs() >= 100 {
                10
            } else {
                1
            };
            let mut resp = ui
                .row(|ui| {
                    ui.txt(&item.label)
                        .sz(crate::theme::fs(10.5))
                        .semibold()
                        .fg(TEXT_PRIMARY)
                        .ellipsis()
                        .grow(1.0);
                    if ui
                        .btn_with(&format!("mm_sm_{}", item.key), |ui| {
                            ui.txt("-").sz(crate::theme::fs(11.0)).bold().fg(TEXT_PRIMARY);
                        })
                        .w(24.0)
                        .py(3.0)
                        .bg(tile_idle_bg())
                        .hover_bg(tile_idle_hover())
                        .bd(1.0, border_subtle())
                        .bevel_rounded(4.5, 1.3)
                        .clicked()
                    {
                        push_click(actions);
                        actions.push(UiAction::CvrSetInt {
                            key: item.key.clone(),
                            value: item.int_val - step,
                        });
                    }
                    ui.txt(&format!("{}", item.int_val))
                        .sz(crate::theme::fs(10.5))
                        .bold()
                        .align(TextAlign::Center)
                        .fg(accent)
                        .min_w(38.0);
                    if ui
                        .btn_with(&format!("mm_sp_{}", item.key), |ui| {
                            ui.txt("+").sz(crate::theme::fs(11.0)).bold().fg(TEXT_PRIMARY);
                        })
                        .w(24.0)
                        .py(3.0)
                        .bg(tile_idle_bg())
                        .hover_bg(tile_idle_hover())
                        .bd(1.0, border_subtle())
                        .bevel_rounded(4.5, 1.3)
                        .clicked()
                    {
                        push_click(actions);
                        actions.push(UiAction::CvrSetInt {
                            key: item.key.clone(),
                            value: item.int_val + step,
                        });
                    }
                })
                .align_center()
                .grow(1.0)
                .px(8.0)
                .py(5.0)
                .gap(5.0)
                .bg(card_inner_bg())
                .bd(1.0, border_subtle())
                .bevel_rounded(6.0, 1.3);
            is_hovered = resp.hovered();
        }
        "float_step" => {
            let step = if item.float_val.abs() <= 2.0 {
                0.05
            } else if item.float_val.abs() <= 10.0 {
                0.1
            } else {
                1.0
            };
            let mut resp = ui
                .row(|ui| {
                    ui.txt(&item.label)
                        .sz(crate::theme::fs(10.5))
                        .semibold()
                        .fg(TEXT_PRIMARY)
                        .ellipsis()
                        .grow(1.0);
                    if ui
                        .btn_with(&format!("mm_fm_{}", item.key), |ui| {
                            ui.txt("-").sz(crate::theme::fs(11.0)).bold().fg(TEXT_PRIMARY);
                        })
                        .w(24.0)
                        .py(3.0)
                        .bg(tile_idle_bg())
                        .hover_bg(tile_idle_hover())
                        .bd(1.0, border_subtle())
                        .bevel_rounded(4.5, 1.3)
                        .clicked()
                    {
                        push_click(actions);
                        let next = ((item.float_val - step) * 100.0).round() / 100.0;
                        actions.push(UiAction::CvrSetFloat {
                            key: item.key.clone(),
                            value: next,
                        });
                    }
                    ui.txt(&format!("{:.2}", item.float_val))
                        .sz(crate::theme::fs(10.5))
                        .bold()
                        .align(TextAlign::Center)
                        .fg(accent)
                        .min_w(42.0);
                    if ui
                        .btn_with(&format!("mm_fp_{}", item.key), |ui| {
                            ui.txt("+").sz(crate::theme::fs(11.0)).bold().fg(TEXT_PRIMARY);
                        })
                        .w(24.0)
                        .py(3.0)
                        .bg(tile_idle_bg())
                        .hover_bg(tile_idle_hover())
                        .bd(1.0, border_subtle())
                        .bevel_rounded(4.5, 1.3)
                        .clicked()
                    {
                        push_click(actions);
                        let next = ((item.float_val + step) * 100.0).round() / 100.0;
                        actions.push(UiAction::CvrSetFloat {
                            key: item.key.clone(),
                            value: next,
                        });
                    }
                })
                .align_center()
                .grow(1.0)
                .px(8.0)
                .py(5.0)
                .gap(5.0)
                .bg(card_inner_bg())
                .bd(1.0, border_subtle())
                .bevel_rounded(6.0, 1.3);
            is_hovered = resp.hovered();
        }
        _ => {
            let mut resp = ui
                .row(|ui| {
                    ui.txt(&item.label)
                        .sz(crate::theme::fs(10.5))
                        .semibold()
                        .fg(TEXT_PRIMARY)
                        .ellipsis()
                        .grow(1.0);
                    let val_txt = if item.str_val.is_empty() {
                        "-"
                    } else {
                        &item.str_val
                    };
                    ui.txt(val_txt).sz(crate::theme::fs(10.5)).fg(TEXT_MUTED).ellipsis();
                })
                .align_center()
                .grow(1.0)
                .px(8.0)
                .py(5.0)
                .gap(6.0)
                .bg(card_inner_bg())
                .bd(1.0, border_subtle())
                .bevel_rounded(6.0, 1.3);
            is_hovered = resp.hovered();
        }
    }

    if is_hovered && !item.tooltip.is_empty() {
        state.main_menu.hovered_tooltip = item.tooltip.clone();
    }
}

/// Renders the compact 2-column Phase 3D Settings page covering ALL ChilloutVR + LumenicMenu settings.
pub fn render_settings_page(
    ui: &mut UiContext,
    state: &mut AppState,
    accent: Color,
    actions: &mut Vec<UiAction>,
) {
    ui.col(|ui| {
        // Header Bar: Title + Live Search across ALL settings + Mod/Vanilla jump
        ui.row(|ui| {
            icons::icon(ui, icons::SETTINGS, 17.0).fg(accent);
            ui.txt("Settings").sz(crate::theme::fs(15.0)).bold().fg(TEXT_PRIMARY);

            ui.row(|ui| {
                icons::icon(ui, icons::SEARCH, 13.0).fg(TEXT_MUTED);
                let mut s_inp = ui
                    .input("mm_set_search", &mut state.main_menu.settings_search_input)
                    .placeholder("Search all settings...")
                    .bg(card_inner_bg())
                    .fg(TEXT_PRIMARY)
                    .grow(1.0);
                if state.host.is_vr && s_inp.clicked() {
                    actions.push(UiAction::OpenVirtualKeyboard {
                        surface_id: 0,
                        field_id: String::from("mm_set_search"),
                        initial_text: state.main_menu.settings_search_input.text(),
                        title: String::from("Search Settings"),
                        placeholder: String::from("Search all settings..."),
                        max_len: 96,
                        hidden: false,
                    });
                }
                if !state.main_menu.settings_search_input.text().is_empty()
                    && ui
                        .btn_with("mm_set_search_clear", |ui| {
                            icons::icon(ui, icons::CLOSE, 12.0).fg(TEXT_MUTED);
                        })
                        .p(4.0)
                        .bg(Color::TRANSPARENT)
                        .hover_bg(tile_idle_hover())
                        .bevel_rounded(4.0, 1.3)
                        .clicked()
                {
                    push_click(actions);
                    state.main_menu.settings_search_input.set_text("");
                }
            })
            .align_center()
            .gap(6.0)
            .grow(1.0)
            .px(8.0);
        })
        .align_center()
        .w_frac(1.0)
        .px(12.0)
        .py(7.0)
        .gap(8.0)
        .bg(card_bg())
        .bd(1.0, border_subtle())
        .bevel_rounded(10.0, 2.0)
        .shadow(14.0, SHADOW_CARD);

        // Body: Compact Left Category List | Dense 2-Column Settings Grid
        ui.row(|ui| {
            ui.col(|ui| {
                ui.scroll("mm_set_tabs_scroll", |ui| {
                    for (idx, (cat_id, icon_p, lbl)) in SETTINGS_TABS.iter().enumerate() {
                        let active = state.main_menu.settings_tab == idx;
                        let count = if *cat_id == "lumenic" {
                            9
                        } else if *cat_id == "colors" {
                            state.host.player_color_presets.len()
                        } else if *cat_id == "support"
                            || *cat_id == "about_lumenic"
                            || *cat_id == "legal"
                        {
                            0
                        } else {
                            state
                                .host
                                .cvr_settings
                                .items
                                .iter()
                                .filter(|it| it.category == *cat_id)
                                .count()
                        };
                        if ui
                            .btn_with(&format!("mm_set_tab_{idx}"), |ui| {
                                ui.row(|ui| {
                                    icons::icon(ui, icon_p, 13.5)
                                        .fg(if active { accent } else { TEXT_MUTED });
                                    ui.txt(lbl)
                                        .sz(crate::theme::fs(11.5))
                                        .semibold()
                                        .fg(if active { TEXT_PRIMARY } else { TEXT_MUTED })
                                        .ellipsis()
                                        .grow(1.0);
                                    if count > 0 {
                                        ui.txt(&format!("{count}"))
                                            .sz(crate::theme::fs(9.5))
                                            .fg(if active { accent } else { TEXT_DIM });
                                    }
                                })
                                .align_center()
                                .gap(7.0)
                                .grow(1.0);
                            })
                            .px(9.0)
                            .py(6.5)
                            .bg(if active {
                                tint_alpha(accent, 48)
                            } else {
                                card_inner_bg()
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
                            .bevel_rounded(6.0, 1.3)
                            .clicked()
                        {
                            push_click(actions);
                            state.main_menu.settings_tab = idx;
                        }
                    }

                    if state.host.uix_available
                        && ui
                            .btn_with("mm_set_jump_mods", |ui| {
                                ui.row(|ui| {
                                    icons::icon(ui, icons::PUZZLE, 13.5).fg(accent);
                                    ui.txt("Mod Settings")
                                        .sz(crate::theme::fs(11.5))
                                        .semibold()
                                        .fg(TEXT_PRIMARY)
                                        .ellipsis()
                                        .grow(1.0);
                                    icons::icon(ui, icons::CHEVRON_RIGHT, 12.0).fg(TEXT_DIM);
                                })
                                .align_center()
                                .gap(7.0)
                                .grow(1.0);
                            })
                            .px(9.0)
                            .py(6.5)
                            .bg(card_inner_bg())
                            .hover_bg(tile_idle_hover())
                            .bd(1.0, tint_alpha(accent, 120))
                            .bevel_rounded(6.0, 1.3)
                            .clicked()
                    {
                        push_click(actions);
                        crate::main_menu::push_nav_history(state);
                        state.main_menu.active_tab = String::from("modSettings");
                        crate::main_menu::close_all_detail_overlays(actions);
                        actions.push(UiAction::RefreshUixSettings);
                    }
                })
                .grow(1.0)
                .gap(4.5)
                .no_scrollbar();
            })
            .w(214.0)
            .h_frac(1.0)
            .p(8.0)
            .bg(card_bg())
            .bd(1.0, border_subtle())
            .bevel_rounded(10.5, 2.0)
            .shadow(14.0, SHADOW_CARD);

            let q = state
                .main_menu
                .settings_search_input
                .text()
                .trim()
                .to_ascii_lowercase();
            let active_cat = SETTINGS_TABS
                .get(state.main_menu.settings_tab)
                .map(|t| t.0)
                .unwrap_or("lumenic");

            ui.col(|ui| {
                if q.is_empty() && active_cat == "safety" {
                    let av_count = state
                        .host
                        .cvr_settings
                        .items
                        .iter()
                        .filter(|it| it.category == "safety" && it.sub_tab != "props")
                        .count();
                    let prop_count = state
                        .host
                        .cvr_settings
                        .items
                        .iter()
                        .filter(|it| it.category == "safety" && it.sub_tab == "props")
                        .count();
                    ui.row(|ui| {
                        for (idx, icon_p, lbl, cnt) in [
                            (0usize, icons::AVATAR, "Avatar Filters", av_count),
                            (1usize, icons::CUBE, "Prop Filters", prop_count),
                        ] {
                            let active = state.main_menu.safety_sub_tab == idx;
                            let btn_id = format!("mm_set_safety_sub_{idx}");
                            if ui
                                .btn_with(&btn_id, |ui| {
                                    ui.row(|ui| {
                                        icons::icon(ui, icon_p, 14.0)
                                            .fg(if active { accent } else { TEXT_MUTED });
                                        ui.txt(lbl)
                                            .sz(crate::theme::fs(12.0))
                                            .semibold()
                                            .fg(if active { TEXT_PRIMARY } else { TEXT_MUTED });
                                        let c_txt = format!("{cnt}");
                                        ui.txt(&c_txt)
                                            .sz(crate::theme::fs(10.0))
                                            .fg(if active { accent } else { TEXT_DIM });
                                    })
                                    .center()
                                    .gap(7.0)
                                    .grow(1.0);
                                })
                                .grow(1.0)
                                .px(12.0)
                                .py(8.0)
                                .bg(if active {
                                    tint_alpha(accent, 48)
                                } else {
                                    card_inner_bg()
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
                                .bevel_rounded(7.0, 1.4)
                                .clicked()
                            {
                                push_click(actions);
                                state.main_menu.safety_sub_tab = idx;
                            }
                        }
                    })
                    .align_center()
                    .gap(8.0)
                    .pb(2.0);
                } else if q.is_empty() && active_cat == "legal" {
                    render_legal_sub_tabs(ui, state, accent, actions);
                }

                ui.scroll("mm_set_body_scroll", |ui| {
                    if q.is_empty() && active_cat == "lumenic" {
                        render_lumenic_settings(ui, state, "", accent, actions);
                    } else if q.is_empty() && active_cat == "about_lumenic" {
                        render_about_lumenic_section(ui, state, accent, actions);
                    } else if q.is_empty() && active_cat == "colors" {
                        render_player_colors_section(ui, state, accent, actions);
                    } else if q.is_empty() && active_cat == "support" {
                        render_support_section(ui, state, accent, actions);
                    } else if q.is_empty() && active_cat == "legal" {
                        render_legal_section(ui, state, accent, actions);
                    } else {
                        let mut total_hits = 0usize;
                        if !q.is_empty() {
                            total_hits += render_lumenic_settings(ui, state, &q, accent, actions);
                            if "about lumenicmenu update version github issues discord whats new"
                                .contains(&q)
                                || q.contains("update")
                                || q.contains("about")
                                || q.contains("lumenic")
                            {
                                render_about_lumenic_section(ui, state, accent, actions);
                                total_hits += 1;
                            }
                            if "player colors colorize menu primary secondary emission preset swatch"
                                .contains(&q)
                                || q.contains("color")
                                || q.contains("colour")
                            {
                                render_player_colors_section(ui, state, accent, actions);
                                total_hits += 1;
                            }
                            if "support help discord github patreon twitter logs bread bug issue"
                                .contains(&q)
                                || q.contains("support")
                                || q.contains("bread")
                                || q.contains("logs")
                            {
                                render_support_section(ui, state, accent, actions);
                                total_hits += 1;
                            }
                        }

                        let filtered: Vec<CvrSettingItem> = state
                            .host
                            .cvr_settings
                            .items
                            .iter()
                            .filter(|it| {
                                if q.is_empty() {
                                    if it.category != active_cat {
                                        return false;
                                    }
                                    if active_cat == "safety" {
                                        let want_props = state.main_menu.safety_sub_tab == 1;
                                        (it.sub_tab == "props") == want_props
                                    } else {
                                        true
                                    }
                                } else {
                                    it.label.to_ascii_lowercase().contains(&q)
                                        || it.key.to_ascii_lowercase().contains(&q)
                                        || it.category.to_ascii_lowercase().contains(&q)
                                }
                            })
                            .cloned()
                            .collect();

                        total_hits += filtered.len();

                        // Group consecutive items into dedicated subcategory cards with larger headings
                        let mut cur_group = String::new();
                        let mut bucket: Vec<&CvrSettingItem> = Vec::new();
                        let mut flush_bucket =
                            |ui: &mut UiContext,
                             state: &mut AppState,
                             grp: &str,
                             items: &mut Vec<&CvrSettingItem>| {
                                if items.is_empty() {
                                    return;
                                }
                                let title = if grp.is_empty() { "General" } else { grp };
                                let count = items.len();
                                render_subcat_card(
                                    ui,
                                    title,
                                    Some(count),
                                    accent,
                                    |_| {},
                                    |ui| {
                                        for pair in items.chunks(2) {
                                            ui.row(|ui| {
                                                render_cvr_item(ui, state, pair[0], accent, actions);
                                                if pair.len() > 1 {
                                                    render_cvr_item(
                                                        ui, state, pair[1], accent, actions,
                                                    );
                                                } else {
                                                    ui.col(|_| {}).grow(1.0);
                                                }
                                            })
                                            .align_center()
                                            .gap(7.0);
                                        }
                                    },
                                );
                                items.clear();
                            };

                        for item in &filtered {
                            if item.group != cur_group {
                                flush_bucket(ui, state, &cur_group, &mut bucket);
                                cur_group = item.group.clone();
                            }
                            bucket.push(item);
                        }
                        flush_bucket(ui, state, &cur_group, &mut bucket);

                        if total_hits == 0 {
                            ui.col(|ui| {
                                icons::icon(ui, icons::SEARCH, 26.0).fg(TEXT_DIM);
                                ui.txt("No matching settings")
                                    .sz(crate::theme::fs(12.5))
                                    .semibold()
                                    .fg(TEXT_MUTED);
                            })
                            .center()
                            .py(36.0)
                            .gap(6.0);
                        }
                    }
                })
                .grow(1.0)
                .gap(9.0)
                .no_scrollbar();
            })
            .grow(1.0)
            .h_frac(1.0)
            .p(9.0)
            .bg(card_bg())
            .bd(1.0, border_subtle())
            .bevel_rounded(10.5, 2.0)
            .shadow(14.0, SHADOW_CARD);
        })
        .w_frac(1.0)
        .grow(1.0)
        .gap(9.0);
    })
    .w_frac(1.0)
    .grow(1.0)
    .gap(9.0);
}

fn render_lumenic_settings(
    ui: &mut UiContext,
    state: &mut AppState,
    q: &str,
    accent: Color,
    actions: &mut Vec<UiAction>,
) -> usize {
    let mut hits = 0usize;
    let m = |lbl: &str, extra: &str| {
        q.is_empty()
            || lbl.to_ascii_lowercase().contains(q)
            || extra.to_ascii_lowercase().contains(q)
            || "lumenicmenu".contains(q)
    };

    // --- APPEARANCE & TYPOGRAPHY ---
    let show_text_size = m("Text Size", "font text size typography scale readability");
    let show_theme_preset = m(
        "Theme Preset",
        "theme preset oled frost glass high contrast slate style appearance",
    );
    let show_panel_opacity = m("Panel Opacity", "panel opacity transparency glass alpha");
    let show_cursor_glow = m("Cursor Glow", "cursor glow background shader hex");
    let show_blend_player_color = m(
        "Blend Player Color",
        "blend player color tint panel background colorize accent",
    );
    if show_text_size
        || show_theme_preset
        || show_panel_opacity
        || show_cursor_glow
        || show_blend_player_color
    {
        render_subcat_card(ui, "Appearance & Typography", None, accent, |_| {}, |ui| {
        if show_text_size {
        hits += 1;
        let cur_pct = (state.host.font_scale.clamp(0.5, 2.5) * 100.0).round() as i32;
        ui.row(|ui| {
            ui.txt("Text Size")
                .sz(crate::theme::fs(11.0))
                .semibold()
                .fg(crate::theme::text_primary())
                .min_w(130.0);
            for (pct, lbl) in [(75, "75%"), (100, "100%"), (125, "125%"), (150, "150%")] {
                let active = (cur_pct - pct).abs() <= 2;
                if ui
                    .btn_with(&format!("mm_set_ts_{pct}"), |ui| {
                        ui.txt(lbl)
                            .sz(crate::theme::fs(10.5))
                            .semibold()
                            .align(TextAlign::Center)
                            .fg(if active { TEXT_PRIMARY } else { TEXT_MUTED })
                            .grow(1.0);
                    })
                    .grow(1.0)
                    .py(5.0)
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
                    .bevel_rounded(5.0, 1.3)
                    .clicked()
                {
                    push_click(actions);
                    actions.push(UiAction::SetFontScale {
                        scale: pct as f32 / 100.0,
                    });
                }
            }
        })
        .align_center()
        .px(8.0)
        .py(6.0)
        .gap(6.0)
        .bg(card_inner_bg())
        .bd(1.0, border_subtle())
        .bevel_rounded(6.0, 1.3);
        }

        if show_theme_preset {
            hits += 1;
            let cur_preset = state.host.theme_preset.as_str();
            let mut row_resp = ui
                .row(|ui| {
                    ui.txt("Theme Preset")
                        .sz(crate::theme::fs(11.0))
                        .semibold()
                        .fg(crate::theme::text_primary())
                        .min_w(130.0);
                    for (preset_id, lbl) in [
                        ("default", "CVR Slate"),
                        ("oled", "OLED Black"),
                        ("frost", "Frost Glass"),
                        ("contrast", "High Contrast"),
                    ] {
                        let active = if matches!(cur_preset, "oled" | "frost" | "contrast") {
                            cur_preset == preset_id
                        } else {
                            preset_id == "default"
                        };
                        if ui
                            .btn_with(&format!("mm_set_theme_{preset_id}"), |ui| {
                                ui.txt(lbl)
                                    .sz(crate::theme::fs(10.0))
                                    .semibold()
                                    .align(TextAlign::Center)
                                    .fg(if active { TEXT_PRIMARY } else { TEXT_MUTED })
                                    .grow(1.0);
                            })
                            .grow(1.0)
                            .py(5.0)
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
                            .bevel_rounded(5.0, 1.3)
                            .clicked()
                        {
                            push_click(actions);
                            actions.push(UiAction::SetThemePreset {
                                preset: String::from(preset_id),
                            });
                        }
                    }
                })
                .align_center()
                .px(8.0)
                .py(6.0)
                .gap(5.0)
                .bg(card_inner_bg())
                .bd(1.0, border_subtle())
                .bevel_rounded(6.0, 1.3);
            if row_resp.hovered() {
                state.main_menu.hovered_tooltip = String::from(
                    "Built-in palette for menus & HUD. When Colorize Menu is ON, CVR Slate tints full panels while OLED Black, Frost Glass, and High Contrast keep clean surfaces and apply your Player Color to accents & borders.",
                );
            }
        }

        if show_panel_opacity {
            hits += 1;
            let op_pct = (state.host.panel_opacity.clamp(0.5, 1.0) * 100.0).round() as i32;
            let slot = state
                .main_menu
                .settings_sliders
                .entry(String::from("lm_panel_opacity"))
                .or_insert((op_pct as f32, op_pct));
            if slot.1 != op_pct {
                slot.0 = op_pct as f32;
                slot.1 = op_pct;
            }
            let (cur_f, last_i) = (&mut slot.0, &mut slot.1);
            let mut emit_opacity: Option<f32> = None;
            let mut row_resp = ui
                .col(|ui| {
                    let sl = crate::widgets::styled_slider(
                        ui,
                        "mm_set_lm_opacity",
                        "Panel Opacity",
                        &format!("{op_pct}%"),
                        cur_f,
                        50.0..=100.0,
                        accent,
                        true,
                        false,
                    );
                    if sl.changed {
                        let next_pct = (*cur_f).round().clamp(50.0, 100.0) as i32;
                        *cur_f = next_pct as f32;
                        if next_pct != *last_i {
                            *last_i = next_pct;
                            emit_opacity = Some(next_pct as f32 / 100.0);
                        }
                    }
                })
                .px(8.0)
                .py(6.0)
                .bg(card_inner_bg())
                .bd(1.0, border_subtle())
                .bevel_rounded(6.0, 1.3);
            if row_resp.hovered() {
                state.main_menu.hovered_tooltip = String::from(
                    "Adjust card and HUD background opacity (50% - 100%).",
                );
            }
            if let Some(next_op) = emit_opacity {
                actions.push(UiAction::SetPanelOpacity { opacity: next_op });
            }
        }

        if show_blend_player_color {
            hits += 1;
            let (clicked, hov) = compact_toggle_cell(
                ui,
                "mm_set_lm_blend_player_color",
                "Blend Player Color into Panels",
                state.host.blend_player_color,
            );
            if hov {
                state.main_menu.hovered_tooltip = String::from(
                    "Tint menu cards and HUD panels with your Player Color when Colorize Menu is enabled. Disable for pure theme backgrounds with player-colored accents.",
                );
            }
            if clicked {
                push_click(actions);
                actions.push(UiAction::SetBlendPlayerColor {
                    enabled: !state.host.blend_player_color,
                });
            }
        }

        if show_cursor_glow {
            hits += 1;
            let (clicked, hov) = compact_toggle_cell(
                ui,
                "mm_set_lm_cursor_glow",
                "Cursor Glow Shader",
                state.host.cursor_glow,
            );
            if hov {
                state.main_menu.hovered_tooltip = String::from(
                    "Render the cursor-follow background glow shader behind menu cards. Disable for lower GPU usage.",
                );
            }
            if clicked {
                push_click(actions);
                actions.push(UiAction::SetCursorGlow {
                    enabled: !state.host.cursor_glow,
                });
            }
        }
        });
    }

    // --- AUDIO ---
    let show_sound_pack = m("UI Sound Pack", "softsub glass vanilla audio");
    let show_click_sounds = m("UI Click & Hover Sounds", "sound audio feedback");
    if show_sound_pack || show_click_sounds {
        render_subcat_card(ui, "Audio", None, accent, |_| {}, |ui| {
        if show_sound_pack {
            hits += 1;
            ui.row(|ui| {
                ui.txt("UI Sound Pack")
                    .sz(crate::theme::fs(11.0))
                    .semibold()
                    .fg(TEXT_PRIMARY)
                    .min_w(130.0);
                for (pack, lbl) in [
                    ("SoftSub", "SoftSub"),
                    ("Glass", "Glass"),
                    ("Vanilla", "Vanilla"),
                ] {
                    let active = state.host.ui_sound_pack.eq_ignore_ascii_case(pack);
                    if ui
                        .btn_with(&format!("mm_set_sp_{pack}"), |ui| {
                            ui.txt(lbl)
                                .sz(crate::theme::fs(10.5))
                                .semibold()
                                .align(TextAlign::Center)
                                .fg(if active { TEXT_PRIMARY } else { TEXT_MUTED })
                                .grow(1.0);
                        })
                        .grow(1.0)
                        .py(5.0)
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
                        .bevel_rounded(5.0, 1.3)
                        .clicked()
                    {
                        actions.push(UiAction::SetUiSoundPack {
                            pack: String::from(pack),
                        });
                        push_click(actions);
                    }
                }
            })
            .align_center()
            .px(8.0)
            .py(6.0)
            .gap(6.0)
            .bg(card_inner_bg())
            .bd(1.0, border_subtle())
            .bevel_rounded(6.0, 1.3);
        }

        if show_click_sounds {
            hits += 1;
            let (clicked, hov) = compact_toggle_cell(
                ui,
                "mm_set_lm_snd",
                "UI Click & Hover Sounds",
                state.host.ui_click_sounds,
            );
            if hov {
                state.main_menu.hovered_tooltip = String::from(
                    "Enable procedural hover and click audio feedback across Lumenic menus.",
                );
            }
            if clicked {
                push_click(actions);
                actions.push(UiAction::ToggleUiClickSounds);
            }
        }
        });
    }

    // --- PERFORMANCE & INPUT ---
    let show_hz = m("Max Redraw Refresh Rate", "hz fps performance");
    let show_desk_scroll = m("Desktop Scroll Speed", "mouse wheel scroll multiplier");
    let show_vr_scroll = m("VR Scroll Speed", "thumbstick stick scroll multiplier");
    if show_hz || show_desk_scroll || show_vr_scroll {
        render_subcat_card(ui, "Performance & Input", None, accent, |_| {}, |ui| {

        if show_hz {
            hits += 1;
            let hz_host = state.host.max_refresh_hz.clamp(15, 144) as i32;
            let (hz_f, last_hz) = state
                .main_menu
                .settings_sliders
                .entry(String::from("_lumenic_hz"))
                .or_insert((hz_host as f32, hz_host));
            if *last_hz != hz_host {
                *hz_f = hz_host as f32;
                *last_hz = hz_host;
            }
            let mut hz_v = *hz_f;
            let hz_txt = format!("{} Hz", hz_v.round() as i32);
            if crate::widgets::styled_slider(
                ui,
                "mm_set_lm_hz",
                "Max Redraw Rate",
                &hz_txt,
                &mut hz_v,
                15.0..=144.0,
                accent,
                true,
                false,
            )
            .changed
            {
                let next = (hz_v.round() as i32).clamp(15, 144);
                *hz_f = next as f32;
                if next != *last_hz {
                    *last_hz = next;
                    actions.push(UiAction::SetMaxRefreshHz { hz: next });
                }
            }
        }

        if show_desk_scroll || show_vr_scroll {
            ui.row(|ui| {
                if show_desk_scroll {
                    hits += 1;
                    let host_v = (state.host.desktop_scroll_multiplier.clamp(4.0, 64.0) * 10.0)
                        .round() as i32;
                    let (cur_f, last_i) = state
                        .main_menu
                        .settings_sliders
                        .entry(String::from("_lumenic_desk_scroll"))
                        .or_insert((host_v as f32 / 10.0, host_v));
                    if *last_i != host_v {
                        *cur_f = host_v as f32 / 10.0;
                        *last_i = host_v;
                    }
                    let mut v = *cur_f;
                    let val_txt = format!("{:.0}", v.round());
                    if crate::widgets::styled_slider(
                        ui,
                        "mm_set_lm_dscroll",
                        "Desktop Scroll",
                        &val_txt,
                        &mut v,
                        4.0..=64.0,
                        accent,
                        true,
                        false,
                    )
                    .changed
                    {
                        let next_i = (v.round() as i32 * 10).clamp(40, 640);
                        *cur_f = next_i as f32 / 10.0;
                        if next_i != *last_i {
                            *last_i = next_i;
                            actions.push(UiAction::SetDesktopScrollMultiplier { value: *cur_f });
                        }
                    }
                }

                if show_vr_scroll {
                    hits += 1;
                    let host_v = (state.host.vr_scroll_multiplier.clamp(0.5, 12.0) * 10.0)
                        .round() as i32;
                    let (cur_f, last_i) = state
                        .main_menu
                        .settings_sliders
                        .entry(String::from("_lumenic_vr_scroll"))
                        .or_insert((host_v as f32 / 10.0, host_v));
                    if *last_i != host_v {
                        *cur_f = host_v as f32 / 10.0;
                        *last_i = host_v;
                    }
                    let mut v = *cur_f;
                    let val_txt = format!("{v:.1}");
                    if crate::widgets::styled_slider(
                        ui,
                        "mm_set_lm_vscroll",
                        "VR Scroll",
                        &val_txt,
                        &mut v,
                        0.5..=12.0,
                        accent,
                        true,
                        false,
                    )
                    .changed
                    {
                        let next_i = (v * 10.0).round().clamp(5.0, 120.0) as i32;
                        *cur_f = next_i as f32 / 10.0;
                        if next_i != *last_i {
                            *last_i = next_i;
                            actions.push(UiAction::SetVrScrollMultiplier { value: *cur_f });
                        }
                    }
                }
            })
            .gap(6.0);
        }
        });
    }

    // --- HUD LAYOUT (DESKTOP / VR AWARE) ---
    let is_vr = state.host.is_vr;
    let show_hud_spread = m("HUD Horizontal Spread", "hud horizontal spread width inward outward vr desktop");
    let show_hud_spread_y = m("HUD Vertical Spread", "hud vertical spread height inward outward vr desktop");
    let show_hud_alerts = m("HUD Alerts & Icons Size", "hud alerts small icons badges status size vr desktop");
    let show_hud_mic = m("HUD Mic Icon Size", "hud mic microphone icon size vr desktop");
    let show_hud_notifs = m("HUD Notifications Size", "hud notifications toasts join feed size vr desktop");
    let show_hud_chat = m("HUD ChatBox Size", "hud chatbox portal size vr desktop");
    if show_hud_spread
        || show_hud_spread_y
        || show_hud_alerts
        || show_hud_mic
        || show_hud_notifs
        || show_hud_chat
    {
        let mode_label = if is_vr { "VR" } else { "Desktop" };
        let hud_title = format!("HUD Layout ({mode_label})");
        let mut reset_all_clicked = false;
        render_subcat_card(
            ui,
            &hud_title,
            None,
            accent,
            |ui| {
                ui.push_id("mm_set_lm_hud_reset_all");
                if ui
                    .btn("Reset All")
                    .sz(crate::theme::fs(10.0))
                    .px(8.0)
                    .py(3.0)
                    .bg(card_inner_bg())
                    .bd(1.0, border_subtle())
                    .bevel_rounded(5.0, 1.2)
                    .clicked()
                {
                    reset_all_clicked = true;
                }
                ui.pop_id();
            },
            |ui| {

            let (raw_spread, raw_spread_y, raw_alerts, raw_mic, raw_notifs, raw_chat) = if is_vr {
                (
                    state.host.hud_spread_vr.clamp(0.5, 1.35),
                    state.host.hud_spread_y_vr.clamp(0.5, 1.35),
                    state.host.hud_alerts_scale_vr.clamp(0.5, 2.0),
                    state.host.hud_mic_scale_vr.clamp(0.5, 2.0),
                    state.host.hud_notifications_scale_vr.clamp(0.5, 2.0),
                    state.host.hud_chatbox_scale_vr.clamp(0.5, 2.0),
                )
            } else {
                (
                    state.host.hud_spread_desktop.clamp(0.5, 1.35),
                    state.host.hud_spread_y_desktop.clamp(0.5, 1.35),
                    state.host.hud_alerts_scale_desktop.clamp(0.5, 2.0),
                    state.host.hud_mic_scale_desktop.clamp(0.5, 2.0),
                    state.host.hud_notifications_scale_desktop.clamp(0.5, 2.0),
                    state.host.hud_chatbox_scale_desktop.clamp(0.5, 2.0),
                )
            };

            let def_spread = if is_vr { 0.85f32 } else { 1.0f32 };
            let hud_specs = [
                (show_hud_spread, "mm_set_lm_hud_spread", format!("_lumenic_hud_spread_{is_vr}"), "Horizontal Spread", raw_spread, def_spread, 0.50f32, 1.35f32, 1u8),
                (show_hud_spread_y, "mm_set_lm_hud_spready", format!("_lumenic_hud_spready_{is_vr}"), "Vertical Spread", raw_spread_y, def_spread, 0.50f32, 1.35f32, 2u8),
                (show_hud_alerts, "mm_set_lm_hud_alerts", format!("_lumenic_hud_alerts_{is_vr}"), "Alerts & Icons Size", raw_alerts, 1.0f32, 0.50f32, 2.00f32, 3u8),
                (show_hud_mic, "mm_set_lm_hud_mic", format!("_lumenic_hud_mic_{is_vr}"), "Mic Icon Size", raw_mic, 1.0f32, 0.50f32, 2.00f32, 4u8),
                (show_hud_notifs, "mm_set_lm_hud_notifs", format!("_lumenic_hud_notifs_{is_vr}"), "Notifications Size", raw_notifs, 1.0f32, 0.50f32, 2.00f32, 5u8),
                (show_hud_chat, "mm_set_lm_hud_chat", format!("_lumenic_hud_chat_{is_vr}"), "ChatBox Size", raw_chat, 1.0f32, 0.50f32, 2.00f32, 6u8),
            ];

            for (visible, push_id, state_key, title, host_val, def_val, min_v, max_v, kind) in hud_specs {
                if !visible {
                    continue;
                }
                hits += 1;
                let host_pct = (host_val * 100.0).round() as i32;
                let def_pct = (def_val * 100.0).round() as i32;
                let (cur_f, last_i) = state
                    .main_menu
                    .settings_sliders
                    .entry(state_key)
                    .or_insert((host_pct as f32, host_pct));
                if *last_i != host_pct {
                    *cur_f = host_pct as f32;
                    *last_i = host_pct;
                }
                let mut v = *cur_f;
                let min_pct = (min_v * 100.0).round();
                let max_pct = (max_v * 100.0).round();
                let val_txt = format!("{}%", v.round() as i32);
                let mut emit_val: Option<f32> = None;
                ui.row(|ui| {
                    if crate::widgets::styled_slider(
                        ui,
                        push_id,
                        title,
                        &val_txt,
                        &mut v,
                        min_pct..=max_pct,
                        accent,
                        true,
                        false,
                    )
                    .changed
                    {
                        let next_pct = (v.round() as i32).clamp(min_pct as i32, max_pct as i32);
                        *cur_f = next_pct as f32;
                        if next_pct != *last_i {
                            *last_i = next_pct;
                            emit_val = Some(next_pct as f32 / 100.0);
                        }
                    }

                    ui.push_id(&format!("{push_id}_rst"));
                    if ui
                        .btn("Reset")
                        .sz(crate::theme::fs(9.5).min(12.5))
                        .px(7.0)
                        .py(4.0)
                        .bg(card_inner_bg())
                        .bd(1.0, border_subtle())
                        .bevel_rounded(5.0, 1.2)
                        .clicked()
                    {
                        *cur_f = def_pct as f32;
                        *last_i = def_pct;
                        emit_val = Some(def_val);
                    }
                    ui.pop_id();
                })
                .align_center()
                .gap(5.0);

                if let Some(next_val) = emit_val {
                    match kind {
                        1 => actions.push(UiAction::SetHudSpread { is_vr, value: next_val }),
                        2 => actions.push(UiAction::SetHudSpreadY { is_vr, value: next_val }),
                        3 => actions.push(UiAction::SetHudElementScale { is_vr, element: String::from("alerts"), value: next_val }),
                        4 => actions.push(UiAction::SetHudElementScale { is_vr, element: String::from("mic"), value: next_val }),
                        5 => actions.push(UiAction::SetHudElementScale { is_vr, element: String::from("notifications"), value: next_val }),
                        _ => actions.push(UiAction::SetHudElementScale { is_vr, element: String::from("chatbox"), value: next_val }),
                    }
                }
            }
            },
        );
        if reset_all_clicked {
            actions.push(UiAction::ResetHudLayout { is_vr });
        }
    }

    // --- LOCAL HISTORY & USER DATA ---
    let show_qm_icon_tabs = m(
        "Quick Menu Mod Tabs Icons Only",
        "quick menu qm mod tabs icon only compact tooltip",
    );
    let show_qm_preset_confirm = m(
        "Quick Menu Outfit Preset Confirm",
        "quick menu qm outfit preset confirm preview auto apply sync",
    );
    let show_recent = m(
        "Track Recent History",
        "recent categories history userdata visited seen spawned",
    );
    let show_clear = m("Clear User Data", "clear reset history pinned pins userdata layouts");
    if show_qm_icon_tabs || show_qm_preset_confirm || show_recent || show_clear {
        render_subcat_card(ui, "Quick Menu & User Data", None, accent, |_| {}, |ui| {

        if show_qm_preset_confirm {
            hits += 1;
            let cur_mode = state.host.qm_preset_confirm_mode.as_str();
            let mut row_resp = ui
                .row(|ui| {
                    ui.txt("QM Outfit Preset Confirm")
                        .sz(crate::theme::fs(11.0))
                        .semibold()
                        .fg(crate::theme::text_primary())
                        .min_w(140.0);
                    for (mode_id, lbl) in [
                        ("sync", "Sync Main"),
                        ("on", "Force On"),
                        ("off", "Force Off"),
                    ] {
                        let active = if cur_mode == "on" || cur_mode == "off" {
                            cur_mode == mode_id
                        } else {
                            mode_id == "sync"
                        };
                        if ui
                            .btn_with(&format!("mm_set_lm_qm_preset_{mode_id}"), |ui| {
                                ui.txt(lbl)
                                    .sz(crate::theme::fs(10.0))
                                    .semibold()
                                    .align(TextAlign::Center)
                                    .fg(if active { TEXT_PRIMARY } else { TEXT_MUTED })
                                    .grow(1.0);
                            })
                            .grow(1.0)
                            .py(5.0)
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
                            .bevel_rounded(5.0, 1.3)
                            .clicked()
                        {
                            push_click(actions);
                            actions.push(UiAction::SetQmPresetConfirmMode {
                                mode: String::from(mode_id),
                            });
                        }
                    }
                })
                .align_center()
                .px(8.0)
                .py(6.0)
                .gap(5.0)
                .bg(card_inner_bg())
                .bd(1.0, border_subtle())
                .bevel_rounded(6.0, 1.3);
            if row_resp.hovered() {
                state.main_menu.hovered_tooltip = String::from(
                    "Sync Main: follows Main Menu's Auto Apply Changes setting. Force On: always preview in 3D and require Confirm. Force Off: apply outfit presets immediately.",
                );
            }
        }

        if show_qm_icon_tabs {
            hits += 1;
            let (clicked, hov) = compact_toggle_cell(
                ui,
                "mm_set_lm_qm_icon_tabs",
                "Quick Menu Mod Tabs: Icons Only",
                state.host.qm_mod_tabs_icon_only,
            );
            if hov {
                state.main_menu.hovered_tooltip = String::from(
                    "Show only icons for added mod tabs in the Quick Menu tab bar, with the mod name shown on hover.",
                );
            }
            if clicked {
                push_click(actions);
                actions.push(UiAction::SetQmModTabsIconOnly {
                    enabled: !state.host.qm_mod_tabs_icon_only,
                });
            }
        }

        if show_recent {
            hits += 1;
            let (clicked, hov) = compact_toggle_cell(
                ui,
                "mm_set_lm_recent",
                "Track Recent Categories (UserData)",
                state.host.enable_recent_history,
            );
            if hov {
                state.main_menu.hovered_tooltip = String::from(
                    "Record and show local Recent categories (Worlds, Avatars, Props, Users) in UserData/LumenicMenu/recent-history.json.",
                );
            }
            if clicked {
                push_click(actions);
                actions.push(UiAction::SetEnableRecentHistory {
                    enabled: !state.host.enable_recent_history,
                });
            }
        }

        if show_clear {
            hits += 3;
            ui.row(|ui| {
                let clear_hist_armed =
                    state.main_menu.safety_confirm_action == "clear_recent_history";
                if ui
                    .btn_with("mm_set_lm_clr_hist", |ui| {
                        ui.txt(if clear_hist_armed {
                            "Confirm Clear History?"
                        } else {
                            "Clear Recent History"
                        })
                        .sz(crate::theme::fs(10.5))
                        .semibold()
                        .align(TextAlign::Center)
                        .fg(if clear_hist_armed {
                            TEXT_PRIMARY
                        } else {
                            crate::theme::AMBER
                        })
                        .grow(1.0);
                    })
                    .grow(1.0)
                    .py(7.0)
                    .bg(if clear_hist_armed {
                        tint_alpha(crate::theme::AMBER, 75)
                    } else {
                        tile_idle_bg()
                    })
                    .hover_bg(tint_alpha(crate::theme::AMBER, 95))
                    .bd(1.0, tint_alpha(crate::theme::AMBER, 165))
                    .bevel_rounded(6.0, 1.3)
                    .clicked()
                {
                    push_click(actions);
                    if clear_hist_armed {
                        state.main_menu.safety_confirm_action.clear();
                        actions.push(UiAction::ClearRecentHistory);
                    } else {
                        state.main_menu.safety_confirm_action =
                            String::from("clear_recent_history");
                    }
                }

                let clear_pins_armed = state.main_menu.safety_confirm_action == "clear_pins";
                if ui
                    .btn_with("mm_set_lm_clr_pins", |ui| {
                        ui.txt(if clear_pins_armed {
                            "Confirm Clear Pins?"
                        } else {
                            "Clear Pinned Items"
                        })
                        .sz(crate::theme::fs(10.5))
                        .semibold()
                        .align(TextAlign::Center)
                        .fg(if clear_pins_armed {
                            TEXT_PRIMARY
                        } else {
                            crate::theme::RED
                        })
                        .grow(1.0);
                    })
                    .grow(1.0)
                    .py(7.0)
                    .bg(if clear_pins_armed {
                        tint_alpha(crate::theme::RED, 75)
                    } else {
                        tile_idle_bg()
                    })
                    .hover_bg(tint_alpha(crate::theme::RED, 95))
                    .bd(1.0, tint_alpha(crate::theme::RED, 165))
                    .bevel_rounded(6.0, 1.3)
                    .clicked()
                {
                    push_click(actions);
                    if clear_pins_armed {
                        state.main_menu.safety_confirm_action.clear();
                        actions.push(UiAction::ClearPinnedItems);
                    } else {
                        state.main_menu.safety_confirm_action = String::from("clear_pins");
                    }
                }

                let reset_tiles_armed = state.main_menu.safety_confirm_action == "reset_tiles";
                if ui
                    .btn_with("mm_set_lm_rst_tiles", |ui| {
                        ui.txt(if reset_tiles_armed {
                            "Confirm Reset Tiles?"
                        } else {
                            "Reset Tile Layouts"
                        })
                        .sz(crate::theme::fs(10.5))
                        .semibold()
                        .align(TextAlign::Center)
                        .fg(TEXT_PRIMARY)
                        .grow(1.0);
                    })
                    .grow(1.0)
                    .py(7.0)
                    .bg(if reset_tiles_armed {
                        tint_alpha(accent, 65)
                    } else {
                        tile_idle_bg()
                    })
                    .hover_bg(tile_idle_hover())
                    .bd(1.0, border_subtle())
                    .bevel_rounded(6.0, 1.3)
                    .clicked()
                {
                    push_click(actions);
                    if reset_tiles_armed {
                        state.main_menu.safety_confirm_action.clear();
                        actions.push(UiAction::ResetTileLayouts);
                    } else {
                        state.main_menu.safety_confirm_action = String::from("reset_tiles");
                    }
                }
            })
            .gap(6.0);
        }
        });
    }

    if q.is_empty() {
        crate::ext_view::render_slot_only(ui, state, "main", "mm.settings.lumenic", accent, actions);
    }

    hits
}

fn render_player_colors_section(
    ui: &mut UiContext,
    state: &mut AppState,
    accent: Color,
    actions: &mut Vec<UiAction>,
) {
    let p1 = Color::rgb(state.host.accent_r, state.host.accent_g, state.host.accent_b);
    let p2 = Color::rgb(
        state.host.accent2_r,
        state.host.accent2_g,
        state.host.accent2_b,
    );
    let colorize = state.host.colorize_menu;
    let emission = state.host.use_primary_as_emission;

    ui.row(|ui| {
        ui.txt("Player Colors & Menu Theme")
            .sz(crate::theme::fs(12.5))
            .bold()
            .fg(accent)
            .grow(1.0);

        ui.row(|ui| {
            ui.txt("").w(14.0).h(14.0).bg(p1).bd(1.0, TEXT_PRIMARY).r(3.0);
            ui.txt("Primary").sz(crate::theme::fs(10.0)).fg(TEXT_MUTED);
            ui.txt("").w(14.0).h(14.0).bg(p2).bd(1.0, TEXT_PRIMARY).r(3.0);
            ui.txt("Secondary").sz(crate::theme::fs(10.0)).fg(TEXT_MUTED);
        })
        .align_center()
        .gap(5.0)
        .px(8.0)
        .py(3.5)
        .bg(card_inner_bg())
        .bd(1.0, border_subtle())
        .bevel_rounded(5.5, 1.3);
    })
    .align_center()
    .gap(8.0)
    .px(3.0);

    ui.row(|ui| {
        let (clk_col, hov_col) =
            compact_toggle_cell(ui, "mm_pc_colorize", "Colorize Menu", colorize);
        if clk_col {
            push_click(actions);
            actions.push(UiAction::SetColorizeMenu { enabled: !colorize });
        }
        if hov_col {
            state.main_menu.hovered_tooltip = String::from(
                "ON: Secondary color is accent and Primary subtly tints panels. OFF: Official ChilloutVR dark orange theme.",
            );
        }

        let (clk_em, hov_em) =
            compact_toggle_cell(ui, "mm_pc_emission", "Use Primary as Emission", emission);
        if clk_em {
            push_click(actions);
            actions.push(UiAction::SetPlayerColorEmission { enabled: !emission });
        }
        if hov_em {
            state.main_menu.hovered_tooltip =
                String::from("Use your Primary player color as the emission color on player indicators.");
        }
    })
    .align_center()
    .gap(6.0);

    let presets = state.host.player_color_presets.clone();
    if !presets.is_empty() {
        ui.txt("Presets")
            .sz(crate::theme::fs(11.0))
            .bold()
            .fg(TEXT_MUTED)
            .pt(4.0)
            .px(3.0);

        for row in presets.chunks(4) {
            ui.row(|ui| {
                for pr in row {
                    let c1 = Color::rgb(pr.r1, pr.g1, pr.b1);
                    let c2 = Color::rgb(pr.r2, pr.g2, pr.b2);
                    let selected = pr.r1 == state.host.accent_r
                        && pr.g1 == state.host.accent_g
                        && pr.b1 == state.host.accent_b
                        && pr.r2 == state.host.accent2_r
                        && pr.g2 == state.host.accent2_g
                        && pr.b2 == state.host.accent2_b;
                    if ui
                        .btn_with(&format!("mm_pc_preset_{}", pr.id), |ui| {
                            ui.row(|ui| {
                                ui.row(|ui| {
                                    ui.txt("").w(12.0).h(18.0).bg(c1).r(3.0);
                                    ui.txt("").w(12.0).h(18.0).bg(c2).r(3.0);
                                })
                                .gap(2.0);
                                ui.txt(&pr.name)
                                    .sz(crate::theme::fs(10.5))
                                    .semibold()
                                    .fg(TEXT_PRIMARY)
                                    .ellipsis()
                                    .grow(1.0);
                            })
                            .align_center()
                            .gap(6.0)
                            .grow(1.0);
                        })
                        .grow(1.0)
                        .px(7.0)
                        .py(6.0)
                        .bg(if selected {
                            tint_alpha(accent, 45)
                        } else {
                            card_inner_bg()
                        })
                        .hover_bg(tile_idle_hover())
                        .bd(
                            if selected { 1.5 } else { 1.0 },
                            if selected { accent } else { border_subtle() },
                        )
                        .bevel_rounded(6.0, 1.3)
                        .clicked()
                    {
                        push_click(actions);
                        actions.push(UiAction::SetPlayerColorPreset { preset_id: pr.id });
                    }
                }
                for _ in row.len()..4 {
                    ui.col(|_| {}).grow(1.0);
                }
            })
            .gap(6.0);
        }
    }

    let swatches = state.host.player_color_swatches.clone();
    if !swatches.is_empty() {
        for (is_primary, heading, prefix, cur_r, cur_g, cur_b) in [
            (
                true,
                "Primary Color (Menu Panel Tint)",
                "mm_pc_sw1",
                state.host.accent_r,
                state.host.accent_g,
                state.host.accent_b,
            ),
            (
                false,
                "Secondary Color (Menu Accent)",
                "mm_pc_sw2",
                state.host.accent2_r,
                state.host.accent2_g,
                state.host.accent2_b,
            ),
        ] {
            ui.txt(heading)
                .sz(crate::theme::fs(11.0))
                .bold()
                .fg(TEXT_MUTED)
                .pt(5.0)
                .px(3.0);

            for row in swatches.chunks(11) {
                ui.row(|ui| {
                    for sw in row {
                        let col = Color::rgb(sw.r, sw.g, sw.b);
                        let active = sw.r == cur_r && sw.g == cur_g && sw.b == cur_b;
                        let resp = ui
                            .btn_with(&format!("{prefix}_{}", sw.id), |ui| {
                                ui.txt("").h(22.0);
                            })
                            .grow(1.0)
                            .h(26.0)
                            .bg(col)
                            .hover_bg(col)
                            .bd(
                                if active { 2.2 } else { 1.0 },
                                if active {
                                    TEXT_PRIMARY
                                } else {
                                    Color::rgba(255, 255, 255, 65)
                                },
                            )
                            .bevel_rounded(5.5, 1.2);
                        if resp.hovered() {
                            state.main_menu.hovered_tooltip = sw.name.clone();
                        }
                        if resp.clicked() {
                            push_click(actions);
                            if is_primary {
                                actions.push(UiAction::SetPlayerColorPrimary { swatch_id: sw.id });
                            } else {
                                actions.push(UiAction::SetPlayerColorSecondary { swatch_id: sw.id });
                            }
                        }
                    }
                    for _ in row.len()..11 {
                        ui.col(|_| {}).grow(1.0);
                    }
                })
                .gap(5.0);
            }
        }
    }
}

fn render_about_lumenic_section(
    ui: &mut UiContext,
    state: &mut AppState,
    accent: Color,
    actions: &mut Vec<UiAction>,
) {
    // --- LUMENICMENU MOD SUPPORT & UPDATES ---
    ui.col(|ui| {
        ui.row(|ui| {
            icons::icon(ui, icons::BOLT, 14.5).fg(accent);
            ui.txt("About LumenicMenu & Mod Support")
                .sz(crate::theme::fs(12.5))
                .bold()
                .fg(TEXT_PRIMARY);
        })
        .align_center()
        .gap(6.0);

        ui.txt("LumenicMenu is an unofficial community mod and is not affiliated with ChilloutVR or the ChilloutVR team. If you encounter any mod issues or bugs, or want to make a feature request, please open an issue or pull request on GitHub, or DM @LensError on Discord.")
            .sz(crate::theme::fs(11.0))
            .fg(TEXT_MUTED)
            .wrap();

        ui.row(|ui| {
            if ui
                .btn_with("mm_sup_lm_gh", |ui| {
                    ui.row(|ui| {
                        icons::icon(ui, icons::WORLD, 13.0).fg(accent);
                        ui.txt("GitHub: LensError/Lumenic-CVR-Menu")
                            .sz(crate::theme::fs(11.0))
                            .semibold()
                            .fg(TEXT_PRIMARY);
                    })
                    .center()
                    .gap(5.0)
                    .grow(1.0);
                })
                .grow(1.0)
                .py(7.5)
                .bg(tint_alpha(accent, 42))
                .hover_bg(tint_alpha(accent, 75))
                .bd(1.0, tint_alpha(accent, 175))
                .bevel_rounded(6.5, 1.3)
                .clicked()
            {
                push_click(actions);
                actions.push(UiAction::OpenExternalUrl {
                    url: String::from("https://github.com/LensError/Lumenic-CVR-Menu"),
                });
            }

            if ui
                .btn_with("mm_sup_lm_issues", |ui| {
                    ui.row(|ui| {
                        icons::icon(ui, icons::MAIL, 13.0).fg(crate::theme::AMBER);
                        ui.txt("Report Mod Issue / Feature Request")
                            .sz(crate::theme::fs(11.0))
                            .semibold()
                            .fg(TEXT_PRIMARY);
                    })
                    .center()
                    .gap(5.0)
                    .grow(1.0);
                })
                .grow(1.0)
                .py(7.5)
                .bg(tile_idle_bg())
                .hover_bg(tile_idle_hover())
                .bd(1.0, border_subtle())
                .bevel_rounded(6.5, 1.3)
                .clicked()
            {
                push_click(actions);
                actions.push(UiAction::OpenExternalUrl {
                    url: String::from("https://github.com/LensError/Lumenic-CVR-Menu/issues"),
                });
            }
        })
        .gap(7.0);

        // --- MOD UPDATES ---
        let upd = state.host.update_status.clone();
        let cur_v = if upd.current_version.is_empty() {
            env!("CARGO_PKG_VERSION")
        } else {
            upd.current_version.as_str()
        };
        let status_label = match upd.status.as_str() {
            "checking" => String::from("Checking GitHub Releases..."),
            "up_to_date" => format!("Up to date (v{cur_v})"),
            "update_available" => format!("Update available: v{} (installed v{cur_v})", upd.latest_version),
            "error" => {
                if upd.error_message.is_empty() {
                    String::from("Update check failed")
                } else {
                    format!("Check failed: {}", upd.error_message)
                }
            }
            _ => format!("Installed v{cur_v}"),
        };
        let status_color = match upd.status.as_str() {
            "update_available" => crate::theme::EMERALD,
            "error" => crate::theme::AMBER,
            _ => TEXT_MUTED,
        };

        ui.row(|ui| {
            ui.col(|ui| {
                ui.txt("Mod Updates")
                    .sz(crate::theme::fs(11.5))
                    .semibold()
                    .fg(TEXT_PRIMARY);
                ui.txt(&status_label)
                    .sz(crate::theme::fs(10.5))
                    .fg(status_color)
                    .wrap();
            })
            .grow(1.0)
            .gap(2.0);

            if ui
                .btn_with("mm_sup_whats_new", |ui| {
                    ui.txt("What's New")
                        .sz(crate::theme::fs(10.5))
                        .semibold()
                        .fg(TEXT_PRIMARY);
                })
                .px(10.0)
                .py(7.0)
                .bg(tile_idle_bg())
                .hover_bg(tile_idle_hover())
                .bd(1.0, border_subtle())
                .bevel_rounded(6.5, 1.3)
                .clicked()
            {
                push_click(actions);
                state.host.show_whats_new = true;
            }

            if upd.status == "update_available"
                && ui
                    .btn_with("mm_sup_view_upd", |ui| {
                        ui.txt("View Update")
                            .sz(crate::theme::fs(10.5))
                            .bold()
                            .fg(TEXT_PRIMARY);
                    })
                    .px(10.0)
                    .py(7.0)
                    .bg(tint_alpha(crate::theme::EMERALD, 50))
                    .hover_bg(tint_alpha(crate::theme::EMERALD, 85))
                    .bd(1.0, crate::theme::EMERALD)
                    .bevel_rounded(6.5, 1.3)
                    .clicked()
            {
                push_click(actions);
                state.host.update_status.show_modal = true;
            }

            if ui
                .btn_with("mm_sup_check_upd", |ui| {
                    ui.txt(if upd.status == "checking" {
                        "Checking..."
                    } else {
                        "Check for Updates"
                    })
                    .sz(crate::theme::fs(10.5))
                    .semibold()
                    .fg(TEXT_PRIMARY);
                })
                .px(10.0)
                .py(7.0)
                .bg(tint_alpha(accent, 42))
                .hover_bg(tint_alpha(accent, 75))
                .bd(1.0, tint_alpha(accent, 175))
                .bevel_rounded(6.5, 1.3)
                .clicked()
            {
                push_click(actions);
                state.host.update_status.status = String::from("checking");
                actions.push(UiAction::CheckForUpdates);
            }
        })
        .align_center()
        .gap(6.0);
    })
    .p(11.0)
    .gap(7.0)
    .bg(card_inner_bg())
    .bd(1.0, border_subtle())
    .bevel_rounded(8.5, 1.5);

    // --- MODDED CLIENT DISCLAIMER ---
    ui.col(|ui| {
        ui.row(|ui| {
            icons::icon(ui, icons::BELL, 15.0).fg(crate::theme::AMBER);
            ui.txt("IMPORTANT - Read Before Reporting Bugs to ChilloutVR")
                .sz(crate::theme::fs(12.0))
                .bold()
                .fg(crate::theme::AMBER);
        })
        .align_center()
        .gap(6.0);

        ui.txt("If you notice ANY bug even remotely related to the UI, menus, input, or rendering, you MUST completely UNINSTALL LumenicMenu (remove LumenicMenu.dll from Mods/ and restart the game - do NOT just toggle it off) and reproduce the issue on a clean vanilla client before reporting anything to ChilloutVR or the CVR team.")
            .sz(crate::theme::fs(11.0))
            .fg(TEXT_PRIMARY)
            .wrap();
    })
    .p(11.0)
    .gap(6.0)
    .bg(tint_alpha(crate::theme::AMBER, 26))
    .bd(1.5, tint_alpha(crate::theme::AMBER, 175))
    .bevel_rounded(8.5, 1.5);
}

fn render_support_section(
    ui: &mut UiContext,
    state: &mut AppState,
    accent: Color,
    actions: &mut Vec<UiAction>,
) {
    // --- BIG MODDED CLIENT DISCLAIMER ---
    ui.col(|ui| {
        ui.row(|ui| {
            icons::icon(ui, icons::BELL, 15.0).fg(crate::theme::AMBER);
            ui.txt("IMPORTANT - Read Before Reporting Bugs to ChilloutVR")
                .sz(crate::theme::fs(12.0))
                .bold()
                .fg(crate::theme::AMBER);
        })
        .align_center()
        .gap(6.0);

        ui.txt("If you notice ANY bug even remotely related to the UI, menus, input, or rendering, you MUST completely UNINSTALL LumenicMenu (remove LumenicMenu.dll from Mods/ and restart the game - do NOT just toggle it off) and reproduce the issue on a clean vanilla client before reporting anything to ChilloutVR or the CVR team.")
            .sz(crate::theme::fs(11.0))
            .fg(TEXT_PRIMARY)
            .wrap();
    })
    .p(11.0)
    .gap(6.0)
    .bg(tint_alpha(crate::theme::AMBER, 26))
    .bd(1.5, tint_alpha(crate::theme::AMBER, 175))
    .bevel_rounded(8.5, 1.5);

    // --- OFFICIAL CHILLOUTVR SUPPORT ---
    ui.col(|ui| {
        ui.txt("Thank you for Playing ChilloutVR!")
            .sz(crate::theme::fs(13.0))
            .bold()
            .fg(TEXT_PRIMARY);
        ui.txt("To get general help or support from the ChilloutVR team, use the official channels below:")
            .sz(crate::theme::fs(11.0))
            .fg(TEXT_MUTED)
            .wrap();

        // 2x2 Contact Grid
        ui.row(|ui| {
            if ui
                .btn_with("mm_sup_cvr_discord", |ui| {
                    ui.row(|ui| {
                        icons::icon(ui, icons::PLAYERS, 13.5).fg(accent);
                        ui.txt("discord.gg/ChilloutVR")
                            .sz(crate::theme::fs(11.0))
                            .semibold()
                            .fg(TEXT_PRIMARY);
                    })
                    .center()
                    .gap(6.0)
                    .grow(1.0);
                })
                .grow(1.0)
                .py(8.0)
                .bg(tile_idle_bg())
                .hover_bg(tile_idle_hover())
                .bd(1.0, border_subtle())
                .bevel_rounded(6.5, 1.3)
                .clicked()
            {
                push_click(actions);
                actions.push(UiAction::OpenHardcodedUrl {
                    key: String::from("Discord"),
                });
            }

            let email_resp = ui
                .btn_with("mm_sup_cvr_email", |ui| {
                    ui.row(|ui| {
                        icons::icon(ui, icons::MAIL, 13.5).fg(TEXT_MUTED);
                        ui.txt("team@chilloutvr.net")
                            .sz(crate::theme::fs(11.0))
                            .semibold()
                            .fg(TEXT_PRIMARY);
                    })
                    .center()
                    .gap(6.0)
                    .grow(1.0);
                })
                .grow(1.0)
                .py(8.0)
                .bg(tile_idle_bg())
                .hover_bg(tile_idle_hover())
                .bd(1.0, border_subtle())
                .bevel_rounded(6.5, 1.3);
            if email_resp.hovered() {
                state.main_menu.hovered_tooltip =
                    String::from("Click to email team@chilloutvr.net.");
            }
            if email_resp.clicked() {
                push_click(actions);
                actions.push(UiAction::CopyText {
                    text: String::from("team@chilloutvr.net"),
                });
            }
        })
        .gap(7.0);

        ui.row(|ui| {
            if ui
                .btn_with("mm_sup_cvr_patreon", |ui| {
                    ui.row(|ui| {
                        icons::icon(ui, icons::STAR, 13.5).fg(crate::theme::AMBER);
                        ui.txt("ChilloutVR Patreon")
                            .sz(crate::theme::fs(11.0))
                            .semibold()
                            .fg(TEXT_PRIMARY);
                    })
                    .center()
                    .gap(6.0)
                    .grow(1.0);
                })
                .grow(1.0)
                .py(8.0)
                .bg(tile_idle_bg())
                .hover_bg(tile_idle_hover())
                .bd(1.0, border_subtle())
                .bevel_rounded(6.5, 1.3)
                .clicked()
            {
                push_click(actions);
                actions.push(UiAction::OpenHardcodedUrl {
                    key: String::from("Patreon"),
                });
            }

            if ui
                .btn_with("mm_sup_cvr_twitter", |ui| {
                    ui.row(|ui| {
                        icons::icon(ui, icons::WORLD, 13.5).fg(TEXT_MUTED);
                        ui.txt("@ChilloutVR")
                            .sz(crate::theme::fs(11.0))
                            .semibold()
                            .fg(TEXT_PRIMARY);
                    })
                    .center()
                    .gap(6.0)
                    .grow(1.0);
                })
                .grow(1.0)
                .py(8.0)
                .bg(tile_idle_bg())
                .hover_bg(tile_idle_hover())
                .bd(1.0, border_subtle())
                .bevel_rounded(6.5, 1.3)
                .clicked()
            {
                push_click(actions);
                actions.push(UiAction::OpenHardcodedUrl {
                    key: String::from("Twitter"),
                });
            }
        })
        .gap(7.0);

        ui.txt("ChilloutVR Bug Report or Feature Request (Vanilla Only)")
            .sz(crate::theme::fs(11.0))
            .semibold()
            .fg(TEXT_MUTED)
            .pt(4.0);

        if ui
            .btn_with("mm_sup_cvr_gh", |ui| {
                ui.row(|ui| {
                    icons::icon(ui, icons::WORLD, 13.5).fg(TEXT_PRIMARY);
                    ui.txt("GitHub: ChilloutVR-Team/ChilloutVR")
                        .sz(crate::theme::fs(11.0))
                        .semibold()
                        .fg(TEXT_PRIMARY);
                })
                .center()
                .gap(6.0)
                .grow(1.0);
            })
            .w_frac(1.0)
            .py(8.0)
            .bg(tile_idle_bg())
            .hover_bg(tile_idle_hover())
            .bd(1.0, border_subtle())
            .bevel_rounded(6.5, 1.3)
            .clicked()
        {
            push_click(actions);
            actions.push(UiAction::OpenHardcodedUrl {
                key: String::from("Github"),
            });
        }

        // Utilities row: Open Logs Folder + Eat Bread
        ui.row(|ui| {
            if ui
                .btn_with("mm_sup_logs", |ui| {
                    ui.txt("Open Logs Folder")
                        .sz(crate::theme::fs(11.0))
                        .semibold()
                        .align(TextAlign::Center)
                        .fg(TEXT_PRIMARY)
                        .grow(1.0);
                })
                .grow(1.0)
                .py(7.5)
                .bg(tile_idle_bg())
                .hover_bg(tile_idle_hover())
                .bd(1.0, border_subtle())
                .bevel_rounded(6.5, 1.3)
                .clicked()
            {
                push_click(actions);
                actions.push(UiAction::OpenLogsFolder);
            }

            let bread_resp = ui
                .btn_with("mm_sup_bread", |ui| {
                    ui.txt("Eat Bread")
                        .sz(crate::theme::fs(11.0))
                        .semibold()
                        .align(TextAlign::Center)
                        .fg(TEXT_PRIMARY)
                        .grow(1.0);
                })
                .grow(1.0)
                .py(7.5)
                .bg(tile_idle_bg())
                .hover_bg(tile_idle_hover())
                .bd(1.0, border_subtle())
                .bevel_rounded(6.5, 1.3);
            if bread_resp.hovered() {
                state.main_menu.hovered_tooltip = String::from("nom");
            }
            if bread_resp.clicked() {
                push_click(actions);
                actions.push(UiAction::AttemptEatBread);
            }
        })
        .gap(7.0)
        .pt(2.0);
    })
    .p(11.0)
    .gap(7.0)
    .bg(card_inner_bg())
    .bd(1.0, border_subtle())
    .bevel_rounded(8.5, 1.5);
}

const LEGAL_TABS: [(&str, &str, &str); 4] = [
    (
        "tos",
        "Terms of Service",
        "https://docs.chilloutvr.net/official/legal/tos/",
    ),
    (
        "behaviour",
        "Behaviour",
        "https://docs.chilloutvr.net/official/legal/behaviour_guidelines/",
    ),
    (
        "content",
        "Content",
        "https://docs.chilloutvr.net/official/legal/content_guidelines/",
    ),
    (
        "privacy",
        "Privacy Policy",
        "https://docs.chilloutvr.net/official/legal/privacy-policy/",
    ),
];

fn render_legal_sub_tabs(
    ui: &mut UiContext,
    state: &mut AppState,
    accent: Color,
    actions: &mut Vec<UiAction>,
) {
    let active_idx = state.main_menu.legal_sub_tab.min(LEGAL_TABS.len() - 1);
    let (active_id, _, active_url) = LEGAL_TABS[active_idx];

    ui.row(|ui| {
        for (idx, (_, lbl, _)) in LEGAL_TABS.iter().enumerate() {
            let active = active_idx == idx;
            let btn_id = format!("mm_set_legal_sub_{idx}");
            if ui
                .btn_with(&btn_id, |ui| {
                    ui.txt(lbl)
                        .sz(crate::theme::fs(11.5))
                        .semibold()
                        .align(TextAlign::Center)
                        .fg(if active { TEXT_PRIMARY } else { TEXT_MUTED })
                        .ellipsis()
                        .grow(1.0);
                })
                .grow(1.0)
                .px(8.0)
                .py(7.5)
                .bg(if active {
                    tint_alpha(accent, 48)
                } else {
                    card_inner_bg()
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
                .bevel_rounded(6.5, 1.3)
                .clicked()
            {
                push_click(actions);
                state.main_menu.legal_sub_tab = idx;
            }
        }

        if ui
            .btn_with("mm_set_legal_refresh", |ui| {
                icons::icon(ui, icons::RELOAD, 13.5).fg(TEXT_PRIMARY);
            })
            .px(9.0)
            .py(7.5)
            .bg(tile_idle_bg())
            .hover_bg(tile_idle_hover())
            .bd(1.0, border_subtle())
            .bevel_rounded(6.5, 1.3)
            .clicked()
        {
            push_click(actions);
            actions.push(UiAction::FetchLegalDoc {
                doc_id: String::from(active_id),
                force: true,
            });
        }

        if ui
            .btn_with("mm_set_legal_browser", |ui| {
                ui.row(|ui| {
                    icons::icon(ui, icons::WORLD, 13.0).fg(accent);
                    ui.txt("Browser")
                        .sz(crate::theme::fs(10.5))
                        .semibold()
                        .fg(TEXT_PRIMARY);
                })
                .center()
                .gap(5.0);
            })
            .px(10.0)
            .py(7.5)
            .bg(tile_idle_bg())
            .hover_bg(tile_idle_hover())
            .bd(1.0, border_subtle())
            .bevel_rounded(6.5, 1.3)
            .clicked()
        {
            push_click(actions);
            actions.push(UiAction::OpenExternalUrl {
                url: String::from(active_url),
            });
        }
    })
    .align_center()
    .gap(6.0)
    .pb(2.0);
}

fn render_legal_section(
    ui: &mut UiContext,
    state: &mut AppState,
    accent: Color,
    actions: &mut Vec<UiAction>,
) {
    let active_idx = state.main_menu.legal_sub_tab.min(LEGAL_TABS.len() - 1);
    let (active_id, active_title, active_url) = LEGAL_TABS[active_idx];

    let doc = state
        .host
        .cvr_settings
        .legal_docs
        .iter()
        .find(|d| d.id == active_id)
        .cloned();

    let status = doc
        .as_ref()
        .map(|d| d.status.as_str())
        .unwrap_or("idle");

    if status == "idle"
        && !state
            .main_menu
            .legal_requested_docs
            .iter()
            .any(|id| id == active_id)
    {
        state
            .main_menu
            .legal_requested_docs
            .push(String::from(active_id));
        actions.push(UiAction::FetchLegalDoc {
            doc_id: String::from(active_id),
            force: false,
        });
    }

    match status {
        "ready" => {
            if let Some(d) = doc {
                ui.col(|ui| {
                    ui.row(|ui| {
                        icons::icon(ui, icons::SHIELD, 15.0).fg(accent);
                        ui.txt(&d.title)
                            .sz(crate::theme::fs(14.0))
                            .bold()
                            .fg(TEXT_PRIMARY)
                            .grow(1.0);
                        ui.txt("docs.chilloutvr.net")
                            .sz(crate::theme::fs(10.0))
                            .fg(TEXT_DIM);
                    })
                    .align_center()
                    .gap(7.0)
                    .pb(4.0);

                    for block in &d.blocks {
                        match block.kind.as_str() {
                            "h1" => {
                                ui.txt(&block.text)
                                    .sz(crate::theme::fs(15.0))
                                    .bold()
                                    .fg(TEXT_PRIMARY)
                                    .wrap()
                                    .pt(6.0);
                            }
                            "h2" => {
                                ui.txt(&block.text)
                                    .sz(crate::theme::fs(13.0))
                                    .bold()
                                    .fg(accent)
                                    .wrap()
                                    .pt(8.0);
                            }
                            "h3" => {
                                ui.txt(&block.text)
                                    .sz(crate::theme::fs(11.8))
                                    .semibold()
                                    .fg(TEXT_PRIMARY)
                                    .wrap()
                                    .pt(5.0);
                            }
                            "li" => {
                                ui.row(|ui| {
                                    ui.txt("-")
                                        .sz(crate::theme::fs(11.0))
                                        .bold()
                                        .fg(accent)
                                        .w(12.0);
                                    ui.txt(&block.text)
                                        .sz(crate::theme::fs(11.0))
                                        .fg(TEXT_MUTED)
                                        .wrap()
                                        .grow(1.0);
                                })
                                .gap(4.0)
                                .pl(6.0);
                            }
                            _ => {
                                ui.txt(&block.text)
                                    .sz(crate::theme::fs(11.0))
                                    .fg(TEXT_PRIMARY)
                                    .wrap();
                            }
                        }
                    }
                })
                .p(13.0)
                .gap(5.0)
                .bg(card_inner_bg())
                .bd(1.0, border_subtle())
                .bevel_rounded(9.0, 1.5);
            }
        }
        "error" => {
            let err_msg = doc
                .as_ref()
                .map(|d| d.error.as_str())
                .filter(|s| !s.is_empty())
                .unwrap_or("Unable to reach docs.chilloutvr.net");
            ui.col(|ui| {
                icons::icon(ui, icons::SHIELD, 24.0).fg(crate::theme::AMBER);
                ui.txt(&format!("Could not load {active_title}"))
                    .sz(crate::theme::fs(13.5))
                    .bold()
                    .fg(TEXT_PRIMARY);
                ui.txt(err_msg)
                    .sz(crate::theme::fs(11.0))
                    .fg(TEXT_MUTED)
                    .wrap();
                ui.row(|ui| {
                    if ui
                        .btn_with("mm_set_legal_retry", |ui| {
                            ui.txt("Retry")
                                .sz(crate::theme::fs(11.0))
                                .semibold()
                                .fg(TEXT_PRIMARY);
                        })
                        .px(14.0)
                        .py(7.0)
                        .bg(tint_alpha(accent, 48))
                        .hover_bg(tint_alpha(accent, 80))
                        .bd(1.0, tint_alpha(accent, 180))
                        .bevel_rounded(6.5, 1.3)
                        .clicked()
                    {
                        push_click(actions);
                        actions.push(UiAction::FetchLegalDoc {
                            doc_id: String::from(active_id),
                            force: true,
                        });
                    }
                    if ui
                        .btn_with("mm_set_legal_err_open", |ui| {
                            ui.txt("Open in Browser")
                                .sz(crate::theme::fs(11.0))
                                .semibold()
                                .fg(TEXT_PRIMARY);
                        })
                        .px(14.0)
                        .py(7.0)
                        .bg(tile_idle_bg())
                        .hover_bg(tile_idle_hover())
                        .bd(1.0, border_subtle())
                        .bevel_rounded(6.5, 1.3)
                        .clicked()
                    {
                        push_click(actions);
                        actions.push(UiAction::OpenExternalUrl {
                            url: String::from(active_url),
                        });
                    }
                })
                .gap(8.0)
                .pt(4.0);
            })
            .center()
            .p(24.0)
            .gap(8.0)
            .bg(card_inner_bg())
            .bd(1.0, border_subtle())
            .bevel_rounded(9.0, 1.5);
        }
        _ => {
            ui.col(|ui| {
                icons::icon(ui, icons::SHIELD, 24.0).fg(accent);
                ui.txt(&format!("Loading {active_title}..."))
                    .sz(crate::theme::fs(13.0))
                    .semibold()
                    .fg(TEXT_PRIMARY);
                ui.txt("Fetching latest document from docs.chilloutvr.net")
                    .sz(crate::theme::fs(10.5))
                    .fg(TEXT_MUTED);
            })
            .center()
            .p(28.0)
            .gap(6.0)
            .bg(card_inner_bg())
            .bd(1.0, border_subtle())
            .bevel_rounded(9.0, 1.5);
        }
    }
}

/// First-use modal shown once until accepted, warning users to fully uninstall LumenicMenu
/// (not merely toggle it off) before reporting any UI-related bug to ChilloutVR.
pub fn render_first_use_disclaimer_modal(
    ui: &mut UiContext,
    state: &mut AppState,
    accent: Color,
    actions: &mut Vec<UiAction>,
    prefix: &str,
) {
    let card_w = if prefix == "qm" { 560.0 } else { 620.0 };
    ui.zstack(|ui| {
        let _ = crate::theme::modal_scrim(
            ui,
            &format!("{prefix}_disc_scrim_fx"),
            &format!("{prefix}_disc_scrim_hit"),
        );

        ui.col(|ui| {
            ui.col(|_| {}).grow(1.0);
            ui.row(|ui| {
                ui.col(|_| {}).grow(1.0);
                let mut card = ui
                    .col(|ui| {
                        ui.row(|ui| {
                            icons::icon(ui, icons::BOLT, 18.0).fg(accent);
                            ui.txt("Welcome to LumenicMenu")
                                .sz(crate::theme::fs(16.0))
                                .bold()
                                .fg(TEXT_PRIMARY)
                                .grow(1.0);
                        })
                        .align_center()
                        .gap(8.0);

                        ui.col(|ui| {
                            ui.row(|ui| {
                                icons::icon(ui, icons::BELL, 15.0).fg(crate::theme::AMBER);
                                ui.txt("IMPORTANT - Read Before Reporting Any Bugs to ChilloutVR")
                                    .sz(crate::theme::fs(12.0))
                                    .bold()
                                    .fg(crate::theme::AMBER);
                            })
                            .align_center()
                            .gap(6.0);

                            ui.txt("If you notice ANY bug even remotely related to the UI, menus, input, or rendering, you MUST completely UNINSTALL LumenicMenu (delete LumenicMenu.dll from your Mods/ folder and restart ChilloutVR - do NOT just toggle it off) and verify the issue on a clean vanilla client before reporting anything to ChilloutVR or the CVR team.")
                                .sz(crate::theme::fs(11.5))
                                .fg(TEXT_PRIMARY)
                                .wrap();
                        })
                        .p(11.0)
                        .gap(6.0)
                        .bg(tint_alpha(crate::theme::AMBER, 30))
                        .bd(1.5, tint_alpha(crate::theme::AMBER, 190))
                        .bevel_rounded(8.0, 1.5);

                        ui.col(|ui| {
                            ui.txt("Mod Support & Bug Reports")
                                .sz(crate::theme::fs(12.0))
                                .bold()
                                .fg(TEXT_PRIMARY);
                            ui.txt("LumenicMenu is an unofficial community mod and is not affiliated with ChilloutVR or the ChilloutVR team. Using it may affect performance or stability; use at your own risk.")
                                .sz(crate::theme::fs(11.0))
                                .fg(TEXT_MUTED)
                                .wrap();
                            ui.txt("If you run into a mod bug or want to request a feature, open an issue or pull request on GitHub (LensError/Lumenic-CVR-Menu), or DM @LensError on Discord. You can also access these links anytime in Settings -> Support.")
                                .sz(crate::theme::fs(11.0))
                                .fg(TEXT_MUTED)
                                .wrap();
                        })
                        .p(10.0)
                        .gap(5.0)
                        .bg(card_inner_bg())
                        .bd(1.0, border_subtle())
                        .bevel_rounded(8.0, 1.4);

                        ui.row(|ui| {
                            if ui
                                .btn_with(&format!("{prefix}_disc_gh"), |ui| {
                                    ui.row(|ui| {
                                        icons::icon(ui, icons::WORLD, 13.0).fg(accent);
                                        ui.txt("Open GitHub")
                                            .sz(crate::theme::fs(11.5))
                                            .semibold()
                                            .fg(TEXT_PRIMARY);
                                    })
                                    .center()
                                    .gap(5.0)
                                    .grow(1.0);
                                })
                                .px(14.0)
                                .py(9.0)
                                .bg(tile_idle_bg())
                                .hover_bg(tile_idle_hover())
                                .bd(1.0, border_subtle())
                                .bevel_rounded(6.5, 1.3)
                                .clicked()
                            {
                                push_click(actions);
                                actions.push(UiAction::OpenExternalUrl {
                                    url: String::from(
                                        "https://github.com/LensError/Lumenic-CVR-Menu",
                                    ),
                                });
                            }

                            if ui
                                .btn_with(&format!("{prefix}_disc_ok"), |ui| {
                                    ui.txt("I Understand")
                                        .sz(crate::theme::fs(12.0))
                                        .bold()
                                        .align(TextAlign::Center)
                                        .fg(TEXT_PRIMARY)
                                        .grow(1.0);
                                })
                                .grow(1.0)
                                .py(9.0)
                                .bg(tint_alpha(accent, 65))
                                .hover_bg(tint_alpha(accent, 98))
                                .bd(1.5, accent)
                                .bevel_rounded(6.5, 1.3)
                                .clicked()
                            {
                                push_click(actions);
                                state.host.disclaimer_accepted = true;
                                actions.push(UiAction::AcceptDisclaimer);
                            }
                        })
                        .gap(8.0);
                    })
                    .w(card_w)
                    .p(16.0)
                    .gap(10.0)
                    .bg(Color::rgba(24, 24, 24, 250))
                    .bd(1.5, accent)
                    .bevel_rounded(12.0, 2.0)
                    .shadow(22.0, Color::rgba(0, 0, 0, 200));
                let _ = card.hovered();
                ui.col(|_| {}).grow(1.0);
            })
            .w_frac(1.0);
            ui.col(|_| {}).grow(1.0);
        })
        .w_frac(1.0)
        .h_frac(1.0);
    })
    .w_frac(1.0)
    .h_frac(1.0);
}

pub fn render_whats_new_modal(
    ui: &mut UiContext,
    state: &mut AppState,
    accent: Color,
    actions: &mut Vec<UiAction>,
    prefix: &str,
) {
    let card_w = if prefix == "qm" { 540.0 } else { 600.0 };
    let cur_v = if state.host.update_status.current_version.is_empty() {
        env!("CARGO_PKG_VERSION")
    } else {
        state.host.update_status.current_version.as_str()
    };
    let notes = if state.host.whats_new_notes.trim().is_empty() {
        "LumenicMenu has been updated."
    } else {
        state.host.whats_new_notes.as_str()
    };

    ui.zstack(|ui| {
        let _ = crate::theme::modal_scrim(
            ui,
            &format!("{prefix}_wn_scrim_fx"),
            &format!("{prefix}_wn_scrim_hit"),
        );

        ui.col(|ui| {
            ui.col(|_| {}).grow(1.0);
            ui.row(|ui| {
                ui.col(|_| {}).grow(1.0);
                let mut card = ui
                    .col(|ui| {
                        ui.row(|ui| {
                            icons::icon(ui, icons::BOLT, 18.0).fg(accent);
                            ui.txt(&format!("What's New in LumenicMenu v{cur_v}"))
                                .sz(crate::theme::fs(15.5))
                                .bold()
                                .fg(TEXT_PRIMARY)
                                .grow(1.0);
                        })
                        .align_center()
                        .gap(8.0);

                        ui.scroll(&format!("{prefix}_wn_notes_scroll"), |ui| {
                            ui.txt(notes)
                                .sz(crate::theme::fs(11.0))
                                .fg(TEXT_PRIMARY)
                                .wrap();
                        })
                        .h(200.0)
                        .p(10.0)
                        .bg(card_inner_bg())
                        .bd(1.0, border_subtle())
                        .bevel_rounded(8.0, 1.3);

                        ui.row(|ui| {
                            if ui
                                .btn_with(&format!("{prefix}_wn_gh"), |ui| {
                                    ui.txt("GitHub Releases")
                                        .sz(crate::theme::fs(11.5))
                                        .semibold()
                                        .align(TextAlign::Center)
                                        .fg(TEXT_PRIMARY)
                                        .grow(1.0);
                                })
                                .grow(1.0)
                                .py(8.5)
                                .bg(tile_idle_bg())
                                .hover_bg(tile_idle_hover())
                                .bd(1.0, border_subtle())
                                .bevel_rounded(6.5, 1.3)
                                .clicked()
                            {
                                push_click(actions);
                                actions.push(UiAction::OpenExternalUrl {
                                    url: String::from("https://github.com/LensError/Lumenic-CVR-Menu/releases"),
                                });
                            }

                            if ui
                                .btn_with(&format!("{prefix}_wn_ok"), |ui| {
                                    ui.txt("Got It")
                                        .sz(crate::theme::fs(11.5))
                                        .bold()
                                        .align(TextAlign::Center)
                                        .fg(TEXT_PRIMARY)
                                        .grow(1.0);
                                })
                                .grow(1.0)
                                .py(8.5)
                                .bg(tint_alpha(accent, 65))
                                .hover_bg(tint_alpha(accent, 98))
                                .bd(1.5, accent)
                                .bevel_rounded(6.5, 1.3)
                                .clicked()
                            {
                                push_click(actions);
                                state.host.show_whats_new = false;
                                actions.push(UiAction::DismissWhatsNew);
                            }
                        })
                        .gap(8.0);
                    })
                    .w(card_w)
                    .p(16.0)
                    .gap(10.0)
                    .bg(Color::rgba(24, 24, 24, 250))
                    .bd(1.5, accent)
                    .bevel_rounded(12.0, 2.0)
                    .shadow(22.0, Color::rgba(0, 0, 0, 200));
                let _ = card.hovered();
                ui.col(|_| {}).grow(1.0);
            })
            .w_frac(1.0);
            ui.col(|_| {}).grow(1.0);
        })
        .w_frac(1.0)
        .h_frac(1.0);
    })
    .w_frac(1.0)
    .h_frac(1.0);
}

pub fn render_mod_update_modal(
    ui: &mut UiContext,
    state: &mut AppState,
    accent: Color,
    actions: &mut Vec<UiAction>,
    prefix: &str,
) {
    let upd = state.host.update_status.clone();
    let card_w = if prefix == "qm" { 540.0 } else { 600.0 };
    let cur_v = if upd.current_version.is_empty() {
        env!("CARGO_PKG_VERSION")
    } else {
        upd.current_version.as_str()
    };
    let latest_v = if upd.latest_version.is_empty() {
        "latest"
    } else {
        upd.latest_version.as_str()
    };
    let notes = if upd.release_notes.trim().is_empty() {
        "A new release of LumenicMenu is available on GitHub."
    } else {
        upd.release_notes.as_str()
    };

    ui.zstack(|ui| {
        let _ = crate::theme::modal_scrim(
            ui,
            &format!("{prefix}_upd_scrim_fx"),
            &format!("{prefix}_upd_scrim_hit"),
        );

        ui.col(|ui| {
            ui.col(|_| {}).grow(1.0);
            ui.row(|ui| {
                ui.col(|_| {}).grow(1.0);
                let mut card = ui
                    .col(|ui| {
                        ui.row(|ui| {
                            icons::icon(ui, icons::BOLT, 18.0).fg(crate::theme::EMERALD);
                            ui.txt("LumenicMenu Update Available")
                                .sz(crate::theme::fs(15.5))
                                .bold()
                                .fg(TEXT_PRIMARY)
                                .grow(1.0);
                            ui.txt(&format!("v{cur_v} -> v{latest_v}"))
                                .sz(crate::theme::fs(11.5))
                                .bold()
                                .fg(crate::theme::EMERALD)
                                .px(8.0)
                                .py(3.5)
                                .bg(tint_alpha(crate::theme::EMERALD, 32))
                                .bd(1.0, tint_alpha(crate::theme::EMERALD, 140))
                                .bevel_rounded(6.0, 1.0);
                        })
                        .align_center()
                        .gap(8.0);

                        if !upd.release_title.trim().is_empty() {
                            ui.txt(&upd.release_title)
                                .sz(crate::theme::fs(12.5))
                                .semibold()
                                .fg(TEXT_PRIMARY)
                                .wrap();
                        }

                        ui.scroll(&format!("{prefix}_upd_notes_scroll"), |ui| {
                            ui.txt(notes)
                                .sz(crate::theme::fs(11.0))
                                .fg(TEXT_MUTED)
                                .wrap();
                        })
                        .h(180.0)
                        .p(10.0)
                        .bg(card_inner_bg())
                        .bd(1.0, border_subtle())
                        .bevel_rounded(8.0, 1.3);

                        ui.row(|ui| {
                            if ui
                                .btn_with(&format!("{prefix}_upd_dl"), |ui| {
                                    ui.txt("Download DLL")
                                        .sz(crate::theme::fs(11.5))
                                        .bold()
                                        .align(TextAlign::Center)
                                        .fg(TEXT_PRIMARY)
                                        .grow(1.0);
                                })
                                .grow(1.0)
                                .py(8.5)
                                .bg(tint_alpha(crate::theme::EMERALD, 58))
                                .hover_bg(tint_alpha(crate::theme::EMERALD, 92))
                                .bd(1.5, crate::theme::EMERALD)
                                .bevel_rounded(6.5, 1.3)
                                .clicked()
                            {
                                push_click(actions);
                                actions.push(UiAction::OpenExternalUrl {
                                    url: upd.download_url.clone(),
                                });
                            }

                            if ui
                                .btn_with(&format!("{prefix}_upd_page"), |ui| {
                                    ui.txt("Release Notes")
                                        .sz(crate::theme::fs(11.5))
                                        .semibold()
                                        .align(TextAlign::Center)
                                        .fg(TEXT_PRIMARY)
                                        .grow(1.0);
                                })
                                .grow(1.0)
                                .py(8.5)
                                .bg(tint_alpha(accent, 42))
                                .hover_bg(tint_alpha(accent, 75))
                                .bd(1.0, tint_alpha(accent, 175))
                                .bevel_rounded(6.5, 1.3)
                                .clicked()
                            {
                                push_click(actions);
                                actions.push(UiAction::OpenExternalUrl {
                                    url: upd.release_url.clone(),
                                });
                            }
                        })
                        .gap(8.0);

                        ui.row(|ui| {
                            if ui
                                .btn_with(&format!("{prefix}_upd_skip"), |ui| {
                                    ui.txt("Skip This Version")
                                        .sz(crate::theme::fs(11.0))
                                        .semibold()
                                        .align(TextAlign::Center)
                                        .fg(TEXT_MUTED)
                                        .grow(1.0);
                                })
                                .grow(1.0)
                                .py(7.5)
                                .bg(tile_idle_bg())
                                .hover_bg(tile_idle_hover())
                                .bd(1.0, border_subtle())
                                .bevel_rounded(6.5, 1.3)
                                .clicked()
                            {
                                push_click(actions);
                                state.host.update_status.show_modal = false;
                                actions.push(UiAction::SkipUpdateVersion {
                                    version: upd.latest_version.clone(),
                                });
                            }

                            if ui
                                .btn_with(&format!("{prefix}_upd_later"), |ui| {
                                    ui.txt("Remind Later")
                                        .sz(crate::theme::fs(11.0))
                                        .semibold()
                                        .align(TextAlign::Center)
                                        .fg(TEXT_PRIMARY)
                                        .grow(1.0);
                                })
                                .grow(1.0)
                                .py(7.5)
                                .bg(tile_idle_bg())
                                .hover_bg(tile_idle_hover())
                                .bd(1.0, border_subtle())
                                .bevel_rounded(6.5, 1.3)
                                .clicked()
                            {
                                push_click(actions);
                                state.host.update_status.show_modal = false;
                                actions.push(UiAction::RemindUpdateLater);
                            }
                        })
                        .gap(8.0);
                    })
                    .w(card_w)
                    .p(16.0)
                    .gap(10.0)
                    .bg(Color::rgba(24, 24, 24, 250))
                    .bd(1.5, crate::theme::EMERALD)
                    .bevel_rounded(12.0, 2.0)
                    .shadow(22.0, Color::rgba(0, 0, 0, 200));
                let _ = card.hovered();
                ui.col(|_| {}).grow(1.0);
            })
            .w_frac(1.0);
            ui.col(|_| {}).grow(1.0);
        })
        .w_frac(1.0)
        .h_frac(1.0);
    })
    .w_frac(1.0)
    .h_frac(1.0);
}

pub fn render_system_alert_modal(
    ui: &mut UiContext,
    alert: &crate::state::SystemAlertDto,
    accent: Color,
    actions: &mut Vec<UiAction>,
    prefix: &str,
) -> bool {
    let mut keep_open = true;
    let card_w = if prefix == "qm" { 500.0 } else { 560.0 };
    let is_confirm = alert.kind == "confirm";
    let border_col = if is_confirm { accent } else { crate::theme::AMBER };
    let title = if alert.title.trim().is_empty() {
        "System Message"
    } else {
        alert.title.as_str()
    };

    ui.zstack(|ui| {
        let _ = crate::theme::modal_scrim(
            ui,
            &format!("{prefix}_sys_scrim_fx"),
            &format!("{prefix}_sys_scrim_hit"),
        );

        ui.col(|ui| {
            ui.col(|_| {}).grow(1.0);
            ui.row(|ui| {
                ui.col(|_| {}).grow(1.0);
                let mut card = ui
                    .col(|ui| {
                        ui.row(|ui| {
                            icons::icon(ui, icons::BELL, 17.0).fg(border_col);
                            ui.txt(title)
                                .sz(crate::theme::fs(15.0))
                                .bold()
                                .fg(TEXT_PRIMARY)
                                .grow(1.0)
                                .wrap();
                        })
                        .align_center()
                        .gap(8.0);

                        if !alert.message.trim().is_empty() {
                            ui.scroll(&format!("{prefix}_sys_msg_scroll"), |ui| {
                                ui.txt(&alert.message)
                                    .sz(crate::theme::fs(11.5))
                                    .fg(TEXT_PRIMARY)
                                    .wrap();
                            })
                            .max_h(220.0)
                            .p(10.0)
                            .bg(card_inner_bg())
                            .bd(1.0, border_subtle())
                            .bevel_rounded(8.0, 1.3);
                        }

                        ui.row(|ui| {
                            if is_confirm {
                                if ui
                                    .btn_with(&format!("{prefix}_sys_cancel"), |ui| {
                                        ui.txt("Cancel")
                                            .sz(crate::theme::fs(11.5))
                                            .semibold()
                                            .align(TextAlign::Center)
                                            .fg(TEXT_PRIMARY)
                                            .grow(1.0);
                                    })
                                    .grow(1.0)
                                    .py(8.5)
                                    .bg(tile_idle_bg())
                                    .hover_bg(tile_idle_hover())
                                    .bd(1.0, border_subtle())
                                    .bevel_rounded(6.5, 1.3)
                                    .clicked()
                                {
                                    push_click(actions);
                                    keep_open = false;
                                    actions.push(UiAction::RespondSystemConfirm {
                                        id: alert.id,
                                        confirmed: false,
                                    });
                                }

                                if ui
                                    .btn_with(&format!("{prefix}_sys_confirm"), |ui| {
                                        ui.txt("Confirm")
                                            .sz(crate::theme::fs(11.5))
                                            .bold()
                                            .align(TextAlign::Center)
                                            .fg(TEXT_PRIMARY)
                                            .grow(1.0);
                                    })
                                    .grow(1.0)
                                    .py(8.5)
                                    .bg(tint_alpha(accent, 65))
                                    .hover_bg(tint_alpha(accent, 98))
                                    .bd(1.5, accent)
                                    .bevel_rounded(6.5, 1.3)
                                    .clicked()
                                {
                                    push_click(actions);
                                    keep_open = false;
                                    actions.push(UiAction::RespondSystemConfirm {
                                        id: alert.id,
                                        confirmed: true,
                                    });
                                }
                            } else if ui
                                .btn_with(&format!("{prefix}_sys_ok"), |ui| {
                                    ui.txt("OK")
                                        .sz(crate::theme::fs(11.5))
                                        .bold()
                                        .align(TextAlign::Center)
                                        .fg(TEXT_PRIMARY)
                                        .grow(1.0);
                                })
                                .grow(1.0)
                                .py(8.5)
                                .bg(tint_alpha(border_col, 55))
                                .hover_bg(tint_alpha(border_col, 88))
                                .bd(1.5, border_col)
                                .bevel_rounded(6.5, 1.3)
                                .clicked()
                            {
                                push_click(actions);
                                keep_open = false;
                                actions.push(UiAction::DismissSystemAlert { id: alert.id });
                            }
                        })
                        .gap(8.0);
                    })
                    .w(card_w)
                    .p(16.0)
                    .gap(10.0)
                    .bg(Color::rgba(24, 24, 24, 250))
                    .bd(1.5, border_col)
                    .bevel_rounded(12.0, 2.0)
                    .shadow(22.0, Color::rgba(0, 0, 0, 200));
                let _ = card.hovered();
                ui.col(|_| {}).grow(1.0);
            })
            .w_frac(1.0);
            ui.col(|_| {}).grow(1.0);
        })
        .w_frac(1.0)
        .h_frac(1.0);
    })
    .w_frac(1.0)
    .h_frac(1.0);
    keep_open
}

pub fn render_dropdown_modal(
    ui: &mut UiContext,
    dropdown: &crate::state::ActiveDropdown,
    accent: Color,
    actions: &mut Vec<UiAction>,
    prefix: &str,
) -> bool {
    let mut keep_open = true;
    let card_w = if prefix == "qm" { 400.0 } else { 440.0 };

    ui.zstack(|ui| {
        if crate::theme::modal_scrim(
            ui,
            &format!("{prefix}_dd_scrim_fx"),
            &format!("{prefix}_dd_scrim_hit"),
        ) {
            push_click(actions);
            keep_open = false;
        }

        ui.col(|ui| {
            ui.col(|_| {}).grow(1.0);
            ui.row(|ui| {
                ui.col(|_| {}).grow(1.0);
                let mut card = ui
                    .col(|ui| {
                        ui.row(|ui| {
                            icons::icon(ui, icons::CHEVRON_DOWN, 18.0).fg(accent);
                            ui.txt(&dropdown.title)
                                .sz(crate::theme::fs(14.0))
                                .bold()
                                .fg(TEXT_PRIMARY)
                                .grow(1.0)
                                .wrap();
                            if ui
                                .btn_with(&format!("{prefix}_dd_close"), |ui| {
                                    icons::icon(ui, icons::CLOSE, 14.0).fg(TEXT_MUTED);
                                })
                                .p(5.0)
                                .bg(tile_idle_bg())
                                .hover_bg(tile_idle_hover())
                                .bd(1.0, border_subtle())
                                .bevel_rounded(5.0, 1.0)
                                .clicked()
                            {
                                push_click(actions);
                                keep_open = false;
                            }
                        })
                        .align_center()
                        .gap(8.0);

                        ui.scroll(&format!("{prefix}_dd_scroll"), |ui| {
                            ui.col(|ui| {
                                for (idx, opt) in dropdown.options.iter().enumerate() {
                                    if ui
                                        .btn_with(&format!("{prefix}_dd_opt_{idx}"), |ui| {
                                            ui.row(|ui| {
                                                ui.txt(&opt.label)
                                                    .sz(crate::theme::fs(11.5))
                                                    .semibold()
                                                    .fg(if opt.selected {
                                                        TEXT_PRIMARY
                                                    } else {
                                                        TEXT_MUTED
                                                    })
                                                    .grow(1.0)
                                                    .wrap();
                                                if opt.selected {
                                                    icons::icon(ui, icons::CHECK, 14.0).fg(accent);
                                                }
                                            })
                                            .align_center()
                                            .gap(8.0)
                                            .grow(1.0);
                                        })
                                        .w_frac(1.0)
                                        .px(10.0)
                                        .py(7.5)
                                        .bg(if opt.selected {
                                            tint_alpha(accent, 48)
                                        } else {
                                            tile_idle_bg()
                                        })
                                        .hover_bg(tile_idle_hover())
                                        .bd(
                                            1.0,
                                            if opt.selected {
                                                accent
                                            } else {
                                                border_subtle()
                                            },
                                        )
                                        .bevel_rounded(6.5, 1.3)
                                        .clicked()
                                    {
                                        push_click(actions);
                                        actions.push(opt.action.clone());
                                        keep_open = false;
                                    }
                                }
                            })
                            .gap(5.0);
                        })
                        .max_h(320.0);
                    })
                    .w(card_w)
                    .p(14.0)
                    .gap(10.0)
                    .bg(Color::rgba(24, 24, 24, 250))
                    .bd(1.5, accent)
                    .bevel_rounded(12.0, 2.0)
                    .shadow(22.0, Color::rgba(0, 0, 0, 200));
                let _ = card.hovered();
                ui.col(|_| {}).grow(1.0);
            })
            .w_frac(1.0);
            ui.col(|_| {}).grow(1.0);
        })
        .w_frac(1.0)
        .h_frac(1.0);
    })
    .w_frac(1.0)
    .h_frac(1.0);
    keep_open
}
