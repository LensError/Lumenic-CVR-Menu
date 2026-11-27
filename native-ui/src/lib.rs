pub mod avatar_settings_view;
pub mod content_browser;
pub mod ext;
pub mod ext_view;
pub mod icons;
pub mod main_menu;
pub mod quick_menu;
pub mod settings_view;
pub mod social_view;
pub mod state;
pub mod theme;
pub mod uilib_view;
pub mod uix_view;
pub mod virtual_keyboard;
pub mod widgets;

pub const ABI_VERSION: u32 = 8;

pub const SURFACE_MAIN: u32 = 0;
pub const SURFACE_QUICK: u32 = 1;

// Domain IDs for incremental state pushes via `lumenic_cvr_push_domain_json`.
pub const DOMAIN_CORE: u32 = 0;
pub const DOMAIN_STATIC_CATALOG: u32 = 1;
pub const DOMAIN_UILIB: u32 = 2;
pub const DOMAIN_CONTENT: u32 = 3;
pub const DOMAIN_SOCIAL: u32 = 4;
pub const DOMAIN_SETTINGS: u32 = 5;
pub const DOMAIN_AVATAR_SETTINGS: u32 = 6;
pub const DOMAIN_UIX: u32 = 7;

pub const STEP_FLAG_RENDERED: u32 = 1 << 0;
pub const STEP_FLAG_TEXT_FOCUSED: u32 = 1 << 1;
pub const STEP_FLAG_POINTER_OVER_UI: u32 = 1 << 2;
pub const STEP_FLAG_HAS_ACTIONS: u32 = 1 << 3;
pub const STEP_FLAG_PIXELS_READ: u32 = 1 << 4;
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

