//! issue: view/main-ui-components-missing/521ce549

use i_slint_backend_testing::{AccessibleRole, ElementHandle, ElementRoot};
use skim_search::{MainWindow, MAIN_UI_COMPONENTS, TOAST_AREA};

#[test]
fn main_ui_components_exist_and_forbidden_absent() {
    i_slint_backend_testing::init_no_event_loop();
    let window = MainWindow::new().unwrap();

    // 9 required components + toast area, exactly once each.
    let mut missing = Vec::new();
    for label in MAIN_UI_COMPONENTS.iter().chain([&TOAST_AREA]) {
        let n = ElementHandle::find_by_accessible_label(&window, label).count();
        common::log::write("test", &format!("view_main_ui component={label} found={n}"));
        if n != 1 {
            missing.push(*label);
        }
    }
    assert!(
        missing.is_empty(),
        "missing or duplicated components: {missing:?}"
    );

    // Only the 3 allowed buttons exist: no search button, no chips, no "VS Code에서 열기"
    // (handover §4, §6, §20, §33).
    let allowed = ["browse-button", "open-file-button", "open-parent-button"];
    let buttons: Vec<String> = window
        .root_element()
        .query_descendants()
        .match_accessible_role(AccessibleRole::Button)
        .find_all()
        .into_iter()
        .map(|b| {
            b.accessible_label()
                .map(|s| s.to_string())
                .unwrap_or_default()
        })
        .collect();
    common::log::write("test", &format!("view_main_ui buttons={buttons:?}"));
    let unexpected: Vec<_> = buttons
        .iter()
        .filter(|b| !allowed.contains(&b.as_str()))
        .collect();
    assert!(unexpected.is_empty(), "unexpected buttons: {unexpected:?}");
    assert_eq!(buttons.len(), allowed.len(), "button count");
}
