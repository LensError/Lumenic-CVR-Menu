use lumenic::{Color, Rect, TextAlign, UiContext};

use crate::icons;
use crate::state::{AppState, AvatarSettingEntry, UiAction};
use crate::theme::{
    push_click, tint_alpha, AMBER, border_subtle, card_bg, card_inner_bg, EMERALD, RED,
    SHADOW_CARD, TEXT_DIM, TEXT_MUTED, TEXT_PRIMARY, tile_idle_bg, tile_idle_hover,
};

/// Shared GPU-composited 3D Avatar Turntable stage used across Main Menu (AAS, User Details,
/// Avatar Details) and Quick Menu (Home, Player Details).
#[allow(clippy::too_many_arguments)]
pub fn render_avatar_turntable_stage(
    ui: &mut UiContext,
    state: &mut AppState,
    surface_id: u32,
    target: &str,
    drag_id: &str,
    status_text: &str,
    accent: Color,
    actions: &mut Vec<UiAction>,
) {
    let [vw, vh] = ui.viewport_size();
    let has_status = !status_text.trim().is_empty();

    let stage_resp = ui
        .col(|ui| {
            ui.col(|ui| {
                if has_status {
                    icons::icon(ui, icons::AVATAR, 30.0)
                        .fg(Color::rgba(148, 163, 184, 90));
                    ui.txt(status_text)
                        .sz(10.5)
                        .semibold()
                        .align(TextAlign::Center)
                        .fg(TEXT_MUTED)
                        .wrap();
                } else {
                    ui.txt("").grow(1.0);
                }
            })
            .center()
            .grow(1.0)
            .gap(6.0)
            .px(8.0);

            if !has_status {
                ui.txt("Drag horizontally to spin")
                    .sz(9.5)
                    .align(TextAlign::Center)
                    .fg(TEXT_DIM)
                    .w_frac(1.0);
            }
        })
        .grow(1.0)
        .w_frac(1.0)
        .p(8.0)
        .bg_radial(Color::rgb(26, 36, 54), Color::rgb(10, 14, 22))
        .bd(1.0, tint_alpha(accent, 95))
        .bevel_rounded(8.5, 1.5);

    if has_status {
        return;
    }

    if surface_id == 0 {
        state.main_menu.aas_preview_rendered = true;
    } else {
        state.quick.avatar_preview_rendered = true;
    }

    if let Some(r) = stage_resp.prev_rect() {
        let stage_r = Rect::new(
            r.x + 2.0,
            r.y + 2.0,
            (r.w - 4.0).max(1.0),
            (r.h - 22.0).max(1.0),
        );

        let norm = [
            (stage_r.x / vw.max(1.0)).clamp(0.0, 1.0),
            (stage_r.y / vh.max(1.0)).clamp(0.0, 1.0),
            (stage_r.w / vw.max(1.0)).clamp(0.0, 1.0),
            (stage_r.h / vh.max(1.0)).clamp(0.0, 1.0),
        ];

        let (rect_slot, last_sync, target_slot, drag_slot) = if surface_id == 0 {
            (
                &mut state.main_menu.aas_preview_rect,
                &mut state.main_menu.aas_last_preview_sync,
                &mut state.main_menu.aas_preview_target,
                &mut state.main_menu.aas_preview_dragging,
            )
        } else {
            (
                &mut state.quick.avatar_preview_rect,
                &mut state.quick.last_avatar_preview_sync,
                &mut state.quick.avatar_preview_target,
                &mut state.quick.avatar_preview_dragging,
            )
        };

        *rect_slot = Some([stage_r.x, stage_r.y, stage_r.w, stage_r.h]);

        let rect_changed = last_sync
            .map(|prev| {
                prev.iter()
                    .zip(norm.iter())
                    .any(|(a, b)| (a - b).abs() > 0.002)
            })
            .unwrap_or(true);
        let target_changed = target_slot.as_str() != target;

        if rect_changed || target_changed {
            *last_sync = Some(norm);
            *target_slot = target.to_owned();
            actions.push(UiAction::SyncAvatarPreview {
                surface_id,
                active: true,
                target: target.to_owned(),
                x: norm[0],
                y: norm[1],
                w: norm[2],
                h: norm[3],
            });
        }

        let g = ui.interact(ui.id(drag_id), stage_r);
        if g.dragging {
            *drag_slot = true;
            let delta_norm = g.delta[0] / stage_r.w.max(1.0);
            actions.push(UiAction::AvatarPreviewRotate {
                delta: delta_norm,
                is_dragging: true,
            });
        } else if *drag_slot {
            *drag_slot = false;
            actions.push(UiAction::AvatarPreviewRotate {
                delta: 0.0,
                is_dragging: false,
            });
        }
    }
}

