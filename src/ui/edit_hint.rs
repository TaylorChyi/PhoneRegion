//! Paint the native edit's cue in the app palette while leaving editing to Windows.

use super::{
    DT_SINGLELINE, DT_VCENTER, GWLP_USERDATA, Handle, Hwnd, Lparam, Lresult, Rect, UiState, Wparam,
    draw_text,
};
use std::ffi::c_void;

const WM_PAINT: u32 = 0x000F;
const WM_PRINTCLIENT: u32 = 0x0318;
const WM_NCDESTROY: u32 = 0x0082;
const EM_SETMARGINS: u32 = 0x00D3;
const EC_LEFTMARGIN: usize = 1;
const EC_RIGHTMARGIN: usize = 2;
const SUBCLASS_ID: usize = 1;

#[link(name = "comctl32")]
unsafe extern "system" {
    fn SetWindowSubclass(
        hwnd: Hwnd,
        callback: Option<
            unsafe extern "system" fn(Hwnd, u32, Wparam, Lparam, usize, usize) -> Lresult,
        >,
        id: usize,
        data: usize,
    ) -> i32;
    fn DefSubclassProc(hwnd: Hwnd, message: u32, wparam: Wparam, lparam: Lparam) -> Lresult;
    fn RemoveWindowSubclass(
        hwnd: Hwnd,
        callback: Option<
            unsafe extern "system" fn(Hwnd, u32, Wparam, Lparam, usize, usize) -> Lresult,
        >,
        id: usize,
    ) -> i32;
}

#[link(name = "user32")]
unsafe extern "system" {
    fn GetClientRect(hwnd: Hwnd, rect: *mut Rect) -> i32;
    fn GetWindowTextLengthW(hwnd: Hwnd) -> i32;
    fn GetWindowLongPtrW(hwnd: Hwnd, index: i32) -> isize;
    fn GetDC(hwnd: Hwnd) -> Handle;
    fn ReleaseDC(hwnd: Hwnd, dc: Handle) -> i32;
    fn FillRect(dc: Handle, rect: *const Rect, brush: Handle) -> i32;
    fn HideCaret(hwnd: Hwnd) -> i32;
    fn ShowCaret(hwnd: Hwnd) -> i32;
}

unsafe fn paint_hint(edit: Hwnd, parent: Hwnd, dc: Handle) {
    if dc.is_null() || unsafe { GetWindowTextLengthW(edit) } != 0 {
        return;
    }
    let state = unsafe { GetWindowLongPtrW(parent, GWLP_USERDATA) as *const UiState };
    if state.is_null() {
        return;
    }
    let (brush, color, font, dpi) = unsafe {
        (
            (*state).theme.field_brush,
            (*state).theme.colors.muted,
            (*state).input_font,
            (*state).dpi,
        )
    };
    let mut rect = Rect {
        left: 0,
        top: 0,
        right: 0,
        bottom: 0,
    };
    unsafe {
        GetClientRect(edit, &mut rect);
    }
    let width = (rect.right - rect.left).max(0);
    let height = (rect.bottom - rect.top).max(0);
    if width == 0 || height == 0 {
        return;
    }
    unsafe {
        let caret_hidden = HideCaret(edit) != 0;
        FillRect(dc, &rect, brush);
        draw_text(
            dc,
            "例如：+86 132 1300 0000",
            font,
            color,
            [
                0.0,
                0.0,
                width as f32 * 96.0 / dpi as f32,
                height as f32 * 96.0 / dpi as f32,
            ],
            dpi,
            DT_SINGLELINE | DT_VCENTER,
        );
        if caret_hidden {
            ShowCaret(edit);
        }
    }
}

unsafe extern "system" fn edit_proc(
    hwnd: Hwnd,
    message: u32,
    wparam: Wparam,
    lparam: Lparam,
    _id: usize,
    data: usize,
) -> Lresult {
    if message == WM_NCDESTROY {
        unsafe {
            RemoveWindowSubclass(hwnd, Some(edit_proc), SUBCLASS_ID);
        }
        return unsafe { DefSubclassProc(hwnd, message, wparam, lparam) };
    }
    let result = unsafe { DefSubclassProc(hwnd, message, wparam, lparam) };
    if message == WM_PAINT {
        let dc = unsafe { GetDC(hwnd) };
        unsafe {
            paint_hint(hwnd, data as Hwnd, dc);
        }
        if !dc.is_null() {
            unsafe {
                ReleaseDC(hwnd, dc);
            }
        }
    } else if message == super::WM_SETFONT {
        // The enclosing field supplies both gutters, independent of the font.
        unsafe { super::SendMessageW(hwnd, EM_SETMARGINS, EC_LEFTMARGIN | EC_RIGHTMARGIN, 0) };
    } else if message == WM_PRINTCLIENT {
        unsafe {
            paint_hint(hwnd, data as Hwnd, wparam as *mut c_void);
        }
    }
    result
}

pub(super) unsafe fn install(edit: Hwnd, parent: Hwnd) {
    if !edit.is_null() {
        unsafe {
            SetWindowSubclass(edit, Some(edit_proc), SUBCLASS_ID, parent as usize);
            super::SendMessageW(edit, EM_SETMARGINS, EC_LEFTMARGIN | EC_RIGHTMARGIN, 0);
        }
    }
}
