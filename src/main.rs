#![cfg_attr(windows, windows_subsystem = "windows")]

#[cfg(windows)]
use phone_region::{batch, lookup};
#[cfg(windows)]
mod ui;

#[cfg(windows)]
fn main() {
    if let Err(reason) = ui::run() {
        use std::os::windows::ffi::OsStrExt;
        #[link(name = "user32")]
        unsafe extern "system" {
            fn MessageBoxW(
                parent: *mut std::ffi::c_void,
                message: *const u16,
                title: *const u16,
                kind: u32,
            ) -> i32;
        }
        let message: Vec<u16> = std::ffi::OsStr::new(&reason)
            .encode_wide()
            .chain(Some(0))
            .collect();
        let title: Vec<u16> = "手机号归属地查询".encode_utf16().chain(Some(0)).collect();
        unsafe {
            MessageBoxW(std::ptr::null_mut(), message.as_ptr(), title.as_ptr(), 0x10);
        }
        std::process::exit(1);
    }
}

#[cfg(not(windows))]
fn main() {
    eprintln!("图形界面需要 Windows 10 或 11。");
}
