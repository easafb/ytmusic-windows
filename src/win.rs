//! Küçük Win32 yardımcıları: tek örnek kilidi, tarayıcıda açma, hata kutusu.

use std::ptr::{null, null_mut};

use windows_sys::Win32::Foundation::{GetLastError, ERROR_ALREADY_EXISTS};
use windows_sys::Win32::System::Threading::CreateMutexW;
use windows_sys::Win32::UI::Shell::ShellExecuteW;
use windows_sys::Win32::UI::WindowsAndMessaging::{
    FindWindowW, IsIconic, MessageBoxW, SetForegroundWindow, ShowWindow, MB_ICONERROR, MB_OK,
    SW_RESTORE, SW_SHOW, SW_SHOWNORMAL,
};

fn wide(s: &str) -> Vec<u16> {
    s.encode_utf16().chain(std::iter::once(0)).collect()
}

/// Uygulama zaten açıksa var olan pencereyi öne getirir ve `false` döner.
pub fn claim_single_instance(window_title: &str) -> bool {
    let name = wide("Local\\YTMusicWindows");
    unsafe {
        // Kilit süreç boyunca açık kalmalı; kapatmıyoruz.
        let handle = CreateMutexW(null(), 1, name.as_ptr());
        if handle.is_null() || GetLastError() != ERROR_ALREADY_EXISTS {
            return true;
        }

        let title = wide(window_title);
        let hwnd = FindWindowW(null(), title.as_ptr());
        if !hwnd.is_null() {
            ShowWindow(hwnd, SW_SHOW);
            if IsIconic(hwnd) != 0 {
                ShowWindow(hwnd, SW_RESTORE);
            }
            SetForegroundWindow(hwnd);
        }
    }
    false
}

/// Bağlantıyı varsayılan tarayıcıda açar (yalnızca http/https).
pub fn open_in_browser(url: &str) {
    if !(url.starts_with("https://") || url.starts_with("http://")) {
        return;
    }
    let operation = wide("open");
    let file = wide(url);
    unsafe {
        ShellExecuteW(null_mut(), operation.as_ptr(), file.as_ptr(), null(), null(), SW_SHOWNORMAL);
    }
}

pub fn error_box(title: &str, message: &str) {
    let title = wide(title);
    let message = wide(message);
    unsafe {
        MessageBoxW(null_mut(), message.as_ptr(), title.as_ptr(), MB_OK | MB_ICONERROR);
    }
}
