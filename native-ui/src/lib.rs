pub mod avatar_settings_view;
pub mod content_browser;
pub mod icons;
pub mod main_menu;
pub mod quick_menu;
pub mod settings_view;
pub mod social_view;
pub mod state;
pub mod theme;
pub mod uilib_view;
pub mod uix_view;

pub const ABI_VERSION: u32 = 4;

pub const SURFACE_MAIN: u32 = 0;
pub const SURFACE_QUICK: u32 = 1;

pub const STEP_FLAG_RENDERED: u32 = 1 << 0;
pub const STEP_FLAG_TEXT_FOCUSED: u32 = 1 << 1;
pub const STEP_FLAG_POINTER_OVER_UI: u32 = 1 << 2;
pub const STEP_FLAG_HAS_ACTIONS: u32 = 1 << 3;
pub const STEP_FLAG_ERROR_OR_PANIC: u32 = 1 << 31;

pub const MOD_SHIFT: u32 = 1 << 0;
pub const MOD_CTRL: u32 = 1 << 1;
pub const MOD_ALT: u32 = 1 << 2;
pub const MOD_SUPER: u32 = 1 << 3;

// Logical key IDs shared between C# VkKeyMapper and Rust FFI.
pub const KEY_TAB: u32 = 1;
pub const KEY_ENTER: u32 = 2;
pub const KEY_SPACE: u32 = 3;
pub const KEY_ESCAPE: u32 = 4;
pub const KEY_BACKSPACE: u32 = 5;
pub const KEY_DELETE: u32 = 6;
pub const KEY_INSERT: u32 = 7;
pub const KEY_LEFT: u32 = 8;
pub const KEY_RIGHT: u32 = 9;
pub const KEY_UP: u32 = 10;
pub const KEY_DOWN: u32 = 11;
pub const KEY_HOME: u32 = 12;
pub const KEY_END: u32 = 13;
pub const KEY_PAGE_UP: u32 = 14;
pub const KEY_PAGE_DOWN: u32 = 15;
pub const KEY_F1: u32 = 16;
pub const KEY_F2: u32 = 17;
pub const KEY_F3: u32 = 18;
pub const KEY_F4: u32 = 19;
pub const KEY_F5: u32 = 20;
pub const KEY_F6: u32 = 21;
pub const KEY_F7: u32 = 22;
pub const KEY_F8: u32 = 23;
pub const KEY_F9: u32 = 24;
pub const KEY_F10: u32 = 25;
pub const KEY_F11: u32 = 26;
pub const KEY_F12: u32 = 27;
// Base offset for Unicode scalar values used in Ctrl/Alt shortcut chords (KEY_CHAR_BASE + ch as u32).
pub const KEY_CHAR_BASE: u32 = 0x1000;

#[cfg(not(target_arch = "wasm32"))]
mod native_ffi {
    use super::*;
    use std::cell::RefCell;
    use std::panic::{catch_unwind, AssertUnwindSafe};
    use std::slice;

    use lumenic::{
        Color, Key, Modifiers, OffscreenHost, OffscreenSurfaceConfig, OffscreenSurfaceId,
        PointerButton,
    };

    use crate::main_menu::render_main_menu;
    use crate::quick_menu::render_quick_menu;
    use crate::state::{AppState, HostStateSnapshot, UiAction};

thread_local! {
    static LAST_ERROR: RefCell<String> = const { RefCell::new(String::new()) };
}

fn set_last_error(msg: impl Into<String>) {
    let s = msg.into();
    LAST_ERROR.with(|cell| {
        *cell.borrow_mut() = s;
    });
}

fn clear_last_error() {
    LAST_ERROR.with(|cell| {
        cell.borrow_mut().clear();
    });
}

fn panic_payload_to_string(payload: Box<dyn std::any::Any + Send>) -> String {
    if let Some(s) = payload.downcast_ref::<&str>() {
        format!("Rust panic: {s}")
    } else if let Some(s) = payload.downcast_ref::<String>() {
        format!("Rust panic: {s}")
    } else {
        String::from("Rust panic: non-string payload")
    }
}

fn decode_modifiers(flags: u32) -> Modifiers {
    Modifiers {
        shift: (flags & MOD_SHIFT) != 0,
        ctrl: (flags & MOD_CTRL) != 0,
        alt: (flags & MOD_ALT) != 0,
        super_key: (flags & MOD_SUPER) != 0,
    }
}

fn decode_key(key_code: u32) -> Option<Key> {
    if key_code >= KEY_CHAR_BASE {
        return char::from_u32(key_code - KEY_CHAR_BASE).map(Key::Char);
    }
    match key_code {
        KEY_TAB => Some(Key::Tab),
        KEY_ENTER => Some(Key::Enter),
        KEY_SPACE => Some(Key::Space),
        KEY_ESCAPE => Some(Key::Escape),
        KEY_BACKSPACE => Some(Key::Backspace),
        KEY_DELETE => Some(Key::Delete),
        KEY_INSERT => Some(Key::Insert),
        KEY_LEFT => Some(Key::Left),
        KEY_RIGHT => Some(Key::Right),
        KEY_UP => Some(Key::Up),
        KEY_DOWN => Some(Key::Down),
        KEY_HOME => Some(Key::Home),
        KEY_END => Some(Key::End),
        KEY_PAGE_UP => Some(Key::PageUp),
        KEY_PAGE_DOWN => Some(Key::PageDown),
        KEY_F1 => Some(Key::F1),
        KEY_F2 => Some(Key::F2),
        KEY_F3 => Some(Key::F3),
        KEY_F4 => Some(Key::F4),
        KEY_F5 => Some(Key::F5),
        KEY_F6 => Some(Key::F6),
        KEY_F7 => Some(Key::F7),
        KEY_F8 => Some(Key::F8),
        KEY_F9 => Some(Key::F9),
        KEY_F10 => Some(Key::F10),
        KEY_F11 => Some(Key::F11),
        KEY_F12 => Some(Key::F12),
        _ => None,
    }
}

fn decode_button(button: u32) -> PointerButton {
    match button {
        1 => PointerButton::Right,
        2 => PointerButton::Middle,
        _ => PointerButton::Left,
    }
}

unsafe fn checked_slice<'a>(ptr: *const u8, len: usize) -> Result<&'a [u8], String> {
    if ptr.is_null() {
        return Err(String::from("null buffer pointer"));
    }
    if len > isize::MAX as usize {
        return Err(format!("buffer length exceeds isize::MAX: {len}"));
    }
    Ok(unsafe { slice::from_raw_parts(ptr, len) })
}

unsafe fn checked_slice_mut<'a>(ptr: *mut u8, len: usize) -> Result<&'a mut [u8], String> {
    if ptr.is_null() {
        return Err(String::from("null mutable buffer pointer"));
    }
    if len > isize::MAX as usize {
        return Err(format!("mutable buffer length exceeds isize::MAX: {len}"));
    }
    Ok(unsafe { slice::from_raw_parts_mut(ptr, len) })
}

/// RAII guard guaranteeing the WGL/EGL context is detached on both normal return and panic unwind.
struct MakeNotCurrentGuard<'a>(&'a mut OffscreenHost);

impl Drop for MakeNotCurrentGuard<'_> {
    fn drop(&mut self) {
        let _ = self.0.make_not_current();
    }
}

/// Tracks click-and-drag scrolling per surface so dragging pans scroll containers
/// and cancels underlying button clicks once pointer movement exceeds the drag threshold.
#[derive(Clone, Copy, Debug, Default)]
struct PointerDragState {
    is_left_down: bool,
    press_pos: [f32; 2],
    last_pos: [f32; 2],
    last_glow_pos: [f32; 2],
    is_dragging_scroll: bool,
    ignore_drag: bool,
}

