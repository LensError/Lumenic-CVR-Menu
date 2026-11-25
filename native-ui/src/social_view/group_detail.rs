use lumenic::{Color, ImageFit, TextAlign, UiContext};

use crate::icons;
use crate::state::{AppState, GroupDetailSnapshot, GroupMemberItem, UiAction};
use crate::theme::{
    border_subtle, card_bg, card_inner_bg, push_click, tile_idle_bg, tile_idle_hover, tint_alpha,
    AMBER, EMERALD, RED, SHADOW_CARD, TEXT_DIM, TEXT_MUTED, TEXT_PRIMARY,
};

fn is_admin_role(role: &str) -> bool {
    role.to_ascii_lowercase().contains("admin")
}

fn is_moderator_role(role: &str) -> bool {
    role.to_ascii_lowercase().contains("mod")
}

fn group_role_color(role: &str, accent: Color) -> Color {
    let lc = role.to_ascii_lowercase();
    if lc.contains("owner") {
        EMERALD
    } else if lc.contains("admin") {
        AMBER
    } else if lc.contains("mod") {
        Color::rgb(56, 189, 248)
    } else if lc.contains("trial") {
        accent
    } else {
        TEXT_MUTED
    }
}

fn is_staff_role(role: &str, is_owner: bool) -> bool {
    is_owner || role.eq_ignore_ascii_case("Owner") || is_admin_role(role) || is_moderator_role(role)
}

fn join_privacy_label(code: i32) -> &'static str {
    match code {
        0 => "Public",
        1 => "Request to Join",
        2 => "Invite Only",
        3 => "Locked",
        _ => "Unknown",
    }
}

fn member_publicity_label(code: i32) -> &'static str {
    match code {
        0 => "Public",
        1 => "Members Only",
        2 => "Hidden",
        _ => "Unknown",
    }
}

fn event_publicity_label(code: i32) -> &'static str {
    match code {
        0 => "Public",
        1 => "Members Only",
        _ => "Unknown",
    }
}

fn open_user_profile(
    state: &mut AppState,
    group_id: &str,
    user_id: &str,
    actions: &mut Vec<UiAction>,
) {
    if user_id.is_empty() {
        return;
    }
    push_click(actions);
    crate::main_menu::push_nav_entry(
        state,
        crate::state::MainNavEntry::GroupDetail {
            tab: state.main_menu.active_tab.clone(),
            group_id: group_id.to_owned(),
        },
    );
    state.main_menu.user_detail_tab = 0;
    state.main_menu.group_edit_id.clear();
    actions.push(UiAction::SocialCloseGroupDetail);
    actions.push(UiAction::SocialLoadUserDetail {
        user_id: user_id.to_owned(),
    });
}

fn info_pill(ui: &mut UiContext, label: &str, value: &str, value_col: Color) {
    ui.row(|ui| {
        ui.txt(label).sz(crate::theme::fs(9.5)).fg(TEXT_MUTED);
        ui.txt(value)
            .sz(crate::theme::fs(9.5))
            .semibold()
            .fg(value_col);
    })
    .align_center()
    .gap(4.0)
    .px(7.5)
    .py(3.5)
    .bg(card_inner_bg())
    .bd(1.0, border_subtle())
    .bevel_rounded(5.0, 1.0);
}

fn render_member_avatar(
    ui: &mut UiContext,
    state: &AppState,
    image_key: &str,
    name: &str,
    role_col: Color,
    size: f32,
) {
    if !image_key.is_empty() {
        if let Some(tex) = state.main_textures.get(image_key) {
            ui.img(tex)
                .w(size)
                .h(size)
                .fit(ImageFit::Cover)
                .hexagon_pointy()
                .r(3.0)
                .bd(1.5, tint_alpha(role_col, 175));
            return;
        }
    }
    let init = name
        .chars()
        .next()
        .unwrap_or('M')
        .to_ascii_uppercase()
        .to_string();
    ui.col(|ui| {
        ui.txt(&init)
            .sz(crate::theme::fs((size * 0.36).clamp(10.0, 16.0)))
            .bold()
            .align(TextAlign::Center)
            .fg(role_col);
    })
    .center()
    .w(size)
    .h(size)
    .bg(tint_alpha(role_col, 28))
    .bd(1.5, tint_alpha(role_col, 155))
    .hexagon_pointy()
    .r(3.0);
}

