//! Small native Windows interface for the bundled, offline number database.

use crate::batch;
use crate::lookup::{Database, Status};
use std::ffi::{OsString, c_void};
use std::os::windows::ffi::{OsStrExt, OsStringExt};
use std::path::{Path, PathBuf};
use std::ptr::{null, null_mut};
use std::sync::{Arc, mpsc};

#[cfg(test)]
mod dpi_tests;
#[path = "ui/drawing.rs"]
mod drawing;
use drawing::Canvas;

type Handle = *mut c_void;
type Hwnd = Handle;
type Wparam = usize;
type Lparam = isize;
type Lresult = isize;

const WS_CAPTION: u32 = 0x00C00000;
const WS_THICKFRAME: u32 = 0x00040000;
const WS_MAXIMIZEBOX: u32 = 0x00010000;
const WS_VSCROLL: u32 = 0x00200000;
const SB_LINEUP: usize = 0;
const SB_LINEDOWN: usize = 1;
const SB_PAGEUP: usize = 2;
const SB_PAGEDOWN: usize = 3;
const SB_THUMBTRACK: usize = 5;
const SWP_NOOWNERZORDER: u32 = 0x0200;
const WS_SYSMENU: u32 = 0x00080000;
const WS_MINIMIZEBOX: u32 = 0x00020000;
const WS_VISIBLE: u32 = 0x10000000;
const WS_CHILD: u32 = 0x40000000;
const WS_TABSTOP: u32 = 0x00010000;
const ES_AUTOHSCROLL: u32 = 0x0080;
const BS_OWNERDRAW: u32 = 0x000B;
const SS_LEFT: u32 = 0;
const SS_OWNERDRAW: u32 = 0x000D;
const WM_CREATE: u32 = 0x0001;
const WM_CLOSE: u32 = 0x0010;
const WM_DESTROY: u32 = 0x0002;
const WM_COMMAND: u32 = 0x0111;
const WM_TIMER: u32 = 0x0113;
const WM_SETTINGCHANGE: u32 = 0x001A;
const WM_SYSCOLORCHANGE: u32 = 0x0015;
const WM_THEMECHANGED: u32 = 0x031A;
const WM_ERASEBKGND: u32 = 0x0014;
const WM_CTLCOLORSTATIC: u32 = 0x0138;
const WM_CTLCOLOREDIT: u32 = 0x0133;
const WM_DRAWITEM: u32 = 0x002B;
const WM_SIZE: u32 = 0x0005;
const WM_DPICHANGED: u32 = 0x02E0;
const WM_VSCROLL: u32 = 0x0115;
const WM_MOUSEWHEEL: u32 = 0x020A;
const WM_GETMINMAXINFO: u32 = 0x0024;
const WM_SETFONT: u32 = 0x0030;
const WM_KEYDOWN: u32 = 0x0100;
const EM_SETCUEBANNER: u32 = 0x1501;
const WM_SETICON: u32 = 0x0080;
const ICON_SMALL: usize = 0;
const IMAGE_ICON: u32 = 1;
const LR_SHARED: u32 = 0x8000;
const VK_RETURN: usize = 13;
const GWLP_USERDATA: i32 = -21;
const SW_SHOW: i32 = 5;
const MB_OK: u32 = 0;
const MB_ICONERROR: u32 = 0x10;
const MB_ICONINFORMATION: u32 = 0x40;
const OFN_HIDEREADONLY: u32 = 0x00000004;
const OFN_PATHMUSTEXIST: u32 = 0x00000800;
const OFN_FILEMUSTEXIST: u32 = 0x00001000;
const OFN_NOCHANGEDIR: u32 = 0x00000008;
const IDC_ARROW: usize = 32512;
const APP_ICON: usize = 102;
const ID_INPUT: i32 = 1001;
const EN_CHANGE: usize = 0x0300;
const ID_QUERY: i32 = 1002;
const ID_IMPORT: i32 = 1003;
const ID_RESULT: i32 = 1004;
const ID_PROGRESS: i32 = 1005;
const ID_RESULT_LABEL: i32 = 1006;
const ID_CAVEAT: i32 = 1007;
const TIMER_WORK: usize = 1;
const COLOR_WINDOW: i32 = 5;
const COLOR_WINDOWTEXT: i32 = 8;
const COLOR_BTNFACE: i32 = 15;
const COLOR_BTNTEXT: i32 = 18;
const COLOR_GRAYTEXT: i32 = 17;
const COLOR_HIGHLIGHT: i32 = 13;
const COLOR_HIGHLIGHTTEXT: i32 = 14;
const SPI_GETHIGHCONTRAST: u32 = 0x0042;
const SPI_GETWORKAREA: u32 = 0x0030;
const HCF_HIGHCONTRASTON: u32 = 1;
const RRF_RT_REG_DWORD: u32 = 0x00000018;
const DT_VCENTER: u32 = 4;
const DT_SINGLELINE: u32 = 0x20;
const TRANSPARENT: i32 = 1;
const ODS_SELECTED: u32 = 1;
const ODS_DISABLED: u32 = 4;
const ODS_FOCUS: u32 = 0x10;
const RDW_INVALIDATE: u32 = 1;
const RDW_ERASE: u32 = 4;
const RDW_ALLCHILDREN: u32 = 0x80;
const SWP_NOZORDER: u32 = 0x0004;
const SWP_NOACTIVATE: u32 = 0x0010;
const SB_VERT: i32 = 1;
const SIF_RANGE: u32 = 1;
const SIF_PAGE: u32 = 2;
const SIF_POS: u32 = 4;
const SIF_TRACKPOS: u32 = 16;
const DT_WORDBREAK: u32 = 0x10;
const DT_NOPREFIX: u32 = 0x800;

#[repr(C)]
struct WndClass {
    style: u32,
    procedure: Option<unsafe extern "system" fn(Hwnd, u32, Wparam, Lparam) -> Lresult>,
    cls_extra: i32,
    wnd_extra: i32,
    instance: Handle,
    icon: Handle,
    cursor: Handle,
    background: Handle,
    menu_name: *const u16,
    class_name: *const u16,
}

#[repr(C)]
struct Message {
    hwnd: Hwnd,
    message: u32,
    wparam: Wparam,
    lparam: Lparam,
    time: u32,
    point: Point,
    private: u32,
}

#[repr(C)]
struct Point {
    x: i32,
    y: i32,
}

#[repr(C)]
#[derive(Clone, Copy)]
struct Rect {
    left: i32,
    top: i32,
    right: i32,
    bottom: i32,
}

#[repr(C)]
#[derive(Clone, Copy)]
struct DrawItem {
    control_type: u32,
    control_id: u32,
    item_id: u32,
    action: u32,
    state: u32,
    hwnd: Hwnd,
    dc: Handle,
    rect: Rect,
    item_data: usize,
}

#[repr(C)]
struct HighContrast {
    size: u32,
    flags: u32,
    default_scheme: *mut u16,
}

#[repr(C)]
struct ScrollInfo {
    size: u32,
    mask: u32,
    min: i32,
    max: i32,
    page: u32,
    pos: i32,
    track_pos: i32,
}

#[repr(C)]
struct MinMaxInfo {
    reserved: Point,
    max_size: Point,
    max_position: Point,
    min_track_size: Point,
    max_track_size: Point,
}

#[repr(C)]
struct OpenFileName {
    size: u32,
    owner: Hwnd,
    instance: Handle,
    filter: *const u16,
    custom_filter: *mut u16,
    custom_filter_size: u32,
    filter_index: u32,
    file: *mut u16,
    max_file: u32,
    file_title: *mut u16,
    max_file_title: u32,
    initial_dir: *const u16,
    title: *const u16,
    flags: u32,
    file_offset: u16,
    file_extension: u16,
    default_extension: *const u16,
    custom_data: isize,
    hook: Handle,
    template_name: *const u16,
    reserved: Handle,
    reserved_dword: u32,
    flags_ex: u32,
}

