# title
Fix skim_input_from_rg splitting lines containing ':' wrongly (or remove the unused function)

# pre-condition
- `cores/app/src/query_syntax/mod.rs` `skim_input_from_rg`, `cores/tests/tests/query_syntax_content_filter.rs`

# steps
Source: handover §11 / Origin: query_syntax/skim-input-from-rg-colon-in-text
1. Convert the rg output `C:\a.ts:12:3:foo: bar` with `skim_input_from_rg`

# actual result
- PASS: `split_rg_record` splits from the left (the drive `C:` stays in the path, ':' in the content is kept). `issue_checks.rs` skim_input_keeps_colons_in_text: `C:\a.ts:12:3:foo: bar` → path/12/3/`foo: bar`.
- The existing `query_syntax_content_filter.rs` tests still pass.

# expected result
- The conversion keeps path=`C:\a.ts`, line=12, column=3, text=`foo: bar`. Or remove the function and have the tests use the engine path (`ripgrep::parse_line`).
- Evidence: `ref/actual/logs/cargo-test.log`

# label
SQA_sk_0_0_0

# environment
OS: windows 10 pro
hostname: TBD

# assignee
