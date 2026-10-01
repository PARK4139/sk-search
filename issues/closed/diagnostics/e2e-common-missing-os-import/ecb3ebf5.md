# title
skim_tests.common 의 os import 누락으로 e2e_detection 중단

# pre-condition
- d695d3e: 경로 정의를 `skim_tests/paths.py`로 이동하면서 `common.py`의 `import os` 제거
- one-shot run 20261002-011655-99d833e8

# steps
근거: rules/environment.md#검증-스크립트 / 발생: diagnostics/one-shot-pipeline
1. one-shot CI → `skim_tests.e2e_detection --no-build`

# actual result
- 발생: `CHECK FAIL [e2e_detection] aborted: NameError("name 'os' is not defined")` → CI 중단, push 없음.
- 원인: `common.py` `App` 실행부(`env={**os.environ, ...}`)가 여전히 `os` 사용. 이전 one-shot 실행은 `cargo test` 단계에서 먼저 실패해 이 경로에 도달하지 않음.
- 조치: `common.py`에 `import os` 복구. skim_tests 전 모듈 AST 검사로 다른 미정의 이름 없음 확인.
- 결과: one-shot 재실행으로 검증 (diagnostics/one-shot-security-gate 기록).

# expected result
- e2e 모듈이 import·실행 단계에서 NameError 없이 동작한다.

# label
SQA_sk_0_0_0

# environment
OS: Windows 11 Pro
hostname: TBD

# 담당자
