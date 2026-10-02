# title
An extra file in the shared fixture breaks the e2e_ui status bar count expectation (14/7)

# pre-condition
- d695d3e: the e2e workspace became a copy of the shared fixture (`cores/tests/fixtures/sample/tree`), which includes `src/fuzzy.ts`
- one-shot run 20261002-013226-a0a43e35

# steps
Source: AC-100 status bar result stats (handover §24) / Origin: diagnostics/one-shot-pipeline
1. one-shot CI → `skim_tests.e2e_ui --no-build` (query `login`)

# actual result
- Occurred: `CHECK FAIL [status_bar/result-stats-not-updated]` — actual `15 결과 · 8 파일`, expected `14 결과 · 7 파일`. The other 20 PASS.
- Cause: `src/fuzzy.ts` (`let lo = g; if (n) {}`) matches `login` (l-o-g-i-n) by sk fuzzy matching → one more result and file. The expectation was for the old workspace.
- Action: expectation updated to `15 결과 · 8 파일` / `15 results` with a comment giving the reason.
- Result: verified by rerunning one-shot (recorded in diagnostics/one-shot-security-gate).
- Note: e2e captures newly made in this run showed a local user path (diagnostics/screenshot-local-user-path/a3073577) and were not committed.

# expected result
- The status bar count check matches the current fixture.

# label
SQA_sk_0_0_0

# environment
OS: Windows 11 Pro
hostname: TBD

# assignee
