#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use common::log;
use skim_search::{app, MainWindow};
use slint::ComponentHandle;

fn main() -> Result<(), slint::PlatformError> {
    log::write(
        "app",
        &format!("start pid={} log={}", std::process::id(), log::log_path().display()),
    );

    let window = MainWindow::new()?;
    log::write(
        "view",
        &format!("main window created components={}", skim_search::MAIN_UI_COMPONENTS.join(",")),
    );

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
    match &result {
        Ok(()) => log::write("app", "exit ok"),
        Err(e) => log::write("app", &format!("exit error={e}")),
    }
    result
}
