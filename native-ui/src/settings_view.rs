use lumenic::{Color, TextAlign, UiContext};

use crate::icons;
use crate::state::{AppState, CvrSettingItem, UiAction};
use crate::theme::{
    push_click, tint_alpha, BORDER_SUBTLE, CARD_BG, CARD_INNER_BG, EMERALD, SHADOW_CARD, TEXT_DIM,
    TEXT_MUTED, TEXT_PRIMARY, TILE_IDLE_BG, TILE_IDLE_HOVER,
};

const SETTINGS_TABS: [(&str, &str, &str); 11] = [
    ("lumenic", icons::BOLT, "LumenicMenu"),
    ("general", icons::HOME, "General"),
    ("audio", icons::VOLUME, "Audio"),
    ("graphics", icons::CAMERA, "Graphics"),
    ("controls", icons::VR_HEADSET, "Controls & VR"),
    ("ik", icons::FBT, "IK & Full Body"),
    ("hud", icons::BELL, "HUD & Plates"),
    ("interactions", icons::CURSOR, "Interactions"),
    ("implementation", icons::PUZZLE, "Implementation"),
    ("safety", icons::CHECK, "Safety & Filter"),
    ("experimental", icons::TOOLBOX, "Experimental"),
];

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
                .sz(11.0)
                .semibold()
                .fg(TEXT_PRIMARY)
                .ellipsis()
                .grow(1.0);

            if ui
                .btn_with(id, |ui| {
                    ui.txt(if active { "ON" } else { "OFF" })
                        .sz(10.0)
                        .bold()
                        .align(TextAlign::Center)
                        .fg(if active { EMERALD } else { TEXT_MUTED })
                        .grow(1.0);
                })
                .w(50.0)
                .h(22.0)
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
        .bg(CARD_INNER_BG)
        .bd(1.0, BORDER_SUBTLE)
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
            let display_lbl = if item.unit.is_empty() {
                item.label.clone()
            } else {
                format!("{} ({})", item.label, item.unit)
            };

            ui.push_id(&item.key);
            let mut resp = ui
                .slider(&display_lbl, &mut v, (min_v as f32)..=(max_v as f32))
                .fill(accent)
                .bg(CARD_INNER_BG)
                .bd(1.0, BORDER_SUBTLE)
                .bevel_rounded(6.0, 1.3)
                .grow(1.0);
            if resp.changed() {
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
            is_hovered = resp.hovered();
            ui.pop_id();
        }
        "int_choice" | "str_choice" => {
            let is_int = item.kind == "int_choice";
            let mut resp = ui
                .col(|ui| {
                    ui.txt(&item.label)
                        .sz(10.5)
                        .semibold()
                        .fg(TEXT_PRIMARY)
                        .ellipsis();
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
                                        .sz(10.0)
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
                })
                .grow(1.0)
                .px(8.0)
                .py(5.0)
                .gap(4.0)
                .bg(CARD_INNER_BG)
                .bd(1.0, BORDER_SUBTLE)
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
                        .sz(10.5)
                        .semibold()
                        .fg(TEXT_PRIMARY)
                        .ellipsis()
                        .grow(1.0);
                    if ui
                        .btn_with(&format!("mm_sm_{}", item.key), |ui| {
                            ui.txt("-").sz(11.0).bold().fg(TEXT_PRIMARY);
                        })
                        .w(24.0)
                        .py(3.0)
                        .bg(TILE_IDLE_BG)
                        .hover_bg(TILE_IDLE_HOVER)
                        .bd(1.0, BORDER_SUBTLE)
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
                        .sz(10.5)
                        .mono()
                        .bold()
                        .align(TextAlign::Center)
                        .fg(accent)
                        .min_w(38.0);
                    if ui
                        .btn_with(&format!("mm_sp_{}", item.key), |ui| {
                            ui.txt("+").sz(11.0).bold().fg(TEXT_PRIMARY);
                        })
                        .w(24.0)
                        .py(3.0)
                        .bg(TILE_IDLE_BG)
                        .hover_bg(TILE_IDLE_HOVER)
                        .bd(1.0, BORDER_SUBTLE)
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
                .bg(CARD_INNER_BG)
                .bd(1.0, BORDER_SUBTLE)
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
                        .sz(10.5)
                        .semibold()
                        .fg(TEXT_PRIMARY)
                        .ellipsis()
                        .grow(1.0);
                    if ui
                        .btn_with(&format!("mm_fm_{}", item.key), |ui| {
                            ui.txt("-").sz(11.0).bold().fg(TEXT_PRIMARY);
                        })
                        .w(24.0)
                        .py(3.0)
                        .bg(TILE_IDLE_BG)
                        .hover_bg(TILE_IDLE_HOVER)
                        .bd(1.0, BORDER_SUBTLE)
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
                        .sz(10.5)
                        .mono()
                        .bold()
                        .align(TextAlign::Center)
                        .fg(accent)
                        .min_w(42.0);
                    if ui
                        .btn_with(&format!("mm_fp_{}", item.key), |ui| {
                            ui.txt("+").sz(11.0).bold().fg(TEXT_PRIMARY);
                        })
                        .w(24.0)
                        .py(3.0)
                        .bg(TILE_IDLE_BG)
                        .hover_bg(TILE_IDLE_HOVER)
                        .bd(1.0, BORDER_SUBTLE)
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
                .bg(CARD_INNER_BG)
                .bd(1.0, BORDER_SUBTLE)
                .bevel_rounded(6.0, 1.3);
            is_hovered = resp.hovered();
        }
        _ => {
            let mut resp = ui
                .row(|ui| {
                    ui.txt(&item.label)
                        .sz(10.5)
                        .semibold()
                        .fg(TEXT_PRIMARY)
                        .ellipsis()
                        .grow(1.0);
                    let val_txt = if item.str_val.is_empty() {
                        "-"
                    } else {
                        &item.str_val
                    };
                    ui.txt(val_txt).sz(10.5).mono().fg(TEXT_MUTED).ellipsis();
                })
                .align_center()
                .grow(1.0)
                .px(8.0)
                .py(5.0)
                .gap(6.0)
                .bg(CARD_INNER_BG)
                .bd(1.0, BORDER_SUBTLE)
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
            ui.txt("Settings").sz(15.0).bold().fg(TEXT_PRIMARY);

            ui.row(|ui| {
                icons::icon(ui, icons::SEARCH, 13.0).fg(TEXT_MUTED);
                let mut s_inp = ui
                    .input("mm_set_search", &mut state.main_menu.settings_search_input)
                    .placeholder("Search all settings...")
                    .bg(CARD_INNER_BG)
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
                        .hover_bg(TILE_IDLE_HOVER)
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

            if state.host.uix_available
                && ui
                    .btn_with("mm_set_jump_mods", |ui| {
                        ui.row(|ui| {
                            icons::icon(ui, icons::PUZZLE, 13.0).fg(accent);
                            ui.txt("Mod Settings").sz(11.0).semibold().fg(TEXT_PRIMARY);
                        })
                        .center()
                        .gap(5.0);
                    })
                    .px(10.0)
                    .py(5.5)
                    .bg(TILE_IDLE_BG)
                    .hover_bg(TILE_IDLE_HOVER)
                    .bd(1.0, tint_alpha(accent, 135))
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
        .align_center()
        .w_frac(1.0)
        .px(12.0)
        .py(7.0)
        .gap(8.0)
        .bg(CARD_BG)
        .bd(1.0, BORDER_SUBTLE)
        .bevel_rounded(10.0, 2.0)
        .shadow(14.0, SHADOW_CARD);

        // Body: Compact Left Category List | Dense 2-Column Settings Grid
        ui.row(|ui| {
            ui.col(|ui| {
                ui.scroll("mm_set_tabs_scroll", |ui| {
                    for (idx, (cat_id, icon_p, lbl)) in SETTINGS_TABS.iter().enumerate() {
                        let active = state.main_menu.settings_tab == idx;
                        let count = if *cat_id == "lumenic" {
                            5
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
                                        .sz(11.5)
                                        .semibold()
                                        .fg(if active { TEXT_PRIMARY } else { TEXT_MUTED })
                                        .ellipsis()
                                        .grow(1.0);
                                    if count > 0 {
                                        ui.txt(&format!("{count}"))
                                            .sz(9.5)
                                            .mono()
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
                            state.main_menu.settings_tab = idx;
                        }
                    }
                })
                .grow(1.0)
                .gap(4.5)
                .no_scrollbar();
            })
            .w(214.0)
            .h_frac(1.0)
            .p(8.0)
            .bg(CARD_BG)
            .bd(1.0, BORDER_SUBTLE)
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
                ui.scroll("mm_set_body_scroll", |ui| {
                    if q.is_empty() && active_cat == "lumenic" {
                        render_lumenic_settings(ui, state, "", accent, actions);
                    } else {
                        let mut total_hits = 0usize;
                        if !q.is_empty() {
                            total_hits += render_lumenic_settings(ui, state, &q, accent, actions);
                        }

                        let filtered: Vec<CvrSettingItem> = state
                            .host
                            .cvr_settings
                            .items
                            .iter()
                            .filter(|it| {
                                if q.is_empty() {
                                    it.category == active_cat
                                } else {
                                    it.label.to_ascii_lowercase().contains(&q)
                                        || it.key.to_ascii_lowercase().contains(&q)
                                        || it.category.to_ascii_lowercase().contains(&q)
                                }
                            })
                            .cloned()
                            .collect();

                        total_hits += filtered.len();

                        // Group consecutive items by their official OG sub-group heading
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
                                if !grp.is_empty() {
                                    ui.txt(grp)
                                        .sz(11.0)
                                        .bold()
                                        .fg(Color::rgb(196, 210, 230))
                                        .pt(6.0)
                                        .px(3.0);
                                }
                                for pair in items.chunks(2) {
                                    ui.row(|ui| {
                                        render_cvr_item(ui, state, pair[0], accent, actions);
                                        if pair.len() > 1 {
                                            render_cvr_item(ui, state, pair[1], accent, actions);
                                        } else {
                                            ui.col(|_| {}).grow(1.0);
                                        }
                                    })
                                    .align_center()
                                    .gap(6.0);
                                }
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
                                    .sz(12.5)
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
                .gap(5.5)
                .no_scrollbar();
            })
            .grow(1.0)
            .h_frac(1.0)
            .p(9.0)
            .bg(CARD_BG)
            .bd(1.0, BORDER_SUBTLE)
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

    if m("UI Sound Pack", "glass softsub vanilla audio") {
        hits += 1;
        ui.row(|ui| {
            ui.txt("UI Sound Pack")
                .sz(11.0)
                .semibold()
                .fg(TEXT_PRIMARY)
                .min_w(130.0);
            for (pack, lbl) in [
                ("Glass", "Glass"),
                ("SoftSub", "SoftSub (VR)"),
                ("Vanilla", "Vanilla"),
            ] {
                let active = state.host.ui_sound_pack.eq_ignore_ascii_case(pack);
                if ui
                    .btn_with(&format!("mm_set_sp_{pack}"), |ui| {
                        ui.txt(lbl)
                            .sz(10.5)
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
        .bg(CARD_INNER_BG)
        .bd(1.0, BORDER_SUBTLE)
        .bevel_rounded(6.0, 1.3);
    }

    if m("UI Click & Hover Sounds", "sound") || m("Stopwatch Performance Logging", "perf ms") {
        ui.row(|ui| {
            if m("UI Click & Hover Sounds", "sound") {
                hits += 1;
                let (clicked, hov) = compact_toggle_cell(
                    ui,
                    "mm_set_lm_snd",
                    "UI Click & Hover Sounds",
                    state.host.ui_click_sounds,
                );
                if hov {
                    state.main_menu.hovered_tooltip = String::from("Enable procedural hover and click audio feedback across Lumenic menus.");
                }
                if clicked {
                    push_click(actions);
                    actions.push(UiAction::ToggleUiClickSounds);
                }
            }
            if m("Stopwatch Performance Logging", "perf ms") {
                hits += 1;
                let (clicked, hov) = compact_toggle_cell(
                    ui,
                    "mm_set_lm_perf",
                    "Stopwatch Perf Logging",
                    state.host.perf_logging,
                );
                if hov {
                    state.main_menu.hovered_tooltip = String::from("Logs step, pixel readback, texture upload, and blit timings in milliseconds.");
                }
                if clicked {
                    push_click(actions);
                    actions.push(UiAction::TogglePerfLogging);
                }
            }
        })
        .gap(6.0);
    }

    if m("Linear Texture Color Space", "srgb") || m("Max Redraw Refresh Rate", "hz fps") {
        ui.row(|ui| {
            if m("Linear Texture Color Space", "srgb") {
                hits += 1;
                let (clicked, hov) = compact_toggle_cell(
                    ui,
                    "mm_set_lm_lin",
                    "Linear Texture Mode",
                    state.host.linear_texture,
                );
                if hov {
                    state.main_menu.hovered_tooltip = String::from("Toggle Texture2D color space between sRGB (default) and Linear.");
                }
                if clicked {
                    push_click(actions);
                    actions.push(UiAction::ToggleLinearTexture);
                }
            }
            if m("Max Redraw Refresh Rate", "hz fps") {
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
                ui.push_id("mm_set_lm_hz");
                if ui
                    .slider("Max Redraw Rate (Hz)", &mut hz_v, 15.0..=144.0)
                    .fill(accent)
                    .bg(CARD_INNER_BG)
                    .bd(1.0, BORDER_SUBTLE)
                    .bevel_rounded(6.0, 1.3)
                    .grow(1.0)
                    .changed()
                {
                    let next = (hz_v.round() as i32).clamp(15, 144);
                    *hz_f = next as f32;
                    if next != *last_hz {
                        *last_hz = next;
                        actions.push(UiAction::SetMaxRefreshHz { hz: next });
                    }
                }
                ui.pop_id();
            }
        })
        .gap(6.0);
    }

    hits
}
