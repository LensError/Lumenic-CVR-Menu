use lumenic::{Color, ImageFit, TextAlign, UiContext};

use crate::icons;
use crate::state::{AppState, UiAction, UserDetailSnapshot};
use crate::theme::{
    push_click, tint_alpha, AMBER, border_subtle, card_bg, card_inner_bg, EMERALD, RED,
    SHADOW_CARD, TEXT_DIM, TEXT_MUTED, TEXT_PRIMARY, tile_idle_bg, tile_idle_hover,
};

use super::render_group_detail_view;

pub fn render_user_detail_view(
    ui: &mut UiContext,
    state: &mut AppState,
    detail: &UserDetailSnapshot,
    accent: Color,
    actions: &mut Vec<UiAction>,
) {
    if let Some(gd) = state.host.social.group_detail.take() {
        render_group_detail_view(ui, state, &gd, accent, actions);
        state.host.social.group_detail = Some(gd);
        return;
    }
    ui.col(|ui| {
        // Header bar with Back button, User name, Rank pill, and Overview / Badges / Groups sub-tabs
        ui.row(|ui| {
            if ui
                .btn_with("mm_ud_back", |ui| {
                    ui.row(|ui| {
                        icons::icon(ui, icons::BACK, 14.0).fg(TEXT_PRIMARY);
                        ui.txt("Back").sz(crate::theme::fs(11.5)).semibold().fg(TEXT_PRIMARY);
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
            .sz(crate::theme::fs(15.0))
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
                    .sz(crate::theme::fs(13.0))
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
                ui.txt(&detail.error).sz(crate::theme::fs(12.5)).fg(AMBER).wrap();
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
                                    .sz(crate::theme::fs(11.0))
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
                            .sz(crate::theme::fs(13.0))
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
                                                    .sz(crate::theme::fs(12.5))
                                                    .bold()
                                                    .fg(TEXT_PRIMARY)
                                                    .ellipsis()
                                                    .grow(1.0);
                                                if b.featured {
                                                    ui.txt("FEATURED")
                                                        .sz(crate::theme::fs(9.0))
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
                                                    .sz(crate::theme::fs(10.5))
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
                                                        .sz(crate::theme::fs(10.5))
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
                            .sz(crate::theme::fs(13.0))
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
                                                    .sz(crate::theme::fs(12.5))
                                                    .bold()
                                                    .fg(TEXT_PRIMARY)
                                                    .ellipsis()
                                                    .grow(1.0);
                                                if !g.tag.is_empty() {
                                                    ui.txt(&format!("[{}]", g.tag))
                                                        .sz(crate::theme::fs(9.5))
                                                        .fg(accent)
                                                        .bg(tint_alpha(accent, 28))
                                                        .px(5.0)
                                                        .py(1.0)
                                                        .bevel_rounded(4.0, 1.1);
                                                }
                                                if g.is_featured {
                                                    ui.txt("FEATURED")
                                                        .sz(crate::theme::fs(9.0))
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
                                            .sz(crate::theme::fs(10.5))
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
                                                                .sz(crate::theme::fs(10.5))
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
                                                            .sz(crate::theme::fs(10.5))
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
                                    .sz(crate::theme::fs(26.0))
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
                                    .sz(crate::theme::fs(18.0))
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
                                                    .sz(crate::theme::fs(10.5))
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
                                                .sz(crate::theme::fs(10.5))
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
                                        ui.txt(&detail.pronouns).sz(crate::theme::fs(11.0)).fg(TEXT_MUTED);
                                    }
                                    if crate::theme::is_special_rank(&detail.rank) {
                                        ui.row(|ui| {
                                            ui.txt(&detail.rank)
                                                .sz(crate::theme::fs(10.0))
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
                                        .sz(crate::theme::fs(11.0))
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
                                        .sz(crate::theme::fs(11.0))
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
                                .sz(crate::theme::fs(11.0))
                                .semibold()
                                .fg(TEXT_MUTED)
                                .grow(1.0);
                            if !detail.profile_status.is_empty() {
                                ui.txt(&detail.profile_status)
                                    .sz(crate::theme::fs(10.0))
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
                            .sz(crate::theme::fs(11.0))
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
                                        .sz(crate::theme::fs(11.0))
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
                                        .sz(crate::theme::fs(11.0))
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
                        ui.txt("Bio").sz(crate::theme::fs(11.0)).semibold().fg(TEXT_MUTED);
                        ui.scroll("mm_ud_bio_scroll", |ui| {
                            let bio = if detail.bio.trim().is_empty() {
                                "No bio set."
                            } else {
                                &detail.bio
                            };
                            ui.txt(bio).sz(crate::theme::fs(12.0)).fg(TEXT_PRIMARY).wrap();
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
                            ui.txt("Notes").sz(crate::theme::fs(11.0)).semibold().fg(TEXT_MUTED).grow(1.0);
                            if !detail.note_status.is_empty() {
                                ui.txt(&detail.note_status)
                                    .sz(crate::theme::fs(10.0))
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
                                            .sz(crate::theme::fs(11.0))
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
                                    .sz(crate::theme::fs(11.0))
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
                        ui.txt(&status_lbl).sz(crate::theme::fs(12.5)).bold().fg(
                            if detail.online_state || detail.is_local {
                                EMERALD
                            } else {
                                TEXT_DIM
                            },
                        );

                        if !detail.world_name.is_empty() {
                            ui.txt(&format!("World: {}", detail.world_name))
                                .sz(crate::theme::fs(11.5))
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
                                                    .sz(crate::theme::fs(8.5))
                                                    .bold()
                                                    .fg(TEXT_DIM);
                                                ui.txt(&detail.avatar_name)
                                                    .sz(crate::theme::fs(11.5))
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
                                    .sz(crate::theme::fs(11.5))
                                    .fg(TEXT_MUTED);
                            }
                        }

                        if !detail.instance_id.is_empty() {
                            ui.row(|ui| {
                                if !detail.is_local
                                    && ui
                                        .btn_with("mm_ud_join_inst", |ui| {
                                            ui.txt("Join Instance")
                                                .sz(crate::theme::fs(12.0))
                                                .bold()
                                                .align(TextAlign::Center)
                                                .fg(TEXT_PRIMARY)
                                                .grow(1.0);
                                        })
                                        .grow(1.0)
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

                                if ui
                                    .btn_with("mm_ud_inst_details", |ui| {
                                        ui.row(|ui| {
                                            icons::icon(ui, icons::INSTANCE, 13.5).fg(accent);
                                            ui.txt("Instance Details")
                                                .sz(crate::theme::fs(11.5))
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
                                    crate::main_menu::push_nav_history(state);
                                    state.main_menu.active_tab = String::from("worlds");
                                    actions.push(UiAction::SocialCloseUserDetail);
                                    actions.push(UiAction::SocialCloseGroupDetail);
                                    actions.push(UiAction::ContentLoadInstanceDetail {
                                        instance_id: detail.instance_id.clone(),
                                    });
                                }
                            })
                            .gap(6.0);
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
                                .sz(crate::theme::fs(12.5))
                                .bold()
                                .fg(TEXT_PRIMARY);
                            ui.txt("Use the Badges and Groups tabs above to inspect or feature your ChilloutVR badges and groups.")
                                .sz(crate::theme::fs(11.0))
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
                                            .sz(crate::theme::fs(11.0))
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
                                            .sz(crate::theme::fs(11.0))
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
                            ui.txt("Actions").sz(crate::theme::fs(12.5)).bold().fg(TEXT_PRIMARY);

                            ui.row(|ui| {
                                if ui
                                    .btn_with("mm_ud_invite", |ui| {
                                        ui.txt("Invite")
                                            .sz(crate::theme::fs(11.5))
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
                                            .sz(crate::theme::fs(11.5))
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
                                                .sz(crate::theme::fs(11.0))
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
                                                .sz(crate::theme::fs(10.5))
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
                                        .sz(crate::theme::fs(10.5))
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
                                .sz(crate::theme::fs(12.5))
                                .bold()
                                .fg(TEXT_PRIMARY);

                            // Avatar Visibility (Hide / Filter / Show)
                            ui.row(|ui| {
                                ui.txt("Avatar").sz(crate::theme::fs(10.0)).fg(TEXT_MUTED).w(44.0);
                                for (mode, lbl) in [(0i32, "Hide"), (1i32, "Filter"), (2i32, "Show")]
                                {
                                    let active = detail.avatar_visibility == mode;
                                    if ui
                                        .btn_with(&format!("mm_ud_avv_{mode}"), |ui| {
                                            ui.txt(lbl)
                                                .sz(crate::theme::fs(10.0))
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
                                ui.txt("Props").sz(crate::theme::fs(10.0)).fg(TEXT_MUTED).w(44.0);
                                for (mode, lbl) in [(0i32, "Hide"), (1i32, "Filter"), (2i32, "Show")]
                                {
                                    let active = detail.prop_visibility == mode;
                                    if ui
                                        .btn_with(&format!("mm_ud_prv_{mode}"), |ui| {
                                            ui.txt(lbl)
                                                .sz(crate::theme::fs(10.0))
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
                                            .sz(crate::theme::fs(10.5))
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
                                            .sz(crate::theme::fs(10.5))
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
                                        .sz(crate::theme::fs(10.5))
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

