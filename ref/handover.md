# skim-search — Claude Handover Prompt

## 역할

너는 `skim-search` 프로젝트를 구현하는 시니어 Rust/Windows 데스크톱 개발자다.

목표는 **Native Windows에서 동작하는 초고속 파일 내용 검색 프로그램**을 구현하는 것이다.

불필요한 리팩토링, 과도한 추상화, 새로운 레이어 추가를 피하고 아래 확정 요구사항을 우선한다.

- 기존 repository/구조가 있으면 최대한 유지한다.
- 최소 수정/최소 계층으로 구현한다.
- 성능, 안정성, 유지보수성, 확장성을 함께 고려한다.
- PoC 수준이 아니라 실제 사용 가능한 v1 품질을 목표로 한다.
- 구현 중 임의로 UX/검색 문법을 변경하지 않는다.
- 아래 내용을 요구사항의 Single Source of Truth로 사용한다.

---

# 1. 프로젝트 기본 정보

```text
프로젝트명 : skim-search
실행파일   : skim-search.exe
플랫폼     : Native Windows
언어       : Rust
UI         : Slint
검색       : rg.exe + sk.exe
```

VSCode Extension이 아니라 **독립 실행 프로그램**이다.

---

# 2. 핵심 사용자 Flow

```text
Ctrl + Shift + F
        ↓
skim-search 표시 / 포커스
        ↓
Search Query 자동 focus
        ↓
사용자 입력 변경 즉시 감지
        ↓
기존 검색 즉시 cancel
        ↓
20ms coalescing
        ↓
Query parse
        ↓
rg.exe 기반 파일 내용 검색
        ↓
sk.exe 기반 표현식 filtering
        ↓
검색 결과 streaming
        ↓
좌측 Result Panel 전체 갱신
        ↓
결과 선택
        ↓
우측 Preview 표시
        ↓
Preview 직접 편집
        ↓
Ctrl + S 저장
        ↓
우측 하단 Toast
        ↓
파일 열기 / 부모경로 열기
```

---

# 3. 검색 성능 정책

## 3.1 200ms debounce 금지

입력 변경 이벤트는 즉시 감지한다.

```text
Query Change Detect    : 즉시
Previous Search Cancel : 즉시
Query Parse            : 즉시
Coalescing             : 20ms
Search Start Target    : <= 25ms
First Result Target    : <= 50ms
```

`전체 검색 완료 <= 50ms`를 보장한다는 의미는 아니다.

목표는 **검색 시작과 첫 결과 표시의 체감 latency를 최대한 50ms 이하로 유지**하는 것이다.

## 3.2 Latest Query Wins

> 이관됨 (2026-10-01) → issues: `search_engine/stale-result-overwrites-latest`, `diagnostics/latest-query-wins-tests`

## 3.3 검색 취소

> 이관됨 (2026-10-01) → issues: `search_engine/stale-rg-sk-process-accumulation`

---

# 4. 검색 버튼 제거

> 이관됨 (2026-10-01) → issues: `view/main-ui-components-missing`, `search_engine/auto-search-on-input`

---

# 5. 검색 경로

> 이관됨 (2026-10-01) → issues: `view/search-root-validation-missing`, `view/include-subfolders-checkbox-conflict`, `view/folder-picker-missing`, `view/rerun-on-search-root-change-missing`, `settings/recent-search-root-persistence`, `view/main-ui-components-missing`

---

# 6. Query가 Single Source of Truth

> 이관됨 (2026-10-01) → issues: `view/main-ui-components-missing`

---

# 7. 검색 문법

> 이관됨 (2026-10-01) → issues: `query_syntax/space-and`, `query_syntax/exact`, `query_syntax/exclude`, `query_syntax/or`, `query_syntax/combined-query`

---

# 8. skim-search 전용 Scope 문법

> 이관됨 (2026-10-01) → issues: `query_syntax/path-include`, `query_syntax/path-exclude`, `query_syntax/ext-include`, `query_syntax/ext-exclude`, `query_syntax/file-token`

---

# 9. Query Parse 규칙

> 이관됨 (2026-10-01) → issues: `query_syntax/scope-token-separation`

---

# 10. Filtering 정확성

> 이관됨 (2026-10-01) → issues: `query_syntax/content-filter-path-contamination`

---

# 11. rg.exe 책임

`rg.exe`는 실제 파일 내용 탐색 담당.

필요 데이터:

```text
file path
line
column
matched text / line text
```

권장 목적 옵션:

```text
--line-number
--column
--no-heading
--color never
```

필요하면 `--json` 사용 가능.

단, parsing 안정성과 streaming 구현이 더 단순하고 안전한 방식을 선택한다.

---

# 12. sk.exe 책임

> 이관됨 (2026-10-01) → issues: `search_engine/rg-sk-executable-detection`, `toast/search-error-toast`

---

# 13. 검색 대상 파일

코드 파일에 한정하지 않는다.

예:

```text
*.rs
*.ts
*.tsx
*.js
*.jsx
*.py
*.java
*.cs
*.cpp
*.h
*.json
*.yaml
*.yml
*.toml
*.md
*.txt
```

