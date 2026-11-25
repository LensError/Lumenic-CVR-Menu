use lumenic::{Color, ImageFit, TextAlign, UiContext};

use crate::icons;
use crate::state::{
    AppState, ContentCardItem, ContentCategoryItem, ContentDetailSnapshot, ContentListSnapshot,
    InstanceDetailSnapshot, UiAction,
};
use crate::theme::{
    push_click, tint_alpha, AMBER, border_subtle, card_bg, card_inner_bg, EMERALD, SHADOW_CARD,
    TEXT_DIM, TEXT_MUTED, TEXT_PRIMARY, tile_idle_bg, tile_idle_hover,
};

fn kind_meta(tab_id: &str) -> (&'static str, &'static str, &'static str) {
    match tab_id {
        "worlds" => ("Worlds", icons::WORLD, "world"),
        "avatars" => ("Avatars", icons::AVATAR, "avatar"),
        "props" => ("Props", icons::CUBE, "prop"),
        _ => ("Content", icons::COMPASS, "world"),
    }
}

/// Renders the unified Phase 3B Content Browser for Worlds, Avatars, or Props.
pub fn render_content_browser(
    ui: &mut UiContext,
    state: &mut AppState,
    tab_id: &str,
    accent: Color,
    actions: &mut Vec<UiAction>,
) {
    let (title, icon_path, detail_kind) = kind_meta(tab_id);

    // If an instance detail or content detail view is active, render it cleanly.
    if let Some(inst) = state.host.content.instance_detail.clone() {
        render_instance_detail_view(ui, state, &inst, accent, actions);
        return;
    }
    if let Some(detail) = state.host.content.detail.clone() {
        render_detail_view(ui, state, &detail, accent, actions);
        return;
    }
    if let Some(gd) = state.host.social.group_detail.clone() {
        crate::social_view::render_group_detail_view(ui, state, &gd, accent, actions);
        return;
    }
    if let Some(user_detail) = state.host.social.user_detail.clone() {
        crate::social_view::render_user_detail_view(ui, state, &user_detail, accent, actions);
        return;
    }

    let (categories, list): (Vec<ContentCategoryItem>, ContentListSnapshot) = match tab_id {
        "worlds" => (
            state.host.content.world_categories.clone(),
            state.host.content.worlds.clone(),
        ),
        "avatars" => (
            state.host.content.avatar_categories.clone(),
            state.host.content.avatars.clone(),
        ),
        _ => (
            state.host.content.prop_categories.clone(),
            state.host.content.props.clone(),
        ),
    };

    // Auto-select & load default category on first visit regardless of entry path (QM, Home, Back, etc.).
    if !list.loaded_once && !list.loading && list.error.is_empty() {
        if !state.main_menu.cb_auto_loaded.contains(tab_id) {
            state.main_menu.cb_auto_loaded.insert(String::from(tab_id));
            actions.push(UiAction::ContentLoadList {
                kind: String::from(tab_id),
                category_key: list.category_key.clone(),
                is_system: list.is_system,
                page: 0,
                sort_order: list.sort_order,
                ascending: list.ascending,
            });
        }
    } else if list.loaded_once {
        state.main_menu.cb_auto_loaded.remove(tab_id);
    }

    ui.col(|ui| {
        // =====================================================================
        // TOP BAR: TITLE + FILTER + SORT + REFRESH
        // =====================================================================
        ui.row(|ui| {
            icons::icon(ui, icon_path, 17.0).fg(accent);
            ui.txt(title).sz(crate::theme::fs(15.0)).bold().fg(TEXT_PRIMARY);

            if !list.items.is_empty() {
                let count_txt = if list.has_more {
                    format!("{}+", list.items.len())
                } else {
                    format!("{}", list.items.len())
                };
                ui.row(|ui| {
                    ui.txt(&count_txt)
                        .sz(crate::theme::fs(10.0))
                        .semibold()
                        .fg(accent);
                })
                .px(7.0)
                .py(2.5)
                .bg(tint_alpha(accent, 30))
                .bd(1.0, tint_alpha(accent, 110))
                .bevel_rounded(4.5, 1.0);
            }

            // Local quick filter for current page
            ui.row(|ui| {
                icons::icon(ui, icons::SEARCH, 13.0).fg(TEXT_MUTED);
                let mut f_inp = ui
                    .input("mm_cb_filter", &mut state.main_menu.browser_local_filter)
                    .placeholder("Filter current page...")
                    .bg(card_inner_bg())
                    .fg(TEXT_PRIMARY)
                    .grow(1.0);
                if state.host.is_vr && f_inp.clicked() {
                    actions.push(UiAction::OpenVirtualKeyboard {
                        surface_id: 0,
                        field_id: String::from("mm_cb_filter"),
                        initial_text: state.main_menu.browser_local_filter.text(),
                        title: String::from("Filter Page"),
                        placeholder: String::from("Filter current page..."),
                        max_len: 96,
                        hidden: false,
                    });
                }
            })
            .align_center()
            .gap(6.0)
            .grow(1.0)
            .px(8.0);

            // Sort chips: Default | Updated | Name
            for (ord, lbl) in [(0i32, "Default"), (3i32, "Updated"), (2i32, "Name")] {
                let active = list.sort_order == ord;
                let id = format!("mm_cb_sort_{tab_id}_{ord}");
                if ui
                    .btn_with(&id, |ui| {
                        ui.txt(lbl)
                            .sz(crate::theme::fs(10.5))
                            .semibold()
                            .align(TextAlign::Center)
                            .fg(if active { TEXT_PRIMARY } else { TEXT_MUTED })
                            .grow(1.0);
                    })
                    .px(9.0)
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
                    actions.push(UiAction::ContentLoadList {
                        kind: String::from(tab_id),
                        category_key: list.category_key.clone(),
                        is_system: list.is_system,
                        page: 0,
                        sort_order: ord,
                        ascending: list.ascending,
                    });
                }
            }

            // Asc / Desc toggle
            let dir_lbl = if list.ascending { "Asc" } else { "Desc" };
            if ui
                .btn_with(&format!("mm_cb_dir_{tab_id}"), |ui| {
                    ui.txt(dir_lbl)
                        .sz(crate::theme::fs(10.5))
                        .semibold()
                        .align(TextAlign::Center)
                        .fg(TEXT_PRIMARY)
                        .grow(1.0);
                })
                .px(9.0)
                .py(5.5)
                .bg(tile_idle_bg())
                .hover_bg(tile_idle_hover())
                .bd(1.0, border_subtle())
                .bevel_rounded(5.5, 1.3)
                .clicked()
            {
                push_click(actions);
                actions.push(UiAction::ContentLoadList {
                    kind: String::from(tab_id),
                    category_key: list.category_key.clone(),
                    is_system: list.is_system,
                    page: 0,
                    sort_order: list.sort_order,
                    ascending: !list.ascending,
                });
            }

            // Refresh
            if ui
                .btn_with(&format!("mm_cb_ref_{tab_id}"), |ui| {
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
                actions.push(UiAction::ContentLoadList {
                    kind: String::from(tab_id),
                    category_key: list.category_key.clone(),
                    is_system: list.is_system,
                    page: 0,
                    sort_order: list.sort_order,
                    ascending: list.ascending,
                });
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

        // =====================================================================
        // BODY: LEFT CATEGORIES SIDEBAR | RIGHT 4-COLUMN CARD GRID + PAGING
        // =====================================================================
        ui.row(|ui| {
            // --- LEFT SIDEBAR: CATEGORIES ---
            ui.col(|ui| {
                ui.txt("Categories")
                    .sz(crate::theme::fs(11.5))
                    .semibold()
                    .fg(TEXT_MUTED)
                    .px(4.0);

                ui.scroll(&format!("mm_cb_cats_{tab_id}"), |ui| {
                    if categories.is_empty() {
                        ui.txt(if state.host.content.categories_loaded {
                            "No categories"
                        } else {
                            "Loading categories..."
                        })
                        .sz(crate::theme::fs(11.0))
                        .fg(TEXT_DIM)
                        .p(8.0);
                    } else {
                        let recent_cats: Vec<&ContentCategoryItem> =
                            categories.iter().filter(|c| c.is_recent).collect();
                        let sys_cats: Vec<&ContentCategoryItem> = categories
                            .iter()
                            .filter(|c| c.is_system && !c.is_recent)
                            .collect();
                        let usr_cats: Vec<&ContentCategoryItem> =
                            categories.iter().filter(|c| !c.is_system).collect();

                        if !recent_cats.is_empty() {
                            ui.txt("Recent")
                                .sz(crate::theme::fs(10.5))
                                .semibold()
                                .fg(accent)
                                .px(4.0);
                            for cat in recent_cats {
                                render_category_btn(ui, tab_id, cat, &list, accent, actions);
                            }
                        }

                        if !sys_cats.is_empty() {
                            ui.txt("System")
                                .sz(crate::theme::fs(10.5))
                                .semibold()
                                .fg(TEXT_DIM)
                                .pt(6.0)
                                .px(4.0);
                            for cat in sys_cats {
                                render_category_btn(ui, tab_id, cat, &list, accent, actions);
                            }
                        }

                        if !usr_cats.is_empty() {
                            ui.txt("Favorites & Lists")
                                .sz(crate::theme::fs(10.5))
                                .semibold()
                                .fg(TEXT_DIM)
                                .pt(8.0)
                                .px(4.0);
                            for cat in usr_cats {
                                render_category_btn(ui, tab_id, cat, &list, accent, actions);
                            }
                        }
                    }
                })
                .grow(1.0)
                .gap(4.5)
                .no_scrollbar();
            })
            .w(218.0)
            .h_frac(1.0)
            .p(9.0)
            .gap(6.0)
            .bg(card_bg())
            .bd(1.0, border_subtle())
            .bevel_rounded(10.5, 2.0)
            .shadow(14.0, SHADOW_CARD);

            // --- RIGHT PANEL: THUMBNAIL GRID (INFINITE LAZY LOADING) ---
            ui.col(|ui| {
                let grid_id = format!("mm_cb_grid_{tab_id}");
                if state.main_menu.cb_last_revisions.get(tab_id).copied() != Some(list.list_revision) {
                    state
                        .main_menu
                        .cb_last_revisions
                        .insert(String::from(tab_id), list.list_revision);
                    state.main_menu.cb_load_more_requested_page.remove(tab_id);
                    ui.scroll_to(&grid_id, 0.0);
                }

                let local_q = state
                    .main_menu
                    .browser_local_filter
                    .text()
                    .trim()
                    .to_ascii_lowercase();
                let filtered: Vec<&ContentCardItem> = list
                    .items
                    .iter()
                    .filter(|it| {
                        local_q.is_empty() || it.name.to_ascii_lowercase().contains(&local_q)
                    })
                    .collect();

                if list.loading && filtered.is_empty() {
                    ui.col(|ui| {
                        icons::icon(ui, icon_path, 28.0).fg(accent);
                        ui.txt("Loading...").sz(crate::theme::fs(13.0)).semibold().fg(TEXT_MUTED);
                    })
                    .center()
                    .grow(1.0)
                    .gap(6.0);
                } else if !list.error.is_empty() && filtered.is_empty() {
                    ui.col(|ui| {
                        ui.txt(&list.error).sz(crate::theme::fs(12.0)).fg(AMBER).wrap();
                    })
                    .center()
                    .grow(1.0)
                    .p(16.0);
                } else if filtered.is_empty() {
                    ui.col(|ui| {
                        icons::icon(ui, icon_path, 28.0).fg(TEXT_DIM);
                        ui.txt("No items found").sz(crate::theme::fs(12.5)).semibold().fg(TEXT_MUTED);
                    })
                    .center()
                    .grow(1.0)
                    .gap(6.0);
                } else {
                    ui.scroll(&grid_id, |ui| {
                        for chunk in filtered.chunks(4) {
                            ui.row(|ui| {
                                for item in chunk {
                                    render_content_card(
                                        ui,
                                        state,
                                        item,
                                        detail_kind,
                                        icon_path,
                                        tab_id == "worlds",
                                        accent,
                                        actions,
                                    );
                                }
                                // Keep partial last row aligned to 4-column grid
                                for _ in chunk.len()..4 {
                                    ui.col(|_| {}).grow(1.0);
                                }
                            })
                            .gap(8.0);
                        }

                        // Inline lazy-load footer
                        ui.row(|ui| {
                            if list.loading_more {
                                ui.row(|ui| {
                                    icons::icon(ui, icons::RELOAD, 12.5).fg(accent);
                                    ui.txt("Loading more...")
                                        .sz(crate::theme::fs(11.0))
                                        .semibold()
                                        .fg(TEXT_MUTED);
                                })
                                .center()
                                .px(14.0)
                                .py(6.0)
                                .gap(6.0)
                                .bg(tint_alpha(accent, 24))
                                .bd(1.0, tint_alpha(accent, 95))
                                .bevel_rounded(6.0, 1.3);
                            } else if !list.error.is_empty() && list.has_more {
                                ui.txt(&list.error).sz(crate::theme::fs(10.5)).fg(AMBER);
                                if ui
                                    .btn_with(&format!("mm_cb_retry_more_{tab_id}"), |ui| {
                                        ui.txt("Retry")
                                            .sz(crate::theme::fs(10.5))
                                            .semibold()
                                            .fg(TEXT_PRIMARY);
                                    })
                                    .px(11.0)
                                    .py(5.0)
                                    .bg(tile_idle_bg())
                                    .hover_bg(tile_idle_hover())
                                    .bd(1.0, border_subtle())
                                    .bevel_rounded(5.5, 1.3)
                                    .clicked()
                                {
                                    push_click(actions);
                                    state
                                        .main_menu
                                        .cb_load_more_requested_page
                                        .insert(String::from(tab_id), (list.list_revision, list.page));
                                    actions.push(UiAction::ContentLoadMore {
                                        kind: String::from(tab_id),
                                    });
                                }
                            } else if list.has_more {
                                if ui
                                    .btn_with(&format!("mm_cb_load_more_{tab_id}"), |ui| {
                                        ui.txt("Load More")
                                            .sz(crate::theme::fs(11.0))
                                            .semibold()
                                            .fg(TEXT_PRIMARY);
                                    })
                                    .px(16.0)
                                    .py(6.0)
                                    .bg(tint_alpha(accent, 34))
                                    .hover_bg(tint_alpha(accent, 65))
                                    .bd(1.0, tint_alpha(accent, 145))
                                    .bevel_rounded(6.0, 1.3)
                                    .clicked()
                                {
                                    push_click(actions);
                                    state
                                        .main_menu
                                        .cb_load_more_requested_page
                                        .insert(String::from(tab_id), (list.list_revision, list.page));
                                    actions.push(UiAction::ContentLoadMore {
                                        kind: String::from(tab_id),
                                    });
                                }
                            } else {
                                ui.txt(&format!("End of list ({} items)", filtered.len()))
                                    .sz(crate::theme::fs(10.5))
                                    .fg(TEXT_DIM);
                            }
                        })
                        .center()
                        .py(8.0)
                        .gap(8.0);
                    })
                    .grow(1.0)
                    .gap(8.0)
                    .no_scrollbar();

                    // Auto-trigger lazy load when scrolled close to bottom (or when grid does not yet overflow)
                    let scroll_y = ui.scroll_offset(&grid_id);
                    let max_y = ui.scroll_max_y(&grid_id);
                    let near_bottom = max_y <= 1.0 || scroll_y >= (max_y - 260.0).max(0.0);
                    let already_requested = state
                        .main_menu
                        .cb_load_more_requested_page
                        .get(tab_id)
                        .copied()
                        == Some((list.list_revision, list.page));
                    if list.has_more
                        && !list.loading
                        && !list.loading_more
                        && list.error.is_empty()
                        && near_bottom
                        && !already_requested
                        && local_q.is_empty()
                    {
                        state
                            .main_menu
                            .cb_load_more_requested_page
                            .insert(String::from(tab_id), (list.list_revision, list.page));
                        actions.push(UiAction::ContentLoadMore {
                            kind: String::from(tab_id),
                        });
                    }
                }
            })
            .grow(1.0)
            .h_frac(1.0)
            .p(10.0)
            .gap(6.0)
            .bg(card_bg())
            .bd(1.0, border_subtle())
            .bevel_rounded(10.5, 2.0)
            .shadow(14.0, SHADOW_CARD);
        })
        .grow(1.0)
        .gap(9.0);
    })
    .grow(1.0)
    .gap(9.0);
}

/// Renders the Phase 3D Discover hub using CVR's official Random endpoints (WorldsRandom, AvatarsRandom, PropsRandom).
pub fn render_discover_page(
    ui: &mut UiContext,
    state: &mut AppState,
    accent: Color,
    actions: &mut Vec<UiAction>,
) {
    if let Some(inst) = state.host.content.instance_detail.clone() {
        render_instance_detail_view(ui, state, &inst, accent, actions);
        return;
    }
    if let Some(detail) = state.host.content.detail.clone() {
        render_detail_view(ui, state, &detail, accent, actions);
        return;
    }
    if let Some(gd) = state.host.social.group_detail.clone() {
        crate::social_view::render_group_detail_view(ui, state, &gd, accent, actions);
        return;
    }
    if let Some(user_detail) = state.host.social.user_detail.clone() {
        crate::social_view::render_user_detail_view(ui, state, &user_detail, accent, actions);
        return;
    }

    if !state.host.content.discover_loaded
        && !state.host.content.discover_loading
        && state.host.content.discover_error.is_empty()
    {
        actions.push(UiAction::ContentLoadDiscover);
    }

    ui.col(|ui| {
        ui.row(|ui| {
            icons::icon(ui, icons::COMPASS, 17.0).fg(accent);
            ui.txt("Discover Random Content")
                .sz(crate::theme::fs(15.0))
                .bold()
                .fg(TEXT_PRIMARY)
                .grow(1.0);

            for (idx, ic, lbl) in [
                (0usize, icons::WORLD, "Random Worlds"),
                (1usize, icons::AVATAR, "Random Avatars"),
                (2usize, icons::CUBE, "Random Props"),
            ] {
                let active = state.main_menu.discover_tab == idx;
                if ui
                    .btn_with(&format!("mm_disc_tab_{idx}"), |ui| {
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
                    state.main_menu.discover_tab = idx;
                }
            }

            if ui
                .btn_with("mm_disc_refresh", |ui| {
                    ui.row(|ui| {
                        icons::icon(ui, icons::RELOAD, 13.5).fg(TEXT_PRIMARY);
                        ui.txt("Roll Again").sz(crate::theme::fs(11.0)).semibold().fg(TEXT_PRIMARY);
                    })
                    .center()
                    .gap(5.0);
                })
                .px(11.0)
                .py(5.5)
                .bg(tint_alpha(accent, 42))
                .hover_bg(tint_alpha(accent, 75))
                .bd(1.0, tint_alpha(accent, 170))
                .bevel_rounded(6.0, 1.3)
                .clicked()
            {
                push_click(actions);
                actions.push(UiAction::ContentLoadDiscover);
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

        ui.col(|ui| {
            let (items, detail_kind, fallback_icon, show_players) = match state.main_menu.discover_tab {
                0 => (
                    state.host.content.discover_worlds.clone(),
                    "world",
                    icons::WORLD,
                    true,
                ),
                1 => (
                    state.host.content.discover_avatars.clone(),
                    "avatar",
                    icons::AVATAR,
                    false,
                ),
                _ => (
                    state.host.content.discover_props.clone(),
                    "prop",
                    icons::CUBE,
                    false,
                ),
            };

            if !state.host.content.discover_error.is_empty() && items.is_empty() {
                ui.col(|ui| {
                    icons::icon(ui, icons::INFO, 30.0).fg(AMBER);
                    ui.txt(&state.host.content.discover_error)
                        .sz(crate::theme::fs(12.5))
                        .semibold()
                        .fg(TEXT_MUTED);
                    if ui
                        .btn_with("mm_disc_retry", |ui| {
                            ui.txt("Retry")
                                .sz(crate::theme::fs(11.0))
                                .bold()
                                .fg(TEXT_PRIMARY);
                        })
                        .px(14.0)
                        .py(6.0)
                        .bg(tint_alpha(accent, 48))
                        .hover_bg(tint_alpha(accent, 78))
                        .bd(1.0, tint_alpha(accent, 180))
                        .bevel_rounded(6.0, 1.3)
                        .clicked()
                    {
                        push_click(actions);
                        actions.push(UiAction::ContentLoadDiscover);
                    }
                })
                .center()
                .grow(1.0)
                .gap(8.0);
            } else if state.host.content.discover_loading && items.is_empty() {
                ui.col(|ui| {
                    icons::icon(ui, icons::COMPASS, 30.0).fg(accent);
                    ui.txt("Rolling random content...")
                        .sz(crate::theme::fs(13.0))
                        .semibold()
                        .fg(TEXT_MUTED);
                })
                .center()
                .grow(1.0)
                .gap(6.0);
            } else if items.is_empty() {
                ui.col(|ui| {
                    icons::icon(ui, fallback_icon, 30.0).fg(TEXT_DIM);
                    ui.txt("No random items loaded yet.")
                        .sz(crate::theme::fs(13.0))
                        .semibold()
                        .fg(TEXT_MUTED);
                })
                .center()
                .grow(1.0)
                .gap(6.0);
            } else {
                ui.scroll("mm_disc_scroll", |ui| {
                    for chunk in items.chunks(4) {
                        ui.row(|ui| {
                            for item in chunk {
                                render_content_card(
                                    ui,
                                    state,
                                    item,
                                    detail_kind,
                                    fallback_icon,
                                    show_players,
                                    accent,
                                    actions,
                                );
                            }
                            for _ in chunk.len()..4 {
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
        .p(11.0)
        .gap(8.0)
        .bg(card_bg())
        .bd(1.0, border_subtle())
        .bevel_rounded(10.5, 2.0)
        .shadow(14.0, SHADOW_CARD);
    })
    .grow(1.0)
    .gap(9.0);
}

/// Renders the Phase 3B Global Search Results view (triggered from Home search bar).
pub fn render_global_search_view(
    ui: &mut UiContext,
    state: &mut AppState,
    accent: Color,
    actions: &mut Vec<UiAction>,
) {
    if let Some(inst) = state.host.content.instance_detail.clone() {
        render_instance_detail_view(ui, state, &inst, accent, actions);
        return;
    }
    if let Some(detail) = state.host.content.detail.clone() {
        render_detail_view(ui, state, &detail, accent, actions);
        return;
    }
    if let Some(gd) = state.host.social.group_detail.clone() {
        crate::social_view::render_group_detail_view(ui, state, &gd, accent, actions);
        return;
    }
    if let Some(user_detail) = state.host.social.user_detail.clone() {
        crate::social_view::render_user_detail_view(ui, state, &user_detail, accent, actions);
        return;
    }

    ui.col(|ui| {
        // Top search bar + Back
        ui.row(|ui| {
            if ui
                .btn_with("mm_srch_back", |ui| {
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

            icons::icon(ui, icons::SEARCH, 15.0).fg(accent);
            let submitted = ui
                .input("mm_srch_page_input", &mut state.main_menu.search_input)
                .placeholder("Search worlds, avatars, props, users...")
                .bg(card_inner_bg())
                .fg(TEXT_PRIMARY)
                .grow(1.0)
                .submitted();

            let go_clicked = ui
                .btn_with("mm_srch_page_go", |ui| {
                    ui.txt("Search").sz(crate::theme::fs(11.0)).semibold().fg(TEXT_PRIMARY);
                })
                .px(12.0)
                .py(6.0)
                .bg(tint_alpha(accent, 48))
                .hover_bg(tint_alpha(accent, 82))
                .bd(1.0, tint_alpha(accent, 170))
                .bevel_rounded(6.0, 1.3)
                .clicked();

            if submitted || go_clicked {
                let q = state.main_menu.search_input.text();
                if !q.trim().is_empty() {
                    state.main_menu.last_search = q.clone();
                    push_click(actions);
                    actions.push(UiAction::ContentGlobalSearch { query: q });
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

        // Filter pills + Results grid
        ui.col(|ui| {
            ui.row(|ui| {
                for (k, lbl) in [
                    ("all", "All"),
                    ("world", "Worlds"),
                    ("avatar", "Avatars"),
                    ("prop", "Props"),
                    ("user", "Users"),
                ] {
                    let active = state.main_menu.search_filter_kind == k;
                    if ui
                        .btn_with(&format!("mm_srch_f_{k}"), |ui| {
                            ui.txt(lbl)
                                .sz(crate::theme::fs(11.0))
                                .semibold()
                                .align(TextAlign::Center)
                                .fg(if active { TEXT_PRIMARY } else { TEXT_MUTED })
                                .grow(1.0);
                        })
                        .px(12.0)
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
                        state.main_menu.search_filter_kind = String::from(k);
                    }
                }
                ui.txt(&format!("{} results", state.host.content.search_results.len()))
                    .sz(crate::theme::fs(11.0))
                    .align(TextAlign::Right)
                    .fg(TEXT_DIM)
                    .grow(1.0);
            })
            .align_center()
            .gap(6.0);

            let fk = state.main_menu.search_filter_kind.as_str();
            let results: Vec<_> = state
                .host
                .content
                .search_results
                .iter()
                .filter(|r| fk == "all" || r.kind == fk)
                .cloned()
                .collect();

            if state.host.content.search_loading {
                ui.col(|ui| {
                    icons::icon(ui, icons::SEARCH, 28.0).fg(accent);
                    ui.txt("Searching ChilloutVR...")
                        .sz(crate::theme::fs(13.0))
                        .semibold()
                        .fg(TEXT_MUTED);
                })
                .center()
                .grow(1.0)
                .gap(6.0);
            } else if results.is_empty() {
                ui.col(|ui| {
                    icons::icon(ui, icons::SEARCH, 28.0).fg(TEXT_DIM);
                    ui.txt("No matching results")
                        .sz(crate::theme::fs(13.0))
                        .semibold()
                        .fg(TEXT_MUTED);
                })
                .center()
                .grow(1.0)
                .gap(6.0);
            } else {
                ui.scroll("mm_srch_grid", |ui| {
                    for chunk in results.chunks(4) {
                        ui.row(|ui| {
                            for r in chunk {
                                let icon_p = match r.kind.as_str() {
                                    "world" => icons::WORLD,
                                    "avatar" => icons::AVATAR,
                                    "prop" => icons::CUBE,
                                    _ => icons::PLAYERS,
                                };
                                let card = ContentCardItem {
                                    id: r.id.clone(),
                                    name: r.name.clone(),
                                    image_key: r.image_key.clone(),
                                    player_count: 0,
                                };
                                render_content_card(
                                    ui,
                                    state,
                                    &card,
                                    &r.kind,
                                    icon_p,
                                    false,
                                    accent,
                                    actions,
                                );
                            }
                            for _ in chunk.len()..4 {
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

fn render_category_btn(
    ui: &mut UiContext,
    tab_id: &str,
    cat: &ContentCategoryItem,
    list: &ContentListSnapshot,
    accent: Color,
    actions: &mut Vec<UiAction>,
) {
    let active = list.category_key == cat.key && list.is_system == cat.is_system;
    let btn_id = format!("mm_cb_cat_{}_{}_{}", tab_id, cat.is_system as u8, cat.key);
    if ui
        .btn_with(&btn_id, |ui| {
            ui.txt(&cat.name)
                .sz(crate::theme::fs(11.5))
                .semibold()
                .fg(if active { TEXT_PRIMARY } else { TEXT_MUTED })
                .ellipsis()
                .grow(1.0);
        })
        .px(9.0)
        .py(7.0)
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
        actions.push(UiAction::ContentLoadList {
            kind: String::from(tab_id),
            category_key: cat.key.clone(),
            is_system: cat.is_system,
            page: 0,
            sort_order: list.sort_order,
            ascending: list.ascending,
        });
    }
}

#[allow(clippy::too_many_arguments)]
fn render_content_card(
    ui: &mut UiContext,
    state: &mut AppState,
    item: &ContentCardItem,
    detail_kind: &str,
    fallback_icon: &str,
    show_players: bool,
    _accent: Color,
    actions: &mut Vec<UiAction>,
) {
    let btn_id = format!("mm_cb_card_{}_{}", detail_kind, item.id);
    if ui
        .btn_with(&btn_id, |ui| {
            ui.col(|ui| {
                // 16:9 Thumbnail Area
                if let Some(tex) = state.main_textures.get(&item.image_key) {
                    ui.img(tex)
                        .w_frac(1.0)
                        .h(102.0)
                        .fit(ImageFit::Cover)
                        .bevel_rounded(6.0, 1.3);
                } else {
                    ui.col(|ui| {
                        icons::icon(ui, fallback_icon, 28.0)
                            .fg(Color::rgba(148, 163, 184, 75));
                    })
                    .center()
                    .w_frac(1.0)
                    .h(102.0)
                    .bg(Color::rgba(12, 16, 24, 210))
                    .bevel_rounded(6.0, 1.3);
                }

                // Footer: Title + optional player count pill
                ui.row(|ui| {
                    ui.txt(&item.name)
                        .sz(crate::theme::fs(11.5))
                        .semibold()
                        .fg(TEXT_PRIMARY)
                        .ellipsis()
                        .grow(1.0);
                    if show_players && item.player_count > 0 {
                        let pc = format!("{}", item.player_count);
                        ui.txt(&pc)
                            .sz(crate::theme::fs(10.0))
                            .bold()
                            .fg(EMERALD)
                            .bg(tint_alpha(EMERALD, 30))
                            .px(5.0)
                            .py(1.5)
                            .bevel_rounded(4.0, 1.3);
                    }
                })
                .align_center()
                .gap(5.0)
                .w_frac(1.0);
            })
            .gap(6.0)
            .grow(1.0);
        })
        .grow(1.0)
        .p(7.0)
        .bg(card_inner_bg())
        .hover_bg(tile_idle_hover())
        .bd(1.0, border_subtle())
        .bevel_rounded(8.0, 1.3)
        .clicked()
    {
        push_click(actions);
        crate::main_menu::push_nav_history(state);
        if detail_kind == "user" {
            state.main_menu.user_detail_tab = 0;
            actions.push(UiAction::SocialLoadUserDetail {
                user_id: item.id.clone(),
            });
        } else {
            actions.push(UiAction::ContentLoadDetail {
                kind: String::from(detail_kind),
                id: item.id.clone(),
            });
        }
    }
}

fn render_detail_view(
    ui: &mut UiContext,
    state: &mut AppState,
    detail: &ContentDetailSnapshot,
    accent: Color,
    actions: &mut Vec<UiAction>,
) {
    let icon_p = match detail.kind.as_str() {
        "world" => icons::WORLD,
        "avatar" => icons::AVATAR,
        _ => icons::CUBE,
    };

    ui.col(|ui| {
        // Header bar with Back
        ui.row(|ui| {
            if ui
                .btn_with("mm_det_back", |ui| {
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

            icons::icon(ui, icon_p, 16.0).fg(accent);
            let title = if !detail.name.is_empty() {
                &detail.name
            } else if !detail.error.is_empty() {
                "Details Unavailable"
            } else {
                "Loading details..."
            };
            ui.txt(title)
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
                icons::icon(ui, icon_p, 30.0).fg(accent);
                ui.txt("Loading details...")
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
        } else if detail.kind == "avatar" || detail.kind == "prop" {
            // =================================================================
            // OVERHAULED 2-COLUMN SHOWCASE LAYOUT FOR AVATARS & PROPS
            // =================================================================
            ui.row(|ui| {
                // --- LEFT SHOWCASE COLUMN (360px): SQUARE IMAGE OR LIVE 3D STUDIO + PRIMARY CTA ---
                ui.col(|ui| {
                    if detail.kind == "avatar" && detail.has_3d_preview {
                        let avtr_target = format!("avatar:{}", detail.id);
                        ui.row(|ui| {
                            icons::icon(ui, icons::AVATAR, 14.0).fg(accent);
                            ui.txt("3D Avatar Studio")
                                .sz(crate::theme::fs(11.5))
                                .semibold()
                                .fg(TEXT_PRIMARY)
                                .grow(1.0);
                            ui.txt("Live")
                                .sz(crate::theme::fs(9.5))
                                .semibold()
                                .fg(EMERALD)
                                .px(6.0)
                                .py(2.0)
                                .bg(tint_alpha(EMERALD, 28))
                                .bevel_rounded(4.0, 1.0);
                        })
                        .align_center()
                        .gap(6.0);

                        crate::avatar_settings_view::render_avatar_turntable_stage(
                            ui,
                            state,
                            0,
                            &avtr_target,
                            "mm_avtr_det_turntable_drag",
                            "",
                            accent,
                            actions,
                        );
                    } else {
                        if let Some(tex) = state.main_textures.get(&detail.image_key) {
                            ui.img(tex)
                                .w_frac(1.0)
                                .h(340.0)
                                .fit(ImageFit::Cover)
                                .bevel_rounded(9.0, 1.6)
                                .bd(1.0, border_subtle());
                        } else {
                            ui.col(|ui| {
                                icons::icon(ui, icon_p, 48.0)
                                    .fg(Color::rgba(148, 163, 184, 75));
                            })
                            .center()
                            .w_frac(1.0)
                            .h(340.0)
                            .bg(card_inner_bg())
                            .bd(1.0, border_subtle())
                            .bevel_rounded(9.0, 1.6);
                        }

                        if detail.kind == "avatar" {
                            ui.txt("3D Studio previews avatars worn in this instance")
                                .sz(crate::theme::fs(9.5))
                                .align(TextAlign::Center)
                                .fg(TEXT_DIM)
                                .w_frac(1.0);
                        }
                        ui.col(|_| {}).grow(1.0);
                    }

                    // Primary CTA(s) pinned at bottom of left showcase
                    if !detail.can_use {
                        ui.row(|ui| {
                            ui.txt(if detail.kind == "avatar" {
                                "Private Avatar - No Access"
                            } else {
                                "Private Prop - No Access"
                            })
                            .sz(crate::theme::fs(11.5))
                            .semibold()
                            .align(TextAlign::Center)
                            .fg(TEXT_MUTED)
                            .grow(1.0);
                        })
                        .w_frac(1.0)
                        .py(10.0)
                        .bg(card_inner_bg())
                        .bd(1.0, border_subtle())
                        .bevel_rounded(7.5, 1.4);
                    } else if detail.kind == "avatar" {
                        ui.row(|ui| {
                            if ui
                                .btn_with("mm_det_wear_av", |ui| {
                                    ui.row(|ui| {
                                        icons::icon(ui, icons::CHECK, 15.0).fg(EMERALD);
                                        ui.txt(if detail.is_current_avatar {
                                            "Wearing Avatar"
                                        } else {
                                            "Wear Avatar"
                                        })
                                        .sz(crate::theme::fs(12.5))
                                        .bold()
                                        .fg(TEXT_PRIMARY);
                                    })
                                    .center()
                                    .grow(1.0)
                                    .gap(6.0);
                                })
                                .grow(1.0)
                                .py(10.0)
                                .bg(tint_alpha(EMERALD, if detail.is_current_avatar { 30 } else { 52 }))
                                .hover_bg(tint_alpha(EMERALD, 80))
                                .bd(1.5, tint_alpha(EMERALD, 195))
                                .bevel_rounded(7.5, 1.4)
                                .clicked()
                            {
                                push_click(actions);
                                actions.push(UiAction::ContentWearAvatar {
                                    avatar_id: detail.id.clone(),
                                });
                            }

                            if detail.is_current_avatar
                                && ui
                                    .btn_with("mm_det_open_aas", |ui| {
                                        ui.txt("Settings")
                                            .sz(crate::theme::fs(11.5))
                                            .semibold()
                                            .fg(TEXT_PRIMARY);
                                    })
                                    .px(12.0)
                                    .py(10.0)
                                    .bg(tile_idle_bg())
                                    .hover_bg(tile_idle_hover())
                                    .bd(1.0, border_subtle())
                                    .bevel_rounded(7.5, 1.4)
                                    .clicked()
                            {
                                push_click(actions);
                                crate::main_menu::push_nav_history(state);
                                actions.push(UiAction::ContentCloseDetail);
                                state.main_menu.active_tab = String::from("avatarSettings");
                            }
                        })
                        .gap(7.0);
                    } else {
                        if ui
                            .btn_with("mm_det_spawn_prop", |ui| {
                                ui.row(|ui| {
                                    icons::icon(ui, icons::CUBE, 15.0).fg(accent);
                                    ui.txt("Spawn Prop")
                                        .sz(crate::theme::fs(12.5))
                                        .bold()
                                        .fg(TEXT_PRIMARY);
                                })
                                .center()
                                .grow(1.0)
                                .gap(6.0);
                            })
                            .w_frac(1.0)
                            .py(10.0)
                            .bg(tint_alpha(accent, 52))
                            .hover_bg(tint_alpha(accent, 85))
                            .bd(1.5, tint_alpha(accent, 195))
                            .bevel_rounded(7.5, 1.4)
                            .clicked()
                        {
                            push_click(actions);
                            actions.push(UiAction::ContentSpawnProp {
                                prop_id: detail.id.clone(),
                            });
                        }
                        ui.txt("Place to spawn - Right-Click (Desktop) or Grip (VR) to cancel")
                            .sz(crate::theme::fs(9.5))
                            .align(TextAlign::Center)
                            .fg(TEXT_DIM)
                            .w_frac(1.0);
                    }

                    // Pin to Quick Menu button (relocated out of top bar into Actions card)
                    if !detail.id.is_empty() && detail.error.is_empty() {
                        let is_pinned = state.host.content.pinned_items.iter().any(|p| {
                            p.kind.eq_ignore_ascii_case(&detail.kind)
                                && p.id.eq_ignore_ascii_case(&detail.id)
                        });
                        let allow_pin = detail.kind == "world" || detail.can_use || is_pinned;
                        if allow_pin
                            && ui
                                .btn_with("mm_det_pin_btn", |ui| {
                                    ui.row(|ui| {
                                        icons::icon(ui, icons::PIN, 13.5)
                                            .fg(if is_pinned { accent } else { TEXT_MUTED });
                                        ui.txt(if is_pinned {
                                            "Pinned to Quick Menu"
                                        } else {
                                            "Pin to Quick Menu"
                                        })
                                        .sz(crate::theme::fs(11.0))
                                        .semibold()
                                        .fg(if is_pinned { TEXT_PRIMARY } else { TEXT_MUTED });
                                    })
                                    .center()
                                    .grow(1.0)
                                    .gap(5.0);
                                })
                                .w_frac(1.0)
                                .py(7.5)
                                .bg(if is_pinned {
                                    tint_alpha(accent, 42)
                                } else {
                                    tile_idle_bg()
                                })
                                .hover_bg(tile_idle_hover())
                                .bd(
                                    1.0,
                                    if is_pinned {
                                        tint_alpha(accent, 185)
                                    } else {
                                        border_subtle()
                                    },
                                )
                                .bevel_rounded(6.5, 1.3)
                                .clicked()
                        {
                            push_click(actions);
                            actions.push(UiAction::ContentTogglePin {
                                kind: detail.kind.clone(),
                                id: detail.id.clone(),
                            });
                        }
                    }
                })
                .w(360.0)
                .h_frac(1.0)
                .p(11.0)
                .gap(9.0)
                .bg(card_bg())
                .bd(1.0, border_subtle())
                .bevel_rounded(10.5, 2.0)
                .shadow(14.0, SHADOW_CARD);

                // --- RIGHT DETAILS COLUMN: AUTHOR + METADATA + DESCRIPTION + FAVORITES ---
                ui.col(|ui| {
                    // Author Card
                    if !detail.author_name.is_empty() {
                        if ui
                            .btn_with("mm_det_author_card", |ui| {
                                ui.row(|ui| {
                                    if let Some(tex) =
                                        state.main_textures.get(&detail.author_image_key)
                                    {
                                        ui.img(tex)
                                            .w(42.0)
                                            .h(42.0)
                                            .fit(ImageFit::Cover)
                                            .hexagon_pointy()
                                            .r(3.5)
                                            .bd(1.5, tint_alpha(accent, 185));
                                    } else {
                                        let init = detail
                                            .author_name
                                            .chars()
                                            .next()
                                            .unwrap_or('A')
                                            .to_ascii_uppercase()
                                            .to_string();
                                        ui.col(|ui| {
                                            ui.txt(&init)
                                                .sz(crate::theme::fs(15.0))
                                                .bold()
                                                .align(TextAlign::Center)
                                                .fg(accent);
                                        })
                                        .center()
                                        .w(42.0)
                                        .h(42.0)
                                        .bg(tint_alpha(accent, 28))
                                        .bd(1.5, tint_alpha(accent, 160))
                                        .hexagon_pointy()
                                        .r(3.5);
                                    }

                                    ui.col(|ui| {
                                        ui.txt("Creator").sz(crate::theme::fs(9.5)).semibold().fg(TEXT_DIM);
                                        ui.txt(&detail.author_name)
                                            .sz(crate::theme::fs(13.0))
                                            .bold()
                                            .fg(TEXT_PRIMARY)
                                            .ellipsis();
                                    })
                                    .gap(2.0)
                                    .grow(1.0);

                                    if !detail.author_id.is_empty() {
                                        ui.row(|ui| {
                                            ui.txt("View Profile")
                                                .sz(crate::theme::fs(10.0))
                                                .semibold()
                                                .fg(TEXT_MUTED);
                                            icons::icon(ui, icons::CHEVRON_RIGHT, 12.0)
                                                .fg(TEXT_DIM);
                                        })
                                        .align_center()
                                        .gap(3.0);
                                    }
                                })
                                .align_center()
                                .gap(10.0)
                                .grow(1.0);
                            })
                            .p(9.0)
                            .bg(card_inner_bg())
                            .hover_bg(tile_idle_hover())
                            .bd(1.0, border_subtle())
                            .bevel_rounded(8.5, 1.5)
                            .clicked()
                            && !detail.author_id.is_empty()
                        {
                            push_click(actions);
                            crate::main_menu::push_nav_history(state);
                            state.main_menu.user_detail_tab = 0;
                            actions.push(UiAction::ContentCloseDetail);
                            actions.push(UiAction::SocialLoadUserDetail {
                                user_id: detail.author_id.clone(),
                            });
                        }
                    }

                    // Metadata Badges Row
                    ui.row(|ui| {
                        let (vis_lbl, vis_col) = if detail.is_public {
                            ("Public", EMERALD)
                        } else if detail.is_shared {
                            ("Shared", Color::rgb(56, 189, 248))
                        } else {
                            ("Private", TEXT_MUTED)
                        };
                        ui.row(|ui| {
                            ui.txt(vis_lbl).sz(crate::theme::fs(10.0)).semibold().fg(vis_col);
                        })
                        .px(8.0)
                        .py(3.5)
                        .bg(tint_alpha(vis_col, 28))
                        .bd(1.0, tint_alpha(vis_col, 130))
                        .bevel_rounded(5.0, 1.2);

                        if !detail.file_size_text.is_empty() {
                            ui.row(|ui| {
                                ui.txt(&detail.file_size_text)
                                    .sz(crate::theme::fs(10.0))
                                    .semibold()
                                    .fg(TEXT_PRIMARY);
                            })
                            .px(8.0)
                            .py(3.5)
                            .bg(tile_idle_bg())
                            .bd(1.0, border_subtle())
                            .bevel_rounded(5.0, 1.2);
                        }

                        if !detail.uploaded_at.is_empty() {
                            ui.row(|ui| {
                                ui.txt(&format!("Uploaded {}", detail.uploaded_at))
                                    .sz(crate::theme::fs(10.0))
                                    .fg(TEXT_MUTED);
                            })
                            .px(8.0)
                            .py(3.5)
                            .bg(tile_idle_bg())
                            .bd(1.0, border_subtle())
                            .bevel_rounded(5.0, 1.2);
                        }

                        if !detail.updated_at.is_empty() && detail.updated_at != detail.uploaded_at {
                            ui.row(|ui| {
                                ui.txt(&format!("Updated {}", detail.updated_at))
                                    .sz(crate::theme::fs(10.0))
                                    .fg(TEXT_MUTED);
                            })
                            .px(8.0)
                            .py(3.5)
                            .bg(tile_idle_bg())
                            .bd(1.0, border_subtle())
                            .bevel_rounded(5.0, 1.2);
                        }

                        for plat in detail.platforms.iter().take(2) {
                            ui.row(|ui| {
                                ui.txt(plat).sz(crate::theme::fs(10.0)).semibold().fg(accent);
                            })
                            .px(8.0)
                            .py(3.5)
                            .bg(tint_alpha(accent, 24))
                            .bd(1.0, tint_alpha(accent, 110))
                            .bevel_rounded(5.0, 1.2);
                        }
                    })
                    .align_center()
                    .gap(6.0);

                    // Description Card
                    ui.col(|ui| {
                        ui.txt("Description")
                            .sz(crate::theme::fs(10.5))
                            .semibold()
                            .fg(TEXT_DIM);
                        ui.scroll("mm_det_desc_scroll", |ui| {
                            let desc = if detail.description.trim().is_empty() {
                                "No description provided."
                            } else {
                                &detail.description
                            };
                            ui.txt(desc).sz(crate::theme::fs(12.0)).fg(TEXT_MUTED).wrap();
                        })
                        .grow(1.0)
                        .no_scrollbar();
                    })
                    .grow(1.0)
                    .p(10.0)
                    .gap(6.0)
                    .bg(card_inner_bg())
                    .bd(1.0, border_subtle())
                    .bevel_rounded(8.5, 1.5);

                    // Favorites & Lists Card
                    let user_cats: Vec<(String, String)> = match detail.kind.as_str() {
                        "avatar" => &state.host.content.avatar_categories,
                        _ => &state.host.content.prop_categories,
                    }
                    .iter()
                    .filter(|c| !c.is_system)
                    .map(|c| (c.key.clone(), c.name.clone()))
                    .collect();

                    if !user_cats.is_empty() {
                        ui.col(|ui| {
                            ui.row(|ui| {
                                icons::icon(ui, icons::STAR, 12.0).fg(AMBER);
                                ui.txt("Favorites & Lists")
                                    .sz(crate::theme::fs(10.5))
                                    .semibold()
                                    .fg(TEXT_MUTED);
                            })
                            .align_center()
                            .gap(5.0);

                            for row_cats in user_cats.chunks(4) {
                                ui.row(|ui| {
                                    for (ckey, cname) in row_cats {
                                        let assigned = detail
                                            .categories
                                            .iter()
                                            .any(|k| k.eq_ignore_ascii_case(ckey));
                                        if ui
                                            .btn_with(&format!("mm_det_cat_{ckey}"), |ui| {
                                                ui.txt(cname)
                                                    .sz(crate::theme::fs(10.5))
                                                    .semibold()
                                                    .align(TextAlign::Center)
                                                    .fg(if assigned {
                                                        TEXT_PRIMARY
                                                    } else {
                                                        TEXT_MUTED
                                                    })
                                                    .ellipsis()
                                                    .grow(1.0);
                                            })
                                            .grow(1.0)
                                            .px(8.0)
                                            .py(5.5)
                                            .bg(if assigned {
                                                tint_alpha(AMBER, 48)
                                            } else {
                                                tile_idle_bg()
                                            })
                                            .hover_bg(tile_idle_hover())
                                            .bd(
                                                1.0,
                                                if assigned {
                                                    tint_alpha(AMBER, 185)
                                                } else {
                                                    border_subtle()
                                                },
                                            )
                                            .bevel_rounded(5.5, 1.3)
                                            .clicked()
                                        {
                                            push_click(actions);
                                            actions.push(UiAction::ContentToggleCategory {
                                                kind: detail.kind.clone(),
                                                id: detail.id.clone(),
                                                category_key: ckey.clone(),
                                            });
                                        }
                                    }
                                    for _ in row_cats.len()..4 {
                                        ui.col(|_| {}).grow(1.0);
                                    }
                                })
                                .gap(6.0);
                            }
                        })
                        .p(10.0)
                        .gap(7.0)
                        .bg(card_inner_bg())
                        .bd(1.0, border_subtle())
                        .bevel_rounded(8.5, 1.5);
                    }
                })
                .grow(1.0)
                .h_frac(1.0)
                .p(12.0)
                .gap(9.0)
                .bg(card_bg())
                .bd(1.0, border_subtle())
                .bevel_rounded(10.5, 2.0)
                .shadow(14.0, SHADOW_CARD);
            })
            .grow(1.0)
            .gap(9.0);
        } else {
            ui.row(|ui| {
                // Optional 3D Turntable Column when browsing an avatar already loaded in the instance
                if detail.kind == "avatar" && detail.has_3d_preview {
                    let avtr_target = format!("avatar:{}", detail.id);
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
                            &avtr_target,
                            "mm_avtr_det_turntable_drag",
                            "",
                            accent,
                            actions,
                        );
                    })
                    .w(242.0)
                    .p(10.0)
                    .gap(7.0)
                    .bg(card_bg())
                    .bd(1.0, border_subtle())
                    .bevel_rounded(10.5, 2.0)
                    .shadow(14.0, SHADOW_CARD);
                }

                // Left: Banner (or 360 World Pano) + Description + Primary Actions
                ui.col(|ui| {
                    let has_pano = detail.kind == "world" && !detail.pano_url.is_empty();
                    let use_pano = has_pano && state.main_menu.world_pano_enabled;
                    if use_pano {
                        let [vw, vh] = ui.viewport_size();
                        let pano_target = format!("worldpano:{}", detail.pano_url);
                        let pano_resp = ui
                            .col(|ui| {
                                ui.row(|ui| {
                                    ui.txt("360° World Preview - Drag to look")
                                        .sz(crate::theme::fs(9.5))
                                        .semibold()
                                        .fg(TEXT_MUTED)
                                        .grow(1.0);
                                    if ui
                                        .btn_with("mm_det_pano_toggle_2d", |ui| {
                                            ui.txt("2D Image").sz(crate::theme::fs(9.5)).semibold().fg(TEXT_PRIMARY);
                                        })
                                        .px(7.0)
                                        .py(2.5)
                                        .bg(Color::rgba(11, 15, 24, 210))
                                        .hover_bg(tile_idle_hover())
                                        .bd(1.0, border_subtle())
                                        .bevel_rounded(4.5, 1.2)
                                        .clicked()
                                    {
                                        push_click(actions);
                                        state.main_menu.world_pano_enabled = false;
                                    }
                                })
                                .align_center()
                                .gap(6.0);

                                ui.txt("").grow(1.0);
                            })
                            .w_frac(1.0)
                            .h(220.0)
                            .p(7.0)
                            .bg_radial(Color::rgb(26, 36, 54), Color::rgb(10, 14, 22))
                            .bd(1.0, tint_alpha(accent, 110))
                            .bevel_rounded(8.0, 1.3);

                        state.main_menu.aas_preview_rendered = true;
                        if let Some(r) = pano_resp.prev_rect() {
                            let stage_r = lumenic::Rect::new(
                                r.x + 2.0,
                                r.y + 26.0,
                                (r.w - 4.0).max(1.0),
                                (r.h - 28.0).max(1.0),
                            );
                            state.main_menu.aas_preview_rect =
                                Some([stage_r.x, stage_r.y, stage_r.w, stage_r.h]);

                            let norm = [
                                (stage_r.x / vw.max(1.0)).clamp(0.0, 1.0),
                                (stage_r.y / vh.max(1.0)).clamp(0.0, 1.0),
                                (stage_r.w / vw.max(1.0)).clamp(0.0, 1.0),
                                (stage_r.h / vh.max(1.0)).clamp(0.0, 1.0),
                            ];
                            let rect_changed = state
                                .main_menu
                                .aas_last_preview_sync
                                .map(|prev| {
                                    prev.iter()
                                        .zip(norm.iter())
                                        .any(|(a, b)| (a - b).abs() > 0.002)
                                })
                                .unwrap_or(true);
                            let target_changed = state.main_menu.aas_preview_target != pano_target;
                            if rect_changed || target_changed {
                                state.main_menu.aas_last_preview_sync = Some(norm);
                                state.main_menu.aas_preview_target = pano_target.clone();
                                actions.push(UiAction::SyncAvatarPreview {
                                    surface_id: 0,
                                    active: true,
                                    target: pano_target,
                                    x: norm[0],
                                    y: norm[1],
                                    w: norm[2],
                                    h: norm[3],
                                });
                            }

                            let g = ui.interact(ui.id("mm_world_pano_drag"), stage_r);
                            if g.dragging {
                                state.main_menu.aas_preview_dragging = true;
                                actions.push(UiAction::WorldPanoLook {
                                    delta_x: g.delta[0] / stage_r.w.max(1.0),
                                    delta_y: g.delta[1] / stage_r.h.max(1.0),
                                    is_dragging: true,
                                });
                            } else if state.main_menu.aas_preview_dragging {
                                state.main_menu.aas_preview_dragging = false;
                                actions.push(UiAction::WorldPanoLook {
                                    delta_x: 0.0,
                                    delta_y: 0.0,
                                    is_dragging: false,
                                });
                            }
                        }
                    } else {
                        if let Some(tex) = state.main_textures.get(&detail.image_key) {
                            ui.img(tex)
                                .w_frac(1.0)
                                .h(220.0)
                                .fit(ImageFit::Cover)
                                .bevel_rounded(8.0, 1.3);
                        } else {
                            ui.col(|ui| {
                                icons::icon(ui, icon_p, 42.0)
                                    .fg(Color::rgba(148, 163, 184, 75));
                            })
                            .center()
                            .w_frac(1.0)
                            .h(220.0)
                            .bg(card_inner_bg())
                            .bevel_rounded(8.0, 1.3);
                        }
                        if has_pano {
                            if ui
                                .btn_with("mm_det_pano_toggle_360", |ui| {
                                    ui.txt("Open 360° World Preview")
                                        .sz(crate::theme::fs(10.5))
                                        .semibold()
                                        .fg(TEXT_PRIMARY)
                                        .align(TextAlign::Center)
                                        .grow(1.0);
                                })
                                .w_frac(1.0)
                                .py(5.0)
                                .bg(tint_alpha(accent, 38))
                                .hover_bg(tint_alpha(accent, 68))
                                .bd(1.0, tint_alpha(accent, 150))
                                .bevel_rounded(5.5, 1.2)
                                .clicked()
                            {
                                push_click(actions);
                                state.main_menu.world_pano_enabled = true;
                            }
                        }
                    }

                    // Creator card for World
                    if !detail.author_name.is_empty() {
                        if ui
                            .btn_with("mm_det_world_author_card", |ui| {
                                ui.row(|ui| {
                                    if let Some(tex) =
                                        state.main_textures.get(&detail.author_image_key)
                                    {
                                        ui.img(tex)
                                            .w(36.0)
                                            .h(36.0)
                                            .fit(ImageFit::Cover)
                                            .hexagon_pointy()
                                            .r(3.0)
                                            .bd(1.5, tint_alpha(accent, 175));
                                    } else {
                                        let init = detail
                                            .author_name
                                            .chars()
                                            .next()
                                            .unwrap_or('C')
                                            .to_ascii_uppercase()
                                            .to_string();
                                        ui.col(|ui| {
                                            ui.txt(&init)
                                                .sz(crate::theme::fs(13.5))
                                                .bold()
                                                .align(TextAlign::Center)
                                                .fg(accent);
                                        })
                                        .center()
                                        .w(36.0)
                                        .h(36.0)
                                        .bg(tint_alpha(accent, 28))
                                        .bd(1.5, tint_alpha(accent, 155))
                                        .hexagon_pointy()
                                        .r(3.0);
                                    }

                                    ui.col(|ui| {
                                        ui.txt("Creator").sz(crate::theme::fs(9.0)).semibold().fg(TEXT_DIM);
                                        ui.txt(&detail.author_name)
                                            .sz(crate::theme::fs(12.0))
                                            .bold()
                                            .fg(TEXT_PRIMARY)
                                            .ellipsis();
                                    })
                                    .gap(1.0)
                                    .grow(1.0);

                                    if !detail.author_id.is_empty() {
                                        ui.row(|ui| {
                                            ui.txt("View Profile")
                                                .sz(crate::theme::fs(10.0))
                                                .semibold()
                                                .fg(TEXT_MUTED);
                                            icons::icon(ui, icons::CHEVRON_RIGHT, 12.0)
                                                .fg(TEXT_DIM);
                                        })
                                        .align_center()
                                        .gap(3.0);
                                    }
                                })
                                .align_center()
                                .gap(9.0)
                                .grow(1.0);
                            })
                            .p(7.5)
                            .bg(card_inner_bg())
                            .hover_bg(tile_idle_hover())
                            .bd(1.0, border_subtle())
                            .bevel_rounded(7.5, 1.3)
                            .clicked()
                            && !detail.author_id.is_empty()
                        {
                            push_click(actions);
                            crate::main_menu::push_nav_history(state);
                            state.main_menu.user_detail_tab = 0;
                            actions.push(UiAction::ContentCloseDetail);
                            actions.push(UiAction::SocialLoadUserDetail {
                                user_id: detail.author_id.clone(),
                            });
                        }
                    }

                    ui.scroll("mm_det_desc_scroll", |ui| {
                        let desc = if detail.description.trim().is_empty() {
                            "No description provided."
                        } else {
                            &detail.description
                        };
                        ui.txt(desc).sz(crate::theme::fs(12.0)).fg(TEXT_MUTED).wrap();
                    })
                    .grow(1.0)
                    .no_scrollbar();

                    // User categories / Favorites assignment strip
                    let user_cats: Vec<(String, String)> = match detail.kind.as_str() {
                        "avatar" => &state.host.content.avatar_categories,
                        "prop" => &state.host.content.prop_categories,
                        _ => &state.host.content.world_categories,
                    }
                    .iter()
                    .filter(|c| !c.is_system)
                    .map(|c| (c.key.clone(), c.name.clone()))
                    .collect();

                    if !user_cats.is_empty() {
                        ui.row(|ui| {
                            icons::icon(ui, icons::STAR, 12.0).fg(AMBER);
                            for (ckey, cname) in user_cats.iter().take(6) {
                                let assigned = detail
                                    .categories
                                    .iter()
                                    .any(|k| k.eq_ignore_ascii_case(ckey));
                                if ui
                                    .btn_with(&format!("mm_det_cat_{ckey}"), |ui| {
                                        ui.txt(cname)
                                            .sz(crate::theme::fs(10.0))
                                            .semibold()
                                            .fg(if assigned { TEXT_PRIMARY } else { TEXT_MUTED })
                                            .ellipsis();
                                    })
                                    .px(8.0)
                                    .py(4.0)
                                    .bg(if assigned {
                                        tint_alpha(AMBER, 48)
                                    } else {
                                        tile_idle_bg()
                                    })
                                    .hover_bg(tile_idle_hover())
                                    .bd(
                                        1.0,
                                        if assigned {
                                            tint_alpha(AMBER, 185)
                                        } else {
                                            border_subtle()
                                        },
                                    )
                                    .bevel_rounded(5.0, 1.3)
                                    .clicked()
                                {
                                    push_click(actions);
                                    actions.push(UiAction::ContentToggleCategory {
                                        kind: detail.kind.clone(),
                                        id: detail.id.clone(),
                                        category_key: ckey.clone(),
                                    });
                                }
                            }
                        })
                        .align_center()
                        .gap(5.0);
                    }

                    // Primary action buttons
                    ui.row(|ui| {
                        match detail.kind.as_str() {
                            "avatar" => {
                                if ui
                                    .btn_with("mm_det_wear_av", |ui| {
                                        ui.row(|ui| {
                                            icons::icon(ui, icons::CHECK, 15.0).fg(EMERALD);
                                            ui.txt("Wear Avatar")
                                                .sz(crate::theme::fs(12.5))
                                                .bold()
                                                .fg(TEXT_PRIMARY);
                                        })
                                        .center()
                                        .grow(1.0)
                                        .gap(6.0);
                                    })
                                    .grow(1.0)
                                    .py(10.0)
                                    .bg(tint_alpha(EMERALD, 50))
                                    .hover_bg(tint_alpha(EMERALD, 80))
                                    .bd(1.5, tint_alpha(EMERALD, 195))
                                    .bevel_rounded(7.0, 1.3)
                                    .clicked()
                                {
                                    push_click(actions);
                                    actions.push(UiAction::ContentWearAvatar {
                                        avatar_id: detail.id.clone(),
                                    });
                                }
                            }
                            "prop" => {
                                if ui
                                    .btn_with("mm_det_spawn_prop", |ui| {
                                        ui.row(|ui| {
                                            icons::icon(ui, icons::CUBE, 15.0).fg(accent);
                                            ui.txt("Spawn Prop")
                                                .sz(crate::theme::fs(12.5))
                                                .bold()
                                                .fg(TEXT_PRIMARY);
                                        })
                                        .center()
                                        .grow(1.0)
                                        .gap(6.0);
                                    })
                                    .grow(1.0)
                                    .py(10.0)
                                    .bg(tint_alpha(accent, 52))
                                    .hover_bg(tint_alpha(accent, 85))
                                    .bd(1.5, tint_alpha(accent, 195))
                                    .bevel_rounded(7.0, 1.3)
                                    .clicked()
                                {
                                    push_click(actions);
                                    actions.push(UiAction::ContentSpawnProp {
                                        prop_id: detail.id.clone(),
                                    });
                                }
                            }
                            _ => {
                                if ui
                                    .btn_with("mm_det_set_home", |ui| {
                                        ui.row(|ui| {
                                            icons::icon(ui, icons::HOME, 15.0).fg(accent);
                                            ui.txt("Set as Home World")
                                                .sz(crate::theme::fs(12.0))
                                                .semibold()
                                                .fg(TEXT_PRIMARY);
                                        })
                                        .center()
                                        .grow(1.0)
                                        .gap(6.0);
                                    })
                                    .grow(1.0)
                                    .py(9.0)
                                    .bg(tile_idle_bg())
                                    .hover_bg(tile_idle_hover())
                                    .bd(1.0, border_subtle())
                                    .bevel_rounded(7.0, 1.3)
                                    .clicked()
                                {
                                    push_click(actions);
                                    actions.push(UiAction::ContentSetHomeWorld {
                                        world_id: detail.id.clone(),
                                    });
                                }

                                if !detail.id.is_empty() && detail.error.is_empty() {
                                    let is_pinned = state.host.content.pinned_items.iter().any(|p| {
                                        p.kind.eq_ignore_ascii_case("world")
                                            && p.id.eq_ignore_ascii_case(&detail.id)
                                    });
                                    if ui
                                        .btn_with("mm_det_world_pin_btn", |ui| {
                                            ui.row(|ui| {
                                                icons::icon(ui, icons::PIN, 14.0).fg(if is_pinned {
                                                    accent
                                                } else {
                                                    TEXT_MUTED
                                                });
                                                ui.txt(if is_pinned {
                                                    "Pinned to QM"
                                                } else {
                                                    "Pin to QM"
                                                })
                                                .sz(crate::theme::fs(12.0))
                                                .semibold()
                                                .fg(if is_pinned {
                                                    TEXT_PRIMARY
                                                } else {
                                                    TEXT_MUTED
                                                });
                                            })
                                            .center()
                                            .grow(1.0)
                                            .gap(6.0);
                                        })
                                        .grow(1.0)
                                        .py(9.0)
                                        .bg(if is_pinned {
                                            tint_alpha(accent, 42)
                                        } else {
                                            tile_idle_bg()
                                        })
                                        .hover_bg(tile_idle_hover())
                                        .bd(
                                            1.0,
                                            if is_pinned {
                                                tint_alpha(accent, 185)
                                            } else {
                                                border_subtle()
                                            },
                                        )
                                        .bevel_rounded(7.0, 1.3)
                                        .clicked()
                                    {
                                        push_click(actions);
                                        actions.push(UiAction::ContentTogglePin {
                                            kind: String::from("world"),
                                            id: detail.id.clone(),
                                        });
                                    }
                                }
                            }
                        }
                    })
                    .gap(8.0);
                })
                .grow(1.05)
                .h_frac(1.0)
                .p(12.0)
                .gap(10.0)
                .bg(card_bg())
                .bd(1.0, border_subtle())
                .bevel_rounded(10.5, 2.0)
                .shadow(14.0, SHADOW_CARD);

                // Right panel for Worlds: Active Instances + Create New Instance
                if detail.kind == "world" {
                    ui.col(|ui| {
                        // Create Instance Card
                        ui.col(|ui| {
                            ui.txt("Create New Instance")
                                .sz(crate::theme::fs(13.0))
                                .bold()
                                .fg(TEXT_PRIMARY);

                            ui.row(|ui| {
                                for (val, lbl) in [
                                    ("Public", "Public"),
                                    ("FriendsOfFriends", "Friends+"),
                                    ("Friends", "Friends"),
                                    ("EveryoneCanInvite", "Invite+"),
                                    ("OwnerMustInvite", "Invite"),
                                ] {
                                    let active = state.main_menu.create_inst_privacy == val;
                                    if ui
                                        .btn_with(&format!("mm_ci_p_{val}"), |ui| {
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
                                        .bevel_rounded(5.0, 1.3)
                                        .clicked()
                                    {
                                        push_click(actions);
                                        state.main_menu.create_inst_privacy = String::from(val);
                                    }
                                }
                            })
                            .gap(4.0);

                            // Group privacy row
                            ui.row(|ui| {
                                for (val, lbl) in [
                                    ("Group", "Group"),
                                    ("GroupPlus", "Group+"),
                                    ("GroupPublic", "Group Public"),
                                ] {
                                    let active = state.main_menu.create_inst_privacy == val;
                                    if ui
                                        .btn_with(&format!("mm_ci_p_{val}"), |ui| {
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
                                        .bevel_rounded(5.0, 1.3)
                                        .clicked()
                                    {
                                        push_click(actions);
                                        state.main_menu.create_inst_privacy = String::from(val);
                                    }
                                }
                            })
                            .gap(4.0);

                            let is_group_privacy = matches!(
                                state.main_menu.create_inst_privacy.as_str(),
                                "Group" | "GroupPlus" | "GroupPublic"
                            );

                            if is_group_privacy {
                                let my_groups = state.host.social.my_groups.clone();
                                if !my_groups.is_empty()
                                    && !my_groups
                                        .iter()
                                        .any(|g| g.id == state.main_menu.create_inst_group_id)
                                {
                                    state.main_menu.create_inst_group_id = my_groups[0].id.clone();
                                }

                                if my_groups.is_empty() {
                                    ui.txt("You are not in any groups.")
                                        .sz(crate::theme::fs(10.5))
                                        .fg(AMBER);
                                } else {
                                    ui.hscroll("mm_ci_grp_pick", |ui| {
                                        for g in &my_groups {
                                            let active =
                                                state.main_menu.create_inst_group_id == g.id;
                                            let lbl = if g.tag.is_empty() {
                                                g.name.clone()
                                            } else {
                                                format!("[{}] {}", g.tag, g.name)
                                            };
                                            if ui
                                                .btn_with(&format!("mm_ci_g_{}", g.id), |ui| {
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
                                                .py(4.5)
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
                                                state.main_menu.create_inst_group_id = g.id.clone();
                                            }
                                        }
                                    })
                                    .gap(4.0)
                                    .no_scrollbar();
                                }
                            }

                            ui.row(|ui| {
                                for (val, lbl) in [("Eu", "Europe"), ("Us", "US"), ("As", "Asia")] {
                                    let active = state.main_menu.create_inst_region == val;
                                    if ui
                                        .btn_with(&format!("mm_ci_r_{val}"), |ui| {
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
                                        .bevel_rounded(5.0, 1.3)
                                        .clicked()
                                    {
                                        push_click(actions);
                                        state.main_menu.create_inst_region = String::from(val);
                                    }
                                }

                                let creating = state.host.content.creating_instance;
                                let missing_group = is_group_privacy
                                    && state.main_menu.create_inst_group_id.is_empty();
                                let disabled = creating || missing_group;
                                if ui
                                    .btn_with("mm_ci_create", |ui| {
                                        ui.txt(if creating {
                                            "Creating..."
                                        } else {
                                            "Create & Join"
                                        })
                                        .sz(crate::theme::fs(11.0))
                                        .bold()
                                        .align(TextAlign::Center)
                                        .fg(TEXT_PRIMARY)
                                        .grow(1.0);
                                    })
                                    .grow(1.4)
                                    .py(6.0)
                                    .bg(if creating {
                                        tint_alpha(AMBER, 45)
                                    } else {
                                        tint_alpha(EMERALD, 52)
                                    })
                                    .hover_bg(if disabled {
                                        tint_alpha(AMBER, 45)
                                    } else {
                                        tint_alpha(EMERALD, 85)
                                    })
                                    .bd(
                                        1.5,
                                        if creating {
                                            tint_alpha(AMBER, 180)
                                        } else {
                                            tint_alpha(EMERALD, 195)
                                        },
                                    )
                                    .bevel_rounded(6.0, 1.3)
                                    .opacity(if disabled { 0.55 } else { 1.0 })
                                    .clicked()
                                    && !disabled
                                {
                                    push_click(actions);
                                    actions.push(UiAction::ContentCreateInstance {
                                        world_id: detail.id.clone(),
                                        privacy: state.main_menu.create_inst_privacy.clone(),
                                        region: state.main_menu.create_inst_region.clone(),
                                        group_id: if is_group_privacy {
                                            state.main_menu.create_inst_group_id.clone()
                                        } else {
                                            String::new()
                                        },
                                    });
                                }
                            })
                            .align_center()
                            .gap(5.0);

                            if !state.host.content.create_instance_error.is_empty() {
                                ui.txt(&state.host.content.create_instance_error)
                                    .sz(crate::theme::fs(10.5))
                                    .fg(AMBER)
                                    .wrap();
                            }
                        })
                        .p(11.0)
                        .gap(7.0)
                        .bg(card_bg())
                        .bd(1.0, border_subtle())
                        .bevel_rounded(10.5, 2.0)
                        .shadow(14.0, SHADOW_CARD);

                        // Active Instances List
                        ui.col(|ui| {
                            ui.txt(&format!("Active Instances ({})", detail.instances.len()))
                                .sz(crate::theme::fs(13.0))
                                .bold()
                                .fg(TEXT_PRIMARY);

                            if detail.instances.is_empty() {
                                ui.col(|ui| {
                                    ui.txt("No active public/friend instances right now.")
                                        .sz(crate::theme::fs(11.5))
                                        .fg(TEXT_DIM);
                                })
                                .center()
                                .grow(1.0);
                            } else {
                                ui.scroll("mm_det_inst_scroll", |ui| {
                                    for inst in &detail.instances {
                                        ui.row(|ui| {
                                            // Main clickable card body -> opens Instance Details
                                            if ui
                                                .btn_with(&format!("mm_det_inst_card_{}", inst.id), |ui| {
                                                    ui.col(|ui| {
                                                        ui.txt(&inst.name)
                                                            .sz(crate::theme::fs(12.0))
                                                            .semibold()
                                                            .fg(TEXT_PRIMARY)
                                                            .ellipsis();
                                                        let meta = format!(
                                                            "{} / {} players  |  {}",
                                                            inst.player_count,
                                                            inst.max_players.max(inst.player_count),
                                                            if inst.region.is_empty() {
                                                                "Global"
                                                            } else {
                                                                &inst.region
                                                            }
                                                        );
                                                        ui.txt(&meta)
                                                            .sz(crate::theme::fs(10.5))
                                                            .fg(TEXT_MUTED);

                                                        // Bottom player preview strip
                                                        if !inst.members.is_empty() || inst.player_count > 0 {
                                                            ui.row(|ui| {
                                                                let shown = inst.members.len().min(3);
                                                                for m in inst.members.iter().take(shown) {
                                                                    ui.row(|ui| {
                                                                        if let Some(tex) =
                                                                            state.main_textures.get(&m.image_key)
                                                                        {
                                                                            ui.img(tex)
                                                                                .w(18.0)
                                                                                .h(18.0)
                                                                                .fit(ImageFit::Cover)
                                                                                .hexagon_pointy()
                                                                                .r(2.0);
                                                                        } else {
                                                                            let init = m
                                                                                .name
                                                                                .chars()
                                                                                .next()
                                                                                .unwrap_or('?')
                                                                                .to_ascii_uppercase()
                                                                                .to_string();
                                                                            ui.col(|ui| {
                                                                                ui.txt(&init)
                                                                                    .sz(crate::theme::fs(8.5))
                                                                                    .bold()
                                                                                    .align(TextAlign::Center)
                                                                                    .fg(accent);
                                                                            })
                                                                            .center()
                                                                            .w(18.0)
                                                                            .h(18.0)
                                                                            .bg(tint_alpha(accent, 30))
                                                                            .hexagon_pointy()
                                                                            .r(2.0);
                                                                        }
                                                                        ui.txt(&m.name)
                                                                            .sz(crate::theme::fs(9.5))
                                                                            .fg(TEXT_MUTED)
                                                                            .max_w(86.0)
                                                                            .ellipsis();
                                                                    })
                                                                    .align_center()
                                                                    .gap(4.0)
                                                                    .px(5.0)
                                                                    .py(2.0)
                                                                    .bg(tile_idle_bg())
                                                                    .bd(1.0, border_subtle())
                                                                    .bevel_rounded(4.5, 1.0);
                                                                }

                                                                let remaining =
                                                                    inst.player_count.saturating_sub(shown as i32);
                                                                if remaining > 0 {
                                                                    let more_txt = if shown == 0 {
                                                                        format!("{} in instance", remaining)
                                                                    } else {
                                                                        format!("+{} more", remaining)
                                                                    };
                                                                    ui.txt(&more_txt)
                                                                        .sz(crate::theme::fs(9.5))
                                                                        .semibold()
                                                                        .fg(accent)
                                                                        .px(6.0)
                                                                        .py(2.0)
                                                                        .bg(tint_alpha(accent, 24))
                                                                        .bd(1.0, tint_alpha(accent, 95))
                                                                        .bevel_rounded(4.5, 1.0);
                                                                }
                                                            })
                                                            .align_center()
                                                            .gap(4.0)
                                                            .pt(3.0);
                                                        }
                                                    })
                                                    .gap(2.5)
                                                    .grow(1.0);
                                                })
                                                .grow(1.0)
                                                .p(8.0)
                                                .bg(Color::TRANSPARENT)
                                                .hover_bg(tile_idle_hover())
                                                .borderless()
                                                .bevel_rounded(6.0, 1.2)
                                                .clicked()
                                            {
                                                push_click(actions);
                                                crate::main_menu::push_nav_history(state);
                                                actions.push(UiAction::ContentLoadInstanceDetail {
                                                    instance_id: inst.id.clone(),
                                                });
                                            }

                                            ui.col(|ui| {
                                                if ui
                                                    .btn_with(&format!("mm_det_join_{}", inst.id), |ui| {
                                                        ui.txt("Join")
                                                            .sz(crate::theme::fs(11.0))
                                                            .bold()
                                                            .align(TextAlign::Center)
                                                            .fg(TEXT_PRIMARY)
                                                            .grow(1.0);
                                                    })
                                                    .w(68.0)
                                                    .py(5.5)
                                                    .bg(tint_alpha(accent, 52))
                                                    .hover_bg(tint_alpha(accent, 85))
                                                    .bd(1.0, tint_alpha(accent, 195))
                                                    .bevel_rounded(6.0, 1.3)
                                                    .clicked()
                                                {
                                                    push_click(actions);
                                                    actions.push(UiAction::ContentJoinInstance {
                                                        instance_id: inst.id.clone(),
                                                    });
                                                }

                                                if ui
                                                    .btn_with(&format!("mm_det_portal_{}", inst.id), |ui| {
                                                        ui.txt("Portal")
                                                            .sz(crate::theme::fs(10.5))
                                                            .semibold()
                                                            .align(TextAlign::Center)
                                                            .fg(TEXT_PRIMARY)
                                                            .grow(1.0);
                                                    })
                                                    .w(68.0)
                                                    .py(5.0)
                                                    .bg(tile_idle_bg())
                                                    .hover_bg(tile_idle_hover())
                                                    .bd(1.0, border_subtle())
                                                    .bevel_rounded(6.0, 1.3)
                                                    .clicked()
                                                {
                                                    push_click(actions);
                                                    actions.push(UiAction::ContentDropPortal {
                                                        instance_id: inst.id.clone(),
                                                    });
                                                }
                                            })
                                            .gap(5.0);
                                        })
                                        .align_center()
                                        .p(4.0)
                                        .pr(8.0)
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
                        .grow(1.0)
                        .p(11.0)
                        .gap(8.0)
                        .bg(card_bg())
                        .bd(1.0, border_subtle())
                        .bevel_rounded(10.5, 2.0)
                        .shadow(14.0, SHADOW_CARD);
                    })
                    .grow(1.15)
                    .h_frac(1.0)
                    .gap(9.0);
                }
            })
            .grow(1.0)
            .gap(10.0);
        }
    })
    .grow(1.0)
    .gap(9.0);
}

fn instance_privacy_color(privacy: &str, accent: Color) -> Color {
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

fn render_instance_detail_view(
    ui: &mut UiContext,
    state: &mut AppState,
    inst: &InstanceDetailSnapshot,
    accent: Color,
    actions: &mut Vec<UiAction>,
) {
    let priv_col = instance_privacy_color(&inst.privacy, accent);
    let shown_players = inst.player_count.max(inst.members.len() as i32);
    let max_players = inst.max_players.max(shown_players);

    ui.col(|ui| {
        // Header bar
        ui.row(|ui| {
            if ui
                .btn_with("mm_id_back", |ui| {
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

            icons::icon(ui, icons::INSTANCE, 17.0).fg(accent);
            ui.col(|ui| {
                let title = if inst.name.is_empty() {
                    "Instance Details"
                } else {
                    &inst.name
                };
                ui.txt(title).sz(crate::theme::fs(14.5)).bold().fg(TEXT_PRIMARY).ellipsis();
                if !inst.world_name.is_empty() {
                    ui.txt(&inst.world_name)
                        .sz(crate::theme::fs(10.5))
                        .fg(TEXT_MUTED)
                        .ellipsis();
                }
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

        if inst.loading {
            ui.col(|ui| {
                icons::icon(ui, icons::INSTANCE, 28.0).fg(accent);
                ui.txt("Loading instance details...")
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
            return;
        }

        if !inst.error.is_empty() {
            ui.col(|ui| {
                ui.txt(&inst.error).sz(crate::theme::fs(12.5)).fg(AMBER).wrap();
            })
            .center()
            .grow(1.0)
            .p(16.0)
            .bg(card_bg())
            .bd(1.0, border_subtle())
            .bevel_rounded(10.5, 2.0)
            .shadow(14.0, SHADOW_CARD);
            return;
        }

        // Body: Left = World & Instance Overview + Actions | Right = Players in Instance
        ui.row(|ui| {
            // --- LEFT COLUMN: INSTANCE & WORLD OVERVIEW ---
            ui.col(|ui| {
                ui.scroll("mm_id_overview_scroll", |ui| {
                    // World hero banner
                    if let Some(tex) = state.main_textures.get(&inst.world_image_key) {
                        ui.img(tex)
                            .w_frac(1.0)
                            .h(152.0)
                            .fit(ImageFit::Cover)
                            .bevel_rounded(8.0, 1.5)
                            .bd(1.0, border_subtle());
                    } else {
                        ui.col(|ui| {
                            icons::icon(ui, icons::WORLD, 32.0).fg(TEXT_DIM);
                        })
                        .center()
                        .w_frac(1.0)
                        .h(152.0)
                        .bg(card_inner_bg())
                        .bd(1.0, border_subtle())
                        .bevel_rounded(8.0, 1.5);
                    }

                    // World info + View World button
                    ui.row(|ui| {
                        ui.col(|ui| {
                            let wname = if inst.world_name.is_empty() {
                                "Unknown World"
                            } else {
                                &inst.world_name
                            };
                            ui.txt(wname)
                                .sz(crate::theme::fs(14.0))
                                .bold()
                                .fg(TEXT_PRIMARY)
                                .ellipsis();
                            if !inst.world_author_name.is_empty() {
                                if ui
                                    .btn_with("mm_id_world_author", |ui| {
                                        ui.txt(&format!("by {}", inst.world_author_name))
                                            .sz(crate::theme::fs(10.5))
                                            .fg(TEXT_MUTED)
                                            .ellipsis();
                                    })
                                    .p(0.0)
                                    .bg(Color::TRANSPARENT)
                                    .hover_bg(Color::TRANSPARENT)
                                    .borderless()
                                    .clicked()
                                    && !inst.world_author_id.is_empty()
                                {
                                    push_click(actions);
                                    crate::main_menu::push_nav_history(state);
                                    state.main_menu.user_detail_tab = 0;
                                    actions.push(UiAction::ContentCloseInstanceDetail);
                                    actions.push(UiAction::ContentCloseDetail);
                                    actions.push(UiAction::SocialLoadUserDetail {
                                        user_id: inst.world_author_id.clone(),
                                    });
                                }
                            }
                        })
                        .gap(2.0)
                        .grow(1.0);

                        if !inst.world_id.is_empty()
                            && ui
                                .btn_with("mm_id_open_world", |ui| {
                                    ui.row(|ui| {
                                        icons::icon(ui, icons::WORLD, 12.5).fg(TEXT_PRIMARY);
                                        ui.txt("World Details")
                                            .sz(crate::theme::fs(10.5))
                                            .semibold()
                                            .fg(TEXT_PRIMARY);
                                    })
                                    .center()
                                    .gap(5.0);
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
                            crate::main_menu::push_nav_history(state);
                            actions.push(UiAction::ContentCloseInstanceDetail);
                            actions.push(UiAction::ContentLoadDetail {
                                kind: String::from("world"),
                                id: inst.world_id.clone(),
                            });
                        }
                    })
                    .align_center()
                    .gap(8.0);

                    // Metadata chips row (Privacy / Region / Players / Game Mode)
                    ui.row(|ui| {
                        if !inst.privacy.is_empty() {
                            ui.row(|ui| {
                                ui.txt(&inst.privacy).sz(crate::theme::fs(10.5)).semibold().fg(priv_col);
                            })
                            .px(8.0)
                            .py(4.5)
                            .bg(tint_alpha(priv_col, 28))
                            .bd(1.0, tint_alpha(priv_col, 130))
                            .bevel_rounded(5.5, 1.2);
                        }
                        if !inst.region.is_empty() {
                            ui.row(|ui| {
                                ui.txt(&inst.region).sz(crate::theme::fs(10.5)).semibold().fg(TEXT_PRIMARY);
                            })
                            .px(8.0)
                            .py(4.5)
                            .bg(card_inner_bg())
                            .bd(1.0, border_subtle())
                            .bevel_rounded(5.5, 1.2);
                        }
                        let cnt = format!("{shown_players} / {max_players} Players");
                        ui.row(|ui| {
                            ui.txt(&cnt).sz(crate::theme::fs(10.5)).semibold().fg(EMERALD);
                        })
                        .px(8.0)
                        .py(4.5)
                        .bg(tint_alpha(EMERALD, 28))
                        .bd(1.0, tint_alpha(EMERALD, 130))
                        .bevel_rounded(5.5, 1.2);
                    })
                    .align_center()
                    .gap(6.0);

                    if !inst.game_mode_name.is_empty() {
                        ui.row(|ui| {
                            ui.txt("Game Mode").sz(crate::theme::fs(10.5)).fg(TEXT_DIM).grow(1.0);
                            ui.txt(&inst.game_mode_name)
                                .sz(crate::theme::fs(11.0))
                                .semibold()
                                .fg(TEXT_PRIMARY);
                        })
                        .align_center()
                        .px(10.0)
                        .py(6.5)
                        .bg(card_inner_bg())
                        .bd(1.0, border_subtle())
                        .bevel_rounded(6.5, 1.3);
                    }

                    // Instance Host / Owner card
                    if !inst.owner_name.is_empty() {
                        if ui
                            .btn_with("mm_id_owner_card", |ui| {
                                ui.row(|ui| {
                                    if let Some(tex) = state.main_textures.get(&inst.owner_image_key) {
                                        ui.img(tex)
                                            .w(38.0)
                                            .h(38.0)
                                            .fit(ImageFit::Cover)
                                            .hexagon_pointy()
                                            .r(3.0)
                                            .bd(1.5, tint_alpha(accent, 175));
                                    } else {
                                        let init = inst
                                            .owner_name
                                            .chars()
                                            .next()
                                            .unwrap_or('H')
                                            .to_ascii_uppercase()
                                            .to_string();
                                        ui.col(|ui| {
                                            ui.txt(&init)
                                                .sz(crate::theme::fs(14.0))
                                                .bold()
                                                .align(TextAlign::Center)
                                                .fg(accent);
                                        })
                                        .center()
                                        .w(38.0)
                                        .h(38.0)
                                        .bg(tint_alpha(accent, 28))
                                        .bd(1.5, tint_alpha(accent, 155))
                                        .hexagon_pointy()
                                        .r(3.0);
                                    }

                                    ui.col(|ui| {
                                        ui.txt("Instance Host")
                                            .sz(crate::theme::fs(9.5))
                                            .semibold()
                                            .fg(TEXT_DIM);
                                        ui.row(|ui| {
                                            ui.txt(&inst.owner_name)
                                                .sz(crate::theme::fs(12.0))
                                                .semibold()
                                                .fg(TEXT_PRIMARY)
                                                .ellipsis()
                                                .grow(1.0);
                                            if !inst.owner_rank.is_empty() {
                                                ui.txt(&inst.owner_rank)
                                                    .sz(crate::theme::fs(9.5))
                                                    .fg(TEXT_MUTED);
                                            }
                                        })
                                        .align_center()
                                        .gap(6.0);
                                    })
                                    .gap(2.0)
                                    .grow(1.0);
                                })
                                .align_center()
                                .gap(9.0)
                                .grow(1.0);
                            })
                            .p(8.0)
                            .bg(card_inner_bg())
                            .hover_bg(tile_idle_hover())
                            .bd(1.0, border_subtle())
                            .bevel_rounded(7.5, 1.3)
                            .clicked()
                            && !inst.owner_id.is_empty()
                        {
                            push_click(actions);
                            crate::main_menu::push_nav_history(state);
                            state.main_menu.user_detail_tab = 0;
                            actions.push(UiAction::ContentCloseInstanceDetail);
                            actions.push(UiAction::ContentCloseDetail);
                            actions.push(UiAction::SocialLoadUserDetail {
                                user_id: inst.owner_id.clone(),
                            });
                        }
                    }

                    // Group card (if group instance)
                    if !inst.group_id.is_empty() && !inst.group_name.is_empty() {
                        if ui
                            .btn_with("mm_id_group_card", |ui| {
                                ui.row(|ui| {
                                    if let Some(tex) = state.main_textures.get(&inst.group_image_key) {
                                        ui.img(tex)
                                            .w(36.0)
                                            .h(36.0)
                                            .fit(ImageFit::Cover)
                                            .bevel_rounded(6.0, 1.2)
                                            .bd(1.0, border_subtle());
                                    } else {
                                        icons::icon(ui, icons::PLAYERS, 20.0).fg(accent);
                                    }
                                    ui.col(|ui| {
                                        ui.txt(if inst.group_verified {
                                            "Verified Group Instance"
                                        } else {
                                            "Group Instance"
                                        })
                                        .sz(crate::theme::fs(9.5))
                                        .semibold()
                                        .fg(Color::rgb(56, 189, 248));
                                        ui.txt(&inst.group_name)
                                            .sz(crate::theme::fs(12.0))
                                            .semibold()
                                            .fg(TEXT_PRIMARY)
                                            .ellipsis();
                                    })
                                    .gap(2.0)
                                    .grow(1.0);
                                })
                                .align_center()
                                .gap(9.0)
                                .grow(1.0);
                            })
                            .p(8.0)
                            .bg(card_inner_bg())
                            .hover_bg(tile_idle_hover())
                            .bd(1.0, tint_alpha(Color::rgb(56, 189, 248), 95))
                            .bevel_rounded(7.5, 1.3)
                            .clicked()
                        {
                            push_click(actions);
                            crate::main_menu::push_nav_history(state);
                            actions.push(UiAction::ContentCloseInstanceDetail);
                            actions.push(UiAction::ContentCloseDetail);
                            actions.push(UiAction::SocialLoadGroupDetail {
                                group_id: inst.group_id.clone(),
                            });
                            state.main_menu.active_tab = String::from("friends");
                        }
                    }

                    if !inst.can_join && !inst.cannot_join_reason.is_empty() {
                        ui.row(|ui| {
                            ui.txt(&inst.cannot_join_reason)
                                .sz(crate::theme::fs(11.0))
                                .fg(AMBER)
                                .wrap()
                                .grow(1.0);
                        })
                        .p(8.0)
                        .bg(tint_alpha(AMBER, 26))
                        .bd(1.0, tint_alpha(AMBER, 130))
                        .bevel_rounded(7.0, 1.3);
                    }
                })
                .grow(1.0)
                .gap(8.0)
                .no_scrollbar();

                // Primary Action Buttons pinned at bottom of overview card
                ui.row(|ui| {
                    if ui
                        .btn_with("mm_id_join", |ui| {
                            ui.txt(if inst.can_join {
                                "Join Instance"
                            } else {
                                "Cannot Join"
                            })
                            .sz(crate::theme::fs(12.0))
                            .bold()
                            .align(TextAlign::Center)
                            .fg(if inst.can_join { TEXT_PRIMARY } else { TEXT_DIM })
                            .grow(1.0);
                        })
                        .grow(1.0)
                        .py(9.0)
                        .bg(if inst.can_join {
                            tint_alpha(EMERALD, 52)
                        } else {
                            tile_idle_bg()
                        })
                        .hover_bg(if inst.can_join {
                            tint_alpha(EMERALD, 85)
                        } else {
                            tile_idle_bg()
                        })
                        .bd(
                            1.5,
                            if inst.can_join {
                                tint_alpha(EMERALD, 195)
                            } else {
                                border_subtle()
                            },
                        )
                        .bevel_rounded(7.0, 1.4)
                        .opacity(if inst.can_join { 1.0 } else { 0.55 })
                        .clicked()
                        && inst.can_join
                    {
                        push_click(actions);
                        actions.push(UiAction::ContentJoinInstance {
                            instance_id: inst.id.clone(),
                        });
                    }

                    if ui
                        .btn_with("mm_id_portal", |ui| {
                            ui.txt("Drop Portal")
                                .sz(crate::theme::fs(11.5))
                                .semibold()
                                .align(TextAlign::Center)
                                .fg(TEXT_PRIMARY)
                                .grow(1.0);
                        })
                        .px(14.0)
                        .py(9.0)
                        .bg(tile_idle_bg())
                        .hover_bg(tile_idle_hover())
                        .bd(1.0, border_subtle())
                        .bevel_rounded(7.0, 1.4)
                        .clicked()
                    {
                        push_click(actions);
                        actions.push(UiAction::ContentDropPortal {
                            instance_id: inst.id.clone(),
                        });
                    }
                })
                .gap(8.0);
            })
            .w(378.0)
            .h_frac(1.0)
            .p(11.0)
            .gap(8.0)
            .bg(card_bg())
            .bd(1.0, border_subtle())
            .bevel_rounded(10.5, 2.0)
            .shadow(14.0, SHADOW_CARD);

            // --- RIGHT COLUMN: PLAYERS IN INSTANCE ---
            ui.col(|ui| {
                ui.row(|ui| {
                    icons::icon(ui, icons::PLAYERS, 15.0).fg(accent);
                    ui.txt(&format!("Players in Instance ({})", inst.members.len()))
                        .sz(crate::theme::fs(12.5))
                        .bold()
                        .fg(TEXT_PRIMARY)
                        .grow(1.0);
                })
                .align_center()
                .gap(6.0);

                if inst.members.is_empty() {
                    ui.col(|ui| {
                        icons::icon(ui, icons::PLAYERS, 28.0).fg(TEXT_DIM);
                        ui.txt("No visible players in this instance.")
                            .sz(crate::theme::fs(12.5))
                            .semibold()
                            .fg(TEXT_MUTED);
                    })
                    .center()
                    .grow(1.0)
                    .gap(6.0);
                } else {
                    ui.scroll("mm_id_members_scroll", |ui| {
                        for chunk in inst.members.chunks(2) {
                            ui.row(|ui| {
                                for m in chunk {
                                    if ui
                                        .btn_with(&format!("mm_id_m_{}", m.id), |ui| {
                                            ui.row(|ui| {
                                                if let Some(tex) =
                                                    state.main_textures.get(&m.image_key)
                                                {
                                                    ui.img(tex)
                                                        .w(40.0)
                                                        .h(40.0)
                                                        .fit(ImageFit::Cover)
                                                        .hexagon_pointy()
                                                        .r(3.5)
                                                        .bd(1.5, tint_alpha(accent, 160));
                                                } else {
                                                    let init = m
                                                        .name
                                                        .chars()
                                                        .next()
                                                        .unwrap_or('P')
                                                        .to_ascii_uppercase()
                                                        .to_string();
                                                    ui.col(|ui| {
                                                        ui.txt(&init)
                                                            .sz(crate::theme::fs(15.0))
                                                            .bold()
                                                            .align(TextAlign::Center)
                                                            .fg(accent);
                                                    })
                                                    .center()
                                                    .w(40.0)
                                                    .h(40.0)
                                                    .bg(tint_alpha(accent, 30))
                                                    .bd(1.5, tint_alpha(accent, 160))
                                                    .hexagon_pointy()
                                                    .r(3.5);
                                                }
                                                ui.txt(&m.name)
                                                    .sz(crate::theme::fs(12.5))
                                                    .semibold()
                                                    .fg(TEXT_PRIMARY)
                                                    .ellipsis()
                                                    .grow(1.0);
                                            })
                                            .align_center()
                                            .gap(10.0)
                                            .grow(1.0);
                                        })
                                        .grow(1.0)
                                        .p(8.5)
                                        .bg(card_inner_bg())
                                        .hover_bg(tile_idle_hover())
                                        .bd(1.0, border_subtle())
                                        .bevel_rounded(8.0, 1.3)
                                        .clicked()
                                    {
                                        push_click(actions);
                                        crate::main_menu::push_nav_history(state);
                                        state.main_menu.user_detail_tab = 0;
                                        actions.push(UiAction::ContentCloseInstanceDetail);
                                        actions.push(UiAction::ContentCloseDetail);
                                        actions.push(UiAction::SocialLoadUserDetail {
                                            user_id: m.id.clone(),
                                        });
                                    }
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
            })
            .grow(1.0)
            .h_frac(1.0)
            .p(11.0)
            .gap(8.0)
            .bg(card_bg())
            .bd(1.0, border_subtle())
            .bevel_rounded(10.5, 2.0)
            .shadow(14.0, SHADOW_CARD);
        })
        .grow(1.0)
        .gap(9.0);
    })
    .grow(1.0)
    .gap(9.0);
}
