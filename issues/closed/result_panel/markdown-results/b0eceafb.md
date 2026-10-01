# title
Markdown 파일 내용 검색 결과 미표시

# pre-condition
- 공통 테스트 workspace (rules/issue.md#공통-테스트-workspace)

# steps
근거: FR-119, AC-106, handover §13 / showreel 00:12~00:14
1. Query `TODO ext:md` 입력
2. `docs/login-guide.md` 결과 선택

# actual result
- PASS: `TODO ext:md` → .md 파일만 (README.md, docs/install.md, docs/login-guide.md, docs/sample file.md), 프리뷰에 md 원문.
- 증거: search_engine_pipeline.rs markdown_todo, `ref/actual/screenshot/frames/e2e_md_edit.png`

# expected result
- `.md` 파일만 결과에 표시, 각 TODO 줄 표시
- Preview에 해당 md 파일 내용 표시
- 검증 증거: `ref/actual/logs/skim-search.log` result count, 스크린샷

# label
SQA_sk_0_0_0

# environment
OS: windows 10 pro
hostname: TBD

# 담당자
