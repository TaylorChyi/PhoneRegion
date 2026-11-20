//! File drops are accepted only by the import button's own HWND.

use std::ffi::{OsString, c_void};
use std::os::windows::ffi::OsStringExt;
use std::path::{Path, PathBuf};
use std::ptr::null_mut;

use super::{Hwnd, Lparam, Lresult, Wparam};

type Handle = *mut c_void;
const WM_DROPFILES: u32 = 0x0233;
const WM_NCDESTROY: u32 = 0x0082;
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
    fn RemoveWindowSubclass(
        hwnd: Hwnd,
        callback: Option<
            unsafe extern "system" fn(Hwnd, u32, Wparam, Lparam, usize, usize) -> Lresult,
        >,
        id: usize,
    ) -> i32;
    fn DefSubclassProc(hwnd: Hwnd, message: u32, wparam: Wparam, lparam: Lparam) -> Lresult;
}

#[link(name = "shell32")]
unsafe extern "system" {
    fn DragAcceptFiles(hwnd: Hwnd, accept: i32);
    fn DragQueryFileW(drop: Handle, index: u32, buffer: *mut u16, capacity: u32) -> u32;
    fn DragFinish(drop: Handle);
}

#[cfg(test)]
#[link(name = "kernel32")]
unsafe extern "system" {
    fn GlobalAlloc(flags: u32, bytes: usize) -> Handle;
    fn GlobalLock(memory: Handle) -> *mut c_void;
    fn GlobalUnlock(memory: Handle) -> i32;
}

#[cfg(test)]
#[link(name = "user32")]
unsafe extern "system" {
    fn GetDlgItem(parent: Hwnd, id: i32) -> Hwnd;
    fn IsWindowVisible(window: Hwnd) -> i32;
}

