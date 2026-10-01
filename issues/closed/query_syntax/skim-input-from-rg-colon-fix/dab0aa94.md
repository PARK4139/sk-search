# title
skim_input_from_rg 의 ':' 포함 줄 오분리 수정 (또는 미사용 함수 제거)

# pre-condition
- `cores/app/src/query_syntax/mod.rs` `skim_input_from_rg`, `cores/tests/tests/query_syntax_content_filter.rs`

# steps
근거: handover §11 / 발생: query_syntax/skim-input-from-rg-colon-in-text
1. rg 출력 `C:\a.ts:12:3:foo: bar` 를 `skim_input_from_rg` 로 변환

# actual result
- PASS: `split_rg_record` 로 왼쪽부터 분리 (드라이브 `C:` 는 경로에 포함, 내용의 ':' 유지). `issue_checks.rs` skim_input_keeps_colons_in_text: `C:\a.ts:12:3:foo: bar` → path/12/3/`foo: bar`.
- 기존 `query_syntax_content_filter.rs` 테스트 유지 통과.

# expected result
- 변환 결과가 path=`C:\a.ts`, line=12, column=3, text=`foo: bar` 를 유지한다. 또는 함수를 제거하고 테스트는 engine 경로(`ripgrep::parse_line`)를 사용한다.
- 검증 증거: `ref/actual/logs/cargo-test.log`

# label
SQA_sk_0_0_0

# environment
OS: windows 10 pro
hostname: TBD

# 담당자
