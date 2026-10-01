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

예:

```text
generation 101 → log
generation 102 → logi
generation 103 → login
```

현재 generation이 `103`이면 101/102 결과는 늦게 도착하더라도 반드시 폐기한다.

권장:

```rust
current_generation: AtomicU64
```

동일 목적의 더 단순하고 안전한 구현도 허용한다.

핵심 규칙:

```text
old result must never overwrite latest query result
```

## 3.3 검색 취소

Query가 변경되면 현재 검색을 즉시 취소한다.

```text
old rg.exe → terminate
old sk.exe → terminate
old async task → cancel/ignore
```

프로세스 누적을 허용하지 않는다.

---

# 4. 검색 버튼 제거

검색 버튼은 존재하지 않는다.

```text
[ 🔍 login path:src ]
```

Query 변경 자체가 검색 trigger다.

---

# 5. 검색 경로

UI 최상단에 조회 기준이 되는 상위 디렉터리 입력 요소 1개를 둔다.

```text
검색 경로
[ D:\Projects\my-project                         ] [찾아보기]
```

규칙:

- 입력값은 directory여야 한다.
- 존재하지 않는 경로면 검색하지 않는다.
- file path이면 검색하지 않는다.
- 검색 범위는 해당 directory의 모든 하위 directory다.
- Folder Picker 제공.
- 검색 경로 변경 시 현재 Query로 재검색.
- 최근 검색 경로는 설정으로 보존.

---

# 6. Query가 Single Source of Truth

다음 Chip 기반 mode UI는 제거한다.

```text
Content
Path
File
Ext
Exclude
```

모든 검색 기능은 항상 활성화한다.

```text
login
!login
'login
login | logout
path:src
!path:test
ext:ts
!ext:json
```

원칙:

```text
Query = 검색 조건의 Single Source of Truth
```

검색창 하단에는 기능 토글 대신 문법 힌트만 표시한다.

```text
문법: 공백=AND   'exact   !exclude   |=OR   path:src   ext:ts/md
```

---

# 7. 검색 문법

Skim 표현식 사용감을 유지한다.

## 7.1 기본

```text
login
```

## 7.2 AND

공백은 AND다.

```text
login error
```

의미:

```text
login AND error
```

## 7.3 Exact

```text
'login
```

Skim exact 조건.

## 7.4 Exclude / NOT

```text
!login
```

조합:

```text
login !test
```

의미:

```text
login AND NOT test
```

## 7.5 OR

```text
login | logout
```

의미:

```text
login OR logout
```

---

# 8. skim-search 전용 Scope 문법

Skim content filtering과 별도로 filesystem 검색 범위를 결정한다.

```text
path:src
!path:test
ext:ts
ext:md
!ext:json
```

향후 필요 시:

```text
file:auth
!file:test
```

를 같은 규칙으로 추가 가능하게 parser를 구성한다.

단, v1의 `file:`은 선택 구현이다.

---

# 9. Query Parse 규칙

예:

```text
login !test path:src ext:ts
```

분리:

```text
Skim/content expression:
    login !test

Scope:
    include path = src
    include ext  = ts
```

또:

```text
login | logout ext:md
```

의미:

```text
(login OR logout)
AND
extension == md
```

`path:`, `!path:`, `ext:`, `!ext:` token은 scope 조건으로 분리하고 나머지를 Skim query로 전달한다.

---

# 10. Filtering 정확성

`sk.exe`에 아래처럼 path/line/text가 합쳐진 전체 문자열을 검색 대상으로 그대로 넘기지 않는다.

```text
C:\path\file.ts:12:3:login(...)
```

그렇게 하면 `!login` 같은 content 조건이 path/file name까지 영향을 줄 수 있다.

요구 의미는 기본적으로 **파일 내용 match**다.

다음 중 안전한 방식을 선택한다.

1. metadata(path/line/column)와 content를 내부적으로 분리하고 `sk`에는 content 중심 payload만 전달
2. sk의 field/nth 계열 옵션으로 검색 대상 column을 content로 제한
3. unique result id를 사용해 metadata와 filtered row를 Rust에서 mapping

핵심 규칙:

```text
Skim content expression은 path metadata 때문에 오염되면 안 된다.
```

`path:` / `ext:`는 Rust Query Parser + rg scope에서 처리한다.

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

`sk.exe`는 Skim 검색 표현식 filtering 담당.

자체 fuzzy/exact/OR/NOT 엔진을 새로 만들지 않는다.

실행 전:

- `sk.exe` 존재 여부 확인
- `rg.exe` 존재 여부 확인

실행 파일 탐색 우선순위 예:

```text
configured path
→ PATH
→ 필요한 경우 알려진 fallback
```

없으면 오류 Toast 표시.

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

좌측 결과를 파일별 그룹화한다.

```text
128 결과 · 42 파일 · 0.18초

▼ TS src/auth/login.ts         6
   12  function login(...)
   23  loginWithToken(...)
   45  login success
   78  login failed

> MD docs/login-guide.md       4
> TS src/api/auth.ts           3
> RS src/search/engine.rs      2
```

파일 유형은 짧은 badge/icon으로 구분 가능.

---

# 15. Result Panel 전체 갱신

새 검색이 실행되면 첫 그룹 하나만 갱신하면 안 된다.