/// Opaque native host owned by C# via pointer and strictly accessed on Unity's main thread.
pub struct HostHandle {
    host: OffscreenHost,
    main_surface: OffscreenSurfaceId,
    quick_surface: OffscreenSurfaceId,
    // Accumulates same-frame UTF-8 text input so multiple OnCharEvent calls never overwrite each other.
    main_pending_text: String,
    quick_pending_text: String,
    main_drag: PointerDragState,
    quick_drag: PointerDragState,
    main_last_hovered: Option<lumenic::WidgetId>,
    quick_last_hovered: Option<lumenic::WidgetId>,
    state: AppState,
    actions: Vec<UiAction>,
    actions_scratch: Vec<u8>,
}

impl HostHandle {
    fn surface_id(&self, surface_id: u32) -> Option<OffscreenSurfaceId> {
        match surface_id {
            SURFACE_MAIN => Some(self.main_surface),
            SURFACE_QUICK => Some(self.quick_surface),
            _ => None,
        }
    }

    fn pending_text_mut(&mut self, surface_id: u32) -> Option<&mut String> {
        match surface_id {
            SURFACE_MAIN => Some(&mut self.main_pending_text),
            SURFACE_QUICK => Some(&mut self.quick_pending_text),
            _ => None,
        }
    }

    fn drag_state_mut(&mut self, surface_id: u32) -> Option<&mut PointerDragState> {
        match surface_id {
            SURFACE_MAIN => Some(&mut self.main_drag),
            SURFACE_QUICK => Some(&mut self.quick_drag),
            _ => None,
        }
    }

    fn last_hovered_mut(&mut self, surface_id: u32) -> Option<&mut Option<lumenic::WidgetId>> {
        match surface_id {
            SURFACE_MAIN => Some(&mut self.main_last_hovered),
            SURFACE_QUICK => Some(&mut self.quick_last_hovered),
            _ => None,
        }
    }

    fn mark_all_dirty(&mut self) {
        if let Some(s) = self.host.surface_mut(self.main_surface) {
            s.mark_dirty();
        }
        if let Some(s) = self.host.surface_mut(self.quick_surface) {
            s.mark_dirty();
        }
    }
}

#[no_mangle]
pub extern "C" fn lumenic_cvr_abi_version() -> u32 {
    ABI_VERSION
}

#[no_mangle]
pub unsafe extern "C" fn lumenic_cvr_last_error(dst_ptr: *mut u8, dst_cap: usize) -> usize {
    let result = catch_unwind(AssertUnwindSafe(|| {
        LAST_ERROR.with(|cell| {
            let s = cell.borrow();
            let bytes = s.as_bytes();
            if !dst_ptr.is_null() && dst_cap > 0 && !bytes.is_empty() {
                let n = bytes.len().min(dst_cap);
                unsafe {
                    std::ptr::copy_nonoverlapping(bytes.as_ptr(), dst_ptr, n);
                }
            }
            bytes.len()
        })
    }));
    result.unwrap_or(0)
}

#[no_mangle]
pub extern "C" fn lumenic_cvr_create(
    mm_w: u32,
    mm_h: u32,
    qm_w: u32,
    qm_h: u32,
    scale: f32,
) -> *mut HostHandle {
    clear_last_error();
    let res = catch_unwind(AssertUnwindSafe(|| -> Result<*mut HostHandle, String> {
        let mut host = OffscreenHost::new().map_err(|e| format!("OffscreenHost::new: {e}"))?;
        let (main_surface, quick_surface) = {
            let guard = MakeNotCurrentGuard(&mut host);
            let safe_scale = if scale.is_finite() && scale > 0.0 {
                scale
            } else {
                1.0
            };

            let mm_cfg = OffscreenSurfaceConfig::new(mm_w.max(1), mm_h.max(1))
                .scale_factor(safe_scale)
                .clear_color(Color::TRANSPARENT);
            let qm_cfg = OffscreenSurfaceConfig::new(qm_w.max(1), qm_h.max(1))
                .scale_factor(safe_scale)
                .clear_color(Color::TRANSPARENT);

            let main_surface = guard
                .0
                .create_surface(mm_cfg)
                .map_err(|e| format!("create_surface(main): {e}"))?;
            let quick_surface = guard
                .0
                .create_surface(qm_cfg)
                .map_err(|e| format!("create_surface(quick): {e}"))?;

            // Disable Lumenic's built-in F12 HUD toggle so Steam screenshots never trigger it.
            if let Some(s) = guard.0.surface_mut(main_surface) {
                s.ui_mut().set_hud_hotkey_enabled(false);
                s.ui_mut().set_hud_visible(false);
            }
            if let Some(s) = guard.0.surface_mut(quick_surface) {
                s.ui_mut().set_hud_hotkey_enabled(false);
                s.ui_mut().set_hud_visible(false);
            }

            (main_surface, quick_surface)
        };

        let mut app_state = AppState::default();
        app_state.host.mm_width = mm_w.max(1);
        app_state.host.mm_height = mm_h.max(1);
        app_state.host.qm_width = qm_w.max(1);
        app_state.host.qm_height = qm_h.max(1);

        let handle = Box::new(HostHandle {
            host,
            main_surface,
            quick_surface,
            main_pending_text: String::new(),
            quick_pending_text: String::new(),
            main_drag: PointerDragState::default(),
            quick_drag: PointerDragState::default(),
            main_last_hovered: None,
            quick_last_hovered: None,
            state: app_state,
            actions: Vec::new(),
            actions_scratch: Vec::new(),
        });
        Ok(Box::into_raw(handle))
    }));

    match res {
        Ok(Ok(ptr)) => ptr,
        Ok(Err(err)) => {
            set_last_error(err);
            std::ptr::null_mut()
        }
        Err(payload) => {
            set_last_error(panic_payload_to_string(payload));
            std::ptr::null_mut()
        }
    }
}

#[no_mangle]
pub unsafe extern "C" fn lumenic_cvr_destroy(handle: *mut HostHandle) {
    if handle.is_null() {
        return;
    }
    if let Err(payload) = catch_unwind(AssertUnwindSafe(|| {
        let boxed = unsafe { Box::from_raw(handle) };
        let _ = boxed.host.make_not_current();
        drop(boxed);
    })) {
        set_last_error(panic_payload_to_string(payload));
    }
}

#[no_mangle]
pub unsafe extern "C" fn lumenic_cvr_resize(
    handle: *mut HostHandle,
    surface_id: u32,
    w: u32,
    h: u32,
    scale: f32,
) -> bool {
    clear_last_error();
    let Some(hnd) = (unsafe { handle.as_mut() }) else {
        set_last_error("lumenic_cvr_resize: null handle");
        return false;
    };
    let res = catch_unwind(AssertUnwindSafe(|| -> Result<(), String> {
        let Some(sid) = hnd.surface_id(surface_id) else {
            return Err(format!("invalid surface_id: {surface_id}"));
        };
        let safe_scale = if scale.is_finite() && scale > 0.0 {
            scale
        } else {
            1.0
        };
        {
            let guard = MakeNotCurrentGuard(&mut hnd.host);
            guard
                .0
                .resize_surface(sid, w.max(1), h.max(1), safe_scale)
                .map_err(|e| format!("resize_surface: {e}"))?;
        }
        if surface_id == SURFACE_MAIN {
            hnd.state.host.mm_width = w.max(1);
            hnd.state.host.mm_height = h.max(1);
        } else {
            hnd.state.host.qm_width = w.max(1);
            hnd.state.host.qm_height = h.max(1);
        }
        Ok(())
    }));

    match res {
        Ok(Ok(())) => true,
        Ok(Err(err)) => {
            set_last_error(err);
            false
        }
        Err(payload) => {
            set_last_error(panic_payload_to_string(payload));
            false
        }
    }
}