/// Renders the beta-aligned 3-column Advanced Avatar Settings page:
/// Left: Live 3D Avatar Preview turntable + Auto Apply / Apply / Discard controls.
/// Middle: Profile list with Default, active/staged indicators, Edit mode (reorder/save/delete), and Add New Profile.
/// Right: Scrollable parameter controls reflecting staged values when Auto Apply is off.
pub fn render_avatar_settings_page(
    ui: &mut UiContext,
    state: &mut AppState,
    accent: Color,
    actions: &mut Vec<UiAction>,
) {
    let entries: Vec<AvatarSettingEntry> = state.host.avatar_settings.entries.clone();
    let profiles: Vec<String> = state.host.avatar_settings.profiles.clone();
    let auto_apply = state.host.avatar_settings.auto_apply;
    let has_staged = state.host.avatar_settings.has_staged;
    let current_profile = state.host.avatar_settings.current_profile.clone();
    let staged_profile = state.host.avatar_settings.staged_profile.clone();
    let active_profile_display = if !staged_profile.is_empty() {
        staged_profile.as_str()
    } else if current_profile.is_empty() {
        "default"
    } else {
        current_profile.as_str()
    };
    let is_vr = state.host.is_vr;

    ui.row(|ui| {
        // =====================================================================
        // COLUMN 1: LIVE 3D AVATAR PREVIEW TURNTABLE + STAGING ACTIONS
        // =====================================================================
        ui.col(|ui| {
            // Top bar: Back to Home + Preview title
            ui.row(|ui| {
                if ui
                    .btn_with("mm_aas_back", |ui| {
                        ui.row(|ui| {
                            icons::icon(ui, icons::BACK, 14.0).fg(TEXT_PRIMARY);
                            ui.txt("Back").sz(11.0).semibold().fg(TEXT_PRIMARY);
                        })
                        .center()
                        .gap(3.0);
                    })
                    .px(9.0)
                    .py(5.0)
                    .bg(tile_idle_bg())
                    .hover_bg(tile_idle_hover())
                    .bd(1.0, border_subtle())
                    .bevel_rounded(5.5, 1.3)
                    .clicked()
                {
                    push_click(actions);
                    crate::main_menu::nav_go_back(state, "home", actions);
                }

                ui.txt("Avatar Preview")
                    .sz(12.0)
                    .bold()
                    .fg(TEXT_PRIMARY)
                    .align(TextAlign::Right)
                    .grow(1.0);
            })
            .align_center()
            .gap(6.0);

            render_avatar_turntable_stage(
                ui,
                state,
                0,
                "local",
                "mm_aas_preview_drag",
                "",
                accent,
                actions,
            );

            // Apply / Discard staged changes controls
            ui.row(|ui| {
                let apply_bg = if has_staged {
                    tint_alpha(EMERALD, 64)
                } else {
                    tile_idle_bg()
                };
                let apply_bd = if has_staged {
                    tint_alpha(EMERALD, 210)
                } else {
                    border_subtle()
                };
                if ui
                    .btn_with("mm_aas_apply_staged", |ui| {
                        ui.row(|ui| {
                            icons::icon(ui, icons::CHECK, 14.0)
                                .fg(if has_staged { EMERALD } else { TEXT_DIM });
                            ui.txt(if has_staged {
                                "Apply Changes"
                            } else {
                                "Up to Date"
                            })
                            .sz(11.0)
                            .semibold()
                            .fg(if has_staged { TEXT_PRIMARY } else { TEXT_MUTED });
                        })
                        .center()
                        .gap(5.0)
                        .grow(1.0);
                    })
                    .grow(1.0)
                    .py(7.0)
                    .bg(apply_bg)
                    .hover_bg(if has_staged {
                        tint_alpha(EMERALD, 90)
                    } else {
                        tile_idle_hover()
                    })
                    .bd(1.0, apply_bd)
                    .bevel_rounded(6.0, 1.3)
                    .clicked()
                    && has_staged
                {
                    push_click(actions);
                    actions.push(UiAction::AvatarApplyStaged);
                }

                if has_staged {
                    if ui
                        .btn_with("mm_aas_discard_staged", |ui| {
                            icons::icon(ui, icons::CLOSE, 14.0).fg(AMBER);
                        })
                        .px(9.0)
                        .py(7.0)
                        .bg(tint_alpha(AMBER, 42))
                        .hover_bg(tint_alpha(AMBER, 70))
                        .bd(1.0, tint_alpha(AMBER, 175))
                        .bevel_rounded(6.0, 1.3)
                        .clicked()
                    {
                        push_click(actions);
                        actions.push(UiAction::AvatarDiscardStaged);
                    }
                }
            })
            .gap(6.0);

            // Auto Apply Changes toggle row
            if ui
                .btn_with("mm_aas_auto_apply", |ui| {
                    ui.row(|ui| {
                        ui.txt("Auto Apply Changes")
                            .sz(10.5)
                            .semibold()
                            .fg(TEXT_PRIMARY)
                            .grow(1.0);
                        ui.txt(if auto_apply { "ON" } else { "OFF" })
                            .sz(10.0)
                            .bold()
                            .fg(if auto_apply { EMERALD } else { TEXT_MUTED })
                            .bg(if auto_apply {
                                tint_alpha(EMERALD, 42)
                            } else {
                                card_inner_bg()
                            })
                            .bd(
                                1.0,
                                if auto_apply {
                                    tint_alpha(EMERALD, 180)
                                } else {
                                    border_subtle()
                                },
                            )
                            .px(7.0)
                            .py(2.0)
                            .bevel_rounded(4.5, 1.2);
                    })
                    .align_center()
                    .gap(6.0)
                    .grow(1.0);
                })
                .px(9.0)
                .py(6.0)
                .bg(card_inner_bg())
                .hover_bg(tile_idle_hover())
                .bd(1.0, border_subtle())
                .bevel_rounded(6.0, 1.3)
                .clicked()
            {
                push_click(actions);
                actions.push(UiAction::AvatarSetAutoApply {
                    enabled: !auto_apply,
                });
            }
        })
        .w(258.0)
        .p(10.0)
        .gap(8.0)
        .bg(card_bg())
        .bd(1.0, border_subtle())
        .bevel_rounded(10.5, 2.0)
        .shadow(14.0, SHADOW_CARD);

        // =====================================================================
        // COLUMN 2: PROFILES PANEL (DEFAULT + SAVED + EDIT/REORDER + ADD NEW)
        // =====================================================================
        let edit_mode = state.main_menu.aas_profile_edit_mode;
        ui.col(|ui| {
            // Header: Profiles + Edit Mode Toggle
            ui.row(|ui| {
                ui.txt("Profiles")
                    .sz(13.0)
                    .bold()
                    .fg(TEXT_PRIMARY)
                    .grow(1.0);

                if ui
                    .btn_with("mm_aas_prof_edit", |ui| {
                        ui.row(|ui| {
                            icons::icon(ui, icons::EDIT, 13.0)
                                .fg(if edit_mode { accent } else { TEXT_MUTED });
                            ui.txt(if edit_mode { "Done" } else { "Edit" })
                                .sz(10.0)
                                .semibold()
                                .fg(if edit_mode { TEXT_PRIMARY } else { TEXT_MUTED });
                        })
                        .center()
                        .gap(4.0);
                    })
                    .px(8.0)
                    .py(4.0)
                    .bg(if edit_mode {
                        tint_alpha(accent, 48)
                    } else {
                        tile_idle_bg()
                    })
                    .hover_bg(tile_idle_hover())
                    .bd(
                        1.0,
                        if edit_mode {
                            tint_alpha(accent, 195)
                        } else {
                            border_subtle()
                        },
                    )
                    .bevel_rounded(5.0, 1.2)
                    .clicked()
                {
                    push_click(actions);
                    state.main_menu.aas_profile_edit_mode = !edit_mode;
                }
            })
            .align_center()
            .gap(6.0);

            // Scrollable profile list
            ui.scroll("mm_aas_profiles_scroll", |ui| {
                // Default profile row
                let default_selected = active_profile_display.eq_ignore_ascii_case("default");
                if ui
                    .btn_with("mm_aas_prof_default", |ui| {
                        ui.row(|ui| {
                            ui.txt("Default")
                                .sz(11.0)
                                .semibold()
                                .fg(if default_selected {
                                    TEXT_PRIMARY
                                } else {
                                    TEXT_MUTED
                                })
                                .grow(1.0);
                            if default_selected {
                                ui.txt(if !staged_profile.is_empty() {
                                    "STAGED"
                                } else {
                                    "ACTIVE"
                                })
                                .sz(8.5)
                                .bold()
                                .fg(if !staged_profile.is_empty() {
                                    AMBER
                                } else {
                                    accent
                                });
                            }
                        })
                        .align_center()
                        .gap(4.0)
                        .grow(1.0);
                    })
                    .px(9.0)
                    .py(6.5)
                    .bg(if default_selected {
                        tint_alpha(accent, 42)
                    } else {
                        card_inner_bg()
                    })
                    .hover_bg(tile_idle_hover())
                    .bd(
                        1.0,
                        if default_selected {
                            tint_alpha(accent, 190)
                        } else {
                            border_subtle()
                        },
                    )
                    .bevel_rounded(6.0, 1.3)
                    .clicked()
                    && !edit_mode
                {
                    push_click(actions);
                    actions.push(UiAction::AvatarLoadProfile {
                        profile: String::from("default"),
                    });
                }

                // Saved user profiles
                for (idx, prof) in profiles.iter().enumerate() {
                    let is_selected = active_profile_display == prof.as_str();
                    let is_staged_this = !staged_profile.is_empty() && staged_profile == *prof;

                    ui.row(|ui| {
                        if ui
                            .btn_with(&format!("mm_aas_prof_item_{idx}"), |ui| {
                                ui.row(|ui| {
                                    ui.txt(prof)
                                        .sz(11.0)
                                        .semibold()
                                        .fg(if is_selected {
                                            TEXT_PRIMARY
                                        } else {
                                            TEXT_MUTED
                                        })
                                        .ellipsis()
                                        .grow(1.0);
                                    if is_selected && !edit_mode {
                                        ui.txt(if is_staged_this { "STAGED" } else { "ACTIVE" })
                                            .sz(8.5)
                                            .bold()
                                            .fg(if is_staged_this { AMBER } else { accent });
                                    }
                                })
                                .align_center()
                                .gap(4.0)
                                .grow(1.0);
                            })
                            .grow(1.0)
                            .min_w(0.0)
                            .px(9.0)
                            .py(6.5)
                            .bg(if is_selected {
                                tint_alpha(accent, 42)
                            } else {
                                card_inner_bg()
                            })
                            .hover_bg(tile_idle_hover())
                            .bd(
                                1.0,
                                if is_selected {
                                    tint_alpha(accent, 190)
                                } else {
                                    border_subtle()
                                },
                            )
                            .bevel_rounded(6.0, 1.3)
                            .clicked()
                            && !edit_mode
                        {
                            push_click(actions);
                            actions.push(UiAction::AvatarLoadProfile {
                                profile: prof.clone(),
                            });
                        }

                        if edit_mode {
                            // Reorder Up
                            if idx > 0 {
                                if ui
                                    .btn_with(&format!("mm_aas_prof_up_{idx}"), |ui| {
                                        icons::icon(ui, icons::ARROW_UP, 13.0).fg(TEXT_PRIMARY);
                                    })
                                    .p(4.0)
                                    .bg(tile_idle_bg())
                                    .hover_bg(tile_idle_hover())
                                    .bd(1.0, border_subtle())
                                    .bevel_rounded(4.5, 1.2)
                                    .clicked()
                                {
                                    push_click(actions);
                                    let mut reordered = profiles.clone();
                                    reordered.swap(idx - 1, idx);
                                    actions.push(UiAction::AvatarReorderProfiles {
                                        profiles: reordered,
                                    });
                                }
                            }
                            // Reorder Down
                            if idx + 1 < profiles.len() {
                                if ui
                                    .btn_with(&format!("mm_aas_prof_dn_{idx}"), |ui| {
                                        icons::icon(ui, icons::ARROW_DOWN, 13.0).fg(TEXT_PRIMARY);
                                    })
                                    .p(4.0)
                                    .bg(tile_idle_bg())
                                    .hover_bg(tile_idle_hover())
                                    .bd(1.0, border_subtle())
                                    .bevel_rounded(4.5, 1.2)
                                    .clicked()
                                {
                                    push_click(actions);
                                    let mut reordered = profiles.clone();
                                    reordered.swap(idx, idx + 1);
                                    actions.push(UiAction::AvatarReorderProfiles {
                                        profiles: reordered,
                                    });
                                }
                            }
                            // Save / Overwrite
                            if ui
                                .btn_with(&format!("mm_aas_prof_save_{idx}"), |ui| {
                                    icons::icon(ui, icons::SAVE, 13.0).fg(EMERALD);
                                })
                                .p(4.0)
                                .bg(tint_alpha(EMERALD, 36))
                                .hover_bg(tint_alpha(EMERALD, 65))
                                .bd(1.0, tint_alpha(EMERALD, 150))
                                .bevel_rounded(4.5, 1.2)
                                .clicked()
                            {
                                push_click(actions);
                                actions.push(UiAction::AvatarSaveProfile {
                                    profile: prof.clone(),
                                });
                            }
                            // Delete
                            if ui
                                .btn_with(&format!("mm_aas_prof_del_{idx}"), |ui| {
                                    icons::icon(ui, icons::TRASH, 13.0).fg(RED);
                                })
                                .p(4.0)
                                .bg(tint_alpha(RED, 36))
                                .hover_bg(tint_alpha(RED, 65))
                                .bd(1.0, tint_alpha(RED, 150))
                                .bevel_rounded(4.5, 1.2)
                                .clicked()
                            {
                                push_click(actions);
                                actions.push(UiAction::AvatarDeleteProfile {
                                    profile: prof.clone(),
                                });
                            }
                        }
                    })
                    .align_center()
                    .gap(3.0);
                }
            })
            .grow(1.0)
            .gap(5.0);

            // Add New Profile row (max 12 chars matching CVR vanilla)
            ui.col(|ui| {
                let mut inp_resp = ui
                    .input(
                        "mm_aas_new_profile",
                        &mut state.main_menu.aas_new_profile_input,
                    )
                    .placeholder("New profile (max 12)")
                    .w_frac(1.0)
                    .px(8.0)
                    .py(5.5)
                    .bg(card_inner_bg())
                    .bd(1.0, border_subtle())
                    .bevel_rounded(5.5, 1.2);

                if is_vr && inp_resp.clicked() {
                    actions.push(UiAction::OpenVirtualKeyboard {
                        surface_id: 0,
                        field_id: String::from("mm_aas_new_profile"),
                        initial_text: state.main_menu.aas_new_profile_input.text().to_owned(),
                        title: String::from("New Avatar Profile"),
                        placeholder: String::from("Profile name..."),
                        max_len: 12,
                        hidden: false,
                    });
                }

                let draft: String = state
                    .main_menu
                    .aas_new_profile_input
                    .text()
                    .trim()
                    .chars()
                    .take(12)
                    .collect();
                let can_save = !draft.is_empty() && !draft.eq_ignore_ascii_case("default");
                if ui
                    .btn_with("mm_aas_prof_add", |ui| {
                        ui.row(|ui| {
                            icons::icon(ui, icons::ADD, 14.0)
                                .fg(if can_save { accent } else { TEXT_DIM });
                            ui.txt("Save New Profile")
                                .sz(10.5)
                                .semibold()
                                .fg(if can_save { TEXT_PRIMARY } else { TEXT_MUTED });
                        })
                        .center()
                        .gap(4.0)
                        .grow(1.0);
                    })
                    .w_frac(1.0)
                    .py(6.0)
                    .bg(if can_save {
                        tint_alpha(accent, 42)
                    } else {
                        tile_idle_bg()
                    })
                    .hover_bg(if can_save {
                        tint_alpha(accent, 72)
                    } else {
                        tile_idle_hover()
                    })
                    .bd(
                        1.0,
                        if can_save {
                            tint_alpha(accent, 170)
                        } else {
                            border_subtle()
                        },
                    )
                    .bevel_rounded(5.5, 1.3)
                    .clicked()
                    && can_save
                {
                    push_click(actions);
                    actions.push(UiAction::AvatarSaveProfile { profile: draft });
                    state.main_menu.aas_new_profile_input = lumenic::TextEditState::new();
                }
            })
            .gap(5.0);
        })
        .w(218.0)
        .p(10.0)
        .gap(8.0)
        .bg(card_bg())
        .bd(1.0, border_subtle())
        .bevel_rounded(10.5, 2.0)
        .shadow(14.0, SHADOW_CARD);

        // =====================================================================
        // COLUMN 3: ADVANCED PARAMETER CONTROLS GRID
        // =====================================================================
        ui.col(|ui| {
            // Header bar
            ui.row(|ui| {
                icons::icon(ui, icons::SLIDERS, 16.0).fg(accent);
                ui.txt(&format!("Avatar Settings ({})", entries.len()))
                    .sz(14.0)
                    .bold()
                    .fg(TEXT_PRIMARY)
                    .grow(1.0);

                if !auto_apply && has_staged {
                    ui.txt("STAGED PREVIEW")
                        .sz(9.0)
                        .bold()
                        .fg(AMBER)
                        .bg(tint_alpha(AMBER, 34))
                        .bd(1.0, tint_alpha(AMBER, 150))
                        .px(7.0)
                        .py(2.5)
                        .bevel_rounded(4.5, 1.2);
                }

                if ui
                    .btn_with("mm_aas_reset_def", |ui| {
                        ui.txt("Reset Defaults")
                            .sz(10.5)
                            .semibold()
                            .fg(TEXT_PRIMARY);
                    })
                    .px(9.0)
                    .py(5.0)
                    .bg(tint_alpha(accent, 38))
                    .hover_bg(tint_alpha(accent, 68))
                    .bd(1.0, tint_alpha(accent, 155))
                    .bevel_rounded(5.5, 1.3)
                    .clicked()
                {
                    push_click(actions);
                    actions.push(UiAction::AvatarResetDefaults);
                }
            })
            .align_center()
            .gap(7.0);

            // Pinned full-width Global Avatar Scale bar (GS2 UniversalScaling)
            {
                let cur_h = state.host.avatar_settings.avatar_height.max(0.05);
                let init_h = state.host.avatar_settings.initial_height.max(0.05);
                let min_h = state.host.avatar_settings.scale_min.max(0.05);
                let max_h = state.host.avatar_settings.scale_max.max(min_h + 0.05);
                let allowed = state.host.avatar_settings.scaling_allowed;
                let mult = cur_h / init_h;

                ui.col(|ui| {
                    ui.row(|ui| {
                        icons::icon(ui, icons::AVATAR, 14.0).fg(if allowed { accent } else { AMBER });
                        ui.txt("Global Avatar Scale")
                            .sz(11.5)
                            .bold()
                            .fg(TEXT_PRIMARY)
                            .grow(1.0);

                        if !allowed {
                            ui.txt("LOCKED BY WORLD")
                                .sz(8.5)
                                .bold()
                                .fg(AMBER)
                                .bg(tint_alpha(AMBER, 34))
                                .bd(1.0, tint_alpha(AMBER, 150))
                                .px(6.0)
                                .py(2.0)
                                .bevel_rounded(4.0, 1.2);
                        } else {
                            ui.txt(&format!("{cur_h:.2}m ({mult:.2}x)"))
                                .sz(10.5)
                                .semibold()
                                .fg(accent)
                                .bg(tint_alpha(accent, 30))
                                .bd(1.0, tint_alpha(accent, 130))
                                .px(7.0)
                                .py(2.0)
                                .bevel_rounded(4.5, 1.2);

                            if ui
                                .btn_with("mm_aas_reset_height", |ui| {
                                    ui.txt(&format!("Reset ({init_h:.2}m)"))
                                        .sz(9.5)
                                        .semibold()
                                        .fg(TEXT_PRIMARY);
                                })
                                .px(7.5)
                                .py(3.0)
                                .bg(tile_idle_bg())
                                .hover_bg(tile_idle_hover())
                                .bd(1.0, border_subtle())
                                .bevel_rounded(4.5, 1.2)
                                .clicked()
                            {
                                push_click(actions);
                                state
                                    .main_menu
                                    .avatar_sliders
                                    .insert(String::from("__universal_height"), (init_h, init_h));
                                actions.push(UiAction::AvatarResetHeight);
                            }
                        }
                    })
                    .align_center()
                    .gap(6.0);

                    if allowed {
                        let (cur_s, last_s) = state
                            .main_menu
                            .avatar_sliders
                            .entry(String::from("__universal_height"))
                            .or_insert((cur_h, cur_h));
                        if (*last_s - cur_h).abs() > 0.005 {
                            *cur_s = cur_h;
                            *last_s = cur_h;
                        }
                        let mut h_val = cur_s.clamp(min_h, max_h);
                        ui.push_id("__universal_height");
                        if ui
                            .slider("Height (m)", &mut h_val, min_h..=max_h)
                            .fill(accent)
                            .bg(card_bg())
                            .bd(1.0, border_subtle())
                            .bevel_rounded(5.5, 1.3)
                            .w_frac(1.0)
                            .changed()
                        {
                            *cur_s = h_val;
                            *last_s = h_val;
                            actions.push(UiAction::AvatarSetHeight { height: h_val });
                        }
                        ui.pop_id();
                    }
                })
                .w_frac(1.0)
                .p(8.5)
                .gap(5.0)
                .bg(card_inner_bg())
                .bd(
                    1.0,
                    if allowed {
                        tint_alpha(accent, 95)
                    } else {
                        tint_alpha(AMBER, 120)
                    },
                )
                .bevel_rounded(7.5, 1.4);
            }

            // Parameter grid
            if entries.is_empty() {
                ui.col(|ui| {
                    icons::icon(ui, icons::AVATAR, 34.0).fg(TEXT_DIM);
                    ui.txt("Current avatar has no Advanced Avatar Settings")
                        .sz(13.0)
                        .semibold()
                        .fg(TEXT_MUTED);
                })
                .center()
                .grow(1.0)
                .gap(8.0);
            } else {
                ui.scroll("mm_aas_scroll", |ui| {
                    for chunk in entries.chunks(2) {
                        ui.row(|ui| {
                            for entry in chunk {
                                render_param_card(ui, state, entry, accent, actions);
                            }
                            if chunk.len() == 1 {
                                ui.col(|_| {}).grow(1.0);
                            }
                        })
                        .gap(8.0);
                    }
                })
                .grow(1.0)
                .gap(8.0);
            }
        })
        .grow(1.0)
        .p(11.0)
        .gap(9.0)
        .bg(card_bg())
        .bd(1.0, border_subtle())
        .bevel_rounded(10.5, 2.0)
        .shadow(14.0, SHADOW_CARD);
    })
    .grow(1.0)
    .gap(9.0);
}

