mod group_detail;
mod user_detail;

pub use group_detail::render_group_detail_view;
pub use user_detail::render_user_detail_view;

use lumenic::{Color, ImageFit, TextAlign, UiContext};

use crate::icons;
use crate::state::{AppState, FriendEntryItem, RecentUserEntryItem, UiAction};
use crate::theme::{
    push_click, tint_alpha, AMBER, border_subtle, card_bg, card_inner_bg, EMERALD, RED,
    SHADOW_CARD, TEXT_DIM, TEXT_MUTED, TEXT_PRIMARY, tile_idle_bg, tile_idle_hover,
};

/// Renders the Phase 3C Friends page (Online / All / Offline filter + search + quick actions).
pub fn render_friends_page(
    ui: &mut UiContext,
    state: &mut AppState,
    accent: Color,
    actions: &mut Vec<UiAction>,
) {
    if let Some(gd) = state.host.social.group_detail.take() {
        render_group_detail_view(ui, state, &gd, accent, actions);
        state.host.social.group_detail = Some(gd);
        return;
    }
    if let Some(detail) = state.host.social.user_detail.take() {
        render_user_detail_view(ui, state, &detail, accent, actions);
        state.host.social.user_detail = Some(detail);
        return;
    }

    let online_count = state
        .host
        .social
        .friends
        .iter()
        .filter(|f| f.is_online)
        .count();
    let total_count = state.host.social.friends.len();
    let recent_count = state.host.social.recent_users.len();
    if !state.host.enable_recent_history && state.main_menu.friends_filter == 3 {
        state.main_menu.friends_filter = 0;
    }
    let is_recent_tab = state.host.enable_recent_history && state.main_menu.friends_filter == 3;

    ui.col(|ui| {
        // Header bar: Title + Filter + Search + Refresh
        ui.row(|ui| {
            icons::icon(ui, icons::PLAYERS, 17.0).fg(accent);
            let header_title = if is_recent_tab {
                format!("Recently Seen ({recent_count})")
            } else {
                format!("Friends ({online_count} / {total_count})")
            };
            ui.txt(&header_title)
                .sz(crate::theme::fs(15.0))
                .bold()
                .fg(TEXT_PRIMARY);

            ui.row(|ui| {
                icons::icon(ui, icons::SEARCH, 13.0).fg(TEXT_MUTED);
                let mut f_inp = ui
                    .input("mm_fr_search", &mut state.main_menu.friends_search_input)
                    .placeholder(if is_recent_tab {
                        "Search user, world, instance..."
                    } else {
                        "Search friends..."
                    })
                    .bg(card_inner_bg())
                    .fg(TEXT_PRIMARY)
                    .grow(1.0);
                if state.host.is_vr && f_inp.clicked() {
                    actions.push(UiAction::OpenVirtualKeyboard {
                        surface_id: 0,
                        field_id: String::from("mm_fr_search"),
                        initial_text: state.main_menu.friends_search_input.text(),
                        title: String::from(if is_recent_tab {
                            "Search Recently Seen"
                        } else {
                            "Search Friends"
                        }),
                        placeholder: String::from(if is_recent_tab {
                            "Search user, world, instance..."
                        } else {
                            "Search friends..."
                        }),
                        max_len: 96,
                        hidden: false,
                    });
                }
            })
            .align_center()
            .gap(6.0)
            .grow(1.0)
            .px(8.0);

            for (idx, lbl) in [
                (0usize, "Online"),
                (1usize, "All"),
                (2usize, "Offline"),
                (3usize, "Recently Seen"),
            ] {
                if idx == 3 && !state.host.enable_recent_history {
                    continue;
                }
                let active = state.main_menu.friends_filter == idx;
                if ui
                    .btn_with(&format!("mm_fr_f_{idx}"), |ui| {
                        ui.txt(lbl)
                            .sz(crate::theme::fs(10.5))
                            .semibold()
                            .align(TextAlign::Center)
                            .fg(if active { TEXT_PRIMARY } else { TEXT_MUTED })
                            .grow(1.0);
                    })
                    .px(10.0)
                    .py(5.5)
                    .bg(if active {
                        tint_alpha(accent, 50)
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
                    state.main_menu.friends_filter = idx;
                }
            }

            if ui
                .btn_with("mm_fr_refresh", |ui| {
                    icons::icon(ui, icons::RELOAD, 14.0).fg(TEXT_PRIMARY);
                })
                .p(6.0)
                .bg(tile_idle_bg())
                .hover_bg(tile_idle_hover())
                .bd(1.0, border_subtle())
                .bevel_rounded(5.5, 1.3)
                .clicked()
            {
                push_click(actions);
                actions.push(UiAction::SocialRefreshFriends);
            }
        })
        .align_center()
        .px(12.0)
        .py(8.0)
        .gap(7.0)
        .bg(card_bg())
        .bd(1.0, border_subtle())
        .bevel_rounded(10.0, 2.0)
        .shadow(14.0, SHADOW_CARD);

        // Friends Grid (3 columns) or Recently Seen Users Grid (2 columns)
        ui.col(|ui| {
            let q = state
                .main_menu
                .friends_search_input
                .text()
                .trim()
                .to_ascii_lowercase();
            let filter_mode = state.main_menu.friends_filter;
            if filter_mode == 3 {
                let filtered_recent: Vec<RecentUserEntryItem> = state
                    .host
                    .social
                    .recent_users
                    .iter()
                    .filter(|u| {
                        q.is_empty()
                            || u.name.to_ascii_lowercase().contains(&q)
                            || u.world_name.to_ascii_lowercase().contains(&q)
                            || u.instance_name.to_ascii_lowercase().contains(&q)
                            || u.instance_privacy.to_ascii_lowercase().contains(&q)
                            || u.rank.to_ascii_lowercase().contains(&q)
                    })
                    .cloned()
                    .collect();

                if filtered_recent.is_empty() {
                    ui.col(|ui| {
                        icons::icon(ui, icons::PLAYERS, 30.0).fg(TEXT_DIM);
                        ui.txt(if state.host.social.recent_users.is_empty() {
                            "No recently seen users yet"
                        } else {
                            "No recently seen users match your search"
                        })
                        .sz(crate::theme::fs(13.0))
                        .semibold()
                        .fg(TEXT_MUTED);
                    })
                    .center()
                    .grow(1.0)
                    .gap(6.0);
                } else {
                    ui.scroll("mm_fr_recent_grid", |ui| {
                        for chunk in filtered_recent.chunks(2) {
                            ui.row(|ui| {
                                for u in chunk {
                                    render_recent_user_card(ui, state, u, accent, actions);
                                }
                                for _ in chunk.len()..2 {
                                    ui.col(|_| {}).grow(1.0);
                                }
                            })
                            .gap(8.0);
                        }
                    })
                    .grow(1.0)
                    .gap(8.0)
                    .no_scrollbar();
                }
            } else {
                let filtered: Vec<FriendEntryItem> = state
                    .host
                    .social
                    .friends
                    .iter()
                    .filter(|f| {
                        let match_status = match filter_mode {
                            0 => f.is_online,
                            2 => !f.is_online,
                            _ => true,
                        };
                        let match_q = q.is_empty()
                            || f.name.to_ascii_lowercase().contains(&q)
                            || f.instance_name.to_ascii_lowercase().contains(&q);
                        match_status && match_q
                    })
                    .cloned()
                    .collect();

                if filtered.is_empty() {
                    ui.col(|ui| {
                        icons::icon(ui, icons::PLAYERS, 30.0).fg(TEXT_DIM);
                        ui.txt(if filter_mode == 0 {
                            "No friends online"
                        } else {
                            "No friends match your filter"
                        })
                        .sz(crate::theme::fs(13.0))
                        .semibold()
                        .fg(TEXT_MUTED);
                    })
                    .center()
                    .grow(1.0)
                    .gap(6.0);
                } else {
                    ui.scroll("mm_fr_grid", |ui| {
                        for chunk in filtered.chunks(3) {
                            ui.row(|ui| {
                                for fr in chunk {
                                    render_friend_card(ui, state, fr, accent, actions);
                                }
                                for _ in chunk.len()..3 {
                                    ui.col(|_| {}).grow(1.0);
                                }
                            })
                            .gap(8.0);
                        }
                    })
                    .grow(1.0)
                    .gap(8.0)
                    .no_scrollbar();
                }
            }
        })
        .grow(1.0)
        .p(10.0)
        .gap(8.0)
        .bg(card_bg())
        .bd(1.0, border_subtle())
        .bevel_rounded(10.5, 2.0)
        .shadow(14.0, SHADOW_CARD);
    })
    .grow(1.0)
    .gap(9.0);
}

fn privacy_badge_color(privacy: &str, accent: Color) -> Color {
    let lower = privacy.to_ascii_lowercase();
    if lower.contains("public") {
        EMERALD
    } else if lower.contains("invite") {
        AMBER
    } else if lower.contains("group") {
        Color::rgb(56, 189, 248)
    } else {
        accent
    }
}

fn render_recent_user_card(
    ui: &mut UiContext,
    state: &mut AppState,
    u: &RecentUserEntryItem,
    accent: Color,
    actions: &mut Vec<UiAction>,
) {
    let priv_col = privacy_badge_color(&u.instance_privacy, accent);
    let loc_label = if !u.world_name.is_empty() {
        u.world_name.as_str()
    } else if !u.instance_name.is_empty() {
        u.instance_name.as_str()
    } else {
        "Unknown World"
    };

    if ui
        .btn_with(&format!("mm_recent_u_{}", u.id), |ui| {
            ui.row(|ui| {
                // Avatar PFP
                if let Some(tex) = state.main_textures.get(&u.image_key) {
                    ui.img(tex)
                        .w(46.0)
                        .h(46.0)
                        .fit(ImageFit::Cover)
                        .hexagon_pointy()
                        .r(3.5)
                        .bd(1.5, tint_alpha(accent, 195));
                } else {
                    let initial = u
                        .name
                        .chars()
                        .next()
                        .unwrap_or('?')
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
                    .w(46.0)
                    .h(46.0)
                    .bg(tint_alpha(accent, 30))
                    .bd(1.5, tint_alpha(accent, 165))
                    .hexagon_pointy()
                    .r(3.5);
                }

                // Details column: Name + Rank + Timestamp, World + Privacy badge
                ui.col(|ui| {
                    ui.row(|ui| {
                        ui.txt(&u.name)
                            .sz(crate::theme::fs(13.0))
                            .bold()
                            .fg(TEXT_PRIMARY)
                            .ellipsis()
                            .grow(1.0);

                        if crate::theme::is_special_rank(&u.rank) {
                            ui.row(|ui| {
                                ui.txt(&u.rank)
                                    .sz(crate::theme::fs(9.5))
                                    .semibold()
                                    .fg(TEXT_PRIMARY);
                            })
                            .px(6.5)
                            .py(2.0)
                            .bg(tint_alpha(accent, 36))
                            .bd(1.0, tint_alpha(accent, 135))
                            .bevel_rounded(4.5, 1.0);
                        }

                        if !u.last_seen_text.is_empty() {
                            ui.txt(&u.last_seen_text)
                                .sz(crate::theme::fs(10.0))
                                .fg(TEXT_DIM);
                        }
                    })
                    .align_center()
                    .gap(6.0);

                    ui.row(|ui| {
                        icons::icon(ui, icons::WORLD, 11.5).fg(TEXT_DIM);
                        ui.txt(loc_label)
                            .sz(crate::theme::fs(10.5))
                            .fg(TEXT_MUTED)
                            .ellipsis()
                            .grow(1.0);

                        if !u.instance_privacy.is_empty() {
                            ui.row(|ui| {
                                ui.txt(&u.instance_privacy)
                                    .sz(crate::theme::fs(9.5))
                                    .semibold()
                                    .fg(priv_col);
                            })
                            .px(6.5)
                            .py(2.0)
                            .bg(tint_alpha(priv_col, 26))
                            .bd(1.0, tint_alpha(priv_col, 120))
                            .bevel_rounded(4.0, 1.0);
                        }
                    })
                    .align_center()
                    .gap(5.0);
                })
                .gap(5.0)
                .grow(1.0);
            })
            .align_center()
            .gap(10.0)
            .grow(1.0);
        })
        .grow(1.0)
        .p(9.0)
        .bg(card_inner_bg())
        .hover_bg(tile_idle_hover())
        .bd(1.0, border_subtle())
        .bevel_rounded(8.5, 1.5)
        .clicked()
    {
        push_click(actions);
        crate::main_menu::push_nav_history(state);
        state.main_menu.user_detail_tab = 0;
        actions.push(UiAction::SocialLoadUserDetail {
            user_id: u.id.clone(),
        });
    }
}

fn render_friend_card(
    ui: &mut UiContext,
    state: &mut AppState,
    fr: &FriendEntryItem,
    accent: Color,
    actions: &mut Vec<UiAction>,
) {
    let status_col = if fr.is_online { EMERALD } else { TEXT_DIM };
    ui.col(|ui| {
        if ui
            .btn_with(&format!("mm_frc_{}", fr.id), |ui| {
                ui.row(|ui| {
                    if let Some(tex) = state.main_textures.get(&fr.image_key) {
                        ui.img(tex)
                            .w(42.0)
                            .h(42.0)
                            .fit(ImageFit::Cover)
                            .hexagon_pointy().r(3.5)
                            .bd(1.5, status_col);
                    } else {
                        let initial = fr
                            .name
                            .chars()
                            .next()
                            .unwrap_or('?')
                            .to_ascii_uppercase()
                            .to_string();
                        ui.col(|ui| {
                            ui.txt(&initial)
                                .sz(crate::theme::fs(15.0))
                                .bold()
                                .align(TextAlign::Center)
                                .fg(status_col);
                        })
                        .center()
                        .w(42.0)
                        .h(42.0)
                        .bg(tint_alpha(status_col, 32))
                        .bd(1.5, status_col)
                        .hexagon_pointy().r(3.5);
                    }

                    ui.col(|ui| {
                        ui.txt(&fr.name)
                            .sz(crate::theme::fs(12.5))
                            .semibold()
                            .fg(TEXT_PRIMARY)
                            .ellipsis();
                        let loc = if !fr.is_online {
                            "Offline"
                        } else if !fr.instance_name.is_empty() {
                            &fr.instance_name
                        } else if fr.is_connected {
                            "Private Instance"
                        } else {
                            "Online"
                        };
                        ui.txt(loc).sz(crate::theme::fs(10.5)).fg(if fr.is_online {
                            TEXT_MUTED
                        } else {
                            TEXT_DIM
                        })
                        .ellipsis();
                    })
                    .gap(2.0)
                    .grow(1.0);
                })
                .align_center()
                .gap(9.0)
                .grow(1.0);
            })
            .p(6.0)
            .bg(Color::TRANSPARENT)
            .hover_bg(tile_idle_hover())
            .borderless()
            .bevel_rounded(6.0, 1.3)
            .clicked()
        {
            push_click(actions);
            crate::main_menu::push_nav_history(state);
            state.main_menu.user_detail_tab = 0;
            actions.push(UiAction::SocialLoadUserDetail {
                user_id: fr.id.clone(),
            });
        }

        if fr.is_online {
            ui.row(|ui| {
                if !fr.instance_id.is_empty()
                    && ui
                        .btn_with(&format!("mm_fr_join_{}", fr.id), |ui| {
                            ui.txt("Join")
                                .sz(crate::theme::fs(10.0))
                                .bold()
                                .align(TextAlign::Center)
                                .fg(TEXT_PRIMARY)
                                .grow(1.0);
                        })
                        .grow(1.0)
                        .py(4.5)
                        .bg(tint_alpha(EMERALD, 45))
                        .hover_bg(tint_alpha(EMERALD, 75))
                        .bd(1.0, tint_alpha(EMERALD, 180))
                        .bevel_rounded(5.0, 1.3)
                        .clicked()
                {
                    push_click(actions);
                    actions.push(UiAction::ContentJoinInstance {
                        instance_id: fr.instance_id.clone(),
                    });
                }

                if ui
                    .btn_with(&format!("mm_fr_inv_{}", fr.id), |ui| {
                        ui.txt("Invite")
                            .sz(crate::theme::fs(10.0))
                            .semibold()
                            .align(TextAlign::Center)
                            .fg(TEXT_PRIMARY)
                            .grow(1.0);
                    })
                    .grow(1.0)
                    .py(4.5)
                    .bg(tile_idle_bg())
                    .hover_bg(tile_idle_hover())
                    .bd(1.0, tint_alpha(accent, 130))
                    .bevel_rounded(5.0, 1.3)
                    .clicked()
                {
                    push_click(actions);
                    actions.push(UiAction::SocialInvitePlayer {
                        user_id: fr.id.clone(),
                    });
                }

                if ui
                    .btn_with(&format!("mm_fr_req_{}", fr.id), |ui| {
                        ui.txt("Req Inv")
                            .sz(crate::theme::fs(10.0))
                            .semibold()
                            .align(TextAlign::Center)
                            .fg(TEXT_MUTED)
                            .grow(1.0);
                    })
                    .grow(1.0)
                    .py(4.5)
                    .bg(tile_idle_bg())
                    .hover_bg(tile_idle_hover())
                    .bd(1.0, border_subtle())
                    .bevel_rounded(5.0, 1.3)
                    .clicked()
                {
                    push_click(actions);
                    actions.push(UiAction::SocialRequestInvite {
                        user_id: fr.id.clone(),
                    });
                }
            })
            .gap(5.0);
        }
    })
    .grow(1.0)
    .p(7.0)
    .gap(5.0)
    .bg(card_inner_bg())
    .bd(1.0, border_subtle())
    .bevel_rounded(8.0, 1.3);
}

/// Renders the Phase 3C Current Instance page (Instance summary + live player roster).
pub fn render_current_instance_page(
    ui: &mut UiContext,
    state: &mut AppState,
    accent: Color,
    actions: &mut Vec<UiAction>,
) {
    if let Some(gd) = state.host.social.group_detail.take() {
        render_group_detail_view(ui, state, &gd, accent, actions);
        state.host.social.group_detail = Some(gd);
        return;
    }
    if let Some(detail) = state.host.social.user_detail.take() {
        render_user_detail_view(ui, state, &detail, accent, actions);
        state.host.social.user_detail = Some(detail);
        return;
    }

    let world = if state.host.world_name.is_empty() {
        String::from("Local World")
    } else {
        state.host.world_name.clone()
    };
    let inst = if state.host.instance_name.is_empty() {
        String::from("Local Instance")
    } else {
        state.host.instance_name.clone()
    };
    let privacy = if state.host.instance_privacy.is_empty() {
        String::from("Unknown")
    } else {
        state.host.instance_privacy.clone()
    };

    ui.col(|ui| {
        // Header bar
        ui.row(|ui| {
            if ui
                .btn_with("mm_ci_back_home", |ui| {
                    ui.row(|ui| {
                        icons::icon(ui, icons::BACK, 14.0).fg(TEXT_PRIMARY);
                        ui.txt(crate::main_menu::back_button_label(state, "Home"))
                            .sz(crate::theme::fs(11.5))
                            .semibold()
                            .fg(TEXT_PRIMARY);
                    })
                    .center()
                    .gap(4.0);
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
                crate::main_menu::nav_go_back(state, "home", actions);
            }

            icons::icon(ui, icons::INSTANCE, 17.0).fg(accent);
            ui.col(|ui| {
                ui.txt(&inst).sz(crate::theme::fs(14.5)).bold().fg(TEXT_PRIMARY).ellipsis();
                ui.txt(&format!("{world}  |  {privacy}"))
                    .sz(crate::theme::fs(11.0))
                    .fg(TEXT_MUTED)
                    .ellipsis();
            })
            .gap(1.0)
            .grow(1.0);

            let cnt = format!("{} Players", state.host.social.instance_players.len().max(1));
            ui.txt(&cnt)
                .sz(crate::theme::fs(11.0))
                .bold()
                .fg(EMERALD)
                .bg(tint_alpha(EMERALD, 30))
                .px(9.0)
                .py(4.0)
                .bevel_rounded(5.0, 1.3);

            if ui
                .btn_with("mm_ci_drop_portal", |ui| {
                    ui.txt("Drop Portal")
                        .sz(crate::theme::fs(11.0))
                        .semibold()
                        .fg(TEXT_PRIMARY);
                })
                .px(11.0)
                .py(5.5)
                .bg(tint_alpha(accent, 42))
                .hover_bg(tint_alpha(accent, 75))
                .bd(1.0, tint_alpha(accent, 175))
                .bevel_rounded(6.0, 1.3)
                .clicked()
            {
                push_click(actions);
                actions.push(UiAction::ContentDropCurrentInstancePortal);
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

        // Player Roster Grid (3 columns)
        ui.col(|ui| {
            let players = state.host.social.instance_players.clone();
            if players.is_empty() {
                ui.col(|ui| {
                    ui.txt("No players in instance").sz(crate::theme::fs(13.0)).fg(TEXT_MUTED);
                })
                .center()
                .grow(1.0);
            } else {
                ui.scroll("mm_ci_players_scroll", |ui| {
                    for chunk in players.chunks(3) {
                        ui.row(|ui| {
                            for p in chunk {
                                let bd_col = if p.is_local {
                                    tint_alpha(accent, 180)
                                } else if p.is_friend {
                                    tint_alpha(EMERALD, 160)
                                } else {
                                    border_subtle()
                                };
                                if ui
                                    .btn_with(&format!("mm_cip_{}", p.id), |ui| {
                                        ui.row(|ui| {
                                            if let Some(tex) = state.main_textures.get(&p.image_key) {
                                                ui.img(tex)
                                                    .w(44.0)
                                                    .h(44.0)
                                                    .fit(ImageFit::Cover)
                                                    .hexagon_pointy().r(3.5)
                                                    .bd(1.5, bd_col);
                                            } else {
                                                let init = p
                                                    .name
                                                    .chars()
                                                    .next()
                                                    .unwrap_or('P')
                                                    .to_ascii_uppercase()
                                                    .to_string();
                                                ui.col(|ui| {
                                                    ui.txt(&init)
                                                        .sz(crate::theme::fs(16.0))
                                                        .bold()
                                                        .align(TextAlign::Center)
                                                        .fg(accent);
                                                })
                                                .center()
                                                .w(44.0)
                                                .h(44.0)
                                                .bg(tint_alpha(accent, 30))
                                                .bd(1.5, bd_col)
                                                .hexagon_pointy().r(3.5);
                                            }

                                            ui.col(|ui| {
                                                ui.txt(&p.name)
                                                    .sz(crate::theme::fs(12.5))
                                                    .semibold()
                                                    .fg(TEXT_PRIMARY)
                                                    .ellipsis();
                                                let special_rank = crate::theme::is_special_rank(&p.rank);
                                                let sub = if p.is_friend {
                                                    if !special_rank {
                                                        String::from("Friend")
                                                    } else {
                                                        format!("Friend  |  {}", p.rank)
                                                    }
                                                } else if !special_rank {
                                                    String::from("Player")
                                                } else {
                                                    p.rank.clone()
                                                };
                                                ui.txt(&sub).sz(crate::theme::fs(10.5)).fg(TEXT_MUTED).ellipsis();
                                            })
                                            .gap(2.0)
                                            .grow(1.0);
                                        })
                                        .align_center()
                                        .gap(10.0)
                                        .grow(1.0);
                                    })
                                    .grow(1.0)
                                    .p(10.0)
                                    .bg(card_inner_bg())
                                    .hover_bg(tile_idle_hover())
                                    .bd(1.0, bd_col)
                                    .bevel_rounded(8.0, 1.3)
                                    .clicked()
                                {
                                    push_click(actions);
                                    crate::main_menu::push_nav_history(state);
                                    state.main_menu.user_detail_tab = 0;
                                    actions.push(UiAction::SocialLoadUserDetail {
                                        user_id: p.id.clone(),
                                    });
                                }
                            }
                            for _ in chunk.len()..3 {
                                ui.col(|_| {}).grow(1.0);
                            }
                        })
                        .gap(8.0);
                    }
                })
                .grow(1.0)
                .gap(8.0)
                .no_scrollbar();
            }
        })
        .grow(1.0)
        .p(10.0)
        .gap(8.0)
        .bg(card_bg())
        .bd(1.0, border_subtle())
        .bevel_rounded(10.5, 2.0)
        .shadow(14.0, SHADOW_CARD);
    })
    .grow(1.0)
    .gap(9.0);
}

/// Renders the Phase 3C Messages / Invites / Friend Requests page.
pub fn render_messages_page(
    ui: &mut UiContext,
    state: &mut AppState,
    accent: Color,
    actions: &mut Vec<UiAction>,
) {
    ui.col(|ui| {
        // Header + category filter pills
        ui.row(|ui| {
            icons::icon(ui, icons::MAIL, 17.0).fg(accent);
            ui.txt(&format!("Messages & Invites ({})", state.host.notifications.len()))
                .sz(crate::theme::fs(15.0))
                .bold()
                .fg(TEXT_PRIMARY)
                .grow(1.0);

            for (key, ic, lbl) in [
                ("all", icons::APPS, "All"),
                ("Invite", icons::MAIL, "Invites"),
                ("InviteRequests", icons::BELL, "Requests"),
                ("FriendRequests", icons::PLAYERS, "Friends"),
                ("Votes", icons::CHECK, "Votes"),
                ("SystemNotif", icons::INFO, "System"),
            ] {
                let active = state.main_menu.messages_filter == key;
                if ui
                    .btn_with(&format!("mm_msg_f_{key}"), |ui| {
                        ui.row(|ui| {
                            icons::icon(ui, ic, 13.0)
                                .fg(if active { accent } else { TEXT_DIM });
                            ui.txt(lbl)
                                .sz(crate::theme::fs(11.0))
                                .semibold()
                                .fg(if active { TEXT_PRIMARY } else { TEXT_MUTED });
                        })
                        .center()
                        .gap(5.0);
                    })
                    .px(11.0)
                    .py(5.5)
                    .bg(if active {
                        tint_alpha(accent, 50)
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
                    .bevel_rounded(6.0, 1.3)
                    .clicked()
                {
                    push_click(actions);
                    state.main_menu.messages_filter = String::from(key);
                }
            }
        })
        .align_center()
        .px(12.0)
        .py(8.0)
        .gap(7.0)
        .bg(card_bg())
        .bd(1.0, border_subtle())
        .bevel_rounded(10.0, 2.0)
        .shadow(14.0, SHADOW_CARD);

        // Messages list
        ui.col(|ui| {
            let mf = state.main_menu.messages_filter.as_str();
            let filtered: Vec<_> = state
                .host
                .notifications
                .iter()
                .filter(|n| mf == "all" || n.category.eq_ignore_ascii_case(mf))
                .cloned()
                .collect();

            if filtered.is_empty() {
                ui.col(|ui| {
                    icons::icon(ui, icons::MAIL, 32.0).fg(TEXT_DIM);
                    ui.txt("No pending messages or invites")
                        .sz(crate::theme::fs(13.0))
                        .semibold()
                        .fg(TEXT_MUTED);
                })
                .center()
                .grow(1.0)
                .gap(6.0);
            } else {
                ui.scroll("mm_msg_scroll", |ui| {
                    for notif in &filtered {
                        ui.row(|ui| {
                            ui.col(|ui| {
                                ui.row(|ui| {
                                    ui.txt(&notif.title)
                                        .sz(crate::theme::fs(12.5))
                                        .bold()
                                        .fg(TEXT_PRIMARY);
                                    if !notif.category.is_empty() {
                                        ui.txt(&notif.category)
                                            .sz(crate::theme::fs(9.5))
                                            .fg(accent)
                                            .bg(tint_alpha(accent, 28))
                                            .px(6.0)
                                            .py(1.5)
                                            .bevel_rounded(4.0, 1.3);
                                    }
                                })
                                .align_center()
                                .gap(7.0);

                                if !notif.body.is_empty() {
                                    ui.txt(&notif.body).sz(crate::theme::fs(11.5)).fg(TEXT_MUTED).wrap();
                                }
                            })
                            .gap(3.0)
                            .grow(1.0);

                            ui.row(|ui| {
                                for btn in &notif.buttons {
                                    if ui
                                        .btn_with(&format!("mm_msg_b_{}", btn.id), |ui| {
                                            ui.txt(&btn.text)
                                                .sz(crate::theme::fs(11.0))
                                                .semibold()
                                                .fg(TEXT_PRIMARY);
                                        })
                                        .px(11.0)
                                        .py(6.0)
                                        .bg(if btn.is_accept {
                                            tint_alpha(EMERALD, 52)
                                        } else if btn.is_decline {
                                            tint_alpha(RED, 52)
                                        } else {
                                            tile_idle_bg()
                                        })
                                        .hover_bg(tile_idle_hover())
                                        .bd(1.0, border_subtle())
                                        .bevel_rounded(5.5, 1.3)
                                        .clicked()
                                    {
                                        push_click(actions);
                                        actions.push(UiAction::NotificationButtonClick {
                                            button_id: btn.id.clone(),
                                        });
                                    }
                                }

                                if ui
                                    .btn_with(&format!("mm_msg_x_{}", notif.id), |ui| {
                                        icons::icon(ui, icons::CLOSE, 13.0).fg(TEXT_MUTED);
                                    })
                                    .p(6.0)
                                    .bg(Color::TRANSPARENT)
                                    .hover_bg(tint_alpha(RED, 60))
                                    .bevel_rounded(5.0, 1.3)
                                    .clicked()
                                {
                                    push_click(actions);
                                    actions.push(UiAction::NotificationDismiss {
                                        message_id: notif.id.clone(),
                                    });
                                }
                            })
                            .align_center()
                            .gap(6.0);
                        })
                        .align_center()
                        .p(11.0)
                        .gap(10.0)
                        .bg(card_inner_bg())
                        .bd(1.0, border_subtle())
                        .bevel_rounded(8.0, 1.3);
                    }
                })
                .grow(1.0)
                .gap(7.0)
                .no_scrollbar();
            }
        })
        .grow(1.0)
        .p(10.0)
        .gap(8.0)
        .bg(card_bg())
        .bd(1.0, border_subtle())
        .bevel_rounded(10.5, 2.0)
        .shadow(14.0, SHADOW_CARD);
    })
    .grow(1.0)
    .gap(9.0);
}