#[no_mangle]
pub unsafe extern "C" fn lumenic_cvr_pointer_move(
    handle: *mut HostHandle,
    surface_id: u32,
    x: f32,
    y: f32,
) {
    let Some(hnd) = (unsafe { handle.as_mut() }) else {
        return;
    };
    if !x.is_finite() || !y.is_finite() {
        return;
    }
    if let Err(payload) = catch_unwind(AssertUnwindSafe(|| {
        let Some(sid) = hnd.surface_id(surface_id) else {
            return;
        };
        let drag_snapshot = hnd.drag_state_mut(surface_id).copied().unwrap_or_default();
        let mut updated_drag = drag_snapshot;
        let tab_strip_rect = if surface_id == SURFACE_QUICK {
            hnd.state.quick.tab_strip_rect
        } else {
            None
        };
        if let Some(surf) = hnd.host.surface_mut(sid) {
            let scale = surf.scale_factor().max(0.01);
            let lx = x / scale;
            let ly = y / scale;
            // Gate background hex-glow redraws (wider deadzone in VR so hand-laser jitter
            // does not force 60 Hz full-surface CPU readbacks).
            let glow_threshold = if hnd.state.host.is_vr { 10.0 } else { 2.0 };
            let glow_moved = (lx - updated_drag.last_glow_pos[0])
                .hypot(ly - updated_drag.last_glow_pos[1])
                > glow_threshold;
            surf.ui_mut().on_pointer_move(lx, ly);
            if glow_moved {
                updated_drag.last_glow_pos = [lx, ly];
                surf.mark_dirty();
            }

            if updated_drag.is_left_down && !updated_drag.ignore_drag {
                let total_dx = lx - updated_drag.press_pos[0];
                let total_dy = ly - updated_drag.press_pos[1];
                let in_tab_strip = tab_strip_rect.is_some_and(|[rx, ry, rw, rh]| {
                    let r = lumenic::Rect::new(rx, ry, rw, rh);
                    r.contains(updated_drag.press_pos[0], updated_drag.press_pos[1])
                });
                let drag_threshold = if hnd.state.host.is_vr { 14.0 } else { 8.0 };
                let wants_scroll = if in_tab_strip {
                    total_dx.abs() > drag_threshold && total_dx.abs() > total_dy.abs() * 1.15
                } else {
                    total_dx.hypot(total_dy) > drag_threshold
                };

                if !updated_drag.is_dragging_scroll && wants_scroll {
                    updated_drag.is_dragging_scroll = true;
                    // Cancel the underlying button press so releasing after a drag-scroll does not trigger a click.
                    surf.ui_mut().on_pointer_cancel();
                    surf.ui_mut().on_pointer_move(lx, ly);
                    surf.mark_dirty();
                }

                if updated_drag.is_dragging_scroll {
                    let step_dx = lx - updated_drag.last_pos[0];
                    let step_dy = ly - updated_drag.last_pos[1];
                    if in_tab_strip {
                        if step_dx.abs() > 0.01 {
                            let max_x = surf.ui().scroll_max_x("qm_uilib_tabs_hscroll").max(0.0);
                            if max_x > 0.0 {
                                let cur_x = surf.ui().scroll_offset_x("qm_uilib_tabs_hscroll");
                                surf.ui_mut().scroll_to_x(
                                    "qm_uilib_tabs_hscroll",
                                    (cur_x - step_dx).clamp(0.0, max_x),
                                );
                                surf.mark_dirty();
                            } else {
                                surf.ui_mut().on_scroll(step_dx / 24.0, 0.0);
                            }
                        }
                    } else if step_dx.abs() > 0.01 || step_dy.abs() > 0.01 {
                        surf.ui_mut().on_scroll(step_dx / 24.0, step_dy / 24.0);
                    }
                }
            }
            updated_drag.last_pos = [lx, ly];
        }
        if let Some(d) = hnd.drag_state_mut(surface_id) {
            *d = updated_drag;
        }
    })) {
        set_last_error(panic_payload_to_string(payload));
    }
}

#[no_mangle]
pub unsafe extern "C" fn lumenic_cvr_pointer_down(
    handle: *mut HostHandle,
    surface_id: u32,
    button: u32,
) {
    let Some(hnd) = (unsafe { handle.as_mut() }) else {
        return;
    };
    if let Err(payload) = catch_unwind(AssertUnwindSafe(|| {
        let Some(sid) = hnd.surface_id(surface_id) else {
            return;
        };
        let btn = decode_button(button);
        let preview_rect = if surface_id == SURFACE_MAIN {
            hnd.state.main_menu.aas_preview_rect
        } else {
            hnd.state.quick.avatar_preview_rect
        };
        let mut new_drag = None;
        if let Some(surf) = hnd.host.surface_mut(sid) {
            let hovered_id = surf.ui().hovered_id();
            let is_slider_or_input = hovered_id.is_some_and(|hid| {
                surf.ui().semantics().iter().any(|n| {
                    n.id == hid
                        && matches!(
                            n.role,
                            lumenic::Role::Slider
                                | lumenic::Role::TextInput
                                | lumenic::Role::SplitDivider
                        )
                })
            });
            surf.ui_mut().on_pointer_down(btn);
            if btn == PointerButton::Left {
                let pos = surf.ui().pointer_pos().unwrap_or([0.0, 0.0]);
                let in_preview = preview_rect.is_some_and(|[rx, ry, rw, rh]| {
                    lumenic::Rect::new(rx, ry, rw, rh).contains(pos[0], pos[1])
                });
                let ignore = is_slider_or_input || in_preview || surf.ui().is_text_input_focused();
                new_drag = Some(PointerDragState {
                    is_left_down: true,
                    press_pos: pos,
                    last_pos: pos,
                    last_glow_pos: pos,
                    is_dragging_scroll: false,
                    ignore_drag: ignore,
                });
            }
        }
        if let (Some(d), Some(nd)) = (hnd.drag_state_mut(surface_id), new_drag) {
            *d = nd;
        }
    })) {
        set_last_error(panic_payload_to_string(payload));
    }
}

#[no_mangle]
pub unsafe extern "C" fn lumenic_cvr_pointer_up(
    handle: *mut HostHandle,
    surface_id: u32,
    button: u32,
) {
    let Some(hnd) = (unsafe { handle.as_mut() }) else {
        return;
    };
    if let Err(payload) = catch_unwind(AssertUnwindSafe(|| {
        let btn = decode_button(button);
        let mut was_dragging_scroll = false;
        if btn == PointerButton::Left {
            if let Some(d) = hnd.drag_state_mut(surface_id) {
                was_dragging_scroll = d.is_dragging_scroll;
                *d = PointerDragState::default();
            }
        }
        if let Some(sid) = hnd.surface_id(surface_id) {
            if let Some(surf) = hnd.host.surface_mut(sid) {
                if btn == PointerButton::Left && was_dragging_scroll {
                    surf.mark_dirty();
                } else {
                    surf.ui_mut().on_pointer_up(btn);
                }
            }
        }
    })) {
        set_last_error(panic_payload_to_string(payload));
    }
}

