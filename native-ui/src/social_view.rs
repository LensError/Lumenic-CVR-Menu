use lumenic::{Color, ImageFit, TextAlign, UiContext};

use crate::icons;
use crate::state::{
    AppState, FriendEntryItem, GroupDetailSnapshot, RecentUserEntryItem, UiAction,
    UserDetailSnapshot,
};
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
    if let Some(gd) = state.host.social.group_detail.clone() {
        render_group_detail_view(ui, state, &gd, accent, actions);
        return;
    }
    if let Some(detail) = state.host.social.user_detail.clone() {
        render_user_detail_view(ui, state, &detail, accent, actions);
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
                .sz(15.0)
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
                            .sz(10.5)
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
                        .sz(13.0)
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
                        .sz(13.0)
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
                            .sz(16.0)
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
                            .sz(13.0)
                            .bold()
                            .fg(TEXT_PRIMARY)
                            .ellipsis()
                            .grow(1.0);

                        if crate::theme::is_special_rank(&u.rank) {
                            ui.row(|ui| {
                                ui.txt(&u.rank)
                                    .sz(9.5)
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
                                .sz(10.0)
                                .fg(TEXT_DIM);
                        }
                    })
                    .align_center()
                    .gap(6.0);

                    ui.row(|ui| {
                        icons::icon(ui, icons::WORLD, 11.5).fg(TEXT_DIM);
                        ui.txt(loc_label)
                            .sz(10.5)
                            .fg(TEXT_MUTED)
                            .ellipsis()
                            .grow(1.0);

                        if !u.instance_privacy.is_empty() {
                            ui.row(|ui| {
                                ui.txt(&u.instance_privacy)
                                    .sz(9.5)
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
                                .sz(15.0)
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
                            .sz(12.5)
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
                        ui.txt(loc).sz(10.5).fg(if fr.is_online {
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
                                .sz(10.0)
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
                            .sz(10.0)
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
                            .sz(10.0)
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
    if let Some(gd) = state.host.social.group_detail.clone() {
        render_group_detail_view(ui, state, &gd, accent, actions);
        return;
    }
    if let Some(detail) = state.host.social.user_detail.clone() {
        render_user_detail_view(ui, state, &detail, accent, actions);
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
                            .sz(11.5)
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
                ui.txt(&inst).sz(14.5).bold().fg(TEXT_PRIMARY).ellipsis();
                ui.txt(&format!("{world}  |  {privacy}"))
                    .sz(11.0)
                    .fg(TEXT_MUTED)
                    .ellipsis();
            })
            .gap(1.0)
            .grow(1.0);

            let cnt = format!("{} Players", state.host.social.instance_players.len().max(1));
            ui.txt(&cnt)
                .sz(11.0)
                .bold()
                .fg(EMERALD)
                .bg(tint_alpha(EMERALD, 30))
                .px(9.0)
                .py(4.0)
                .bevel_rounded(5.0, 1.3);

            if ui
                .btn_with("mm_ci_drop_portal", |ui| {
                    ui.txt("Drop Portal")
                        .sz(11.0)
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
                    ui.txt("No players in instance").sz(13.0).fg(TEXT_MUTED);
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
                                                        .sz(16.0)
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
                                                    .sz(12.5)
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
                                                ui.txt(&sub).sz(10.5).fg(TEXT_MUTED).ellipsis();
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
                .sz(15.0)
                .bold()
                .fg(TEXT_PRIMARY)
                .grow(1.0);

            for (key, lbl) in [
                ("all", "All"),
                ("Invite", "Invites"),
                ("InviteRequests", "Requests"),
                ("FriendRequests", "Friends"),
                ("Votes", "Votes"),
                ("SystemNotif", "System"),
            ] {
                let active = state.main_menu.messages_filter == key;
                if ui
                    .btn_with(&format!("mm_msg_f_{key}"), |ui| {
                        ui.txt(lbl)
                            .sz(10.5)
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
                        .sz(13.0)
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
                                        .sz(12.5)
                                        .bold()
                                        .fg(TEXT_PRIMARY);
                                    if !notif.category.is_empty() {
                                        ui.txt(&notif.category)
                                            .sz(9.5)
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
                                    ui.txt(&notif.body).sz(11.5).fg(TEXT_MUTED).wrap();
                                }
                            })
                            .gap(3.0)
                            .grow(1.0);

                            ui.row(|ui| {
                                for btn in &notif.buttons {
                                    if ui
                                        .btn_with(&format!("mm_msg_b_{}", btn.id), |ui| {
                                            ui.txt(&btn.text)
                                                .sz(11.0)
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

/// Renders the Phase 3C User Details profile view (accessible from Friends, Current Instance, Home Leaflet, Profile Card, and Search).
pub fn render_user_detail_view(
    ui: &mut UiContext,
    state: &mut AppState,
    detail: &UserDetailSnapshot,
    accent: Color,
    actions: &mut Vec<UiAction>,
) {
    if let Some(gd) = state.host.social.group_detail.clone() {
        render_group_detail_view(ui, state, &gd, accent, actions);
        return;
    }
    ui.col(|ui| {
        // Header bar with Back button, User name, Rank pill, and Overview / Badges / Groups sub-tabs
        ui.row(|ui| {
            if ui
                .btn_with("mm_ud_back", |ui| {
                    ui.row(|ui| {
                        icons::icon(ui, icons::BACK, 14.0).fg(TEXT_PRIMARY);
                        ui.txt("Back").sz(11.5).semibold().fg(TEXT_PRIMARY);
                    })
                    .center()
                    .gap(4.0);
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
                let fallback = state.main_menu.active_tab.clone();
                crate::main_menu::nav_go_back(state, &fallback, actions);
            }

            icons::icon(ui, icons::AVATAR, 16.0).fg(accent);
            ui.txt(if detail.name.is_empty() {
                "Loading profile..."
            } else {
                &detail.name
            })
            .sz(15.0)
            .bold()
            .fg(TEXT_PRIMARY)
            .ellipsis()
            .grow(1.0);
        })
        .align_center()
        .px(12.0)
        .py(8.0)
        .gap(8.0)
        .bg(card_bg())
        .bd(1.0, border_subtle())
        .bevel_rounded(10.0, 2.0)
        .shadow(14.0, SHADOW_CARD);

        if detail.loading {
            ui.col(|ui| {
                icons::icon(ui, icons::AVATAR, 30.0).fg(accent);
                ui.txt("Loading user profile...")
                    .sz(13.0)
                    .semibold()
                    .fg(TEXT_MUTED);
            })
            .center()
            .grow(1.0)
            .gap(8.0)
            .bg(card_bg())
            .bd(1.0, border_subtle())
            .bevel_rounded(10.5, 2.0)
            .shadow(14.0, SHADOW_CARD);
        } else if !detail.error.is_empty() {
            ui.col(|ui| {
                ui.txt(&detail.error).sz(12.5).fg(AMBER).wrap();
            })
            .center()
            .grow(1.0)
            .p(16.0)
            .bg(card_bg())
            .bd(1.0, border_subtle())
            .bevel_rounded(10.5, 2.0)
            .shadow(14.0, SHADOW_CARD);
        } else {
            // Dedicated compact section switcher card (relocated out of top bar)
            ui.row(|ui| {
                let badges_lbl = format!("Badges ({})", detail.badges.len());
                let groups_lbl = format!("Groups ({})", detail.groups.len());
                for (tab_idx, ic, lbl) in [
                    (0usize, icons::AVATAR, "Overview"),
                    (1usize, icons::STAR, badges_lbl.as_str()),
                    (2usize, icons::PLAYERS, groups_lbl.as_str()),
                ] {
                    let active = state.main_menu.user_detail_tab == tab_idx;
                    if ui
                        .btn_with(&format!("mm_ud_subtab_{tab_idx}"), |ui| {
                            ui.row(|ui| {
                                icons::icon(ui, ic, 12.5)
                                    .fg(if active { accent } else { TEXT_DIM });
                                ui.txt(lbl)
                                    .sz(11.0)
                                    .semibold()
                                    .fg(if active { TEXT_PRIMARY } else { TEXT_MUTED });
                            })
                            .center()
                            .gap(5.0)
                            .grow(1.0);
                        })
                        .grow(1.0)
                        .px(10.0)
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
                                tint_alpha(accent, 195)
                            } else {
                                border_subtle()
                            },
                        )
                        .bevel_rounded(6.0, 1.3)
                        .clicked()
                    {
                        push_click(actions);
                        state.main_menu.user_detail_tab = tab_idx;
                    }
                }
            })
            .align_center()
            .p(6.0)
            .gap(6.0)
            .bg(card_bg())
            .bd(1.0, border_subtle())
            .bevel_rounded(9.0, 1.8)
            .shadow(10.0, SHADOW_CARD);

            if state.main_menu.user_detail_tab == 1 {
            // =================================================================
            // BADGES SUB-TAB
            // =================================================================
            ui.col(|ui| {
                if detail.badges.is_empty() {
                    ui.col(|ui| {
                        icons::icon(ui, icons::STAR, 30.0).fg(TEXT_DIM);
                        ui.txt("No badges found for this user.")
                            .sz(13.0)
                            .semibold()
                            .fg(TEXT_MUTED);
                    })
                    .center()
                    .grow(1.0)
                    .gap(6.0);
                } else {
                    ui.scroll("mm_ud_badges_scroll", |ui| {
                        for chunk in detail.badges.chunks(2) {
                            ui.row(|ui| {
                                for b in chunk {
                                    let bd_col = if b.featured {
                                        tint_alpha(accent, 195)
                                    } else {
                                        border_subtle()
                                    };
                                    ui.row(|ui| {
                                        if let Some(tex) = state.main_textures.get(&b.image_key) {
                                            ui.img(tex)
                                                .w(44.0)
                                                .h(44.0)
                                                .fit(ImageFit::Contain)
                                                .bevel_rounded(6.0, 1.3);
                                        } else {
                                            ui.col(|ui| {
                                                icons::icon(ui, icons::STAR, 20.0).fg(accent);
                                            })
                                            .center()
                                            .w(44.0)
                                            .h(44.0)
                                            .bg(tint_alpha(accent, 28))
                                            .bd(1.0, border_subtle())
                                            .bevel_rounded(6.0, 1.3);
                                        }

                                        ui.col(|ui| {
                                            ui.row(|ui| {
                                                ui.txt(&b.name)
                                                    .sz(12.5)
                                                    .bold()
                                                    .fg(TEXT_PRIMARY)
                                                    .ellipsis()
                                                    .grow(1.0);
                                                if b.featured {
                                                    ui.txt("FEATURED")
                                                        .sz(9.0)
                                                        .bold()
                                                        .fg(EMERALD)
                                                        .bg(tint_alpha(EMERALD, 32))
                                                        .px(6.0)
                                                        .py(1.5)
                                                        .bevel_rounded(4.0, 1.1);
                                                }
                                            })
                                            .align_center()
                                            .gap(6.0);

                                            if !b.description.is_empty() {
                                                ui.txt(&b.description)
                                                    .sz(10.5)
                                                    .fg(TEXT_MUTED)
                                                    .wrap();
                                            }
                                        })
                                        .gap(2.5)
                                        .grow(1.0);

                                        if detail.is_local && !b.id.is_empty() {
                                            let btn_lbl = if b.featured {
                                                "Unfeature"
                                            } else {
                                                "Feature"
                                            };
                                            if ui
                                                .btn_with(&format!("mm_ud_feat_b_{}", b.id), |ui| {
                                                    ui.txt(btn_lbl)
                                                        .sz(10.5)
                                                        .semibold()
                                                        .fg(if b.featured {
                                                            TEXT_MUTED
                                                        } else {
                                                            TEXT_PRIMARY
                                                        });
                                                })
                                                .px(10.0)
                                                .py(5.5)
                                                .bg(if b.featured {
                                                    tile_idle_bg()
                                                } else {
                                                    tint_alpha(accent, 48)
                                                })
                                                .hover_bg(tile_idle_hover())
                                                .bd(
                                                    1.0,
                                                    if b.featured {
                                                        border_subtle()
                                                    } else {
                                                        tint_alpha(accent, 180)
                                                    },
                                                )
                                                .bevel_rounded(5.5, 1.3)
                                                .clicked()
                                            {
                                                push_click(actions);
                                                actions.push(UiAction::SocialSetFeaturedBadge {
                                                    badge_id: b.id.clone(),
                                                    featured: !b.featured,
                                                });
                                            }
                                        }
                                    })
                                    .align_center()
                                    .grow(1.0)
                                    .p(10.0)
                                    .gap(10.0)
                                    .bg(card_inner_bg())
                                    .bd(1.0, bd_col)
                                    .bevel_rounded(8.0, 1.3);
                                }
                                if chunk.len() == 1 {
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
        } else if state.main_menu.user_detail_tab == 2 {
            // =================================================================
            // GROUPS SUB-TAB
            // =================================================================
            ui.col(|ui| {
                if detail.groups.is_empty() {
                    ui.col(|ui| {
                        icons::icon(ui, icons::PLAYERS, 30.0).fg(TEXT_DIM);
                        ui.txt("No groups found for this user.")
                            .sz(13.0)
                            .semibold()
                            .fg(TEXT_MUTED);
                    })
                    .center()
                    .grow(1.0)
                    .gap(6.0);
                } else {
                    ui.scroll("mm_ud_groups_scroll", |ui| {
                        for chunk in detail.groups.chunks(2) {
                            ui.row(|ui| {
                                for g in chunk {
                                    let bd_col = if g.is_featured {
                                        tint_alpha(accent, 195)
                                    } else {
                                        border_subtle()
                                    };
                                    ui.row(|ui| {
                                        if let Some(tex) = state.main_textures.get(&g.image_key) {
                                            ui.img(tex)
                                                .w(44.0)
                                                .h(44.0)
                                                .fit(ImageFit::Cover)
                                                .bevel_rounded(6.0, 1.3);
                                        } else {
                                            ui.col(|ui| {
                                                icons::icon(ui, icons::PLAYERS, 20.0).fg(accent);
                                            })
                                            .center()
                                            .w(44.0)
                                            .h(44.0)
                                            .bg(tint_alpha(accent, 28))
                                            .bd(1.0, border_subtle())
                                            .bevel_rounded(6.0, 1.3);
                                        }

                                        ui.col(|ui| {
                                            ui.row(|ui| {
                                                ui.txt(&g.name)
                                                    .sz(12.5)
                                                    .bold()
                                                    .fg(TEXT_PRIMARY)
                                                    .ellipsis()
                                                    .grow(1.0);
                                                if !g.tag.is_empty() {
                                                    ui.txt(&format!("[{}]", g.tag))
                                                        .sz(9.5)
                                                        .fg(accent)
                                                        .bg(tint_alpha(accent, 28))
                                                        .px(5.0)
                                                        .py(1.0)
                                                        .bevel_rounded(4.0, 1.1);
                                                }
                                                if g.is_featured {
                                                    ui.txt("FEATURED")
                                                        .sz(9.0)
                                                        .bold()
                                                        .fg(EMERALD)
                                                        .bg(tint_alpha(EMERALD, 32))
                                                        .px(6.0)
                                                        .py(1.5)
                                                        .bevel_rounded(4.0, 1.1);
                                                }
                                            })
                                            .align_center()
                                            .gap(5.0);

                                            let role = if g.is_owner { "Owner" } else { "Member" };
                                            ui.txt(&format!(
                                                "{role}  |  {} members",
                                                g.member_count.max(1)
                                            ))
                                            .sz(10.5)
                                            .fg(TEXT_MUTED);
                                        })
                                        .gap(2.5)
                                        .grow(1.0);

                                        if !g.id.is_empty() {
                                            ui.row(|ui| {
                                                if detail.is_local && !g.is_featured {
                                                    if ui
                                                        .btn_with(&format!("mm_ud_feat_g_{}", g.id), |ui| {
                                                            ui.txt("Feature")
                                                                .sz(10.5)
                                                                .semibold()
                                                                .fg(TEXT_PRIMARY);
                                                        })
                                                        .px(9.0)
                                                        .py(5.5)
                                                        .bg(tint_alpha(accent, 48))
                                                        .hover_bg(tile_idle_hover())
                                                        .bd(1.0, tint_alpha(accent, 180))
                                                        .bevel_rounded(5.5, 1.3)
                                                        .clicked()
                                                    {
                                                        push_click(actions);
                                                        actions.push(UiAction::SocialSetFeaturedGroup {
                                                            group_id: g.id.clone(),
                                                        });
                                                    }
                                                }

                                                let open_lbl = if detail.is_local && g.is_owner {
                                                    "Manage"
                                                } else {
                                                    "View"
                                                };
                                                if ui
                                                    .btn_with(&format!("mm_ud_open_g_{}", g.id), |ui| {
                                                        ui.txt(open_lbl)
                                                            .sz(10.5)
                                                            .semibold()
                                                            .fg(TEXT_PRIMARY);
                                                    })
                                                    .px(10.0)
                                                    .py(5.5)
                                                    .bg(if detail.is_local && g.is_owner {
                                                        tint_alpha(EMERALD, 45)
                                                    } else {
                                                        tile_idle_bg()
                                                    })
                                                    .hover_bg(tile_idle_hover())
                                                    .bd(
                                                        1.0,
                                                        if detail.is_local && g.is_owner {
                                                            tint_alpha(EMERALD, 180)
                                                        } else {
                                                            border_subtle()
                                                        },
                                                    )
                                                    .bevel_rounded(5.5, 1.3)
                                                    .clicked()
                                                {
                                                    push_click(actions);
                                                    crate::main_menu::push_nav_history(state);
                                                    state.main_menu.safety_confirm_action.clear();
                                                    actions.push(UiAction::SocialLoadGroupDetail {
                                                        group_id: g.id.clone(),
                                                    });
                                                }
                                            })
                                            .align_center()
                                            .gap(5.0);
                                        }
                                    })
                                    .align_center()
                                    .grow(1.0)
                                    .p(10.0)
                                    .gap(10.0)
                                    .bg(card_inner_bg())
                                    .bd(1.0, bd_col)
                                    .bevel_rounded(8.0, 1.3);
                                }
                                if chunk.len() == 1 {
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
        } else {
            ui.row(|ui| {
                // Optional Left Turntable Column when player's avatar is loaded in the current instance
                if detail.has_3d_preview {
                    let player_target = format!("player:{}", detail.id);
                    ui.col(|ui| {
                        ui.row(|ui| {
                            icons::icon(ui, icons::AVATAR, 14.0).fg(accent);
                            ui.txt("3D Avatar")
                                .sz(12.0)
                                .semibold()
                                .fg(TEXT_PRIMARY)
                                .grow(1.0);
                        })
                        .align_center()
                        .gap(6.0);

                        crate::avatar_settings_view::render_avatar_turntable_stage(
                            ui,
                            state,
                            0,
                            &player_target,
                            "mm_ud_turntable_drag",
                            "",
                            accent,
                            actions,
                        );
                    })
                    .w(232.0)
                    .p(10.0)
                    .gap(7.0)
                    .bg(card_bg())
                    .bd(1.0, border_subtle())
                    .bevel_rounded(10.5, 2.0)
                    .shadow(14.0, SHADOW_CARD);
                }

                // Left: Profile Identity + Bio
                ui.col(|ui| {
                    ui.row(|ui| {
                        let ring = if detail.online_state || detail.is_local {
                            EMERALD
                        } else {
                            accent
                        };
                        let pfp_tex = state.main_textures.get(&detail.image_key).or_else(|| {
                            if detail.is_local {
                                state.main_textures.get("local_pfp")
                            } else {
                                None
                            }
                        });
                        if let Some(tex) = pfp_tex {
                            ui.img(tex)
                                .w(78.0)
                                .h(78.0)
                                .fit(ImageFit::Cover)
                                .hexagon_pointy()
                                .r(3.5)
                                .bd(2.0, ring);
                        } else {
                            let init = detail
                                .name
                                .chars()
                                .next()
                                .unwrap_or('?')
                                .to_ascii_uppercase()
                                .to_string();
                            ui.col(|ui| {
                                ui.txt(&init)
                                    .sz(26.0)
                                    .bold()
                                    .align(TextAlign::Center)
                                    .fg(ring);
                            })
                            .center()
                            .w(78.0)
                            .h(78.0)
                            .bg(tint_alpha(ring, 32))
                            .bd(2.0, ring)
                            .hexagon_pointy()
                            .r(3.5);
                        }

                        ui.col(|ui| {
                            ui.row(|ui| {
                                ui.txt(&detail.name)
                                    .sz(18.0)
                                    .bold()
                                    .fg(TEXT_PRIMARY)
                                    .ellipsis()
                                    .grow(1.0);

                                if detail.is_local {
                                    if ui
                                        .btn_with("mm_ud_open_colors", |ui| {
                                            ui.row(|ui| {
                                                icons::icon(ui, icons::STAR, 12.0).fg(accent);
                                                ui.txt("Colors")
                                                    .sz(10.5)
                                                    .semibold()
                                                    .fg(TEXT_PRIMARY);
                                            })
                                            .center()
                                            .gap(4.0);
                                        })
                                        .px(8.0)
                                        .py(4.0)
                                        .bg(tile_idle_bg())
                                        .hover_bg(tile_idle_hover())
                                        .bd(1.0, tint_alpha(accent, 140))
                                        .bevel_rounded(5.5, 1.3)
                                        .clicked()
                                    {
                                        push_click(actions);
                                        crate::main_menu::push_nav_history(state);
                                        state.main_menu.settings_tab = 1;
                                        state.main_menu.active_tab = String::from("settings");
                                        crate::main_menu::close_all_detail_overlays(actions);
                                    }

                                    let edit_lbl = if state.main_menu.profile_editing {
                                        "Done"
                                    } else {
                                        "Edit Profile"
                                    };
                                    if ui
                                        .btn_with("mm_ud_edit_profile", |ui| {
                                            ui.txt(edit_lbl)
                                                .sz(10.5)
                                                .semibold()
                                                .fg(TEXT_PRIMARY);
                                        })
                                        .px(8.0)
                                        .py(4.0)
                                        .bg(if state.main_menu.profile_editing {
                                            tint_alpha(accent, 55)
                                        } else {
                                            tile_idle_bg()
                                        })
                                        .hover_bg(tile_idle_hover())
                                        .bd(
                                            1.0,
                                            if state.main_menu.profile_editing {
                                                accent
                                            } else {
                                                border_subtle()
                                            },
                                        )
                                        .bevel_rounded(5.5, 1.3)
                                        .clicked()
                                    {
                                        push_click(actions);
                                        state.main_menu.profile_editing =
                                            !state.main_menu.profile_editing;
                                        if state.main_menu.profile_editing {
                                            state.main_menu.profile_edit_user_id =
                                                detail.id.clone();
                                            state
                                                .main_menu
                                                .profile_pronouns_input
                                                .set_text(&detail.pronouns);
                                            state
                                                .main_menu
                                                .profile_bio_input
                                                .set_text(&detail.bio);
                                        }
                                    }
                                }
                            })
                            .align_center()
                            .gap(6.0);
                            if !detail.pronouns.is_empty() || crate::theme::is_special_rank(&detail.rank) {
                                ui.row(|ui| {
                                    if !detail.pronouns.is_empty() {
                                        ui.txt(&detail.pronouns).sz(11.0).fg(TEXT_MUTED);
                                    }
                                    if crate::theme::is_special_rank(&detail.rank) {
                                        ui.row(|ui| {
                                            ui.txt(&detail.rank)
                                                .sz(10.0)
                                                .semibold()
                                                .fg(TEXT_PRIMARY);
                                        })
                                        .px(7.0)
                                        .py(2.0)
                                        .bg(tint_alpha(accent, 42))
                                        .bd(1.0, tint_alpha(accent, 150))
                                        .bevel_rounded(4.5, 1.2);
                                    }
                                })
                                .align_center()
                                .gap(6.0);
                            }
                            if !detail.featured_badge.is_empty() {
                                ui.row(|ui| {
                                    if let Some(tex) =
                                        state.main_textures.get(&detail.featured_badge_image_key)
                                    {
                                        ui.img(tex)
                                            .w(18.0)
                                            .h(18.0)
                                            .fit(ImageFit::Contain)
                                            .bevel_rounded(3.5, 1.3);
                                    } else {
                                        icons::icon(ui, icons::STAR, 14.0).fg(accent);
                                    }
                                    ui.txt(&detail.featured_badge)
                                        .sz(11.0)
                                        .fg(accent)
                                        .ellipsis();
                                })
                                .align_center()
                                .gap(5.0);
                            }
                            if !detail.featured_group.is_empty() {
                                ui.row(|ui| {
                                    if let Some(tex) =
                                        state.main_textures.get(&detail.featured_group_image_key)
                                    {
                                        ui.img(tex)
                                            .w(18.0)
                                            .h(18.0)
                                            .fit(ImageFit::Cover)
                                            .bevel_rounded(3.5, 1.3);
                                    } else {
                                        icons::icon(ui, icons::PLAYERS, 14.0).fg(TEXT_MUTED);
                                    }
                                    ui.txt(&detail.featured_group)
                                        .sz(11.0)
                                        .fg(TEXT_MUTED)
                                        .ellipsis();
                                })
                                .align_center()
                                .gap(5.0);
                            }
                        })
                        .gap(2.5)
                        .grow(1.0);
                    })
                    .align_center()
                    .gap(12.0);

                    if detail.is_local && state.main_menu.profile_editing {
                        if state.main_menu.profile_edit_user_id != detail.id {
                            state.main_menu.profile_edit_user_id = detail.id.clone();
                            state
                                .main_menu
                                .profile_pronouns_input
                                .set_text(&detail.pronouns);
                            state.main_menu.profile_bio_input.set_text(&detail.bio);
                        }

                        ui.row(|ui| {
                            ui.txt("Pronouns (max 32)")
                                .sz(11.0)
                                .semibold()
                                .fg(TEXT_MUTED)
                                .grow(1.0);
                            if !detail.profile_status.is_empty() {
                                ui.txt(&detail.profile_status)
                                    .sz(10.0)
                                    .semibold()
                                    .fg(if detail.profile_status == "Saved" {
                                        EMERALD
                                    } else {
                                        AMBER
                                    });
                            }
                        })
                        .align_center();

                        let mut pron_inp = ui
                            .input("mm_prof_pronouns", &mut state.main_menu.profile_pronouns_input)
                            .placeholder("e.g. they/them, he/him, she/her...")
                            .bg(card_inner_bg())
                            .fg(TEXT_PRIMARY)
                            .w_frac(1.0);
                        if state.host.is_vr && pron_inp.clicked() {
                            actions.push(UiAction::OpenVirtualKeyboard {
                                surface_id: 0,
                                field_id: String::from("mm_prof_pronouns"),
                                initial_text: state.main_menu.profile_pronouns_input.text(),
                                title: String::from("Editing Your Pronouns"),
                                placeholder: String::from("Pronouns (max 32)..."),
                                max_len: 32,
                                hidden: false,
                            });
                        }

                        ui.txt("Bio (max 512)")
                            .sz(11.0)
                            .semibold()
                            .fg(TEXT_MUTED);

                        let mut bio_inp = ui
                            .input("mm_prof_bio", &mut state.main_menu.profile_bio_input)
                            .placeholder("Write something about yourself...")
                            .bg(card_inner_bg())
                            .fg(TEXT_PRIMARY)
                            .w_frac(1.0);
                        if state.host.is_vr && bio_inp.clicked() {
                            actions.push(UiAction::OpenVirtualKeyboard {
                                surface_id: 0,
                                field_id: String::from("mm_prof_bio"),
                                initial_text: state.main_menu.profile_bio_input.text(),
                                title: String::from("Editing Your Bio"),
                                placeholder: String::from("Bio (max 512)..."),
                                max_len: 512,
                                hidden: false,
                            });
                        }

                        ui.row(|ui| {
                            let save_lbl = if detail.profile_saving {
                                "Saving..."
                            } else {
                                "Save Profile"
                            };
                            if ui
                                .btn_with("mm_prof_save", |ui| {
                                    ui.txt(save_lbl)
                                        .sz(11.0)
                                        .bold()
                                        .align(TextAlign::Center)
                                        .fg(TEXT_PRIMARY)
                                        .grow(1.0);
                                })
                                .grow(1.0)
                                .py(6.5)
                                .bg(tint_alpha(accent, 55))
                                .hover_bg(tint_alpha(accent, 88))
                                .bd(1.0, accent)
                                .bevel_rounded(6.0, 1.3)
                                .clicked()
                                && !detail.profile_saving
                            {
                                push_click(actions);
                                actions.push(UiAction::SocialUpdateOwnProfile {
                                    pronouns: state.main_menu.profile_pronouns_input.text(),
                                    bio: state.main_menu.profile_bio_input.text(),
                                    update_pronouns: true,
                                    update_bio: true,
                                });
                            }

                            if ui
                                .btn_with("mm_prof_cancel", |ui| {
                                    ui.txt("Cancel")
                                        .sz(11.0)
                                        .semibold()
                                        .align(TextAlign::Center)
                                        .fg(TEXT_MUTED)
                                        .grow(1.0);
                                })
                                .px(12.0)
                                .py(6.5)
                                .bg(tile_idle_bg())
                                .hover_bg(tile_idle_hover())
                                .bd(1.0, border_subtle())
                                .bevel_rounded(6.0, 1.3)
                                .clicked()
                            {
                                push_click(actions);
                                state.main_menu.profile_editing = false;
                            }
                        })
                        .gap(6.0);
                    } else {
                        ui.txt("Bio").sz(11.0).semibold().fg(TEXT_MUTED);
                        ui.scroll("mm_ud_bio_scroll", |ui| {
                            let bio = if detail.bio.trim().is_empty() {
                                "No bio set."
                            } else {
                                &detail.bio
                            };
                            ui.txt(bio).sz(12.0).fg(TEXT_PRIMARY).wrap();
                        })
                        .grow(1.0)
                        .p(9.0)
                        .bg(card_inner_bg())
                        .bd(1.0, border_subtle())
                        .bevel_rounded(7.0, 1.3)
                        .no_scrollbar();
                    }

                    // Friend Note (below Bio)
                    if !detail.is_local {
                        if state.main_menu.user_note_user_id != detail.id {
                            state.main_menu.user_note_user_id = detail.id.clone();
                            state.main_menu.user_note_input.set_text(&detail.note);
                        }

                        ui.row(|ui| {
                            ui.txt("Notes").sz(11.0).semibold().fg(TEXT_MUTED).grow(1.0);
                            if !detail.note_status.is_empty() {
                                ui.txt(&detail.note_status)
                                    .sz(10.0)
                                    .semibold()
                                    .fg(if detail.note_status == "Saved" {
                                        EMERALD
                                    } else {
                                        AMBER
                                    });
                            }
                        })
                        .align_center()
                        .gap(6.0);

                        if detail.is_friend {
                            ui.row(|ui| {
                                let mut n_inp = ui
                                    .input("mm_ud_note_input", &mut state.main_menu.user_note_input)
                                    .placeholder("Add a private note about this friend...")
                                    .bg(card_inner_bg())
                                    .fg(TEXT_PRIMARY)
                                    .grow(1.0);
                                if state.host.is_vr && n_inp.clicked() {
                                    actions.push(UiAction::OpenVirtualKeyboard {
                                        surface_id: 0,
                                        field_id: String::from("mm_ud_note_input"),
                                        initial_text: state.main_menu.user_note_input.text(),
                                        title: format!("Note for {}", detail.name),
                                        placeholder: String::from("Add a private note..."),
                                        max_len: 512,
                                        hidden: false,
                                    });
                                }

                                if ui
                                    .btn_with("mm_ud_note_save", |ui| {
                                        ui.txt(if detail.note_saving { "Saving..." } else { "Save" })
                                            .sz(11.0)
                                            .semibold()
                                            .fg(TEXT_PRIMARY);
                                    })
                                    .px(11.0)
                                    .py(6.5)
                                    .bg(tint_alpha(accent, 45))
                                    .hover_bg(tint_alpha(accent, 75))
                                    .bd(1.0, tint_alpha(accent, 175))
                                    .bevel_rounded(6.0, 1.3)
                                    .clicked()
                                    && !detail.note_saving
                                {
                                    push_click(actions);
                                    actions.push(UiAction::SocialSaveUserNote {
                                        user_id: detail.id.clone(),
                                        note: state.main_menu.user_note_input.text(),
                                    });
                                }
                            })
                            .align_center()
                            .gap(6.0);
                        } else {
                            ui.row(|ui| {
                                let note_txt = if detail.note.trim().is_empty() {
                                    "Add as friend to save private notes."
                                } else {
                                    &detail.note
                                };
                                ui.txt(note_txt)
                                    .sz(11.0)
                                    .fg(TEXT_DIM)
                                    .wrap()
                                    .grow(1.0);
                            })
                            .p(8.0)
                            .bg(card_inner_bg())
                            .bd(1.0, border_subtle())
                            .bevel_rounded(7.0, 1.3);
                        }
                    }
                })
                .grow(1.2)
                .h_frac(1.0)
                .p(12.0)
                .gap(9.0)
                .bg(card_bg())
                .bd(1.0, border_subtle())
                .bevel_rounded(10.5, 2.0)
                .shadow(14.0, SHADOW_CARD);

                // Right: Online Status, Instance & Social Actions
                ui.col(|ui| {
                    // Status & Instance card
                    ui.col(|ui| {
                        let status_lbl = if detail.online_state || detail.is_local {
                            if !detail.instance_name.is_empty() {
                                format!("Online  |  {}", detail.instance_name)
                            } else if detail.is_connected {
                                String::from("Online  |  Private Instance")
                            } else {
                                String::from("Online")
                            }
                        } else {
                            String::from("Offline")
                        };
                        ui.txt(&status_lbl).sz(12.5).bold().fg(
                            if detail.online_state || detail.is_local {
                                EMERALD
                            } else {
                                TEXT_DIM
                            },
                        );

                        if !detail.world_name.is_empty() {
                            ui.txt(&format!("World: {}", detail.world_name))
                                .sz(11.5)
                                .fg(TEXT_MUTED);
                        }
                        if !detail.avatar_name.is_empty() {
                            if !detail.avatar_id.is_empty() {
                                if ui
                                    .btn_with("mm_ud_open_worn_avatar", |ui| {
                                        ui.row(|ui| {
                                            if let Some(tex) =
                                                state.main_textures.get(&detail.avatar_image_key)
                                            {
                                                ui.img(tex)
                                                    .w(34.0)
                                                    .h(34.0)
                                                    .fit(ImageFit::Cover)
                                                    .bevel_rounded(5.0, 1.2);
                                            } else {
                                                ui.col(|ui| {
                                                    icons::icon(ui, icons::AVATAR, 16.0).fg(accent);
                                                })
                                                .center()
                                                .w(34.0)
                                                .h(34.0)
                                                .bg(tint_alpha(accent, 28))
                                                .bd(1.0, tint_alpha(accent, 110))
                                                .bevel_rounded(5.0, 1.2);
                                            }

                                            ui.col(|ui| {
                                                ui.txt("WORN AVATAR")
                                                    .sz(8.5)
                                                    .bold()
                                                    .fg(TEXT_DIM);
                                                ui.txt(&detail.avatar_name)
                                                    .sz(11.5)
                                                    .semibold()
                                                    .fg(TEXT_PRIMARY)
                                                    .ellipsis();
                                            })
                                            .gap(1.5)
                                            .grow(1.0);

                                            icons::icon(ui, icons::CHEVRON_RIGHT, 15.0)
                                                .fg(TEXT_MUTED);
                                        })
                                        .align_center()
                                        .gap(8.0)
                                        .grow(1.0);
                                    })
                                    .w_frac(1.0)
                                    .p(6.5)
                                    .bg(tile_idle_bg())
                                    .hover_bg(tile_idle_hover())
                                    .bd(1.0, tint_alpha(accent, 120))
                                    .bevel_rounded(6.5, 1.3)
                                    .clicked()
                                {
                                    push_click(actions);
                                    crate::main_menu::push_nav_history(state);
                                    state.main_menu.active_tab = String::from("avatars");
                                    actions.push(UiAction::SocialCloseUserDetail);
                                    actions.push(UiAction::SocialCloseGroupDetail);
                                    actions.push(UiAction::ContentLoadDetail {
                                        kind: String::from("avatar"),
                                        id: detail.avatar_id.clone(),
                                    });
                                }
                            } else {
                                ui.txt(&format!("Avatar: {}", detail.avatar_name))
                                    .sz(11.5)
                                    .fg(TEXT_MUTED);
                            }
                        }

                        if !detail.is_local
                            && !detail.instance_id.is_empty()
                            && ui
                                .btn_with("mm_ud_join_inst", |ui| {
                                    ui.txt("Join Instance")
                                        .sz(12.0)
                                        .bold()
                                        .align(TextAlign::Center)
                                        .fg(TEXT_PRIMARY)
                                        .grow(1.0);
                                })
                                .py(8.0)
                                .bg(tint_alpha(EMERALD, 50))
                                .hover_bg(tint_alpha(EMERALD, 80))
                                .bd(1.5, tint_alpha(EMERALD, 195))
                                .bevel_rounded(6.5, 1.3)
                                .clicked()
                        {
                            push_click(actions);
                            actions.push(UiAction::ContentJoinInstance {
                                instance_id: detail.instance_id.clone(),
                            });
                        }
                    })
                    .p(11.0)
                    .gap(6.0)
                    .bg(card_bg())
                    .bd(1.0, border_subtle())
                    .bevel_rounded(10.5, 2.0)
                    .shadow(14.0, SHADOW_CARD);

                    // Social Actions or Quick Profile shortcuts
                    if detail.is_local {
                        ui.col(|ui| {
                            ui.txt("Your Profile")
                                .sz(12.5)
                                .bold()
                                .fg(TEXT_PRIMARY);
                            ui.txt("Use the Badges and Groups tabs above to inspect or feature your ChilloutVR badges and groups.")
                                .sz(11.0)
                                .fg(TEXT_MUTED)
                                .wrap();

                            ui.row(|ui| {
                                if ui
                                    .btn_with("mm_ud_go_badges", |ui| {
                                        ui.row(|ui| {
                                            icons::icon(ui, icons::STAR, 13.0).fg(accent);
                                            ui.txt(&format!(
                                                "Manage Badges ({})",
                                                detail.badges.len()
                                            ))
                                            .sz(11.0)
                                            .semibold()
                                            .fg(TEXT_PRIMARY);
                                        })
                                        .center()
                                        .gap(5.0)
                                        .grow(1.0);
                                    })
                                    .grow(1.0)
                                    .py(8.0)
                                    .bg(tile_idle_bg())
                                    .hover_bg(tile_idle_hover())
                                    .bd(1.0, tint_alpha(accent, 150))
                                    .bevel_rounded(6.5, 1.3)
                                    .clicked()
                                {
                                    push_click(actions);
                                    state.main_menu.user_detail_tab = 1;
                                }

                                if ui
                                    .btn_with("mm_ud_go_groups", |ui| {
                                        ui.row(|ui| {
                                            icons::icon(ui, icons::PLAYERS, 13.0).fg(accent);
                                            ui.txt(&format!(
                                                "Manage Groups ({})",
                                                detail.groups.len()
                                            ))
                                            .sz(11.0)
                                            .semibold()
                                            .fg(TEXT_PRIMARY);
                                        })
                                        .center()
                                        .gap(5.0)
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
                                    state.main_menu.user_detail_tab = 2;
                                }
                            })
                            .gap(7.0);
                        })
                        .grow(1.0)
                        .p(11.0)
                        .gap(8.0)
                        .bg(card_bg())
                        .bd(1.0, border_subtle())
                        .bevel_rounded(10.5, 2.0)
                        .shadow(14.0, SHADOW_CARD);
                    } else {
                        // Top Card: Actions (Invite, Request Invite, Voice Volume, Reload Avatar)
                        ui.col(|ui| {
                            ui.txt("Actions").sz(12.5).bold().fg(TEXT_PRIMARY);

                            ui.row(|ui| {
                                if ui
                                    .btn_with("mm_ud_invite", |ui| {
                                        ui.txt("Invite")
                                            .sz(11.5)
                                            .semibold()
                                            .align(TextAlign::Center)
                                            .fg(TEXT_PRIMARY)
                                            .grow(1.0);
                                    })
                                    .grow(1.0)
                                    .py(7.5)
                                    .bg(tile_idle_bg())
                                    .hover_bg(tile_idle_hover())
                                    .bd(1.0, tint_alpha(accent, 150))
                                    .bevel_rounded(6.5, 1.3)
                                    .clicked()
                                {
                                    push_click(actions);
                                    actions.push(UiAction::SocialInvitePlayer {
                                        user_id: detail.id.clone(),
                                    });
                                }

                                if ui
                                    .btn_with("mm_ud_req_inv", |ui| {
                                        ui.txt("Request Invite")
                                            .sz(11.5)
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
                                    actions.push(UiAction::SocialRequestInvite {
                                        user_id: detail.id.clone(),
                                    });
                                }

                                if !detail.is_friend
                                    && ui
                                        .btn_with("mm_ud_addfriend", |ui| {
                                            ui.txt("Add Friend")
                                                .sz(11.0)
                                                .semibold()
                                                .align(TextAlign::Center)
                                                .fg(TEXT_PRIMARY)
                                                .grow(1.0);
                                        })
                                        .grow(1.0)
                                        .py(7.5)
                                        .bg(tint_alpha(accent, 48))
                                        .hover_bg(tint_alpha(accent, 80))
                                        .bd(1.0, tint_alpha(accent, 180))
                                        .bevel_rounded(6.5, 1.3)
                                        .clicked()
                                {
                                    push_click(actions);
                                    state.main_menu.safety_confirm_action.clear();
                                    actions.push(UiAction::SocialSendFriendRequest {
                                        user_id: detail.id.clone(),
                                    });
                                }
                            })
                            .gap(7.0);

                            // Mute + Voice Volume slider
                            ui.row(|ui| {
                                if ui
                                    .btn_with("mm_ud_mute", |ui| {
                                        ui.row(|ui| {
                                            icons::icon(
                                                ui,
                                                if detail.is_muted {
                                                    icons::MIC_OFF
                                                } else {
                                                    icons::VOLUME
                                                },
                                                12.0,
                                            )
                                            .fg(if detail.is_muted { RED } else { EMERALD });
                                            ui.txt(if detail.is_muted { "Muted" } else { "Mute" })
                                                .sz(10.5)
                                                .semibold()
                                                .fg(TEXT_PRIMARY);
                                        })
                                        .center()
                                        .gap(4.0);
                                    })
                                    .px(9.0)
                                    .py(5.5)
                                    .bg(if detail.is_muted {
                                        tint_alpha(RED, 52)
                                    } else {
                                        tile_idle_bg()
                                    })
                                    .hover_bg(tile_idle_hover())
                                    .bd(
                                        1.0,
                                        if detail.is_muted {
                                            tint_alpha(RED, 180)
                                        } else {
                                            border_subtle()
                                        },
                                    )
                                    .bevel_rounded(5.5, 1.3)
                                    .clicked()
                                {
                                    push_click(actions);
                                    actions.push(UiAction::SocialSetPlayerMute {
                                        user_id: detail.id.clone(),
                                        muted: !detail.is_muted,
                                    });
                                }

                                let vol_key = format!("_ud_vol_{}", detail.id);
                                let host_vol = detail.voice_volume.clamp(0, 200);
                                let (vol_f, last_vol) = state
                                    .main_menu
                                    .settings_sliders
                                    .entry(vol_key.clone())
                                    .or_insert((host_vol as f32, host_vol));
                                if *last_vol != host_vol {
                                    *vol_f = host_vol as f32;
                                    *last_vol = host_vol;
                                }
                                let mut slider_v = *vol_f;
                                ui.push_id(&vol_key);
                                if ui
                                    .slider(
                                        &format!("Voice {}%", slider_v.round() as i32),
                                        &mut slider_v,
                                        0.0..=200.0,
                                    )
                                    .fill(accent)
                                    .bg(card_inner_bg())
                                    .bd(1.0, border_subtle())
                                    .bevel_rounded(5.5, 1.3)
                                    .grow(1.0)
                                    .changed()
                                {
                                    let next = (slider_v.round() as i32).clamp(0, 200);
                                    *vol_f = next as f32;
                                    if next != *last_vol {
                                        *last_vol = next;
                                        actions.push(UiAction::SocialSetPlayerVolume {
                                            user_id: detail.id.clone(),
                                            volume: next,
                                        });
                                    }
                                }
                                ui.pop_id();
                            })
                            .align_center()
                            .gap(6.0);

                            // Reload Avatar
                            if ui
                                .btn_with("mm_ud_reload_av", |ui| {
                                    ui.txt("Reload Avatar")
                                        .sz(10.5)
                                        .semibold()
                                        .align(TextAlign::Center)
                                        .fg(TEXT_PRIMARY)
                                        .grow(1.0);
                                })
                                .w_frac(1.0)
                                .py(6.5)
                                .bg(tile_idle_bg())
                                .hover_bg(tile_idle_hover())
                                .bd(1.0, border_subtle())
                                .bevel_rounded(5.5, 1.3)
                                .clicked()
                            {
                                push_click(actions);
                                actions.push(UiAction::SocialReloadPlayerAvatar {
                                    user_id: detail.id.clone(),
                                });
                            }
                        })
                        .p(11.0)
                        .gap(7.0)
                        .bg(card_bg())
                        .bd(1.0, border_subtle())
                        .bevel_rounded(10.5, 2.0)
                        .shadow(14.0, SHADOW_CARD);

                        // Bottom Card: Moderation & Safety (Avatar/Prop Visibility, Unfriend, Block, Kick)
                        ui.col(|ui| {
                            ui.txt("Moderation & Safety")
                                .sz(12.5)
                                .bold()
                                .fg(TEXT_PRIMARY);

                            // Avatar Visibility (Hide / Filter / Show)
                            ui.row(|ui| {
                                ui.txt("Avatar").sz(10.0).fg(TEXT_MUTED).w(44.0);
                                for (mode, lbl) in [(0i32, "Hide"), (1i32, "Filter"), (2i32, "Show")]
                                {
                                    let active = detail.avatar_visibility == mode;
                                    if ui
                                        .btn_with(&format!("mm_ud_avv_{mode}"), |ui| {
                                            ui.txt(lbl)
                                                .sz(10.0)
                                                .semibold()
                                                .align(TextAlign::Center)
                                                .fg(if active { TEXT_PRIMARY } else { TEXT_MUTED })
                                                .grow(1.0);
                                        })
                                        .grow(1.0)
                                        .py(4.5)
                                        .bg(if active {
                                            tint_alpha(accent, 50)
                                        } else {
                                            tile_idle_bg()
                                        })
                                        .hover_bg(tile_idle_hover())
                                        .bd(
                                            1.0,
                                            if active {
                                                tint_alpha(accent, 185)
                                            } else {
                                                border_subtle()
                                            },
                                        )
                                        .bevel_rounded(5.0, 1.3)
                                        .clicked()
                                    {
                                        push_click(actions);
                                        actions.push(UiAction::SocialSetAvatarVisibility {
                                            user_id: detail.id.clone(),
                                            mode,
                                        });
                                    }
                                }
                            })
                            .align_center()
                            .gap(5.0);

                            // Prop Visibility (Hide / Filter / Show)
                            ui.row(|ui| {
                                ui.txt("Props").sz(10.0).fg(TEXT_MUTED).w(44.0);
                                for (mode, lbl) in [(0i32, "Hide"), (1i32, "Filter"), (2i32, "Show")]
                                {
                                    let active = detail.prop_visibility == mode;
                                    if ui
                                        .btn_with(&format!("mm_ud_prv_{mode}"), |ui| {
                                            ui.txt(lbl)
                                                .sz(10.0)
                                                .semibold()
                                                .align(TextAlign::Center)
                                                .fg(if active { TEXT_PRIMARY } else { TEXT_MUTED })
                                                .grow(1.0);
                                        })
                                        .grow(1.0)
                                        .py(4.5)
                                        .bg(if active {
                                            tint_alpha(accent, 50)
                                        } else {
                                            tile_idle_bg()
                                        })
                                        .hover_bg(tile_idle_hover())
                                        .bd(
                                            1.0,
                                            if active {
                                                tint_alpha(accent, 185)
                                            } else {
                                                border_subtle()
                                            },
                                        )
                                        .bevel_rounded(5.0, 1.3)
                                        .clicked()
                                    {
                                        push_click(actions);
                                        actions.push(UiAction::SocialSetPropVisibility {
                                            user_id: detail.id.clone(),
                                            mode,
                                        });
                                    }
                                }
                            })
                            .align_center()
                            .gap(5.0);

                            // Unfriend + Block + Kick
                            ui.row(|ui| {
                                if detail.is_friend {
                                    let unfriend_armed =
                                        state.main_menu.safety_confirm_action == "unfriend";
                                    if ui
                                        .btn_with("mm_ud_unfriend", |ui| {
                                            ui.txt(if unfriend_armed {
                                                "Confirm Unfriend?"
                                            } else {
                                                "Unfriend"
                                            })
                                            .sz(10.5)
                                            .semibold()
                                            .align(TextAlign::Center)
                                            .fg(if unfriend_armed { TEXT_PRIMARY } else { AMBER })
                                            .grow(1.0);
                                        })
                                        .grow(1.0)
                                        .py(6.5)
                                        .bg(if unfriend_armed {
                                            tint_alpha(AMBER, 75)
                                        } else {
                                            tile_idle_bg()
                                        })
                                        .hover_bg(tint_alpha(AMBER, 95))
                                        .bd(1.0, tint_alpha(AMBER, 165))
                                        .bevel_rounded(6.0, 1.3)
                                        .clicked()
                                    {
                                        push_click(actions);
                                        if unfriend_armed {
                                            state.main_menu.safety_confirm_action.clear();
                                            actions.push(UiAction::SocialUnfriend {
                                                user_id: detail.id.clone(),
                                            });
                                        } else {
                                            state.main_menu.safety_confirm_action =
                                                String::from("unfriend");
                                        }
                                    }
                                }

                                let block_armed = !detail.is_blocked
                                    && state.main_menu.safety_confirm_action == "block";
                                let blk_lbl = if detail.is_blocked {
                                    "Unblock"
                                } else if block_armed {
                                    "Confirm Block?"
                                } else {
                                    "Block"
                                };
                                if ui
                                    .btn_with("mm_ud_block", |ui| {
                                        ui.txt(blk_lbl)
                                            .sz(10.5)
                                            .semibold()
                                            .align(TextAlign::Center)
                                            .fg(if detail.is_blocked || block_armed {
                                                TEXT_PRIMARY
                                            } else {
                                                RED
                                            })
                                            .grow(1.0);
                                    })
                                    .grow(1.0)
                                    .py(6.5)
                                    .bg(if detail.is_blocked || block_armed {
                                        tint_alpha(RED, 72)
                                    } else {
                                        tile_idle_bg()
                                    })
                                    .hover_bg(tint_alpha(RED, 95))
                                    .bd(1.0, tint_alpha(RED, 165))
                                    .bevel_rounded(6.0, 1.3)
                                    .clicked()
                                {
                                    push_click(actions);
                                    if detail.is_blocked || block_armed {
                                        state.main_menu.safety_confirm_action.clear();
                                        actions.push(UiAction::SocialSetBlocked {
                                            user_id: detail.id.clone(),
                                            blocked: !detail.is_blocked,
                                        });
                                    } else {
                                        state.main_menu.safety_confirm_action =
                                            String::from("block");
                                    }
                                }

                                let kick_armed = state.main_menu.safety_confirm_action == "kick";
                                if ui
                                    .btn_with("mm_ud_kick", |ui| {
                                        ui.txt(if kick_armed {
                                            "Confirm Kick?"
                                        } else {
                                            "Kick"
                                        })
                                        .sz(10.5)
                                        .semibold()
                                        .align(TextAlign::Center)
                                        .fg(if kick_armed { TEXT_PRIMARY } else { RED })
                                        .grow(1.0);
                                    })
                                    .grow(1.0)
                                    .py(6.5)
                                    .bg(if kick_armed {
                                        tint_alpha(RED, 75)
                                    } else {
                                        tile_idle_bg()
                                    })
                                    .hover_bg(tint_alpha(RED, 95))
                                    .bd(1.0, tint_alpha(RED, 155))
                                    .bevel_rounded(6.0, 1.3)
                                    .clicked()
                                {
                                    push_click(actions);
                                    if kick_armed {
                                        state.main_menu.safety_confirm_action.clear();
                                        actions.push(UiAction::SocialKickUser {
                                            user_id: detail.id.clone(),
                                        });
                                    } else {
                                        state.main_menu.safety_confirm_action =
                                            String::from("kick");
                                    }
                                }
                            })
                            .gap(6.0);
                        })
                        .grow(1.0)
                        .p(11.0)
                        .gap(7.0)
                        .bg(card_bg())
                        .bd(1.0, border_subtle())
                        .bevel_rounded(10.5, 2.0)
                        .shadow(14.0, SHADOW_CARD);
                    }
                })
                .grow(1.0)
                .h_frac(1.0)
                .gap(9.0);
            })
            .grow(1.0)
            .gap(10.0);
            }
        }
    })
    .grow(1.0)
    .gap(9.0);
}

/// Renders the Group Detail & Manage Group view (view members, edit owned group name/description, feature, kick, leave/delete).
pub fn render_group_detail_view(
    ui: &mut UiContext,
    state: &mut AppState,
    gd: &GroupDetailSnapshot,
    accent: Color,
    actions: &mut Vec<UiAction>,
) {
    if !gd.loading && !gd.id.is_empty() && state.main_menu.group_edit_id != gd.id {
        state.main_menu.group_edit_id = gd.id.clone();
        state.main_menu.group_name_input.set_text(&gd.name);
        state.main_menu.group_desc_input.set_text(&gd.description);
    }

    ui.col(|ui| {
        // Header bar
        ui.row(|ui| {
            if ui
                .btn_with("mm_gd_back", |ui| {
                    ui.row(|ui| {
                        icons::icon(ui, icons::BACK, 14.0).fg(TEXT_PRIMARY);
                        ui.txt("Back").sz(11.5).semibold().fg(TEXT_PRIMARY);
                    })
                    .center()
                    .gap(4.0);
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
                let fallback = state.main_menu.active_tab.clone();
                crate::main_menu::nav_go_back(state, &fallback, actions);
            }

            if let Some(tex) = state.main_textures.get(&gd.image_key) {
                ui.img(tex)
                    .w(32.0)
                    .h(32.0)
                    .fit(ImageFit::Cover)
                    .bevel_rounded(6.0, 1.3)
                    .bd(1.0, tint_alpha(accent, 160));
            } else {
                icons::icon(ui, icons::PLAYERS, 17.0).fg(accent);
            }

            ui.col(|ui| {
                let title = if gd.name.is_empty() {
                    "Loading group..."
                } else {
                    &gd.name
                };
                ui.txt(title).sz(14.5).bold().fg(TEXT_PRIMARY).ellipsis();

                let sub = format!(
                    "{} members{}",
                    gd.member_count.max(gd.members.len() as i32),
                    if gd.owner_name.is_empty() {
                        String::new()
                    } else {
                        format!("  |  Owner: {}", gd.owner_name)
                    }
                );
                ui.txt(&sub).sz(10.5).fg(TEXT_MUTED).ellipsis();
            })
            .gap(1.0)
            .grow(1.0);
        })
        .align_center()
        .px(12.0)
        .py(8.0)
        .gap(8.0)
        .bg(card_bg())
        .bd(1.0, border_subtle())
        .bevel_rounded(10.0, 2.0)
        .shadow(14.0, SHADOW_CARD);

        if gd.loading {
            ui.col(|ui| {
                icons::icon(ui, icons::PLAYERS, 30.0).fg(accent);
                ui.txt("Loading group details...")
                    .sz(13.0)
                    .semibold()
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
                // Left: Group Details / Actions / Owner Edit Form
                ui.col(|ui| {
                    if !gd.tag.is_empty() || gd.is_owner || gd.is_member {
                        ui.row(|ui| {
                            if !gd.tag.is_empty() {
                                ui.row(|ui| {
                                    ui.txt(&format!("[{}]", gd.tag))
                                        .sz(10.0)
                                        .semibold()
                                        .fg(TEXT_PRIMARY);
                                })
                                .px(7.0)
                                .py(3.0)
                                .bg(tint_alpha(accent, 38))
                                .bd(1.0, tint_alpha(accent, 145))
                                .bevel_rounded(4.5, 1.1);
                            }
                            if gd.is_owner {
                                ui.row(|ui| {
                                    ui.txt("Owner")
                                        .sz(10.0)
                                        .semibold()
                                        .fg(EMERALD);
                                })
                                .px(7.0)
                                .py(3.0)
                                .bg(tint_alpha(EMERALD, 32))
                                .bd(1.0, tint_alpha(EMERALD, 140))
                                .bevel_rounded(4.5, 1.1);
                            }
                            ui.col(|_| {}).grow(1.0);

                            if gd.is_member && !gd.is_featured && !gd.id.is_empty() {
                                if ui
                                    .btn_with("mm_gd_feature", |ui| {
                                        ui.txt("Feature").sz(10.5).semibold().fg(TEXT_PRIMARY);
                                    })
                                    .px(10.0)
                                    .py(5.0)
                                    .bg(tint_alpha(accent, 48))
                                    .hover_bg(tint_alpha(accent, 80))
                                    .bd(1.0, tint_alpha(accent, 185))
                                    .bevel_rounded(5.5, 1.3)
                                    .clicked()
                                {
                                    push_click(actions);
                                    actions.push(UiAction::SocialSetFeaturedGroup {
                                        group_id: gd.id.clone(),
                                    });
                                }
                            }

                            if gd.is_member && !gd.is_owner && !gd.id.is_empty() {
                                let leave_armed =
                                    state.main_menu.safety_confirm_action == "leave_group";
                                if ui
                                    .btn_with("mm_gd_leave", |ui| {
                                        ui.txt(if leave_armed {
                                            "Confirm Leave?"
                                        } else {
                                            "Leave Group"
                                        })
                                        .sz(10.5)
                                        .semibold()
                                        .fg(if leave_armed { TEXT_PRIMARY } else { AMBER });
                                    })
                                    .px(10.0)
                                    .py(5.0)
                                    .bg(if leave_armed {
                                        tint_alpha(AMBER, 72)
                                    } else {
                                        tile_idle_bg()
                                    })
                                    .hover_bg(tint_alpha(AMBER, 95))
                                    .bd(1.0, tint_alpha(AMBER, 175))
                                    .bevel_rounded(5.5, 1.3)
                                    .clicked()
                                {
                                    push_click(actions);
                                    if leave_armed {
                                        state.main_menu.safety_confirm_action.clear();
                                        state.main_menu.group_edit_id.clear();
                                        actions.push(UiAction::SocialLeaveGroup {
                                            group_id: gd.id.clone(),
                                        });
                                    } else {
                                        state.main_menu.safety_confirm_action =
                                            String::from("leave_group");
                                    }
                                }
                            }

                            if gd.is_owner && !gd.id.is_empty() {
                                let del_armed =
                                    state.main_menu.safety_confirm_action == "delete_group";
                                if ui
                                    .btn_with("mm_gd_delete", |ui| {
                                        ui.txt(if del_armed {
                                            "Confirm Delete?"
                                        } else {
                                            "Delete Group"
                                        })
                                        .sz(10.5)
                                        .semibold()
                                        .fg(if del_armed { TEXT_PRIMARY } else { RED });
                                    })
                                    .px(10.0)
                                    .py(5.0)
                                    .bg(if del_armed {
                                        tint_alpha(RED, 75)
                                    } else {
                                        tile_idle_bg()
                                    })
                                    .hover_bg(tint_alpha(RED, 95))
                                    .bd(1.0, tint_alpha(RED, 175))
                                    .bevel_rounded(5.5, 1.3)
                                    .clicked()
                                {
                                    push_click(actions);
                                    if del_armed {
                                        state.main_menu.safety_confirm_action.clear();
                                        state.main_menu.group_edit_id.clear();
                                        actions.push(UiAction::SocialDeleteGroup {
                                            group_id: gd.id.clone(),
                                        });
                                    } else {
                                        state.main_menu.safety_confirm_action =
                                            String::from("delete_group");
                                    }
                                }
                            }
                        })
                        .align_center()
                        .gap(6.0);
                    }
                    if !gd.status_msg.is_empty() {
                        ui.txt(&gd.status_msg)
                            .sz(11.0)
                            .semibold()
                            .fg(EMERALD)
                            .bg(tint_alpha(EMERALD, 28))
                            .px(9.0)
                            .py(5.0)
                            .bevel_rounded(5.5, 1.3);
                    }
                    if !gd.error.is_empty() {
                        ui.txt(&gd.error)
                            .sz(11.0)
                            .semibold()
                            .fg(AMBER)
                            .bg(tint_alpha(AMBER, 28))
                            .px(9.0)
                            .py(5.0)
                            .bevel_rounded(5.5, 1.3)
                            .wrap();
                    }

                    if gd.is_owner {
                        ui.txt("Manage Group")
                            .sz(13.0)
                            .bold()
                            .fg(TEXT_PRIMARY);

                        ui.txt("Group Name").sz(10.5).semibold().fg(TEXT_MUTED);
                        ui.row(|ui| {
                            let mut name_inp = ui
                                .input("mm_grp_name_input", &mut state.main_menu.group_name_input)
                                .placeholder("Group name...")
                                .bg(card_inner_bg())
                                .fg(TEXT_PRIMARY)
                                .grow(1.0);
                            if state.host.is_vr && name_inp.clicked() {
                                actions.push(UiAction::OpenVirtualKeyboard {
                                    surface_id: 0,
                                    field_id: String::from("mm_grp_name_input"),
                                    initial_text: state.main_menu.group_name_input.text(),
                                    title: String::from("Group Name"),
                                    placeholder: String::from("Enter group name..."),
                                    max_len: 64,
                                    hidden: false,
                                });
                            }
                            if ui
                                .btn_with("mm_gd_save_name", |ui| {
                                    ui.txt(if gd.saving { "Saving..." } else { "Save Name" })
                                        .sz(10.5)
                                        .bold()
                                        .fg(TEXT_PRIMARY);
                                })
                                .px(11.0)
                                .py(6.5)
                                .bg(tint_alpha(accent, 52))
                                .hover_bg(tint_alpha(accent, 85))
                                .bd(1.0, tint_alpha(accent, 190))
                                .bevel_rounded(6.0, 1.3)
                                .clicked()
                                && !gd.saving
                            {
                                let next_name = state.main_menu.group_name_input.text().trim().to_string();
                                if !next_name.is_empty() {
                                    push_click(actions);
                                    actions.push(UiAction::SocialUpdateGroupName {
                                        group_id: gd.id.clone(),
                                        name: next_name,
                                    });
                                }
                            }
                        })
                        .align_center()
                        .gap(6.0);

                        ui.txt("Description")
                            .sz(10.5)
                            .semibold()
                            .fg(TEXT_MUTED)
                            .pt(4.0);
                        let mut desc_inp = ui
                            .input("mm_grp_desc_input", &mut state.main_menu.group_desc_input)
                            .placeholder("Group description...")
                            .bg(card_inner_bg())
                            .fg(TEXT_PRIMARY)
                            .w_frac(1.0);
                        if state.host.is_vr && desc_inp.clicked() {
                            actions.push(UiAction::OpenVirtualKeyboard {
                                surface_id: 0,
                                field_id: String::from("mm_grp_desc_input"),
                                initial_text: state.main_menu.group_desc_input.text(),
                                title: String::from("Group Description"),
                                placeholder: String::from("Enter group description..."),
                                max_len: 512,
                                hidden: false,
                            });
                        }

                        ui.row(|ui| {
                            if ui
                                .btn_with("mm_gd_save_desc", |ui| {
                                    ui.txt(if gd.saving {
                                        "Saving..."
                                    } else {
                                        "Save Description"
                                    })
                                    .sz(11.0)
                                    .bold()
                                    .align(TextAlign::Center)
                                    .fg(TEXT_PRIMARY)
                                    .grow(1.0);
                                })
                                .grow(1.0)
                                .py(7.0)
                                .bg(tint_alpha(EMERALD, 48))
                                .hover_bg(tint_alpha(EMERALD, 80))
                                .bd(1.0, tint_alpha(EMERALD, 185))
                                .bevel_rounded(6.0, 1.3)
                                .clicked()
                                && !gd.saving
                            {
                                push_click(actions);
                                actions.push(UiAction::SocialUpdateGroupDesc {
                                    group_id: gd.id.clone(),
                                    description: state.main_menu.group_desc_input.text(),
                                });
                            }
                        })
                        .gap(6.0);
                    } else {
                        ui.txt("About Group")
                            .sz(13.0)
                            .bold()
                            .fg(TEXT_PRIMARY);
                        ui.scroll("mm_gd_desc_ro", |ui| {
                            let desc = if gd.description.trim().is_empty() {
                                "No description provided."
                            } else {
                                &gd.description
                            };
                            ui.txt(desc).sz(12.0).fg(TEXT_PRIMARY).wrap();
                        })
                        .grow(1.0)
                        .p(9.0)
                        .bg(card_inner_bg())
                        .bd(1.0, border_subtle())
                        .bevel_rounded(7.0, 1.3)
                        .no_scrollbar();
                    }
                })
                .grow(1.0)
                .h_frac(1.0)
                .p(12.0)
                .gap(8.0)
                .bg(card_bg())
                .bd(1.0, border_subtle())
                .bevel_rounded(10.5, 2.0)
                .shadow(14.0, SHADOW_CARD);

                // Right: Members List
                ui.col(|ui| {
                    ui.txt(&format!("Members ({})", gd.members.len()))
                        .sz(13.0)
                        .bold()
                        .fg(TEXT_PRIMARY);

                    if gd.members.is_empty() {
                        ui.col(|ui| {
                            icons::icon(ui, icons::PLAYERS, 26.0).fg(TEXT_DIM);
                            ui.txt("No members loaded.").sz(12.0).fg(TEXT_MUTED);
                        })
                        .center()
                        .grow(1.0)
                        .gap(6.0);
                    } else {
                        ui.scroll("mm_gd_members_scroll", |ui| {
                            for m in &gd.members {
                                ui.row(|ui| {
                                    if ui
                                        .btn_with(&format!("mm_gd_m_{}", m.id), |ui| {
                                            ui.row(|ui| {
                                                if let Some(tex) = state.main_textures.get(&m.image_key) {
                                                    ui.img(tex)
                                                        .w(36.0)
                                                        .h(36.0)
                                                        .fit(ImageFit::Cover)
                                                        .hexagon_pointy()
                                                        .r(3.0)
                                                        .bd(1.5, tint_alpha(accent, 150));
                                                } else {
                                                    let init = m
                                                        .name
                                                        .chars()
                                                        .next()
                                                        .unwrap_or('M')
                                                        .to_ascii_uppercase()
                                                        .to_string();
                                                    ui.col(|ui| {
                                                        ui.txt(&init)
                                                            .sz(13.0)
                                                            .bold()
                                                            .align(TextAlign::Center)
                                                            .fg(accent);
                                                    })
                                                    .center()
                                                    .w(36.0)
                                                    .h(36.0)
                                                    .bg(tint_alpha(accent, 28))
                                                    .bd(1.5, tint_alpha(accent, 150))
                                                    .hexagon_pointy()
                                                    .r(3.0);
                                                }

                                                ui.col(|ui| {
                                                    ui.txt(&m.name)
                                                        .sz(12.0)
                                                        .semibold()
                                                        .fg(TEXT_PRIMARY)
                                                        .ellipsis();
                                                    if !m.role.is_empty() {
                                                        ui.txt(&m.role).sz(10.0).fg(TEXT_MUTED);
                                                    }
                                                })
                                                .gap(1.0)
                                                .grow(1.0);
                                            })
                                            .align_center()
                                            .gap(9.0)
                                            .grow(1.0);
                                        })
                                        .grow(1.0)
                                        .p(4.0)
                                        .bg(Color::TRANSPARENT)
                                        .hover_bg(tile_idle_hover())
                                        .borderless()
                                        .bevel_rounded(6.0, 1.3)
                                        .clicked()
                                    {
                                        push_click(actions);
                                        crate::main_menu::push_nav_history(state);
                                        state.main_menu.user_detail_tab = 0;
                                        state.main_menu.group_edit_id.clear();
                                        actions.push(UiAction::SocialCloseGroupDetail);
                                        actions.push(UiAction::SocialLoadUserDetail {
                                            user_id: m.id.clone(),
                                        });
                                    }

                                    if gd.is_owner && !m.id.is_empty() && m.id != gd.owner_id {
                                        let kick_key = format!("kick_grp_{}", m.id);
                                        let armed = state.main_menu.safety_confirm_action == kick_key;
                                        if ui
                                            .btn_with(&format!("mm_gd_kick_{}", m.id), |ui| {
                                                ui.txt(if armed { "Confirm?" } else { "Kick" })
                                                    .sz(10.0)
                                                    .semibold()
                                                    .fg(if armed { TEXT_PRIMARY } else { RED });
                                            })
                                            .px(9.0)
                                            .py(5.0)
                                            .bg(if armed { tint_alpha(RED, 75) } else { tile_idle_bg() })
                                            .hover_bg(tint_alpha(RED, 95))
                                            .bd(1.0, tint_alpha(RED, 160))
                                            .bevel_rounded(5.0, 1.3)
                                            .clicked()
                                        {
                                            push_click(actions);
                                            if armed {
                                                state.main_menu.safety_confirm_action.clear();
                                                actions.push(UiAction::SocialKickGroupMember {
                                                    group_id: gd.id.clone(),
                                                    target_id: m.id.clone(),
                                                });
                                            } else {
                                                state.main_menu.safety_confirm_action = kick_key;
                                            }
                                        }
                                    }
                                })
                                .align_center()
                                .p(6.0)
                                .gap(6.0)
                                .bg(card_inner_bg())
                                .bd(1.0, border_subtle())
                                .bevel_rounded(7.0, 1.3);
                            }
                        })
                        .grow(1.0)
                        .gap(6.0)
                        .no_scrollbar();
                    }
                })
                .grow(1.1)
                .h_frac(1.0)
                .p(12.0)
                .gap(8.0)
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
