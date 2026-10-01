# issue — issue 작성·추적·완료 규칙

## 경로와 상태

```text
issues/
├─ backlog/   ← 착수 전
│  └─ {function family}/{sub family}/{uuid:8}.md
├─ working/   ← 실제 진행 중
│  └─ {function family}/{sub family}/{uuid:8}.md
├─ closed/    ← 완료 또는 작업 중 생긴 일의 기록
│  └─ {function family}/{sub family}/{uuid:8}.md
└─ priority.md
```

- 상태는 `backlog` / `working` / `closed` 폴더로 표시하고 issue 본문에 status 항목을 두지 않는다.
- issue 경로는 `{function family}/{sub family}/{uuid:8}`이다. 상태 폴더는 참조에서 생략한다.
- 상태 변경 시 issue 파일 하나만 같은 family/subfamily 경로로 이동한다. 폴더째 이동하지 않는다. 빈 폴더는 이동 후 삭제한다.
- 모든 기존 issue도 UUID 파일명으로 이관한다. 상태 폴더, family, subfamily는 유지한다.

## 이름과 UUID

- 모든 폴더 이름은 영어 ASCII. function family는 `families.md`의 snake_case 이름, sub family는 kebab-case로 쓴다.
- 새 issue 파일명은 UUID 문자열의 앞 8자리 소문자 16진수로 한다. 실제 이름 예: `a1b2c3d4.md`. `{uuid:8}`은 규칙 표기이며 중괄호와 콜론은 파일명에 넣지 않는다.
- ID 생성·배정 SSOT는 Git 공통 디렉터리의 `issue_ids.sqlite3` 한 개다. 생성 진입점은 `cores/scripts/issue_ids.py`의 `get_issue_id()` 하나로 통일한다. CLI: `uv run --project cores/tests/py python cores/scripts/issue_ids.py`.
- 최초 초기화 트랜잭션에서만 `backlog` / `working` / `closed`의 기존 ID와 이전 예약 파일을 DB로 이관한다. 이후 전체 폴더 검색 없이 DB의 고유 키·트랜잭션으로 충돌 시 재생성하고 커밋 후 반환한다. 이전 예약 파일은 이관 커밋 후 제거한다. SQLite 임시 journal과 런타임 로그는 복구·검증용이며 ID 배정 SSOT가 아니다.
- 모든 이슈 작성자가 이 API를 사용하며 DB를 삭제·재생성하거나 수동으로 ID를 배정하지 않는다. 초기화 이후 외부 이슈를 들여올 때는 배정 중단 상태에서 ID 등록·중복 검증을 먼저 수행한다. 별도 clone은 DB가 공유되지 않으므로 병합 전 등록·중복 검증이 필요하다.
- 상태를 이동해도 UUID는 유지한다.
- Windows 금지 문자 `\\ / : * ? " < > |`를 경로에 쓰지 않는다.

## 우선순위

| 등급 | 기준 |
|------|------|
| Critical | 데이터 손실 위험, 잘못된 파일 표시·수정·저장, 또는 핵심 Flow(handover §2)를 막아 즉시 처리해야 하는 문제 |
| High | 필수 기능 또는 AC 실패로 주요 사용 흐름이 불완전하지만 Critical 기준에 해당하지 않는 문제 |
| Normal | 사용성·표시 품질·진단·성능 목표 개선. 핵심 동작은 가능하지만 요구 품질에 미달하는 문제 |
| Low | 선택 기능(`file:` 등), showreel 전용 요소, 요구가 확정되지 않은 `추정:` 항목 |

## 우선순위 목록

`issues/priority.md`는 활성 UUID issue의 목록이며 다음 세 열만 사용한다.

| 우선순위 | UUID | 근거 |
|---|---|---|
| Critical / High / Normal / Low | 8자리 소문자 16진수 | FR/AC, handover 절, showreel 시각 등 |

- 업무 제목이나 상태 열을 추가하지 않는다. 업무 내용은 개별 issue의 `title`에 둔다.
- `backlog`와 `working` issue만 목록에 둔다. `closed`로 이동하면 행을 제거한다.
- 목록은 등급(Critical → High → Normal → Low), 선행 의존, `families.md` 순으로 정렬한다.
- 각 행의 근거는 대응하는 개별 issue의 `steps`에 적힌 요구사항 근거와 일치해야 한다.

## 분할 원칙

- issue 하나는 PASS/FAIL 판정 가능한 결과 하나를 다룬다.
- AC 하나 이하 또는 FR 1~2개에 대응한다. 초과하면 분리한다.
- 문법 token은 token 단위 분리 가능 (`!path:`와 `!ext:` 등).
- UI 구성과 동작은 분리한다.
- showreel에서 확인한 결함은 관찰 1건당 issue 하나로 작성한다.
- handover가 명시한 구현 제약만 인용하고 구현 방법은 강제하지 않는다.
- handover §33 금지사항을 요구하는 issue는 만들지 않는다.
- `backlog` / `working` / `closed` 전체에서 같은 FR/AC를 다룬 기존 issue를 확인해 중복을 피한다.

## 작성 절차