pub(super) enum DropChoice {
    File(PathBuf),
    Error(&'static str),
}

pub(super) fn install(button: Hwnd, parent: Hwnd) -> Result<(), String> {
    // SAFETY: Both HWNDs belong to the UI thread, and the subclass callback is static.
    if unsafe {
        SetWindowSubclass(
            button,
            Some(import_button_proc),
            SUBCLASS_ID,
            parent as usize,
        )
    } == 0
    {
        return Err("无法启用文件拖入".into());
    }
    // SAFETY: The button remains live until WM_NCDESTROY removes this registration.
    unsafe { DragAcceptFiles(button, 1) };
    Ok(())
}

pub(super) fn supported_input(path: &Path) -> bool {
    path.extension()
        .and_then(|value| value.to_str())
        .is_some_and(|value| matches!(value.to_ascii_lowercase().as_str(), "xls" | "xlsx" | "csv"))
}

struct DropGuard(Handle);
impl Drop for DropGuard {
    fn drop(&mut self) {
        // SAFETY: The HDROP is uniquely owned by this WM_DROPFILES handler.
        unsafe { DragFinish(self.0) };
    }
}

unsafe fn read_drop(drop: Handle) -> DropChoice {
    let _guard = DropGuard(drop);
    // SAFETY: The HDROP stays live until _guard is dropped.
    let count = unsafe { DragQueryFileW(drop, u32::MAX, null_mut(), 0) };
    if count != 1 {
        return DropChoice::Error("一次只能拖入一个 Excel 或 CSV 文件。");
    }
    let length = unsafe { DragQueryFileW(drop, 0, null_mut(), 0) };
    if length == 0 || length >= 32_767 {
        return DropChoice::Error("无法读取拖入文件的路径。");
    }
    let mut buffer = vec![0u16; length as usize + 1];
    let written = unsafe { DragQueryFileW(drop, 0, buffer.as_mut_ptr(), buffer.len() as u32) };
    if written != length {
        return DropChoice::Error("无法读取拖入文件的路径。");
    }
    let path = PathBuf::from(OsString::from_wide(&buffer[..written as usize]));
    if !supported_input(&path) || !path.is_file() {
        return DropChoice::Error("请拖入一个 .xls、.xlsx 或 .csv 文件。");
    }
    DropChoice::File(path)
}

unsafe extern "system" fn import_button_proc(
    hwnd: Hwnd,
    message: u32,
    wparam: Wparam,
    lparam: Lparam,
    id: usize,
    parent: usize,
) -> Lresult {
    if message == WM_DROPFILES {
        let choice = unsafe { read_drop(wparam as Handle) };
        super::handle_drop(parent as Hwnd, choice);
        return 0;
    }
    if message == WM_NCDESTROY {
        // SAFETY: The callback and subclass id match the registration above.
        unsafe {
            DragAcceptFiles(hwnd, 0);
            RemoveWindowSubclass(hwnd, Some(import_button_proc), id);
        }
    }
    unsafe { DefSubclassProc(hwnd, message, wparam, lparam) }
}

#[cfg(test)]
pub(super) fn assert_native_drop(window: Hwnd) {
    use std::os::windows::ffi::OsStrExt;
    use std::time::{Duration, Instant};

    #[repr(C)]
    struct DropFiles {
        offset: u32,
        point: super::Point,
        nonclient: i32,
        wide: i32,
    }

    let directory = tempfile::tempdir().unwrap();
    let input = directory.path().join("手机号 测试.CSV");
    std::fs::write(&input, "手机号\n13213000000\n").unwrap();
    let path = input
        .as_os_str()
        .encode_wide()
        .chain([0, 0])
        .collect::<Vec<_>>();
    let header = DropFiles {
        offset: std::mem::size_of::<DropFiles>() as u32,
        point: super::Point { x: 0, y: 0 },
        nonclient: 0,
        wide: 1,
    };
    let bytes = std::mem::size_of::<DropFiles>() + path.len() * 2;
    // A moveable HGLOBAL with DROPFILES and a double-NUL UTF-16 path mirrors Shell HDROP.
    let memory = unsafe { GlobalAlloc(0x0042, bytes) };
    assert!(!memory.is_null());
    let target = unsafe { GlobalLock(memory) } as *mut u8;
    assert!(!target.is_null());
    unsafe {
        std::ptr::copy_nonoverlapping(
            (&header as *const DropFiles).cast::<u8>(),
            target,
            std::mem::size_of::<DropFiles>(),
        );
        std::ptr::copy_nonoverlapping(
            path.as_ptr().cast::<u8>(),
            target.add(header.offset as usize),
            path.len() * 2,
        );
        GlobalUnlock(memory);
    }
    let import = unsafe { GetDlgItem(window, super::ID_IMPORT) };
    assert!(!import.is_null());
    // The subclass owns HDROP and calls DragFinish, including on validation errors.
    unsafe { super::SendMessageW(import, WM_DROPFILES, memory as usize, 0) };

    let deadline = Instant::now() + Duration::from_secs(10);
    loop {
        let state = unsafe {
            super::GetWindowLongPtrW(window, super::GWLP_USERDATA) as *mut super::UiState
        };
        assert!(!state.is_null());
        unsafe { super::SendMessageW(window, super::WM_TIMER, super::TIMER_WORK, 0) };
        if unsafe { (*state).prepared.is_some() } {
            assert_eq!(
                unsafe { (*state).prepared.as_ref().unwrap().summary.found },
                1
            );
            assert_ne!(unsafe { IsWindowVisible((*state).export) }, 0);
            break;
        }
        assert!(
            Instant::now() < deadline,
            "dragged file never reached ready state"
        );
        std::thread::sleep(Duration::from_millis(20));
    }
    assert_eq!(std::fs::read_dir(directory.path()).unwrap().count(), 1);
}

#[cfg(test)]
mod tests {
    use super::supported_input;
    use std::path::Path;

    #[test]
    fn validates_case_insensitive_file_extensions() {
        for path in [
            r"C:\数据 文件\号码.XLS",
            r"C:\数据 文件\号码.XlSx",
            r"C:\数据 文件\号码.CSV",
        ] {
            assert!(supported_input(Path::new(path)));
        }
        for path in [r"C:\号码.txt", r"C:\号码.csv.exe", r"C:\号码"] {
            assert!(!supported_input(Path::new(path)));
        }
    }
}
