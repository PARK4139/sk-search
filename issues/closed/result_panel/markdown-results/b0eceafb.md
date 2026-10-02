# title
Markdown file content search results are not shown

# pre-condition
- Common test workspace (rules/issue.md#common-test-workspace)

# steps
Source: FR-119, AC-106, handover §13 / showreel 00:12~00:14
1. Type query `TODO ext:md`
2. Select the `docs/login-guide.md` result

# actual result
- PASS: `TODO ext:md` → only .md files (README.md, docs/install.md, docs/login-guide.md, docs/sample file.md), md source in the preview.
- Evidence: search_engine_pipeline.rs markdown_todo, `ref/actual/screenshot/frames/e2e_md_edit.png`

# expected result
- Only `.md` files in the results, each TODO line shown
- The preview shows the content of that md file
- Evidence: result count in `ref/actual/logs/skim-search.log`, screenshot

# label
SQA_sk_0_0_0

# environment
OS: windows 10 pro
hostname: TBD

# assignee
