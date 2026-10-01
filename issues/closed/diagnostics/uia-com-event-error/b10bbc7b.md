# title
Python e2e 재실행이 UIA COM 오류(0x80040201)로 판정 3개 후 중단

# pre-condition
- PowerShell/Python 비교 직후 skim_tests.e2e_ui 재실행 (사용자 PC 사용 중)

# steps
근거: rules/environment.md#검증-스크립트 / 발생: diagnostics/python-test-script-migration
1. `uv run python -m skim_tests.e2e_ui --no-build`

# actual result
- 발생: `COMError(-2147220991, '이벤트에서 가입자를 불러낼 수 없습니다.')` → `aborted`, exit 1.
- 원인: 추정 — UIA 이벤트 전달 중 일시적 COM 오류 (같은 스크립트 직전 2회는 21 PASS). 재현 조사는 사용자 요청으로 중단.
- 조치: 기록만 남김. 전체 재실행 검증은 `diagnostics/test-cmd-full-run-after-python-migration` (backlog) 에서 확인.
- 결과: 미해결 — 재현 여부 미확인.

# expected result
- e2e_ui 가 일시적 UIA 오류 없이 끝까지 실행된다 (또는 재시도로 복구된다).

# label
SQA_sk_0_0_0

# environment
OS: windows 10 pro
hostname: TBD

# 담당자
