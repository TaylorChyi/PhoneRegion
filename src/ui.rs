//! Small native Windows interface for the bundled, offline number database.

use crate::batch;
use crate::lookup::{Database, Status};
use std::ffi::{OsString, c_void};
use std::os::windows::ffi::{OsStrExt, OsStringExt};
use std::path::{Path, PathBuf};
use std::ptr::{null, null_mut};
use std::sync::{Arc, mpsc};

type Handle = *mut c_void;
type Hwnd = Handle;
type Wparam = usize;
type Lparam = isize;
type Lresult = isize;

const WS_CAPTION: u32 = 0x00C00000;
const WS_SYSMENU: u32 = 0x00080000;
const WS_MINIMIZEBOX: u32 = 0x00020000;
const WS_VISIBLE: u32 = 0x10000000;
const WS_CHILD: u32 = 0x40000000;
const WS_TABSTOP: u32 = 0x00010000;
const ES_AUTOHSCROLL: u32 = 0x0080;
const BS_OWNERDRAW: u32 = 0x000B;
const SS_LEFT: u32 = 0;
const SS_CENTER: u32 = 1;
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
const WM_SETFONT: u32 = 0x0030;
const WM_KEYDOWN: u32 = 0x0100;
const EM_SETCUEBANNER: u32 = 0x1501;
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
const DT_CENTER: u32 = 1;
const DT_VCENTER: u32 = 4;
const DT_SINGLELINE: u32 = 0x20;
const TRANSPARENT: i32 = 1;
const ODS_SELECTED: u32 = 1;
const ODS_DISABLED: u32 = 4;
const ODS_FOCUS: u32 = 0x10;
const RDW_INVALIDATE: u32 = 1;
const RDW_ERASE: u32 = 4;
const RDW_ALLCHILDREN: u32 = 0x80;

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
    fn GetSystemMetrics(index: i32) -> i32;
    fn IsDialogMessageW(hwnd: Hwnd, message: *const Message) -> i32;
    fn GetSysColor(index: i32) -> u32;
    fn SystemParametersInfoW(action: u32, param: u32, result: *mut c_void, flags: u32) -> i32;
    fn RedrawWindow(hwnd: Hwnd, rect: *const Rect, region: Handle, flags: u32) -> i32;
    fn FillRect(dc: Handle, rect: *const Rect, brush: Handle) -> i32;
    fn DrawTextW(dc: Handle, text: *const u16, count: i32, rect: *mut Rect, format: u32) -> i32;
    fn DrawFocusRect(dc: Handle, rect: *const Rect) -> i32;
    fn GetClientRect(hwnd: Hwnd, rect: *mut Rect) -> i32;
}

#[link(name = "kernel32")]
unsafe extern "system" {
    fn GetModuleHandleW(name: *const u16) -> Handle;
}

#[link(name = "gdi32")]
unsafe extern "system" {
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
    fn CreatePen(style: i32, width: i32, color: u32) -> Handle;
    fn RoundRect(
        dc: Handle,
        left: i32,
        top: i32,
        right: i32,
        bottom: i32,
        ellipse_width: i32,
        ellipse_height: i32,
    ) -> i32;
    fn Ellipse(dc: Handle, left: i32, top: i32, right: i32, bottom: i32) -> i32;
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
    input: Hwnd,
    result: Hwnd,
    result_label: Hwnd,
    caveat: Hwnd,
    progress: Hwnd,
    import: Hwnd,
    font: Handle,
    title_font: Handle,
    input_font: Handle,
    compact: bool,
    theme: Theme,
    busy: bool,
    receiver: Option<mpsc::Receiver<Result<(PathBuf, batch::Summary), String>>>,
}

#[derive(Clone, Copy)]
struct Colors {
    surface: u32,
    text: u32,
    field: u32,
    button: u32,
    border: u32,
    disabled: u32,
    button_text: u32,
    muted: u32,
    accent: u32,
    accent_text: u32,
}

