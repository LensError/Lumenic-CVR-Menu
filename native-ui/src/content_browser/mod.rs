mod detail;

use detail::{render_detail_view, render_instance_detail_view};

use lumenic::{Color, ImageFit, TextAlign, UiContext};

use crate::icons;
use crate::state::{
    AppState, ContentCardItem, ContentCategoryItem, ContentListSnapshot, UiAction,
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
    if let Some(inst) = state.host.content.instance_detail.take() {
        render_instance_detail_view(ui, state, &inst, accent, actions);
        state.host.content.instance_detail = Some(inst);
        return;
    }
    if let Some(detail) = state.host.content.detail.take() {
        render_detail_view(ui, state, &detail, accent, actions);
        state.host.content.detail = Some(detail);
        return;
    }
    if let Some(gd) = state.host.social.group_detail.take() {
        crate::social_view::render_group_detail_view(ui, state, &gd, accent, actions);
        state.host.social.group_detail = Some(gd);
        return;
    }
    if let Some(user_detail) = state.host.social.user_detail.take() {
        crate::social_view::render_user_detail_view(ui, state, &user_detail, accent, actions);
        state.host.social.user_detail = Some(user_detail);
        return;
    }

    let (categories, list): (Vec<ContentCategoryItem>, ContentListSnapshot) = match tab_id {
        "worlds" => (
            std::mem::take(&mut state.host.content.world_categories),
            std::mem::take(&mut state.host.content.worlds),
        ),
        "avatars" => (
            std::mem::take(&mut state.host.content.avatar_categories),
            std::mem::take(&mut state.host.content.avatars),
        ),
        _ => (
            std::mem::take(&mut state.host.content.prop_categories),
            std::mem::take(&mut state.host.content.props),
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

    match tab_id {
        "worlds" => {
            state.host.content.world_categories = categories;
            state.host.content.worlds = list;
        }
        "avatars" => {
            state.host.content.avatar_categories = categories;
            state.host.content.avatars = list;
        }
        _ => {
            state.host.content.prop_categories = categories;
            state.host.content.props = list;
        }
    }
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

