//! Global hotkey Ctrl+Shift+F (handover §22, FR-102) and window foreground helpers.

use common::log;
use windows::core::PCWSTR;
use windows::Win32::Foundation::HWND;
use windows::Win32::UI::Input::KeyboardAndMouse::{RegisterHotKey, MOD_CONTROL, MOD_NOREPEAT, MOD_SHIFT};
use windows::Win32::UI::WindowsAndMessaging::{
    FindWindowW, GetMessageW, SetForegroundWindow, ShowWindow, MSG, SW_RESTORE, WM_HOTKEY,
};

const HOTKEY_ID: i32 = 0x5346; // "SF"
const VK_F: u32 = 0x46;

/// Registers Ctrl+Shift+F on a dedicated thread and calls `on_hotkey` for each press.
/// Returns false if registration failed (e.g. another app owns the hotkey).
pub fn register<F>(on_hotkey: F) -> bool
where
    F: Fn() + Send + 'static,
{
    let (tx, rx) = std::sync::mpsc::channel();
    std::thread::spawn(move || {
        // SAFETY: plain Win32 calls on this thread's message queue.
        let ok = unsafe { RegisterHotKey(None, HOTKEY_ID, MOD_CONTROL | MOD_SHIFT | MOD_NOREPEAT, VK_F) }.is_ok();
        let _ = tx.send(ok);
        if !ok {
            return;
        }
        let mut msg = MSG::default();
        // SAFETY: standard message loop; GetMessageW returns 0 on WM_QUIT, -1 on error.
        while unsafe { GetMessageW(&mut msg, None, 0, 0) }.0 > 0 {
            if msg.message == WM_HOTKEY && msg.wParam.0 as i32 == HOTKEY_ID {
                log::write("shortcut", "hotkey Ctrl+Shift+F received");
                on_hotkey();
            }
        }
    });
    let ok = rx.recv().unwrap_or(false);
    log::write("shortcut", &format!("register Ctrl+Shift+F ok={ok}"));
    ok
}

pub fn find_window(title: &str) -> Option<HWND> {
    let t: Vec<u16> = title.encode_utf16().chain(std::iter::once(0)).collect();
    // SAFETY: NUL-terminated title buffer outlives the call.
    unsafe { FindWindowW(PCWSTR::null(), PCWSTR(t.as_ptr())) }.ok().filter(|h| !h.is_invalid())
}

/// Restores and brings the window with `title` to the foreground.
pub fn bring_to_front(title: &str) -> bool {
    let Some(hwnd) = find_window(title) else {
        log::write("shortcut", &format!("bring_to_front window not found title={title}"));
        return false;
    };
    // SAFETY: hwnd comes from FindWindowW.
    let ok = unsafe {
        let _ = ShowWindow(hwnd, SW_RESTORE);
        SetForegroundWindow(hwnd).as_bool()
    };
    log::write("shortcut", &format!("bring_to_front foreground={ok}"));
    ok
}
