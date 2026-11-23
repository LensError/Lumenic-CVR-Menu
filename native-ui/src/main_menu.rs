use lumenic::{App, Color, ImageFit, TextAlign, UiContext};

use crate::icons;
use crate::state::{AppState, UiAction};
use crate::theme::{
    push_click, resolve_player_accent, tint_alpha, AMBER, BG_GLOW_FRAG as MM_BG_GLOW_FRAG,
    BORDER_SUBTLE, CARD_BG, CARD_INNER_BG, EMERALD, RED, SHADOW_CARD, TEXT_DIM, TEXT_MUTED,
    TEXT_PRIMARY, TILE_DISABLED_BG, TILE_IDLE_BG, TILE_IDLE_HOVER,
};

#[allow(clippy::too_many_arguments)]
fn util_tile(
    ui: &mut UiContext,
    id: &str,
    icon_path: &str,
    label: &str,
    active: bool,
    enabled: bool,
    active_color: Color,
) -> bool {
    let (bg, hover_bg, border, icon_col, title_col) = if !enabled {
        (
            TILE_DISABLED_BG,
            TILE_DISABLED_BG,
            Color::rgba(100, 116, 139, 32),
            TEXT_DIM,
            TEXT_DIM,
        )
    } else if active {
        (
            tint_alpha(active_color, 54),
            tint_alpha(active_color, 78),
            tint_alpha(active_color, 195),
            active_color,
            TEXT_PRIMARY,
        )
    } else {
        (
            TILE_IDLE_BG,
            TILE_IDLE_HOVER,
            BORDER_SUBTLE,
            Color::rgb(203, 213, 225),
            TEXT_PRIMARY,
        )
    };

    let resp = ui
        .btn_with(id, |ui| {
            ui.col(|ui| {
                icons::icon(ui, icon_path, 20.0).fg(icon_col);
                ui.txt(label)
                    .sz(11.0)
                    .semibold()
                    .align(TextAlign::Center)
                    .fg(title_col)
                    .ellipsis();
            })
            .center()
            .gap(4.0)
            .grow(1.0);
        })
        .grow(1.0)
        .h_frac(1.0)
        .min_h(64.0)
        .p(6.0)
        .bg(bg)
        .hover_bg(hover_bg)
        .bd(if active { 1.5 } else { 1.0 }, border)
        .bevel_rounded(7.5, 1.3)
        .opacity(if enabled { 1.0 } else { 0.5 });

    enabled && resp.clicked()
}

/// Shared sleek Obsidian Slate floating tooltip overlay positioned near the cursor.
pub fn render_floating_tooltip(ui: &mut UiContext, text: &str, accent: Color) {
    if text.trim().is_empty() {
        return;
    }
    let Some([px, py]) = ui.pointer_pos() else {
        return;
    };
    let [vw, vh] = ui.viewport_size();
    let max_w = 300.0_f32;
    let tx = if px + 16.0 + max_w > vw - 10.0 {
        (px - max_w - 14.0).max(10.0)
    } else {
        px + 16.0
    };
    let ty = if py + 84.0 > vh - 10.0 {
        (py - 64.0).max(10.0)
    } else {
        py + 18.0
    };

    ui.overlay(|ui| {
        ui.col(|ui| {
            ui.txt(text).sz(11.0).fg(TEXT_PRIMARY).wrap();
        })
        .pos(tx, ty)
        .max_w(max_w)
        .px(10.0)
        .py(7.0)
        .bg(Color::rgba(11, 15, 24, 246))
        .bd(1.2, tint_alpha(accent, 195))
        .bevel_rounded(7.0, 1.5)
        .shadow(14.0, Color::rgba(0, 0, 0, 190))
        .passthrough();
    });
}

