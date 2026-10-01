# title
bash heredoc 안 Python 문자열의 백슬래시가 이스케이프로 해석되어 치환 실패 반복

# pre-condition
- bash heredoc(`<<'PYEOF'`)으로 Python 치환 스크립트를 실행하며 Windows 경로·`\n` 리터럴을 문자열에 포함

# steps
근거: rules/environment.md#스크립트-작성-주의 / 발생: build_env/paths-ssot, diagnostics/one-shot-failure-issue
1. heredoc Python에서 `"...%LOGS%\bench-sk.out.log..."`, `"\n"` 등을 포함한 치환 실행

# actual result
- 발생: 2026-10-02 세 차례 — `\b`가 백스페이스로 해석돼 assert 실패(test.cmd), 여러 줄 블록 매칭 실패(test_pipeline), 정규식 `\S` 경고. 매번 파일은 변경되지 않음(assert로 중단).
- 원인: heredoc 자체는 그대로 전달하지만 Python 일반 문자열 리터럴이 `\b`, `\p`, `\c` 등을 이스케이프로 처리. 기존 재발 방지(`closed/process/script-edit-backslash-mangled/fd494e04`: Windows 경로 수정은 Edit 도구)를 heredoc Python에는 적용하지 않음.
- 조치: Write 도구로 raw 문자열(`r"..."`) 스크립트 파일을 만들어 실행, 또는 Edit 도구 사용. 모든 치환은 `count == 1` assert로 부분 적용 방지.
- 결과: PASS — 이후 치환 정상 적용.
- 재발 방지: 백슬래시가 들어가는 치환은 heredoc 인라인 Python 대신 Write 도구 파일(raw 문자열) 또는 Edit 도구로 한다 (rules/environment.md 반영).

# expected result
- 치환 스크립트가 원문 그대로의 문자열로 한 번에 적용된다.

# label
SQA_sk_0_0_0

# environment
OS: Windows 11 Pro
hostname: TBD

# 담당자
