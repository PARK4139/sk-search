# title
프리뷰 현재 줄 강조가 아래 줄로 밀려 표시됨 (81행 선택 → 85행 강조)

# pre-condition
- src/long.ts (163줄), Query `loginTarget` → 81행 선택

# steps
근거: handover §17 (match line highlight) / 발생: search_engine/auto-search-on-input 외 1차 묶음 (e2e 스크린샷)
1. release exe에서 `loginTarget` 검색, 프리뷰 확인

# actual result
- 발생: 선택 영역은 81행인데 강조 막대는 85행에 표시 (`ref/actual/screenshot/frames/e2e_scroll_highlight.png` 1차).
- 원인: 줄 높이를 별도 probe Text의 preferred-height로 계산 → TextInput 실제 줄 간격과 약 5% 차이, 줄 수에 비례해 누적.
- 조치: line-h = editor.preferred-height / preview-line-count (편집기 자체 레이아웃 기준), Rust에서 line-count 설정.
- 결과: PASS — 81행에 강조, 위쪽 1/3로 스크롤 (재캡처).
- 재발 방지: 줄 위치 계산은 해당 TextInput 자체 측정값을 사용.

# expected result
- 강조 막대와 선택 줄이 일치한다.

# label
SQA_sk_0_0_0

# environment
OS: windows 10 pro
hostname: TBD

# 담당자
