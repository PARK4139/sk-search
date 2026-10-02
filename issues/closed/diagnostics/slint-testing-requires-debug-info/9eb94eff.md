# title
Slint tests find 0 elements by accessible-label

# pre-condition
- `cores/app/build.rs` compiles `.slint` without debug info
- `cores/tests/tests/view_main_ui.rs` uses `ElementHandle::find_by_accessible_label`

# steps
Source: rules/implementation.md#tests / Origin: view/main-ui-components-missing/521ce549
1. `cargo test`

# actual result
- Occurred: all 10 labels `found=0` → test FAIL.
- Cause: the i-slint-backend-testing ElementHandle API needs compiler debug info (`MISSING_DEBUG_INFO_MESSAGE`).
- Action: added `CompilerConfiguration::with_debug_info(true)`.
- Result: PASS — each label `found=1`, test passes (`[test]` entries in `ref/actual/logs/skim-search.log`).

# expected result
- UI integration tests can find elements by accessible-label.

# label
SQA_sk_0_0_0

# environment
OS: windows 10 pro
hostname: TBD

# assignee
