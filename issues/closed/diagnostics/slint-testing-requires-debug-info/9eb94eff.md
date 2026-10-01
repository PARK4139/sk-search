# title
Slint 테스트에서 accessible-label 요소가 0개로 검색됨

# pre-condition
- `cores/app/build.rs` 가 debug info 없이 `.slint` compile
- `cores/tests/tests/view_main_ui.rs` 가 `ElementHandle::find_by_accessible_label` 사용

# steps
근거: rules/implementation.md#테스트 / 발생: view/main-ui-components-missing/521ce549
1. `cargo test`

# actual result
- 발생: 10개 label 모두 `found=0` → 테스트 FAIL.
- 원인: i-slint-backend-testing ElementHandle API는 compiler debug info 필요 (`MISSING_DEBUG_INFO_MESSAGE`).
- 조치: `CompilerConfiguration::with_debug_info(true)` 추가.
- 결과: PASS — 각 label `found=1`, 테스트 통과 (`ref/actual/logs/skim-search.log` `[test]` 항목).

# expected result
- UI 통합 테스트가 accessible-label로 요소를 찾을 수 있다.

# label
SQA_sk_0_0_0

# environment
OS: windows 10 pro
hostname: TBD

# 담당자
