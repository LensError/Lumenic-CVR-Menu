use lumenic::{Color, ImageFit, TextAlign, UiContext};

use crate::icons;
use crate::state::{AppState, GroupDetailSnapshot, UiAction};
use crate::theme::{
    push_click, tint_alpha, AMBER, border_subtle, card_bg, card_inner_bg, EMERALD, RED,
    SHADOW_CARD, TEXT_DIM, TEXT_MUTED, TEXT_PRIMARY, tile_idle_bg, tile_idle_hover,
};

fn group_role_color(role: &str, accent: Color) -> Color {
    match role.to_ascii_lowercase().as_str() {
        "owner" => EMERALD,
        "admin" => AMBER,
        "moderator" => Color::rgb(56, 189, 248),
        "trial" => accent,
        _ => TEXT_MUTED,
    }
}

/// Renders the revamped Group Detail & Management view (Members with individual/mass role tools + Settings & Info).
pub fn render_group_detail_view(
    ui: &mut UiContext,
    state: &mut AppState,
    gd: &GroupDetailSnapshot,
    accent: Color,
    actions: &mut Vec<UiAction>,
) {
    if !gd.loading && !gd.id.is_empty() && state.main_menu.group_edit_id != gd.id {
        state.main_menu.group_edit_id = gd.id.clone();
        state.main_menu.group_detail_tab = 0;
        state.main_menu.group_name_input.set_text(&gd.name);
        state.main_menu.group_desc_input.set_text(&gd.description);
        state.main_menu.group_member_search.set_text("");
        state.main_menu.group_member_role_filter = String::from("all");
        state.main_menu.group_selected_members.clear();
        state.main_menu.group_mass_confirm = false;
    }

    // Prune any selected members that are no longer eligible (e.g. just became Member or kicked).
    state.main_menu.group_selected_members.retain(|uid| {
        gd.members.iter().any(|m| {
            m.id == *uid
                && m.id != gd.owner_id
                && m.id != gd.local_user_id
                && !m.role.eq_ignore_ascii_case("Member")
        })
    });
    if state.main_menu.group_selected_members.is_empty() {
        state.main_menu.group_mass_confirm = false;
    }

    ui.col(|ui| {
        // =====================================================================
        // HEADER BAR
        // =====================================================================
        ui.row(|ui| {
            if ui
                .btn_with("mm_gd_back", |ui| {
                    ui.row(|ui| {
                        icons::icon(ui, icons::BACK, 14.0).fg(TEXT_PRIMARY);
                        ui.txt("Back")
                            .sz(crate::theme::fs(11.5))
                            .semibold()
                            .fg(TEXT_PRIMARY);
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
                    .w(34.0)
                    .h(34.0)
                    .fit(ImageFit::Cover)
                    .bevel_rounded(6.0, 1.3)
                    .bd(1.0, tint_alpha(accent, 160));
            } else {
                icons::icon(ui, icons::PLAYERS, 17.0).fg(accent);
            }

            ui.col(|ui| {
                ui.row(|ui| {
                    let title = if gd.name.is_empty() {
                        "Loading group..."
                    } else {
                        &gd.name
                    };
                    ui.txt(title)
                        .sz(crate::theme::fs(14.5))
                        .bold()
                        .fg(TEXT_PRIMARY)
                        .ellipsis();

                    if !gd.tag.is_empty() {
                        ui.txt(&format!("[{}]", gd.tag))
                            .sz(crate::theme::fs(9.5))
                            .semibold()
                            .fg(TEXT_PRIMARY)
                            .px(6.0)
                            .py(1.5)
                            .bg(tint_alpha(accent, 36))
                            .bd(1.0, tint_alpha(accent, 140))
                            .bevel_rounded(4.0, 1.0);
                    }

                    if !gd.local_role.is_empty() {
                        let rcol = group_role_color(&gd.local_role, accent);
                        ui.txt(&gd.local_role)
                            .sz(crate::theme::fs(9.5))
                            .semibold()
                            .fg(rcol)
                            .px(6.0)
                            .py(1.5)
                            .bg(tint_alpha(rcol, 28))
                            .bd(1.0, tint_alpha(rcol, 140))
                            .bevel_rounded(4.0, 1.0);
                    }
                })
                .align_center()
                .gap(6.0);

                let sub = format!(
                    "{} members{}",
                    gd.member_count.max(gd.members.len() as i32),
                    if gd.owner_name.is_empty() {
                        String::new()
                    } else {
                        format!("  |  Owner: {}", gd.owner_name)
                    }
                );
                ui.txt(&sub)
                    .sz(crate::theme::fs(10.5))
                    .fg(TEXT_MUTED)
                    .ellipsis();
            })
            .gap(1.5)
            .grow(1.0);

            if !gd.loading && !gd.id.is_empty() {
                if gd.is_member && !gd.is_featured {
                    if ui
                        .btn_with("mm_gd_feature", |ui| {
                            ui.txt("Feature")
                                .sz(crate::theme::fs(10.5))
                                .semibold()
                                .fg(TEXT_PRIMARY);
                        })
                        .px(10.0)
                        .py(5.5)
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

                if gd.is_member && !gd.is_owner {
                    let leave_armed = state.main_menu.safety_confirm_action == "leave_group";
                    if ui
                        .btn_with("mm_gd_leave", |ui| {
                            ui.txt(if leave_armed {
                                "Confirm Leave?"
                            } else {
                                "Leave"
                            })
                            .sz(crate::theme::fs(10.5))
                            .semibold()
                            .fg(if leave_armed { TEXT_PRIMARY } else { AMBER });
                        })
                        .px(10.0)
                        .py(5.5)
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
                            state.main_menu.safety_confirm_action = String::from("leave_group");
                        }
                    }
                }

                if gd.is_owner {
                    let del_armed = state.main_menu.safety_confirm_action == "delete_group";
                    if ui
                        .btn_with("mm_gd_delete", |ui| {
                            ui.txt(if del_armed {
                                "Confirm Delete?"
                            } else {
                                "Delete Group"
                            })
                            .sz(crate::theme::fs(10.5))
                            .semibold()
                            .fg(if del_armed { TEXT_PRIMARY } else { RED });
                        })
                        .px(10.0)
                        .py(5.5)
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
                            state.main_menu.safety_confirm_action = String::from("delete_group");
                        }
                    }
                }
            }
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
        } else {
            // =================================================================
            // SUB-TAB SWITCHER + STATUS FEEDBACK
            // =================================================================
            ui.row(|ui| {
                let members_lbl = format!(
                    "Members ({} / {})",
                    gd.members.len(),
                    gd.member_count.max(gd.members.len() as i32)
                );
                let settings_lbl = if gd.can_edit {
                    "Settings & Info"
                } else {
                    "About & Settings"
                };

                for (tab_idx, ic, lbl) in [
                    (0usize, icons::PLAYERS, members_lbl.as_str()),
                    (1usize, icons::SETTINGS, settings_lbl),
                ] {
                    let active = state.main_menu.group_detail_tab == tab_idx;
                    if ui
                        .btn_with(&format!("mm_gd_tab_{tab_idx}"), |ui| {
                            ui.row(|ui| {
                                icons::icon(ui, ic, 13.0)
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
                        state.main_menu.group_detail_tab = tab_idx;
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

            if !gd.status_msg.is_empty() || !gd.error.is_empty() {
                ui.row(|ui| {
                    if !gd.status_msg.is_empty() {
                        ui.txt(&gd.status_msg)
                            .sz(crate::theme::fs(11.0))
                            .semibold()
                            .fg(EMERALD)
                            .bg(tint_alpha(EMERALD, 28))
                            .px(9.0)
                            .py(5.0)
                            .bevel_rounded(5.5, 1.3)
                            .grow(1.0);
                    }
                    if !gd.error.is_empty() {
                        ui.txt(&gd.error)
                            .sz(crate::theme::fs(11.0))
                            .semibold()
                            .fg(AMBER)
                            .bg(tint_alpha(AMBER, 28))
                            .px(9.0)
                            .py(5.0)
                            .bevel_rounded(5.5, 1.3)
                            .wrap()
                            .grow(1.0);
                    }
                })
                .gap(8.0);
            }

            if state.main_menu.group_detail_tab == 0 {
                // =============================================================
                // TAB 0: MEMBERS (SEARCH + ROLE FILTER + MASS ROLE + LIST)
                // =============================================================
                let trial_ids: Vec<String> = gd
                    .members
                    .iter()
                    .filter(|m| {
                        m.id != gd.owner_id
                            && m.id != gd.local_user_id
                            && m.role.eq_ignore_ascii_case("Trial")
                    })
                    .map(|m| m.id.clone())
                    .collect();

                let q = state
                    .main_menu
                    .group_member_search
                    .text()
                    .trim()
                    .to_ascii_lowercase();
                let role_f = state.main_menu.group_member_role_filter.as_str();
                let filtered: Vec<&crate::state::GroupMemberItem> = gd
                    .members
                    .iter()
                    .filter(|m| {
                        let match_role =
                            role_f == "all" || m.role.eq_ignore_ascii_case(role_f);
                        let match_q = q.is_empty()
                            || m.name.to_ascii_lowercase().contains(&q)
                            || m.role.to_ascii_lowercase().contains(&q);
                        match_role && match_q
                    })
                    .collect();

                let visible_eligible_ids: Vec<String> = filtered
                    .iter()
                    .filter(|m| {
                        m.id != gd.owner_id
                            && m.id != gd.local_user_id
                            && !m.role.eq_ignore_ascii_case("Member")
                    })
                    .map(|m| m.id.clone())
                    .collect();

                ui.col(|ui| {
                    // Search + Role Filter Row
                    ui.row(|ui| {
                        ui.row(|ui| {
                            icons::icon(ui, icons::SEARCH, 13.0).fg(TEXT_MUTED);
                            let mut s_inp = ui
                                .input(
                                    "mm_gd_member_search",
                                    &mut state.main_menu.group_member_search,
                                )
                                .placeholder("Search members...")
                                .bg(card_inner_bg())
                                .fg(TEXT_PRIMARY)
                                .grow(1.0);
                            if state.host.is_vr && s_inp.clicked() {
                                actions.push(UiAction::OpenVirtualKeyboard {
                                    surface_id: 0,
                                    field_id: String::from("mm_gd_member_search"),
                                    initial_text: state.main_menu.group_member_search.text(),
                                    title: String::from("Search Group Members"),
                                    placeholder: String::from("Search members..."),
                                    max_len: 64,
                                    hidden: false,
                                });
                            }
                        })
                        .align_center()
                        .gap(6.0)
                        .grow(1.0);

                        let trial_lbl = if trial_ids.is_empty() {
                            String::from("Trial")
                        } else {
                            format!("Trial ({})", trial_ids.len())
                        };
                        for (fkey, lbl) in [
                            ("all", "All"),
                            ("Trial", trial_lbl.as_str()),
                            ("Member", "Member"),
                            ("Moderator", "Mod"),
                            ("Admin", "Admin"),
                        ] {
                            let active = state.main_menu.group_member_role_filter == fkey;
                            if ui
                                .btn_with(&format!("mm_gd_rf_{fkey}"), |ui| {
                                    ui.txt(lbl)
                                        .sz(crate::theme::fs(10.5))
                                        .semibold()
                                        .fg(if active { TEXT_PRIMARY } else { TEXT_MUTED });
                                })
                                .px(9.5)
                                .py(5.0)
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
                                state.main_menu.group_member_role_filter = String::from(fkey);
                            }
                        }
                    })
                    .align_center()
                    .gap(6.0);

                    // Mass Role Action Bar (Admin / Owner only - strictly sets role to Member)
                    if gd.can_manage_roles {
                        ui.row(|ui| {
                            if gd.mass_role_running {
                                icons::icon(ui, icons::RELOAD, 14.0).fg(accent);
                                let prog = if gd.mass_role_failed > 0 {
                                    format!(
                                        "Setting to Member: {} / {} ({} failed)...",
                                        gd.mass_role_done, gd.mass_role_total, gd.mass_role_failed
                                    )
                                } else {
                                    format!(
                                        "Setting to Member: {} / {}...",
                                        gd.mass_role_done, gd.mass_role_total
                                    )
                                };
                                ui.txt(&prog)
                                    .sz(crate::theme::fs(11.5))
                                    .semibold()
                                    .fg(TEXT_PRIMARY)
                                    .grow(1.0);

                                if ui
                                    .btn_with("mm_gd_mass_cancel", |ui| {
                                        ui.txt("Cancel")
                                            .sz(crate::theme::fs(10.5))
                                            .semibold()
                                            .fg(RED);
                                    })
                                    .px(11.0)
                                    .py(5.0)
                                    .bg(tile_idle_bg())
                                    .hover_bg(tint_alpha(RED, 60))
                                    .bd(1.0, tint_alpha(RED, 165))
                                    .bevel_rounded(5.5, 1.3)
                                    .clicked()
                                {
                                    push_click(actions);
                                    actions.push(UiAction::SocialCancelMassGroupRole);
                                }
                            } else {
                                let sel_count = state.main_menu.group_selected_members.len();
                                ui.txt(&format!("{sel_count} selected"))
                                    .sz(crate::theme::fs(11.0))
                                    .semibold()
                                    .fg(if sel_count > 0 { accent } else { TEXT_MUTED });

                                if !trial_ids.is_empty()
                                    && ui
                                        .btn_with("mm_gd_sel_trials", |ui| {
                                            ui.txt(&format!("Select All Trial ({})", trial_ids.len()))
                                                .sz(crate::theme::fs(10.5))
                                                .semibold()
                                                .fg(TEXT_PRIMARY);
                                        })
                                        .px(9.5)
                                        .py(4.5)
                                        .bg(tint_alpha(accent, 34))
                                        .hover_bg(tint_alpha(accent, 65))
                                        .bd(1.0, tint_alpha(accent, 145))
                                        .bevel_rounded(5.0, 1.2)
                                        .clicked()
                                {
                                    push_click(actions);
                                    state.main_menu.group_mass_confirm = false;
                                    for uid in &trial_ids {
                                        state
                                            .main_menu
                                            .group_selected_members
                                            .insert(uid.clone());
                                    }
                                }

                                if !visible_eligible_ids.is_empty()
                                    && ui
                                        .btn_with("mm_gd_sel_visible", |ui| {
                                            ui.txt("Select Visible")
                                                .sz(crate::theme::fs(10.5))
                                                .semibold()
                                                .fg(TEXT_MUTED);
                                        })
                                        .px(9.0)
                                        .py(4.5)
                                        .bg(tile_idle_bg())
                                        .hover_bg(tile_idle_hover())
                                        .bd(1.0, border_subtle())
                                        .bevel_rounded(5.0, 1.2)
                                        .clicked()
                                {
                                    push_click(actions);
                                    state.main_menu.group_mass_confirm = false;
                                    for uid in &visible_eligible_ids {
                                        state
                                            .main_menu
                                            .group_selected_members
                                            .insert(uid.clone());
                                    }
                                }

                                if sel_count > 0
                                    && ui
                                        .btn_with("mm_gd_sel_clear", |ui| {
                                            ui.txt("Clear")
                                                .sz(crate::theme::fs(10.5))
                                                .semibold()
                                                .fg(TEXT_MUTED);
                                        })
                                        .px(8.5)
                                        .py(4.5)
                                        .bg(tile_idle_bg())
                                        .hover_bg(tile_idle_hover())
                                        .bd(1.0, border_subtle())
                                        .bevel_rounded(5.0, 1.2)
                                        .clicked()
                                {
                                    push_click(actions);
                                    state.main_menu.group_selected_members.clear();
                                    state.main_menu.group_mass_confirm = false;
                                }

                                ui.col(|_| {}).grow(1.0);

                                let armed = state.main_menu.group_mass_confirm && sel_count > 0;
                                let mass_lbl = if armed {
                                    format!("Confirm: Set {sel_count} to Member?")
                                } else {
                                    String::from("Set Selected to Member")
                                };
                                if ui
                                    .btn_with("mm_gd_mass_set_member", |ui| {
                                        ui.row(|ui| {
                                            icons::icon(ui, icons::CHECK, 13.0).fg(if sel_count > 0 {
                                                EMERALD
                                            } else {
                                                TEXT_DIM
                                            });
                                            ui.txt(&mass_lbl)
                                                .sz(crate::theme::fs(10.5))
                                                .bold()
                                                .fg(if sel_count > 0 {
                                                    TEXT_PRIMARY
                                                } else {
                                                    TEXT_DIM
                                                });
                                        })
                                        .center()
                                        .gap(5.0);
                                    })
                                    .px(11.0)
                                    .py(5.0)
                                    .bg(if armed {
                                        tint_alpha(EMERALD, 75)
                                    } else if sel_count > 0 {
                                        tint_alpha(EMERALD, 40)
                                    } else {
                                        tile_idle_bg()
                                    })
                                    .hover_bg(if sel_count > 0 {
                                        tint_alpha(EMERALD, 85)
                                    } else {
                                        tile_idle_bg()
                                    })
                                    .bd(
                                        1.0,
                                        if sel_count > 0 {
                                            tint_alpha(EMERALD, 185)
                                        } else {
                                            border_subtle()
                                        },
                                    )
                                    .bevel_rounded(5.5, 1.3)
                                    .opacity(if sel_count > 0 { 1.0 } else { 0.5 })
                                    .clicked()
                                    && sel_count > 0
                                {
                                    push_click(actions);
                                    if armed {
                                        let ids: Vec<String> = state
                                            .main_menu
                                            .group_selected_members
                                            .iter()
                                            .cloned()
                                            .collect();
                                        state.main_menu.group_mass_confirm = false;
                                        actions.push(
                                            UiAction::SocialMassSetGroupMembersToMember {
                                                group_id: gd.id.clone(),
                                                user_ids: ids,
                                            },
                                        );
                                    } else {
                                        state.main_menu.group_mass_confirm = true;
                                    }
                                }
                            }
                        })
                        .align_center()
                        .px(9.0)
                        .py(6.5)
                        .gap(6.0)
                        .bg(card_inner_bg())
                        .bd(1.0, tint_alpha(accent, 95))
                        .bevel_rounded(7.0, 1.3);
                    }

                    // Member Roster Scroll
                    if filtered.is_empty() {
                        ui.col(|ui| {
                            icons::icon(ui, icons::PLAYERS, 26.0).fg(TEXT_DIM);
                            ui.txt(if gd.members.is_empty() {
                                "No members loaded."
                            } else {
                                "No members match the current filter."
                            })
                            .sz(crate::theme::fs(12.0))
                            .fg(TEXT_MUTED);
                        })
                        .center()
                        .grow(1.0)
                        .gap(6.0);
                    } else {
                        ui.scroll("mm_gd_members_scroll", |ui| {
                            for m in &filtered {
                                let is_owner_row = !m.id.is_empty() && m.id == gd.owner_id;
                                let is_self_row = !m.id.is_empty() && m.id == gd.local_user_id;
                                let can_edit_member =
                                    gd.can_manage_roles && !m.id.is_empty() && !is_owner_row && !is_self_row;
                                let can_mass_select =
                                    can_edit_member && !m.role.eq_ignore_ascii_case("Member");
                                let selected = state.main_menu.group_selected_members.contains(&m.id);
                                let role_lbl = if is_owner_row {
                                    "Owner"
                                } else if m.role.is_empty() {
                                    "Member"
                                } else {
                                    m.role.as_str()
                                };
                                let role_col = group_role_color(role_lbl, accent);

                                ui.row(|ui| {
                                    if gd.can_manage_roles {
                                        if ui
                                            .btn_with(&format!("mm_gd_chk_{}", m.id), |ui| {
                                                if selected {
                                                    icons::icon(ui, icons::CHECK, 13.0).fg(EMERALD);
                                                }
                                            })
                                            .center()
                                            .w(24.0)
                                            .h(24.0)
                                            .bg(if selected {
                                                tint_alpha(EMERALD, 48)
                                            } else {
                                                tile_idle_bg()
                                            })
                                            .hover_bg(if can_mass_select {
                                                tile_idle_hover()
                                            } else {
                                                tile_idle_bg()
                                            })
                                            .bd(
                                                1.0,
                                                if selected {
                                                    tint_alpha(EMERALD, 195)
                                                } else {
                                                    border_subtle()
                                                },
                                            )
                                            .bevel_rounded(5.0, 1.1)
                                            .opacity(if can_mass_select && !gd.mass_role_running {
                                                1.0
                                            } else {
                                                0.28
                                            })
                                            .clicked()
                                            && can_mass_select
                                            && !gd.mass_role_running
                                        {
                                            push_click(actions);
                                            state.main_menu.group_mass_confirm = false;
                                            if selected {
                                                state.main_menu.group_selected_members.remove(&m.id);
                                            } else {
                                                state
                                                    .main_menu
                                                    .group_selected_members
                                                    .insert(m.id.clone());
                                            }
                                        }
                                    }

                                    // Clickable member identity -> opens User Details
                                    if ui
                                        .btn_with(&format!("mm_gd_m_{}", m.id), |ui| {
                                            ui.row(|ui| {
                                                if let Some(tex) = state.main_textures.get(&m.image_key) {
                                                    ui.img(tex)
                                                        .w(34.0)
                                                        .h(34.0)
                                                        .fit(ImageFit::Cover)
                                                        .hexagon_pointy()
                                                        .r(3.0)
                                                        .bd(1.5, tint_alpha(role_col, 175));
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
                                                            .sz(crate::theme::fs(12.5))
                                                            .bold()
                                                            .align(TextAlign::Center)
                                                            .fg(role_col);
                                                    })
                                                    .center()
                                                    .w(34.0)
                                                    .h(34.0)
                                                    .bg(tint_alpha(role_col, 28))
                                                    .bd(1.5, tint_alpha(role_col, 155))
                                                    .hexagon_pointy()
                                                    .r(3.0);
                                                }

                                                ui.txt(&m.name)
                                                    .sz(crate::theme::fs(12.0))
                                                    .semibold()
                                                    .fg(TEXT_PRIMARY)
                                                    .ellipsis()
                                                    .grow(1.0);

                                                ui.txt(role_lbl)
                                                    .sz(crate::theme::fs(9.5))
                                                    .semibold()
                                                    .fg(role_col)
                                                    .px(6.5)
                                                    .py(2.0)
                                                    .bg(tint_alpha(role_col, 26))
                                                    .bd(1.0, tint_alpha(role_col, 125))
                                                    .bevel_rounded(4.5, 1.0);
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

                                    // Individual Role Picker (Owner / Admin)
                                    if can_edit_member {
                                        if ui
                                            .btn_with(&format!("mm_gd_role_{}", m.id), |ui| {
                                                ui.row(|ui| {
                                                    ui.txt(role_lbl)
                                                        .sz(crate::theme::fs(10.0))
                                                        .semibold()
                                                        .fg(TEXT_PRIMARY)
                                                        .grow(1.0);
                                                    icons::icon(ui, icons::CHEVRON_DOWN, 12.5)
                                                        .fg(accent);
                                                })
                                                .align_center()
                                                .gap(4.0)
                                                .grow(1.0);
                                            })
                                            .w(96.0)
                                            .px(8.0)
                                            .py(5.0)
                                            .bg(tile_idle_bg())
                                            .hover_bg(tile_idle_hover())
                                            .bd(1.0, tint_alpha(accent, 130))
                                            .bevel_rounded(5.0, 1.2)
                                            .opacity(if gd.mass_role_running { 0.5 } else { 1.0 })
                                            .clicked()
                                            && !gd.mass_role_running
                                        {
                                            push_click(actions);
                                            let roles = ["Trial", "Member", "Moderator", "Admin"];
                                            let options = roles
                                                .iter()
                                                .map(|r| crate::state::DropdownOption {
                                                    label: String::from(*r),
                                                    action: UiAction::SocialSetGroupMemberRole {
                                                        group_id: gd.id.clone(),
                                                        user_id: m.id.clone(),
                                                        role: String::from(*r),
                                                    },
                                                    selected: m.role.eq_ignore_ascii_case(r),
                                                })
                                                .collect();
                                            state.main_menu.active_dropdown =
                                                Some(crate::state::ActiveDropdown {
                                                    title: format!("Set Role: {}", m.name),
                                                    options,
                                                });
                                        }
                                    }

                                    // Kick button (Owner / Kick permission)
                                    if gd.can_kick && !m.id.is_empty() && !is_owner_row && !is_self_row {
                                        let kick_key = format!("kick_grp_{}", m.id);
                                        let armed = state.main_menu.safety_confirm_action == kick_key;
                                        if ui
                                            .btn_with(&format!("mm_gd_kick_{}", m.id), |ui| {
                                                ui.txt(if armed { "Confirm?" } else { "Kick" })
                                                    .sz(crate::theme::fs(10.0))
                                                    .semibold()
                                                    .fg(if armed { TEXT_PRIMARY } else { RED });
                                            })
                                            .px(9.0)
                                            .py(5.0)
                                            .bg(if armed {
                                                tint_alpha(RED, 75)
                                            } else {
                                                tile_idle_bg()
                                            })
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
                                .px(7.0)
                                .py(5.0)
                                .gap(6.0)
                                .bg(if selected {
                                    tint_alpha(EMERALD, 16)
                                } else {
                                    card_inner_bg()
                                })
                                .bd(
                                    1.0,
                                    if selected {
                                        tint_alpha(EMERALD, 120)
                                    } else {
                                        border_subtle()
                                    },
                                )
                                .bevel_rounded(7.0, 1.3);
                            }

                            // Pagination footer
                            if gd.members_page + 1 < gd.members_total_pages {
                                ui.row(|ui| {
                                    if gd.loading_more_members {
                                        ui.txt("Loading more members...")
                                            .sz(crate::theme::fs(11.0))
                                            .semibold()
                                            .fg(TEXT_MUTED);
                                    } else if ui
                                        .btn_with("mm_gd_load_more_members", |ui| {
                                            ui.txt(&format!(
                                                "Load More Members (Page {} / {})",
                                                gd.members_page + 2,
                                                gd.members_total_pages
                                            ))
                                            .sz(crate::theme::fs(11.0))
                                            .semibold()
                                            .fg(TEXT_PRIMARY);
                                        })
                                        .px(14.0)
                                        .py(6.0)
                                        .bg(tint_alpha(accent, 38))
                                        .hover_bg(tint_alpha(accent, 72))
                                        .bd(1.0, tint_alpha(accent, 155))
                                        .bevel_rounded(6.0, 1.3)
                                        .clicked()
                                    {
                                        push_click(actions);
                                        actions.push(UiAction::SocialLoadMoreGroupMembers {
                                            group_id: gd.id.clone(),
                                        });
                                    }
                                })
                                .center()
                                .py(6.0);
                            }
                        })
                        .grow(1.0)
                        .gap(5.5)
                        .no_scrollbar();
                    }
                })
                .grow(1.0)
                .p(11.0)
                .gap(8.0)
                .bg(card_bg())
                .bd(1.0, border_subtle())
                .bevel_rounded(10.5, 2.0)
                .shadow(14.0, SHADOW_CARD);
            } else {
                // =============================================================
                // TAB 1: SETTINGS & INFO (IDENTITY + PRIVACY SETTINGS)
                // =============================================================
                ui.row(|ui| {
                    // Left: Group Name & Description
                    ui.col(|ui| {
                        if gd.can_edit {
                            ui.txt("Group Identity")
                                .sz(crate::theme::fs(13.0))
                                .bold()
                                .fg(TEXT_PRIMARY);

                            ui.txt("Group Name")
                                .sz(crate::theme::fs(10.5))
                                .semibold()
                                .fg(TEXT_MUTED);
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
                                            .sz(crate::theme::fs(10.5))
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
                                    let next_name =
                                        state.main_menu.group_name_input.text().trim().to_string();
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
                                .sz(crate::theme::fs(10.5))
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

                            if ui
                                .btn_with("mm_gd_save_desc", |ui| {
                                    ui.txt(if gd.saving {
                                        "Saving..."
                                    } else {
                                        "Save Description"
                                    })
                                    .sz(crate::theme::fs(11.0))
                                    .bold()
                                    .align(TextAlign::Center)
                                    .fg(TEXT_PRIMARY)
                                    .grow(1.0);
                                })
                                .w_frac(1.0)
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
                        } else {
                            ui.txt("About Group")
                                .sz(crate::theme::fs(13.0))
                                .bold()
                                .fg(TEXT_PRIMARY);
                            ui.scroll("mm_gd_desc_ro", |ui| {
                                let desc = if gd.description.trim().is_empty() {
                                    "No description provided."
                                } else {
                                    &gd.description
                                };
                                ui.txt(desc)
                                    .sz(crate::theme::fs(12.0))
                                    .fg(TEXT_PRIMARY)
                                    .wrap();
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

                    // Right: Group Privacy & Visibility Settings
                    ui.col(|ui| {
                        ui.txt("Privacy & Visibility")
                            .sz(crate::theme::fs(13.0))
                            .bold()
                            .fg(TEXT_PRIMARY);

                        // Listed in Search toggle
                        ui.row(|ui| {
                            ui.col(|ui| {
                                ui.txt("Listed in Search")
                                    .sz(crate::theme::fs(11.5))
                                    .semibold()
                                    .fg(TEXT_PRIMARY);
                                ui.txt("Allow players to find this group via search")
                                    .sz(crate::theme::fs(10.0))
                                    .fg(TEXT_MUTED);
                            })
                            .gap(1.5)
                            .grow(1.0);

                            if gd.can_edit {
                                if ui
                                    .btn_with("mm_gd_set_listed", |ui| {
                                        ui.txt(if gd.setting_listed { "ON" } else { "OFF" })
                                            .sz(crate::theme::fs(10.5))
                                            .bold()
                                            .align(TextAlign::Center)
                                            .fg(if gd.setting_listed {
                                                EMERALD
                                            } else {
                                                TEXT_MUTED
                                            })
                                            .grow(1.0);
                                    })
                                    .w(54.0)
                                    .py(5.5)
                                    .bg(if gd.setting_listed {
                                        tint_alpha(EMERALD, 45)
                                    } else {
                                        tile_idle_bg()
                                    })
                                    .hover_bg(tile_idle_hover())
                                    .bd(
                                        1.0,
                                        if gd.setting_listed {
                                            tint_alpha(EMERALD, 185)
                                        } else {
                                            border_subtle()
                                        },
                                    )
                                    .bevel_rounded(5.5, 1.2)
                                    .clicked()
                                    && !gd.saving
                                {
                                    push_click(actions);
                                    actions.push(UiAction::SocialUpdateGroupSettings {
                                        group_id: gd.id.clone(),
                                        listed: !gd.setting_listed,
                                        member_publicity: gd.setting_member_publicity,
                                        event_publicity: gd.setting_event_publicity,
                                        privacy_join: gd.setting_privacy_join,
                                    });
                                }
                            } else {
                                ui.txt(if gd.setting_listed { "Yes" } else { "No" })
                                    .sz(crate::theme::fs(11.0))
                                    .semibold()
                                    .fg(TEXT_PRIMARY);
                            }
                        })
                        .align_center()
                        .p(9.0)
                        .gap(8.0)
                        .bg(card_inner_bg())
                        .bd(1.0, border_subtle())
                        .bevel_rounded(7.0, 1.3);

                        // Join Privacy
                        ui.col(|ui| {
                            ui.txt("Join Privacy")
                                .sz(crate::theme::fs(11.0))
                                .semibold()
                                .fg(TEXT_MUTED);
                            ui.row(|ui| {
                                for (idx, lbl) in [
                                    (0i32, "Public"),
                                    (1i32, "Request"),
                                    (2i32, "Invite"),
                                    (3i32, "Locked"),
                                ] {
                                    let active = gd.setting_privacy_join == idx;
                                    if ui
                                        .btn_with(&format!("mm_gd_sjoin_{idx}"), |ui| {
                                            ui.txt(lbl)
                                                .sz(crate::theme::fs(10.5))
                                                .semibold()
                                                .align(TextAlign::Center)
                                                .fg(if active { TEXT_PRIMARY } else { TEXT_MUTED })
                                                .grow(1.0);
                                        })
                                        .grow(1.0)
                                        .py(5.5)
                                        .bg(if active {
                                            tint_alpha(accent, 50)
                                        } else {
                                            tile_idle_bg()
                                        })
                                        .hover_bg(if gd.can_edit {
                                            tile_idle_hover()
                                        } else if active {
                                            tint_alpha(accent, 50)
                                        } else {
                                            tile_idle_bg()
                                        })
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
                                        && gd.can_edit
                                        && !gd.saving
                                        && !active
                                    {
                                        push_click(actions);
                                        actions.push(UiAction::SocialUpdateGroupSettings {
                                            group_id: gd.id.clone(),
                                            listed: gd.setting_listed,
                                            member_publicity: gd.setting_member_publicity,
                                            event_publicity: gd.setting_event_publicity,
                                            privacy_join: idx,
                                        });
                                    }
                                }
                            })
                            .gap(5.0);
                        })
                        .p(9.0)
                        .gap(6.0)
                        .bg(card_inner_bg())
                        .bd(1.0, border_subtle())
                        .bevel_rounded(7.0, 1.3);

                        // Member Publicity
                        ui.col(|ui| {
                            ui.txt("Member List Visibility")
                                .sz(crate::theme::fs(11.0))
                                .semibold()
                                .fg(TEXT_MUTED);
                            ui.row(|ui| {
                                for (idx, lbl) in [(0i32, "Public"), (1i32, "Members"), (2i32, "Hidden")] {
                                    let active = gd.setting_member_publicity == idx;
                                    if ui
                                        .btn_with(&format!("mm_gd_smem_{idx}"), |ui| {
                                            ui.txt(lbl)
                                                .sz(crate::theme::fs(10.5))
                                                .semibold()
                                                .align(TextAlign::Center)
                                                .fg(if active { TEXT_PRIMARY } else { TEXT_MUTED })
                                                .grow(1.0);
                                        })
                                        .grow(1.0)
                                        .py(5.5)
                                        .bg(if active {
                                            tint_alpha(accent, 50)
                                        } else {
                                            tile_idle_bg()
                                        })
                                        .hover_bg(if gd.can_edit {
                                            tile_idle_hover()
                                        } else if active {
                                            tint_alpha(accent, 50)
                                        } else {
                                            tile_idle_bg()
                                        })
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
                                        && gd.can_edit
                                        && !gd.saving
                                        && !active
                                    {
                                        push_click(actions);
                                        actions.push(UiAction::SocialUpdateGroupSettings {
                                            group_id: gd.id.clone(),
                                            listed: gd.setting_listed,
                                            member_publicity: idx,
                                            event_publicity: gd.setting_event_publicity,
                                            privacy_join: gd.setting_privacy_join,
                                        });
                                    }
                                }
                            })
                            .gap(5.0);
                        })
                        .p(9.0)
                        .gap(6.0)
                        .bg(card_inner_bg())
                        .bd(1.0, border_subtle())
                        .bevel_rounded(7.0, 1.3);

                        // Event Publicity
                        ui.col(|ui| {
                            ui.txt("Event Visibility")
                                .sz(crate::theme::fs(11.0))
                                .semibold()
                                .fg(TEXT_MUTED);
                            ui.row(|ui| {
                                for (idx, lbl) in [(0i32, "Public"), (1i32, "Members")] {
                                    let active = gd.setting_event_publicity == idx;
                                    if ui
                                        .btn_with(&format!("mm_gd_sevt_{idx}"), |ui| {
                                            ui.txt(lbl)
                                                .sz(crate::theme::fs(10.5))
                                                .semibold()
                                                .align(TextAlign::Center)
                                                .fg(if active { TEXT_PRIMARY } else { TEXT_MUTED })
                                                .grow(1.0);
                                        })
                                        .grow(1.0)
                                        .py(5.5)
                                        .bg(if active {
                                            tint_alpha(accent, 50)
                                        } else {
                                            tile_idle_bg()
                                        })
                                        .hover_bg(if gd.can_edit {
                                            tile_idle_hover()
                                        } else if active {
                                            tint_alpha(accent, 50)
                                        } else {
                                            tile_idle_bg()
                                        })
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
                                        && gd.can_edit
                                        && !gd.saving
                                        && !active
                                    {
                                        push_click(actions);
                                        actions.push(UiAction::SocialUpdateGroupSettings {
                                            group_id: gd.id.clone(),
                                            listed: gd.setting_listed,
                                            member_publicity: gd.setting_member_publicity,
                                            event_publicity: idx,
                                            privacy_join: gd.setting_privacy_join,
                                        });
                                    }
                                }
                            })
                            .gap(5.0);
                        })
                        .p(9.0)
                        .gap(6.0)
                        .bg(card_inner_bg())
                        .bd(1.0, border_subtle())
                        .bevel_rounded(7.0, 1.3);
                    })
                    .grow(1.0)
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
        }
    })
    .grow(1.0)
    .gap(9.0);
}