/// Renders the Phase 3A Main Menu Shell & Home Dashboard (1280x720 logical 16:9).
pub fn render_main_menu(
    ui: &mut UiContext,
    _app: &mut App,
    state: &mut AppState,
    actions: &mut Vec<UiAction>,
) {
    state.main_menu.hovered_tooltip.clear();
    state.main_menu.aas_preview_rendered = false;
    if state.host.nav_seq != state.main_menu.last_nav_seq {
        state.main_menu.last_nav_seq = state.host.nav_seq;
        if !state.host.nav_tab.is_empty() {
            state.main_menu.active_tab = state.host.nav_tab.clone();
        }
    }
    let accent = resolve_player_accent(state);

    // Menu-wide hex cursor-follow glow shader behind all floating cards
    let [vw, vh] = ui.viewport_size();
    let mm_rect = match state.main_menu.mm_root_rect {
        Some([x, y, w, h]) if w > 1.0 && h > 1.0 => lumenic::Rect::new(x, y, w, h),
        _ => lumenic::Rect::new(0.0, 0.0, vw.max(1.0), vh.max(1.0)),
    };
    let (ptr_u, ptr_v, ptr_active) = if let Some([px, py]) = ui.pointer_pos() {
        let u = (px - mm_rect.x) / mm_rect.w.max(1.0);
        let v = (py - mm_rect.y) / mm_rect.h.max(1.0);
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
    let shader_uniforms = [ptr_u, ptr_v, ptr_active, ac[0], ac[1], ac[2]];

    let outer_resp = ui
        .zstack(|ui| {
            ui.viewport("mm_bg_glow", move |c2d, _s3d| {
                c2d.shader(0x4C55_4D10, MM_BG_GLOW_FRAG, &shader_uniforms);
            })
            .w_frac(1.0)
            .h_frac(1.0)
            .no_bg()
            .borderless()
            .r(0.0)
            .passthrough();

            ui.col(|ui| {
                // =============================================================
                // 1. MAIN CONTENT AREA (HOME DASHBOARD OR PAGE PLACEHOLDER)
                // =============================================================
                let active_tab = state.main_menu.active_tab.clone();
                match active_tab.as_str() {
                    "home" => render_home_dashboard(ui, state, accent, actions),
                    "modSettings" => crate::uix_view::render_uix_mod_settings(ui, state, accent, actions),
                    "worlds" | "avatars" | "props" => {
                        crate::content_browser::render_content_browser(ui, state, &active_tab, accent, actions);
                    }
                    "search" => {
                        crate::content_browser::render_global_search_view(ui, state, accent, actions);
                    }
                    "friends" => {
                        crate::social_view::render_friends_page(ui, state, accent, actions);
                    }
                    "currentInstance" => {
                        crate::social_view::render_current_instance_page(ui, state, accent, actions);
                    }
                    "messages" => {
                        crate::social_view::render_messages_page(ui, state, accent, actions);
                    }
                    "discover" => {
                        crate::content_browser::render_discover_page(ui, state, accent, actions);
                    }
                    "settings" => {
                        crate::settings_view::render_settings_page(ui, state, accent, actions);
                    }
                    "avatarSettings" => {
                        crate::avatar_settings_view::render_avatar_settings_page(
                            ui, state, accent, actions,
                        );
                    }
                    _ => render_placeholder_page(ui, state, &active_tab, accent, actions),
                }

                // =============================================================
                // 2. PETITE BOTTOM NAVIGATION DOCK
                // =============================================================
                render_bottom_dock(ui, state, accent, actions);
            })
            .w_frac(1.0)
            .h_frac(1.0)
            .p(14.0)
            .gap(10.0);

            if let Some(prompt) = state.host.content.group_mod_prompt.clone() {
                crate::quick_menu::render_group_mod_prompt_modal(ui, &prompt, accent, actions, "mm");
            }
        })
        .w_frac(1.0)
        .h_frac(1.0);

    if let Some(r) = outer_resp.prev_rect() {
        state.main_menu.mm_root_rect = Some([r.x, r.y, r.w, r.h]);
    }

    if state.main_menu.info_modal_open {
        render_game_instance_info_modal(ui, state, accent, actions);
    }

    if !state.main_menu.aas_preview_rendered {
        state.main_menu.aas_preview_rect = None;
        if state.main_menu.aas_preview_dragging {
            state.main_menu.aas_preview_dragging = false;
            if state.main_menu.aas_preview_target.starts_with("worldpano:") {
                actions.push(UiAction::WorldPanoLook {
                    delta_x: 0.0,
                    delta_y: 0.0,
                    is_dragging: false,
                });
            } else {
                actions.push(UiAction::AvatarPreviewRotate {
                    delta: 0.0,
                    is_dragging: false,
                });
            }
        }
        if state.main_menu.aas_last_preview_sync.take().is_some() {
            state.main_menu.aas_preview_target.clear();
            actions.push(UiAction::SyncAvatarPreview {
                surface_id: 0,
                active: false,
                target: String::new(),
                x: 0.0,
                y: 0.0,
                w: 0.0,
                h: 0.0,
            });
        }
    }

    if !state.main_menu.hovered_tooltip.is_empty() {
        let tip = state.main_menu.hovered_tooltip.clone();
        render_floating_tooltip(ui, &tip, accent);
    }
}

fn render_home_dashboard(
    ui: &mut UiContext,
    state: &mut AppState,
    accent: Color,
    actions: &mut Vec<UiAction>,
) {
    ui.row(|ui| {
        // =====================================================================
        // LEFT COLUMN: PROFILE + COMPACT GAME/INSTANCE INFO + SIDE LEAFLET
        // =====================================================================
        ui.col(|ui| {
            // --- 1. PROFILE HERO CARD ---
            ui.col(|ui| {
                // Clickable top identity banner -> opens local user's native Profile Overview
                if ui
                    .btn_with("mm_btn_local_profile_hero", |ui| {
                        ui.row(|ui| {
                            let pfp_key = if state.host.local_pfp_key.is_empty() {
                                "local_pfp"
                            } else {
                                &state.host.local_pfp_key
                            };
                            if let Some(tex) = state.main_textures.get(pfp_key) {
                                ui.img(tex)
                                    .w(56.0)
                                    .h(56.0)
                                    .fit(ImageFit::Cover)
                                    .hexagon_pointy()
                                    .r(3.5)
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
                                        .sz(20.0)
                                        .bold()
                                        .align(TextAlign::Center)
                                        .fg(accent);
                                })
                                .center()
                                .w(56.0)
                                .h(56.0)
                                .bg(tint_alpha(accent, 36))
                                .bd(2.0, accent)
                                .hexagon_pointy()
                                .r(3.5);
                            }

                            ui.col(|ui| {
                                ui.row(|ui| {
                                    let user = if state.host.username.is_empty() {
                                        "Player"
                                    } else {
                                        &state.host.username
                                    };
                                    ui.txt(user)
                                        .sz(15.5)
                                        .bold()
                                        .fg(TEXT_PRIMARY)
                                        .ellipsis()
                                        .grow(1.0);

                                    let rank_lbl = if state.host.user_rank.is_empty() {
                                        "User"
                                    } else {
                                        &state.host.user_rank
                                    };
                                    ui.txt(rank_lbl)
                                        .sz(9.5)
                                        .mono()
                                        .bold()
                                        .fg(accent)
                                        .bg(tint_alpha(accent, 34))
                                        .bd(1.0, tint_alpha(accent, 130))
                                        .px(6.0)
                                        .py(1.5)
                                        .bevel_rounded(4.0, 1.2);
                                })
                                .align_center()
                                .gap(6.0);

                                // Featured Badge row with badge thumbnail or star icon
                                ui.row(|ui| {
                                    if !state.host.featured_badge_image_key.is_empty() {
                                        if let Some(btex) = state
                                            .main_textures
                                            .get(&state.host.featured_badge_image_key)
                                        {
                                            ui.img(btex)
                                                .w(14.0)
                                                .h(14.0)
                                                .fit(ImageFit::Contain)
                                                .r(2.5);
                                        } else {
                                            icons::icon(ui, icons::STAR, 12.0).fg(accent);
                                        }
                                    } else {
                                        icons::icon(ui, icons::STAR, 12.0).fg(accent);
                                    }
                                    let badge_line = if state.host.featured_badge.is_empty() {
                                        "No Featured Badge"
                                    } else {
                                        &state.host.featured_badge
                                    };
                                    ui.txt(badge_line)
                                        .sz(10.5)
                                        .semibold()
                                        .fg(TEXT_MUTED)
                                        .ellipsis()
                                        .grow(1.0);
                                })
                                .align_center()
                                .gap(5.0);

                                // Featured Group row with group thumbnail or players icon
                                ui.row(|ui| {
                                    if !state.host.featured_group_image_key.is_empty() {
                                        if let Some(gtex) = state
                                            .main_textures
                                            .get(&state.host.featured_group_image_key)
                                        {
                                            ui.img(gtex)
                                                .w(14.0)
                                                .h(14.0)
                                                .fit(ImageFit::Cover)
                                                .r(2.5);
                                        } else {
                                            icons::icon(ui, icons::PLAYERS, 12.0).fg(TEXT_DIM);
                                        }
                                    } else {
                                        icons::icon(ui, icons::PLAYERS, 12.0).fg(TEXT_DIM);
                                    }
                                    let grp_line = if state.host.featured_group.is_empty() {
                                        "No Featured Group"
                                    } else {
                                        &state.host.featured_group
                                    };
                                    ui.txt(grp_line)
                                        .sz(10.0)
                                        .fg(TEXT_DIM)
                                        .ellipsis()
                                        .grow(1.0);
                                })
                                .align_center()
                                .gap(5.0);
                            })
                            .gap(3.0)
                            .grow(1.0);
                        })
                        .align_center()
                        .gap(10.0)
                        .grow(1.0);
                    })
                    .p(6.0)
                    .bg(CARD_INNER_BG)
                    .hover_bg(TILE_IDLE_HOVER)
                    .bd(1.0, BORDER_SUBTLE)
                    .bevel_rounded(7.5, 1.5)
                    .clicked()
                {
                    push_click(actions);
                    if !state.host.user_id.is_empty() {
                        state.main_menu.user_detail_tab = 0;
                        state.main_menu.active_tab = String::from("friends");
                        actions.push(UiAction::SocialLoadUserDetail {
                            user_id: state.host.user_id.clone(),
                        });
                    }
                }

                // Native Profile / Badges / Groups quick-nav pills
                ui.row(|ui| {
                    for (tab_idx, btn_id, ic, lbl) in [
                        (0usize, "mm_btn_my_profile", icons::AVATAR, "Profile"),
                        (1usize, "mm_btn_badges", icons::STAR, "Badges"),
                        (2usize, "mm_btn_groups", icons::PLAYERS, "Groups"),
                    ] {
                        if ui
                            .btn_with(btn_id, |ui| {
                                ui.row(|ui| {
                                    icons::icon(ui, ic, 12.5).fg(accent);
                                    ui.txt(lbl).sz(10.5).semibold().fg(TEXT_PRIMARY);
                                })
                                .center()
                                .grow(1.0)
                                .gap(4.0);
                            })
                            .grow(1.0)
                            .py(5.5)
                            .bg(TILE_IDLE_BG)
                            .hover_bg(TILE_IDLE_HOVER)
                            .bd(1.0, BORDER_SUBTLE)
                            .bevel_rounded(6.0, 1.3)
                            .clicked()
                        {
                            push_click(actions);
                            if !state.host.user_id.is_empty() {
                                state.main_menu.user_detail_tab = tab_idx;
                                state.main_menu.active_tab = String::from("friends");
                                actions.push(UiAction::SocialLoadUserDetail {
                                    user_id: state.host.user_id.clone(),
                                });
                            }
                        }
                    }
                })
                .gap(5.0);
            })
            .p(9.0)
            .gap(7.0)
            .bg(CARD_BG)
            .bd(1.0, tint_alpha(accent, 105))
            .bevel_rounded(10.5, 2.0)
            .shadow(14.0, SHADOW_CARD);

            // --- 2. REDESIGNED GAME INFO / INSTANCE INFO TELEMETRY CARD ---
            ui.col(|ui| {
                // Segmented header pill bar + Expand Details button
                ui.row(|ui| {
                    for (idx, ic, lbl) in [
                        (0usize, icons::SLIDERS, "Game Info"),
                        (1usize, icons::INSTANCE, "Instance"),
                    ] {
                        let active = state.main_menu.info_tab == idx;
                        let id = format!("mm_info_tab_{idx}");
                        if ui
                            .btn_with(&id, |ui| {
                                ui.row(|ui| {
                                    icons::icon(ui, ic, 12.0)
                                        .fg(if active { accent } else { TEXT_DIM });
                                    ui.txt(lbl)
                                        .sz(10.5)
                                        .semibold()
                                        .fg(if active { TEXT_PRIMARY } else { TEXT_MUTED });
                                })
                                .center()
                                .gap(4.0)
                                .grow(1.0);
                            })
                            .grow(1.0)
                            .py(4.5)
                            .bg(if active {
                                tint_alpha(accent, 45)
                            } else {
                                CARD_INNER_BG
                            })
                            .hover_bg(TILE_IDLE_HOVER)
                            .bd(
                                1.0,
                                if active {
                                    tint_alpha(accent, 185)
                                } else {
                                    BORDER_SUBTLE
                                },
                            )
                            .bevel_rounded(5.5, 1.3)
                            .clicked()
                        {
                            push_click(actions);
                            state.main_menu.info_tab = idx;
                        }
                    }

                    if ui
                        .btn_with("mm_btn_expand_info_icon", |ui| {
                            icons::icon(ui, icons::INFO, 13.0).fg(TEXT_MUTED);
                        })
                        .px(7.0)
                        .py(4.5)
                        .bg(CARD_INNER_BG)
                        .hover_bg(TILE_IDLE_HOVER)
                        .bd(1.0, BORDER_SUBTLE)
                        .bevel_rounded(5.5, 1.3)
                        .clicked()
                    {
                        push_click(actions);
                        state.main_menu.info_modal_open = true;
                    }
                })
                .align_center()
                .gap(5.0);

                if state.main_menu.info_tab == 0 {
                    // 2x2 Live Telemetry Grid
                    let fps_val = state.host.fps.max(0);
                    let fps_col = if fps_val >= 60 {
                        EMERALD
                    } else if fps_val >= 36 {
                        AMBER
                    } else {
                        RED
                    };
                    let g_ping = state.host.game_ping.max(0);
                    let v_ping = state.host.voice_ping.max(0);
                    let mode_txt = if state.host.is_vr { "VR Mode" } else { "Desktop" };

                    ui.row(|ui| {
                        // Tile 1: Frame Rate
                        ui.col(|ui| {
                            ui.txt("FRAME RATE").sz(8.5).bold().fg(TEXT_DIM);
                            ui.txt(&format!("{fps_val} FPS"))
                                .sz(12.5)
                                .mono()
                                .bold()
                                .fg(fps_col);
                        })
                        .grow(1.0)
                        .px(8.0)
                        .py(5.0)
                        .gap(1.5)
                        .bg(CARD_INNER_BG)
                        .bd(1.0, BORDER_SUBTLE)
                        .bevel_rounded(5.5, 1.2);

                        // Tile 2: Game Ping
                        ui.col(|ui| {
                            ui.txt("GAME PING").sz(8.5).bold().fg(TEXT_DIM);
                            ui.txt(&format!("{g_ping} ms"))
                                .sz(12.5)
                                .mono()
                                .bold()
                                .fg(TEXT_PRIMARY);
                        })
                        .grow(1.0)
                        .px(8.0)
                        .py(5.0)
                        .gap(1.5)
                        .bg(CARD_INNER_BG)
                        .bd(1.0, BORDER_SUBTLE)
                        .bevel_rounded(5.5, 1.2);
                    })
                    .gap(5.0);

                    ui.row(|ui| {
                        // Tile 3: Voice Ping
                        ui.col(|ui| {
                            ui.txt("VOICE PING").sz(8.5).bold().fg(TEXT_DIM);
                            ui.txt(&format!("{v_ping} ms"))
                                .sz(12.0)
                                .mono()
                                .semibold()
                                .fg(TEXT_PRIMARY);
                        })
                        .grow(1.0)
                        .px(8.0)
                        .py(5.0)
                        .gap(1.5)
                        .bg(CARD_INNER_BG)
                        .bd(1.0, BORDER_SUBTLE)
                        .bevel_rounded(5.5, 1.2);

                        // Tile 4: Rig / Build
                        ui.col(|ui| {
                            ui.txt("RIG / BUILD").sz(8.5).bold().fg(TEXT_DIM);
                            ui.txt(mode_txt)
                                .sz(11.5)
                                .semibold()
                                .fg(accent)
                                .ellipsis();
                        })
                        .grow(1.0)
                        .px(8.0)
                        .py(5.0)
                        .gap(1.5)
                        .bg(CARD_INNER_BG)
                        .bd(1.0, BORDER_SUBTLE)
                        .bevel_rounded(5.5, 1.2);
                    })
                    .gap(5.0);
                } else {
                    // Instance Info compact card + Roster shortcut
                    let world = if state.host.world_name.is_empty() {
                        "Local World"
                    } else {
                        &state.host.world_name
                    };
                    let inst = if state.host.instance_name.is_empty() {
                        "Offline / Local Instance"
                    } else {
                        &state.host.instance_name
                    };
                    let priv_txt = if state.host.instance_privacy.is_empty() {
                        "Local"
                    } else {
                        &state.host.instance_privacy
                    };
                    let pl_cnt = state.host.instance_players.max(1);

                    if ui
                        .btn_with("mm_btn_inst_card_open", |ui| {
                            ui.col(|ui| {
                                ui.row(|ui| {
                                    icons::icon(ui, icons::WORLD, 13.5).fg(accent);
                                    ui.txt(world)
                                        .sz(11.5)
                                        .bold()
                                        .fg(TEXT_PRIMARY)
                                        .ellipsis()
                                        .grow(1.0);
                                    ui.txt(&format!("{pl_cnt} ppl"))
                                        .sz(10.0)
                                        .mono()
                                        .bold()
                                        .fg(EMERALD)
                                        .bg(tint_alpha(EMERALD, 30))
                                        .px(5.5)
                                        .py(1.5)
                                        .bevel_rounded(4.0, 1.2);
                                })
                                .align_center()
                                .gap(5.0);

                                ui.row(|ui| {
                                    ui.txt(inst)
                                        .sz(10.0)
                                        .fg(TEXT_MUTED)
                                        .ellipsis()
                                        .grow(1.0);
                                    ui.txt(priv_txt)
                                        .sz(9.5)
                                        .mono()
                                        .fg(accent)
                                        .bg(tint_alpha(accent, 26))
                                        .px(5.0)
                                        .py(1.0)
                                        .bevel_rounded(3.5, 1.1);
                                })
                                .align_center()
                                .gap(5.0);
                            })
                            .gap(4.0)
                            .grow(1.0);
                        })
                        .p(7.5)
                        .bg(CARD_INNER_BG)
                        .hover_bg(TILE_IDLE_HOVER)
                        .bd(1.0, BORDER_SUBTLE)
                        .bevel_rounded(6.0, 1.3)
                        .clicked()
                    {
                        push_click(actions);
                        state.main_menu.active_tab = String::from("currentInstance");
                    }
                }
            })
            .p(9.0)
            .gap(5.0)
            .bg(CARD_BG)
            .bd(1.0, BORDER_SUBTLE)
            .bevel_rounded(10.5, 2.0)
            .shadow(14.0, SHADOW_CARD);

            // --- 3. TABBED SIDE LEAFLET: ONLINE FRIENDS | ACTIVE WORLDS ---
            ui.col(|ui| {
                ui.row(|ui| {
                    let f_lbl = format!("Friends ({})", state.host.online_friends.len());
                    let w_lbl = format!("Worlds ({})", state.host.active_worlds.len());
                    for (idx, lbl) in [(0usize, f_lbl), (1usize, w_lbl)] {
                        let active = state.main_menu.leaflet_tab == idx;
                        let id = format!("mm_leaflet_tab_{idx}");
                        if ui
                            .btn_with(&id, |ui| {
                                ui.txt(&lbl)
                                    .sz(11.0)
                                    .semibold()
                                    .align(TextAlign::Center)
                                    .fg(if active { TEXT_PRIMARY } else { TEXT_MUTED })
                                    .grow(1.0);
                            })
                            .grow(1.0)
                            .py(5.0)
                            .bg(if active {
                                tint_alpha(accent, 45)
                            } else {
                                Color::TRANSPARENT
                            })
                            .hover_bg(TILE_IDLE_HOVER)
                            .bd(
                                1.0,
                                if active {
                                    tint_alpha(accent, 185)
                                } else {
                                    BORDER_SUBTLE
                                },
                            )
                            .bevel_rounded(5.5, 1.3)
                            .clicked()
                        {
                            push_click(actions);
                            state.main_menu.leaflet_tab = idx;
                        }
                    }
                })
                .gap(5.0);

                ui.scroll("mm_leaflet_scroll", |ui| {
                    if state.main_menu.leaflet_tab == 0 {
                        if state.host.online_friends.is_empty() {
                            ui.col(|ui| {
                                icons::icon(ui, icons::PLAYERS, 22.0)
                                    .fg(Color::rgba(148, 163, 184, 70));
                                ui.txt("No friends online")
                                    .sz(11.5)
                                    .semibold()
                                    .fg(TEXT_MUTED);
                            })
                            .center()
                            .py(28.0)
                            .gap(5.0);
                        } else {
                            for fr in &state.host.online_friends {
                                let btn_id = format!("mm_fr_{}", fr.id);
                                if ui
                                    .btn_with(&btn_id, |ui| {
                                        ui.row(|ui| {
                                            ui.txt("")
                                                .w(7.0)
                                                .h(7.0)
                                                .bg(EMERALD)
                                                .circle();
                                            ui.col(|ui| {
                                                ui.txt(&fr.name)
                                                    .sz(11.5)
                                                    .semibold()
                                                    .fg(TEXT_PRIMARY)
                                                    .ellipsis();
                                                let loc = if fr.world_name.is_empty() {
                                                    if fr.status.is_empty() {
                                                        "Online"
                                                    } else {
                                                        &fr.status
                                                    }
                                                } else {
                                                    &fr.world_name
                                                };
                                                ui.txt(loc).sz(10.0).fg(TEXT_MUTED).ellipsis();
                                            })
                                            .gap(1.0)
                                            .grow(1.0);
                                        })
                                        .align_center()
                                        .gap(7.0)
                                        .grow(1.0);
                                    })
                                    .px(8.0)
                                    .py(6.0)
                                    .bg(CARD_INNER_BG)
                                    .hover_bg(TILE_IDLE_HOVER)
                                    .bd(1.0, BORDER_SUBTLE)
                                    .bevel_rounded(6.0, 1.3)
                                    .clicked()
                                {
                                    push_click(actions);
                                    state.main_menu.user_detail_tab = 0;
                                    state.main_menu.active_tab = String::from("friends");
                                    actions.push(UiAction::SocialLoadUserDetail {
                                        user_id: fr.id.clone(),
                                    });
                                }
                            }
                        }
                    } else if state.host.active_worlds.is_empty() {
                        ui.col(|ui| {
                            icons::icon(ui, icons::WORLD, 22.0)
                                .fg(Color::rgba(148, 163, 184, 70));
                            ui.txt("No active worlds loaded")
                                .sz(11.5)
                                .semibold()
                                .fg(TEXT_MUTED);
                        })
                        .center()
                        .py(28.0)
                        .gap(5.0);
                    } else {
                        for w in &state.host.active_worlds {
                            let btn_id = format!("mm_aw_{}", w.id);
                            if ui
                                .btn_with(&btn_id, |ui| {
                                    ui.row(|ui| {
                                        ui.col(|ui| {
                                            ui.txt(&w.name)
                                                .sz(11.5)
                                                .semibold()
                                                .fg(TEXT_PRIMARY)
                                                .ellipsis();
                                            let reg = if w.region.is_empty() {
                                                "Active Instance"
                                            } else {
                                                &w.region
                                            };
                                            ui.txt(reg).sz(10.0).fg(TEXT_MUTED).ellipsis();
                                        })
                                        .gap(1.0)
                                        .grow(1.0);

                                        let cnt = format!("{}", w.player_count.max(0));
                                        ui.txt(&cnt)
                                            .sz(10.5)
                                            .mono()
                                            .bold()
                                            .fg(accent)
                                            .bg(tint_alpha(accent, 30))
                                            .px(6.0)
                                            .py(2.0)
                                            .bevel_rounded(4.0, 1.3);
                                    })
                                    .align_center()
                                    .gap(6.0)
                                    .grow(1.0);
                                })
                                .px(8.0)
                                .py(6.0)
                                .bg(CARD_INNER_BG)
                                .hover_bg(TILE_IDLE_HOVER)
                                .bd(1.0, BORDER_SUBTLE)
                                .bevel_rounded(6.0, 1.3)
                                .clicked()
                            {
                                push_click(actions);
                                state.main_menu.active_tab = String::from("worlds");
                                actions.push(UiAction::ContentLoadDetail {
                                    kind: String::from("world"),
                                    id: w.id.clone(),
                                });
                            }
                        }
                    }
                })
                .grow(1.0)
                .gap(5.0)
                .no_scrollbar();
            })
            .grow(1.0)
            .p(9.0)
            .gap(7.0)
            .bg(CARD_BG)
            .bd(1.0, BORDER_SUBTLE)
            .bevel_rounded(10.5, 2.0)
            .shadow(14.0, SHADOW_CARD);
        })
        .w(268.0)
        .h_frac(1.0)
        .gap(9.0);

        // =====================================================================
        // CENTER COLUMN: GLOBAL SEARCH BAR + ACTIVITY FEED + QUICK ACTION STRIP
        // =====================================================================
        ui.col(|ui| {
            // --- GLOBAL SEARCH BAR (WIDE & CLEAR AT TOP OF CENTER COLUMN) ---
            ui.row(|ui| {
                icons::icon(ui, icons::SEARCH, 16.0).fg(accent);
                let mut inp = ui
                    .input("mm_search_input", &mut state.main_menu.search_input)
                    .placeholder("Search worlds, avatars, props, users...")
                    .bg(CARD_INNER_BG)
                    .fg(TEXT_PRIMARY)
                    .grow(1.0);
                let submitted = inp.submitted();
                if state.host.is_vr && inp.clicked() {
                    actions.push(UiAction::OpenVirtualKeyboard {
                        surface_id: 0,
                        field_id: String::from("mm_search_input"),
                        initial_text: state.main_menu.search_input.text(),
                        title: String::from("Global Search"),
                        placeholder: String::from("Search worlds, avatars, props, users..."),
                        max_len: 128,
                        hidden: false,
                    });
                }

                let go_clicked = ui
                    .btn_with("mm_btn_global_search", |ui| {
                        ui.txt("Search")
                            .sz(11.0)
                            .semibold()
                            .fg(TEXT_PRIMARY);
                    })
                    .px(12.0)
                    .py(6.0)
                    .bg(tint_alpha(accent, 48))
                    .hover_bg(tint_alpha(accent, 82))
                    .bd(1.0, tint_alpha(accent, 165))
                    .bevel_rounded(6.0, 1.3)
                    .clicked();

                if submitted || go_clicked {
                    let q = state.main_menu.search_input.text();
                    if !q.trim().is_empty() {
                        state.main_menu.last_search = q.clone();
                        state.main_menu.active_tab = String::from("search");
                        state.main_menu.search_filter_kind = String::from("all");
                        push_click(actions);
                        actions.push(UiAction::ContentCloseDetail);
                        actions.push(UiAction::ContentGlobalSearch { query: q });
                    }
                }
            })
            .align_center()
            .px(10.0)
            .py(7.0)
            .gap(8.0)
            .bg(CARD_BG)
            .bd(1.0, BORDER_SUBTLE)
            .bevel_rounded(10.0, 2.0)
            .shadow(14.0, SHADOW_CARD);

            // --- ACTIVITY FEED CARD ---
            ui.col(|ui| {
                ui.row(|ui| {
                    icons::icon(ui, icons::BELL, 14.0).fg(accent);
                    ui.txt("Activity Feed")
                        .sz(12.5)
                        .semibold()
                        .fg(TEXT_PRIMARY)
                        .grow(1.0);
                })
                .align_center()
                .gap(6.0);

                if state.host.activity_feed.is_empty() && state.host.notifications.is_empty() {
                    ui.col(|ui| {
                        icons::icon(ui, icons::BELL, 28.0)
                            .fg(Color::rgba(148, 163, 184, 65));
                        ui.txt("No recent activity")
                            .sz(12.5)
                            .semibold()
                            .fg(TEXT_MUTED);
                        ui.txt("Friend updates, invites, and instance events appear here.")
                            .sz(11.0)
                            .fg(TEXT_DIM);
                    })
                    .center()
                    .grow(1.0)
                    .gap(5.0);
                } else {
                    ui.scroll("mm_activity_scroll", |ui| {
                        for notif in &state.host.notifications {
                            ui.row(|ui| {
                                ui.col(|ui| {
                                    ui.txt(&notif.title)
                                        .sz(12.0)
                                        .semibold()
                                        .fg(TEXT_PRIMARY);
                                    if !notif.body.is_empty() {
                                        ui.txt(&notif.body).sz(11.0).fg(TEXT_MUTED).wrap();
                                    }
                                })
                                .gap(2.0)
                                .grow(1.0);

                                if !notif.buttons.is_empty() {
                                    ui.row(|ui| {
                                        for btn in &notif.buttons {
                                            let btn_id = format!("mm_nbtn_{}", btn.id);
                                            if ui
                                                .btn_with(&btn_id, |ui| {
                                                    ui.txt(&btn.text)
                                                        .sz(10.5)
                                                        .semibold()
                                                        .fg(TEXT_PRIMARY);
                                                })
                                                .px(9.0)
                                                .py(5.0)
                                                .bg(if btn.is_accept {
                                                    tint_alpha(EMERALD, 55)
                                                } else if btn.is_decline {
                                                    tint_alpha(RED, 55)
                                                } else {
                                                    TILE_IDLE_BG
                                                })
                                                .hover_bg(tint_alpha(accent, 80))
                                                .bd(1.0, BORDER_SUBTLE)
                                                .bevel_rounded(5.0, 1.3)
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
                                    .gap(5.0);
                                }
                            })
                            .align_center()
                            .p(9.0)
                            .gap(8.0)
                            .bg(CARD_INNER_BG)
                            .bd(1.0, tint_alpha(accent, 80))
                            .bevel_rounded(7.0, 1.3);
                        }

                        for item in &state.host.activity_feed {
                            ui.row(|ui| {
                                ui.col(|ui| {
                                    ui.txt(&item.title).sz(12.0).semibold().fg(TEXT_PRIMARY);
                                    if !item.detail.is_empty() {
                                        ui.txt(&item.detail).sz(11.0).fg(TEXT_MUTED).wrap();
                                    }
                                })
                                .gap(2.0)
                                .grow(1.0);

                                if !item.time_text.is_empty() {
                                    ui.txt(&item.time_text).sz(10.0).mono().fg(TEXT_DIM);
                                }
                            })
                            .align_center()
                            .p(9.0)
                            .gap(8.0)
                            .bg(CARD_INNER_BG)
                            .bd(1.0, BORDER_SUBTLE)
                            .bevel_rounded(7.0, 1.3);
                        }
                    })
                    .grow(1.0)
                    .gap(6.0)
                    .no_scrollbar();
                }
            })
            .grow(1.0)
            .p(11.0)
            .gap(8.0)
            .bg(CARD_BG)
            .bd(1.0, BORDER_SUBTLE)
            .bevel_rounded(10.5, 2.0)
            .shadow(14.0, SHADOW_CARD);

            // --- QUICK ACTION BAR BELOW ACTIVITY FEED (WITH SPLIT PROP BUTTON) ---
            ui.row(|ui| {
                // 1. Current Instance
                if ui
                    .btn_with("mm_act_instance", |ui| {
                        ui.col(|ui| {
                            icons::icon(ui, icons::COMPASS, 20.0).fg(accent);
                            ui.txt("Current Instance")
                                .sz(11.0)
                                .semibold()
                                .align(TextAlign::Center)
                                .fg(TEXT_PRIMARY);
                        })
                        .center()
                        .gap(5.0)
                        .grow(1.0);
                    })
                    .grow(1.0)
                    .h_frac(1.0)
                    .p(7.0)
                    .bg(TILE_IDLE_BG)
                    .hover_bg(TILE_IDLE_HOVER)
                    .bd(1.0, BORDER_SUBTLE)
                    .bevel_rounded(8.0, 1.3)
                    .clicked()
                {
                    push_click(actions);
                    state.main_menu.active_tab = String::from("currentInstance");
                }

                // 2. Split Clear Button: Top = Remove All Avatars | Bottom = Remove All Props
                ui.col(|ui| {
                    if ui
                        .btn_with("mm_act_rm_avatars", |ui| {
                            ui.row(|ui| {
                                icons::icon(ui, icons::AVATAR, 14.0)
                                    .fg(Color::rgb(203, 213, 225));
                                ui.txt("Remove All Avatars")
                                    .sz(10.5)
                                    .semibold()
                                    .fg(TEXT_PRIMARY);
                            })
                            .center()
                            .gap(5.0)
                            .grow(1.0);
                        })
                        .grow(1.0)
                        .w_frac(1.0)
                        .px(6.0)
                        .py(4.0)
                        .bg(TILE_IDLE_BG)
                        .hover_bg(TILE_IDLE_HOVER)
                        .bd(1.0, BORDER_SUBTLE)
                        .bevel_rounded(6.0, 1.3)
                        .clicked()
                    {
                        push_click(actions);
                        actions.push(UiAction::RemoveAllAvatars);
                    }

                    if ui
                        .btn_with("mm_act_rm_all_props", |ui| {
                            ui.row(|ui| {
                                icons::icon(ui, icons::TRASH, 14.0).fg(RED);
                                ui.txt("Remove All Props")
                                    .sz(10.5)
                                    .semibold()
                                    .fg(TEXT_PRIMARY);
                            })
                            .center()
                            .gap(5.0)
                            .grow(1.0);
                        })
                        .grow(1.0)
                        .w_frac(1.0)
                        .px(6.0)
                        .py(4.0)
                        .bg(TILE_IDLE_BG)
                        .hover_bg(tint_alpha(RED, 55))
                        .bd(1.0, tint_alpha(RED, 95))
                        .bevel_rounded(6.0, 1.3)
                        .clicked()
                    {
                        push_click(actions);
                        actions.push(UiAction::RemoveAllProps);
                    }
                })
                .grow(1.15)
                .h_frac(1.0)
                .gap(5.0);

                // 3. Split Prop Button: Top = Delete My Props | Bottom = Delete Mode
                ui.col(|ui| {
                    if ui
                        .btn_with("mm_act_rm_my_props", |ui| {
                            ui.row(|ui| {
                                icons::icon(ui, icons::CUBE, 14.0).fg(accent);
                                ui.txt("Delete My Props")
                                    .sz(10.5)
                                    .semibold()
                                    .fg(TEXT_PRIMARY);
                            })
                            .center()
                            .gap(5.0)
                            .grow(1.0);
                        })
                        .grow(1.0)
                        .w_frac(1.0)
                        .px(6.0)
                        .py(4.0)
                        .bg(TILE_IDLE_BG)
                        .hover_bg(TILE_IDLE_HOVER)
                        .bd(1.0, BORDER_SUBTLE)
                        .bevel_rounded(6.0, 1.3)
                        .clicked()
                    {
                        push_click(actions);
                        actions.push(UiAction::RemoveMyProps);
                    }

                    if ui
                        .btn_with("mm_act_prop_del_mode", |ui| {
                            ui.row(|ui| {
                                icons::icon(ui, icons::TRASH, 14.0).fg(AMBER);
                                ui.txt("Delete Mode")
                                    .sz(10.5)
                                    .semibold()
                                    .fg(TEXT_PRIMARY);
                            })
                            .center()
                            .gap(5.0)
                            .grow(1.0);
                        })
                        .grow(1.0)
                        .w_frac(1.0)
                        .px(6.0)
                        .py(4.0)
                        .bg(TILE_IDLE_BG)
                        .hover_bg(tint_alpha(AMBER, 55))
                        .bd(1.0, tint_alpha(AMBER, 90))
                        .bevel_rounded(6.0, 1.3)
                        .clicked()
                    {
                        push_click(actions);
                        actions.push(UiAction::TogglePropDeleteMode);
                    }
                })
                .grow(1.15)
                .h_frac(1.0)
                .gap(5.0);

                // 4. Recalibrate
                if ui
                    .btn_with("mm_act_recalibrate", |ui| {
                        ui.col(|ui| {
                            icons::icon(ui, icons::FBT, 20.0).fg(Color::rgb(203, 213, 225));
                            ui.txt("Recalibrate")
                                .sz(11.0)
                                .semibold()
                                .align(TextAlign::Center)
                                .fg(TEXT_PRIMARY);
                        })
                        .center()
                        .gap(5.0)
                        .grow(1.0);
                    })
                    .grow(1.0)
                    .h_frac(1.0)
                    .p(7.0)
                    .bg(TILE_IDLE_BG)
                    .hover_bg(TILE_IDLE_HOVER)
                    .bd(1.0, BORDER_SUBTLE)
                    .bevel_rounded(8.0, 1.3)
                    .clicked()
                {
                    push_click(actions);
                    actions.push(UiAction::RecalibrateAvatar);
                }
            })
            .h(78.0)
            .p(8.0)
            .gap(7.0)
            .bg(CARD_BG)
            .bd(1.0, BORDER_SUBTLE)
            .bevel_rounded(10.5, 2.0)
            .shadow(14.0, SHADOW_CARD);
        })
        .grow(1.0)
        .h_frac(1.0)
        .gap(9.0);

        // =====================================================================
        // RIGHT COLUMN: SLIM MEDIA CONTROL CARD (TOP) + QUICK TOOLS CARD (BOTTOM)
        // =====================================================================
        ui.col(|ui| {
            // --- TOP RIGHT: SLIM MEDIA CONTROLS BOX ---
            ui.row(|ui| {
                icons::icon(ui, icons::VOLUME, 14.0).fg(accent);
                ui.txt("Media")
                    .sz(11.5)
                    .semibold()
                    .fg(TEXT_PRIMARY)
                    .grow(1.0);

                for (id, icon, act) in [
                    ("mm_med_prev", icons::PREV, UiAction::MediaPrev),
                    ("mm_med_play", icons::PLAY_PAUSE, UiAction::MediaPlayPause),
                    ("mm_med_next", icons::NEXT, UiAction::MediaNext),
                ] {
                    if ui
                        .btn_with(id, |ui| {
                            icons::icon(ui, icon, 15.0).fg(TEXT_PRIMARY);
                        })
                        .px(11.0)
                        .py(6.0)
                        .bg(TILE_IDLE_BG)
                        .hover_bg(TILE_IDLE_HOVER)
                        .bd(1.0, BORDER_SUBTLE)
                        .bevel_rounded(5.5, 1.3)
                        .clicked()
                    {
                        push_click(actions);
                        actions.push(act);
                    }
                }
            })
            .align_center()
            .px(10.0)
            .py(7.0)
            .gap(6.0)
            .bg(CARD_BG)
            .bd(1.0, BORDER_SUBTLE)
            .bevel_rounded(10.0, 2.0)
            .shadow(14.0, SHADOW_CARD);

            // --- BOTTOM RIGHT: 3-COLUMN UTILITY TILE GRID ---
            ui.col(|ui| {
                const MM_DEFAULT_TOOLS: [&str; 12] = [
                    "vr", "seated", "flight", "respawn", "camera", "advav", "chat", "cursor",
                    "mirror", "home", "fbt", "mods",
                ];
                let mut tool_order = crate::state::resolve_tile_order(
                    &state.host.mm_quick_tools_order,
                    &MM_DEFAULT_TOOLS,
                );
                let reorder_mode = state.main_menu.mm_tools_reorder_mode;

                ui.row(|ui| {
                    icons::icon(ui, icons::BOLT, 14.0).fg(accent);
                    ui.txt("Quick Tools")
                        .sz(12.0)
                        .semibold()
                        .fg(TEXT_PRIMARY)
                        .grow(1.0);

                    if reorder_mode {
                        if ui
                            .btn_with("mm_tools_reset_order", |ui| {
                                ui.txt("Reset").sz(9.5).semibold().fg(TEXT_MUTED);
                            })
                            .px(7.0)
                            .py(3.0)
                            .bg(TILE_IDLE_BG)
                            .hover_bg(TILE_IDLE_HOVER)
                            .bd(1.0, BORDER_SUBTLE)
                            .bevel_rounded(4.5, 1.0)
                            .clicked()
                        {
                            push_click(actions);
                            state.main_menu.mm_tools_reorder_selected = None;
                            state.host.mm_quick_tools_order.clear();
                            actions.push(UiAction::SetTileOrder {
                                target: String::from("mm_quick_tools"),
                                order: Vec::new(),
                            });
                        }
                    }

                    if ui
                        .btn_with("mm_tools_reorder_toggle", |ui| {
                            ui.txt(if reorder_mode { "Done" } else { "Reorder" })
                                .sz(9.5)
                                .semibold()
                                .fg(if reorder_mode { TEXT_PRIMARY } else { TEXT_MUTED });
                        })
                        .px(8.0)
                        .py(3.0)
                        .bg(if reorder_mode {
                            tint_alpha(accent, 52)
                        } else {
                            TILE_IDLE_BG
                        })
                        .hover_bg(TILE_IDLE_HOVER)
                        .bd(
                            1.0,
                            if reorder_mode {
                                tint_alpha(accent, 195)
                            } else {
                                BORDER_SUBTLE
                            },
                        )
                        .bevel_rounded(4.5, 1.0)
                        .clicked()
                    {
                        push_click(actions);
                        state.main_menu.mm_tools_reorder_mode = !reorder_mode;
                        state.main_menu.mm_tools_reorder_selected = None;
                    }
                })
                .align_center()
                .gap(6.0);

                let mut swapped_pair: Option<(usize, usize)> = None;
                for row_idx in 0..4 {
                    ui.row(|ui| {
                        for col_idx in 0..3 {
                            let slot_idx = row_idx * 3 + col_idx;
                            let tool_key = tool_order[slot_idx].as_str();
                            let is_selected = reorder_mode
                                && state.main_menu.mm_tools_reorder_selected == Some(slot_idx);

                            let (btn_id, icon, label, active, enabled, tile_accent) = match tool_key
                            {
                                "vr" => (
                                    "mm_util_vr",
                                    icons::VR_HEADSET,
                                    if state.host.is_vr {
                                        "Desktop"
                                    } else {
                                        "Switch VR"
                                    },
                                    state.host.is_vr,
                                    true,
                                    accent,
                                ),
                                "seated" => (
                                    "mm_util_seated",
                                    icons::SEATED,
                                    "Seated",
                                    state.host.is_seated,
                                    true,
                                    accent,
                                ),
                                "flight" => (
                                    "mm_util_flight",
                                    icons::FLIGHT,
                                    "Flight",
                                    state.host.is_flying,
                                    state.host.flight_allowed,
                                    accent,
                                ),
                                "respawn" => (
                                    "mm_util_respawn",
                                    icons::RESPAWN,
                                    "Respawn",
                                    false,
                                    true,
                                    accent,
                                ),
                                "camera" => (
                                    "mm_util_cam",
                                    icons::CAMERA,
                                    if state.host.camera_open {
                                        "Cam On"
                                    } else {
                                        "Camera"
                                    },
                                    state.host.camera_open,
                                    true,
                                    accent,
                                ),
                                "advav" => (
                                    "mm_util_advav",
                                    icons::AVATAR,
                                    "Adv Avtr",
                                    false,
                                    true,
                                    accent,
                                ),
                                "chat" => (
                                    "mm_util_chat",
                                    icons::CHAT,
                                    "Text Chat",
                                    false,
                                    true,
                                    accent,
                                ),
                                "cursor" => (
                                    "mm_util_cursor",
                                    icons::CURSOR,
                                    "Cursorlock",
                                    false,
                                    true,
                                    accent,
                                ),
                                "mirror" => (
                                    "mm_util_mirror",
                                    icons::MIRROR,
                                    "Mirror",
                                    state.host.mirror_open,
                                    true,
                                    accent,
                                ),
                                "home" => (
                                    "mm_util_home",
                                    icons::HOME,
                                    if state.main_menu.go_home_confirm {
                                        "Confirm?"
                                    } else {
                                        "Go Home"
                                    },
                                    state.main_menu.go_home_confirm,
                                    true,
                                    if state.main_menu.go_home_confirm {
                                        AMBER
                                    } else {
                                        accent
                                    },
                                ),
                                "fbt" => (
                                    "mm_util_fbt",
                                    icons::FBT,
                                    "Full Body",
                                    state.host.fbt_enabled && state.host.fbt_available,
                                    state.host.fbt_available,
                                    accent,
                                ),
                                _ => (
                                    "mm_util_mods",
                                    icons::PUZZLE,
                                    "Mods",
                                    state.main_menu.active_tab == "modSettings",
                                    state.host.uix_available,
                                    accent,
                                ),
                            };

                            let eff_active = if reorder_mode { is_selected } else { active };
                            let eff_enabled = if reorder_mode { true } else { enabled };
                            let eff_accent = if is_selected { AMBER } else { tile_accent };

                            if util_tile(
                                ui,
                                btn_id,
                                icon,
                                label,
                                eff_active,
                                eff_enabled,
                                eff_accent,
                            ) {
                                push_click(actions);
                                if reorder_mode {
                                    if let Some(prev) = state.main_menu.mm_tools_reorder_selected {
                                        if prev == slot_idx {
                                            state.main_menu.mm_tools_reorder_selected = None;
                                        } else {
                                            swapped_pair = Some((prev, slot_idx));
                                            state.main_menu.mm_tools_reorder_selected = None;
                                        }
                                    } else {
                                        state.main_menu.mm_tools_reorder_selected = Some(slot_idx);
                                    }
                                } else {
                                    match tool_key {
                                        "vr" => actions.push(UiAction::SwitchVrMode),
                                        "seated" => {
                                            actions.push(UiAction::ToggleSeatedOrRecalibrate)
                                        }
                                        "flight" => actions.push(UiAction::ToggleFlight),
                                        "respawn" => actions.push(UiAction::Respawn),
                                        "camera" => actions.push(UiAction::ToggleCamera),
                                        "advav" => {
                                            state.main_menu.active_tab =
                                                String::from("avatarSettings");
                                        }
                                        "chat" => actions.push(UiAction::OpenTextChat),
                                        "cursor" => actions.push(UiAction::ToggleCursorLock),
                                        "mirror" => actions.push(UiAction::ToggleMirror),
                                        "home" => {
                                            if state.main_menu.go_home_confirm {
                                                state.main_menu.go_home_confirm = false;
                                                actions.push(UiAction::GoHome);
                                            } else {
                                                state.main_menu.go_home_confirm = true;
                                            }
                                        }
                                        "fbt" => actions.push(UiAction::ToggleFullBody),
                                        _ => {
                                            state.main_menu.active_tab =
                                                String::from("modSettings");
                                            actions.push(UiAction::RefreshUixSettings);
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
                    tool_order.swap(a, b);
                    state.host.mm_quick_tools_order = tool_order.clone();
                    actions.push(UiAction::SetTileOrder {
                        target: String::from("mm_quick_tools"),
                        order: tool_order,
                    });
                }
            })
            .grow(1.0)
            .p(10.0)
            .gap(7.0)
            .bg(CARD_BG)
            .bd(1.0, BORDER_SUBTLE)
            .bevel_rounded(10.5, 2.0)
            .shadow(14.0, SHADOW_CARD);
        })
        .w(306.0)
        .h_frac(1.0)
        .gap(9.0);
    })
    .w_frac(1.0)
    .grow(1.0)
    .gap(10.0);
}

fn render_placeholder_page(
    ui: &mut UiContext,
    state: &mut AppState,
    tab_id: &str,
    accent: Color,
    actions: &mut Vec<UiAction>,
) {
    let title = match tab_id {
        "friends" => "Friends",
        "avatars" => "Avatars",
        "worlds" => "Worlds",
        "props" => "Props",
        "messages" => "Messages",
        "discover" => "Discover",
        "settings" => "Settings",
        "currentInstance" => "Current Instance",
        _ => "Menu Page",
    };

    ui.col(|ui| {
        icons::icon(ui, icons::PUZZLE, 34.0).fg(accent);
        ui.txt(title).sz(20.0).bold().fg(TEXT_PRIMARY);
        ui.txt("This page is not yet implemented in LumenicMenu.")
            .sz(12.5)
            .fg(TEXT_MUTED);

        ui.row(|ui| {
            if ui
                .btn("Back to Home")
                .px(16.0)
                .py(9.0)
                .bg(TILE_IDLE_BG)
                .hover_bg(TILE_IDLE_HOVER)
                .bd(1.0, BORDER_SUBTLE)
                .bevel_rounded(7.0, 1.3)
                .clicked()
            {
                push_click(actions);
                state.main_menu.active_tab = String::from("home");
            }
        })
        .gap(10.0)
        .pt(6.0);
    })
    .center()
    .grow(1.0)
    .gap(8.0)
    .bg(CARD_BG)
    .bd(1.0, BORDER_SUBTLE)
    .bevel_rounded(11.0, 2.0)
    .shadow(14.0, SHADOW_CARD);
}

fn dock_nav_btn(
    ui: &mut UiContext,
    id: &str,
    icon: &str,
    label: &str,
    active: bool,
    accent: Color,
) -> bool {
    let (bg, hover_bg, bd, fg, icon_col) = if active {
        (
            tint_alpha(accent, 58),
            tint_alpha(accent, 85),
            tint_alpha(accent, 210),
            TEXT_PRIMARY,
            accent,
        )
    } else {
        (
            TILE_IDLE_BG,
            TILE_IDLE_HOVER,
            BORDER_SUBTLE,
            TEXT_MUTED,
            Color::rgb(203, 213, 225),
        )
    };

    ui.btn_with(id, |ui| {
        ui.col(|ui| {
            icons::icon(ui, icon, 16.0).fg(icon_col);
            ui.txt(label)
                .sz(10.0)
                .semibold()
                .align(TextAlign::Center)
                .fg(fg);
        })
        .center()
        .grow(1.0)
        .gap(2.5);
    })
    .grow(1.0)
    .min_w(0.0)
    .h(46.0)
    .px(3.0)
    .py(4.0)
    .bg(bg)
    .hover_bg(hover_bg)
    .bd(if active { 1.5 } else { 1.0 }, bd)
    .bevel_rounded(6.5, 1.3)
    .clicked()
}

fn render_bottom_dock(
    ui: &mut UiContext,
    state: &mut AppState,
    accent: Color,
    actions: &mut Vec<UiAction>,
) {
    ui.row(|ui| {
        // Left 5 navigation items (symmetric 5 | Clock | 5 dock)
        ui.row(|ui| {
            for (tab_id, icon, label) in [
                ("home", icons::HOME, "Home"),
                ("friends", icons::PLAYERS, "Friends"),
                ("avatars", icons::AVATAR, "Avatars"),
                ("worlds", icons::WORLD, "Worlds"),
                ("props", icons::CUBE, "Props"),
            ] {
                let btn_id = format!("mm_dock_{tab_id}");
                if dock_nav_btn(
                    ui,
                    &btn_id,
                    icon,
                    label,
                    state.main_menu.active_tab == tab_id,
                    accent,
                ) {
                    state.main_menu.quit_confirm = false;
                    state.main_menu.go_home_confirm = false;
                    push_click(actions);
                    state.main_menu.active_tab = String::from(tab_id);
                    actions.push(UiAction::SocialCloseUserDetail);
                    actions.push(UiAction::SocialCloseGroupDetail);
                    actions.push(UiAction::ContentCloseInstanceDetail);
                    if tab_id == "worlds" || tab_id == "avatars" || tab_id == "props" {
                        state.main_menu.browser_local_filter.set_text("");
                        actions.push(UiAction::ContentCloseDetail);
                        let list = match tab_id {
                            "worlds" => &state.host.content.worlds,
                            "avatars" => &state.host.content.avatars,
                            _ => &state.host.content.props,
                        };
                        if !list.loaded_once && !list.loading {
                            actions.push(UiAction::ContentLoadList {
                                kind: String::from(tab_id),
                                category_key: list.category_key.clone(),
                                is_system: list.is_system,
                                page: 0,
                                sort_order: list.sort_order,
                                ascending: list.ascending,
                            });
                        }
                    }
                }
            }
        })
        .grow(5.0)
        .gap(6.0);

        // Center Large Clock + Date Display (dead center between 5 left & 5 right buttons)
        ui.col(|ui| {
            let clock = if state.host.clock_text.is_empty() {
                "--:--"
            } else {
                &state.host.clock_text
            };
            ui.txt(clock)
                .sz(19.5)
                .mono()
                .bold()
                .align(TextAlign::Center)
                .fg(TEXT_PRIMARY);

            if !state.host.date_text.is_empty() {
                ui.txt(&state.host.date_text)
                    .sz(10.0)
                    .align(TextAlign::Center)
                    .fg(TEXT_MUTED);
            }
        })
        .center()
        .w(136.0)
        .h(46.0)
        .px(6.0)
        .gap(1.0);

        // Right 5 controls: Mute | Messages | Discover | Settings | Exit
        ui.row(|ui| {
            let (mic_icon, mic_lbl, mic_col) = if state.host.is_muted {
                (icons::MIC_OFF, "Unmute", RED)
            } else {
                (icons::MIC, "Mute", EMERALD)
            };
            if dock_nav_btn(ui, "mm_dock_mute", mic_icon, mic_lbl, true, mic_col) {
                state.main_menu.quit_confirm = false;
                actions.push(UiAction::ToggleMute);
            }

            for (tab_id, icon, label) in [
                ("messages", icons::MAIL, "Messages"),
                ("discover", icons::COMPASS, "Discover"),
                ("settings", icons::SETTINGS, "Settings"),
            ] {
                let btn_id = format!("mm_dock_{tab_id}");
                if dock_nav_btn(
                    ui,
                    &btn_id,
                    icon,
                    label,
                    state.main_menu.active_tab == tab_id,
                    accent,
                ) {
                    state.main_menu.quit_confirm = false;
                    state.main_menu.go_home_confirm = false;
                    push_click(actions);
                    state.main_menu.active_tab = String::from(tab_id);
                    actions.push(UiAction::SocialCloseUserDetail);
                    actions.push(UiAction::SocialCloseGroupDetail);
                    actions.push(UiAction::ContentCloseInstanceDetail);
                    if tab_id == "discover" {
                        actions.push(UiAction::ContentCloseDetail);
                        if !state.host.content.discover_loaded
                            && !state.host.content.discover_loading
                            && state.host.content.discover_error.is_empty()
                        {
                            actions.push(UiAction::ContentLoadDiscover);
                        }
                    }
                }
            }

            // Exit (2-step confirm)
            let exit_lbl = if state.main_menu.quit_confirm {
                "Confirm?"
            } else {
                "Exit"
            };
            if dock_nav_btn(
                ui,
                "mm_dock_exit",
                icons::LOGOUT,
                exit_lbl,
                state.main_menu.quit_confirm,
                if state.main_menu.quit_confirm { AMBER } else { RED },
            ) {
                if state.main_menu.quit_confirm {
                    state.main_menu.quit_confirm = false;
                    push_click(actions);
                    actions.push(UiAction::QuitApplication);
                } else {
                    state.main_menu.quit_confirm = true;
                    actions.push(UiAction::PlayUiSound {
                        sound: String::from("Select"),
                    });
                }
            }
        })
        .grow(5.0)
        .gap(6.0);
    })
    .w_frac(1.0)
    .align_center()
    .px(9.0)
    .py(6.0)
    .gap(8.0)
    .bg(CARD_BG)
    .bd(1.0, BORDER_SUBTLE)
    .bevel_rounded(10.0, 2.0)
    .shadow(14.0, SHADOW_CARD);
}

fn modal_info_row(ui: &mut UiContext, label: &str, value: &str, value_col: Color) {
    ui.row(|ui| {
        ui.txt(label).sz(11.5).fg(TEXT_MUTED).grow(1.0);
        ui.txt(value)
            .sz(11.5)
            .mono()
            .semibold()
            .align(TextAlign::Right)
            .fg(value_col);
    })
    .align_center()
    .px(10.0)
    .py(6.0)
    .bg(CARD_INNER_BG)
    .bd(1.0, BORDER_SUBTLE)
    .bevel_rounded(5.0, 1.3);
}

fn render_game_instance_info_modal(
    ui: &mut UiContext,
    state: &mut AppState,
    accent: Color,
    actions: &mut Vec<UiAction>,
) {
    ui.overlay(|ui| {
        ui.zstack(|ui| {
            if ui
                .btn_with("mm_info_modal_scrim", |_| {})
                .w_frac(1.0)
                .h_frac(1.0)
                .bg(Color::rgba(4, 7, 13, 175))
                .hover_bg(Color::rgba(4, 7, 13, 175))
                .borderless()
                .r(0.0)
                .clicked()
            {
                push_click(actions);
                state.main_menu.info_modal_open = false;
            }

            ui.col(|ui| {
                let mut card = ui
                    .col(|ui| {
                        ui.row(|ui| {
                            ui.txt("Game & Instance Details")
                                .sz(15.0)
                                .bold()
                                .fg(TEXT_PRIMARY)
                                .grow(1.0);
                            if ui
                                .btn_with("mm_info_modal_close", |ui| {
                                    icons::icon(ui, icons::CLOSE, 14.0).fg(TEXT_MUTED);
                                })
                                .p(5.0)
                                .bg(Color::TRANSPARENT)
                                .hover_bg(Color::rgba(239, 68, 68, 80))
                                .bevel_rounded(4.5, 1.3)
                                .clicked()
                            {
                                push_click(actions);
                                state.main_menu.info_modal_open = false;
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
                        let fps_v = format!("{} FPS", state.host.fps.max(0));
                        let ping_v = format!("{} ms", state.host.game_ping.max(0));
                        let vc_v = format!("{} ms", state.host.voice_ping.max(0));
                        let ppl_v = format!("{}", state.host.instance_players.max(1));

                        modal_info_row(ui, "FPS", &fps_v, fps_col);
                        modal_info_row(ui, "Game Ping", &ping_v, TEXT_PRIMARY);
                        modal_info_row(ui, "Voice Ping", &vc_v, TEXT_PRIMARY);
                        modal_info_row(
                            ui,
                            "Platform / Mode",
                            if state.host.is_vr { "PC VR" } else { "PC Desktop" },
                            accent,
                        );
                        if !state.host.cvr_version.is_empty() {
                            let ver = if state.host.cvr_branch.is_empty() {
                                state.host.cvr_version.clone()
                            } else {
                                format!("{} ({})", state.host.cvr_version, state.host.cvr_branch)
                            };
                            modal_info_row(ui, "Version", &ver, TEXT_MUTED);
                        }
                        if !state.host.gs_version.is_empty() {
                            modal_info_row(ui, "Game Server (GS2)", &state.host.gs_version, TEXT_MUTED);
                        }
                        modal_info_row(
                            ui,
                            "World",
                            if state.host.world_name.is_empty() {
                                "Local World"
                            } else {
                                &state.host.world_name
                            },
                            TEXT_PRIMARY,
                        );
                        modal_info_row(
                            ui,
                            "Instance",
                            if state.host.instance_name.is_empty() {
                                "Local Instance"
                            } else {
                                &state.host.instance_name
                            },
                            TEXT_PRIMARY,
                        );
                        if !state.host.instance_owner.is_empty() {
                            modal_info_row(ui, "Instance Owner", &state.host.instance_owner, TEXT_PRIMARY);
                        }
                        modal_info_row(
                            ui,
                            "Privacy",
                            if state.host.instance_privacy.is_empty() {
                                "Unknown"
                            } else if state.host.instance_locked {
                                "Locked"
                            } else {
                                &state.host.instance_privacy
                            },
                            if state.host.instance_locked { AMBER } else { accent },
                        );
                        modal_info_row(ui, "Players in Instance", &ppl_v, EMERALD);
                        if state.host.instance_max_props > 0 {
                            let props_v = format!("{} / user", state.host.instance_max_props);
                            modal_info_row(ui, "Prop Limit", &props_v, TEXT_MUTED);
                        }
                        if state.host.instance_uptime_mins >= 0 {
                            let hrs = state.host.instance_uptime_mins / 60;
                            let mins = state.host.instance_uptime_mins % 60;
                            let up_v = if hrs > 0 {
                                format!("{hrs}h {mins}m")
                            } else {
                                format!("{mins}m")
                            };
                            modal_info_row(ui, "Instance Uptime", &up_v, TEXT_MUTED);
                        }

                        if ui
                            .btn_with("mm_modal_open_inst", |ui| {
                                ui.row(|ui| {
                                    icons::icon(ui, icons::INSTANCE, 14.0).fg(accent);
                                    ui.txt("Open Instance Roster")
                                        .sz(11.5)
                                        .bold()
                                        .fg(TEXT_PRIMARY);
                                })
                                .center()
                                .grow(1.0)
                                .gap(6.0);
                            })
                            .mt(6.0)
                            .py(7.0)
                            .bg(tint_alpha(accent, 45))
                            .hover_bg(tint_alpha(accent, 78))
                            .bd(1.0, tint_alpha(accent, 180))
                            .bevel_rounded(6.5, 1.3)
                            .clicked()
                        {
                            push_click(actions);
                            state.main_menu.info_modal_open = false;
                            state.main_menu.active_tab = String::from("currentInstance");
                        }
                    })
                    .w(420.0)
                    .p(16.0)
                    .gap(6.5)
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
