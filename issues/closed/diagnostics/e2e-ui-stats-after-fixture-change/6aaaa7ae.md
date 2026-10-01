# title
공용 fixture 추가 파일로 e2e_ui 상태바 결과 수 기대값(14/7)이 어긋남

# pre-condition
- d695d3e: e2e workspace를 공용 fixture(`cores/tests/fixtures/sample/tree`) 복사로 변경, fixture에 `src/fuzzy.ts` 포함
- one-shot run 20261002-013226-a0a43e35

# steps
근거: AC-100 상태바 결과 통계 (handover §24) / 발생: diagnostics/one-shot-pipeline
1. one-shot CI → `skim_tests.e2e_ui --no-build` (query `login`)

# actual result
- 발생: `CHECK FAIL [status_bar/result-stats-not-updated]` — 실제 `15 결과 · 8 파일`, 기대 `14 결과 · 7 파일`. 나머지 20개 PASS.
- 원인: `src/fuzzy.ts`(`let lo = g; if (n) {}`)가 sk 퍼지 매칭으로 `login`(l-o-g-i-n)에 일치 → 1건·1파일 증가. 기대값은 이전 workspace 기준.
- 조치: 기대값을 `15 결과 · 8 파일` / `15 results`로 갱신하고 근거 주석 추가.
- 결과: one-shot 재실행으로 검증 (diagnostics/one-shot-security-gate 기록).
- 참고: 이 실행에서 새로 생성된 e2e 캡처는 로컬 사용자 경로가 표시되어(diagnostics/screenshot-local-user-path/a3073577) commit하지 않음.

# expected result
- 상태바 결과 수 검증이 현재 fixture 기준과 일치한다.

# label
SQA_sk_0_0_0

# environment
OS: Windows 11 Pro
hostname: TBD

# 담당자
