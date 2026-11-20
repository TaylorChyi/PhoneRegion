//! Exercise the same HWND, controls, fonts and WM_DPICHANGED handler as the shipping app.
//! The test sends a monitor-change notification in process. The runner's monitor DPI is unchanged.
use super::{
    DestroyWindow, GetClientRect, Handle, Hwnd, ID_INPUT, ID_QUERY, ID_RESULT, Rect, SendMessageW,
    ShowWindow, UpdateWindow, WM_DPICHANGED, create_main_window, set_text, text_of,
};
use std::ffi::c_void;
use std::io::Write;
use std::path::Path;
use std::ptr::null_mut;

#[link(name = "user32")]
unsafe extern "system" {
    fn GetDlgItem(parent: Hwnd, id: i32) -> Hwnd;
    fn GetDC(hwnd: Hwnd) -> Handle;
    fn ReleaseDC(hwnd: Hwnd, dc: Handle) -> i32;
    fn GetWindow(parent: Hwnd, relation: u32) -> Hwnd;
    fn IsWindowVisible(window: Hwnd) -> i32;
    fn MapWindowPoints(from: Hwnd, to: Hwnd, points: *mut super::Point, count: u32) -> i32;
    fn SetThreadDpiAwarenessContext(context: Handle) -> Handle;
}

#[link(name = "gdi32")]
unsafe extern "system" {
    fn GetObjectW(handle: Handle, bytes: i32, result: *mut c_void) -> i32;
    fn CreateCompatibleDC(dc: Handle) -> Handle;
    fn DeleteDC(dc: Handle) -> i32;
    fn CreateCompatibleBitmap(dc: Handle, width: i32, height: i32) -> Handle;
    fn BitBlt(
        to: Handle,
        x: i32,
        y: i32,
        w: i32,
        h: i32,
        from: Handle,
        from_x: i32,
        from_y: i32,
        operation: u32,
    ) -> i32;
    fn CreateDIBSection(
        dc: Handle,
        info: *const u32,
        usage: u32,
        bits: *mut *mut u8,
        section: Handle,
        offset: u32,
    ) -> Handle;
    fn GdiFlush() -> i32;
}

fn client_size(window: Hwnd) -> (i32, i32) {
    let mut rect = Rect {
        left: 0,
        top: 0,
        right: 0,
        bottom: 0,
    };
    assert_ne!(unsafe { GetClientRect(window, &mut rect) }, 0);
    (rect.right - rect.left, rect.bottom - rect.top)
}

fn font_height(edit: Hwnd) -> i32 {
    // A Windows LOGFONTW is five LONGs, eight BYTEs and a 32-WCHAR face name.
    let font = unsafe {
        SendMessageW(edit, 0x0031 /* WM_GETFONT */, 0, 0)
    } as Handle;
    assert!(!font.is_null());
    let mut logfont = [0i32; 23];
    assert_eq!(
        unsafe { GetObjectW(font, 92, logfont.as_mut_ptr().cast()) },
        92
    );
    logfont[0].abs()
}

// On a headless Windows desktop the compositor can return black for PrintWindow.
// Paint the actual background and each native control into a DC at its real size.
unsafe fn print_controls(window: Hwnd, dc: Handle) {
    const GW_CHILD: u32 = 5;
    const GW_HWNDNEXT: u32 = 2;
    const WM_PRINT: u32 = 0x0317;
    const PRF_CLIENT_ERASE: isize = 4 | 8;
    const SRCCOPY: u32 = 0x00cc0020;
    unsafe {
        SendMessageW(window, super::WM_ERASEBKGND, dc as usize, 0);
        let child_dc = CreateCompatibleDC(dc);
        assert!(!child_dc.is_null());
        let mut child = GetWindow(window, GW_CHILD);
        while !child.is_null() {
            let (w, h) = client_size(child);
            if IsWindowVisible(child) != 0 && w > 0 && h > 0 {
                let mut pos = super::Point { x: 0, y: 0 };
                MapWindowPoints(child, window, &mut pos, 1);
                let bitmap = CreateCompatibleBitmap(dc, w, h);
                assert!(!bitmap.is_null());
                let previous = super::SelectObject(child_dc, bitmap);
                assert_ne!(BitBlt(child_dc, 0, 0, w, h, dc, pos.x, pos.y, SRCCOPY), 0);
                SendMessageW(child, WM_PRINT, child_dc as usize, PRF_CLIENT_ERASE);
                assert_ne!(BitBlt(dc, pos.x, pos.y, w, h, child_dc, 0, 0, SRCCOPY), 0);
                super::SelectObject(child_dc, previous);
                super::DeleteObject(bitmap);
            }
            child = GetWindow(child, GW_HWNDNEXT);
        }
        DeleteDC(child_dc);
    }
}

