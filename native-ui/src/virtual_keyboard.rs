use lumenic::TextEditState;

use crate::state::{AppState, ExtTextInputSlot, UiAction};

/// Routes a submitted ChilloutVR virtual keyboard value to the target Rust `TextEditState`
/// and emits any follow-up actions (search submit, profile save, extension submit).
pub fn commit_virtual_keyboard(
    state: &mut AppState,
    actions: &mut Vec<UiAction>,
    field_id: &str,
    text: &str,
) {
    match field_id {
        "mm_search_input" | "mm_srch_page_input" => {
            state.main_menu.search_input = TextEditState::with_text(text);
            state.main_menu.last_search = String::from(text);
            let trimmed = text.trim();
            if !trimmed.is_empty() {
                if state.main_menu.active_tab != "search" {
                    crate::main_menu::push_nav_history(state);
                }
                state.main_menu.active_tab = String::from("search");
                state.main_menu.search_filter_kind = String::from("all");
                crate::main_menu::close_all_detail_overlays(actions);
                actions.push(UiAction::ContentGlobalSearch {
                    query: String::from(trimmed),
                });
            }
        }
        "mm_cb_filter" => {
            state.main_menu.browser_local_filter = TextEditState::with_text(text);
        }
        "mm_fr_search" => {
            state.main_menu.friends_search_input = TextEditState::with_text(text);
        }
        "mm_set_search" => {
            state.main_menu.settings_search_input = TextEditState::with_text(text);
        }
        "mm_uix_cat_search" => {
            state.main_menu.uix_search_input = TextEditState::with_text(text);
        }
        "mm_grp_name_input" => {
            state.main_menu.group_name_input = TextEditState::with_text(text);
        }
        "mm_grp_desc_input" => {
            state.main_menu.group_desc_input = TextEditState::with_text(text);
        }
        "mm_gd_member_search" => {
            state.main_menu.group_member_search = TextEditState::with_text(text);
        }
        "mm_prof_pronouns" => {
            let clipped: String = text.trim().chars().take(32).collect();
            state.main_menu.profile_pronouns_input = TextEditState::with_text(&clipped);
        }
        "mm_prof_bio" => {
            let clipped: String = text.trim().chars().take(512).collect();
            state.main_menu.profile_bio_input = TextEditState::with_text(&clipped);
        }
        "mm_aas_new_profile" => {
            let trimmed: String = text.trim().chars().take(12).collect();
            if !trimmed.is_empty() && !trimmed.eq_ignore_ascii_case("default") {
                actions.push(UiAction::AvatarSaveProfile { profile: trimmed });
            }
            state.main_menu.aas_new_profile_input = TextEditState::new();
        }
        other if other.starts_with("ext|") => {
            // Format: ext|<surface>|<ext_id>|<target_id>|<node_id>
            // Uses '|' instead of ':' because hub ext_ids contain ':' (e.g. "hub:lumenic.mods").
            let parts: Vec<&str> = other.splitn(5, '|').collect();
            if parts.len() == 5 {
                let widget_id = format!("ext_{}_{}_{}_{}", parts[1], parts[2], parts[3], parts[4]);
                state
                    .ext_text_inputs
                    .insert(widget_id, ExtTextInputSlot::new(text));
                actions.push(UiAction::ExtEvent {
                    ext_id: parts[2].to_owned(),
                    target_id: parts[3].to_owned(),
                    node_id: parts[4].to_owned(),
                    kind: String::from("submit"),
                    string_val: text.to_owned(),
                    float_val: 0.0,
                    bool_val: false,
                });
            }
        }
        _ => {}
    }
}