#[link(name = "user32")]
unsafe extern "system" {
    fn RegisterClassW(class: *const WndClass) -> u16;
    fn CreateWindowExW(
        ex: u32,
        class: *const u16,
        title: *const u16,
        style: u32,
        x: i32,
        y: i32,
        width: i32,
        height: i32,
        parent: Hwnd,
        menu: Handle,
        instance: Handle,
        parameter: Handle,
    ) -> Hwnd;
    fn DefWindowProcW(hwnd: Hwnd, msg: u32, wparam: Wparam, lparam: Lparam) -> Lresult;
    #[cfg(test)]
    fn DestroyWindow(hwnd: Hwnd) -> i32;
    fn GetMessageW(message: *mut Message, hwnd: Hwnd, min: u32, max: u32) -> i32;
    fn TranslateMessage(message: *const Message) -> i32;
    fn DispatchMessageW(message: *const Message) -> Lresult;
    fn PostQuitMessage(code: i32);
    fn ShowWindow(hwnd: Hwnd, command: i32) -> i32;
    fn UpdateWindow(hwnd: Hwnd) -> i32;
    fn SendMessageW(hwnd: Hwnd, msg: u32, wparam: Wparam, lparam: Lparam) -> Lresult;
    fn SetWindowTextW(hwnd: Hwnd, text: *const u16) -> i32;
    fn GetWindowTextW(hwnd: Hwnd, buffer: *mut u16, capacity: i32) -> i32;
    fn GetWindowTextLengthW(hwnd: Hwnd) -> i32;
    fn MessageBoxW(hwnd: Hwnd, text: *const u16, title: *const u16, style: u32) -> i32;
    fn GetWindowLongPtrW(hwnd: Hwnd, index: i32) -> isize;
    fn SetWindowLongPtrW(hwnd: Hwnd, index: i32, value: isize) -> isize;
    fn SetTimer(hwnd: Hwnd, id: usize, timeout: u32, callback: Handle) -> usize;
    fn KillTimer(hwnd: Hwnd, id: usize) -> i32;
    fn EnableWindow(hwnd: Hwnd, enable: i32) -> i32;
    fn SetFocus(hwnd: Hwnd) -> Hwnd;
    fn GetSysColorBrush(index: i32) -> Handle;
    fn LoadCursorW(instance: Handle, name: *const u16) -> Handle;
    fn LoadIconW(instance: Handle, name: *const u16) -> Handle;
    fn LoadImageW(
        instance: Handle,
        name: *const u16,
        image_type: u32,
        width: i32,
        height: i32,
        flags: u32,
    ) -> Handle;
    fn GetSystemMetrics(index: i32) -> i32;
    fn IsDialogMessageW(hwnd: Hwnd, message: *const Message) -> i32;
    fn GetSysColor(index: i32) -> u32;
    fn SystemParametersInfoW(action: u32, param: u32, result: *mut c_void, flags: u32) -> i32;
    fn RedrawWindow(hwnd: Hwnd, rect: *const Rect, region: Handle, flags: u32) -> i32;
    fn FillRect(dc: Handle, rect: *const Rect, brush: Handle) -> i32;
    fn DrawTextW(dc: Handle, text: *const u16, count: i32, rect: *mut Rect, format: u32) -> i32;
    fn GetClientRect(hwnd: Hwnd, rect: *mut Rect) -> i32;
    fn SetWindowPos(
        hwnd: Hwnd,
        after: Hwnd,
        x: i32,
        y: i32,
        width: i32,
        height: i32,
        flags: u32,
    ) -> i32;
    fn InvalidateRect(hwnd: Hwnd, rect: *const Rect, erase: i32) -> i32;
    fn GetDC(hwnd: Hwnd) -> Handle;
    fn ReleaseDC(hwnd: Hwnd, dc: Handle) -> i32;
    fn SetScrollInfo(hwnd: Hwnd, bar: i32, info: *const ScrollInfo, redraw: i32) -> i32;
    fn GetScrollInfo(hwnd: Hwnd, bar: i32, info: *mut ScrollInfo) -> i32;
    fn ShowScrollBar(hwnd: Hwnd, bar: i32, show: i32) -> i32;
    fn DrawIconEx(
        dc: Handle,
        x: i32,
        y: i32,
        icon: Handle,
        width: i32,
        height: i32,
        frame: u32,
        background: Handle,
        flags: u32,
    ) -> i32;
}

#[link(name = "kernel32")]
unsafe extern "system" {
    fn GetModuleHandleW(name: *const u16) -> Handle;
    fn GetProcAddress(module: Handle, name: *const u8) -> *const c_void;
}

#[link(name = "gdi32")]
unsafe extern "system" {
    fn GetDeviceCaps(dc: Handle, index: i32) -> i32;
    fn CreateFontW(
        height: i32,
        width: i32,
        escapement: i32,
        orientation: i32,
        weight: i32,
        italic: u32,
        underline: u32,
        strikeout: u32,
        charset: u32,
        out_precision: u32,
        clip_precision: u32,
        quality: u32,
        pitch_and_family: u32,
        face: *const u16,
    ) -> Handle;
    fn DeleteObject(object: Handle) -> i32;
    fn CreateSolidBrush(color: u32) -> Handle;
    fn SetTextColor(dc: Handle, color: u32) -> u32;
    fn SetBkColor(dc: Handle, color: u32) -> u32;
    fn SetBkMode(dc: Handle, mode: i32) -> i32;
    fn SelectObject(dc: Handle, object: Handle) -> Handle;
}

#[link(name = "advapi32")]
unsafe extern "system" {
    fn RegGetValueW(
        key: Handle,
        subkey: *const u16,
        value: *const u16,
        flags: u32,
        kind: *mut u32,
        data: *mut c_void,
        size: *mut u32,
    ) -> i32;
}

#[link(name = "dwmapi")]
unsafe extern "system" {
    fn DwmSetWindowAttribute(hwnd: Hwnd, attribute: u32, value: *const c_void, size: u32) -> i32;
}

#[link(name = "comdlg32")]
unsafe extern "system" {
    fn GetOpenFileNameW(ofn: *mut OpenFileName) -> i32;
    fn GetSaveFileNameW(ofn: *mut OpenFileName) -> i32;
}

struct UiState {
    database: Arc<Database>,
    brand: Hwnd,
    offline: Hwnd,
    title: Hwnd,
    subtitle: Hwnd,
    input: Hwnd,
    query: Hwnd,
    result: Hwnd,
    result_label: Hwnd,
    caveat: Hwnd,
    progress: Hwnd,
    import: Hwnd,
    font: Handle,
    title_font: Handle,
    input_font: Handle,
    small_font: Handle,
    dpi: u32,
    scroll: i32,
    layout: Layout,
    icon: Handle,
    result_view: Option<ResultView>,
    result_error: bool,
    scroll_visible: bool,
    theme: Theme,
    busy: bool,
    receiver: Option<mpsc::Receiver<Result<(PathBuf, batch::Summary), String>>>,
}

#[derive(Clone)]
struct ResultView {
    carrier: String,
    province: String,
    city: String,
}

#[derive(Clone, Copy, Default)]
struct Layout {
    margin: f32,
    width: f32,
    result_y: f32,
    result_height: f32,
    batch_y: f32,
    status_y: f32,
    content_height: f32,
    columns: bool,
}

impl Layout {
    fn new(window_width: f32, result: bool, error: bool) -> Self {
        let margin = if window_width < 680.0 { 24.0 } else { 48.0 };
        let width = (window_width - 2.0 * margin).clamp(1.0, 1080.0);
        let columns = width >= 720.0;
        let result_y = 296.0;
        let result_height = if result {
            if columns { 112.0 } else { 216.0 }
        } else if error {
            56.0
        } else {
            0.0
        };
        let batch_y = if result_height > 0.0 {
            result_y + result_height + 43.0
        } else {
            338.0
        };
        let status_y = batch_y + 91.0;
        Self {
            margin,
            width,
            result_y,
            result_height,
            batch_y,
            status_y,
            content_height: status_y + 80.0,
            columns,
        }
    }
}