fn capture_client(window: Hwnd, path: &Path) {
    let (width, height) = client_size(window);
    assert!(
        width >= 1700 && height >= 1050,
        "unexpected 2x client size: {width}×{height}"
    );
    let stride = (width as u32 * 3 + 3) & !3;
    let length = stride * height as u32;
    let header = [
        40u32,
        width as u32,
        height as u32,
        1 | (24 << 16),
        0,
        length,
        0,
        0,
        0,
        0,
    ];
    let pixels = unsafe {
        let screen = GetDC(window);
        assert!(!screen.is_null());
        let dc = CreateCompatibleDC(screen);
        let mut bits = null_mut();
        let bitmap = CreateDIBSection(screen, header.as_ptr(), 0, &mut bits, null_mut(), 0);
        ReleaseDC(window, screen);
        assert!(!dc.is_null() && !bitmap.is_null() && !bits.is_null());
        let previous = super::SelectObject(dc, bitmap);
        print_controls(window, dc);
        GdiFlush();
        let bytes = std::slice::from_raw_parts(bits, length as usize).to_vec();
        super::SelectObject(dc, previous);
        super::DeleteObject(bitmap);
        DeleteDC(dc);
        bytes
    };
    let mut bmp = std::fs::File::create(path).unwrap();
    bmp.write_all(b"BM").unwrap();
    for word in [54 + length, 0, 54].into_iter().chain(header) {
        bmp.write_all(&word.to_le_bytes()).unwrap();
    }
    bmp.write_all(&pixels).unwrap();
}

#[test]
fn native_window_rescales_after_monitor_change() {
    // The unit-test host may have a different executable manifest from the product.
    let previous = unsafe { SetThreadDpiAwarenessContext((-4isize) as Handle) };
    assert!(!previous.is_null());
    let window = unsafe { create_main_window() }.expect("create the product window");
    let edit = unsafe { GetDlgItem(window, ID_INPUT) };
    let query = unsafe { GetDlgItem(window, ID_QUERY) };
    let result = unsafe { GetDlgItem(window, ID_RESULT) };
    assert!(!edit.is_null() && !query.is_null() && !result.is_null());
    unsafe {
        ShowWindow(window, 5);
        set_text(edit, "+86 132 1300 0000");
    }

    let mut baseline = None;
    for dpi in [96i32, 144, 192, 96] {
        let from_dips = |value: i32| (value * dpi + 48) / 96;
        let suggested = Rect {
            left: 0,
            top: 0,
            right: from_dips(1024),
            bottom: from_dips(750),
        };
        let param = dpi as usize | ((dpi as usize) << 16);
        unsafe {
            SendMessageW(
                window,
                WM_DPICHANGED,
                param,
                std::ptr::from_ref(&suggested) as isize,
            );
            UpdateWindow(window);
        }
        let metrics = [font_height(edit), client_size(edit).1, client_size(query).1];
        let reference = *baseline.get_or_insert(metrics);
        for (actual, original) in metrics.into_iter().zip(reference) {
            let expected = from_dips(original);
            assert!(
                (actual - expected).abs() <= 2,
                "DPI {dpi}: expected {expected}, got {actual}"
            );
        }
        let current = unsafe { text_of(result) };
        for field in ["中国联通", "河南省", "郑州市"] {
            assert!(
                current.contains(field),
                "lost {field} at {dpi} DPI: {current}"
            );
        }
        if dpi == 192
            && let Some(path) = std::env::var_os("PHONE_REGION_DPI_CAPTURE")
        {
            // The CI desktop can be physically smaller than this logical second monitor.
            // Set the capture extent after validating the real controls' DPI transition.
            // The native controls can draw into a DC beyond the desktop edge.
            const SWP_NOSENDCHANGING: u32 = 0x0400;
            unsafe {
                assert_ne!(
                    super::SetWindowPos(
                        window,
                        null_mut(),
                        0,
                        0,
                        suggested.right,
                        suggested.bottom,
                        super::SWP_NOZORDER | super::SWP_NOACTIVATE | SWP_NOSENDCHANGING
                    ),
                    0
                );
                UpdateWindow(window);
            }
            capture_client(window, Path::new(&path));
        }
    }
    unsafe {
        DestroyWindow(window);
        SetThreadDpiAwarenessContext(previous);
    }
}
