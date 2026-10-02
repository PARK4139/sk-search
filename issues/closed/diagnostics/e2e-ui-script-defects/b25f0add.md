# title
e2e_ui.ps1 defects (Korean without BOM, matching old log lines, reading Text values, folder dialog, cargo stderr)

# pre-condition
- `cores/tests/scripts/e2e_ui.ps1` (real app e2e with UI Automation)

# steps
Source: rules/environment.md#test-scripts / Origin: first batch incl. search_engine/auto-search-on-input
1. Run the script in PowerShell 5.1

# actual result
- Occurred: (1) UTF-8 (no BOM) Korean patterns read as ANSI caused regex errors (2) completion logs matched lines of earlier runs (3) Text elements returned their name (accessible-label) instead of the value (4) folder dialog controls exposed to UIA only as Pane (5) cargo stderr progress output treated as a PowerShell error, stopping the run.
- Action: (1) save with BOM (2) check only logs written after the run starts (MarkLog) (3) add accessible-value to the app (4) Win32 GetDlgItem/WM_SETTEXT/BM_CLICK (5) `cmd /c "cargo ... 2>&1"`.
- Result: PASS — the full scenario runs to the end.
- Prevention: BOM rule added to rules/environment.md#script-writing-cautions.

# expected result
- The e2e script runs the full scenario to the end from a cwd outside the repository.

# label
SQA_sk_0_0_0

# environment
OS: windows 10 pro
hostname: TBD

# assignee
