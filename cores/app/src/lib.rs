slint::include_modules!();

pub mod app;
pub mod external_open;
pub mod preview_editor;
pub mod query_syntax;
pub mod result_panel;
pub mod search_engine;
pub mod shortcut;
pub mod toast;
pub mod view;

/// Names of the required main UI components (handover §24).
/// Each is exposed as an `accessible-label` in `ui/main.slint`.
pub const MAIN_UI_COMPONENTS: [&str; 9] = [
    "search-root-input",
    "browse-button",
    "query-input",
    "syntax-hint",
    "result-panel",
    "preview-editor",
    "open-file-button",
    "open-parent-button",
    "status-bar",
];

/// Toast area is an overlay with no content until a toast is pushed.
pub const TOAST_AREA: &str = "toast-area";
