#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use common::log;
use skim_search::{app, MainWindow};
use slint::ComponentHandle;

fn main() -> Result<(), slint::PlatformError> {
    if std::env::args().any(|arg| arg == "--version") {
        println!("skim-search {} {}", env!("SKIM_SEARCH_BUILD_VERSION"), env!("SKIM_SEARCH_BUILD_SHA"));
        return Ok(());
    }
    if std::env::args().any(|arg| arg == "--version") {
        println!("skim-search {} {}", env!("SKIM_SEARCH_BUILD_VERSION"), env!("SKIM_SEARCH_BUILD_SHA"));
        return Ok(());
    }
    log::write(
        "app",
        &format!("start pid={} log={} version={} sha={}", std::process::id(), log::log_path().display(),
                 env!("SKIM_SEARCH_BUILD_VERSION"), env!("SKIM_SEARCH_BUILD_SHA")),
    );

    let window = MainWindow::new()?;
    log::write(
        "view",
        &format!("main window created components={}", skim_search::MAIN_UI_COMPONENTS.join(",")),
    );

    // 1ms timer resolution: the 20ms coalescing timer and the 1ms engine poll would otherwise
    // wake on the default 15.6ms tick (first-result latency, AC-103). Process-wide while running.
    // SAFETY: plain winmm call; paired with timeEndPeriod at exit.
    let period = unsafe { windows::Win32::Media::timeBeginPeriod(1) };
    log::write("app", &format!("timeBeginPeriod(1) result={}", period));

    app::init(&window, app::detect_tools());
    app::register_hotkey();

    // Esc hides the window (handover §22); the event loop keeps running for the hotkey.
    // Closing the window (X) quits.
    window.window().on_close_requested(|| {
        log::write("app", "close requested");
        let _ = slint::quit_event_loop();
        slint::CloseRequestResponse::HideWindow
    });
    window.show()?;
    let result = slint::run_event_loop_until_quit();
    // SAFETY: matches the timeBeginPeriod(1) above.
    unsafe {
        let _ = windows::Win32::Media::timeEndPeriod(1);
    }
    match &result {
        Ok(()) => log::write("app", "exit ok"),
        Err(e) => log::write("app", &format!("exit error={e}")),
    }
    result
}