struct Theme {
    colors: Colors,
    high_contrast: bool,
    surface_brush: Handle,
    field_brush: Handle,
    button_brush: Handle,
    border_brush: Handle,
    accent_brush: Handle,
    border_pen: Handle,
    accent_pen: Handle,
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
                    button_text: GetSysColor(COLOR_BTNTEXT),
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
                button_brush: CreateSolidBrush(colors.button),
                border_brush: CreateSolidBrush(colors.border),
                accent_brush: CreateSolidBrush(colors.accent),
                border_pen: CreatePen(0, 1, colors.border),
                accent_pen: CreatePen(0, 1, colors.accent),
            }
        }
    }

    unsafe fn delete(self) {
        unsafe {
            DeleteObject(self.surface_brush);
            DeleteObject(self.field_brush);
            DeleteObject(self.button_brush);
            DeleteObject(self.border_brush);
            DeleteObject(self.accent_brush);
            DeleteObject(self.border_pen);
            DeleteObject(self.accent_pen);
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
            button_text: 0x00F3F3F3,
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
            button_text: 0x001E1E1E,
            muted: 0x006A6A6A,
            accent: 0x001E1E1E,
            accent_text: 0x00FFFFFF,
        }
    }
}

fn wide(value: &str) -> Vec<u16> {
    value.encode_utf16().chain(std::iter::once(0)).collect()
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
                let font = CreateFontW(
                    -18,
                    0,
                    0,
                    0,
                    400,
                    0,
                    0,
                    0,
                    1,
                    0,
                    0,
                    5,
                    0,
                    wide("Microsoft YaHei UI").as_ptr(),
                );
                if font.is_null() {
                    return -1;
                }
                let title_font = CreateFontW(
                    -30,
                    0,
                    0,
                    0,
                    600,
                    0,
                    0,
                    0,
                    1,
                    0,
                    0,
                    5,
                    0,
                    wide("Microsoft YaHei UI").as_ptr(),
                );
                let input_font = CreateFontW(
                    -25,
                    0,
                    0,
                    0,
                    400,
                    0,
                    0,
                    0,
                    1,
                    0,
                    0,
                    5,
                    0,
                    wide("Microsoft YaHei UI").as_ptr(),
                );
                if title_font.is_null() || input_font.is_null() {
                    DeleteObject(font);
                    if !title_font.is_null() {
                        DeleteObject(title_font);
                    }
                    if !input_font.is_null() {
                        DeleteObject(input_font);
                    }
                    return -1;
                }
                let theme = Theme::read(hwnd);
                let compact = work_area().bottom - work_area().top < 610;
                create_control(
                    hwnd,
                    "STATIC",
                    "手机号归属地",
                    SS_CENTER,
                    0,
                    if compact {
                        [64, 16, 592, 42]
                    } else {
                        [64, 62, 592, 48]
                    },
                    title_font,
                );
                let input = create_control(
                    hwnd,
                    "EDIT",
                    "",
                    WS_TABSTOP | ES_AUTOHSCROLL,
                    ID_INPUT,
                    if compact {
                        [90, 89, 475, 38]
                    } else {
                        [90, 177, 475, 42]
                    },
                    input_font,
                );
                SendMessageW(
                    input,
                    EM_SETCUEBANNER,
                    1,
                    wide("例如 13213000000").as_ptr() as isize,
                );
                let import = create_control(
                    hwnd,
                    "BUTTON",
                    "＋  导入文件",
                    WS_TABSTOP | BS_OWNERDRAW,
                    ID_IMPORT,
                    if compact {
                        [84, 137, 171, 40]
                    } else {
                        [84, 230, 171, 42]
                    },
                    font,
                );
                create_control(
                    hwnd,
                    "BUTTON",
                    "查询",
                    WS_TABSTOP | BS_OWNERDRAW,
                    ID_QUERY,
                    if compact {
                        [580, 132, 48, 48]
                    } else {
                        [580, 225, 48, 48]
                    },
                    input_font,
                );
                let result_label = create_control(
                    hwnd,
                    "STATIC",
                    "",
                    SS_LEFT,
                    ID_RESULT_LABEL,
                    if compact {
                        [78, 207, 560, 24]
                    } else {
                        [78, 319, 560, 27]
                    },
                    font,
                );
                let result = create_control(
                    hwnd,
                    "STATIC",
                    "",
                    SS_LEFT,
                    ID_RESULT,
                    if compact {
                        [78, 235, 560, 76]
                    } else {
                        [78, 354, 560, 89]
                    },
                    input_font,
                );
                let caveat = create_control(
                    hwnd,
                    "STATIC",
                    "",
                    SS_LEFT,
                    ID_CAVEAT,
                    if compact {
                        [78, 312, 560, 22]
                    } else {
                        [78, 450, 560, 26]
                    },
                    font,
                );
                let progress = create_control(
                    hwnd,
                    "STATIC",
                    "支持 Excel / CSV，读取第一列，结果另存为四列文件。",
                    SS_LEFT,
                    ID_PROGRESS,
                    if compact {
                        [78, 342, 560, 64]
                    } else {
                        [78, 494, 560, 64]
                    },
                    font,
                );
                let state = Box::new(UiState {
                    database,
                    input,
                    result,
                    result_label,
                    caveat,
                    progress,
                    import,
                    font,
                    title_font,
                    input_font,
                    compact,
                    theme,
                    busy: false,
                    receiver: None,
                });
                SetWindowLongPtrW(hwnd, GWLP_USERDATA, Box::into_raw(state) as isize);
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
                    let detail = match found.status {
                        Status::Found => format!(
                            "{}\r\n{}  {}",
                            found.carrier,
                            found.province,
                            if found.city.is_empty() {
                                "离线库未提供唯一地市"
                            } else {
                                &found.city
                            }
                        ),
                        Status::Invalid if input_changed => "等待完整的 11 位手机号。".to_string(),
                        Status::Invalid => "请输入 11 位中国大陆手机号。".to_string(),
                        Status::Missing => "离线号段库未收录该号码前 7 位。".to_string(),
                        Status::Empty if input_changed => String::new(),
                        Status::Empty => "请输入手机号。".to_string(),
                    };
                    unsafe {
                        set_text(label_hwnd, if success { "号段查询结果" } else { "" });
                        set_text(result_hwnd, &detail);
                        set_text(
                            caveat_hwnd,
                            if success {
                                "运营商为原号段信息。携号转网后，当前运营商可能不同。"
                            } else {
                                ""
                            },
                        );
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
                            path.display(),
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
            let field_brush = unsafe { (*state).theme.field_brush };
            let border_pen = unsafe { (*state).theme.border_pen };
            let compact = unsafe { (*state).compact };
            let mut rect = Rect {
                left: 0,
                top: 0,
                right: 0,
                bottom: 0,
            };
            unsafe {
                GetClientRect(hwnd, &mut rect);
                FillRect(wparam as Handle, &rect, brush);
                let old_brush = SelectObject(wparam as Handle, field_brush);
                let old_pen = SelectObject(wparam as Handle, border_pen);
                if compact {
                    RoundRect(wparam as Handle, 64, 70, 656, 190, 34, 34);
                } else {
                    RoundRect(wparam as Handle, 64, 155, 656, 287, 34, 34);
                }
                SelectObject(wparam as Handle, old_pen);
                SelectObject(wparam as Handle, old_brush);
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
                && (lparam as Hwnd == unsafe { (*state).result_label }
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
            if item.control_id != ID_QUERY as u32 && item.control_id != ID_IMPORT as u32 {
                return 0;
            }
            let colors = unsafe { (*state).theme.colors };
            let high_contrast = unsafe { (*state).theme.high_contrast };
            let field_brush = unsafe { (*state).theme.field_brush };
            let button_brush = unsafe { (*state).theme.button_brush };
            let accent_brush = unsafe { (*state).theme.accent_brush };
            let border_pen = unsafe { (*state).theme.border_pen };
            let accent_pen = unsafe { (*state).theme.accent_pen };
            let font = unsafe {
                if item.control_id == ID_QUERY as u32 {
                    (*state).input_font
                } else {
                    (*state).font
                }
            };
            let disabled = item.state & ODS_DISABLED != 0;
            let selected = item.state & ODS_SELECTED != 0;
            let query = item.control_id == ID_QUERY as u32;
            let text_color = if query || selected && high_contrast {
                colors.accent_text
            } else if disabled {
                colors.disabled
            } else {
                colors.button_text
            };
            let label = if query {
                "↑".to_string()
            } else {
                unsafe { text_of(item.hwnd) }
            };
            let mut rect = item.rect;
            unsafe {
                FillRect(item.dc, &rect, field_brush);
                if query {
                    let old_brush = SelectObject(item.dc, accent_brush);
                    let old_pen = SelectObject(item.dc, accent_pen);
                    Ellipse(
                        item.dc,
                        rect.left + 1,
                        rect.top + 1,
                        rect.right - 1,
                        rect.bottom - 1,
                    );
                    SelectObject(item.dc, old_pen);
                    SelectObject(item.dc, old_brush);
                } else if selected {
                    let fill = if high_contrast {
                        accent_brush
                    } else {
                        button_brush
                    };
                    let old_brush = SelectObject(item.dc, fill);
                    let old_pen = SelectObject(item.dc, border_pen);
                    RoundRect(
                        item.dc,
                        rect.left + 1,
                        rect.top + 1,
                        rect.right - 1,
                        rect.bottom - 1,
                        20,
                        20,
                    );
                    SelectObject(item.dc, old_pen);
                    SelectObject(item.dc, old_brush);
                }
                SelectObject(item.dc, font);
                SetBkMode(item.dc, TRANSPARENT);
                SetTextColor(item.dc, text_color);
                DrawTextW(
                    item.dc,
                    wide(&label).as_ptr(),
                    -1,
                    &mut rect,
                    DT_CENTER | DT_VCENTER | DT_SINGLELINE,
                );
                if item.state & ODS_FOCUS != 0 {
                    let focus = Rect {
                        left: rect.left + 4,
                        top: rect.top + 4,
                        right: rect.right - 4,
                        bottom: rect.bottom - 4,
                    };
                    DrawFocusRect(item.dc, &focus);
                }
            }
            1
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
                    state.theme.delete();
                }
            }
            unsafe { PostQuitMessage(0) };
            0
        }
        _ => unsafe { DefWindowProcW(hwnd, msg, wparam, lparam) },
    }
}

pub fn run() -> Result<(), String> {
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
            background: GetSysColorBrush(15),
            menu_name: null(),
            class_name: class_name.as_ptr(),
        };
        if RegisterClassW(&class) == 0 {
            return Err("无法注册窗口".to_string());
        }
        let title = wide("手机号归属地 · 离线查询");
        let available = work_area();
        let height = if available.bottom - available.top < 610 {
            440
        } else {
            610
        };
        let hwnd = CreateWindowExW(
            0,
            class_name.as_ptr(),
            title.as_ptr(),
            WS_CAPTION | WS_SYSMENU | WS_MINIMIZEBOX | WS_VISIBLE,
            available.left + (available.right - available.left - 720).max(0) / 2,
            available.top + (available.bottom - available.top - height).max(0) / 2,
            720,
            height,
            null_mut(),
            null_mut(),
            instance,
            null_mut(),
        );
        if hwnd.is_null() {
            return Err("无法创建窗口或读取离线号段库".to_string());
        }
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
            assert!(contrast(colors.button_text, colors.button) >= 4.5);
            assert!(contrast(colors.disabled, colors.button) >= 3.0);
        }
    }
}