#[no_mangle]
pub unsafe extern "C" fn lumenic_cvr_pointer_scroll(
    handle: *mut HostHandle,
    surface_id: u32,
    dx: f32,
    dy: f32,
) {
    let Some(hnd) = (unsafe { handle.as_mut() }) else {
        return;
    };
    if !dx.is_finite() || !dy.is_finite() {
        return;
    }
    if let Err(payload) = catch_unwind(AssertUnwindSafe(|| {
        let tab_strip_rect = if surface_id == SURFACE_QUICK {
            hnd.state.quick.tab_strip_rect
        } else {
            None
        };
        if let Some(sid) = hnd.surface_id(surface_id) {
            if let Some(surf) = hnd.host.surface_mut(sid) {
                let over_tab_strip = tab_strip_rect.is_some_and(|[rx, ry, rw, rh]| {
                    surf.ui()
                        .pointer_pos()
                        .is_some_and(|[px, py]| lumenic::Rect::new(rx, ry, rw, rh).contains(px, py))
                });
                if over_tab_strip && dx.abs() < 0.01 && dy.abs() > 0.001 {
                    let max_x = surf.ui().scroll_max_x("qm_uilib_tabs_hscroll").max(0.0);
                    if max_x > 0.0 {
                        let cur_x = surf.ui().scroll_offset_x("qm_uilib_tabs_hscroll");
                        surf.ui_mut().scroll_to_x(
                            "qm_uilib_tabs_hscroll",
                            (cur_x - dy * 36.0).clamp(0.0, max_x),
                        );
                        surf.mark_dirty();
                        return;
                    }
                }
                surf.ui_mut().on_scroll(dx, dy);
            }
        }
    })) {
        set_last_error(panic_payload_to_string(payload));
    }
}

#[no_mangle]
pub unsafe extern "C" fn lumenic_cvr_pointer_leave(handle: *mut HostHandle, surface_id: u32) {
    let Some(hnd) = (unsafe { handle.as_mut() }) else {
        return;
    };
    if let Err(payload) = catch_unwind(AssertUnwindSafe(|| {
        if let Some(d) = hnd.drag_state_mut(surface_id) {
            *d = PointerDragState::default();
        }
        if surface_id == SURFACE_MAIN && hnd.state.main_menu.aas_preview_dragging {
            hnd.state.main_menu.aas_preview_dragging = false;
            if hnd.state.main_menu.aas_preview_target.starts_with("worldpano:") {
                hnd.actions.push(UiAction::WorldPanoLook {
                    delta_x: 0.0,
                    delta_y: 0.0,
                    is_dragging: false,
                });
            } else {
                hnd.actions.push(UiAction::AvatarPreviewRotate {
                    delta: 0.0,
                    is_dragging: false,
                });
            }
        } else if surface_id == SURFACE_QUICK && hnd.state.quick.avatar_preview_dragging {
            hnd.state.quick.avatar_preview_dragging = false;
            hnd.actions.push(UiAction::AvatarPreviewRotate {
                delta: 0.0,
                is_dragging: false,
            });
        }
        if let Some(lh) = hnd.last_hovered_mut(surface_id) {
            *lh = None;
        }
        if let Some(sid) = hnd.surface_id(surface_id) {
            if let Some(surf) = hnd.host.surface_mut(sid) {
                surf.ui_mut().on_pointer_leave();
            }
        }
    })) {
        set_last_error(panic_payload_to_string(payload));
    }
}

#[no_mangle]
pub unsafe extern "C" fn lumenic_cvr_pointer_cancel(handle: *mut HostHandle, surface_id: u32) {
    let Some(hnd) = (unsafe { handle.as_mut() }) else {
        return;
    };
    if let Err(payload) = catch_unwind(AssertUnwindSafe(|| {
        if let Some(d) = hnd.drag_state_mut(surface_id) {
            *d = PointerDragState::default();
        }
        if surface_id == SURFACE_MAIN && hnd.state.main_menu.aas_preview_dragging {
            hnd.state.main_menu.aas_preview_dragging = false;
            if hnd.state.main_menu.aas_preview_target.starts_with("worldpano:") {
                hnd.actions.push(UiAction::WorldPanoLook {
                    delta_x: 0.0,
                    delta_y: 0.0,
                    is_dragging: false,
                });
            } else {
                hnd.actions.push(UiAction::AvatarPreviewRotate {
                    delta: 0.0,
                    is_dragging: false,
                });
            }
        } else if surface_id == SURFACE_QUICK && hnd.state.quick.avatar_preview_dragging {
            hnd.state.quick.avatar_preview_dragging = false;
            hnd.actions.push(UiAction::AvatarPreviewRotate {
                delta: 0.0,
                is_dragging: false,
            });
        }
        if let Some(sid) = hnd.surface_id(surface_id) {
            if let Some(surf) = hnd.host.surface_mut(sid) {
                surf.ui_mut().on_pointer_cancel();
            }
        }
    })) {
        set_last_error(panic_payload_to_string(payload));
    }
}

#[no_mangle]
pub unsafe extern "C" fn lumenic_cvr_key_down(
    handle: *mut HostHandle,
    surface_id: u32,
    key_code: u32,
    mod_flags: u32,
) {
    let Some(hnd) = (unsafe { handle.as_mut() }) else {
        return;
    };
    if let Err(payload) = catch_unwind(AssertUnwindSafe(|| {
        let Some(key) = decode_key(key_code) else {
            return;
        };
        let mods = decode_modifiers(mod_flags);
        let Some(sid) = hnd.surface_id(surface_id) else {
            return;
        };
        if let Some(surf) = hnd.host.surface_mut(sid) {
            // Prevent Escape lockout when C# InputLock blocks CVR's Escape handler.
            if key == Key::Escape && surf.ui().is_text_input_focused() {
                surf.ui_mut().clear_focus();
                if let Some(buf) = hnd.pending_text_mut(surface_id) {
                    buf.clear();
                }
                return;
            }
            // Authoritative double-space guard: OnCharEvent already commits ' ' when text-focused.
            if key == Key::Space && surf.ui().is_text_input_focused() {
                return;
            }
            surf.ui_mut().on_key_down(key, mods);
        }
    })) {
        set_last_error(panic_payload_to_string(payload));
    }
}

#[no_mangle]
pub unsafe extern "C" fn lumenic_cvr_text_input(
    handle: *mut HostHandle,
    surface_id: u32,
    utf8_ptr: *const u8,
    len: usize,
) {
    let Some(hnd) = (unsafe { handle.as_mut() }) else {
        return;
    };
    if utf8_ptr.is_null() || len == 0 {
        return;
    }
    if let Err(payload) = catch_unwind(AssertUnwindSafe(|| {
        let Ok(bytes) = (unsafe { checked_slice(utf8_ptr, len) }) else {
            return;
        };
        if let Ok(text) = std::str::from_utf8(bytes) {
            if !text.is_empty() {
                if let Some(buf) = hnd.pending_text_mut(surface_id) {
                    buf.push_str(text);
                }
                if let Some(sid) = hnd.surface_id(surface_id) {
                    if let Some(surf) = hnd.host.surface_mut(sid) {
                        surf.mark_dirty();
                    }
                }
            }
        }
    })) {
        set_last_error(panic_payload_to_string(payload));
    }
}

