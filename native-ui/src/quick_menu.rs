use lumenic::{App, Color, ImageFit, TextAlign, UiContext};

use crate::icons;
use crate::state::{AppState, GroupModPromptSnapshot, UiAction};
use crate::theme::{
    push_click, resolve_player_accent, tint_alpha, AMBER, border_subtle, card_bg, card_inner_bg,
    EMERALD, RED, SHADOW_CARD, TEXT_DIM, TEXT_MUTED, TEXT_PRIMARY, tile_idle_bg, tile_idle_hover,
};
use crate::uilib_view::{render_uilib_modal, render_uilib_page, render_uilib_toast};

#[allow(clippy::too_many_arguments)]
fn action_tile(
    ui: &mut UiContext,
    id: &str,
    icon_path: &str,
    label: &str,
    status: &str,
    active: bool,
    enabled: bool,
    active_color: Color,
) -> bool {
    let c = crate::widgets::resolve_tile_colors(active, enabled, active_color);

    let resp = ui
        .btn_with(id, |ui| {
            ui.col(|ui| {
                icons::icon(ui, icon_path, 19.0).fg(c.icon);
                ui.txt(label)
                    .sz(crate::theme::fs(12.0))
                    .semibold()
                    .align(TextAlign::Center)
                    .fg(c.title);
                ui.txt(status)
                    .sz(crate::theme::fs(9.5))
                    .medium()
                    .align(TextAlign::Center)
                    .fg(c.status);
            })
            .center()
            .gap(2.5)
            .grow(1.0);
        })
        .grow(1.0)
        .h_frac(1.0)
        .min_h(crate::theme::tile_min_h())
        .p(crate::theme::pad(5.0))
        .bg(c.bg)
        .hover_bg(c.hover_bg)
        .bd(if active { 1.5 } else { 1.0 }, c.border)
        .bevel_rounded(7.8, 1.3)
        .opacity(if enabled { 1.0 } else { 0.52 });

    enabled && resp.clicked()
}

