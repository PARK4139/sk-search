# title
e2e_detection aborts: missing os import in skim_tests.common

# pre-condition
- d695d3e: moving path definitions to `skim_tests/paths.py` removed `import os` from `common.py`
- one-shot run 20261002-011655-99d833e8

# steps
Source: rules/environment.md#test-scripts / Origin: diagnostics/one-shot-pipeline
1. one-shot CI → `skim_tests.e2e_detection --no-build`

# actual result
- Occurred: `CHECK FAIL [e2e_detection] aborted: NameError("name 'os' is not defined")` → CI stopped, no push.
- Cause: the `App` launcher in `common.py` (`env={**os.environ, ...}`) still uses `os`. The earlier one-shot run failed in `cargo test` first and never reached this path.
- Action: restored `import os` in `common.py`. An AST check of all skim_tests modules found no other undefined names.
- Result: verified by rerunning one-shot (recorded in diagnostics/one-shot-security-gate).

# expected result
- e2e modules import and run without NameError.

# label
SQA_sk_0_0_0

# environment
OS: Windows 11 Pro
hostname: TBD

# assignee
