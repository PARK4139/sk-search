# title
e2e_ui Ctrl+S 검증이 첫 md 결과를 README.md로 가정해 간헐 실패

# pre-condition
- `skim_tests.e2e_ui` E 단계: `TODO ext:md` 검색 후 첫 결과(프리뷰)를 편집하고 Ctrl+S

# steps
근거: AC-109 저장 (handover §18) / 발생: diagnostics/screenshot-local-user-path
1. `uv run --locked --project cores/tests/py python -m skim_tests.e2e_ui`

# actual result
- 발생: `CHECK FAIL [preview_editor/ctrl-s-save] keyboard Ctrl+S wrote README.md` (2026-10-02 02:02, 20 PASS / 1 FAIL).
- 원인: 첫 md 결과는 rg 출력 순서에 따라 바뀜(이번엔 `docs/sample file.md`). 저장은 정상(`[preview_editor] saved path=...\docs\sample file.md`)이나 검증이 `README.md`를 고정으로 읽음.
- 조치: 앱 로그의 `saved path=`에서 저장 대상 경로를 읽어 그 파일 내용을 검증.
- 결과: PASS — 재실행 21/21 (`ctrl-s-save ... wrote sample file.md`).

# expected result
- Ctrl+S 검증은 실제로 저장된 파일을 대상으로 한다.

# label
SQA_sk_0_0_0

# environment
OS: Windows 11 Pro
hostname: TBD

# 담당자
