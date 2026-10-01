# title
Esc 로 skim-search 숨김 미동작

# pre-condition
- skim-search.exe 표시 상태

# steps
근거: FR-130, handover §22
1. Esc
2. Ctrl+Shift+F

# actual result
미구현

# expected result
- 1: 창 숨김 (프로세스 유지)
- 2: 이전 Query/결과 상태로 재표시
- 검증 증거: `ref/actual/logs/skim-search.log`에 hide/show 기록

# label
SQA_sk_0_0_0

# environment
OS: windows 10 pro
hostname: TBD

# 담당자