#[no_mangle]
pub unsafe extern "C" fn lumenic_cvr_clear_focus(handle: *mut HostHandle, surface_id: u32) {
    let Some(hnd) = (unsafe { handle.as_mut() }) else {
        return;
    };
    if let Err(payload) = catch_unwind(AssertUnwindSafe(|| {
        if let Some(buf) = hnd.pending_text_mut(surface_id) {
            buf.clear();
        }
        if let Some(sid) = hnd.surface_id(surface_id) {
            if let Some(surf) = hnd.host.surface_mut(sid) {
                surf.ui_mut().clear_focus();
            }
        }
        if surface_id == SURFACE_QUICK {
            hnd.state.quick.go_home_confirm = false;
            if hnd.state.quick.uilib_sliders.len() > 256 {
                hnd.state.quick.uilib_sliders.clear();
            }
        } else {
            hnd.state.main_menu.quit_confirm = false;
            hnd.state.main_menu.go_home_confirm = false;
            if hnd.state.main_menu.uix_sliders.len() > 256 {
                hnd.state.main_menu.uix_sliders.clear();
            }
            if hnd.state.main_menu.uix_text_inputs.len() > 256 {
                hnd.state.main_menu.uix_text_inputs.clear();
            }
            if hnd.state.main_menu.settings_sliders.len() > 256 {
                hnd.state.main_menu.settings_sliders.clear();
            }
            if hnd.state.main_menu.avatar_sliders.len() > 256 {
                hnd.state.main_menu.avatar_sliders.clear();
            }
            hnd.state.main_menu.safety_confirm_action.clear();
        }
    })) {
        set_last_error(panic_payload_to_string(payload));
    }
}

#[no_mangle]
pub unsafe extern "C" fn lumenic_cvr_commit_virtual_keyboard(
    handle: *mut HostHandle,
    surface_id: u32,
    field_id_ptr: *const u8,
    field_id_len: usize,
    text_ptr: *const u8,
    text_len: usize,
) -> bool {
    clear_last_error();
    let Some(hnd) = (unsafe { handle.as_mut() }) else {
        set_last_error("lumenic_cvr_commit_virtual_keyboard: null handle");
        return false;
    };
    if field_id_ptr.is_null() || field_id_len == 0 {
        set_last_error("lumenic_cvr_commit_virtual_keyboard: empty field_id");
        return false;
    }
    let res = catch_unwind(AssertUnwindSafe(|| -> Result<(), String> {
        let field_bytes = unsafe { checked_slice(field_id_ptr, field_id_len)? };
        let field_id =
            std::str::from_utf8(field_bytes).map_err(|e| format!("invalid field_id utf8: {e}"))?;
        let text = if text_ptr.is_null() || text_len == 0 {
            ""
        } else {
            let text_bytes = unsafe { checked_slice(text_ptr, text_len)? };
            std::str::from_utf8(text_bytes).map_err(|e| format!("invalid text utf8: {e}"))?
        };

        match field_id {
            "mm_search_input" | "mm_srch_page_input" => {
                hnd.state.main_menu.search_input = lumenic::TextEditState::with_text(text);
                hnd.state.main_menu.last_search = String::from(text);
                let trimmed = text.trim();
                if !trimmed.is_empty() {
                    if hnd.state.main_menu.active_tab != "search" {
                        crate::main_menu::push_nav_history(&mut hnd.state);
                    }
                    hnd.state.main_menu.active_tab = String::from("search");
                    hnd.state.main_menu.search_filter_kind = String::from("all");
                    crate::main_menu::close_all_detail_overlays(&mut hnd.actions);
                    hnd.actions.push(UiAction::ContentGlobalSearch {
                        query: String::from(trimmed),
                    });
                }
            }
            "mm_cb_filter" => {
                hnd.state.main_menu.browser_local_filter = lumenic::TextEditState::with_text(text);
            }
            "mm_fr_search" => {
                hnd.state.main_menu.friends_search_input = lumenic::TextEditState::with_text(text);
            }
            "mm_set_search" => {
                hnd.state.main_menu.settings_search_input = lumenic::TextEditState::with_text(text);
            }
            "mm_uix_cat_search" => {
                hnd.state.main_menu.uix_search_input = lumenic::TextEditState::with_text(text);
            }
            "mm_grp_name_input" => {
                hnd.state.main_menu.group_name_input = lumenic::TextEditState::with_text(text);
            }
            "mm_grp_desc_input" => {
                hnd.state.main_menu.group_desc_input = lumenic::TextEditState::with_text(text);
            }
            "mm_aas_new_profile" => {
                let trimmed: String = text.trim().chars().take(12).collect();
                if !trimmed.is_empty() && !trimmed.eq_ignore_ascii_case("default") {
                    hnd.actions.push(UiAction::AvatarSaveProfile { profile: trimmed });
                }
                hnd.state.main_menu.aas_new_profile_input = lumenic::TextEditState::new();
            }
            _ => {}
        }

        if let Some(sid) = hnd.surface_id(surface_id) {
            if let Some(surf) = hnd.host.surface_mut(sid) {
                surf.ui_mut().clear_focus();
                surf.mark_dirty();
            }
        }
        Ok(())
    }));

    match res {
        Ok(Ok(())) => true,
        Ok(Err(err)) => {
            set_last_error(err);
            false
        }
        Err(payload) => {
            set_last_error(panic_payload_to_string(payload));
            false
        }
    }
}

#[no_mangle]
pub unsafe extern "C" fn lumenic_cvr_upload_image_bytes(
    handle: *mut HostHandle,
    surface_id: u32,
    key_ptr: *const u8,
    key_len: usize,
    bytes_ptr: *const u8,
    bytes_len: usize,
) -> bool {
    clear_last_error();
    let Some(hnd) = (unsafe { handle.as_mut() }) else {
        set_last_error("lumenic_cvr_upload_image_bytes: null handle");
        return false;
    };
    if key_ptr.is_null() || key_len == 0 || bytes_ptr.is_null() || bytes_len == 0 {
        set_last_error("lumenic_cvr_upload_image_bytes: empty key or bytes");
        return false;
    }
    let res = catch_unwind(AssertUnwindSafe(|| -> Result<(), String> {
        let Some(sid) = hnd.surface_id(surface_id) else {
            return Err(format!("invalid surface_id: {surface_id}"));
        };
        let key_bytes = unsafe { checked_slice(key_ptr, key_len)? };
        let key = std::str::from_utf8(key_bytes)
            .map_err(|e| format!("invalid image key utf8: {e}"))?
            .to_owned();
        let raw_bytes = unsafe { checked_slice(bytes_ptr, bytes_len)? };
        let decoded =
            lumenic::decode_image(raw_bytes).map_err(|e| format!("decode_image({key}): {e}"))?;

        let HostHandle { host, state, .. } = hnd;
        let guard = MakeNotCurrentGuard(host);
        let surf = guard
            .0
            .surface_mut(sid)
            .ok_or_else(|| String::from("surface missing"))?;
        let tex_id = surf
            .app_mut()
            .create_texture_rgba(decoded.width, decoded.height, &decoded.data)
            .map_err(|e| format!("create_texture_rgba({key}): {e}"))?;

        let cache = if surface_id == SURFACE_MAIN {
            &mut state.main_textures
        } else {
            &mut state.quick_textures
        };
        cache.insert(surf.app_mut(), key, tex_id);
        surf.mark_dirty();
        Ok(())
    }));

    match res {
        Ok(Ok(())) => true,
        Ok(Err(err)) => {
            set_last_error(err);
            false
        }
        Err(payload) => {
            set_last_error(panic_payload_to_string(payload));
            false
        }
    }
}