#[derive(Clone, Copy)]
struct Colors {
    surface: u32,
    text: u32,
    field: u32,
    button: u32,
    border: u32,
    disabled: u32,
    muted: u32,
    accent: u32,
    accent_text: u32,
}

struct Theme {
    colors: Colors,
    high_contrast: bool,
    surface_brush: Handle,
    field_brush: Handle,
    border_brush: Handle,
}

impl Theme {
    unsafe fn read(hwnd: Hwnd) -> Self {
        let mut contrast = HighContrast {
            size: std::mem::size_of::<HighContrast>() as u32,
            flags: 0,
            default_scheme: null_mut(),
        };
        let high_contrast = unsafe {
            SystemParametersInfoW(
                SPI_GETHIGHCONTRAST,
                contrast.size,
                &mut contrast as *mut _ as *mut c_void,
                0,
            ) != 0
        } && contrast.flags & HCF_HIGHCONTRASTON != 0;
        let mut light = 1u32;
        let mut size = std::mem::size_of::<u32>() as u32;
        let subkey = wide("Software\\Microsoft\\Windows\\CurrentVersion\\Themes\\Personalize");
        let value = wide("AppsUseLightTheme");
        unsafe {
            RegGetValueW(
                (-2147483647isize) as Handle,
                subkey.as_ptr(),
                value.as_ptr(),
                RRF_RT_REG_DWORD,
                null_mut(),
                &mut light as *mut _ as *mut c_void,
                &mut size,
            );
        }
        let colors = if high_contrast {
            unsafe {
                Colors {
                    surface: GetSysColor(COLOR_WINDOW),
                    text: GetSysColor(COLOR_WINDOWTEXT),
                    field: GetSysColor(COLOR_WINDOW),
                    button: GetSysColor(COLOR_BTNFACE),
                    border: GetSysColor(COLOR_BTNTEXT),
                    disabled: GetSysColor(COLOR_GRAYTEXT),
                    muted: GetSysColor(COLOR_WINDOWTEXT),
                    accent: GetSysColor(COLOR_HIGHLIGHT),
                    accent_text: GetSysColor(COLOR_HIGHLIGHTTEXT),
                }
            }
        } else {
            colors_for_mode(light == 0)
        };
        let dark_title: i32 = if !high_contrast && light == 0 { 1 } else { 0 };
        unsafe {
            DwmSetWindowAttribute(hwnd, 20, &dark_title as *const _ as *const c_void, 4);
        }
        unsafe {
            Self {
                colors,
                high_contrast,
                surface_brush: CreateSolidBrush(colors.surface),
                field_brush: CreateSolidBrush(colors.field),
                border_brush: CreateSolidBrush(colors.border),
            }
        }
    }

    unsafe fn delete(self) {
        unsafe {
            DeleteObject(self.surface_brush);
            DeleteObject(self.field_brush);
            DeleteObject(self.border_brush);
        }
    }
}

fn colors_for_mode(dark: bool) -> Colors {
    if dark {
        Colors {
            surface: 0x001E1E1E,
            text: 0x00F3F3F3,
            field: 0x002A2A2A,
            button: 0x001E1E1E,
            border: 0x00545454,
            disabled: 0x00AAAAAA,
            muted: 0x00B9B9B9,
            accent: 0x00F3F3F3,
            accent_text: 0x001E1E1E,
        }
    } else {
        Colors {
            surface: 0x00FAFAFA,
            text: 0x001E1E1E,
            field: 0x00FFFFFF,
            button: 0x00FFFFFF,
            border: 0x00D1D1D1,
            disabled: 0x006A6A6A,
            muted: 0x006A6A6A,
            accent: 0x001E1E1E,
            accent_text: 0x00FFFFFF,
        }
    }
}

fn wide(value: &str) -> Vec<u16> {
    value.encode_utf16().chain(std::iter::once(0)).collect()
}

// GetDpiForWindow/System arrived after the first Windows 10 release. Resolve
// them at runtime so the executable can still start on earlier builds.
unsafe fn device_dpi() -> u32 {
    unsafe {
        let dc = GetDC(null_mut());
        if dc.is_null() {
            return 96;
        }
        let dpi = GetDeviceCaps(dc, 88); // LOGPIXELSX
        ReleaseDC(null_mut(), dc);
        if dpi > 0 { dpi as u32 } else { 96 }
    }
}

unsafe fn dpi_for_window(hwnd: Hwnd) -> u32 {
    unsafe {
        let user32 = GetModuleHandleW(wide("user32.dll").as_ptr());
        if !user32.is_null() {
            let address = GetProcAddress(user32, c"GetDpiForWindow".as_ptr().cast());
            if !address.is_null() {
                let function: unsafe extern "system" fn(Hwnd) -> u32 = std::mem::transmute(address);
                let dpi = function(hwnd);
                if dpi != 0 {
                    return dpi;
                }
            }
        }
        device_dpi()
    }
}

unsafe fn system_dpi() -> u32 {
    unsafe {
        let user32 = GetModuleHandleW(wide("user32.dll").as_ptr());
        if !user32.is_null() {
            let address = GetProcAddress(user32, c"GetDpiForSystem".as_ptr().cast());
            if !address.is_null() {
                let function: unsafe extern "system" fn() -> u32 = std::mem::transmute(address);
                let dpi = function();
                if dpi != 0 {
                    return dpi;
                }
            }
        }
        device_dpi()
    }
}

fn pixels(dip: f32, dpi: u32) -> i32 {
    (dip * dpi as f32 / 96.0).round() as i32
}

fn rgb(colorref: u32) -> u32 {
    ((colorref & 0x0000ff) << 16) | (colorref & 0x00ff00) | ((colorref & 0xff0000) >> 16)
}

unsafe fn make_font(size: f32, weight: i32, dpi: u32) -> Handle {
    unsafe {
        CreateFontW(
            -pixels(size, dpi),
            0,
            0,
            0,
            weight,
            0,
            0,
            0,
            1,
            0,
            0,
            5,
            0,
            wide("Microsoft YaHei UI").as_ptr(),
        )
    }
}

unsafe fn place(hwnd: Hwnd, x: f32, y: f32, width: f32, height: f32, dpi: u32, scroll: i32) {
    unsafe {
        SetWindowPos(
            hwnd,
            null_mut(),
            pixels(x, dpi),
            pixels(y, dpi) - scroll,
            pixels(width, dpi),
            pixels(height, dpi),
            SWP_NOZORDER | SWP_NOACTIVATE,
        );
    }
}

