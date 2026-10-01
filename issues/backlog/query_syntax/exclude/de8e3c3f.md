# title
! exclude 조건 미동작

# pre-condition
- skim-search.exe 실행
- 공통 테스트 workspace (rules/issue.md#공통-테스트-workspace)

# steps
근거: FR-113, AC-105, handover §7, §8 / showreel 00:07~00:08
1. Query `login !test` 입력 (Enter 없이)

# actual result
미구현

# expected result
- `login` 포함 줄 중 `test` 포함 줄 제외. 단독 `!login`은 `login` 미포함 줄만 표시
- 검증 증거: `ref/actual/logs/skim-search.log`의 parsed skim query / scope filters / result count

# label
SQA_sk_0_0_0

# environment
OS: windows 10 pro
hostname: TBD

# 담당자