fn ffi_bool(f: impl FnOnce() -> Result<(), String>) -> bool {
    match catch_unwind(AssertUnwindSafe(f)) {
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

fn ffi_void(f: impl FnOnce()) {
    if let Err(payload) = catch_unwind(AssertUnwindSafe(f)) {
        set_last_error(panic_payload_to_string(payload));
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

const MAX_IMAGE_DIM: u32 = 4096;
const MAX_IMAGE_PIXELS: u64 = (MAX_IMAGE_DIM as u64) * (MAX_IMAGE_DIM as u64);

fn validate_image_dims(w: u32, h: u32, ctx: &str) -> Result<(), String> {
    if w == 0 || h == 0 || w > MAX_IMAGE_DIM || h > MAX_IMAGE_DIM || (w as u64) * (h as u64) > MAX_IMAGE_PIXELS {
        return Err(format!(
            "{ctx}: dimensions {w}x{h} exceed {MAX_IMAGE_DIM}x{MAX_IMAGE_DIM} limit"
        ));
    }
    Ok(())
}

/// Pre-validates declared dimensions from PNG/GIF/BMP/WebP/JPEG headers before decoding
/// to prevent decompression bombs from allocating gigabytes of RGBA memory.
fn check_image_header_dimensions(raw: &[u8], ctx: &str) -> Result<(), String> {
    // PNG: 8-byte signature + IHDR chunk (width/height at bytes 16..24)
    if raw.len() >= 24 && &raw[0..8] == b"\x89PNG\r\n\x1a\n" && &raw[12..16] == b"IHDR" {
        let w = u32::from_be_bytes([raw[16], raw[17], raw[18], raw[19]]);
        let h = u32::from_be_bytes([raw[20], raw[21], raw[22], raw[23]]);
        return validate_image_dims(w, h, ctx);
    }
    // GIF: GIF87a / GIF89a, width/height LE u16 at bytes 6..10
    if raw.len() >= 10 && (&raw[0..6] == b"GIF87a" || &raw[0..6] == b"GIF89a") {
        let w = u16::from_le_bytes([raw[6], raw[7]]) as u32;
        let h = u16::from_le_bytes([raw[8], raw[9]]) as u32;
        return validate_image_dims(w, h, ctx);
    }
    // BMP: BM + BITMAPINFOHEADER (>= 40 bytes)
    if raw.len() >= 26 && &raw[0..2] == b"BM" {
        let dib_size = u32::from_le_bytes([raw[14], raw[15], raw[16], raw[17]]);
        if dib_size >= 40 {
            let w = i32::from_le_bytes([raw[18], raw[19], raw[20], raw[21]]).unsigned_abs();
            let h = i32::from_le_bytes([raw[22], raw[23], raw[24], raw[25]]).unsigned_abs();
            return validate_image_dims(w, h, ctx);
        }
    }
    // WebP: RIFF....WEBP + VP8X / VP8L / VP8
    if raw.len() >= 30 && &raw[0..4] == b"RIFF" && &raw[8..12] == b"WEBP" {
        match &raw[12..16] {
            b"VP8X" => {
                let w = 1 + (raw[24] as u32 | ((raw[25] as u32) << 8) | ((raw[26] as u32) << 16));
                let h = 1 + (raw[27] as u32 | ((raw[28] as u32) << 8) | ((raw[29] as u32) << 16));
                return validate_image_dims(w, h, ctx);
            }
            b"VP8L" if raw[20] == 0x2f => {
                let bits = u32::from_le_bytes([raw[21], raw[22], raw[23], raw[24]]);
                let w = (bits & 0x3FFF) + 1;
                let h = ((bits >> 14) & 0x3FFF) + 1;
                return validate_image_dims(w, h, ctx);
            }
            b"VP8 " if raw[23] == 0x9d && raw[24] == 0x01 && raw[25] == 0x2a => {
                let w = (u16::from_le_bytes([raw[26], raw[27]]) & 0x3FFF) as u32;
                let h = (u16::from_le_bytes([raw[28], raw[29]]) & 0x3FFF) as u32;
                return validate_image_dims(w, h, ctx);
            }
            _ => {}
        }
    }
    // JPEG: SOI (FF D8) followed by marker segments until SOF
    if raw.len() >= 4 && raw[0] == 0xFF && raw[1] == 0xD8 {
        let mut i = 2usize;
        while i + 3 < raw.len() {
            if raw[i] != 0xFF {
                break;
            }
            while i < raw.len() && raw[i] == 0xFF {
                i += 1;
            }
            if i >= raw.len() {
                break;
            }
            let marker = raw[i];
            i += 1;
            if marker == 0xD9 || marker == 0xDA {
                break;
            }
            if (0xD0..=0xD8).contains(&marker) || marker == 0x01 {
                continue;
            }
            if i + 1 >= raw.len() {
                break;
            }
            let seg_len = u16::from_be_bytes([raw[i], raw[i + 1]]) as usize;
            if seg_len < 2 || i + seg_len > raw.len() {
                break;
            }
            let is_sof = (0xC0..=0xCF).contains(&marker)
                && marker != 0xC4
                && marker != 0xC8
                && marker != 0xCC;
            if is_sof && seg_len >= 7 && i + 7 <= raw.len() {
                let h = u16::from_be_bytes([raw[i + 3], raw[i + 4]]) as u32;
                let w = u16::from_be_bytes([raw[i + 5], raw[i + 6]]) as u32;
                return validate_image_dims(w, h, ctx);
            }
            i += seg_len;
        }
    }
    Ok(())
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

    fn mark_surface_dirty(&mut self, surface_id: u32) {
        if let Some(sid) = self.surface_id(surface_id) {
            if let Some(s) = self.host.surface_mut(sid) {
                s.mark_dirty();
            }
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
            // does not force 60 Hz full-surface CPU readbacks; skipped entirely if glow is disabled).
            let glow_enabled = hnd.state.extensions.theme.glow_intensity > 0.001;
            let glow_threshold = if hnd.state.host.is_vr { 10.0 } else { 2.0 };
            let glow_moved = glow_enabled
                && (lx - updated_drag.last_glow_pos[0]).hypot(ly - updated_drag.last_glow_pos[1])
                    > glow_threshold;
            if !updated_drag.is_dragging_scroll {
                surf.ui_mut().on_pointer_move(lx, ly);
            }
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
                    // Cancel the underlying button press so releasing after a drag-scroll does not trigger a click,
                    // and anchor pointer_pos at press_pos so out-of-bounds drags still target the scroll container.
                    surf.ui_mut().on_pointer_cancel();
                    surf.ui_mut()
                        .on_pointer_move(updated_drag.press_pos[0], updated_drag.press_pos[1]);
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

fn reset_pointer_drag_and_preview(hnd: &mut HostHandle, surface_id: u32) {
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
}

#[no_mangle]
pub unsafe extern "C" fn lumenic_cvr_pointer_leave(handle: *mut HostHandle, surface_id: u32) {
    let Some(hnd) = (unsafe { handle.as_mut() }) else {
        return;
    };
    ffi_void(|| {
        reset_pointer_drag_and_preview(hnd, surface_id);
        if let Some(lh) = hnd.last_hovered_mut(surface_id) {
            *lh = None;
        }
        if let Some(sid) = hnd.surface_id(surface_id) {
            if let Some(surf) = hnd.host.surface_mut(sid) {
                surf.ui_mut().on_pointer_leave();
            }
        }
    });
}

#[no_mangle]
pub unsafe extern "C" fn lumenic_cvr_pointer_cancel(handle: *mut HostHandle, surface_id: u32) {
    let Some(hnd) = (unsafe { handle.as_mut() }) else {
        return;
    };
    ffi_void(|| {
        reset_pointer_drag_and_preview(hnd, surface_id);
        if let Some(sid) = hnd.surface_id(surface_id) {
            if let Some(surf) = hnd.host.surface_mut(sid) {
                surf.ui_mut().on_pointer_cancel();
            }
        }
    });
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
        if hnd.state.ext_sliders.len() > 256 {
            hnd.state.ext_sliders.clear();
        }
        if hnd.state.ext_text_inputs.len() > 256 {
            hnd.state.ext_text_inputs.clear();
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
    ffi_bool(|| {
        let field_bytes = unsafe { checked_slice(field_id_ptr, field_id_len)? };
        let field_id =
            std::str::from_utf8(field_bytes).map_err(|e| format!("invalid field_id utf8: {e}"))?;
        let text = if text_ptr.is_null() || text_len == 0 {
            ""
        } else {
            let text_bytes = unsafe { checked_slice(text_ptr, text_len)? };
            std::str::from_utf8(text_bytes).map_err(|e| format!("invalid text utf8: {e}"))?
        };

        crate::virtual_keyboard::commit_virtual_keyboard(
            &mut hnd.state,
            &mut hnd.actions,
            field_id,
            text,
        );

        if let Some(sid) = hnd.surface_id(surface_id) {
            if let Some(surf) = hnd.host.surface_mut(sid) {
                surf.ui_mut().clear_focus();
                surf.mark_dirty();
            }
        }
        Ok(())
    })
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
        check_image_header_dimensions(raw_bytes, &format!("decode_image({key})"))?;
        let decoded =
            lumenic::decode_image(raw_bytes).map_err(|e| format!("decode_image({key}): {e}"))?;
        validate_image_dims(decoded.width, decoded.height, &format!("decode_image({key})"))?;

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
pub unsafe extern "C" fn lumenic_cvr_register_font(
    name_ptr: *const u8,
    name_len: usize,
    bytes_ptr: *const u8,
    bytes_len: usize,
) -> bool {
    clear_last_error();
    if bytes_ptr.is_null() || bytes_len == 0 {
        set_last_error("lumenic_cvr_register_font: empty font bytes");
        return false;
    }
    let res = catch_unwind(AssertUnwindSafe(|| -> Result<(), String> {
        let raw_bytes = unsafe { checked_slice(bytes_ptr, bytes_len)? };
        if name_ptr.is_null() || name_len == 0 {
            lumenic::register_font_bytes(raw_bytes);
        } else {
            let name_bytes = unsafe { checked_slice(name_ptr, name_len)? };
            let name = std::str::from_utf8(name_bytes)
                .map_err(|e| format!("invalid font name utf8: {e}"))?;
            lumenic::register_font(name, raw_bytes);
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
pub unsafe extern "C" fn lumenic_cvr_set_default_font_family(
    name_ptr: *const u8,
    name_len: usize,
) -> bool {
    clear_last_error();
    if name_ptr.is_null() || name_len == 0 {
        set_last_error("lumenic_cvr_set_default_font_family: empty name");
        return false;
    }
    let res = catch_unwind(AssertUnwindSafe(|| -> Result<(), String> {
        let name_bytes = unsafe { checked_slice(name_ptr, name_len)? };
        let name = std::str::from_utf8(name_bytes)
            .map_err(|e| format!("invalid font family utf8: {e}"))?;
        lumenic::set_default_font_family(name);
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
pub unsafe extern "C" fn lumenic_cvr_push_domain_json(
    handle: *mut HostHandle,
    domain_id: u32,
    utf8_ptr: *const u8,
    len: usize,
) -> bool {
    clear_last_error();
    let Some(hnd) = (unsafe { handle.as_mut() }) else {
        set_last_error("lumenic_cvr_push_domain_json: null handle");
        return false;
    };
    if utf8_ptr.is_null() || len == 0 {
        set_last_error("lumenic_cvr_push_domain_json: empty payload");
        return false;
    }
    let res = catch_unwind(AssertUnwindSafe(|| -> Result<(), String> {
        let bytes = unsafe { checked_slice(utf8_ptr, len)? };
        match domain_id {
            DOMAIN_CORE => {
                let patch: crate::state::CoreStatePatch = serde_json::from_slice(bytes)
                    .map_err(|e| format!("core JSON parse error: {e}"))?;
                hnd.state.host.apply_core_patch(patch);
                hnd.mark_all_dirty();
            }
            DOMAIN_STATIC_CATALOG => {
                let catalog: crate::state::StaticCatalogSnapshot = serde_json::from_slice(bytes)
                    .map_err(|e| format!("static catalog JSON parse error: {e}"))?;
                hnd.state.host.player_color_presets = catalog.player_color_presets;
                hnd.state.host.player_color_swatches = catalog.player_color_swatches;
                hnd.state.host.whats_new_notes = catalog.whats_new_notes;
                hnd.mark_surface_dirty(SURFACE_MAIN);
            }
            DOMAIN_UILIB => {
                let uilib: crate::state::UiLibSnapshot = serde_json::from_slice(bytes)
                    .map_err(|e| format!("uilib JSON parse error: {e}"))?;
                hnd.state.host.uilib = uilib;
                hnd.mark_surface_dirty(SURFACE_QUICK);
            }
            DOMAIN_CONTENT => {
                let content: crate::state::ContentBrowserSnapshot = serde_json::from_slice(bytes)
                    .map_err(|e| format!("content JSON parse error: {e}"))?;
                let qm_changed = hnd.state.host.content.pinned_items != content.pinned_items
                    || hnd.state.host.content.group_mod_prompt != content.group_mod_prompt;
                hnd.state.host.content = content;
                hnd.mark_surface_dirty(SURFACE_MAIN);
                if qm_changed {
                    hnd.mark_surface_dirty(SURFACE_QUICK);
                }
            }
            DOMAIN_SOCIAL => {
                let social_dom: crate::state::SocialDomainSnapshot = serde_json::from_slice(bytes)
                    .map_err(|e| format!("social JSON parse error: {e}"))?;
                hnd.state.host.online_friends = social_dom.online_friends;
                hnd.state.host.active_worlds = social_dom.active_worlds;
                hnd.state.host.activity_feed = social_dom.activity_feed;
                hnd.state.host.social = social_dom.social;
                hnd.mark_surface_dirty(SURFACE_MAIN);
            }
            DOMAIN_SETTINGS => {
                let settings: crate::state::CvrSettingsSnapshot = serde_json::from_slice(bytes)
                    .map_err(|e| format!("settings JSON parse error: {e}"))?;
                hnd.state.host.cvr_settings = settings;
                hnd.mark_surface_dirty(SURFACE_MAIN);
            }
            DOMAIN_AVATAR_SETTINGS => {
                let avatar_settings: crate::state::AvatarSettingsSnapshot =
                    serde_json::from_slice(bytes)
                        .map_err(|e| format!("avatar_settings JSON parse error: {e}"))?;
                hnd.state.host.avatar_settings = avatar_settings;
                hnd.mark_surface_dirty(SURFACE_MAIN);
            }
            DOMAIN_UIX => {
                let uix: crate::state::UixDomainSnapshot = serde_json::from_slice(bytes)
                    .map_err(|e| format!("uix JSON parse error: {e}"))?;
                hnd.state.host.uix_available = uix.uix_available;
                hnd.state.host.mod_settings_source = uix.mod_settings_source;
                hnd.state.host.mod_settings_both_available = uix.mod_settings_both_available;
                hnd.state.host.uix_categories = uix.uix_categories;
                hnd.mark_surface_dirty(SURFACE_MAIN);
            }
            _ => return Err(format!("invalid domain_id: {domain_id}")),
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
pub unsafe extern "C" fn lumenic_cvr_push_extensions_json(
    handle: *mut HostHandle,
    utf8_ptr: *const u8,
    len: usize,
) -> bool {
    clear_last_error();
    let Some(hnd) = (unsafe { handle.as_mut() }) else {
        set_last_error("lumenic_cvr_push_extensions_json: null handle");
        return false;
    };
    if utf8_ptr.is_null() || len == 0 {
        set_last_error("lumenic_cvr_push_extensions_json: empty payload");
        return false;
    }
    let res = catch_unwind(AssertUnwindSafe(|| -> Result<(), String> {
        let bytes = unsafe { checked_slice(utf8_ptr, len)? };
        let snapshot: crate::ext::ExtensionsSnapshot =
            serde_json::from_slice(bytes).map_err(|e| format!("extensions JSON parse error: {e}"))?;
        hnd.state.extensions = snapshot;
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

/// Stateless, thread-safe image decoder callable from C# background worker threads
/// without touching the WGL context or `HostHandle`.
#[no_mangle]
pub unsafe extern "C" fn lumenic_cvr_decode_image(
    bytes_ptr: *const u8,
    bytes_len: usize,
    out_width: *mut u32,
    out_height: *mut u32,
    out_rgba_ptr: *mut *mut u8,
    out_rgba_len: *mut usize,
) -> bool {
    if !out_width.is_null() {
        unsafe { *out_width = 0 };
    }
    if !out_height.is_null() {
        unsafe { *out_height = 0 };
    }
    if !out_rgba_ptr.is_null() {
        unsafe { *out_rgba_ptr = std::ptr::null_mut() };
    }
    if !out_rgba_len.is_null() {
        unsafe { *out_rgba_len = 0 };
    }
    if bytes_ptr.is_null()
        || bytes_len == 0
        || out_width.is_null()
        || out_height.is_null()
        || out_rgba_ptr.is_null()
        || out_rgba_len.is_null()
    {
        return false;
    }

    let res = catch_unwind(AssertUnwindSafe(|| -> Result<(), String> {
        let raw_bytes = unsafe { checked_slice(bytes_ptr, bytes_len)? };
        check_image_header_dimensions(raw_bytes, "decode_image")?;
        let decoded =
            lumenic::decode_image(raw_bytes).map_err(|e| format!("decode_image: {e}"))?;
        validate_image_dims(decoded.width, decoded.height, "decode_image")?;
        let mut boxed = decoded.data.into_boxed_slice();
        let len = boxed.len();
        let ptr = boxed.as_mut_ptr();
        std::mem::forget(boxed);
        unsafe {
            *out_width = decoded.width;
            *out_height = decoded.height;
            *out_rgba_ptr = ptr;
            *out_rgba_len = len;
        }
        Ok(())
    }));

    matches!(res, Ok(Ok(())))
}

#[no_mangle]
pub unsafe extern "C" fn lumenic_cvr_free_decoded(ptr: *mut u8, len: usize) {
    if ptr.is_null() || len == 0 {
        return;
    }
    let _ = catch_unwind(AssertUnwindSafe(|| {
        let slice_ptr = std::ptr::slice_from_raw_parts_mut(ptr, len);
        drop(unsafe { Box::from_raw(slice_ptr) });
    }));
}

#[repr(C)]
pub struct RgbaUploadItem {
    pub surface_id: u32,
    pub width: u32,
    pub height: u32,
    pub key_ptr: *const u8,
    pub key_len: usize,
    pub rgba_ptr: *const u8,
    pub rgba_len: usize,
}

/// Uploads a batch of pre-decoded RGBA8 textures inside a single WGL context bind.
/// Writes `1` (success) or `0` (failure) into `out_results[i]` when non-null.
#[no_mangle]
pub unsafe extern "C" fn lumenic_cvr_upload_rgba_batch(
    handle: *mut HostHandle,
    items_ptr: *const RgbaUploadItem,
    count: usize,
    out_results_ptr: *mut u8,
) -> usize {
    clear_last_error();
    let Some(hnd) = (unsafe { handle.as_mut() }) else {
        set_last_error("lumenic_cvr_upload_rgba_batch: null handle");
        return 0;
    };
    if items_ptr.is_null() || count == 0 {
        return 0;
    }
    let res = catch_unwind(AssertUnwindSafe(|| -> Result<usize, String> {
        let items_bytes = count
            .checked_mul(std::mem::size_of::<RgbaUploadItem>())
            .ok_or_else(|| String::from("rgba batch count overflow"))?;
        if items_bytes > isize::MAX as usize || count > isize::MAX as usize {
            return Err(format!("rgba batch count exceeds isize::MAX: {count}"));
        }
        let items = unsafe { slice::from_raw_parts(items_ptr, count) };
        let results = if out_results_ptr.is_null() {
            None
        } else {
            Some(unsafe { slice::from_raw_parts_mut(out_results_ptr, count) })
        };

        let HostHandle {
            host,
            state,
            main_surface,
            quick_surface,
            ..
        } = hnd;
        let guard = MakeNotCurrentGuard(host);
        let mut uploaded = 0usize;
        let mut results = results;

        for (i, item) in items.iter().enumerate() {
            let mut ok = false;
            let sid = match item.surface_id {
                SURFACE_MAIN => Some(*main_surface),
                SURFACE_QUICK => Some(*quick_surface),
                _ => None,
            };
            let dims_ok = validate_image_dims(item.width, item.height, "upload_rgba_batch").is_ok();
            let expected_len = (item.width as usize)
                .checked_mul(item.height as usize)
                .and_then(|px| px.checked_mul(4));

            if let (Some(sid), Some(expected_len)) = (sid, expected_len) {
                if dims_ok
                    && !item.key_ptr.is_null()
                    && item.key_len > 0
                    && !item.rgba_ptr.is_null()
                    && item.rgba_len >= expected_len
                {
                    if let (Ok(key_bytes), Ok(rgba)) = (
                        unsafe { checked_slice(item.key_ptr, item.key_len) },
                        unsafe { checked_slice(item.rgba_ptr, expected_len) },
                    ) {
                        if let Ok(key) = std::str::from_utf8(key_bytes) {
                            if let Some(surf) = guard.0.surface_mut(sid) {
                                if let Ok(tex_id) = surf.app_mut().create_texture_rgba(
                                    item.width,
                                    item.height,
                                    rgba,
                                ) {
                                    let cache = if item.surface_id == SURFACE_MAIN {
                                        &mut state.main_textures
                                    } else {
                                        &mut state.quick_textures
                                    };
                                    cache.insert(surf.app_mut(), key.to_owned(), tex_id);
                                    surf.mark_dirty();
                                    ok = true;
                                    uploaded += 1;
                                }
                            }
                        }
                    }
                }
            }
            if let Some(res_slice) = results.as_deref_mut() {
                res_slice[i] = if ok { 1 } else { 0 };
            }
        }
        Ok(uploaded)
    }));

    match res {
        Ok(Ok(n)) => n,
        Ok(Err(err)) => {
            set_last_error(err);
            0
        }
        Err(payload) => {
            set_last_error(panic_payload_to_string(payload));
            0
        }
    }
}

#[no_mangle]
pub unsafe extern "C" fn lumenic_cvr_upload_rgba_texture(
    handle: *mut HostHandle,
    surface_id: u32,
    key_ptr: *const u8,
    key_len: usize,
    width: u32,
    height: u32,
    rgba_ptr: *const u8,
    rgba_len: usize,
) -> bool {
    clear_last_error();
    let Some(hnd) = (unsafe { handle.as_mut() }) else {
        set_last_error("lumenic_cvr_upload_rgba_texture: null handle");
        return false;
    };
    if key_ptr.is_null() || key_len == 0 || rgba_ptr.is_null() || width == 0 || height == 0 {
        set_last_error("lumenic_cvr_upload_rgba_texture: invalid arguments");
        return false;
    }
    if let Err(err) = validate_image_dims(width, height, "lumenic_cvr_upload_rgba_texture") {
        set_last_error(err);
        return false;
    }
    let Some(expected_len) = (width as usize)
        .checked_mul(height as usize)
        .and_then(|px| px.checked_mul(4))
    else {
        set_last_error("lumenic_cvr_upload_rgba_texture: dimensions overflow");
        return false;
    };
    if rgba_len < expected_len {
        set_last_error("lumenic_cvr_upload_rgba_texture: rgba buffer too small");
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
        let rgba = unsafe { checked_slice(rgba_ptr, expected_len)? };

        let HostHandle { host, state, .. } = hnd;
        let guard = MakeNotCurrentGuard(host);
        let surf = guard
            .0
            .surface_mut(sid)
            .ok_or_else(|| String::from("surface missing"))?;
        let tex_id = surf
            .app_mut()
            .create_texture_rgba(width, height, rgba)
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

fn step_surface_inner(
    hnd: &mut HostHandle,
    surface_id: u32,
    dt_secs: f32,
    force_redraw: bool,
    dst_ptr: *mut u8,
    dst_len: usize,
) -> Result<u32, String> {
    let safe_dt = if dt_secs.is_finite() && dt_secs > 0.0 {
        dt_secs.min(0.25)
    } else {
        0.0
    };
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

    let (rendered, pixels_read) = {
        let guard = MakeNotCurrentGuard(host);
        let step_result = if force_redraw {
            guard.0.step_frame_forced(sid, safe_dt, render_fn)
        } else {
            guard.0.step_frame(sid, safe_dt, render_fn)
        };
        let rendered = step_result.map_err(|e| format!("step_frame: {e}"))?;
        let mut pixels_read = false;
        if (rendered || force_redraw) && !dst_ptr.is_null() && dst_len > 0 {
            let expected = guard
                .0
                .surface(sid)
                .map(|s| s.rgba_byte_len())
                .ok_or_else(|| String::from("surface missing"))?;
            if dst_len < expected {
                return Err(format!(
                    "buffer too small: got {dst_len} bytes, need {expected}"
                ));
            }
            let dst = unsafe { checked_slice_mut(dst_ptr, expected)? };
            guard
                .0
                .read_pixels_rgba_into(sid, dst, true)
                .map_err(|e| format!("read_pixels_rgba_into: {e}"))?;
            pixels_read = true;
        }
        (rendered, pixels_read)
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
    if pixels_read {
        flags |= STEP_FLAG_PIXELS_READ;
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
    let res = catch_unwind(AssertUnwindSafe(|| {
        step_surface_inner(hnd, surface_id, dt_secs, force_redraw, std::ptr::null_mut(), 0)
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

/// Steps the UI surface and, if rendered (or forced), reads RGBA8 pixels into `dst_ptr`
/// within the same WGL context bind to avoid redundant context switches.
#[no_mangle]
pub unsafe extern "C" fn lumenic_cvr_step_and_read(
    handle: *mut HostHandle,
    surface_id: u32,
    dt_secs: f32,
    force_redraw: bool,
    dst_ptr: *mut u8,
    dst_len: usize,
) -> u32 {
    clear_last_error();
    let Some(hnd) = (unsafe { handle.as_mut() }) else {
        set_last_error("lumenic_cvr_step_and_read: null handle");
        return STEP_FLAG_ERROR_OR_PANIC;
    };
    let res = catch_unwind(AssertUnwindSafe(|| {
        step_surface_inner(hnd, surface_id, dt_secs, force_redraw, dst_ptr, dst_len)
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
                        status_dots: Vec::new(),
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
    fn ffi_quick_menu_drag_scroll_outside_bounds() {
        let handle = lumenic_cvr_create(1280, 720, 800, 800, 1.0);
        assert!(!handle.is_null());

        unsafe {
            {
                let hnd = &mut *handle;
                hnd.state.host.notifications = (0..12)
                    .map(|i| state::NotificationItem {
                        id: format!("notif_{i}"),
                        title: format!("Notification {i}"),
                        body: format!("Body {i}"),
                        category: String::from("info"),
                        buttons: Vec::new(),
                    })
                    .collect();
            }

            let _ = lumenic_cvr_step(handle, SURFACE_QUICK, 0.016, true);
            let _ = lumenic_cvr_step(handle, SURFACE_QUICK, 0.016, true);

            let (scroll_rect, max_y) = {
                let hnd = &*handle;
                let surf = hnd.host.surface(hnd.quick_surface).unwrap();
                let rect = surf
                    .ui()
                    .semantics()
                    .iter()
                    .find(|n| n.label.contains("Notification 0"))
                    .map(|n| n.bounds)
                    .expect("expected Notification 0 semantic node");
                (rect, surf.ui().scroll_max_y("qm_notifications_scroll"))
            };
            assert!(max_y > 10.0, "expected vertical overflow, got max_y={max_y}");

            let start_x = scroll_rect.x + scroll_rect.w * 0.5;
            let start_y = scroll_rect.y + scroll_rect.h * 0.5;
            lumenic_cvr_pointer_move(handle, SURFACE_QUICK, start_x, start_y);
            lumenic_cvr_pointer_down(handle, SURFACE_QUICK, 0);
            // Drag outside surface bounds (x = -120, y = -80)
            for step in 1..=6 {
                lumenic_cvr_pointer_move(
                    handle,
                    SURFACE_QUICK,
                    start_x - (step as f32) * 80.0,
                    start_y - (step as f32) * 90.0,
                );
            }
            lumenic_cvr_pointer_up(handle, SURFACE_QUICK, 0);

            let _ = lumenic_cvr_step(handle, SURFACE_QUICK, 0.016, true);
            let _ = lumenic_cvr_step(handle, SURFACE_QUICK, 0.016, true);

            let offset_y = {
                let hnd = &*handle;
                let surf = hnd.host.surface(hnd.quick_surface).unwrap();
                surf.ui().scroll_offset("qm_notifications_scroll")
            };
            assert!(
                offset_y > 10.0,
                "expected out-of-bounds drag to scroll qm_notifications_scroll, got offset_y={offset_y}"
            );

            lumenic_cvr_destroy(handle);
        }
    }

    #[test]
    fn ffi_audit_max_font_scale_bounds() {
        let handle = lumenic_cvr_create(1280, 720, 800, 800, 1.0);
        assert!(!handle.is_null());

        unsafe {
            {
                let hnd = &mut *handle;
                hnd.state.host.font_scale = 1.5;
                hnd.state.host.username = String::from("LensErrorLongName");
                hnd.state.host.user_rank = String::from("Community Guide");
                hnd.state.host.featured_badge = String::from("Founder Badge");
                hnd.state.host.featured_group = String::from("Lumenic Builders");
                hnd.state.host.world_name = String::from("ChilloutVR Home World");
                hnd.state.host.notifications = (0..5)
                    .map(|i| state::NotificationItem {
                        id: format!("n_{i}"),
                        title: format!("Notification Title {i}"),
                        body: format!("Notification body text {i}"),
                        category: String::from("info"),
                        buttons: Vec::new(),
                    })
                    .collect();
            }

            // Audit Quick Menu (800x800) & Main Menu (1280x720) for Button-vs-Label overflow at font_scale=1.5
            let check_surface = |name: &str, surf: &lumenic::OffscreenSurface, vw: f32| {
                let nodes = surf.ui().semantics();
                let mut issues = Vec::new();
                let buttons: Vec<_> = nodes
                    .iter()
                    .filter(|n| n.role == lumenic::Role::Button)
                    .collect();
                for n in nodes {
                    let b = n.bounds;
                    if n.role == lumenic::Role::Label
                        && !n.label.is_empty()
                        && !n.label.starts_with("Notification ")
                        && !(name == "MM[settings]" && b.y >= 615.0)
                    {
                        let mut best_btn: Option<&&lumenic::SemanticNode> = None;
                        for btn in &buttons {
                            let bb = btn.bounds;
                            let cx = b.x + b.w * 0.5;
                            let cy = b.y + b.h * 0.5;
                            if cx >= bb.x - 4.0 && cx <= bb.x + bb.w + 4.0 && cy >= bb.y - 4.0 && cy <= bb.y + bb.h + 4.0 {
                                if best_btn.is_none_or(|prev| bb.w * bb.h < prev.bounds.w * prev.bounds.h) {
                                    best_btn = Some(btn);
                                }
                            }
                        }
                        if let Some(btn) = best_btn {
                            let bb = btn.bounds;
                            let ov_x = (b.x < bb.x - 1.5) || (b.x + b.w > bb.x + bb.w + 1.5);
                            let ov_y = (b.y < bb.y - 1.5) || (b.y + b.h > bb.y + bb.h + 1.5);
                            if ov_x || ov_y {
                                issues.push(format!(
                                    "{name} BTN-CLIP '{}' lbl=[{:.0},{:.0},{:.0}x{:.0}] btn=[{:.0},{:.0},{:.0}x{:.0}]",
                                    n.label, b.x, b.y, b.w, b.h, bb.x, bb.y, bb.w, bb.h
                                ));
                            }
                        }
                    }
                    if b.x < -1.0 || b.x + b.w > vw + 1.0 {
                        issues.push(format!(
                            "{name} X-OVF {:?} '{}' [{:.0},{:.0},{:.0}x{:.0}]",
                            n.role, n.label, b.x, b.y, b.w, b.h
                        ));
                    }
                }
                assert!(
                    issues.is_empty(),
                    "{name} has max font_scale (1.5) layout issues:\n{}",
                    issues.join("\n")
                );
            };

            let _ = lumenic_cvr_step(handle, SURFACE_QUICK, 0.016, true);
            let _ = lumenic_cvr_step(handle, SURFACE_QUICK, 0.016, true);
            check_surface("QM", (&*handle).host.surface((&*handle).quick_surface).unwrap(), 800.0);

            for tab in ["home", "friends", "worlds", "avatars", "props", "settings"] {
                (&mut *handle).state.main_menu.active_tab = String::from(tab);
                let _ = lumenic_cvr_step(handle, SURFACE_MAIN, 0.016, true);
                let _ = lumenic_cvr_step(handle, SURFACE_MAIN, 0.016, true);
                check_surface(
                    &format!("MM[{tab}]"),
                    (&*handle).host.surface((&*handle).main_surface).unwrap(),
                    1280.0,
                );
            }

            lumenic_cvr_destroy(handle);
        }
    }

    #[test]
    fn ffi_slider_drag_updates_value() {
        let handle = lumenic_cvr_create(1280, 720, 800, 800, 1.0);
        assert!(!handle.is_null());

        unsafe {
            {
                let hnd = &mut *handle;
                hnd.state.host.colorize_menu = true;
                hnd.state.host.accent_r = 40;
                hnd.state.host.accent_g = 140;
                hnd.state.host.accent_b = 60;
                hnd.state.host.accent2_r = 255;
                hnd.state.host.accent2_g = 80;
                hnd.state.host.accent2_b = 0;
                hnd.state.host.audio_master = 50;
                hnd.state.quick.master_slider = 50.0;
                hnd.state.quick.last_host_master = 50;
            }
            let _ = lumenic_cvr_step(handle, SURFACE_QUICK, 0.016, true);
            let _ = lumenic_cvr_step(handle, SURFACE_QUICK, 0.016, true);

            let slider_rect = {
                let hnd = &*handle;
                let surf = hnd.host.surface(hnd.quick_surface).unwrap();
                let sems = surf.ui().semantics();
                assert!(
                    !sems
                        .iter()
                        .any(|n| n.role != lumenic::Role::Slider && n.label == "__track"),
                    "compact slider must not render internal __track header label"
                );
                let rect = sems
                    .iter()
                    .find(|n| n.role == lumenic::Role::Slider)
                    .map(|n| n.bounds)
                    .expect("expected Master Volume slider");
                assert!(
                    rect.h <= 22.0,
                    "compact slider track height should be ~14px without built-in header, got {}",
                    rect.h
                );
                rect
            };
            {
                let mut px = vec![0u8; 800 * 800 * 4];
                assert!(lumenic_cvr_read_pixels(handle, SURFACE_QUICK, px.as_mut_ptr(), px.len()));
                let fy = 799usize.saturating_sub((slider_rect.y + slider_rect.h * 0.5) as usize);
                let fill_x = (slider_rect.x + slider_rect.w * 0.25) as usize;
                let fill_idx = (fy * 800 + fill_x) * 4;
                let (fr, fg, fb) = (px[fill_idx], px[fill_idx + 1], px[fill_idx + 2]);
                assert!(
                    fr > 180 && fb < 60,
                    "expected slider active fill to use secondary player color (orange), got rgb({fr},{fg},{fb})"
                );
                let bg_x = (slider_rect.x + slider_rect.w * 0.75) as usize;
                let bg_idx = (fy * 800 + bg_x) * 4;
                let (br, bg, bb) = (px[bg_idx], px[bg_idx + 1], px[bg_idx + 2]);
                assert!(
                    bg >= br && bg >= bb && bb < 120,
                    "expected slider track background to use primary player color (green tint), got rgb({br},{bg},{bb})"
                );
            }

            let start_x = slider_rect.x + slider_rect.w * 0.5;
            let cy = slider_rect.y + slider_rect.h * 0.5;
            lumenic_cvr_pointer_move(handle, SURFACE_QUICK, start_x, cy);
            let _ = lumenic_cvr_step(handle, SURFACE_QUICK, 0.016, false);
            lumenic_cvr_pointer_down(handle, SURFACE_QUICK, 0);
            let _ = lumenic_cvr_step(handle, SURFACE_QUICK, 0.016, false);

            for i in 1..=5 {
                lumenic_cvr_pointer_move(handle, SURFACE_QUICK, start_x + (i as f32) * 16.0, cy);
                let _ = lumenic_cvr_step(handle, SURFACE_QUICK, 0.016, false);
            }
            let val_after = (&*handle).state.quick.master_slider;
            lumenic_cvr_pointer_up(handle, SURFACE_QUICK, 0);

            assert!(val_after > 65.0, "dragging QM slider right should increase value past 65, got {val_after}");

            // Also verify MM Settings slider (previously broke drag due to dynamic label WidgetId)
            (&mut *handle).state.main_menu.active_tab = String::from("settings");
            let _ = lumenic_cvr_step(handle, SURFACE_MAIN, 0.016, true);
            let _ = lumenic_cvr_step(handle, SURFACE_MAIN, 0.016, true);
            let mm_slider_rect = {
                let hnd = &*handle;
                let surf = hnd.host.surface(hnd.main_surface).unwrap();
                surf.ui()
                    .semantics()
                    .iter()
                    .find(|n| n.role == lumenic::Role::Slider)
                    .map(|n| n.bounds)
                    .expect("expected Settings slider")
            };
            let mm_sx = mm_slider_rect.x + mm_slider_rect.w * 0.35;
            let mm_cy = mm_slider_rect.y + mm_slider_rect.h * 0.5;
            (&mut *handle).actions.clear();
            lumenic_cvr_pointer_move(handle, SURFACE_MAIN, mm_sx, mm_cy);
            let _ = lumenic_cvr_step(handle, SURFACE_MAIN, 0.016, false);
            lumenic_cvr_pointer_down(handle, SURFACE_MAIN, 0);
            let _ = lumenic_cvr_step(handle, SURFACE_MAIN, 0.016, false);
            for i in 1..=5 {
                lumenic_cvr_pointer_move(handle, SURFACE_MAIN, mm_sx + (i as f32) * 40.0, mm_cy);
                let _ = lumenic_cvr_step(handle, SURFACE_MAIN, 0.016, false);
            }
            lumenic_cvr_pointer_up(handle, SURFACE_MAIN, 0);
            let slider_updates: Vec<i32> = (&*handle)
                .actions
                .iter()
                .filter_map(|a| match a {
                    state::UiAction::SetMaxRefreshHz { hz } => Some(*hz),
                    state::UiAction::SetPanelOpacity { opacity } => {
                        Some((*opacity * 100.0).round() as i32)
                    }
                    _ => None,
                })
                .collect();
            assert!(
                slider_updates.len() >= 3,
                "dragging MM Settings slider should continuously emit updates across frames, got {slider_updates:?}"
            );
            lumenic_cvr_destroy(handle);
        }
    }

    #[test]
    fn ffi_quick_menu_wheel_scroll_without_click() {
        let handle = lumenic_cvr_create(1280, 720, 800, 800, 1.0);
        assert!(!handle.is_null());

        unsafe {
            {
                let hnd = &mut *handle;
                hnd.state.host.notifications = (0..12)
                    .map(|i| state::NotificationItem {
                        id: format!("notif_{i}"),
                        title: format!("Notification {i}"),
                        body: format!("Body {i}"),
                        category: String::from("info"),
                        buttons: Vec::new(),
                    })
                    .collect();
            }

            let _ = lumenic_cvr_step(handle, SURFACE_QUICK, 0.016, true);
            let _ = lumenic_cvr_step(handle, SURFACE_QUICK, 0.016, true);

            let (scroll_rect, max_y) = {
                let hnd = &*handle;
                let surf = hnd.host.surface(hnd.quick_surface).unwrap();
                let rect = surf
                    .ui()
                    .semantics()
                    .iter()
                    .find(|n| n.label.contains("Notification 0"))
                    .map(|n| n.bounds)
                    .expect("expected Notification 0 semantic node");
                (rect, surf.ui().scroll_max_y("qm_notifications_scroll"))
            };
            assert!(max_y > 10.0, "expected vertical overflow on qm_notifications_scroll, got max_y={max_y}");

            let cx = scroll_rect.x + scroll_rect.w * 0.5;
            let cy = scroll_rect.y + scroll_rect.h * 0.5;
            lumenic_cvr_pointer_move(handle, SURFACE_QUICK, cx, cy);
            lumenic_cvr_pointer_scroll(handle, SURFACE_QUICK, 0.0, -3.0);

            let flags = lumenic_cvr_step(handle, SURFACE_QUICK, 0.016, false);
            let offset_y = {
                let hnd = &*handle;
                let surf = hnd.host.surface(hnd.quick_surface).unwrap();
                surf.ui().scroll_offset("qm_notifications_scroll")
            };
            assert_ne!(flags & STEP_FLAG_RENDERED, 0, "scroll without click should render a frame");
            assert!(offset_y > 1.0, "scroll without click should scroll qm_notifications_scroll, got offset_y={offset_y}");

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

    #[test]
    fn ffi_step_and_read_and_upload_rgba_batch() {
        let handle = lumenic_cvr_create(320, 240, 256, 256, 1.0);
        assert!(!handle.is_null());

        unsafe {
            let key = b"test_batch_tex";
            let rgba = vec![255u8; 4 * 4 * 4];
            let item = RgbaUploadItem {
                surface_id: SURFACE_MAIN,
                width: 4,
                height: 4,
                key_ptr: key.as_ptr(),
                key_len: key.len(),
                rgba_ptr: rgba.as_ptr(),
                rgba_len: rgba.len(),
            };
            let mut results = [0u8; 1];
            let uploaded = lumenic_cvr_upload_rgba_batch(handle, &item, 1, results.as_mut_ptr());
            assert_eq!(uploaded, 1);
            assert_eq!(results[0], 1);

            let mut pixels = vec![0u8; 320 * 240 * 4];
            let flags = lumenic_cvr_step_and_read(
                handle,
                SURFACE_MAIN,
                0.016,
                true,
                pixels.as_mut_ptr(),
                pixels.len(),
            );
            assert_eq!(flags & STEP_FLAG_ERROR_OR_PANIC, 0);
            assert_ne!(flags & STEP_FLAG_RENDERED, 0);
            assert_ne!(flags & STEP_FLAG_PIXELS_READ, 0);
            let non_zero_alpha = pixels.chunks_exact(4).filter(|p| p[3] > 0).count();
            assert!(non_zero_alpha > 1_000);

            lumenic_cvr_destroy(handle);
        }
    }

    #[test]
    fn ffi_push_domain_json_dirties_only_target_surface() {
        let handle = lumenic_cvr_create(320, 240, 256, 256, 1.0);
        assert!(!handle.is_null());

        unsafe {
            // Settle both surfaces so idle frames skip rendering.
            for _ in 0..120 {
                let _ = lumenic_cvr_step(handle, SURFACE_MAIN, 0.016, false);
                let _ = lumenic_cvr_step(handle, SURFACE_QUICK, 0.016, false);
            }
            assert_eq!(lumenic_cvr_step(handle, SURFACE_MAIN, 0.016, false) & STEP_FLAG_RENDERED, 0);
            assert_eq!(lumenic_cvr_step(handle, SURFACE_QUICK, 0.016, false) & STEP_FLAG_RENDERED, 0);

            // Pushing UILib domain must dirty Quick Menu ONLY, never Main Menu.
            let uilib_json = br#"{"toast":"hello"}"#;
            assert!(lumenic_cvr_push_domain_json(
                handle,
                DOMAIN_UILIB,
                uilib_json.as_ptr(),
                uilib_json.len()
            ));
            assert_eq!(
                lumenic_cvr_step(handle, SURFACE_MAIN, 0.016, false) & STEP_FLAG_RENDERED,
                0,
                "DOMAIN_UILIB must not dirty Main Menu"
            );
            assert_ne!(
                lumenic_cvr_step(handle, SURFACE_QUICK, 0.016, false) & STEP_FLAG_RENDERED,
                0,
                "DOMAIN_UILIB must dirty Quick Menu"
            );

            // Settle Quick Menu again, then push Social domain -> must dirty Main Menu ONLY.
            for _ in 0..120 {
                let _ = lumenic_cvr_step(handle, SURFACE_QUICK, 0.016, false);
            }
            assert_eq!(lumenic_cvr_step(handle, SURFACE_QUICK, 0.016, false) & STEP_FLAG_RENDERED, 0);

            let social_json = br#"{"online_friends":[],"active_worlds":[],"activity_feed":[],"social":{}}"#;
            assert!(lumenic_cvr_push_domain_json(
                handle,
                DOMAIN_SOCIAL,
                social_json.as_ptr(),
                social_json.len()
            ));
            assert_eq!(
                lumenic_cvr_step(handle, SURFACE_QUICK, 0.016, false) & STEP_FLAG_RENDERED,
                0,
                "DOMAIN_SOCIAL must not dirty Quick Menu"
            );
            assert_ne!(
                lumenic_cvr_step(handle, SURFACE_MAIN, 0.016, false) & STEP_FLAG_RENDERED,
                0,
                "DOMAIN_SOCIAL must dirty Main Menu"
            );

            lumenic_cvr_destroy(handle);
        }
    }

    #[test]
    fn ffi_dropdown_modal_select_and_dismiss_closes_modal() {
        let handle = lumenic_cvr_create(1280, 720, 800, 800, 1.0);
        assert!(!handle.is_null());

        unsafe {
            {
                let hnd = &mut *handle;
                hnd.state.host.disclaimer_accepted = true;
                hnd.state.host.show_whats_new = false;
                hnd.state.main_menu.active_dropdown = Some(crate::state::ActiveDropdown {
                    title: String::from("TTS Voice"),
                    options: vec![
                        crate::state::DropdownOption {
                            label: String::from("Voice Alpha"),
                            selected: true,
                            action: UiAction::CvrSetString {
                                key: String::from("AudioTTSVoice"),
                                value: String::from("alpha"),
                            },
                        },
                        crate::state::DropdownOption {
                            label: String::from("Voice Beta"),
                            selected: false,
                            action: UiAction::CvrSetString {
                                key: String::from("AudioTTSVoice"),
                                value: String::from("beta"),
                            },
                        },
                    ],
                });
            }
            let _ = lumenic_cvr_step(handle, SURFACE_MAIN, 0.016, true);
            let _ = lumenic_cvr_step(handle, SURFACE_MAIN, 0.016, true);

            let opt_rect = {
                let hnd = &*handle;
                let surf = hnd.host.surface(hnd.main_surface).unwrap();
                surf.ui()
                    .semantics()
                    .iter()
                    .find(|n| n.label.contains("Voice Beta"))
                    .map(|n| n.bounds)
                    .expect("expected Voice Beta option in dropdown modal")
            };
            let cx = opt_rect.x + opt_rect.w * 0.5;
            let cy = opt_rect.y + opt_rect.h * 0.5;
            lumenic_cvr_pointer_move(handle, SURFACE_MAIN, cx, cy);
            let _ = lumenic_cvr_step(handle, SURFACE_MAIN, 0.016, false);
            lumenic_cvr_pointer_down(handle, SURFACE_MAIN, 0);
            let _ = lumenic_cvr_step(handle, SURFACE_MAIN, 0.016, false);
            lumenic_cvr_pointer_up(handle, SURFACE_MAIN, 0);
            let _ = lumenic_cvr_step(handle, SURFACE_MAIN, 0.016, false);

            assert!(
                (&*handle).state.main_menu.active_dropdown.is_none(),
                "clicking a dropdown option must close active_dropdown (no softlock)"
            );

            // Re-open and verify backdrop scrim click also closes it.
            {
                let hnd = &mut *handle;
                hnd.state.main_menu.active_dropdown = Some(crate::state::ActiveDropdown {
                    title: String::from("Preferred Display Mode"),
                    options: vec![crate::state::DropdownOption {
                        label: String::from("Above Head"),
                        selected: true,
                        action: UiAction::CvrSetInt {
                            key: String::from("AudioChatBoxDisplayMode"),
                            value: 0,
                        },
                    }],
                });
            }
            let _ = lumenic_cvr_step(handle, SURFACE_MAIN, 0.016, true);
            lumenic_cvr_pointer_move(handle, SURFACE_MAIN, 20.0, 20.0);
            let _ = lumenic_cvr_step(handle, SURFACE_MAIN, 0.016, false);
            lumenic_cvr_pointer_down(handle, SURFACE_MAIN, 0);
            let _ = lumenic_cvr_step(handle, SURFACE_MAIN, 0.016, false);
            lumenic_cvr_pointer_up(handle, SURFACE_MAIN, 0);
            let _ = lumenic_cvr_step(handle, SURFACE_MAIN, 0.016, false);

            assert!(
                (&*handle).state.main_menu.active_dropdown.is_none(),
                "clicking dropdown backdrop scrim must close active_dropdown"
            );

            lumenic_cvr_destroy(handle);
        }
    }

    #[test]
    fn image_header_precheck_rejects_oversized_png_and_allows_valid() {
        let mut png_bomb = Vec::from(&b"\x89PNG\r\n\x1a\n\x00\x00\x00\x0dIHDR"[..]);
        png_bomb.extend_from_slice(&30_000u32.to_be_bytes());
        png_bomb.extend_from_slice(&30_000u32.to_be_bytes());
        assert!(check_image_header_dimensions(&png_bomb, "test").is_err());

        let mut png_ok = Vec::from(&b"\x89PNG\r\n\x1a\n\x00\x00\x00\x0dIHDR"[..]);
        png_ok.extend_from_slice(&512u32.to_be_bytes());
        png_ok.extend_from_slice(&512u32.to_be_bytes());
        assert!(check_image_header_dimensions(&png_ok, "test").is_ok());

        let mut gif_bomb = Vec::from(&b"GIF89a"[..]);
        gif_bomb.extend_from_slice(&5000u16.to_le_bytes());
        gif_bomb.extend_from_slice(&5000u16.to_le_bytes());
        assert!(check_image_header_dimensions(&gif_bomb, "test").is_err());
    }
}
}

#[cfg(not(target_arch = "wasm32"))]
pub use native_ffi::*;
