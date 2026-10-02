# title
Python e2e rerun stops after 3 checks with a UIA COM error (0x80040201)

# pre-condition
- Rerun of skim_tests.e2e_ui right after the PowerShell/Python comparison (user using the PC)

# steps
Source: rules/environment.md#test-scripts / Origin: diagnostics/python-test-script-migration
1. `uv run python -m skim_tests.e2e_ui --no-build`

# actual result
- Occurred: `COMError(-2147220991, '이벤트에서 가입자를 불러낼 수 없습니다.')` (Windows message: an event subscriber could not be invoked) → `aborted`, exit 1.
- Cause: Assumption — a transient COM error during UIA event delivery (the two runs of the same script right before had 21 PASS). Investigation stopped at the user's request.
- Action: recorded only. Full rerun verification in `diagnostics/test-cmd-full-run-after-python-migration` (backlog).
- Result: Unresolved — reproduction not confirmed.

# expected result
- e2e_ui runs to the end without transient UIA errors (or recovers by retrying).

# label
SQA_sk_0_0_0

# environment
OS: windows 10 pro
hostname: TBD

# assignee
