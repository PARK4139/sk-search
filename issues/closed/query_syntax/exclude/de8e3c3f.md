# title
! exclude condition does not work

# pre-condition
- skim-search.exe running
- Common test workspace (rules/issue.md#common-test-workspace)

# steps
Source: FR-113, AC-105, handover §7, §8 / showreel 00:07~00:08
1. Type query `login !test` (no Enter)

# actual result
- PASS (real rg + sk): login !test, and `!login` alone → only lines without login (`cores/tests/tests/search_engine_pipeline.rs` ac105_syntax_cases).
- Log: `[search_engine] generation=N raw=... skim=... scope=... rg_pattern=... rg_globs=...`, `[test] pipeline query=...`
- Automated tests: `ref/actual/logs/cargo-test.log` (32 passed), clippy 0 (`ref/actual/logs/cargo-clippy.log`)

# expected result
- Lines containing `login` but containing `test` are excluded. `!login` alone shows only lines without `login`
- Evidence: parsed skim query / scope filters / result count in `ref/actual/logs/skim-search.log`

# label
SQA_sk_0_0_0

# environment
OS: windows 10 pro
hostname: TBD

# assignee
