# families — function family 목록

issue 폴더 1단계 이름이자 `cores/app/src/` module 이름이다.
표 순서는 같은 우선순위 issue의 착수 순서 기준으로도 쓴다 (`issue.md#상태-전이와-착수`).

| function family | 의미 | 근거 FR / AC | 구현 위치 (`cores/` 기준) |
|-----------------|------|--------------|---------------------------|
| view            | 조회기능 (메인 화면, 검색 경로) | FR-100, 101, 103, 104 / AC-100, handover §5, §24 | `app/ui/view.slint`, `app/src/app.rs` |
| query_syntax    | 쿼리문법 | FR-111~118 / AC-105 | `app/src/query_syntax/` |
| search_engine   | 검색엔진 | FR-105~110, 120, 132 / AC-102~104, 114 | `app/src/search_engine/` |
| result_panel    | 결과패널 | FR-119, 121, 122 / AC-106, 107 | `app/src/result_panel/`, `app/ui/result_panel.slint` |
| preview_editor  | 프리뷰편집 | FR-123~126 / AC-108~110 | `app/src/preview_editor/`, `app/ui/preview_editor.slint` |
| external_open   | 외부열기 | FR-127, 128 / AC-111, 112 | `app/src/external_open/` |
| shortcut        | 단축키 | FR-102, 130, handover §22 / AC-101 | `app/src/shortcut/` |
| toast           | 토스트 | FR-129, 133 / AC-113 | `app/src/toast/`, `app/ui/toast.slint` |
| status_bar      | 상태표시 | handover §24 status bar, §14 stats | `app/ui/status_bar.slint` |
| settings        | 설정 | FR-131, handover §20 editor 설정 | `common/src/settings.rs` |
| diagnostics     | 진단성능 | handover §30 로그, AC-103 측정, handover §29 테스트, 검증 도구 | `common/src/log.rs`, `tests/`, `tests/py/` |
| build_env       | 빌드·실행 환경 | handover §26 의존성, handover §12 rg/sk 준비, 개발 환경 제약 | `Cargo.toml`, `.cargo/` |
| process         | 작업 절차 | `rules/issue.md` 위반·사고 (제품 코드 무관) | (코드 없음) |

## 규칙

- 신규 family 추가 시 이 표를 먼저 갱신한다.
- 이름: snake_case 영어 (`issue.md#이름과-UUID`).
- 구현 위치는 목표 형태다. 실제 파일은 issue 착수 시 필요한 것만 만든다 (`implementation.md#구조-규칙`).
