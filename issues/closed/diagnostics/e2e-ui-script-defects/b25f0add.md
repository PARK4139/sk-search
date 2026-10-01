# title
e2e_ui.ps1 결함 (BOM 없는 한글, 이전 로그 매칭, Text 값 읽기, 폴더 대화상자, cargo stderr)

# pre-condition
- `cores/tests/scripts/e2e_ui.ps1` (UI Automation 기반 실제 앱 e2e)

# steps
근거: rules/environment.md#검증-스크립트 / 발생: search_engine/auto-search-on-input 외 1차 묶음
1. PowerShell 5.1에서 스크립트 실행

# actual result
- 발생: (1) UTF-8(BOM 없음) 한글 패턴이 ANSI로 읽혀 정규식 오류 (2) 완료 로그를 이전 실행 줄과 매칭 (3) Text 요소 값 대신 이름(accessible-label) 반환 (4) 폴더 대화상자 컨트롤이 UIA에 Pane으로만 노출 (5) cargo stderr 진행 출력이 PowerShell 오류로 처리되어 중단.
- 조치: (1) BOM 저장 (2) 실행 시점 이후 로그만 검사 (MarkLog) (3) 앱에 accessible-value 추가 (4) Win32 GetDlgItem/WM_SETTEXT/BM_CLICK (5) `cmd /c "cargo ... 2>&1"`.
- 결과: PASS — 전체 시나리오 실행 완료.
- 재발 방지: rules/environment.md#스크립트-작성-주의 에 BOM 규칙 추가.

# expected result
- e2e 스크립트가 저장소 밖 cwd에서 전체 시나리오를 끝까지 실행한다.

# label
SQA_sk_0_0_0

# environment
OS: windows 10 pro
hostname: TBD

# 담당자
