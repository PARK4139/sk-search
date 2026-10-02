# title
No visual distinction between info/success/warning/error toasts

# pre-condition
- skim-search.exe running

# steps
Source: handover §23 / showreel 00:15~00:16
1. Trigger search completed (info), save completed (success), external change conflict (warning), rg not detected (error)

# actual result
- PASS: border and bar colors per type — info blue, success green, warning yellow, error red.
- Evidence: `ref/actual/screenshot/frames/e2e_toasts.png` (info+warning), `ref/actual/screenshot/frames/e2e_toast_success.png` (success), `ref/actual/screenshot/frames/e2e_toast_error.png` (error)

# expected result
- Icons/colors distinguish the types
- Evidence: screenshot `ref/actual/screenshot/frames/type-distinction_300.png`

# label
SQA_sk_0_0_0

# environment
OS: windows 10 pro
hostname: TBD

# assignee