1. `sources.md` 기준으로 handover FR/AC 및 showreel에서 검증 단위를 찾는다.
2. `families.md`에서 family를 정하고 sub family 이름을 정한다.
3. 이 문서의 우선순위 기준으로 등급을 정한다.
4. `#이름과-UUID`의 `get_issue_id()`로 새 ID를 예약한다.
5. 제안 목록(경로 + title + 등급 + 근거)을 먼저 보고하고 승인 후 파일을 만든다.
6. `issues/priority.md`에 우선순위·UUID·근거 행을 추가한다.

## 상태 전이와 착수

```text
issues/backlog/… ──착수──▶ issues/working/… ──PASS──▶ issues/closed/…
```

- 착수 전 `추정:` issue가 아닌지 확인한다. 요구가 미확정이면 착수하지 않는다.
- 기본은 한 번에 issue 하나만 `backlog`에서 `working`으로 옮긴다. 코드 변경을 묶는 편이 효율적이면 issue 목록과 사유를 먼저 보고한다.
- `working`에는 실제 진행 중인 issue만 둔다.
- 착수 순서는 `issues/priority.md`를 따른다. 같은 등급에서는 선행 의존을 먼저 처리하고, 의존이 없으면 `families.md` 순서를 따른다.
- 사용자가 특정 issue를 지목했는데 이미 `closed`이면 재개하거나 재작업하지 않는다. `priority.md`에서 그 issue를 건너뛰고, 착수 가능한 다음 `backlog` issue를 선택해 진행한다. 이후 항목도 모두 `closed`이면 다음으로 우선순위가 높은 `backlog` issue를 찾고, 활성 `backlog`가 없을 때만 완료 상태를 보고한다.
- 수정 대상 파일과 이유를 편집 전에 보고한다.

## 완료 조건과 보고

- 모든 expected result가 PASS이고 증거가 있어야 완료다. FAIL이 남으면 issue를 `working`에 둔다.
- 의존 issue 때문에 직접 검증할 수 없으면 대체 검증 방법과 의존 issue를 기록하고 판단을 요청한다.
- 완료 시 build, `cargo test`, `cargo clippy` 결과를 `ref/actual/logs/`에 저장하고 필요한 e2e·화면 증거를 모은다.
- issue의 `actual result`를 실측 결과와 증거 경로로 갱신한 뒤 `closed`로 이동하고 `priority.md` 행을 제거한다.
- 완료 보고에는 결론, expected result별 판정과 증거, 변경 파일, 미검증 항목과 이유, 기록 issue, 다음 착수 대상을 포함한다.

## 파일 템플릿

섹션명과 순서는 바꾸지 않는다.

```markdown
# title
{한 줄, 80자 이내}

# pre-condition
{실행 상태, 검색 경로, 테스트 데이터 등}

# steps
근거: FR-xxx, AC-xxx (handover §n) / showreel 00:SS
1. ...

# actual result
{현재 동작. 미구현이면 "미구현"}

# expected result
{PASS 판정 기준. 측정값이 필요하면 수치와 확인 위치 명시}

# label
SQA_sk_0_0_0

# environment
OS: windows 10 pro
hostname: TBD

# 담당자
```

### 필드와 근거

- `steps` 첫 줄에는 FR/AC/handover 절/showreel 시각 근거를 쓴다. 기록 issue는 `발생: {issue 경로}`도 쓴다.
- showreel 관찰은 관찰 내용 그대로 적고 해석은 `추정:`으로 분리한다.
- 작업 issue 완료 시 `actual result`는 실측 결과와 증거 경로로 바꾼다.
- expected result는 로그 등 런타임 증거로 검증 가능하게 쓴다. 성능 목표는 목표와 실측 기록 위치를 함께 적고, 미달도 보고한다.
- 별도 pre-condition이 없으면 아래 공통 테스트 workspace를 쓴다.
- `label` 기본값은 `SQA_sk_0_0_0`이다.

### 공통 테스트 workspace

공통 검색 fixture는 `cores/tests/fixtures/sample/tree/`에 커밋한다. Rust·Python 테스트는 이를 임시 경로에 복사해 쓰고 fixture 원본은 변경하지 않는다. scope 전용 추가 파일은 해당 테스트 코드에서 만든다.

## 작업 중 생긴 일

제품 결함·사고·환경 제약·절차 실수·도구 오류·빌드/테스트 실패는 예외 없이 발생 즉시, 늦어도 해당 작업 완료 보고 전까지 기록 issue로 남긴다. 기록 issue는 항상 `closed`에 둔다.

- family는 제품 결함이면 해당 기능, 검증 도구면 `diagnostics`, 환경이면 `build_env`, 절차 실수면 `process`로 한다.
- 기록 issue의 `steps`에 `발생: {issue 경로}`를 쓴다.
- `actual result`는 `발생` / `원인` / `조치` / `결과`(PASS 또는 `미해결`) / `재발 방지` 순서로 쓴다.
- expected result에는 해당 일이 없었을 때의 정상 동작을 쓴다.
- 미해결이면 `결과: 미해결`로 두고 해결용 issue를 `backlog`에 별도로 작성해 서로 경로를 기록한다.
- 재발 방지책이 규칙이면 이 문서에 반영한다.