#[no_mangle]
pub unsafe extern "C" fn lumenic_cvr_push_state_json(
    handle: *mut HostHandle,
    utf8_ptr: *const u8,
    len: usize,
) -> bool {
    clear_last_error();
    let Some(hnd) = (unsafe { handle.as_mut() }) else {
        set_last_error("lumenic_cvr_push_state_json: null handle");
        return false;
    };
    if utf8_ptr.is_null() || len == 0 {
        set_last_error("lumenic_cvr_push_state_json: empty payload");
        return false;
    }
    let res = catch_unwind(AssertUnwindSafe(|| -> Result<(), String> {
        let bytes = unsafe { checked_slice(utf8_ptr, len)? };
        let prev_fps = hnd.state.host.fps;
        let prev_game_ping = hnd.state.host.game_ping;
        let prev_voice_ping = hnd.state.host.voice_ping;
        let prev_clock = std::mem::take(&mut hnd.state.host.clock_text);
        let prev_date = std::mem::take(&mut hnd.state.host.date_text);
        let mut snapshot: HostStateSnapshot =
            serde_json::from_slice(bytes).map_err(|e| format!("state JSON parse error: {e}"))?;
        if snapshot.clock_text.is_empty() {
            snapshot.fps = prev_fps;
            snapshot.game_ping = prev_game_ping;
            snapshot.voice_ping = prev_voice_ping;
            snapshot.clock_text = prev_clock;
            snapshot.date_text = prev_date;
        }
        hnd.state.host = snapshot;
        hnd.mark_all_dirty();
        Ok(())
    }));

    match res {
        Ok(Ok(())) => true,
        Ok(Err(err)) => {
            set_last_error(err);
            false
        }
        Err(payload) => {
            set_last_error(panic_payload_to_string(payload));
            false
        }
    }
}

#[no_mangle]
pub unsafe extern "C" fn lumenic_cvr_push_telemetry(
    handle: *mut HostHandle,
    fps: i32,
    game_ping: i32,
    voice_ping: i32,
    clock_ptr: *const u8,
    clock_len: usize,
    date_ptr: *const u8,
    date_len: usize,
) -> bool {
    clear_last_error();
    let Some(hnd) = (unsafe { handle.as_mut() }) else {
        set_last_error("lumenic_cvr_push_telemetry: null handle");
        return false;
    };
    let res = catch_unwind(AssertUnwindSafe(|| -> Result<(), String> {
        let clock = if clock_ptr.is_null() || clock_len == 0 {
            ""
        } else {
            let b = unsafe { checked_slice(clock_ptr, clock_len)? };
            std::str::from_utf8(b).map_err(|e| format!("invalid clock utf8: {e}"))?
        };
        let date = if date_ptr.is_null() || date_len == 0 {
            ""
        } else {
            let b = unsafe { checked_slice(date_ptr, date_len)? };
            std::str::from_utf8(b).map_err(|e| format!("invalid date utf8: {e}"))?
        };

        let stats_changed = hnd.state.host.fps != fps
            || hnd.state.host.game_ping != game_ping
            || hnd.state.host.voice_ping != voice_ping;
        let clock_changed =
            hnd.state.host.clock_text != clock || hnd.state.host.date_text != date;

        if stats_changed {
            hnd.state.host.fps = fps;
            hnd.state.host.game_ping = game_ping;
            hnd.state.host.voice_ping = voice_ping;
        }
        if clock_changed {
            hnd.state.host.clock_text.clear();
            hnd.state.host.clock_text.push_str(clock);
            hnd.state.host.date_text.clear();
            hnd.state.host.date_text.push_str(date);
        }

        if clock_changed || stats_changed {
            if let Some(s) = hnd.host.surface_mut(hnd.quick_surface) {
                s.mark_dirty();
            }
        }
        if clock_changed || (stats_changed && hnd.state.main_menu.info_modal_open) {
            if let Some(s) = hnd.host.surface_mut(hnd.main_surface) {
                s.mark_dirty();
            }
        }
        Ok(())
    }));

    match res {
        Ok(Ok(())) => true,
        Ok(Err(err)) => {
            set_last_error(err);
            false
        }
        Err(payload) => {
            set_last_error(panic_payload_to_string(payload));
            false
        }
    }
}

#[no_mangle]
pub unsafe extern "C" fn lumenic_cvr_step(
    handle: *mut HostHandle,
    surface_id: u32,
    dt_secs: f32,
    force_redraw: bool,
) -> u32 {
    clear_last_error();
    let Some(hnd) = (unsafe { handle.as_mut() }) else {
        set_last_error("lumenic_cvr_step: null handle");
        return STEP_FLAG_ERROR_OR_PANIC;
    };
    let safe_dt = if dt_secs.is_finite() && dt_secs > 0.0 {
        dt_secs.min(0.25)
    } else {
        0.0
    };
    let res = catch_unwind(AssertUnwindSafe(|| -> Result<u32, String> {
        let Some(sid) = hnd.surface_id(surface_id) else {
            return Err(format!("invalid surface_id: {surface_id}"));
        };

        // Flush accumulated same-frame UTF-8 text into a single on_ime_commit call.
        if let Some(buf) = hnd.pending_text_mut(surface_id) {
            if !buf.is_empty() {
                let committed = std::mem::take(buf);
                if let Some(surf) = hnd.host.surface_mut(sid) {
                    surf.ui_mut().on_ime_commit(committed);
                }
            }
        }

        let HostHandle {
            host,
            state,
            actions,
            main_last_hovered,
            quick_last_hovered,
            ..
        } = hnd;

        let render_fn = |ui: &mut lumenic::UiContext, app: &mut lumenic::App| {
            if surface_id == SURFACE_MAIN {
                render_main_menu(ui, app, state, actions);
            } else {
                render_quick_menu(ui, app, state, actions);
            }
        };

        let rendered = {
            let guard = MakeNotCurrentGuard(host);
            let step_result = if force_redraw {
                guard.0.step_frame_forced(sid, safe_dt, render_fn)
            } else {
                guard.0.step_frame(sid, safe_dt, render_fn)
            };
            step_result.map_err(|e| format!("step_frame: {e}"))?
        };

        let mut flags = 0u32;
        if rendered {
            flags |= STEP_FLAG_RENDERED;
            // First frame entering any 3D avatar turntable view has prev_rect() == None;
            // schedule one extra frame so SyncAvatarPreview emits immediately.
            let needs_preview_rect_pass = (surface_id == SURFACE_MAIN
                && state.main_menu.aas_preview_rendered
                && state.main_menu.aas_last_preview_sync.is_none())
                || (surface_id == SURFACE_QUICK
                    && state.quick.avatar_preview_rendered
                    && state.quick.last_avatar_preview_sync.is_none());
            if needs_preview_rect_pass {
                if let Some(surf) = host.surface_mut(sid) {
                    surf.mark_dirty();
                }
            }
        }
        let current_hovered = host.surface(sid).and_then(|s| s.ui().hovered_id());
        let prev_hovered = if surface_id == SURFACE_MAIN {
            main_last_hovered
        } else {
            quick_last_hovered
        };
        if current_hovered != *prev_hovered {
            *prev_hovered = current_hovered;
            if current_hovered.is_some() {
                actions.push(UiAction::PlayUiSound {
                    sound: String::from("Hover"),
                });
            }
        }
        if let Some(surf) = host.surface(sid) {
            if surf.ui().is_text_input_focused() {
                flags |= STEP_FLAG_TEXT_FOCUSED;
            }
            if surf.ui().is_pointer_over_ui() {
                flags |= STEP_FLAG_POINTER_OVER_UI;
            }
        }
        if !actions.is_empty() {
            flags |= STEP_FLAG_HAS_ACTIONS;
        }
        Ok(flags)
    }));

    match res {
        Ok(Ok(flags)) => flags,
        Ok(Err(err)) => {
            set_last_error(err);
            STEP_FLAG_ERROR_OR_PANIC
        }
        Err(payload) => {
            set_last_error(panic_payload_to_string(payload));
            STEP_FLAG_ERROR_OR_PANIC
        }
    }
}