/// Renders the Milestone 2+ Obsidian Slate + Player Accent Quick Menu with Player PFP and UILib Mods.
pub fn render_quick_menu(
    ui: &mut UiContext,
    _app: &mut App,
    state: &mut AppState,
    actions: &mut Vec<UiAction>,
) {
    // Sync volume sliders whenever CVR settings change externally.
    if state.host.audio_master != state.quick.last_host_master {
        state.quick.last_host_master = state.host.audio_master;
        state.quick.master_slider = state.host.audio_master.clamp(0, 100) as f32;
    }
    if state.host.audio_voice != state.quick.last_host_voice {
        state.quick.last_host_voice = state.host.audio_voice;
        state.quick.voice_slider = state.host.audio_voice.clamp(0, 100) as f32;
    }
    if state.host.audio_video_player != state.quick.last_host_video {
        state.quick.last_host_video = state.host.audio_video_player;
        state.quick.video_slider = state.host.audio_video_player.clamp(0, 100) as f32;
    }

    crate::theme::begin_frame(state);
    let accent = resolve_player_accent(state);
    state.quick.avatar_preview_rendered = false;
    let accent_border = tint_alpha(accent, 110);
    let active_uilib_page = state.host.uilib.active_page.take();
    let active_ext_page = state.extensions.qm_active_page.take();
    let ext_tabs = std::mem::take(&mut state.extensions.qm_tabs);
    let tabs = std::mem::take(&mut state.host.uilib.tabs);

    // Menu-wide textured cursor-follow glow shader positioned behind all cards
    let shader_uniforms = crate::widgets::compute_bg_glow_uniforms(
        ui,
        state.quick.qm_root_rect,
        accent,
        state.quick.studio_cutout_rect,
    );

    let outer_resp = ui
        .zstack(|ui| {
            crate::widgets::render_bg_glow(ui, "qm_bg_glow", 0x4C55_4D01, shader_uniforms);

            ui.col(|ui| {
        // =====================================================================
        // 1. TOP TELEMETRY & PLAYER PFP CARD
        // =====================================================================
        ui.row(|ui| {
            // Clickable Player PFP + Greeting + World (opens local player's own QM Player Details page)
            if ui
                .btn_with("qm_btn_local_profile", |ui| {
                    ui.row(|ui| {
                        let pfp_key = if state.host.local_pfp_key.is_empty() {
                            "local_pfp"
                        } else {
                            &state.host.local_pfp_key
                        };
                        if let Some(tex) = state.quick_textures.get(pfp_key) {
                            ui.img(tex)
                                .w(40.0)
                                .h(40.0)
                                .fit(ImageFit::Cover)
                                .hexagon_pointy().r(3.5)
                                .bd(2.0, accent);
                        } else {
                            let initial = state
                                .host
                                .username
                                .chars()
                                .next()
                                .unwrap_or('P')
                                .to_ascii_uppercase()
                                .to_string();
                            ui.col(|ui| {
                                ui.txt(&initial)
                                    .sz(crate::theme::fs(16.0))
                                    .bold()
                                    .align(TextAlign::Center)
                                    .fg(accent);
                            })
                            .center()
                            .w(40.0)
                            .h(40.0)
                            .bg(tint_alpha(accent, 34))
                            .bd(2.0, accent)
                            .hexagon_pointy().r(3.5);
                        }

                        ui.col(|ui| {
                            let user = if state.host.username.is_empty() {
                                "Player"
                            } else {
                                &state.host.username
                            };
                            let greeting = format!("Welcome, {user}");
                            ui.txt(&greeting).sz(crate::theme::fs(14.5)).bold().fg(TEXT_PRIMARY);

                            let world_raw = if state.host.world_name.is_empty() {
                                "Local World"
                            } else {
                                &state.host.world_name
                            };
                            let world: String = if world_raw.chars().count() > 46 {
                                let mut s: String = world_raw.chars().take(43).collect();
                                s.push_str("...");
                                s
                            } else {
                                world_raw.to_owned()
                            };
                            ui.txt(&world).sz(crate::theme::fs(11.0)).fg(TEXT_MUTED);
                        })
                        .gap(1.5)
                        .grow(1.0);
                    })
                    .align_center()
                    .gap(9.0)
                    .grow(1.0);
                })
                .px(4.0)
                .py(2.0)
                .bg(Color::TRANSPARENT)
                .hover_bg(Color::rgba(148, 163, 184, 24))
                .borderless()
                .bevel_rounded(7.0, 1.3)
                .grow(1.0)
                .clicked()
            {
                state.quick.go_home_confirm = false;
                state.quick.show_info_modal = false;
                push_click(actions);
                actions.push(UiAction::OpenLocalPlayerQmPage);
            }

            // Slim clickable Telemetry Pill (FPS + Ping -> System Info modal) + Right-Aligned Clock
            ui.row(|ui| {
                let fps_col = if state.host.fps >= 60 {
                    EMERALD
                } else if state.host.fps >= 36 {
                    AMBER
                } else {
                    RED
                };
                let fps_txt = format!("{} FPS", state.host.fps.max(0));
                let ping_txt = format!("{} ms", state.host.game_ping.max(0));

                if ui
                    .btn_with("qm_btn_fps_info", |ui| {
                        ui.row(|ui| {
                            ui.col(|_| {})
                                .w(6.0)
                                .h(6.0)
                                .r(3.0)
                                .bg(fps_col);
                            ui.txt(&fps_txt).sz(crate::theme::fs(11.5)).bold().fg(fps_col);
                            ui.col(|_| {})
                                .w(1.0)
                                .h(12.0)
                                .bg(Color::rgba(148, 163, 184, 65));
                            ui.txt(&ping_txt).sz(crate::theme::fs(11.0)).semibold().fg(TEXT_MUTED);
                        })
                        .align_center()
                        .gap(6.0);
                    })
                    .h(crate::theme::text_box(28.0))
                    .bg(if state.quick.show_info_modal {
                        tint_alpha(accent, 45)
                    } else {
                        card_inner_bg()
                    })
                    .hover_bg(tile_idle_hover())
                    .bd(
                        1.0,
                        if state.quick.show_info_modal {
                            accent
                        } else {
                            border_subtle()
                        },
                    )
                    .px(10.0)
                    .py(0.0)
                    .bevel_rounded(6.0, 1.3)
                    .clicked()
                {
                    push_click(actions);
                    state.quick.show_info_modal = !state.quick.show_info_modal;
                }

                ui.col(|_| {})
                    .w(1.0)
                    .h(crate::theme::text_box(28.0))
                    .bg(Color::rgba(148, 163, 184, 55));

                ui.col(|ui| {
                    let clock = if state.host.clock_text.is_empty() {
                        "--:--"
                    } else {
                        &state.host.clock_text
                    };
                    ui.txt(clock)
                        .sz(crate::theme::fs(15.5))
                        .bold()
                        .align(TextAlign::Right)
                        .fg(TEXT_PRIMARY)
                        .w_frac(1.0);
                    if !state.host.date_text.is_empty() {
                        ui.txt(&state.host.date_text)
                            .sz(crate::theme::fs(10.0))
                            .medium()
                            .align(TextAlign::Right)
                            .fg(TEXT_MUTED)
                            .w_frac(1.0);
                    }
                })
                .w(crate::theme::text_box(78.0))
                .pt(2.0)
                .gap(1.0);
            })
            .align_center()
            .gap(10.0);
        })
        .align_center()
        .px(11.0)
        .py(7.0)
        .gap(10.0)
        .bg(card_bg())
        .bd(1.0, accent_border)
        .bevel_rounded(10.9, 2.0)
        .shadow(14.0, SHADOW_CARD);

        // =====================================================================
        // 2. CENTER REGION: ACTIVE EXTENSION PAGE, UILIB MOD PAGE, OR HOME
        // =====================================================================
        if let Some(ref ext_page) = active_ext_page {
            crate::ext_view::render_ext_page(ui, state, "quick", ext_page, accent, actions);
        } else if let Some(ref page) = active_uilib_page {
            render_uilib_page(ui, state, page, accent, actions);
        } else {
            crate::ext_view::render_slot_only(ui, state, "quick", "qm.home.banner", accent, actions);
            render_home_center(ui, state, accent, actions);
        }
        state.host.uilib.active_page = active_uilib_page;

        // =====================================================================
        // 3. BOTTOM UILIB MOD TAB BAR (Touch-drag & wheel scrollable, zero scrollbar gutter)
        // =====================================================================
        let suppress_tab_click = false;

        ui.row(|ui| {
            let tab_scroll_x = ui.scroll_offset_x("qm_uilib_tabs_hscroll");
            if tab_scroll_x > 4.0
                && ui
                    .btn_with("qm_uilib_tabs_scroll_home", |ui| {
                        icons::icon(ui, icons::BACK, 18.0).fg(accent);
                    })
                    .w(crate::theme::text_box(30.0))
                    .h(crate::theme::text_box(44.0))
                    .center()
                    .bg(tint_alpha(accent, 42))
                    .hover_bg(tint_alpha(accent, 75))
                    .bd(1.0, tint_alpha(accent, 185))
                    .bevel_rounded(7.8, 1.3)
                    .clicked()
            {
                push_click(actions);
                ui.scroll_to_x("qm_uilib_tabs_hscroll", 0.0);
            }

            let hscroll_resp = ui
                .hscroll("qm_uilib_tabs_hscroll", |ui| {
                    let is_official = |t: &crate::state::UiLibTab| {
                        matches!(t.kind.as_str(), "home" | "players" | "props")
                    };

                    // Helper closure to render a UILib/official tab button.
                    let render_uilib_tab =
                        |ui: &mut UiContext,
                         state: &mut AppState,
                         actions: &mut Vec<UiAction>,
                         tab: &crate::state::UiLibTab| {
                        let tab_btn_id = format!("qm_tab_{}", tab.id);
                        let is_active = tab.active && active_ext_page.is_none();
                        let (bg, hover_bg, bd) = if is_active {
                            (
                                tint_alpha(accent, 52),
                                tint_alpha(accent, 75),
                                tint_alpha(accent, 210),
                            )
                        } else {
                            (tile_idle_bg(), tile_idle_hover(), border_subtle())
                        };

                        let icon_only = state.host.qm_mod_tabs_icon_only && !is_official(tab);
                        let mut tab_btn = ui.btn_with(&tab_btn_id, |ui| {
                            let mut row = ui.row(|ui| {
                                match tab.kind.as_str() {
                                    "home" => {
                                        icons::icon(ui, icons::HOME, 17.0)
                                            .fg(if is_active { accent } else { TEXT_MUTED });
                                    }
                                    "players" => {
                                        icons::icon(ui, icons::PLAYERS, 17.0)
                                            .fg(if is_active { accent } else { TEXT_MUTED });
                                    }
                                    "props" => {
                                        icons::icon(ui, icons::CUBE, 17.0)
                                            .fg(if is_active { accent } else { TEXT_MUTED });
                                    }
                                    "toolbox" => {
                                        icons::icon(ui, icons::TOOLBOX, 17.0)
                                            .fg(if is_active { accent } else { TEXT_MUTED });
                                    }
                                    _ => {
                                        if !tab.icon_key.is_empty() {
                                            if let Some(tex) =
                                                state.quick_textures.get(&tab.icon_key)
                                            {
                                                ui.img(tex)
                                                    .w(22.0)
                                                    .h(22.0)
                                                    .fit(ImageFit::Contain)
                                                    .bevel_rounded(3.9, 1.3);
                                            } else {
                                                icons::icon(ui, icons::PUZZLE, 17.0).fg(
                                                    if is_active { accent } else { TEXT_MUTED },
                                                );
                                            }
                                        } else {
                                            icons::icon(ui, icons::PUZZLE, 17.0)
                                                .fg(if is_active { accent } else { TEXT_MUTED });
                                        }
                                    }
                                }
                                if !icon_only {
                                    ui.txt(&tab.label)
                                        .sz(crate::theme::fs(11.5))
                                        .semibold()
                                        .fg(if is_active { TEXT_PRIMARY } else { TEXT_MUTED })
                                        .ellipsis()
                                        .grow(1.0);
                                }
                            });
                            if icon_only {
                                row = row.center().grow(1.0);
                            } else {
                                row = row.align_center().gap(7.0).grow(1.0);
                            }
                            let _ = row;
                        });
                        if icon_only {
                            tab_btn = tab_btn
                                .w(crate::theme::text_box(44.0))
                                .h(crate::theme::text_box(44.0))
                                .p(6.0);
                        } else {
                            tab_btn = tab_btn
                                .min_w(crate::theme::text_box(88.0))
                                .max_w(crate::theme::text_box(136.0))
                                .h(crate::theme::text_box(44.0))
                                .px(10.0)
                                .py(6.0);
                        }
                        let tab_resp = tab_btn
                            .bg(bg)
                            .hover_bg(hover_bg)
                            .bd(if is_active { 1.5 } else { 1.0 }, bd)
                            .bevel_rounded(7.8, 1.3);

                        if icon_only && tab_resp.hovered() {
                            let tip = if !tab.mod_name.is_empty() {
                                &tab.mod_name
                            } else {
                                &tab.label
                            };
                            if !tip.is_empty() {
                                state.quick.hovered_tooltip = tip.clone();
                            }
                        }

                        if tab.kind == "props" && tab_resp.double_clicked() && !suppress_tab_click {
                            state.quick.go_home_confirm = false;
                            push_click(actions);
                            crate::main_menu::nav_reset_to_tab(state, "props", actions);
                            actions.push(UiAction::OpenMainMenuPage {
                                page: String::from("props"),
                            });
                        } else if tab_resp.clicked() && !suppress_tab_click {
                            state.quick.go_home_confirm = false;
                            push_click(actions);
                            actions.push(UiAction::UiLibSelectTab {
                                tab_id: tab.id.clone(),
                            });
                        }
                    };

                    // 1. Official built-in tabs (Home, Players, Props)
                    for tab in tabs.iter().filter(|t| is_official(t)) {
                        render_uilib_tab(ui, state, actions, tab);
                    }

                    // 2. Lumenic Extension tabs (including shared "Extensions" hub) right after official tabs
                    for etab in &ext_tabs {
                        let tab_btn_id = format!("qm_ext_tab_{}_{}", etab.ext_id, etab.id);
                        let is_active = active_ext_page
                            .as_ref()
                            .is_some_and(|p| p.ext_id == etab.ext_id && p.tab_id == etab.id);
                        let (bg, hover_bg, bd) = if is_active {
                            (
                                tint_alpha(accent, 52),
                                tint_alpha(accent, 75),
                                tint_alpha(accent, 210),
                            )
                        } else {
                            (tile_idle_bg(), tile_idle_hover(), border_subtle())
                        };

                        let icon_only = state.host.qm_mod_tabs_icon_only;
                        let mut ext_btn = ui.btn_with(&tab_btn_id, |ui| {
                            let mut row = ui.row(|ui| {
                                if !etab.icon_key.is_empty() {
                                    if let Some(tex) = state.quick_textures.get(&etab.icon_key) {
                                        ui.img(tex)
                                            .w(20.0)
                                            .h(20.0)
                                            .fit(ImageFit::Contain)
                                            .bevel_rounded(3.9, 1.3);
                                    } else {
                                        let ic = if etab.icon.is_empty() { icons::PUZZLE } else { &etab.icon };
                                        icons::icon(ui, ic, 17.0)
                                            .fg(if is_active { accent } else { TEXT_MUTED });
                                    }
                                } else {
                                    let ic = if etab.icon.is_empty() { icons::PUZZLE } else { &etab.icon };
                                    icons::icon(ui, ic, 17.0)
                                        .fg(if is_active { accent } else { TEXT_MUTED });
                                }
                                if !icon_only {
                                    ui.txt(&etab.label)
                                        .sz(crate::theme::fs(11.5))
                                        .semibold()
                                        .fg(if is_active { TEXT_PRIMARY } else { TEXT_MUTED })
                                        .ellipsis()
                                        .grow(1.0);
                                }
                                if !etab.badge.is_empty() {
                                    ui.txt(&etab.badge)
                                        .sz(crate::theme::fs(9.5))
                                        .bold()
                                        .fg(Color::rgb(12, 18, 28))
                                        .bg(accent)
                                        .px(4.5)
                                        .py(1.0)
                                        .bevel_rounded(3.5, 1.0);
                                }
                            });
                            if icon_only && etab.badge.is_empty() {
                                row = row.center().grow(1.0);
                            } else {
                                row = row.align_center().gap(6.0).grow(1.0);
                            }
                            let _ = row;
                        });
                        if icon_only && etab.badge.is_empty() {
                            ext_btn = ext_btn
                                .w(crate::theme::text_box(44.0))
                                .h(crate::theme::text_box(44.0))
                                .p(6.0);
                        } else if icon_only {
                            ext_btn = ext_btn
                                .min_w(crate::theme::text_box(56.0))
                                .h(crate::theme::text_box(44.0))
                                .px(8.0)
                                .py(6.0);
                        } else {
                            ext_btn = ext_btn
                                .min_w(crate::theme::text_box(88.0))
                                .max_w(crate::theme::text_box(148.0))
                                .h(crate::theme::text_box(44.0))
                                .px(10.0)
                                .py(6.0);
                        }
                        let ext_resp = ext_btn
                            .bg(bg)
                            .hover_bg(hover_bg)
                            .bd(if is_active { 1.5 } else { 1.0 }, bd)
                            .bevel_rounded(7.8, 1.3);
                        if icon_only && ext_resp.hovered() && !etab.label.is_empty() {
                            state.quick.hovered_tooltip = etab.label.clone();
                        }
                        if ext_resp.clicked() && !suppress_tab_click
                        {
                            state.quick.go_home_confirm = false;
                            push_click(actions);
                            actions.push(UiAction::ExtSelectTab {
                                surface: String::from("quick"),
                                tab_id: format!("{}:{}", etab.ext_id, etab.id),
                            });
                        }
                    }

                    // 3. Remaining UILib mod tabs
                    for tab in tabs.iter().filter(|t| !is_official(t)) {
                        render_uilib_tab(ui, state, actions, tab);
                    }
                })
                .min_w(0.0)
                .grow(1.0)
                .gap(6.0)
                .scrollbar_w(0.0)
                .no_scrollbar();

            if let Some(tr) = hscroll_resp.prev_rect() {
                state.quick.tab_strip_rect = Some([tr.x, tr.y, tr.w, tr.h]);
            }
        })
        .align_center()
        .px(10.0)
        .py(8.0)
        .gap(8.0)
        .bg(card_bg())
        .bd(1.0, border_subtle())
        .bevel_rounded(10.1, 2.0)
        .shadow(12.0, SHADOW_CARD);
        state.extensions.qm_active_page = active_ext_page;
        state.extensions.qm_tabs = ext_tabs;
        state.host.uilib.tabs = tabs;
    })
            .w_frac(1.0)
            .h_frac(1.0)
            .px(24.0)
            .py(44.0)
            .gap(10.0);

            if !state.host.disclaimer_accepted {
                crate::settings_view::render_first_use_disclaimer_modal(
                    ui, state, accent, actions, "qm",
                );
            } else if state.host.show_whats_new {
                crate::settings_view::render_whats_new_modal(
                    ui, state, accent, actions, "qm",
                );
            } else if state.host.update_status.show_modal {
                crate::settings_view::render_mod_update_modal(
                    ui, state, accent, actions, "qm",
                );
            } else if let Some(alert) = state.host.system_alert.take() {
                crate::settings_view::render_system_alert_modal(
                    ui, state, &alert, accent, actions, "qm",
                );
                state.host.system_alert = Some(alert);
            } else if let Some(ext_modal) = state.extensions.qm_modal.take() {
                crate::ext_view::render_ext_modal(ui, state, "quick", &ext_modal, accent, actions);
                state.extensions.qm_modal = Some(ext_modal);
            } else if let Some(modal) = state.host.uilib.modal.take() {
                render_uilib_modal(ui, state, &modal, accent, actions);
                state.host.uilib.modal = Some(modal);
            } else if let Some(prompt) = state.host.content.group_mod_prompt.take() {
                state.quick.qm_world_modal = None;
                render_group_mod_prompt_modal(ui, &prompt, accent, actions, "qm");
                state.host.content.group_mod_prompt = Some(prompt);
            } else if let Some((world_id, world_name)) = state.quick.qm_world_modal.take() {
                render_qm_world_instance_modal(ui, state, &world_id, &world_name, accent, actions);
                state.quick.qm_world_modal = Some((world_id, world_name));
            } else if state.quick.show_info_modal {
                render_system_info_modal(ui, state, accent, actions);
            }
            render_uilib_toast(ui, &state.host.uilib.toast, accent);
        })
        .w_frac(1.0)
        .h_frac(1.0);

    if let Some(r) = outer_resp.prev_rect() {
        state.quick.qm_root_rect = Some([r.x, r.y, r.w, r.h]);
    }

    if !state.quick.avatar_preview_rendered {
        state.quick.avatar_preview_rect = None;
        if state.quick.avatar_preview_dragging {
            state.quick.avatar_preview_dragging = false;
            actions.push(UiAction::AvatarPreviewRotate {
                delta: 0.0,
                is_dragging: false,
            });
        }
        if state.quick.last_avatar_preview_sync.take().is_some() {
            state.quick.avatar_preview_target.clear();
            actions.push(UiAction::SyncAvatarPreview {
                surface_id: 1,
                active: false,
                target: String::new(),
                x: 0.0,
                y: 0.0,
                w: 0.0,
                h: 0.0,
            });
        }
    }

    if !state.quick.hovered_tooltip.is_empty() {
        let tip = state.quick.hovered_tooltip.clone();
        crate::main_menu::render_floating_tooltip(ui, &tip, accent);
    }
    state.quick.hovered_tooltip.clear();
}

