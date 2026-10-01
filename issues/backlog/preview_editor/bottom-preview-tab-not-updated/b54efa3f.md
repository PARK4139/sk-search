# title
하단 미리보기 영역의 파일:줄 표시가 고정됨

# pre-condition
- 공통 테스트 workspace (rules/issue.md#공통-테스트-workspace)

# steps
근거: AC-108 / showreel 00:01~00:16
1. 여러 Query/결과 선택 후 하단 `미리보기` 탭 확인

# actual result
- 모든 시점에 `src/auth/login.ts:12` 와 해당 코드 고정 표시

# expected result
- 선택 결과의 파일:줄과 일치하게 갱신 (또는 handover §24에 없는 영역이므로 제거 — 하단 탭 요구 확인 issue와 함께 결정)
- 검증 증거: 스크린샷

# label
SQA_sk_0_0_0

# environment
OS: windows 10 pro
hostname: TBD

# 담당자