#[no_mangle]
pub unsafe extern "C" fn lumenic_cvr_read_pixels(
    handle: *mut HostHandle,
    surface_id: u32,
    dst_ptr: *mut u8,
    dst_len: usize,
) -> bool {
    clear_last_error();
    let Some(hnd) = (unsafe { handle.as_mut() }) else {
        set_last_error("lumenic_cvr_read_pixels: null handle");
        return false;
    };
    if dst_ptr.is_null() || dst_len == 0 {
        set_last_error("lumenic_cvr_read_pixels: null/empty destination buffer");
        return false;
    }
    let res = catch_unwind(AssertUnwindSafe(|| -> Result<(), String> {
        let Some(sid) = hnd.surface_id(surface_id) else {
            return Err(format!("invalid surface_id: {surface_id}"));
        };
        let expected = hnd
            .host
            .surface(sid)
            .map(|s| s.rgba_byte_len())
            .ok_or_else(|| String::from("surface missing"))?;
        if dst_len < expected {
            return Err(format!(
                "buffer too small: got {dst_len} bytes, need {expected}"
            ));
        }
        let dst = unsafe { checked_slice_mut(dst_ptr, expected)? };
        let guard = MakeNotCurrentGuard(&mut hnd.host);
        guard
            .0
            .read_pixels_rgba_into(sid, dst, true)
            .map_err(|e| format!("read_pixels_rgba_into: {e}"))
    }));

    match res {
        Ok(Ok(())) => true,
        Ok(Err(err)) => {
            set_last_error(err);
            false
        }
        Err(payload) => {
            set_last_error(panic_payload_to_string(payload));
            false
        }
    }
}