fn info_stat_row(ui: &mut UiContext, label: &str, value: &str, value_col: Color) {
    ui.row(|ui| {
        ui.txt(label).sz(crate::theme::fs(11.5)).fg(TEXT_MUTED).grow(1.0);
        ui.txt(value)
            .sz(crate::theme::fs(11.5))
            .semibold()
            .align(TextAlign::Right)
            .fg(value_col);
    })
    .align_center()
    .px(9.0)
    .py(6.0)
    .bg(card_inner_bg())
    .bd(1.0, border_subtle())
    .bevel_rounded(5.0, 1.3);
}

fn render_system_info_modal(
    ui: &mut UiContext,
    state: &mut AppState,
    accent: Color,
    actions: &mut Vec<UiAction>,
) {
    ui.overlay(|ui| {
        ui.zstack(|ui| {
            if crate::theme::modal_scrim(ui, "qm_info_modal_scrim_fx", "qm_info_modal_scrim") {
                push_click(actions);
                state.quick.show_info_modal = false;
            }

            ui.col(|ui| {
                let mut card = ui
                    .col(|ui| {
                        ui.row(|ui| {
                            ui.txt("System & Telemetry")
                                .sz(crate::theme::fs(15.0))
                                .bold()
                                .fg(TEXT_PRIMARY)
                                .grow(1.0);
                            if ui
                                .btn_with("qm_info_modal_close", |ui| {
                                    icons::icon(ui, icons::CLOSE, 14.0).fg(TEXT_MUTED);
                                })
                                .p(5.0)
                                .bg(Color::TRANSPARENT)
                                .hover_bg(Color::rgba(239, 68, 68, 80))
                                .bevel_rounded(4.5, 1.3)
                                .clicked()
                            {
                                push_click(actions);
                                state.quick.show_info_modal = false;
                            }
                        })
                        .align_center()
                        .gap(8.0);

                        let fps_col = if state.host.fps >= 60 {
                            EMERALD
                        } else if state.host.fps >= 36 {
                            AMBER
                        } else {
                            RED
                        };
                        let fps_val = format!("{} FPS", state.host.fps.max(0));
                        info_stat_row(ui, "Framerate", &fps_val, fps_col);

                        let gping_val = format!("{} ms", state.host.game_ping.max(0));
                        info_stat_row(ui, "Game Ping", &gping_val, TEXT_PRIMARY);

                        let vping_val = format!("{} ms", state.host.voice_ping.max(0));
                        info_stat_row(ui, "Voice Ping", &vping_val, TEXT_PRIMARY);

                        let mode_val = if state.host.is_vr { "VR" } else { "Desktop" };
                        info_stat_row(ui, "Mode", mode_val, accent);

                        let world_val = if state.host.world_name.is_empty() {
                            "Local World"
                        } else {
                            &state.host.world_name
                        };
                        info_stat_row(ui, "World", world_val, TEXT_PRIMARY);

                        let flight_val = if !state.host.flight_allowed {
                            "Locked by World"
                        } else if state.host.is_flying {
                            "Active"
                        } else {
                            "Allowed"
                        };
                        info_stat_row(
                            ui,
                            "Flight",
                            flight_val,
                            if state.host.flight_allowed {
                                EMERALD
                            } else {
                                AMBER
                            },
                        );

                        let fbt_val = if !state.host.fbt_available {
                            "No Trackers"
                        } else if state.host.fbt_enabled {
                            "Active"
                        } else {
                            "Ready (Off)"
                        };
                        info_stat_row(ui, "Full Body Tracking", fbt_val, TEXT_PRIMARY);

                        if state.host.perf_logging {
                            let timing_val = format!(
                                "S {:.1} | R {:.1} | U {:.1} | B {:.1} ms",
                                state.host.qm_step_ms,
                                state.host.qm_readback_ms,
                                state.host.qm_upload_ms,
                                state.host.qm_blit_ms
                            );
                            info_stat_row(ui, "Frame Cost", &timing_val, EMERALD);
                        }
                    })
                    .w(380.0)
                    .p(16.0)
                    .gap(7.0)
                    .bg(Color::rgba(15, 19, 28, 248))
                    .bd(1.5, accent)
                    .bevel_rounded(12.0, 2.0)
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

fn render_home_center(
    ui: &mut UiContext,
    state: &mut AppState,
    accent: Color,
    actions: &mut Vec<UiAction>,
) {
    ui.row(|ui| {
        // =====================================================================
        // LEFT COLUMN: COMPACT NOTIFICATIONS (TOP) + COMPACT QUICK ACTIONS (BOTTOM)
        // =====================================================================
        ui.col(|ui| {
            // --- TOP LEFT: NOTIFICATIONS CARD (section: qm.home.notifications) ---
            crate::ext_view::render_section(ui, state, "quick", "qm.home.notifications", accent, actions, |ui, state, actions| {
            ui.col(|ui| {
                ui.row(|ui| {
                    icons::icon(ui, icons::BELL, 14.0).fg(accent);
                    ui.txt("Notifications")
                        .sz(crate::theme::fs(12.0))
                        .semibold()
                        .fg(TEXT_PRIMARY);

                    let count = state.host.notifications.len();
                    if count > 0 {
                        let badge = format!("{count}");
                        ui.txt(&badge)
                            .sz(crate::theme::fs(10.0))
                            .bold()
                            .fg(Color::rgb(12, 18, 28))
                            .bg(accent)
                            .px(5.5)
                            .py(1.5)
                            .bevel_rounded(3.9, 1.3);
                    } else {
                        ui.txt("All caught up")
                            .sz(crate::theme::fs(11.0))
                            .fg(TEXT_DIM)
                            .grow(1.0);
                    }
                })
                .align_center()
                .gap(6.0);

                if !state.host.notifications.is_empty() {
                    ui.scroll("qm_notifications_scroll", |ui| {
                        for notif in &state.host.notifications {
                            ui.row(|ui| {
                                ui.col(|ui| {
                                    ui.txt(&notif.title)
                                        .sz(crate::theme::fs(11.5))
                                        .semibold()
                                        .fg(TEXT_PRIMARY)
                                        .ellipsis();
                                    if !notif.body.is_empty() {
                                        ui.txt(&notif.body)
                                            .sz(crate::theme::fs(10.0))
                                            .fg(TEXT_MUTED)
                                            .ellipsis();
                                    }
                                })
                                .gap(1.5)
                                .grow(1.0);

                                if !notif.buttons.is_empty() {
                                    ui.row(|ui| {
                                        for btn in &notif.buttons {
                                            let btn_bg = if btn.is_accept {
                                                tint_alpha(EMERALD, 55)
                                            } else if btn.is_decline {
                                                tint_alpha(RED, 55)
                                            } else {
                                                tile_idle_bg()
                                            };
                                            let btn_bd = if btn.is_accept {
                                                EMERALD
                                            } else if btn.is_decline {
                                                RED
                                            } else {
                                                border_subtle()
                                            };
                                            let btn_id = format!("nbtn_{}", btn.id);
                                            if ui
                                                .btn_with(&btn_id, |ui| {
                                                    ui.txt(&btn.text)
                                                        .sz(crate::theme::fs(10.0))
                                                        .semibold()
                                                        .fg(TEXT_PRIMARY);
                                                })
                                                .px(7.0)
                                                .py(3.5)
                                                .bg(btn_bg)
                                                .hover_bg(tint_alpha(accent, 80))
                                                .bd(1.0, btn_bd)
                                                .bevel_rounded(3.9, 1.3)
                                                .clicked()
                                            {
                                                push_click(actions);
                                                actions.push(UiAction::NotificationButtonClick {
                                                    button_id: btn.id.clone(),
                                                });
                                            }
                                        }
                                    })
                                    .align_center()
                                    .gap(4.0);
                                }

                                let dismiss_id = format!("dismiss_{}", notif.id);
                                if ui
                                    .btn_with(&dismiss_id, |ui| {
                                        icons::icon(ui, icons::CLOSE, 11.0).fg(TEXT_MUTED);
                                    })
                                    .p(3.5)
                                    .bg(Color::TRANSPARENT)
                                    .hover_bg(Color::rgba(239, 68, 68, 70))
                                    .bevel_rounded(3.9, 1.3)
                                    .clicked()
                                {
                                    push_click(actions);
                                    actions.push(UiAction::NotificationDismiss {
                                        message_id: notif.id.clone(),
                                    });
                                }
                            })
                            .align_center()
                            .px(8.0)
                            .py(5.5)
                            .gap(6.0)
                            .bg(card_inner_bg())
                            .bd(1.0, border_subtle())
                            .bevel_rounded(6.2, 1.3);
                        }
                    })
                    .h(crate::theme::text_box(88.0))
                    .gap(5.0)
                    .p(3.0)
                    .bg(Color::rgba(10, 14, 22, 115))
                    .bevel_rounded(7.0, 1.3)
                    .no_scrollbar();
                } else {
                    ui.col(|ui| {
                        ui.txt("No active notifications")
                            .sz(crate::theme::fs(11.0))
                            .fg(TEXT_DIM);
                    })
                    .center()
                    .h(crate::theme::text_box(88.0))
                    .w_frac(1.0)
                    .bg(Color::rgba(10, 14, 22, 90))
                    .bevel_rounded(7.0, 1.3);
                }
            })
            .px(10.0)
            .py(8.0)
            .gap(6.0)
            .bg(card_bg())
            .bd(1.0, border_subtle())
            .bevel_rounded(10.1, 2.0)
            .shadow(12.0, SHADOW_CARD);
            });

            // --- BOTTOM LEFT: COMPACT QUICK ACTIONS (section: qm.home.actions) ---
            crate::ext_view::render_section(ui, state, "quick", "qm.home.actions", accent, actions, |ui, state, actions| {
            ui.col(|ui| {
                const QM_DEFAULT_TILES: [&str; 9] = [
                    "mute", "flight", "seated", "camera", "mirror", "respawn", "home", "chat",
                    "fbt",
                ];
                let mut tile_order = crate::state::resolve_tile_order(
                    &state.host.qm_quick_actions_order,
                    &QM_DEFAULT_TILES,
                );
                let reorder_mode = state.quick.qm_reorder_mode;

                ui.row(|ui| {
                    icons::icon(ui, icons::BOLT, 14.0).fg(accent);
                    ui.txt("Quick Actions")
                        .sz(crate::theme::fs(12.0))
                        .semibold()
                        .fg(TEXT_PRIMARY)
                        .grow(1.0);

                    if reorder_mode {
                        let hint = if state.quick.qm_reorder_selected.is_some() {
                            "Tap target"
                        } else {
                            "Tap to swap"
                        };
                        ui.txt(hint).sz(crate::theme::fs(9.5)).fg(accent);

                        if ui
                            .btn_with("qm_act_reset_order", |ui| {
                                ui.txt("Reset").sz(crate::theme::fs(9.5)).semibold().fg(TEXT_MUTED);
                            })
                            .px(7.0)
                            .py(3.0)
                            .bg(tile_idle_bg())
                            .hover_bg(tile_idle_hover())
                            .bd(1.0, border_subtle())
                            .bevel_rounded(4.5, 1.0)
                            .clicked()
                        {
                            push_click(actions);
                            state.quick.qm_reorder_selected = None;
                            state.host.qm_quick_actions_order.clear();
                            actions.push(UiAction::SetTileOrder {
                                target: String::from("qm_quick_actions"),
                                order: Vec::new(),
                            });
                        }
                    }

                    if ui
                        .btn_with("qm_act_reorder_toggle", |ui| {
                            ui.txt(if reorder_mode { "Done" } else { "Reorder" })
                                .sz(crate::theme::fs(9.5))
                                .semibold()
                                .fg(if reorder_mode { TEXT_PRIMARY } else { TEXT_MUTED });
                        })
                        .px(8.0)
                        .py(3.0)
                        .bg(if reorder_mode {
                            tint_alpha(accent, 52)
                        } else {
                            tile_idle_bg()
                        })
                        .hover_bg(tile_idle_hover())
                        .bd(
                            1.0,
                            if reorder_mode {
                                tint_alpha(accent, 195)
                            } else {
                                border_subtle()
                            },
                        )
                        .bevel_rounded(4.5, 1.0)
                        .clicked()
                    {
                        push_click(actions);
                        state.quick.qm_reorder_mode = !reorder_mode;
                        state.quick.qm_reorder_selected = None;
                    }
                })
                .align_center()
                .gap(6.0);

                let mut swapped_pair: Option<(usize, usize)> = None;
                let cols = crate::theme::qm_action_cols();
                let visible_indices: Vec<usize> = (0..tile_order.len())
                    .filter(|&i| {
                        reorder_mode
                            || !state
                                .extensions
                                .hidden_tiles
                                .iter()
                                .any(|h| h == &tile_order[i])
                    })
                    .collect();
                for chunk in visible_indices.chunks(cols) {
                    ui.row(|ui| {
                        for &slot_idx in chunk {
                            let tile_key = tile_order[slot_idx].as_str();
                            let is_selected =
                                reorder_mode && state.quick.qm_reorder_selected == Some(slot_idx);

                            let (btn_id, icon, label, status, active, enabled, tile_accent) =
                                match tile_key {
                                    "mute" => {
                                        let (ic, st, col) = if state.host.is_muted {
                                            (icons::MIC_OFF, "Muted", RED)
                                        } else {
                                            (icons::MIC, "Live", EMERALD)
                                        };
                                        ("qm_tile_mute", ic, "Microphone", st, true, true, col)
                                    }
                                    "flight" => {
                                        let st = if !state.host.flight_allowed {
                                            "Locked"
                                        } else if state.host.is_flying {
                                            "Active"
                                        } else {
                                            "Off"
                                        };
                                        (
                                            "qm_tile_flight",
                                            icons::FLIGHT,
                                            "Flight",
                                            st,
                                            state.host.is_flying,
                                            state.host.flight_allowed,
                                            accent,
                                        )
                                    }
                                    "seated" => {
                                        let use_recal =
                                            state.host.fbt_enabled && state.host.fbt_available;
                                        let (lbl, st, act) = if use_recal {
                                            ("Calibrate", "FBT", false)
                                        } else if state.host.is_seated {
                                            ("Seated", "Active", true)
                                        } else {
                                            ("Seated", "Standing", false)
                                        };
                                        ("qm_tile_seated", icons::SEATED, lbl, st, act, true, accent)
                                    }
                                    "camera" => (
                                        "qm_tile_camera",
                                        icons::CAMERA,
                                        "Camera",
                                        if state.host.camera_open { "Open" } else { "Off" },
                                        state.host.camera_open,
                                        true,
                                        accent,
                                    ),
                                    "mirror" => (
                                        "qm_tile_mirror",
                                        icons::MIRROR,
                                        "Mirror",
                                        if state.host.mirror_open { "Active" } else { "Off" },
                                        state.host.mirror_open,
                                        true,
                                        accent,
                                    ),
                                    "respawn" => (
                                        "qm_tile_respawn",
                                        icons::RESPAWN,
                                        "Respawn",
                                        "Reset Pos",
                                        false,
                                        true,
                                        accent,
                                    ),
                                    "home" => {
                                        if state.quick.go_home_confirm {
                                            (
                                                "qm_tile_home",
                                                icons::HOME,
                                                "Confirm Home?",
                                                "Tap again",
                                                true,
                                                true,
                                                AMBER,
                                            )
                                        } else {
                                            (
                                                "qm_tile_home",
                                                icons::HOME,
                                                "Go Home",
                                                "Leave World",
                                                false,
                                                true,
                                                accent,
                                            )
                                        }
                                    }
                                    "chat" => (
                                        "qm_tile_chat",
                                        icons::CHAT,
                                        "Text Chat",
                                        "Keyboard",
                                        false,
                                        true,
                                        accent,
                                    ),
                                    _ => {
                                        let st = if !state.host.fbt_available {
                                            "No Trackers"
                                        } else if state.host.fbt_enabled {
                                            "Enabled"
                                        } else {
                                            "Disabled"
                                        };
                                        (
                                            "qm_tile_fbt",
                                            icons::FBT,
                                            "Full Body",
                                            st,
                                            state.host.fbt_enabled && state.host.fbt_available,
                                            state.host.fbt_available,
                                            accent,
                                        )
                                    }
                                };

                            let eff_status = if is_selected {
                                "Selected"
                            } else if reorder_mode {
                                "Swap"
                            } else {
                                status
                            };
                            let eff_active = if reorder_mode { is_selected } else { active };
                            let eff_enabled = if reorder_mode { true } else { enabled };
                            let eff_accent = if is_selected { AMBER } else { tile_accent };

                            if action_tile(
                                ui,
                                btn_id,
                                icon,
                                label,
                                eff_status,
                                eff_active,
                                eff_enabled,
                                eff_accent,
                            ) {
                                if reorder_mode {
                                    push_click(actions);
                                    if let Some(prev) = state.quick.qm_reorder_selected {
                                        if prev == slot_idx {
                                            state.quick.qm_reorder_selected = None;
                                        } else {
                                            swapped_pair = Some((prev, slot_idx));
                                            state.quick.qm_reorder_selected = None;
                                        }
                                    } else {
                                        state.quick.qm_reorder_selected = Some(slot_idx);
                                    }
                                } else {
                                    match tile_key {
                                        "mute" => {
                                            state.quick.go_home_confirm = false;
                                            actions.push(UiAction::ToggleMute);
                                        }
                                        "flight" => {
                                            state.quick.go_home_confirm = false;
                                            push_click(actions);
                                            actions.push(UiAction::ToggleFlight);
                                        }
                                        "seated" => {
                                            state.quick.go_home_confirm = false;
                                            push_click(actions);
                                            actions.push(UiAction::ToggleSeatedOrRecalibrate);
                                        }
                                        "camera" => {
                                            state.quick.go_home_confirm = false;
                                            push_click(actions);
                                            actions.push(UiAction::ToggleCamera);
                                        }
                                        "mirror" => {
                                            state.quick.go_home_confirm = false;
                                            push_click(actions);
                                            actions.push(UiAction::ToggleMirror);
                                        }
                                        "respawn" => {
                                            state.quick.go_home_confirm = false;
                                            push_click(actions);
                                            actions.push(UiAction::Respawn);
                                        }
                                        "home" => {
                                            if state.quick.go_home_confirm {
                                                state.quick.go_home_confirm = false;
                                                push_click(actions);
                                                actions.push(UiAction::GoHome);
                                            } else {
                                                state.quick.go_home_confirm = true;
                                                actions.push(UiAction::PlayUiSound {
                                                    sound: String::from("Select"),
                                                });
                                            }
                                        }
                                        "chat" => {
                                            state.quick.go_home_confirm = false;
                                            push_click(actions);
                                            actions.push(UiAction::OpenTextChat);
                                        }
                                        _ => {
                                            state.quick.go_home_confirm = false;
                                            push_click(actions);
                                            actions.push(UiAction::ToggleFullBody);
                                        }
                                    }
                                }
                            }
                        }
                    })
                    .grow(1.0)
                    .gap(7.0);
                }

                if let Some((a, b)) = swapped_pair {
                    tile_order.swap(a, b);
                    state.host.qm_quick_actions_order = tile_order.clone();
                    actions.push(UiAction::SetTileOrder {
                        target: String::from("qm_quick_actions"),
                        order: tile_order,
                    });
                }
            })
            .grow(1.0)
            .p(10.0)
            .gap(7.0)
            .bg(card_bg())
            .bd(1.0, border_subtle())
            .bevel_rounded(10.1, 2.0)
            .shadow(14.0, SHADOW_CARD);
            });

            // --- SEPARATE BOTTOM LEFT CARD: MAIN MENU SHORTCUTS ---
            ui.col(|ui| {
                ui.row(|ui| {
                    icons::icon(ui, icons::HOME, 14.0).fg(accent);
                    ui.txt("Main Menu")
                        .sz(crate::theme::fs(12.0))
                        .semibold()
                        .fg(TEXT_PRIMARY);
                })
                .align_center()
                .gap(6.0);

                ui.row(|ui| {
                    for (id, icon, label, page) in [
                        ("qm_nav_home", icons::HOME, "Home", "home"),
                        ("qm_nav_worlds", icons::WORLD, "Worlds", "worlds"),
                        ("qm_nav_avatars", icons::AVATAR, "Avatars", "avatars"),
                    ] {
                        if ui
                            .btn_with(id, |ui| {
                                ui.row(|ui| {
                                    icons::icon(ui, icon, 15.0).fg(accent);
                                    ui.txt(label)
                                        .sz(crate::theme::fs(11.0))
                                        .semibold()
                                        .fg(TEXT_PRIMARY);
                                })
                                .center()
                                .gap(5.0)
                                .grow(1.0);
                            })
                            .bg(tile_idle_bg())
                            .hover_bg(tint_alpha(accent, 64))
                            .bd(1.0, border_subtle())
                            .min_h(crate::theme::text_box(34.0))
                            .px(8.0)
                            .py(4.0)
                            .bevel_rounded(6.5, 1.3)
                            .grow(1.0)
                            .clicked()
                        {
                            state.quick.go_home_confirm = false;
                            push_click(actions);
                            crate::main_menu::nav_reset_to_tab(state, page, actions);
                            actions.push(UiAction::OpenMainMenuPage {
                                page: String::from(page),
                            });
                        }
                    }
                })
                .gap(6.0);

                ui.row(|ui| {
                    for (id, icon, label, page) in [
                        ("qm_nav_friends", icons::PLAYERS, "Friends", "friends"),
                        ("qm_nav_instance", icons::INSTANCE, "Instance", "currentInstance"),
                        ("qm_nav_settings", icons::SETTINGS, "Settings", "settings"),
                    ] {
                        if ui
                            .btn_with(id, |ui| {
                                ui.row(|ui| {
                                    icons::icon(ui, icon, 15.0).fg(accent);
                                    ui.txt(label)
                                        .sz(crate::theme::fs(11.0))
                                        .semibold()
                                        .fg(TEXT_PRIMARY);
                                })
                                .center()
                                .gap(5.0)
                                .grow(1.0);
                            })
                            .bg(tile_idle_bg())
                            .hover_bg(tint_alpha(accent, 64))
                            .bd(1.0, border_subtle())
                            .min_h(crate::theme::text_box(34.0))
                            .px(8.0)
                            .py(4.0)
                            .bevel_rounded(6.5, 1.3)
                            .grow(1.0)
                            .clicked()
                        {
                            state.quick.go_home_confirm = false;
                            push_click(actions);
                            crate::main_menu::nav_reset_to_tab(state, page, actions);
                            actions.push(UiAction::OpenMainMenuPage {
                                page: String::from(page),
                            });
                        }
                    }
                })
                .gap(6.0);
            })
            .p(10.0)
            .gap(6.0)
            .bg(card_bg())
            .bd(1.0, border_subtle())
            .bevel_rounded(10.1, 2.0)
            .shadow(14.0, SHADOW_CARD);
        })
        .grow(1.22)
        .gap(9.0);

        // =====================================================================
        // RIGHT COLUMN: GPU AVATAR TURNTABLE + AVATAR SETTINGS (TOP) + QUICK AUDIO (BOTTOM)
        // =====================================================================
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

        ui.col(|ui| {
            // Top Right Split: Thinner 3D Avatar Turntable (Left, ~55%) + Pinned Quick-Use List (Right, ~45%)
            ui.row(|ui| {
                // Top-Left of Right Column: GPU-composited AvatarSettingsPreview turntable + Avatar Settings shortcut
                ui.col(|ui| {
                    ui.row(|ui| {
                        icons::icon(ui, icons::AVATAR, 14.0).fg(accent);
                        ui.txt("Avatar")
                            .sz(crate::theme::fs(12.0))
                            .semibold()
                            .fg(TEXT_PRIMARY)
                            .grow(1.0);
                    })
                    .align_center()
                    .gap(6.0);

                    let qm_modal_open = state.quick.qm_world_modal.is_some()
                        || state.quick.show_info_modal
                        || state.host.content.group_mod_prompt.is_some()
                        || state.host.uilib.modal.is_some()
                        || state.host.show_whats_new
                        || state.host.update_status.show_modal
                        || state.host.system_alert.is_some();
                    crate::avatar_settings_view::render_avatar_turntable_stage(
                        ui,
                        state,
                        1,
                        "local",
                        "qm_avatar_turntable_drag",
                        if qm_modal_open { "Modal active" } else { "" },
                        accent,
                        actions,
                    );

                    if ui
                        .btn_with("qm_btn_avatar_settings", |ui| {
                            ui.row(|ui| {
                                icons::icon(ui, icons::SLIDERS, 14.0).fg(accent);
                                ui.txt("Avatar Settings")
                                    .sz(crate::theme::fs(11.0))
                                    .semibold()
                                    .fg(TEXT_PRIMARY);
                            })
                            .center()
                            .gap(5.0)
                            .grow(1.0);
                        })
                        .w_frac(1.0)
                        .py(6.5)
                        .bg(tint_alpha(accent, 38))
                        .hover_bg(tint_alpha(accent, 68))
                        .bd(1.0, tint_alpha(accent, 165))
                        .bevel_rounded(6.5, 1.3)
                        .clicked()
                    {
                        state.quick.go_home_confirm = false;
                        state.main_menu.active_tab = String::from("avatarSettings");
                        push_click(actions);
                        actions.push(UiAction::OpenMainMenuPage {
                            page: String::from("avatarSettings"),
                        });
                    }
                })
                .grow(1.15)
                .h_frac(1.0)
                .p(9.0)
                .gap(6.0)
                .bg(card_bg())
                .bd(1.0, border_subtle())
                .bevel_rounded(10.9, 2.0)
                .shadow(14.0, SHADOW_CARD);

                // Top-Right: Pinned Avatars / Worlds / Props compact quick-use card
                ui.col(|ui| {
                    let edit_mode = state.quick.qm_pinned_edit;
                    ui.row(|ui| {
                        icons::icon(ui, icons::PIN, 13.5).fg(accent);
                        ui.txt("Pinned")
                            .sz(crate::theme::fs(11.5))
                            .semibold()
                            .fg(TEXT_PRIMARY)
                            .grow(1.0);

                        if ui
                            .btn_with("qm_pin_edit_toggle", |ui| {
                                ui.txt(if edit_mode { "Done" } else { "Edit" })
                                    .sz(crate::theme::fs(9.5))
                                    .semibold()
                                    .fg(if edit_mode { TEXT_PRIMARY } else { TEXT_MUTED });
                            })
                            .px(7.0)
                            .py(3.0)
                            .bg(if edit_mode {
                                tint_alpha(accent, 50)
                            } else {
                                tile_idle_bg()
                            })
                            .hover_bg(tile_idle_hover())
                            .bd(
                                1.0,
                                if edit_mode {
                                    tint_alpha(accent, 190)
                                } else {
                                    border_subtle()
                                },
                            )
                            .bevel_rounded(4.5, 1.0)
                            .clicked()
                        {
                            push_click(actions);
                            state.quick.qm_pinned_edit = !edit_mode;
                        }
                    })
                    .align_center()
                    .gap(5.0);

                    // Compact icon-only filter tabs: All | Avatars | Worlds | Props
                    ui.row(|ui| {
                        for (idx, ic) in [
                            (0usize, icons::APPS),
                            (1usize, icons::AVATAR),
                            (2usize, icons::WORLD),
                            (3usize, icons::CUBE),
                        ] {
                            let active = state.quick.qm_pinned_filter == idx;
                            if ui
                                .btn_with(&format!("qm_pin_tab_{idx}"), |ui| {
                                    icons::icon(ui, ic, 13.0)
                                        .fg(if active { accent } else { TEXT_MUTED });
                                })
                                .grow(1.0)
                                .py(4.0)
                                .bg(if active {
                                    tint_alpha(accent, 44)
                                } else {
                                    card_inner_bg()
                                })
                                .hover_bg(tile_idle_hover())
                                .bd(
                                    1.0,
                                    if active {
                                        tint_alpha(accent, 180)
                                    } else {
                                        border_subtle()
                                    },
                                )
                                .bevel_rounded(5.0, 1.2)
                                .clicked()
                            {
                                push_click(actions);
                                state.quick.qm_pinned_filter = idx;
                            }
                        }
                    })
                    .gap(4.0);

                    let filter_idx = state.quick.qm_pinned_filter;
                    let pinned: Vec<crate::state::PinnedContentItem> = state
                        .host
                        .content
                        .pinned_items
                        .iter()
                        .filter(|p| match filter_idx {
                            1 => p.kind == "avatar",
                            2 => p.kind == "world",
                            3 => p.kind == "prop",
                            _ => true,
                        })
                        .cloned()
                        .collect();

                    if pinned.is_empty() {
                        ui.col(|ui| {
                            icons::icon(ui, icons::PIN, 20.0).fg(TEXT_DIM);
                            ui.txt(if state.host.content.pinned_items.is_empty() {
                                "Pin items from\nMain Menu"
                            } else {
                                "None in filter"
                            })
                            .sz(crate::theme::fs(10.0))
                            .align(TextAlign::Center)
                            .fg(TEXT_DIM)
                            .wrap();
                        })
                        .center()
                        .grow(1.0)
                        .gap(4.0);
                    } else {
                        ui.scroll("qm_pinned_list", |ui| {
                            for p in &pinned {
                                let kind_ic = match p.kind.as_str() {
                                    "avatar" => icons::AVATAR,
                                    "world" => icons::WORLD,
                                    _ => icons::CUBE,
                                };
                                ui.row(|ui| {
                                    if ui
                                        .btn_with(&format!("qm_pin_item_{}_{}", p.kind, p.id), |ui| {
                                            ui.row(|ui| {
                                                let has_tex = if let Some(tex) =
                                                    state.quick_textures.get(&p.image_key)
                                                {
                                                    ui.img(tex)
                                                        .w(34.0)
                                                        .h(34.0)
                                                        .fit(ImageFit::Cover)
                                                        .bevel_rounded(5.0, 1.1);
                                                    true
                                                } else {
                                                    ui.col(|ui| {
                                                        icons::icon(ui, kind_ic, 16.0).fg(accent);
                                                    })
                                                    .center()
                                                    .w(34.0)
                                                    .h(34.0)
                                                    .bg(card_bg())
                                                    .bevel_rounded(5.0, 1.1);
                                                    false
                                                };

                                                if has_tex {
                                                    icons::icon(ui, kind_ic, 11.0).fg(TEXT_DIM);
                                                }
                                                ui.txt(&p.name)
                                                    .sz(crate::theme::fs(11.0))
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
                                        .py(5.5)
                                        .bg(card_inner_bg())
                                        .hover_bg(tile_idle_hover())
                                        .bd(1.0, border_subtle())
                                        .bevel_rounded(5.5, 1.2)
                                        .clicked()
                                        && !edit_mode
                                    {
                                        state.quick.go_home_confirm = false;
                                        push_click(actions);
                                        match p.kind.as_str() {
                                            "prop" => {
                                                actions.push(UiAction::ContentSpawnProp {
                                                    prop_id: p.id.clone(),
                                                });
                                            }
                                            "avatar" => {
                                                actions.push(UiAction::ContentWearAvatar {
                                                    avatar_id: p.id.clone(),
                                                });
                                            }
                                            _ => {
                                                state.quick.qm_world_modal =
                                                    Some((p.id.clone(), p.name.clone()));
                                            }
                                        }
                                    }

                                    if edit_mode
                                        && ui
                                            .btn_with(
                                                &format!("qm_unpin_{}_{}", p.kind, p.id),
                                                |ui| {
                                                    icons::icon(ui, icons::CLOSE, 12.0).fg(RED);
                                                },
                                            )
                                            .p(5.0)
                                            .bg(tint_alpha(RED, 28))
                                            .hover_bg(tint_alpha(RED, 60))
                                            .bd(1.0, tint_alpha(RED, 140))
                                            .bevel_rounded(5.0, 1.2)
                                            .clicked()
                                    {
                                        push_click(actions);
                                        actions.push(UiAction::ContentUnpin {
                                            kind: p.kind.clone(),
                                            id: p.id.clone(),
                                        });
                                    }
                                })
                                .align_center()
                                .gap(4.0);
                            }
                        })
                        .grow(1.0)
                        .gap(4.5)
                        .no_scrollbar();
                    }
                })
                .grow(0.95)
                .h_frac(1.0)
                .p(9.0)
                .gap(6.0)
                .bg(card_bg())
                .bd(1.0, border_subtle())
                .bevel_rounded(10.9, 2.0)
                .shadow(14.0, SHADOW_CARD);
            })
            .grow(1.0)
            .w_frac(1.0)
            .gap(8.0);

            // Bottom Right Card: Quick Audio Sliders (Master + Voice + Video Player)
            ui.col(|ui| {
                ui.row(|ui| {
                    icons::icon(ui, icons::VOLUME, 15.0).fg(accent);
                    ui.txt("Quick Audio")
                        .sz(crate::theme::fs(12.5))
                        .semibold()
                        .fg(TEXT_PRIMARY)
                        .grow(1.0);

                    if ui
                        .btn_with("qm_btn_audio_settings", |ui| {
                            ui.row(|ui| {
                                icons::icon(ui, icons::SETTINGS, 12.0).fg(accent);
                                ui.txt("Open in Menu")
                                    .sz(crate::theme::fs(10.0))
                                    .semibold()
                                    .fg(TEXT_PRIMARY);
                            })
                            .center()
                            .gap(4.0);
                        })
                        .px(7.5)
                        .py(3.5)
                        .bg(tile_idle_bg())
                        .hover_bg(tint_alpha(accent, 64))
                        .bd(1.0, border_subtle())
                        .bevel_rounded(5.0, 1.2)
                        .clicked()
                    {
                        state.quick.go_home_confirm = false;
                        state.main_menu.active_tab = String::from("settings");
                        state.main_menu.settings_tab = 2; // Audio
                        push_click(actions);
                        actions.push(UiAction::OpenMainMenuPage {
                            page: String::from("settings"),
                        });
                    }
                })
                .align_center()
                .gap(6.0);

                let master_before = state.quick.master_slider.round() as i32;
                let m_resp = ui
                    .slider("Master Volume", &mut state.quick.master_slider, 0.0..=100.0)
                    .fill(accent)
                    .no_bg()
                    .borderless()
                    .p(2.0);
                if m_resp.changed() {
                    let next = (state.quick.master_slider.round() as i32).clamp(0, 100);
                    if next != master_before {
                        state.quick.last_host_master = next;
                        state.host.audio_master = next;
                        actions.push(UiAction::SetAudioVolume {
                            key: String::from("AudioMaster"),
                            value: next,
                        });
                    }
                }

                let voice_before = state.quick.voice_slider.round() as i32;
                let v_resp = ui
                    .slider("Player Voices", &mut state.quick.voice_slider, 0.0..=100.0)
                    .fill(accent)
                    .no_bg()
                    .borderless()
                    .p(2.0);
                if v_resp.changed() {
                    let next = (state.quick.voice_slider.round() as i32).clamp(0, 100);
                    if next != voice_before {
                        state.quick.last_host_voice = next;
                        state.host.audio_voice = next;
                        actions.push(UiAction::SetAudioVolume {
                            key: String::from("AudioPlayerVoice"),
                            value: next,
                        });
                    }
                }

                let video_before = state.quick.video_slider.round() as i32;
                let vp_resp = ui
                    .slider("Video Player", &mut state.quick.video_slider, 0.0..=100.0)
                    .fill(accent)
                    .no_bg()
                    .borderless()
                    .p(2.0);
                if vp_resp.changed() {
                    let next = (state.quick.video_slider.round() as i32).clamp(0, 100);
                    if next != video_before {
                        state.quick.last_host_video = next;
                        state.host.audio_video_player = next;
                        actions.push(UiAction::SetAudioVolume {
                            key: String::from("AudioVideoPlayer"),
                            value: next,
                        });
                    }
                }
            })
            .p(11.0)
            .gap(5.0)
            .bg(card_bg())
            .bd(1.0, border_subtle())
            .bevel_rounded(10.9, 2.0)
            .shadow(14.0, SHADOW_CARD);
        })
        .grow(1.0)
        .gap(9.0);
    })
    .grow(1.0)
    .gap(9.0);
}

fn render_qm_world_instance_modal(
    ui: &mut UiContext,
    state: &mut AppState,
    world_id: &str,
    world_name: &str,
    accent: Color,
    actions: &mut Vec<UiAction>,
) {
    ui.zstack(|ui| {
        if crate::theme::modal_scrim(ui, "qm_world_modal_scrim_fx", "qm_world_modal_scrim") {
            push_click(actions);
            state.quick.qm_world_modal = None;
        }

        ui.col(|ui| {
            ui.col(|_| {}).grow(1.0);
            ui.row(|ui| {
                ui.col(|_| {}).grow(1.0);
                let mut card = ui
                    .col(|ui| {
                        ui.row(|ui| {
                            icons::icon(ui, icons::WORLD, 16.0).fg(accent);
                            ui.txt(world_name)
                                .sz(crate::theme::fs(14.0))
                                .bold()
                                .fg(TEXT_PRIMARY)
                                .ellipsis()
                                .grow(1.0);
                            if ui
                                .btn_with("qm_world_modal_close", |ui| {
                                    icons::icon(ui, icons::CLOSE, 14.0).fg(TEXT_MUTED);
                                })
                                .p(5.0)
                                .bg(Color::TRANSPARENT)
                                .hover_bg(Color::rgba(239, 68, 68, 80))
                                .bevel_rounded(4.5, 1.3)
                                .clicked()
                            {
                                push_click(actions);
                                state.quick.qm_world_modal = None;
                            }
                        })
                        .align_center()
                        .gap(8.0);

                        ui.txt("Privacy").sz(crate::theme::fs(10.5)).semibold().fg(TEXT_MUTED);
                        ui.row(|ui| {
                            for (val, lbl) in [
                                ("Public", "Public"),
                                ("FriendsOfFriends", "Friends+"),
                                ("Friends", "Friends"),
                                ("EveryoneCanInvite", "Invite+"),
                                ("OwnerMustInvite", "Invite"),
                            ] {
                                let active = state.quick.qm_world_privacy == val;
                                if ui
                                    .btn_with(&format!("qm_wm_priv_{val}"), |ui| {
                                        ui.txt(lbl)
                                            .sz(crate::theme::fs(10.0))
                                            .semibold()
                                            .align(TextAlign::Center)
                                            .fg(if active { TEXT_PRIMARY } else { TEXT_MUTED })
                                            .grow(1.0);
                                    })
                                    .grow(1.0)
                                    .py(6.0)
                                    .bg(if active {
                                        tint_alpha(accent, 48)
                                    } else {
                                        tile_idle_bg()
                                    })
                                    .hover_bg(tile_idle_hover())
                                    .bd(
                                        1.0,
                                        if active {
                                            tint_alpha(accent, 190)
                                        } else {
                                            border_subtle()
                                        },
                                    )
                                    .bevel_rounded(5.5, 1.3)
                                    .clicked()
                                {
                                    push_click(actions);
                                    state.quick.qm_world_privacy = String::from(val);
                                }
                            }
                        })
                        .gap(5.0);

                        ui.row(|ui| {
                            for (val, lbl) in [
                                ("Group", "Group"),
                                ("GroupPlus", "Group+"),
                                ("GroupPublic", "Group Public"),
                            ] {
                                let active = state.quick.qm_world_privacy == val;
                                if ui
                                    .btn_with(&format!("qm_wm_priv_{val}"), |ui| {
                                        ui.txt(lbl)
                                            .sz(crate::theme::fs(10.0))
                                            .semibold()
                                            .align(TextAlign::Center)
                                            .fg(if active { TEXT_PRIMARY } else { TEXT_MUTED })
                                            .grow(1.0);
                                    })
                                    .grow(1.0)
                                    .py(6.0)
                                    .bg(if active {
                                        tint_alpha(accent, 48)
                                    } else {
                                        tile_idle_bg()
                                    })
                                    .hover_bg(tile_idle_hover())
                                    .bd(
                                        1.0,
                                        if active {
                                            tint_alpha(accent, 190)
                                        } else {
                                            border_subtle()
                                        },
                                    )
                                    .bevel_rounded(5.5, 1.3)
                                    .clicked()
                                {
                                    push_click(actions);
                                    state.quick.qm_world_privacy = String::from(val);
                                }
                            }
                        })
                        .gap(5.0);

                        let is_group_privacy = matches!(
                            state.quick.qm_world_privacy.as_str(),
                            "Group" | "GroupPlus" | "GroupPublic"
                        );

                        if is_group_privacy {
                            let my_groups = state.host.social.my_groups.clone();
                            if !my_groups.is_empty()
                                && !my_groups
                                    .iter()
                                    .any(|g| g.id == state.quick.qm_world_group_id)
                            {
                                state.quick.qm_world_group_id = my_groups[0].id.clone();
                            }

                            if my_groups.is_empty() {
                                ui.txt("You are not in any groups.")
                                    .sz(crate::theme::fs(10.5))
                                    .fg(AMBER);
                            } else {
                                ui.hscroll("qm_wm_grp_pick", |ui| {
                                    for g in &my_groups {
                                        let active = state.quick.qm_world_group_id == g.id;
                                        let lbl = if g.tag.is_empty() {
                                            g.name.clone()
                                        } else {
                                            format!("[{}] {}", g.tag, g.name)
                                        };
                                        if ui
                                            .btn_with(&format!("qm_wm_g_{}", g.id), |ui| {
                                                ui.txt(&lbl)
                                                    .sz(crate::theme::fs(10.0))
                                                    .semibold()
                                                    .fg(if active {
                                                        TEXT_PRIMARY
                                                    } else {
                                                        TEXT_MUTED
                                                    });
                                            })
                                            .px(9.0)
                                            .py(5.0)
                                            .bg(if active {
                                                tint_alpha(EMERALD, 48)
                                            } else {
                                                card_inner_bg()
                                            })
                                            .hover_bg(tile_idle_hover())
                                            .bd(
                                                1.0,
                                                if active {
                                                    tint_alpha(EMERALD, 185)
                                                } else {
                                                    border_subtle()
                                                },
                                            )
                                            .bevel_rounded(5.0, 1.3)
                                            .clicked()
                                        {
                                            push_click(actions);
                                            state.quick.qm_world_group_id = g.id.clone();
                                        }
                                    }
                                })
                                .gap(4.0)
                                .no_scrollbar();
                            }
                        }

                        ui.txt("Region").sz(crate::theme::fs(10.5)).semibold().fg(TEXT_MUTED);
                        ui.row(|ui| {
                            for (val, lbl) in [("Eu", "EU"), ("Us", "US"), ("As", "Asia")] {
                                let active = state.quick.qm_world_region == val;
                                if ui
                                    .btn_with(&format!("qm_wm_reg_{val}"), |ui| {
                                        ui.txt(lbl)
                                            .sz(crate::theme::fs(10.5))
                                            .semibold()
                                            .align(TextAlign::Center)
                                            .fg(if active { TEXT_PRIMARY } else { TEXT_MUTED })
                                            .grow(1.0);
                                    })
                                    .grow(1.0)
                                    .py(6.5)
                                    .bg(if active {
                                        tint_alpha(accent, 48)
                                    } else {
                                        tile_idle_bg()
                                    })
                                    .hover_bg(tile_idle_hover())
                                    .bd(
                                        1.0,
                                        if active {
                                            tint_alpha(accent, 190)
                                        } else {
                                            border_subtle()
                                        },
                                    )
                                    .bevel_rounded(5.5, 1.3)
                                    .clicked()
                                {
                                    push_click(actions);
                                    state.quick.qm_world_region = String::from(val);
                                }
                            }
                        })
                        .gap(6.0);

                        if !state.host.content.create_instance_error.is_empty() {
                            ui.txt(&state.host.content.create_instance_error)
                                .sz(crate::theme::fs(10.5))
                                .fg(AMBER)
                                .wrap();
                        }

                        ui.row(|ui| {
                            let busy = state.host.content.creating_instance;
                            if ui
                                .btn_with("qm_wm_create", |ui| {
                                    ui.txt(if busy {
                                        "Creating Instance..."
                                    } else {
                                        "Create & Join"
                                    })
                                    .sz(crate::theme::fs(11.5))
                                    .bold()
                                    .align(TextAlign::Center)
                                    .fg(TEXT_PRIMARY)
                                    .grow(1.0);
                                })
                                .grow(1.0)
                                .py(9.0)
                                .bg(tint_alpha(EMERALD, 52))
                                .hover_bg(tint_alpha(EMERALD, 85))
                                .bd(1.5, tint_alpha(EMERALD, 195))
                                .bevel_rounded(6.5, 1.3)
                                .opacity(if busy { 0.6 } else { 1.0 })
                                .clicked()
                                && !busy
                            {
                                push_click(actions);
                                actions.push(UiAction::ContentCreateInstance {
                                    world_id: String::from(world_id),
                                    privacy: state.quick.qm_world_privacy.clone(),
                                    region: state.quick.qm_world_region.clone(),
                                    group_id: if is_group_privacy {
                                        state.quick.qm_world_group_id.clone()
                                    } else {
                                        String::new()
                                    },
                                });
                            }

                            if ui
                                .btn_with("qm_wm_details", |ui| {
                                    ui.txt("World Details")
                                        .sz(crate::theme::fs(11.0))
                                        .semibold()
                                        .align(TextAlign::Center)
                                        .fg(TEXT_PRIMARY)
                                        .grow(1.0);
                                })
                                .px(12.0)
                                .py(9.0)
                                .bg(tile_idle_bg())
                                .hover_bg(tile_idle_hover())
                                .bd(1.0, border_subtle())
                                .bevel_rounded(6.5, 1.3)
                                .clicked()
                            {
                                push_click(actions);
                                state.quick.qm_world_modal = None;
                                state.main_menu.nav_history.clear();
                                state.main_menu.active_tab = String::from("worlds");
                                actions.push(UiAction::QmOpenWorldDetails {
                                    world_id: String::from(world_id),
                                });
                            }
                        })
                        .gap(8.0);
                    })
                    .w(440.0)
                    .p(16.0)
                    .gap(8.0)
                    .bg(Color::rgba(15, 19, 28, 248))
                    .bd(1.5, accent)
                    .bevel_rounded(12.0, 2.0)
                    .shadow(22.0, Color::rgba(0, 0, 0, 190));
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

pub fn render_group_mod_prompt_modal(
    ui: &mut UiContext,
    prompt: &GroupModPromptSnapshot,
    accent: Color,
    actions: &mut Vec<UiAction>,
    id_prefix: &str,
) {
    let scrim_fx_id = format!("{}_group_mod_scrim_fx", id_prefix);
    let scrim_id = format!("{}_group_mod_scrim", id_prefix);
    let close_id = format!("{}_group_mod_close", id_prefix);
    let cancel_id = format!("{}_group_mod_cancel", id_prefix);
    let never_id = format!("{}_group_mod_never", id_prefix);
    let join_id = format!("{}_group_mod_join", id_prefix);

    ui.zstack(|ui| {
        if crate::theme::modal_scrim(ui, &scrim_fx_id, &scrim_id) {
            push_click(actions);
            actions.push(UiAction::ContentResolveGroupModPrompt {
                accept: false,
                never_show: false,
            });
        }

        ui.col(|ui| {
            ui.col(|_| {}).grow(1.0);
            ui.row(|ui| {
                ui.col(|_| {}).grow(1.0);
                let mut card = ui
                    .col(|ui| {
                        ui.row(|ui| {
                            icons::icon(ui, icons::PLAYERS, 16.0).fg(AMBER);
                            ui.txt("Group Moderated Instance")
                                .sz(crate::theme::fs(13.5))
                                .bold()
                                .fg(TEXT_PRIMARY)
                                .grow(1.0);
                            if ui
                                .btn_with(&close_id, |ui| {
                                    icons::icon(ui, icons::CLOSE, 13.0).fg(TEXT_MUTED);
                                })
                                .p(5.0)
                                .bg(tile_idle_bg())
                                .hover_bg(tile_idle_hover())
                                .r(5.0)
                                .clicked()
                            {
                                push_click(actions);
                                actions.push(UiAction::ContentResolveGroupModPrompt {
                                    accept: false,
                                    never_show: false,
                                });
                            }
                        })
                        .align_center()
                        .gap(8.0);

                        ui.col(|ui| {
                            if !prompt.group_name.is_empty() {
                                ui.row(|ui| {
                                    ui.txt(&prompt.group_name)
                                        .sz(crate::theme::fs(12.5))
                                        .bold()
                                        .fg(TEXT_PRIMARY);
                                    if prompt.is_verified {
                                        ui.txt("VERIFIED")
                                            .sz(crate::theme::fs(9.0))
                                            .bold()
                                            .fg(EMERALD)
                                            .px(5.0)
                                            .py(2.0)
                                            .bg(tint_alpha(EMERALD, 35))
                                            .r(4.0);
                                    }
                                })
                                .align_center()
                                .gap(6.0);
                            }
                            if !prompt.world_name.is_empty() || !prompt.instance_name.is_empty() {
                                let label = if !prompt.world_name.is_empty()
                                    && !prompt.instance_name.is_empty()
                                {
                                    format!("{} - {}", prompt.world_name, prompt.instance_name)
                                } else if !prompt.world_name.is_empty() {
                                    prompt.world_name.clone()
                                } else {
                                    prompt.instance_name.clone()
                                };
                                ui.txt(&label).sz(crate::theme::fs(10.5)).fg(TEXT_MUTED).wrap();
                            }
                            let privacy_label = if prompt.privacy.is_empty() {
                                "group"
                            } else {
                                prompt.privacy.as_str()
                            };
                            let mod_note = if prompt.moderator_count > 0 {
                                format!(
                                    "This {} instance is moderated by its group ({} moderator{}). Group moderators can kick or moderate within this instance.",
                                    privacy_label,
                                    prompt.moderator_count,
                                    if prompt.moderator_count == 1 { "" } else { "s" }
                                )
                            } else {
                                format!(
                                    "This {} instance is moderated by its group. Group moderators can kick or moderate within this instance.",
                                    privacy_label
                                )
                            };
                            ui.txt(&mod_note).sz(crate::theme::fs(10.5)).fg(TEXT_DIM).wrap();
                        })
                        .p(10.0)
                        .gap(5.0)
                        .bg(card_inner_bg())
                        .bd(1.0, border_subtle())
                        .r(8.0);

                        ui.row(|ui| {
                            if ui
                                .btn_with(&cancel_id, |ui| {
                                    ui.txt("Cancel")
                                        .sz(crate::theme::fs(11.0))
                                        .semibold()
                                        .align(TextAlign::Center)
                                        .fg(TEXT_PRIMARY)
                                        .grow(1.0);
                                })
                                .px(12.0)
                                .py(8.5)
                                .bg(tile_idle_bg())
                                .hover_bg(tile_idle_hover())
                                .bd(1.0, border_subtle())
                                .bevel_rounded(6.5, 1.3)
                                .clicked()
                            {
                                push_click(actions);
                                actions.push(UiAction::ContentResolveGroupModPrompt {
                                    accept: false,
                                    never_show: false,
                                });
                            }

                            if ui
                                .btn_with(&never_id, |ui| {
                                    ui.txt("Don't Ask Again")
                                        .sz(crate::theme::fs(10.5))
                                        .semibold()
                                        .align(TextAlign::Center)
                                        .fg(TEXT_MUTED)
                                        .grow(1.0);
                                })
                                .px(10.0)
                                .py(8.5)
                                .bg(tile_idle_bg())
                                .hover_bg(tile_idle_hover())
                                .bd(1.0, border_subtle())
                                .bevel_rounded(6.5, 1.3)
                                .clicked()
                            {
                                push_click(actions);
                                actions.push(UiAction::ContentResolveGroupModPrompt {
                                    accept: true,
                                    never_show: true,
                                });
                            }

                            if ui
                                .btn_with(&join_id, |ui| {
                                    ui.txt("Join Instance")
                                        .sz(crate::theme::fs(11.5))
                                        .bold()
                                        .align(TextAlign::Center)
                                        .fg(TEXT_PRIMARY)
                                        .grow(1.0);
                                })
                                .grow(1.0)
                                .py(8.5)
                                .bg(tint_alpha(EMERALD, 52))
                                .hover_bg(tint_alpha(EMERALD, 85))
                                .bd(1.5, tint_alpha(EMERALD, 195))
                                .bevel_rounded(6.5, 1.3)
                                .clicked()
                            {
                                push_click(actions);
                                actions.push(UiAction::ContentResolveGroupModPrompt {
                                    accept: true,
                                    never_show: false,
                                });
                            }
                        })
                        .gap(8.0);
                    })
                    .w(430.0)
                    .p(16.0)
                    .gap(10.0)
                    .bg(Color::rgba(15, 19, 28, 248))
                    .bd(1.5, accent)
                    .bevel_rounded(12.0, 2.0)
                    .shadow(22.0, Color::rgba(0, 0, 0, 190));
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