unsafe fn reflow(hwnd: Hwnd) {
    let state = unsafe { GetWindowLongPtrW(hwnd, GWLP_USERDATA) as *mut UiState };
    if state.is_null() {
        return;
    }
    let mut client = Rect {
        left: 0,
        top: 0,
        right: 0,
        bottom: 0,
    };
    unsafe {
        GetClientRect(hwnd, &mut client);
    }
    let dpi = unsafe { (*state).dpi };
    let width = client.right as f32 * 96.0 / dpi as f32;
    let result = unsafe { (*state).result_view.is_some() };
    let error = unsafe { (*state).result_error };
    let layout = Layout::new(width, result, error);
    let max_scroll = (pixels(layout.content_height, dpi) - client.bottom).max(0);
    let scroll = unsafe { (*state).scroll.clamp(0, max_scroll) };
    let visible = max_scroll > 0;
    unsafe {
        (*state).layout = layout;
        (*state).scroll = scroll;
    }
    let current_visible = unsafe { (*state).scroll_visible };
    if current_visible != visible {
        unsafe {
            (*state).scroll_visible = visible;
            ShowScrollBar(hwnd, SB_VERT, visible as i32);
        }
    }
    let info = ScrollInfo {
        size: std::mem::size_of::<ScrollInfo>() as u32,
        mask: SIF_RANGE | SIF_PAGE | SIF_POS,
        min: 0,
        max: pixels(layout.content_height, dpi),
        page: client.bottom as u32,
        pos: scroll,
        track_pos: 0,
    };
    unsafe {
        SetScrollInfo(hwnd, SB_VERT, &info, 1);
    }

    let left = layout.margin + ((width - 2.0 * layout.margin - layout.width).max(0.0) / 2.0);
    let full = layout.width;
    let content_scroll = scroll;
    unsafe {
        place((*state).brand, left + 36.0, 17.0, 220.0, 30.0, dpi, 0);
        place(
            (*state).offline,
            left + full - 160.0,
            19.0,
            160.0,
            28.0,
            dpi,
            0,
        );
        place((*state).title, left, 88.0, full, 50.0, dpi, content_scroll);
        place(
            (*state).subtitle,
            left,
            142.0,
            full,
            31.0,
            dpi,
            content_scroll,
        );
        place(
            (*state).input,
            left + 20.0,
            208.0,
            (full - 106.0).max(80.0),
            42.0,
            dpi,
            content_scroll,
        );
        place(
            (*state).query,
            left + full - 65.0,
            204.0,
            48.0,
            48.0,
            dpi,
            content_scroll,
        );
        place(
            (*state).result_label,
            left,
            273.0,
            full,
            22.0,
            dpi,
            content_scroll,
        );
        place(
            (*state).result,
            left,
            layout.result_y,
            full,
            layout.result_height,
            dpi,
            content_scroll,
        );
        place(
            (*state).caveat,
            left,
            layout.result_y + layout.result_height + 7.0,
            full,
            27.0,
            dpi,
            content_scroll,
        );
        place(
            (*state).import,
            left,
            layout.batch_y,
            full,
            72.0,
            dpi,
            content_scroll,
        );
        place(
            (*state).progress,
            left,
            layout.status_y,
            full,
            64.0,
            dpi,
            content_scroll,
        );
        InvalidateRect(hwnd, null(), 1);
    }
}

unsafe fn draw_text(
    dc: Handle,
    text: &str,
    font: Handle,
    color: u32,
    bounds: [f32; 4],
    dpi: u32,
    format: u32,
) {
    let mut rect = Rect {
        left: pixels(bounds[0], dpi),
        top: pixels(bounds[1], dpi),
        right: pixels(bounds[0] + bounds[2], dpi),
        bottom: pixels(bounds[1] + bounds[3], dpi),
    };
    unsafe {
        let previous = SelectObject(dc, font);
        SetTextColor(dc, color);
        SetBkMode(dc, TRANSPARENT);
        DrawTextW(dc, wide(text).as_ptr(), -1, &mut rect, format | DT_NOPREFIX);
        SelectObject(dc, previous);
    }
}

unsafe fn work_area() -> Rect {
    let mut rect = Rect {
        left: 0,
        top: 0,
        right: 0,
        bottom: 0,
    };
    let found =
        unsafe { SystemParametersInfoW(SPI_GETWORKAREA, 0, &mut rect as *mut _ as *mut c_void, 0) };
    if found == 0 {
        rect.right = unsafe { GetSystemMetrics(0) };
        rect.bottom = unsafe { GetSystemMetrics(1) };
    }
    rect
}

unsafe fn create_control(
    parent: Hwnd,
    class: &str,
    title: &str,
    style: u32,
    id: i32,
    bounds: [i32; 4],
    font: Handle,
) -> Hwnd {
    let class = wide(class);
    let title = wide(title);
    let hwnd = unsafe {
        CreateWindowExW(
            0,
            class.as_ptr(),
            title.as_ptr(),
            WS_CHILD | WS_VISIBLE | style,
            bounds[0],
            bounds[1],
            bounds[2],
            bounds[3],
            parent,
            id as usize as Handle,
            null_mut(),
            null_mut(),
        )
    };
    unsafe { SendMessageW(hwnd, WM_SETFONT, font as usize, 1) };
    hwnd
}

unsafe fn text_of(hwnd: Hwnd) -> String {
    let len = unsafe { GetWindowTextLengthW(hwnd) }.max(0) as usize;
    let mut buffer = vec![0u16; len + 1];
    let read = unsafe { GetWindowTextW(hwnd, buffer.as_mut_ptr(), buffer.len() as i32) };
    String::from_utf16_lossy(&buffer[..read.max(0) as usize])
}

unsafe fn set_text(hwnd: Hwnd, text: &str) {
    unsafe { SetWindowTextW(hwnd, wide(text).as_ptr()) };
}

unsafe fn alert(hwnd: Hwnd, message: &str, error: bool) {
    let style = MB_OK
        | if error {
            MB_ICONERROR
        } else {
            MB_ICONINFORMATION
        };
    unsafe {
        MessageBoxW(
            hwnd,
            wide(message).as_ptr(),
            wide("手机号归属地").as_ptr(),
            style,
        )
    };
}

fn extension(input: &Path) -> &str {
    input.extension().and_then(|s| s.to_str()).unwrap_or("")
}

fn visible_output_path(path: &Path) -> String {
    let full = path.display().to_string();
    if full.chars().count() <= 55 {
        return full;
    }
    let name = path.file_name().unwrap_or_default().to_string_lossy();
    let chars: Vec<char> = name.chars().collect();
    if chars.len() <= 34 {
        return name.into_owned();
    }
    let start: String = chars[..14].iter().collect();
    let end: String = chars[chars.len() - 17..].iter().collect();
    format!("{start}…{end}")
}

unsafe fn file_dialog(
    owner: Hwnd,
    save: bool,
    suggested: Option<&Path>,
    preferred_filter: u32,
) -> Option<(PathBuf, u32)> {
    let filter = if save {
        wide("Excel 文件 (*.xlsx)\0*.xlsx\0CSV 文件 (*.csv)\0*.csv\0")
    } else {
        wide("Excel 或 CSV 文件\0*.xlsx;*.xls;*.csv\0Excel 文件\0*.xlsx;*.xls\0CSV 文件\0*.csv\0")
    };
    let mut file = [0u16; 32768];
    if let Some(path) = suggested {
        let value: Vec<u16> = path
            .as_os_str()
            .encode_wide()
            .chain(std::iter::once(0))
            .collect();
        let count = value.len().min(file.len());
        file[..count].copy_from_slice(&value[..count]);
    }
    let title = wide(if save {
        "保存查询结果"
    } else {
        "选择手机号文件"
    });
    let mut dialog = OpenFileName {
        size: std::mem::size_of::<OpenFileName>() as u32,
        owner,
        instance: null_mut(),
        filter: filter.as_ptr(),
        custom_filter: null_mut(),
        custom_filter_size: 0,
        filter_index: preferred_filter,
        file: file.as_mut_ptr(),
        max_file: file.len() as u32,
        file_title: null_mut(),
        max_file_title: 0,
        initial_dir: null(),
        title: title.as_ptr(),
        flags: OFN_HIDEREADONLY
            | OFN_PATHMUSTEXIST
            | OFN_NOCHANGEDIR
            | if save { 0 } else { OFN_FILEMUSTEXIST },
        file_offset: 0,
        file_extension: 0,
        default_extension: null(),
        custom_data: 0,
        hook: null_mut(),
        template_name: null(),
        reserved: null_mut(),
        reserved_dword: 0,
        flags_ex: 0,
    };
    let accepted = unsafe {
        if save {
            GetSaveFileNameW(&mut dialog)
        } else {
            GetOpenFileNameW(&mut dialog)
        }
    };
    if accepted == 0 {
        return None;
    }
    let len = file
        .iter()
        .position(|unit| *unit == 0)
        .unwrap_or(file.len());
    Some((
        PathBuf::from(OsString::from_wide(&file[..len])),
        dialog.filter_index,
    ))
}

