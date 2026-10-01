# title
Toast info/success/warning/error 시각 구분 부재

# pre-condition
- skim-search.exe 실행

# steps
근거: handover §23 / showreel 00:15~00:16
1. 검색 완료(info), 저장 완료(success), 외부 변경 충돌(warning), rg 미탐지(error) 발생

# actual result
- PASS: 종류별 테두리·막대 색 — info 파랑, success 초록, warning 노랑, error 빨강.
- 증거: `ref/actual/screenshot/frames/e2e_toasts.png` (info+warning), `ref/actual/screenshot/frames/e2e_toast_success.png` (success), `ref/actual/screenshot/frames/e2e_toast_error.png` (error)

# expected result
- 유형별 아이콘/색상 구분
- 검증 증거: 스크린샷 `ref/actual/screenshot/frames/type-distinction_300.png`

# label
SQA_sk_0_0_0

# environment
OS: windows 10 pro
hostname: TBD

# 담당자
