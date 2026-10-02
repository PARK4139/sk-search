# title
No search block for a nonexistent path / file path as the search root

# pre-condition
- skim-search.exe running
- Common test workspace (rules/issue.md#common-test-workspace)

# steps
Source: FR-103, handover §5
1. Search root `D:\Projects\not-exist`, query `login`
2. Search root `D:\Projects\my-project\README.md` (a file), query `login`
3. Search root `D:\Projects\my-project`, query `login`

# actual result
- PASS: nonexistent path / file path / relative path / empty → no search, results cleared, stats `검색 경로 오류: ...`, warning toast (once per identical error). rg not run.
- Evidence: issue_checks root_validation, ui_flow, e2e `ref/actual/screenshot/frames/e2e_toasts.png`, log `[view] generation=N root rejected: ...`

# expected result
- 1, 2: rg/sk not run, no results, path error shown (warning toast or an error on the box)
- 3: normal search
- Evidence: rejection reasons for 1 and 2 and no rg spawn in `ref/actual/logs/skim-search.log`

# label
SQA_sk_0_0_0

# environment
OS: windows 10 pro
hostname: TBD

# assignee