Markdown은 정식 지원 대상이다.

예:

```text
TODO ext:md
```

결과 예:

```text
docs/login-guide.md
README.md
docs/architecture.md
docs/configuration.md
CHANGELOG.md
```

---

# 14. Result Panel

> 이관됨 (2026-10-01) → issues: `result_panel/group-by-file`, `result_panel/file-type-badge`, `status_bar/result-stats-not-updated`

---

# 15. Result Panel 전체 갱신

> 이관됨 (2026-10-01) → issues: `result_panel/full-panel-refresh`

---

# 16. Streaming

> 이관됨 (2026-10-01) → issues: `search_engine/result-streaming`

---

# 17. Preview

> 이관됨 (2026-10-01) → issues: `preview_editor/preview-content-mismatch`, `preview_editor/edit-text-overlap`, `preview_editor/match-scroll-highlight`, `preview_editor/duplicate-line-numbers`, `preview_editor/breadcrumb-not-updated`

---

# 18. Preview 편집 / 저장

> 이관됨 (2026-10-01) → issues: `preview_editor/ctrl-s-save`

---

# 19. 외부 변경 충돌 방지

> 이관됨 (2026-10-01) → issues: `preview_editor/external-change-conflict`

---

# 20. 파일 열기

> 이관됨 (2026-10-01) → issues: `external_open/open-file-goto-line-column`, `settings/open-editor-selection`

---

# 21. 부모경로 열기

> 이관됨 (2026-10-01) → issues: `external_open/open-parent-select-file`

---

# 22. 키보드 UX

> 이관됨 (2026-10-01) → issues: `shortcut/global-hotkey-ctrl-shift-f`, `shortcut/arrow-key-navigation`, `shortcut/enter-ctrl-enter-actions`, `shortcut/esc-hide`, `preview_editor/ctrl-s-save`, `search_engine/auto-search-on-input`

---

# 23. Toast

> 이관됨 (2026-10-01) → issues: `toast/exceeds-max-three`, `toast/horizontal-overlap`, `toast/auto-dismiss`, `toast/type-distinction`, `toast/search-error-toast`

---

# 24. 최종 UI

> 이관됨 (2026-10-01) → issues: `view/main-ui-components-missing`

---

# 25. 권장 프로젝트 구조

```text
skim-search/
├─ Cargo.toml
├─ build.rs
├─ src/
│  ├─ main.rs
│  ├─ app.rs
│  ├─ settings.rs
│  ├─ hotkey.rs
│  ├─ search/
│  │  ├─ mod.rs
│  │  ├─ query.rs
│  │  ├─ engine.rs
│  │  ├─ ripgrep.rs
│  │  └─ skim.rs
│  └─ editor/
│     └─ mod.rs
├─ ui/
│  └─ main.slint
└─ assets/
```

현재 단계에서 임의로 추가하지 말 것:

```text
manager/
repository/
service/
adapter/
domain/
```

필요성이 명확해질 때만 추가.

---

# 26. 의존성 방향

필요 최소 범위에서 선택.

예:

```text
slint
tokio
serde
serde_json
anyhow
thiserror
directories
windows
```

대형 framework 추가 금지.
Electron/Tauri 전환 금지.

---

# 27. FR 목록

```text
FR-100  Native Windows 실행
FR-101  Rust + Slint UI
FR-102  Ctrl+Shift+F Global Hotkey
FR-103  검색 Root 입력
FR-104  Folder Picker
FR-105  Query Change 즉시 감지
FR-106  20ms Coalescing
FR-107  이전 검색 즉시 Cancel
FR-108  Latest Query Wins
FR-109  rg.exe 내용 검색
FR-110  sk.exe 표현식 filtering
FR-111  공백 AND
FR-112  ' exact
FR-113  ! exclude
FR-114  | OR
FR-115  path:
FR-116  !path:
FR-117  ext:
FR-118  !ext:
FR-119  Markdown 내용 검색
FR-120  결과 Streaming
FR-121  파일별 Result Group
FR-122  Result Panel 전체 갱신
FR-123  Preview
FR-124  Preview 직접 편집
FR-125  Ctrl+S 저장
FR-126  외부 변경 충돌 감지
FR-127  파일 열기
FR-128  부모경로 열기
FR-129  Toast Stack
FR-130  Esc 숨김
FR-131  설정 저장
FR-132  rg/sk executable detection
FR-133  검색 오류 Toast
```

---

# 28. Acceptance Criteria

## AC-100 실행
- Windows에서 `skim-search.exe` 실행
- Slint UI 정상 표시
- crash 없이 종료

## AC-101 Global Hotkey
- 다른 프로그램 활성화 상태에서 `Ctrl+Shift+F`
- skim-search foreground 표시
- Query input focus

## AC-102 자동 검색
`login` 입력 후 검색 버튼/Enter 없이 자동 검색.

## AC-103 검색 반응
- Query change 즉시 감지
- 20ms coalescing
- 검색 시작 목표 <= 25ms
- 정상 로컬 SSD workspace에서 first-result target <= 50ms
- 측정값을 status/debug log에서 확인 가능
- 목표 미달 시 실제 측정값을 숨기지 말고 보고