unsafe extern "system" fn window_proc(
    hwnd: Hwnd,
    msg: u32,
    wparam: Wparam,
    lparam: Lparam,
) -> Lresult {
    match msg {
        WM_CREATE => {
            let database = match Database::load() {
                Ok(db) => Arc::new(db),
                Err(reason) => {
                    unsafe {
                        alert(hwnd, &format!("无法加载离线号段库：{reason}"), true)
                    };
                    return -1;
                }
            };
            unsafe {
                let dpi = dpi_for_window(hwnd).max(96);
                let font = make_font(17.0, 400, dpi);
                let title_font = make_font(34.0, 650, dpi);
                let input_font = make_font(24.0, 400, dpi);
                let small_font = make_font(14.0, 400, dpi);
                if [font, title_font, input_font, small_font]
                    .iter()
                    .any(|handle| handle.is_null())
                {
                    for handle in [font, title_font, input_font, small_font] {
                        if !handle.is_null() {
                            DeleteObject(handle);
                        }
                    }
                    return -1;
                }
                let theme = Theme::read(hwnd);
                let icon = LoadImageW(
                    GetModuleHandleW(null()),
                    APP_ICON as *const u16,
                    IMAGE_ICON,
                    pixels(24.0, dpi),
                    pixels(24.0, dpi),
                    LR_SHARED,
                );
                if !icon.is_null() {
                    SendMessageW(hwnd, WM_SETICON, ICON_SMALL, icon as isize);
                }

                let brand = create_control(
                    hwnd,
                    "STATIC",
                    "PhoneRegion",
                    SS_LEFT,
                    0,
                    [0, 0, 1, 1],
                    font,
                );
                let offline = create_control(
                    hwnd,
                    "STATIC",
                    "离线 · 本机查询",
                    SS_LEFT,
                    0,
                    [0, 0, 1, 1],
                    small_font,
                );
                let title = create_control(
                    hwnd,
                    "STATIC",
                    "手机号归属地",
                    SS_LEFT,
                    0,
                    [0, 0, 1, 1],
                    title_font,
                );
                let subtitle = create_control(
                    hwnd,
                    "STATIC",
                    "输入中国大陆手机号，查看号段对应的运营商、省份和地级市",
                    SS_LEFT,
                    0,
                    [0, 0, 1, 1],
                    font,
                );
                let input = create_control(
                    hwnd,
                    "EDIT",
                    "",
                    WS_TABSTOP | ES_AUTOHSCROLL,
                    ID_INPUT,
                    [0, 0, 1, 1],
                    input_font,
                );
                SendMessageW(
                    input,
                    EM_SETCUEBANNER,
                    1,
                    wide("输入或粘贴手机号").as_ptr() as isize,
                );
                let query = create_control(
                    hwnd,
                    "BUTTON",
                    "查询",
                    WS_TABSTOP | BS_OWNERDRAW,
                    ID_QUERY,
                    [0, 0, 1, 1],
                    input_font,
                );
                let result_label = create_control(
                    hwnd,
                    "STATIC",
                    "",
                    SS_LEFT,
                    ID_RESULT_LABEL,
                    [0, 0, 1, 1],
                    small_font,
                );
                let result = create_control(
                    hwnd,
                    "STATIC",
                    "",
                    SS_OWNERDRAW,
                    ID_RESULT,
                    [0, 0, 1, 1],
                    font,
                );
                let caveat = create_control(
                    hwnd,
                    "STATIC",
                    "",
                    SS_LEFT,
                    ID_CAVEAT,
                    [0, 0, 1, 1],
                    small_font,
                );
                let import = create_control(
                    hwnd,
                    "BUTTON",
                    "导入 Excel / CSV",
                    WS_TABSTOP | BS_OWNERDRAW,
                    ID_IMPORT,
                    [0, 0, 1, 1],
                    font,
                );
                let progress = create_control(
                    hwnd,
                    "STATIC",
                    "支持 .xlsx、.xls、.csv，读取首列并另存为四列文件。",
                    SS_LEFT,
                    ID_PROGRESS,
                    [0, 0, 1, 1],
                    small_font,
                );
                let state = Box::new(UiState {
                    database,
                    brand,
                    offline,
                    title,
                    subtitle,
                    input,
                    query,
                    result,
                    result_label,
                    caveat,
                    progress,
                    import,
                    font,
                    title_font,
                    input_font,
                    small_font,
                    dpi,
                    scroll: 0,
                    layout: Layout::default(),
                    icon,
                    result_view: None,
                    result_error: false,
                    scroll_visible: false,
                    theme,
                    busy: false,
                    receiver: None,
                });
                SetWindowLongPtrW(hwnd, GWLP_USERDATA, Box::into_raw(state) as isize);
                reflow(hwnd);
                SetFocus(input);
            }
            0
        }
        WM_COMMAND => {
            let id = (wparam & 0xFFFF) as i32;
            let state_ptr = unsafe { GetWindowLongPtrW(hwnd, GWLP_USERDATA) as *mut UiState };
            if state_ptr.is_null() {
                return 0;
            }
            let input_changed = id == ID_INPUT && (wparam >> 16) == EN_CHANGE;
            match id {
                ID_INPUT | ID_QUERY if input_changed || id == ID_QUERY => {
                    let input_hwnd = unsafe { (*state_ptr).input };
                    let result_hwnd = unsafe { (*state_ptr).result };
                    let label_hwnd = unsafe { (*state_ptr).result_label };
                    let caveat_hwnd = unsafe { (*state_ptr).caveat };
                    let database = unsafe { Arc::clone(&(*state_ptr).database) };
                    let input = unsafe { text_of(input_hwnd) };
                    let found = database.lookup(&input);
                    let success = found.status == Status::Found;
                    let view = if success {
                        Some(ResultView {
                            carrier: found.carrier.clone(),
                            province: found.province.clone(),
                            city: if found.city.is_empty() {
                                "离线库未提供唯一地市".to_string()
                            } else {
                                found.city.clone()
                            },
                        })
                    } else {
                        None
                    };
                    let detail = match found.status {
                        Status::Found => format!(
                            "运营商 {}\r\n省份 {}\r\n地级市 {}",
                            found.carrier,
                            found.province,
                            if found.city.is_empty() {
                                "离线库未提供唯一地市"
                            } else {
                                &found.city
                            }
                        ),
                        Status::Invalid if input_changed => String::new(),
                        Status::Invalid => "请输入 11 位中国大陆手机号。".to_string(),
                        Status::Missing => "离线号段库未收录该号码前 7 位。".to_string(),
                        Status::Empty if input_changed => String::new(),
                        Status::Empty => "请输入手机号。".to_string(),
                    };
                    unsafe {
                        (*state_ptr).result_view = view;
                        (*state_ptr).result_error = !success && !detail.is_empty();
                    }
                    unsafe {
                        set_text(label_hwnd, if success { "查询结果" } else { "" });
                        set_text(result_hwnd, &detail);
                        set_text(
                            caveat_hwnd,
                            if success {
                                "运营商为原号段信息。携号转网后，当前运营商可能不同。"
                            } else {
                                ""
                            },
                        );
                        reflow(hwnd);
                    }
                }
                ID_IMPORT if !unsafe { (*state_ptr).busy } => {
                    unsafe { (*state_ptr).busy = true };
                    let input = match unsafe { file_dialog(hwnd, false, None, 1) } {
                        Some((input, _)) => input,
                        None => {
                            unsafe { (*state_ptr).busy = false };
                            return 0;
                        }
                    };
                    let suffix = if extension(&input).eq_ignore_ascii_case("csv") {
                        "csv"
                    } else {
                        "xlsx"
                    };
                    let default = input.with_file_name(format!(
                        "{}_归属地结果",
                        input.file_stem().unwrap_or_default().to_string_lossy()
                    ));
                    let preferred_filter = if suffix == "csv" { 2 } else { 1 };
                    let (mut output, selected_filter) = match unsafe {
                        file_dialog(hwnd, true, Some(&default), preferred_filter)
                    } {
                        Some(selection) => selection,
                        None => {
                            unsafe { (*state_ptr).busy = false };
                            return 0;
                        }
                    };
                    if output.extension().is_none() {
                        output.set_extension(if selected_filter == 2 { "csv" } else { "xlsx" });
                    }
                    let output_suffix = extension(&output).to_ascii_lowercase();
                    if output_suffix != "csv" && output_suffix != "xlsx" {
                        unsafe { (*state_ptr).busy = false };
                        unsafe {
                            alert(hwnd, "结果文件请选择 .xlsx 或 .csv 格式。", true)
                        };
                        return 0;
                    }
                    if output.exists() || input == output {
                        unsafe { (*state_ptr).busy = false };
                        unsafe {
                            alert(hwnd, "目标文件已经存在，请另选文件名。", true)
                        };
                        return 0;
                    }
                    let db = unsafe { Arc::clone(&(*state_ptr).database) };
                    let (tx, rx) = mpsc::channel();
                    std::thread::spawn(move || {
                        let result =
                            batch::convert(&input, &output, &db).map(|summary| (output, summary));
                        let _ = tx.send(result);
                    });
                    let import_hwnd = unsafe { (*state_ptr).import };
                    let progress_hwnd = unsafe { (*state_ptr).progress };
                    unsafe { (*state_ptr).receiver = Some(rx) };
                    unsafe {
                        EnableWindow(import_hwnd, 0);
                        set_text(progress_hwnd, "正在查询，请稍候…");
                        SetTimer(hwnd, TIMER_WORK, 100, null_mut());
                    }
                }
                _ => {}
            }
            0
        }
        WM_TIMER if wparam == TIMER_WORK => {
            let state_ptr = unsafe { GetWindowLongPtrW(hwnd, GWLP_USERDATA) as *mut UiState };
            if state_ptr.is_null() {
                return 0;
            }
            let result =
                unsafe { (*state_ptr).receiver.as_ref() }.and_then(|rx| match rx.try_recv() {
                    Ok(result) => Some(result),
                    Err(mpsc::TryRecvError::Disconnected) => {
                        Some(Err("后台处理意外中断，请重试。".to_string()))
                    }
                    Err(mpsc::TryRecvError::Empty) => None,
                });
            if let Some(result) = result {
                let import_hwnd = unsafe { (*state_ptr).import };
                let progress_hwnd = unsafe { (*state_ptr).progress };
                unsafe {
                    (*state_ptr).receiver = None;
                    (*state_ptr).busy = false
                };
                unsafe {
                    KillTimer(hwnd, TIMER_WORK);
                    EnableWindow(import_hwnd, 1);
                }
                match result {
                    Ok((path, summary)) => {
                        let detail = format!(
                            "已保存：{}\r\n共 {} 行，查到 {} 行；格式错误 {} 行，未收录 {} 行，空行 {} 行。",
                            visible_output_path(&path),
                            summary.rows,
                            summary.found,
                            summary.invalid,
                            summary.missing,
                            summary.empty
                        );
                        unsafe { set_text(progress_hwnd, &detail) };
                    }
                    Err(reason) => unsafe {
                        set_text(progress_hwnd, "处理失败。可检查文件后重试。");
                        alert(hwnd, &reason, true);
                    },
                }
            }
            0
        }
        WM_ERASEBKGND => {
            let state = unsafe { GetWindowLongPtrW(hwnd, GWLP_USERDATA) as *mut UiState };
            if state.is_null() {
                return 0;
            }
            let brush = unsafe { (*state).theme.surface_brush };
            let colors = unsafe { (*state).theme.colors };
            let layout = unsafe { (*state).layout };
            let dpi = unsafe { (*state).dpi };
            let scroll = unsafe { (*state).scroll } as f32 * 96.0 / dpi as f32;
            let icon = unsafe { (*state).icon };
            let mut rect = Rect {
                left: 0,
                top: 0,
                right: 0,
                bottom: 0,
            };
            unsafe {
                GetClientRect(hwnd, &mut rect);
                FillRect(wparam as Handle, &rect, brush);
                if !icon.is_null() {
                    DrawIconEx(
                        wparam as Handle,
                        pixels(
                            layout.margin
                                + ((rect.right as f32 * 96.0 / dpi as f32
                                    - 2.0 * layout.margin
                                    - layout.width)
                                    .max(0.0)
                                    / 2.0),
                            dpi,
                        ),
                        pixels(20.0, dpi),
                        icon,
                        pixels(24.0, dpi),
                        pixels(24.0, dpi),
                        0,
                        null_mut(),
                        3,
                    );
                }
            }
            if let Ok(canvas) = Canvas::new(wparam as Handle, dpi) {
                let client_width = rect.right as f32 * 96.0 / dpi as f32;
                let _ = canvas.line(0.0, 65.0, client_width, 65.0, 1.0, rgb(colors.border));
                let _ = canvas.fill_round_rect(
                    layout.margin
                        + ((rect.right as f32 * 96.0 / dpi as f32
                            - 2.0 * layout.margin
                            - layout.width)
                            .max(0.0)
                            / 2.0),
                    190.0 - scroll,
                    layout.width,
                    72.0,
                    18.0,
                    rgb(colors.field),
                );
                let _ = canvas.stroke_round_rect(
                    layout.margin
                        + ((rect.right as f32 * 96.0 / dpi as f32
                            - 2.0 * layout.margin
                            - layout.width)
                            .max(0.0)
                            / 2.0),
                    190.0 - scroll,
                    layout.width,
                    72.0,
                    18.0,
                    1.0,
                    rgb(colors.border),
                );
            }
            1
        }
        WM_CTLCOLORSTATIC | WM_CTLCOLOREDIT => {
            let state = unsafe { GetWindowLongPtrW(hwnd, GWLP_USERDATA) as *mut UiState };
            if state.is_null() {
                return 0;
            }
            let colors = unsafe { (*state).theme.colors };
            let surface_brush = unsafe { (*state).theme.surface_brush };
            let field_brush = unsafe { (*state).theme.field_brush };
            let edit = msg == WM_CTLCOLOREDIT;
            let secondary = !edit
                && (lparam as Hwnd == unsafe { (*state).offline }
                    || lparam as Hwnd == unsafe { (*state).subtitle }
                    || lparam as Hwnd == unsafe { (*state).result_label }
                    || lparam as Hwnd == unsafe { (*state).caveat }
                    || lparam as Hwnd == unsafe { (*state).progress });
            unsafe {
                SetTextColor(
                    wparam as Handle,
                    if secondary { colors.muted } else { colors.text },
                );
                SetBkColor(
                    wparam as Handle,
                    if edit { colors.field } else { colors.surface },
                );
                SetBkMode(wparam as Handle, TRANSPARENT);
            }
            if edit {
                field_brush as Lresult
            } else {
                surface_brush as Lresult
            }
        }
        WM_DRAWITEM => {
            let state = unsafe { GetWindowLongPtrW(hwnd, GWLP_USERDATA) as *mut UiState };
            if state.is_null() || lparam == 0 {
                return 0;
            }
            let item = unsafe { *(lparam as *const DrawItem) };
            if ![ID_QUERY as u32, ID_IMPORT as u32, ID_RESULT as u32].contains(&item.control_id) {
                return 0;
            }
            let colors = unsafe { (*state).theme.colors };
            let dpi = unsafe { (*state).dpi };
            let font = unsafe { (*state).font };
            let small_font = unsafe { (*state).small_font };
            let input_font = unsafe { (*state).input_font };
            let result = unsafe { (*state).result_view.clone() };
            let columns = unsafe { (*state).layout.columns };
            let high_contrast = unsafe { (*state).theme.high_contrast };
            let w = (item.rect.right - item.rect.left) as f32 * 96.0 / dpi as f32;
            let h = (item.rect.bottom - item.rect.top) as f32 * 96.0 / dpi as f32;
            let selected = item.state & ODS_SELECTED != 0;
            let disabled = item.state & ODS_DISABLED != 0;
            let focused = item.state & ODS_FOCUS != 0;
            unsafe {
                FillRect(item.dc, &item.rect, (*state).theme.surface_brush);
            }
            if let Ok(canvas) = Canvas::new(item.dc, dpi) {
                match item.control_id as i32 {
                    ID_QUERY => {
                        let fill = if selected {
                            colors.field
                        } else {
                            colors.accent
                        };
                        let ink = if selected {
                            colors.text
                        } else {
                            colors.accent_text
                        };
                        let _ = canvas.fill_circle(w / 2.0, h / 2.0, 23.0, rgb(fill));
                        if selected || high_contrast {
                            let _ = canvas.stroke_circle(
                                w / 2.0,
                                h / 2.0,
                                22.5,
                                1.0,
                                rgb(colors.border),
                            );
                        }
                        let cx = w / 2.0;
                        let cy = h / 2.0;
                        let _ = canvas.line(cx, cy + 9.0, cx, cy - 8.0, 2.7, rgb(ink));
                        let _ = canvas.line(cx - 7.0, cy - 1.0, cx, cy - 8.0, 2.7, rgb(ink));
                        let _ = canvas.line(cx, cy - 8.0, cx + 7.0, cy - 1.0, 2.7, rgb(ink));
                        if focused {
                            let _ = canvas.stroke_circle(cx, cy, 18.0, 1.5, rgb(ink));
                        }
                    }
                    ID_IMPORT => {
                        let surface = if selected {
                            colors.button
                        } else {
                            colors.field
                        };
                        let _ =
                            canvas.fill_round_rect(1.0, 1.0, w - 2.0, h - 2.0, 15.0, rgb(surface));
                        let _ = canvas.stroke_round_rect(
                            1.0,
                            1.0,
                            w - 2.0,
                            h - 2.0,
                            15.0,
                            if focused { 2.0 } else { 1.0 },
                            rgb(if focused { colors.text } else { colors.border }),
                        );
                        let icon_ink = if disabled {
                            colors.disabled
                        } else {
                            colors.text
                        };
                        let _ = canvas.stroke_round_rect(
                            19.0,
                            19.0,
                            32.0,
                            32.0,
                            9.0,
                            1.2,
                            rgb(colors.border),
                        );
                        let _ = canvas.line(35.0, 28.0, 35.0, 42.0, 2.2, rgb(icon_ink));
                        let _ = canvas.line(28.0, 35.0, 42.0, 35.0, 2.2, rgb(icon_ink));
                        unsafe {
                            draw_text(
                                item.dc,
                                "导入 Excel 或 CSV 文件",
                                font,
                                icon_ink,
                                [64.0, 11.0, w - 80.0, 27.0],
                                dpi,
                                DT_SINGLELINE | DT_VCENTER,
                            );
                            draw_text(
                                item.dc,
                                "读取首列手机号，另存为含运营商、省份和地级市的四列文件",
                                small_font,
                                colors.muted,
                                [64.0, 38.0, w - 80.0, 25.0],
                                dpi,
                                DT_SINGLELINE | DT_VCENTER,
                            );
                        }
                    }
                    ID_RESULT => {
                        if let Some(view) = result {
                            let _ = canvas.fill_round_rect(
                                1.0,
                                1.0,
                                w - 2.0,
                                h - 2.0,
                                16.0,
                                rgb(colors.field),
                            );
                            let _ = canvas.stroke_round_rect(
                                1.0,
                                1.0,
                                w - 2.0,
                                h - 2.0,
                                16.0,
                                1.0,
                                rgb(colors.border),
                            );
                            let entries = [
                                ("运营商", view.carrier),
                                ("归属地省份", view.province),
                                ("归属地地级市", view.city),
                            ];
                            for (index, (label, value)) in entries.iter().enumerate() {
                                let (x, y, cell_w) = if columns {
                                    (
                                        20.0 + index as f32 * (w - 40.0) / 3.0,
                                        17.0,
                                        (w - 40.0) / 3.0 - 16.0,
                                    )
                                } else {
                                    (20.0, 14.0 + index as f32 * 68.0, w - 40.0)
                                };
                                unsafe {
                                    draw_text(
                                        item.dc,
                                        label,
                                        small_font,
                                        colors.muted,
                                        [x, y, cell_w, 22.0],
                                        dpi,
                                        DT_SINGLELINE | DT_VCENTER,
                                    );
                                    draw_text(
                                        item.dc,
                                        value,
                                        input_font,
                                        colors.text,
                                        [x, y + 27.0, cell_w, 67.0],
                                        dpi,
                                        DT_WORDBREAK,
                                    );
                                }
                            }
                        } else {
                            let message = unsafe { text_of(item.hwnd) };
                            unsafe {
                                draw_text(
                                    item.dc,
                                    &message,
                                    font,
                                    colors.text,
                                    [0.0, 0.0, w, h],
                                    dpi,
                                    DT_WORDBREAK | DT_NOPREFIX,
                                );
                            }
                        }
                    }
                    _ => {}
                }
            }
            1
        }
        WM_SIZE => {
            unsafe {
                reflow(hwnd);
            }
            0
        }
        WM_DPICHANGED => {
            let state = unsafe { GetWindowLongPtrW(hwnd, GWLP_USERDATA) as *mut UiState };
            if state.is_null() {
                return 0;
            }
            let dpi = ((wparam & 0xffff) as u32).max(96);
            let new_fonts = unsafe {
                [
                    make_font(17.0, 400, dpi),
                    make_font(34.0, 650, dpi),
                    make_font(24.0, 400, dpi),
                    make_font(14.0, 400, dpi),
                ]
            };
            if new_fonts.iter().any(|font| font.is_null()) {
                for font in new_fonts {
                    if !font.is_null() {
                        unsafe {
                            DeleteObject(font);
                        }
                    }
                }
                return 0;
            }
            let controls = unsafe {
                [
                    (*state).brand,
                    (*state).offline,
                    (*state).title,
                    (*state).subtitle,
                    (*state).input,
                    (*state).query,
                    (*state).result_label,
                    (*state).result,
                    (*state).caveat,
                    (*state).import,
                    (*state).progress,
                ]
            };
            let old_fonts = unsafe {
                [
                    (*state).font,
                    (*state).title_font,
                    (*state).input_font,
                    (*state).small_font,
                ]
            };
            unsafe {
                (*state).font = new_fonts[0];
                (*state).title_font = new_fonts[1];
                (*state).input_font = new_fonts[2];
                (*state).small_font = new_fonts[3];
                (*state).dpi = dpi;
                (*state).scroll = 0;
                for (index, control) in controls.into_iter().enumerate() {
                    let font = match index {
                        2 => new_fonts[1],
                        4 | 5 | 7 => new_fonts[2],
                        1 | 6 | 8 | 10 => new_fonts[3],
                        _ => new_fonts[0],
                    };
                    SendMessageW(control, WM_SETFONT, font as usize, 0);
                }
                for font in old_fonts {
                    DeleteObject(font);
                }
                let icon = LoadImageW(
                    GetModuleHandleW(null()),
                    APP_ICON as *const u16,
                    IMAGE_ICON,
                    pixels(24.0, dpi),
                    pixels(24.0, dpi),
                    LR_SHARED,
                );
                if !icon.is_null() {
                    (*state).icon = icon;
                    SendMessageW(hwnd, WM_SETICON, ICON_SMALL, icon as isize);
                }
                if lparam != 0 {
                    let rect = *(lparam as *const Rect);
                    SetWindowPos(
                        hwnd,
                        null_mut(),
                        rect.left,
                        rect.top,
                        rect.right - rect.left,
                        rect.bottom - rect.top,
                        SWP_NOZORDER | SWP_NOACTIVATE | SWP_NOOWNERZORDER,
                    );
                }
                reflow(hwnd);
            }
            0
        }
        WM_GETMINMAXINFO => {
            if lparam != 0 {
                let dpi = unsafe { dpi_for_window(hwnd) }.max(96);
                unsafe {
                    (*(lparam as *mut MinMaxInfo)).min_track_size = Point {
                        x: pixels(540.0, dpi),
                        y: pixels(350.0, dpi),
                    };
                }
            }
            0
        }
        WM_VSCROLL | WM_MOUSEWHEEL => {
            let state = unsafe { GetWindowLongPtrW(hwnd, GWLP_USERDATA) as *mut UiState };
            if state.is_null() {
                return 0;
            }
            let dpi = unsafe { (*state).dpi };
            let mut scroll = unsafe { (*state).scroll };
            if msg == WM_MOUSEWHEEL {
                let delta = ((wparam >> 16) as u16) as i16 as i32;
                scroll -= delta.signum() * pixels(65.0, dpi);
            } else {
                let request = wparam & 0xffff;
                let mut client = Rect {
                    left: 0,
                    top: 0,
                    right: 0,
                    bottom: 0,
                };
                unsafe {
                    GetClientRect(hwnd, &mut client);
                }
                scroll = match request {
                    SB_LINEUP => scroll - pixels(40.0, dpi),
                    SB_LINEDOWN => scroll + pixels(40.0, dpi),
                    SB_PAGEUP => scroll - client.bottom,
                    SB_PAGEDOWN => scroll + client.bottom,
                    SB_THUMBTRACK => {
                        let mut info = ScrollInfo {
                            size: std::mem::size_of::<ScrollInfo>() as u32,
                            mask: SIF_TRACKPOS,
                            min: 0,
                            max: 0,
                            page: 0,
                            pos: 0,
                            track_pos: 0,
                        };
                        unsafe {
                            GetScrollInfo(hwnd, SB_VERT, &mut info);
                        }
                        info.track_pos
                    }
                    _ => scroll,
                };
            }
            unsafe {
                (*state).scroll = scroll;
                reflow(hwnd);
            }
            0
        }
        WM_SETTINGCHANGE | WM_SYSCOLORCHANGE | WM_THEMECHANGED => {
            let state = unsafe { GetWindowLongPtrW(hwnd, GWLP_USERDATA) as *mut UiState };
            if !state.is_null() {
                let theme = unsafe { Theme::read(hwnd) };
                let old = unsafe { std::mem::replace(&mut (*state).theme, theme) };
                unsafe {
                    old.delete();
                    RedrawWindow(
                        hwnd,
                        null(),
                        null_mut(),
                        RDW_INVALIDATE | RDW_ERASE | RDW_ALLCHILDREN,
                    );
                }
            }
            0
        }
        WM_CLOSE => {
            let state = unsafe { GetWindowLongPtrW(hwnd, GWLP_USERDATA) as *mut UiState };
            if !state.is_null() && unsafe { (*state).busy } {
                let progress = unsafe { (*state).progress };
                unsafe {
                    set_text(progress, "正在处理文件。处理完成后即可关闭窗口。")
                };
                return 0;
            }
            unsafe { DefWindowProcW(hwnd, msg, wparam, lparam) }
        }
        WM_DESTROY => {
            let state = unsafe { SetWindowLongPtrW(hwnd, GWLP_USERDATA, 0) as *mut UiState };
            if !state.is_null() {
                let state = unsafe { Box::from_raw(state) };
                unsafe {
                    DeleteObject(state.font);
                    DeleteObject(state.title_font);
                    DeleteObject(state.input_font);
                    DeleteObject(state.small_font);
                    state.theme.delete();
                }
            }
            unsafe { PostQuitMessage(0) };
            0
        }
        _ => unsafe { DefWindowProcW(hwnd, msg, wparam, lparam) },
    }
}

