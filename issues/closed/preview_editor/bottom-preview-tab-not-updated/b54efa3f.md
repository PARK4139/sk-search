# title
The file:line display of the bottom preview area is fixed

# pre-condition
- Common test workspace (rules/issue.md#common-test-workspace)

# steps
Source: AC-108 / showreel 00:01~00:16
1. Select several queries/results and check the bottom `미리보기` tab

# actual result
- PASS (area not in handover §24 → removal chosen): the bottom preview tab area is not built. The preview header (path + line:col) updates with the selection.
- Evidence: `ref/actual/screenshot/frames/e2e_login.png`, `ref/actual/screenshot/frames/e2e_scroll_highlight.png`

# expected result
- Updates to match the selected result's file:line (or, since the area is not in handover §24, remove it — decided together with the issue confirming the bottom tab requirement)
- Evidence: screenshot

# label
SQA_sk_0_0_0

# environment
OS: windows 10 pro
hostname: TBD

# assignee