/// Renders the member-first Group Detail view:
/// Tab 0 = Overview (About, Owner & Staff, Membership),
/// Tab 1 = Members (2-column directory, with optional Manage toggle for staff),
/// Tab 2 = Manage (Admin/Owner only: Identity, Privacy Settings, Danger Zone).
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
        state.main_menu.group_manage_members_mode = false;
        state.main_menu.group_name_input.set_text(&gd.name);
        state.main_menu.group_desc_input.set_text(&gd.description);
        state.main_menu.group_member_search.set_text("");
        state.main_menu.group_member_role_filter = String::from("all");
        state.main_menu.group_selected_members.clear();
        state.main_menu.group_mass_confirm = false;
    }

    let show_manage_tab = gd.can_edit || gd.is_owner;
    if !show_manage_tab && state.main_menu.group_detail_tab >= 2 {
        state.main_menu.group_detail_tab = 0;
    }
    if !(gd.can_manage_roles || gd.can_kick) {
        state.main_menu.group_manage_members_mode = false;
    }

    // Prune any selected members that are no longer eligible.
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

    let total_members = gd.member_count.max(gd.members.len() as i32);

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
                    .w(36.0)
                    .h(36.0)
                    .fit(ImageFit::Cover)
                    .bevel_rounded(6.0, 1.3)
                    .bd(1.0, tint_alpha(accent, 160));
            } else {
                icons::icon(ui, icons::PLAYERS, 18.0).fg(accent);
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

                    if gd.is_featured {
                        ui.txt("Featured")
                            .sz(crate::theme::fs(9.5))
                            .semibold()
                            .fg(EMERALD)
                            .px(6.0)
                            .py(1.5)
                            .bg(tint_alpha(EMERALD, 30))
                            .bd(1.0, tint_alpha(EMERALD, 150))
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
                    "{total_members} members  |  {}",
                    join_privacy_label(gd.setting_privacy_join)
                );
                ui.txt(&sub)
                    .sz(crate::theme::fs(10.5))
                    .fg(TEXT_MUTED)
                    .ellipsis();
            })
            .gap(1.5)
            .grow(1.0);

            if !gd.loading && !gd.id.is_empty() && gd.is_member && !gd.is_featured {
                if ui
                    .btn_with("mm_gd_feature", |ui| {
                        ui.row(|ui| {
                            icons::icon(ui, icons::STAR, 12.5).fg(accent);
                            ui.txt("Feature")
                                .sz(crate::theme::fs(10.5))
                                .semibold()
                                .fg(TEXT_PRIMARY);
                        })
                        .center()
                        .gap(4.0);
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
            // SUB-TAB SWITCHER
            // =================================================================
            ui.row(|ui| {
                let members_lbl = format!("Members ({total_members})");
                let mut tabs: Vec<(usize, &str, &str)> = vec![
                    (0, icons::INFO, "Overview"),
                    (1, icons::PLAYERS, members_lbl.as_str()),
                ];
                if show_manage_tab {
                    tabs.push((2, icons::SETTINGS, "Manage"));
                }

                for (tab_idx, ic, lbl) in tabs {
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

            match state.main_menu.group_detail_tab {
                0 => render_overview_tab(ui, state, gd, total_members, accent, actions),
                1 => render_members_tab(ui, state, gd, accent, actions),
                _ => render_manage_tab(ui, state, gd, accent, actions),
            }
        }
    })
    .grow(1.0)
    .gap(9.0);
}

// =============================================================================
// TAB 0: OVERVIEW (MEMBER-FIRST ABOUT, OWNER & STAFF, MEMBERSHIP)
// =============================================================================
fn render_overview_tab(
    ui: &mut UiContext,
    state: &mut AppState,
    gd: &GroupDetailSnapshot,
    total_members: i32,
    accent: Color,
    actions: &mut Vec<UiAction>,
) {
    ui.row(|ui| {
        // Left: Hero Emblem, Quick Info Pills, and Full About Description
        ui.col(|ui| {
            ui.row(|ui| {
                if let Some(tex) = state.main_textures.get(&gd.image_key) {
                    ui.img(tex)
                        .w(72.0)
                        .h(72.0)
                        .fit(ImageFit::Cover)
                        .bevel_rounded(10.0, 2.0)
                        .bd(1.5, tint_alpha(accent, 175));
                } else {
                    ui.col(|ui| {
                        icons::icon(ui, icons::PLAYERS, 30.0).fg(accent);
                    })
                    .center()
                    .w(72.0)
                    .h(72.0)
                    .bg(tint_alpha(accent, 28))
                    .bd(1.5, tint_alpha(accent, 150))
                    .bevel_rounded(10.0, 2.0);
                }

                ui.col(|ui| {
                    ui.row(|ui| {
                        ui.txt(&gd.name)
                            .sz(crate::theme::fs(16.0))
                            .bold()
                            .fg(TEXT_PRIMARY)
                            .ellipsis();
                        if !gd.tag.is_empty() {
                            ui.txt(&format!("[{}]", gd.tag))
                                .sz(crate::theme::fs(10.5))
                                .semibold()
                                .fg(accent)
                                .px(7.0)
                                .py(2.0)
                                .bg(tint_alpha(accent, 32))
                                .bd(1.0, tint_alpha(accent, 140))
                                .bevel_rounded(4.5, 1.0);
                        }
                    })
                    .align_center()
                    .gap(6.0);

                    ui.row(|ui| {
                        info_pill(
                            ui,
                            "Join:",
                            join_privacy_label(gd.setting_privacy_join),
                            if gd.setting_privacy_join == 0 {
                                EMERALD
                            } else {
                                TEXT_PRIMARY
                            },
                        );
                        info_pill(
                            ui,
                            "Search:",
                            if gd.setting_listed { "Listed" } else { "Unlisted" },
                            if gd.setting_listed { EMERALD } else { TEXT_MUTED },
                        );
                    })
                    .align_center()
                    .gap(6.0);

                    ui.row(|ui| {
                        info_pill(
                            ui,
                            "Members:",
                            member_publicity_label(gd.setting_member_publicity),
                            TEXT_PRIMARY,
                        );
                        info_pill(
                            ui,
                            "Events:",
                            event_publicity_label(gd.setting_event_publicity),
                            TEXT_PRIMARY,
                        );
                    })
                    .align_center()
                    .gap(6.0);
                })
                .gap(5.0)
                .grow(1.0);
            })
            .align_center()
            .gap(12.0);

            ui.txt("About This Group")
                .sz(crate::theme::fs(12.0))
                .bold()
                .fg(TEXT_MUTED)
                .pt(2.0);

            ui.scroll("mm_gd_overview_desc", |ui| {
                let desc = if gd.description.trim().is_empty() {
                    "No group description provided."
                } else {
                    gd.description.trim()
                };
                ui.txt(desc)
                    .sz(crate::theme::fs(12.0))
                    .fg(if gd.description.trim().is_empty() {
                        TEXT_MUTED
                    } else {
                        TEXT_PRIMARY
                    })
                    .wrap();
            })
            .grow(1.0)
            .p(11.0)
            .bg(card_inner_bg())
            .bd(1.0, border_subtle())
            .bevel_rounded(8.0, 1.5)
            .no_scrollbar();
        })
        .grow(1.45)
        .h_frac(1.0)
        .p(13.0)
        .gap(9.0)
        .bg(card_bg())
        .bd(1.0, border_subtle())
        .bevel_rounded(10.5, 2.0)
        .shadow(14.0, SHADOW_CARD);

        // Right: Owner & Leadership + Your Membership Card
        ui.col(|ui| {
            // Owner & Staff Card
            ui.col(|ui| {
                ui.row(|ui| {
                    icons::icon(ui, icons::SHIELD, 14.0).fg(accent);
                    ui.txt("Leadership & Staff")
                        .sz(crate::theme::fs(12.5))
                        .bold()
                        .fg(TEXT_PRIMARY)
                        .grow(1.0);
                    if gd.staff_scanning {
                        ui.txt("Scanning...")
                            .sz(crate::theme::fs(9.5))
                            .fg(TEXT_MUTED);
                    }
                })
                .align_center()
                .gap(6.0);

                // Clickable Owner row
                if !gd.owner_name.is_empty() || !gd.owner_id.is_empty() {
                    let owner_img = gd
                        .staff
                        .iter()
                        .chain(gd.members.iter())
                        .find(|m| !gd.owner_id.is_empty() && m.id == gd.owner_id && !m.image_key.is_empty())
                        .map(|m| m.image_key.as_str())
                        .unwrap_or("");
                    let owner_disp = if gd.owner_name.is_empty() {
                        "Group Owner"
                    } else {
                        &gd.owner_name
                    };
                    if ui
                        .btn_with("mm_gd_ov_owner", |ui| {
                            ui.row(|ui| {
                                render_member_avatar(ui, state, owner_img, owner_disp, EMERALD, 34.0);
                                ui.col(|ui| {
                                    ui.txt(owner_disp)
                                        .sz(crate::theme::fs(12.0))
                                        .semibold()
                                        .fg(TEXT_PRIMARY)
                                        .ellipsis();
                                    ui.txt("Owner")
                                        .sz(crate::theme::fs(9.5))
                                        .semibold()
                                        .fg(EMERALD);
                                })
                                .gap(1.0)
                                .grow(1.0);
                                icons::icon(ui, icons::CHEVRON_RIGHT, 14.0).fg(TEXT_DIM);
                            })
                            .align_center()
                            .gap(9.0)
                            .grow(1.0);
                        })
                        .w_frac(1.0)
                        .px(8.0)
                        .py(6.0)
                        .bg(card_inner_bg())
                        .hover_bg(tile_idle_hover())
                        .bd(1.0, tint_alpha(EMERALD, 115))
                        .bevel_rounded(7.0, 1.3)
                        .clicked()
                    {
                        open_user_profile(state, &gd.id, &gd.owner_id, actions);
                    }
                }

                // Other staff: Admins first, then Moderators (merged from background staff scan + loaded members)
                let mut seen_staff_ids = std::collections::HashSet::new();
                let mut staff: Vec<&GroupMemberItem> = Vec::new();
                for m in gd.staff.iter().chain(gd.members.iter()) {
                    if m.id.is_empty() || m.id == gd.owner_id {
                        continue;
                    }
                    if (is_admin_role(&m.role) || is_moderator_role(&m.role))
                        && seen_staff_ids.insert(m.id.as_str())
                    {
                        staff.push(m);
                    }
                }
                staff.sort_by(|a, b| {
                    let rank = |r: &str| if is_admin_role(r) { 0u8 } else { 1u8 };
                    rank(&a.role)
                        .cmp(&rank(&b.role))
                        .then_with(|| a.name.to_ascii_lowercase().cmp(&b.name.to_ascii_lowercase()))
                });

                if !staff.is_empty() {
                    let admin_count = staff.iter().filter(|m| is_admin_role(&m.role)).count();
                    let mod_count = staff.iter().filter(|m| is_moderator_role(&m.role)).count();
                    let staff_hdr = match (admin_count > 0, mod_count > 0) {
                        (true, true) => format!("Admins ({admin_count}) & Moderators ({mod_count})"),
                        (true, false) => format!("Admins ({admin_count})"),
                        (false, true) => format!("Moderators ({mod_count})"),
                        (false, false) => format!("Staff ({})", staff.len()),
                    };
                    ui.txt(&staff_hdr)
                        .sz(crate::theme::fs(10.0))
                        .semibold()
                        .fg(TEXT_MUTED);

                    ui.scroll("mm_gd_ov_staff_scroll", |ui| {
                        for m in &staff {
                            let rcol = group_role_color(&m.role, accent);
                            if ui
                                .btn_with(&format!("mm_gd_ov_staff_{}", m.id), |ui| {
                                    ui.row(|ui| {
                                        render_member_avatar(
                                            ui,
                                            state,
                                            &m.image_key,
                                            &m.name,
                                            rcol,
                                            28.0,
                                        );
                                        ui.txt(&m.name)
                                            .sz(crate::theme::fs(11.5))
                                            .semibold()
                                            .fg(TEXT_PRIMARY)
                                            .ellipsis()
                                            .grow(1.0);
                                        ui.txt(&m.role)
                                            .sz(crate::theme::fs(9.0))
                                            .semibold()
                                            .fg(rcol)
                                            .px(6.0)
                                            .py(1.5)
                                            .bg(tint_alpha(rcol, 24))
                                            .bd(1.0, tint_alpha(rcol, 115))
                                            .bevel_rounded(4.0, 1.0);
                                    })
                                    .align_center()
                                    .gap(8.0)
                                    .grow(1.0);
                                })
                                .w_frac(1.0)
                                .px(7.0)
                                .py(4.5)
                                .bg(card_inner_bg())
                                .hover_bg(tile_idle_hover())
                                .bd(1.0, border_subtle())
                                .bevel_rounded(6.5, 1.2)
                                .clicked()
                            {
                                open_user_profile(state, &gd.id, &m.id, actions);
                            }
                        }
                    })
                    .grow(1.0)
                    .gap(4.5)
                    .no_scrollbar();
                } else {
                    ui.txt("Browse the Members tab to meet everyone in this group.")
                        .sz(crate::theme::fs(10.5))
                        .fg(TEXT_MUTED)
                        .wrap();
                }
            })
            .grow(1.0)
            .p(12.0)
            .gap(8.0)
            .bg(card_bg())
            .bd(1.0, border_subtle())
            .bevel_rounded(10.5, 2.0)
            .shadow(14.0, SHADOW_CARD);

            // Your Membership & Quick Actions Card
            ui.col(|ui| {
                ui.row(|ui| {
                    ui.txt("Your Membership")
                        .sz(crate::theme::fs(12.0))
                        .bold()
                        .fg(TEXT_PRIMARY)
                        .grow(1.0);

                    let role_disp = if !gd.is_member {
                        "Visitor"
                    } else if gd.local_role.is_empty() {
                        "Member"
                    } else {
                        gd.local_role.as_str()
                    };
                    let rcol = if gd.is_member {
                        group_role_color(role_disp, accent)
                    } else {
                        TEXT_MUTED
                    };
                    ui.txt(role_disp)
                        .sz(crate::theme::fs(9.5))
                        .semibold()
                        .fg(rcol)
                        .px(7.0)
                        .py(2.0)
                        .bg(tint_alpha(rcol, 26))
                        .bd(1.0, tint_alpha(rcol, 130))
                        .bevel_rounded(4.5, 1.0);
                })
                .align_center()
                .gap(6.0);

                if ui
                    .btn_with("mm_gd_ov_browse_members", |ui| {
                        ui.row(|ui| {
                            icons::icon(ui, icons::PLAYERS, 13.5).fg(accent);
                            ui.txt(&format!("Browse All {total_members} Members"))
                                .sz(crate::theme::fs(11.0))
                                .semibold()
                                .fg(TEXT_PRIMARY);
                        })
                        .center()
                        .gap(6.0)
                        .grow(1.0);
                    })
                    .w_frac(1.0)
                    .py(7.0)
                    .bg(tile_idle_bg())
                    .hover_bg(tile_idle_hover())
                    .bd(1.0, border_subtle())
                    .bevel_rounded(6.0, 1.3)
                    .clicked()
                {
                    push_click(actions);
                    state.main_menu.group_detail_tab = 1;
                }

                if gd.is_member {
                    ui.row(|ui| {
                        if gd.is_featured {
                            ui.row(|ui| {
                                icons::icon(ui, icons::CHECK, 13.0).fg(EMERALD);
                                ui.txt("Featured on Profile")
                                    .sz(crate::theme::fs(10.5))
                                    .semibold()
                                    .fg(EMERALD);
                            })
                            .center()
                            .gap(5.0)
                            .grow(1.0)
                            .py(6.5)
                            .bg(tint_alpha(EMERALD, 24))
                            .bd(1.0, tint_alpha(EMERALD, 135))
                            .bevel_rounded(6.0, 1.3);
                        } else if ui
                            .btn_with("mm_gd_ov_feature", |ui| {
                                ui.row(|ui| {
                                    icons::icon(ui, icons::STAR, 13.0).fg(accent);
                                    ui.txt("Feature Group")
                                        .sz(crate::theme::fs(10.5))
                                        .semibold()
                                        .fg(TEXT_PRIMARY);
                                })
                                .center()
                                .gap(5.0)
                                .grow(1.0);
                            })
                            .grow(1.0)
                            .py(6.5)
                            .bg(tint_alpha(accent, 44))
                            .hover_bg(tint_alpha(accent, 78))
                            .bd(1.0, tint_alpha(accent, 180))
                            .bevel_rounded(6.0, 1.3)
                            .clicked()
                        {
                            push_click(actions);
                            actions.push(UiAction::SocialSetFeaturedGroup {
                                group_id: gd.id.clone(),
                            });
                        }

                        if !gd.is_owner {
                            let leave_armed =
                                state.main_menu.safety_confirm_action == "leave_group";
                            if ui
                                .btn_with("mm_gd_leave", |ui| {
                                    ui.txt(if leave_armed {
                                        "Confirm Leave?"
                                    } else {
                                        "Leave"
                                    })
                                    .sz(crate::theme::fs(10.5))
                                    .semibold()
                                    .align(TextAlign::Center)
                                    .fg(if leave_armed { TEXT_PRIMARY } else { AMBER });
                                })
                                .px(12.0)
                                .py(6.5)
                                .bg(if leave_armed {
                                    tint_alpha(AMBER, 72)
                                } else {
                                    tile_idle_bg()
                                })
                                .hover_bg(tint_alpha(AMBER, 95))
                                .bd(1.0, tint_alpha(AMBER, 175))
                                .bevel_rounded(6.0, 1.3)
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
                    })
                    .align_center()
                    .gap(6.0);
                }
            })
            .p(12.0)
            .gap(8.0)
            .bg(card_bg())
            .bd(1.0, border_subtle())
            .bevel_rounded(10.5, 2.0)
            .shadow(14.0, SHADOW_CARD);
        })
        .grow(1.0)
        .h_frac(1.0)
        .gap(9.0);
    })
    .grow(1.0)
    .gap(10.0);
}

// =============================================================================
// TAB 1: MEMBERS (CLEAN 2-COL DIRECTORY + OPTIONAL STAFF MANAGE MODE)
// =============================================================================
fn render_members_tab(
    ui: &mut UiContext,
    state: &mut AppState,
    gd: &GroupDetailSnapshot,
    accent: Color,
    actions: &mut Vec<UiAction>,
) {
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
    let filtered: Vec<&GroupMemberItem> = gd
        .members
        .iter()
        .filter(|m| {
            let is_owner_row = !m.id.is_empty() && m.id == gd.owner_id;
            let match_role = match role_f {
                "all" => true,
                "staff" => is_staff_role(&m.role, is_owner_row),
                other => m.role.eq_ignore_ascii_case(other),
            };
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

    let can_moderate = gd.can_manage_roles || gd.can_kick;
    let manage_mode = can_moderate && state.main_menu.group_manage_members_mode;

    ui.col(|ui| {
        // Search + Filter + Optional Staff Manage Toggle
        ui.row(|ui| {
            ui.row(|ui| {
                icons::icon(ui, icons::SEARCH, 13.0).fg(TEXT_MUTED);
                let mut s_inp = ui
                    .input("mm_gd_member_search", &mut state.main_menu.group_member_search)
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
            let mut filters: Vec<(&str, &str)> = vec![
                ("all", "All"),
                ("staff", "Staff"),
                ("Member", "Members"),
            ];
            if !trial_ids.is_empty() || gd.can_manage_roles {
                filters.push(("Trial", trial_lbl.as_str()));
            }
            for (fkey, lbl) in filters {
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

            if can_moderate {
                if ui
                    .btn_with("mm_gd_toggle_manage", |ui| {
                        ui.row(|ui| {
                            icons::icon(ui, icons::SHIELD, 12.5)
                                .fg(if manage_mode { accent } else { TEXT_MUTED });
                            ui.txt(if manage_mode { "Done" } else { "Manage" })
                                .sz(crate::theme::fs(10.5))
                                .semibold()
                                .fg(if manage_mode {
                                    TEXT_PRIMARY
                                } else {
                                    TEXT_MUTED
                                });
                        })
                        .center()
                        .gap(4.0);
                    })
                    .px(10.0)
                    .py(5.0)
                    .bg(if manage_mode {
                        tint_alpha(accent, 52)
                    } else {
                        tile_idle_bg()
                    })
                    .hover_bg(tile_idle_hover())
                    .bd(
                        1.0,
                        if manage_mode {
                            tint_alpha(accent, 195)
                        } else {
                            border_subtle()
                        },
                    )
                    .bevel_rounded(5.5, 1.3)
                    .clicked()
                {
                    push_click(actions);
                    state.main_menu.group_manage_members_mode = !manage_mode;
                    if !state.main_menu.group_manage_members_mode {
                        state.main_menu.group_selected_members.clear();
                        state.main_menu.group_mass_confirm = false;
                    }
                }
            }
        })
        .align_center()
        .gap(6.0);

        // Mass Role Action Bar (only shown when staff toggled Manage mode ON)
        if manage_mode && gd.can_manage_roles {
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
                            actions.push(UiAction::SocialMassSetGroupMembersToMember {
                                group_id: gd.id.clone(),
                                user_ids: ids,
                            });
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

        // Member Roster
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
                if !manage_mode {
                    // Clean 2-column member directory cards
                    for pair in filtered.chunks(2) {
                        ui.row(|ui| {
                            for m in pair {
                                let is_owner_row = !m.id.is_empty() && m.id == gd.owner_id;
                                let is_self_row = !m.id.is_empty() && m.id == gd.local_user_id;
                                let role_lbl = if is_owner_row {
                                    "Owner"
                                } else if m.role.is_empty() {
                                    "Member"
                                } else {
                                    m.role.as_str()
                                };
                                let role_col = group_role_color(role_lbl, accent);

                                if ui
                                    .btn_with(&format!("mm_gd_m_{}", m.id), |ui| {
                                        ui.row(|ui| {
                                            render_member_avatar(
                                                ui,
                                                state,
                                                &m.image_key,
                                                &m.name,
                                                role_col,
                                                36.0,
                                            );
                                            ui.col(|ui| {
                                                ui.row(|ui| {
                                                    ui.txt(&m.name)
                                                        .sz(crate::theme::fs(12.0))
                                                        .semibold()
                                                        .fg(TEXT_PRIMARY)
                                                        .ellipsis();
                                                    if is_self_row {
                                                        ui.txt("You")
                                                            .sz(crate::theme::fs(9.0))
                                                            .semibold()
                                                            .fg(accent);
                                                    }
                                                })
                                                .align_center()
                                                .gap(5.0);

                                                ui.txt(role_lbl)
                                                    .sz(crate::theme::fs(9.5))
                                                    .semibold()
                                                    .fg(role_col);
                                            })
                                            .gap(1.5)
                                            .grow(1.0);

                                            icons::icon(ui, icons::CHEVRON_RIGHT, 14.0)
                                                .fg(TEXT_DIM);
                                        })
                                        .align_center()
                                        .gap(9.0)
                                        .grow(1.0);
                                    })
                                    .grow(1.0)
                                    .px(9.0)
                                    .py(7.0)
                                    .bg(card_inner_bg())
                                    .hover_bg(tile_idle_hover())
                                    .bd(
                                        1.0,
                                        if is_owner_row {
                                            tint_alpha(EMERALD, 110)
                                        } else {
                                            border_subtle()
                                        },
                                    )
                                    .bevel_rounded(7.5, 1.3)
                                    .clicked()
                                {
                                    open_user_profile(state, &gd.id, &m.id, actions);
                                }
                            }
                            if pair.len() == 1 {
                                ui.col(|_| {}).grow(1.0);
                            }
                        })
                        .gap(6.0);
                    }
                } else {
                    // Staff Manage Mode: single-column rows with checkbox, role dropdown, kick
                    for m in &filtered {
                        let is_owner_row = !m.id.is_empty() && m.id == gd.owner_id;
                        let is_self_row = !m.id.is_empty() && m.id == gd.local_user_id;
                        let can_edit_member = gd.can_manage_roles
                            && !m.id.is_empty()
                            && !is_owner_row
                            && !is_self_row;
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

                            if ui
                                .btn_with(&format!("mm_gd_m_{}", m.id), |ui| {
                                    ui.row(|ui| {
                                        render_member_avatar(
                                            ui,
                                            state,
                                            &m.image_key,
                                            &m.name,
                                            role_col,
                                            34.0,
                                        );
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
                                open_user_profile(state, &gd.id, &m.id, actions);
                            }

                            if can_edit_member {
                                if ui
                                    .btn_with(&format!("mm_gd_role_{}", m.id), |ui| {
                                        ui.row(|ui| {
                                            ui.txt(role_lbl)
                                                .sz(crate::theme::fs(10.0))
                                                .semibold()
                                                .fg(TEXT_PRIMARY)
                                                .grow(1.0);
                                            icons::icon(ui, icons::CHEVRON_DOWN, 12.5).fg(accent);
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
            .gap(6.0)
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
}

// =============================================================================
// TAB 2: MANAGE (ADMIN / OWNER ONLY - IDENTITY, PRIVACY, DANGER ZONE)
// =============================================================================
fn render_manage_tab(
    ui: &mut UiContext,
    state: &mut AppState,
    gd: &GroupDetailSnapshot,
    accent: Color,
    actions: &mut Vec<UiAction>,
) {
    ui.row(|ui| {
        // Left: Group Identity + Owner Danger Zone
        ui.col(|ui| {
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

            ui.col(|_| {}).grow(1.0);

            if gd.is_owner {
                ui.row(|ui| {
                    ui.col(|ui| {
                        ui.txt("Danger Zone")
                            .sz(crate::theme::fs(11.0))
                            .bold()
                            .fg(RED);
                        ui.txt("Permanently deletes this group for all members.")
                            .sz(crate::theme::fs(9.5))
                            .fg(TEXT_MUTED);
                    })
                    .gap(1.5)
                    .grow(1.0);

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
                        .px(11.0)
                        .py(6.0)
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
                })
                .align_center()
                .p(9.0)
                .gap(8.0)
                .bg(tint_alpha(RED, 16))
                .bd(1.0, tint_alpha(RED, 110))
                .bevel_rounded(7.0, 1.3);
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

        // Right: Privacy & Visibility Controls
        ui.col(|ui| {
            ui.txt("Privacy & Visibility")
                .sz(crate::theme::fs(13.0))
                .bold()
                .fg(TEXT_PRIMARY);

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