## AC-104 Latest Query Wins
빠르게:

```text
l
lo
log
login
```

입력했을 때 최종 UI는 반드시 `login` 결과.

이전 query 결과가 화면을 덮으면 FAIL.

## AC-105 Query 문법
아래 각각 독립 동작:

```text
login
login error
'login
!login
login !test
login | logout
path:src
!path:test
ext:ts
!ext:json
TODO ext:md
login !test path:src ext:ts
```

## AC-106 Markdown

```text
TODO ext:md
```

검색 시 `.md` 파일 내용이 Result/Preview에 표시.

## AC-107 Result Panel
새 Query마다 좌측 panel 전체가 최신 검색 결과로 변경.
첫 파일 하나만 변경되면 FAIL.

## AC-108 Preview
선택한 결과의 정확한 파일/위치 주변 내용 표시.

## AC-109 Edit / Save
Preview 수정 후 `Ctrl+S`로 실제 파일 저장.
성공 Toast 표시.

## AC-110 Conflict
Preview open 이후 외부 변경 시 저장 단계에서 감지.
자동 overwrite 금지.

## AC-111 File Open
`Enter` 또는 `[파일 열기]`로 설정 editor/system에서 파일 열기.
지원 editor는 line/column 이동.

## AC-112 Parent Open
`Ctrl+Enter` 또는 `[부모경로 열기]`로 Explorer를 열고 현재 파일 select.

## AC-113 Toast
최대 3개 우측 하단 stack.
새 Toast 아래, 기존 Toast 위.

## AC-114 Cancel
연속 입력 시 이전 rg/sk 프로세스 누적 금지.

---

# 29. 테스트 요구사항

> 이관됨 (2026-10-01) → issues: `diagnostics/query-parser-tests`, `diagnostics/latest-query-wins-tests`, `search_engine/windows-path-with-spaces-parsing`, `diagnostics/save-conflict-tests`

---

# 30. 로그 / 진단

> 이관됨 (2026-10-01) → issues: `diagnostics/debug-log-fields`

---

# 31. 구현 순서

```text
Phase 1
- Cargo project
- Slint base UI
- Search Root
- Query input
- Result model

Phase 2
- Query parser
- rg integration
- sk integration
- result grouping

Phase 3
- auto search
- 20ms coalescing
- cancellation
- Latest Query Wins
- streaming

Phase 4
- Preview
- edit/save
- conflict detection

Phase 5
- Global hotkey
- file open
- parent path open
- Toast stack
- settings persistence

Phase 6
- tests
- performance measurement
- release build
```

Phase는 구현 순서일 뿐 별도 architecture를 만들라는 의미가 아니다.

---

# 32. 완료 보고 형식

```text
1. 구현 완료 항목
2. 변경 파일 목록
3. 주요 구조
4. 검색 문법 지원 현황
5. 테스트 결과
6. cargo build 결과
7. cargo test 결과
8. clippy 결과
9. 실제 검색 latency 측정값
10. 알려진 제한사항
11. 실행 방법
```

성공하지 않은 항목을 PASS처럼 표현하지 않는다.

---

# 33. 금지사항

```text
- Electron/Tauri 전환
- 자체 fuzzy search 엔진 구현
- 검색 버튼 다시 추가
- Chip 기반 mode toggle 추가
- VSCode 전용 프로그램으로 변경
- Preview read-only로 축소
- Markdown 검색 제거
- 200ms 이상 debounce 적용
- 이전 검색 결과가 최신 UI를 덮어쓰게 방치
- 검색 결과 첫 그룹만 부분 갱신
- manager/repository/service 등 불필요한 계층 추가
- SQLite/DB 추가
- Agent/LLM/MCP 기능 추가
```

---

# 34. 최종 핵심 규칙

```text
1. Native Windows + Rust + Slint
2. Query가 Single Source of Truth
3. 검색 버튼 없음
4. Query change → 20ms coalescing → 자동 검색
5. 즉시 cancel + Latest Query Wins
6. rg = filesystem/content search, sk = Skim expression filtering
7. 코드 + Markdown 동일 검색/Preview/편집
8. Result → Preview 편집/저장 → 파일 열기/부모경로 열기
```

---

# 최종 요청

위 요구사항을 기준으로 `skim-search`를 구현하라.

먼저 기존 repository가 있으면 구조와 코드를 확인하고 **기존 구조를 최대한 유지한 최소 변경 방식**으로 구현하라.

repository가 비어 있거나 신규 프로젝트라면 위 권장 구조를 기준으로 시작하라.

작업 도중 사소한 선택사항 때문에 사용자에게 반복 확인하지 말고, 위 원칙에 맞는 가장 단순하고 유지보수 가능한 방향으로 판단해서 진행하라.

단, 요구사항을 변경하거나 기능을 제거해야 하는 명확한 기술적 제약이 발견되면 임의 변경하지 말고 그 제약과 최소 대안을 명확히 보고하라.
