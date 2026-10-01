# title
존재하지 않는 경로 / 파일 경로 입력 시 검색 차단 부재

# pre-condition
- skim-search.exe 실행
- 공통 테스트 workspace (rules/issue.md#공통-테스트-workspace)

# steps
근거: FR-103, handover §5
1. 검색 경로에 `D:\Projects\not-exist` 입력, Query `login`
2. 검색 경로에 `D:\Projects\my-project\README.md` (file) 입력, Query `login`
3. 검색 경로에 `D:\Projects\my-project` 입력, Query `login`

# actual result
미구현

# expected result
- 1, 2: rg/sk 미실행, 결과 없음, 경로 오류 표시 (warning Toast 또는 입력란 오류 표시)
- 3: 정상 검색
- 검증 증거: `ref/actual/logs/skim-search.log`에 1, 2의 경로 거부 사유와 rg spawn 없음 기록

# label
SQA_sk_0_0_0

# environment
OS: windows 10 pro
hostname: TBD

# 담당자