unsafe fn create_main_window() -> Result<Hwnd, String> {
    unsafe {
        let instance = GetModuleHandleW(null());
        let class_name = wide("PhoneRegionMainWindow");
        let class = WndClass {
            style: 0,
            procedure: Some(window_proc),
            cls_extra: 0,
            wnd_extra: 0,
            instance,
            icon: LoadIconW(instance, APP_ICON as *const u16),
            cursor: LoadCursorW(null_mut(), IDC_ARROW as *const u16),
            background: GetSysColorBrush(COLOR_BTNFACE),
            menu_name: null(),
            class_name: class_name.as_ptr(),
        };
        if RegisterClassW(&class) == 0 {
            return Err("无法注册窗口".to_string());
        }
        let title = wide("手机号归属地 · 离线查询");
        let available = work_area();
        let system_dpi = system_dpi().max(96);
        let width = pixels(960.0, system_dpi).min(available.right - available.left);
        let height = pixels(700.0, system_dpi).min(available.bottom - available.top);
        let hwnd = CreateWindowExW(
            0,
            class_name.as_ptr(),
            title.as_ptr(),
            WS_CAPTION
                | WS_SYSMENU
                | WS_MINIMIZEBOX
                | WS_MAXIMIZEBOX
                | WS_THICKFRAME
                | WS_VSCROLL
                | WS_VISIBLE,
            available.left + (available.right - available.left - width).max(0) / 2,
            available.top + (available.bottom - available.top - height).max(0) / 2,
            width,
            height,
            null_mut(),
            null_mut(),
            instance,
            null_mut(),
        );
        if hwnd.is_null() {
            Err("无法创建窗口或读取离线号段库".to_string())
        } else {
            Ok(hwnd)
        }
    }
}

