# title
Esc does not hide skim-search

# pre-condition
- skim-search.exe visible

# steps
Source: FR-130, handover §22
1. Esc
2. Ctrl+Shift+F

# actual result
- PASS: in the real app Esc → window hidden (process and event loop kept), Ctrl+Shift+F → shown again with the previous query (`auth`) and results.
- Evidence: e2e log `after Esc visible=False`, `after hotkey visible=True ... query='auth'`, `ref/actual/screenshot/frames/e2e_after_hotkey.png`

# expected result
- 1: window hidden (process kept)
- 2: shown again with the previous query/results
- Evidence: hide/show records in `ref/actual/logs/skim-search.log`

# label
SQA_sk_0_0_0

# environment
OS: windows 10 pro
hostname: TBD

# assignee