#[no_mangle]
pub unsafe extern "C" fn lumenic_cvr_poll_actions_json(
    handle: *mut HostHandle,
    out_ptr: *mut *const u8,
    out_len: *mut usize,
) -> bool {
    clear_last_error();
    if !out_ptr.is_null() {
        unsafe { *out_ptr = std::ptr::null() };
    }
    if !out_len.is_null() {
        unsafe { *out_len = 0 };
    }
    let Some(hnd) = (unsafe { handle.as_mut() }) else {
        set_last_error("lumenic_cvr_poll_actions_json: null handle");
        return false;
    };
    let res = catch_unwind(AssertUnwindSafe(|| -> Result<(), String> {
        hnd.actions_scratch.clear();
        if !hnd.actions.is_empty() {
            serde_json::to_writer(&mut hnd.actions_scratch, &hnd.actions)
                .map_err(|e| format!("action JSON serialize error: {e}"))?;
            hnd.actions.clear();
        }
        if !out_ptr.is_null() {
            unsafe {
                *out_ptr = if hnd.actions_scratch.is_empty() {
                    std::ptr::null()
                } else {
                    hnd.actions_scratch.as_ptr()
                };
            }
        }
        if !out_len.is_null() {
            unsafe { *out_len = hnd.actions_scratch.len() };
        }
        Ok(())
    }));

    match res {
        Ok(Ok(())) => true,
        Ok(Err(err)) => {
            set_last_error(err);
            false
        }
        Err(payload) => {
            set_last_error(panic_payload_to_string(payload));
            false
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn read_last_error() -> String {
        let len = unsafe { lumenic_cvr_last_error(std::ptr::null_mut(), 0) };
        if len == 0 {
            return String::new();
        }
        let mut buf = vec![0u8; len];
        unsafe { lumenic_cvr_last_error(buf.as_mut_ptr(), buf.len()) };
        String::from_utf8_lossy(&buf).into_owned()
    }

    #[test]
    fn ffi_abi_version_and_catch_unwind_last_error() {
        assert_eq!(lumenic_cvr_abi_version(), ABI_VERSION);
        clear_last_error();
        let caught = catch_unwind(AssertUnwindSafe(|| {
            panic!("intentional unwind verification");
        }));
        assert!(caught.is_err());
        set_last_error(panic_payload_to_string(caught.unwrap_err()));
        let msg = read_last_error();
        assert!(msg.contains("intentional unwind verification"));
    }

    #[test]
    fn ffi_offscreen_render_transparent_margin_resize_and_idle_skip() {
        let handle = lumenic_cvr_create(640, 480, 512, 512, 1.0);
        assert!(
            !handle.is_null(),
            "lumenic_cvr_create failed: {}",
            read_last_error()
        );

        unsafe {
            // First step renders.
            let flags1 = lumenic_cvr_step(handle, SURFACE_MAIN, 0.016, false);
            assert_eq!(flags1 & STEP_FLAG_ERROR_OR_PANIC, 0);
            assert_ne!(flags1 & STEP_FLAG_RENDERED, 0);

            // Read bottom-up RGBA8 pixels.
            let mut pixels = vec![0u8; 640 * 480 * 4];
            assert!(
                lumenic_cvr_read_pixels(handle, SURFACE_MAIN, pixels.as_mut_ptr(), pixels.len()),
                "read_pixels failed: {}",
                read_last_error()
            );

            // Outer corner (0, 0) lies in the 24px transparent margin -> alpha must be 0.
            assert_eq!(
                pixels[3], 0,
                "expected transparent corner pixel, got rgba={:?}",
                &pixels[0..4]
            );

            // Some interior pixels must be drawn (non-zero alpha).
            let non_zero_alpha = pixels.chunks_exact(4).filter(|p| p[3] > 0).count();
            assert!(
                non_zero_alpha > 10_000,
                "expected rendered UI pixels, only found {non_zero_alpha}"
            );

            // Verify F12 does NOT toggle Lumenic debug HUD.
            lumenic_cvr_key_down(handle, SURFACE_MAIN, KEY_F12, 0);
            let hnd_ref = &*handle;
            assert!(!hnd_ref.host.surface(hnd_ref.main_surface).unwrap().ui().hud_visible());

            // Resize main surface and render again.
            assert!(lumenic_cvr_resize(handle, SURFACE_MAIN, 800, 600, 1.0));
            let flags_resized = lumenic_cvr_step(handle, SURFACE_MAIN, 0.016, false);
            assert_ne!(flags_resized & STEP_FLAG_RENDERED, 0);

            let mut resized_pixels = vec![0u8; 800 * 600 * 4];
            assert!(lumenic_cvr_read_pixels(
                handle,
                SURFACE_MAIN,
                resized_pixels.as_mut_ptr(),
                resized_pixels.len()
            ));

            // Step until any initial springs settle, then verify idle frames skip rendering.
            for _ in 0..120 {
                let _ = lumenic_cvr_step(handle, SURFACE_MAIN, 0.016, false);
            }
            let idle_flags = lumenic_cvr_step(handle, SURFACE_MAIN, 0.016, false);
            assert_eq!(idle_flags & STEP_FLAG_RENDERED, 0, "idle frame should skip render");

            // Render Quick Menu surface as well.
            let qm_flags = lumenic_cvr_step(handle, SURFACE_QUICK, 0.016, true);
            assert_ne!(qm_flags & STEP_FLAG_RENDERED, 0);

            lumenic_cvr_destroy(handle);
        }
    }

    #[test]
    fn ffi_same_frame_multi_char_text_input_and_escape_clear_focus() {
        let handle = lumenic_cvr_create(1280, 720, 800, 800, 1.0);
        assert!(!handle.is_null());

        unsafe {
            // Initial layout pass so widgets have rects.
            let _ = lumenic_cvr_step(handle, SURFACE_MAIN, 0.016, true);

            // Locate the text input semantic node directly to click it accurately.
            let input_rect = {
                let hnd = &*handle;
                let surf = hnd.host.surface(hnd.main_surface).unwrap();
                surf.ui()
                    .semantics()
                    .iter()
                    .find(|n| n.role == lumenic::Role::TextInput)
                    .map(|n| n.bounds)
                    .expect("expected TextInput semantic node in Main Menu")
            };

            let cx = input_rect.x + input_rect.w * 0.5;
            let cy = input_rect.y + input_rect.h * 0.5;
            lumenic_cvr_pointer_move(handle, SURFACE_MAIN, cx, cy);
            lumenic_cvr_pointer_down(handle, SURFACE_MAIN, 0);
            lumenic_cvr_pointer_up(handle, SURFACE_MAIN, 0);

            let f1 = lumenic_cvr_step(handle, SURFACE_MAIN, 0.016, false);
            assert_ne!(
                f1 & STEP_FLAG_TEXT_FOCUSED,
                0,
                "clicking text input should report TEXT_FOCUSED"
            );

            // Send two separate characters in the same frame before stepping.
            lumenic_cvr_text_input(handle, SURFACE_MAIN, b"A".as_ptr(), 1);
            lumenic_cvr_text_input(handle, SURFACE_MAIN, b"B".as_ptr(), 1);
            let _ = lumenic_cvr_step(handle, SURFACE_MAIN, 0.016, false);

            let typed = (&*handle).state.main_menu.search_input.text();
            assert_eq!(typed, "AB", "same-frame multi-char input must not drop chars");

            // Escape while text-focused must clear focus cleanly.
            lumenic_cvr_key_down(handle, SURFACE_MAIN, KEY_ESCAPE, 0);
            let f2 = lumenic_cvr_step(handle, SURFACE_MAIN, 0.016, false);
            assert_eq!(
                f2 & STEP_FLAG_TEXT_FOCUSED,
                0,
                "Escape must clear text focus"
            );

            lumenic_cvr_destroy(handle);
        }
    }

    #[test]
    fn ffi_quick_menu_uilib_tab_strip_drag_scroll() {
        let handle = lumenic_cvr_create(1280, 720, 800, 800, 1.0);
        assert!(!handle.is_null());

        unsafe {
            {
                let hnd = &mut *handle;
                hnd.state.host.uilib.tabs = (0..14)
                    .map(|i| state::UiLibTab {
                        id: format!("tab_{i}"),
                        label: format!("ModTab{i}"),
                        mod_name: format!("Mod{i}"),
                        icon_key: String::new(),
                        active: i == 0,
                        kind: String::from("mod"),
                    })
                    .collect();
            }

            let _ = lumenic_cvr_step(handle, SURFACE_QUICK, 0.016, true);
            let _ = lumenic_cvr_step(handle, SURFACE_QUICK, 0.016, true);

            let (strip_rect, max_x) = {
                let hnd = &*handle;
                let surf = hnd.host.surface(hnd.quick_surface).unwrap();
                (
                    hnd.state.quick.tab_strip_rect.expect("expected tab_strip_rect"),
                    surf.ui().scroll_max_x("qm_uilib_tabs_hscroll"),
                )
            };
            assert!(max_x > 10.0, "expected horizontal overflow on qm_uilib_tabs_hscroll, got max_x={max_x}");

            (&mut *handle).actions.clear();

            let start_x = strip_rect[0] + strip_rect[2] * 0.5;
            let start_y = strip_rect[1] + strip_rect[3] * 0.5;
            lumenic_cvr_pointer_move(handle, SURFACE_QUICK, start_x, start_y);
            lumenic_cvr_pointer_down(handle, SURFACE_QUICK, 0);
            for step in 1..=6 {
                lumenic_cvr_pointer_move(handle, SURFACE_QUICK, start_x - (step as f32) * 20.0, start_y);
            }
            lumenic_cvr_pointer_up(handle, SURFACE_QUICK, 0);

            let _ = lumenic_cvr_step(handle, SURFACE_QUICK, 0.016, true);
            let _ = lumenic_cvr_step(handle, SURFACE_QUICK, 0.016, true);

            let (offset_x, actions_snapshot) = {
                let hnd = &*handle;
                let surf = hnd.host.surface(hnd.quick_surface).unwrap();
                (
                    surf.ui().scroll_offset_x("qm_uilib_tabs_hscroll"),
                    hnd.actions.clone(),
                )
            };
            assert!(offset_x > 10.0, "expected horizontal drag to scroll qm_uilib_tabs_hscroll, got offset_x={offset_x}");
            assert!(
                !actions_snapshot
                    .iter()
                    .any(|a| matches!(a, state::UiAction::UiLibSelectTab { .. })),
                "drag-scroll should cancel underlying tab click, got actions: {actions_snapshot:?}"
            );

            lumenic_cvr_destroy(handle);
        }
    }

    #[test]
    fn ffi_quick_menu_world_modal_is_centered() {
        let handle = lumenic_cvr_create(1280, 720, 800, 800, 1.0);
        assert!(!handle.is_null());

        unsafe {
            {
                let hnd = &mut *handle;
                hnd.state.quick.qm_world_modal = Some((
                    String::from("w-test"),
                    String::from("Test World"),
                ));
            }
            let _ = lumenic_cvr_step(handle, SURFACE_QUICK, 0.016, true);
            let _ = lumenic_cvr_step(handle, SURFACE_QUICK, 0.016, true);

            let (scrim_bounds, create_bounds) = {
                let hnd = &*handle;
                let surf = hnd.host.surface(hnd.quick_surface).unwrap();
                let sems = surf.ui().semantics();
                let create = sems.iter().find(|n| n.label.contains("Create")).map(|n| n.bounds);
                let scrim = sems.iter().find(|n| n.bounds.w > 700.0 && n.bounds.h > 700.0 && n.bounds.x == 0.0 && n.bounds.y == 0.0).map(|n| n.bounds);
                (scrim, create)
            };
            let scrim = scrim_bounds.expect("expected full-viewport modal scrim");
            assert!((scrim.w - 800.0).abs() < 1.0 && (scrim.h - 800.0).abs() < 1.0);
            let create = create_bounds.expect("expected Create & Join button");
            assert!(
                create.x > 150.0 && create.x < 250.0,
                "expected Create button centered inside 440px modal, got {create:?}"
            );
            assert!(
                create.y > 250.0 && create.y < 600.0,
                "expected Create button vertically centered, got {create:?}"
            );

            lumenic_cvr_destroy(handle);
        }
    }

    #[test]
    fn ffi_quick_menu_group_mod_prompt_is_centered() {
        let handle = lumenic_cvr_create(1280, 720, 800, 800, 1.0);
        assert!(!handle.is_null());

        unsafe {
            {
                let hnd = &mut *handle;
                hnd.state.host.content.group_mod_prompt = Some(crate::state::GroupModPromptSnapshot {
                    instance_id: String::from("i-test"),
                    instance_name: String::from("Test Instance"),
                    world_name: String::from("Test World"),
                    group_id: String::from("g-test"),
                    group_name: String::from("Test Group"),
                    privacy: String::from("GroupPublic"),
                    is_verified: true,
                    moderator_count: 2,
                });
            }
            let _ = lumenic_cvr_step(handle, SURFACE_QUICK, 0.016, true);
            let _ = lumenic_cvr_step(handle, SURFACE_QUICK, 0.016, true);

            let join_bounds = {
                let hnd = &*handle;
                let surf = hnd.host.surface(hnd.quick_surface).unwrap();
                let sems = surf.ui().semantics();
                sems.iter().find(|n| n.label.contains("Join Instance")).map(|n| n.bounds)
            };
            let join = join_bounds.expect("expected Join Instance button inside Quick Menu");
            assert!(
                join.x > 150.0 && join.x < 650.0,
                "expected Join Instance button horizontally inside 800px Quick Menu, got {join:?}"
            );
            assert!(
                join.y > 200.0 && join.y < 600.0,
                "expected Join Instance button vertically inside 800px Quick Menu, got {join:?}"
            );

            lumenic_cvr_destroy(handle);
        }
    }
}
}

#[cfg(not(target_arch = "wasm32"))]
pub use native_ffi::*;