반드시 아래 전체가 최신 검색 결과로 바뀌어야 한다.

```text
stats
file group list
match count
visible match lines
selected result
```

---

# 16. Streaming

가능하면 검색 완료까지 기다리지 말고 부분 결과를 UI에 전달한다.

```text
첫 결과 빠르게 표시
→ 이후 결과 점진적 추가
```

단, Latest Query Wins를 항상 우선한다.

이전 generation의 chunk는 즉시 폐기한다.

---

# 17. Preview

우측 Preview는 read-only가 아니다.

선택한 검색 결과의 파일 내용을 표시하고 직접 수정 가능해야 한다.

지원:

```text
source code
Markdown
plain text
```

선택 match 위치 주변으로 자동 이동하고 가능하면 line/column highlight를 표시한다.

---

# 18. Preview 편집 / 저장

Preview에서 직접 수정.

저장:

```text
Ctrl + S
```

성공 시 Toast:

```text
저장 완료
src/auth/login.ts
```

---

# 19. 외부 변경 충돌 방지

Preview load 이후 외부 프로그램이 같은 파일을 수정했는데 무조건 overwrite하면 안 된다.

최소 다음 중 하나로 저장 직전 원본 변경 여부를 확인한다.

```text
mtime
file size + mtime
hash
```

충돌 시:

```text
파일이 외부에서 변경됨
```

을 표시하고 자동 overwrite하지 않는다.

---

# 20. 파일 열기

기존의:

```text
VS Code에서 열기
```

는 제거.

최종:

```text
[ 파일 열기 ]
```

UI에 특정 Editor 이름을 노출하지 않는다.

내부 설정:

```text
VSCode
Cursor
System Default
```

VSCode:

```text
code --goto "file:line:column"
```

Cursor:

```text
cursor --goto "file:line:column"
```

System Default는 Windows Shell open.

---

# 21. 부모경로 열기

버튼:

```text
[ 부모경로 열기 ]
```

Windows Explorer에서 현재 파일을 선택한 상태로 연다.

```text
explorer.exe /select,"D:\Projects\my-project\src\auth\login.ts"
```

---

# 22. 키보드 UX

```text
Ctrl + Shift + F
→ skim-search 실행 / 표시 / query focus

Typing
→ 자동 검색

↑ / ↓
→ 결과 이동

Enter
→ 파일 열기

Ctrl + Enter
→ 부모경로 열기

Ctrl + S
→ Preview 저장

Esc
→ skim-search 숨기기
```

Enter는 검색 실행에 사용하지 않는다.

---

# 23. Toast

우측 하단 Stack.

```text
┌────────────────────────┐
│ 검색 완료              │
│ 128 results · 0.18s   │
└────────────────────────┘
┌────────────────────────┐
│ 저장 완료              │
│ login.ts               │
└────────────────────────┘
┌────────────────────────┐
│ 파일 열기              │
│ login.ts:45            │
└────────────────────────┘
```

규칙:

- 최신 Toast가 아래
- 기존 Toast가 위로 이동
- 최대 3개
- 일정 시간 후 자동 제거
- info/success/warning/error 구분 가능

---

# 24. 최종 UI

Chip 없음.
검색 버튼 없음.

```text
┌───────────────────────────────────────────────────────────────┐
│ 검색 경로                                                     │
│ [ D:\Projects\my-project                          ] [찾아보기]│
│                                                               │
│ 🔍 login !test path:src ext:ts                                │
│                                                               │
│ 문법: 공백=AND  'exact  !exclude  |=OR  path:  ext:           │
├───────────────────────────────┬───────────────────────────────┤
│ 128 결과 · 42 파일 · 0.18초  │ src/auth/login.ts        45:3 │
│                               │                               │
│ ▼ TS src/auth/login.ts     6  │ [파일 열기] [부모경로 열기] │
│   12 function login           │                               │
│   23 loginWithToken           │ Preview / Editor              │
│   45 login success            │                               │
│                               │                               │
│ > MD docs/login-guide.md   4  │                               │
│ > TS src/api/auth.ts       3  │                               │
├───────────────────────────────┴───────────────────────────────┤
│ rg 14.x · sk x.x · 128 results · 0.18 sec                    │
└───────────────────────────────────────────────────────────────┘
```

기존 확정 dark theme / VSCode 계열 visual language 유지.
과도한 animation 금지.

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

최소 자동 테스트:

## Query Parser

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
login !test path:src ext:ts
TODO ext:md
```

Scope token과 Skim expression 분리 검증.

## Latest Query Wins
이전 검색 결과를 일부러 늦게 반환시켜 최신 generation만 UI model에 반영되는지 테스트.

## Result Parsing
Windows path 안전 처리:

```text
C:\Projects\test\src\main.rs
D:\My Projects\sample file.md
```

공백 포함 path 필수 지원.

## Save Conflict
외부 변경을 simulated하여 conflict detection 검증.

---

# 30. 로그 / 진단

Debug build에서 최소 추적 가능:

```text
query generation
raw query
parsed skim query
scope filters
search root
rg spawn time
sk spawn time
first result latency
search completed latency
result count
cancelled generation
error
```

일반 UI는 로그로 오염시키지 않는다.

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