pub fn run() -> Result<(), String> {
    unsafe {
        let hwnd = create_main_window()?;
        ShowWindow(hwnd, SW_SHOW);
        UpdateWindow(hwnd);
        let mut message = std::mem::zeroed::<Message>();
        while GetMessageW(&mut message, null_mut(), 0, 0) > 0 {
            let state = GetWindowLongPtrW(hwnd, GWLP_USERDATA) as *mut UiState;
            if message.message == WM_KEYDOWN
                && message.wparam == VK_RETURN
                && !state.is_null()
                && message.hwnd == (*state).input
            {
                SendMessageW(hwnd, WM_COMMAND, ID_QUERY as usize, 0);
                continue;
            }
            if IsDialogMessageW(hwnd, &message) != 0 {
                continue;
            }
            TranslateMessage(&message);
            DispatchMessageW(&message);
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::colors_for_mode;

    fn luminance(color: u32) -> f64 {
        let channels = [color & 0xff, (color >> 8) & 0xff, (color >> 16) & 0xff];
        let mut linear = [0.0; 3];
        for (index, channel) in channels.into_iter().enumerate() {
            let value = channel as f64 / 255.0;
            linear[index] = if value <= 0.04045 {
                value / 12.92
            } else {
                ((value + 0.055) / 1.055).powf(2.4)
            };
        }
        0.2126 * linear[0] + 0.7152 * linear[1] + 0.0722 * linear[2]
    }

    fn contrast(first: u32, second: u32) -> f64 {
        let a = luminance(first);
        let b = luminance(second);
        (a.max(b) + 0.05) / (a.min(b) + 0.05)
    }

    #[test]
    fn text_is_readable_in_both_application_themes() {
        for dark in [false, true] {
            let colors = colors_for_mode(dark);
            assert!(contrast(colors.text, colors.surface) >= 4.5);
            assert!(contrast(colors.text, colors.field) >= 4.5);
            assert!(contrast(colors.muted, colors.surface) >= 4.5);
            assert!(contrast(colors.disabled, colors.button) >= 3.0);
            assert!(contrast(colors.accent_text, colors.accent) >= 7.0);
        }
    }

    #[test]
    fn long_output_path_keeps_a_readable_filename() {
        let path = std::path::Path::new(
            r"C:\Users\Example\Very Long Directory Name\Another Long Directory\result.csv",
        );
        assert_eq!(super::visible_output_path(path), "result.csv");
    }
}