fn render_slider_axis(
    ui: &mut UiContext,
    state: &mut AppState,
    key: &str,
    label: &str,
    host_val: f32,
    min_v: f32,
    max_v: f32,
    accent: Color,
    actions: &mut Vec<UiAction>,
) {
    let safe_max = if max_v > min_v { max_v } else { min_v + 1.0 };
    let (cur, last_host) = state
        .main_menu
        .avatar_sliders
        .entry(key.to_owned())
        .or_insert((host_val, host_val));
    if (*last_host - host_val).abs() > 0.001 {
        *cur = host_val;
        *last_host = host_val;
    }
    let mut v = cur.clamp(min_v, safe_max);
    ui.push_id(key);
    if ui
        .slider(label, &mut v, min_v..=safe_max)
        .fill(accent)
        .bg(card_inner_bg())
        .bd(1.0, border_subtle())
        .bevel_rounded(6.0, 1.3)
        .w_frac(1.0)
        .changed()
    {
        *cur = v;
        *last_host = v;
        actions.push(UiAction::AvatarSetParam {
            param: key.to_owned(),
            value: v,
        });
    }
    ui.pop_id();
}

fn render_param_card(
    ui: &mut UiContext,
    state: &mut AppState,
    entry: &AvatarSettingEntry,
    accent: Color,
    actions: &mut Vec<UiAction>,
) {
    ui.col(|ui| match entry.kind.as_str() {
        "toggle" => {
            let on = entry.val_x >= 0.5;
            ui.row(|ui| {
                ui.txt(&entry.name)
                    .sz(11.5)
                    .semibold()
                    .fg(TEXT_PRIMARY)
                    .ellipsis()
                    .grow(1.0);
                if ui
                    .btn_with(&format!("mm_aas_tog_{}", entry.param), |ui| {
                        ui.txt(if on { "ON" } else { "OFF" })
                            .sz(10.5)
                            .bold()
                            .align(TextAlign::Center)
                            .fg(if on { TEXT_PRIMARY } else { TEXT_MUTED })
                            .grow(1.0);
                    })
                    .w(54.0)
                    .py(5.0)
                    .bg(if on { tint_alpha(EMERALD, 58) } else { tile_idle_bg() })
                    .hover_bg(tile_idle_hover())
                    .bd(
                        1.0,
                        if on {
                            tint_alpha(EMERALD, 195)
                        } else {
                            border_subtle()
                        },
                    )
                    .bevel_rounded(5.5, 1.3)
                    .clicked()
                {
                    push_click(actions);
                    actions.push(UiAction::AvatarSetParam {
                        param: entry.param.clone(),
                        value: if on { 0.0 } else { 1.0 },
                    });
                }
            })
            .align_center()
            .gap(8.0);
        }
        "dropdown" => {
            ui.txt(&entry.name)
                .sz(11.5)
                .semibold()
                .fg(TEXT_PRIMARY)
                .ellipsis();
            let cur_idx = entry.val_x.round() as i32;
            ui.row(|ui| {
                for (i, opt) in entry.options.iter().enumerate() {
                    let active = cur_idx == i as i32;
                    if ui
                        .btn_with(&format!("mm_aas_dd_{}_{i}", entry.param), |ui| {
                            ui.txt(opt)
                                .sz(10.0)
                                .semibold()
                                .align(TextAlign::Center)
                                .fg(if active { TEXT_PRIMARY } else { TEXT_MUTED })
                                .ellipsis()
                                .grow(1.0);
                        })
                        .grow(1.0)
                        .min_w(0.0)
                        .py(5.0)
                        .bg(if active { tint_alpha(accent, 52) } else { tile_idle_bg() })
                        .hover_bg(tile_idle_hover())
                        .bd(
                            1.0,
                            if active {
                                tint_alpha(accent, 190)
                            } else {
                                border_subtle()
                            },
                        )
                        .bevel_rounded(5.0, 1.3)
                        .clicked()
                    {
                        push_click(actions);
                        actions.push(UiAction::AvatarSetParam {
                            param: entry.param.clone(),
                            value: i as f32,
                        });
                    }
                }
            })
            .gap(4.0);
        }
        "colorpicker" => {
            let swatch = Color::rgb(
                (entry.val_x.clamp(0.0, 1.0) * 255.0) as u8,
                (entry.val_y.clamp(0.0, 1.0) * 255.0) as u8,
                (entry.val_z.clamp(0.0, 1.0) * 255.0) as u8,
            );
            ui.row(|ui| {
                ui.txt(&entry.name)
                    .sz(11.5)
                    .semibold()
                    .fg(TEXT_PRIMARY)
                    .ellipsis()
                    .grow(1.0);
                ui.col(|_| {})
                    .w(28.0)
                    .h(16.0)
                    .bg(swatch)
                    .bd(1.0, border_subtle())
                    .bevel_rounded(4.0, 1.3);
            })
            .align_center()
            .gap(6.0);
            render_slider_axis(
                ui,
                state,
                &format!("{}-r", entry.param),
                "R",
                entry.val_x,
                0.0,
                1.0,
                Color::rgb(239, 68, 68),
                actions,
            );
            render_slider_axis(
                ui,
                state,
                &format!("{}-g", entry.param),
                "G",
                entry.val_y,
                0.0,
                1.0,
                EMERALD,
                actions,
            );
            render_slider_axis(
                ui,
                state,
                &format!("{}-b", entry.param),
                "B",
                entry.val_z,
                0.0,
                1.0,
                Color::rgb(59, 130, 246),
                actions,
            );
        }
        "joystick2d" | "inputvector2" => {
            ui.txt(&entry.name)
                .sz(11.5)
                .semibold()
                .fg(TEXT_PRIMARY)
                .ellipsis();
            render_slider_axis(
                ui,
                state,
                &format!("{}-x", entry.param),
                "X",
                entry.val_x,
                entry.min_x,
                entry.max_x,
                accent,
                actions,
            );
            render_slider_axis(
                ui,
                state,
                &format!("{}-y", entry.param),
                "Y",
                entry.val_y,
                entry.min_y,
                entry.max_y,
                accent,
                actions,
            );
        }
        "joystick3d" | "inputvector3" => {
            ui.txt(&entry.name)
                .sz(11.5)
                .semibold()
                .fg(TEXT_PRIMARY)
                .ellipsis();
            render_slider_axis(
                ui,
                state,
                &format!("{}-x", entry.param),
                "X",
                entry.val_x,
                entry.min_x,
                entry.max_x,
                accent,
                actions,
            );
            render_slider_axis(
                ui,
                state,
                &format!("{}-y", entry.param),
                "Y",
                entry.val_y,
                entry.min_y,
                entry.max_y,
                accent,
                actions,
            );
            render_slider_axis(
                ui,
                state,
                &format!("{}-z", entry.param),
                "Z",
                entry.val_z,
                entry.min_z,
                entry.max_z,
                accent,
                actions,
            );
        }
        _ => {
            render_slider_axis(
                ui,
                state,
                &entry.param,
                &entry.name,
                entry.val_x,
                entry.min_x,
                entry.max_x,
                accent,
                actions,
            );
        }
    })
    .grow(1.0)
    .p(9.0)
    .gap(5.0)
    .bg(card_inner_bg())
    .bd(1.0, border_subtle())
    .bevel_rounded(7.0, 1.3);
}
