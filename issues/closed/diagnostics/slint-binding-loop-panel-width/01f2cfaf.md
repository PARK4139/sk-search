# title
패널 폭을 parent.width 비율로 지정하자 Slint binding loop 경고

# pre-condition
- 결과/프리뷰 패널에 `width: (parent.width - 8px) * 0.45`

# steps
근거: rules/implementation.md#UI / 발생: view/panel-width-shifts-with-content 조치 중
1. `cargo build`

# actual result
- 발생: `binding loop (root.layoutinfo-h -> keys.width -> ...)` 경고, "runtime panic 가능" 표시.
- 원인: 레이아웃 자식이 부모 폭을 참조 → 레이아웃 정보 계산과 순환.
- 조치: preferred-width 0 + horizontal-stretch 45:55 로 변경.
- 결과: PASS — 경고 0.

# expected result
- 레이아웃 binding loop 경고가 없다.

# label
SQA_sk_0_0_0

# environment
OS: windows 10 pro
hostname: TBD

# 담당자
