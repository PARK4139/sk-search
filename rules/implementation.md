# implementation — 코드 구조 (`cores/`)

## 위치

저장소 루트 (`build_env/repo-layout/d9be0edb`):

```text
.cargo/config.toml   ← [build] target-dir = "target" (cargo가 정한 위치라 루트)
configs/             ← 프로젝트가 읽는 설정: one-shot.json, security-exceptions.json
scripts/             ← 실행 진입점만 (.cmd → .ps1 → Python 모듈). 로직 없음
cores/rust/          ← Cargo workspace (Rust 관례)
cores/python/        ← uv 프로젝트 (Python 관례, src layout, 패키지 skim_search)
cores/common/        ← 언어 중립 SSOT (build_env/paths-ssot)
target/              ← cargo 출력 (git 제외). one-shot 격리 빌드는 target/one-shot/
```

- 구현 코드는 모두 `cores/` 아래에 둔다. `issues/`, `ref/`, `rules/`, `scripts/`, `configs/`에 로직을 두지 않는다. 루트에는 실행 파일을 두지 않는다.
- 진입점 위치·언어별 책임은 `one-shot.md#실행-구조`, `one-shot.md#언어별-책임`을 따른다.
- 테스트는 각 언어 프로젝트 안에 둔다: Rust `cores/rust/tests/`, Python `cores/python/tests/`(벤치마크는 `cores/python/benchmarks/`).
- `cores/rust/` = Cargo workspace. member는 `common`(lib), `app`(lib+bin), `tests`(통합 테스트) 3개.
- 기능 코드는 `cores/rust/app/src/` 안에서 function family 이름과 같은 module로 tree 구조화한다 (위치표: `families.md`).
- 2개 이상 function family가 사용하는 Rust 공통 기능은 `cores/rust/common/`에 둔다.
- 모든 폴더/파일/module 이름은 영어(ASCII, snake_case).

## 목표 tree (Rust)

```text
cores/rust/
├─ Cargo.toml                  ← [workspace] members = ["common", "app", "tests"]
├─ common/                     ← 공통 lib crate
│  ├─ Cargo.toml
│  └─ src/
│     ├─ lib.rs
│     ├─ log.rs                ← 런타임 증거 로그 (environment.md#증거-경로)
│     ├─ settings.rs           ← 설정 load/save
│     ├─ process.rs            ← 외부 프로세스 spawn/kill
│     └─ path.rs               ← Windows 경로 처리
├─ app/                        ← lib + bin crate: skim-search.exe
│  ├─ Cargo.toml
│  ├─ build.rs
│  ├─ ui/
│  │  ├─ main.slint
│  │  ├─ view.slint
│  │  ├─ result_panel.slint
│  │  ├─ preview_editor.slint
│  │  ├─ status_bar.slint
│  │  └─ toast.slint
│  └─ src/
│     ├─ main.rs               ← bin entry (얇게 유지)
│     ├─ lib.rs                ← module 공개 (tests crate에서 사용)
│     ├─ app.rs                ← UI model ↔ module 연결
│     ├─ query_syntax/         ← 쿼리 parser
│     ├─ search_engine/        ← rg, sk, generation, cancel, streaming
│     ├─ result_panel/         ← 결과 grouping model
│     ├─ preview_editor/       ← load/save/conflict
│     ├─ external_open/        ← editor 실행, explorer /select
│     ├─ shortcut/             ← global hotkey
│     └─ toast/                ← toast stack model
└─ tests/                      ← 통합 테스트 crate (diagnostics, handover §29)
   ├─ Cargo.toml               ← dev-dependency: common, app
   ├─ fixtures/sample/tree/    ← 골든 샘플 트리 (issue.md 공통 테스트 workspace)
   └─ tests/{function family}_*.rs
```

## Python tree

```text
cores/python/
├─ pyproject.toml, uv.lock     ← 프로젝트 skim-search (uv_build, src layout)
├─ src/skim_search/
│  └─ diagnostics/
│     ├─ one_shot/             ← pipeline, 단계 모듈, failure_issue (python -m skim_search.diagnostics.one_shot)
│     ├─ security_policy.py
│     └─ issue_ids.py
├─ tests/                      ← test_*.py (unittest), e2e 스크립트, 공용 헬퍼
│  ├─ support/                 ← common, capture, paths(테스트 경로 SSOT)
│  ├─ e2e/                     ← detection, ui
│  ├─ one_shot/ security/ issue_ids/
└─ benchmarks/                 ← rg, sk
```

## 구조 규칙

- 위 tree는 목표 형태다. 파일/module은 해당 issue 착수 시 필요한 것만 만든다. 빈 module 선생성 금지.
- `common/`에는 실제로 2개 이상 function family에서 쓰는 코드만 둔다. 미래 사용을 가정한 선제 공통화 금지.
- 의존 방향: `tests` → `app` → `common`. 역방향 참조 금지. `app` 내부 module 간 순환 의존 금지.
- Python 검증 도구의 경로 상수는 `cores/python/tests/support/paths.py` 한 곳에서 정의한다 (언어 중립 SSOT 전환: `build_env/paths-ssot`).
- handover §25의 금지 계층(`manager/`, `repository/`, `service/`, `adapter/`, `domain/`)은 만들지 않는다.
- 새 function family 추가 시 `families.md` 표를 먼저 갱신한다.
- handover §25 권장 구조(단일 crate)와 다르다. 사용자 지시(`cores/` tree, 언어별 분리)가 우선한다.

## UI

- Dark theme: `app/build.rs`에서 style `fluent-dark` (handover §24).
- 기본 폰트 `Segoe UI`. `Malgun Gothic`은 `\`를 `₩`로 표시해 Windows 경로가 깨진다 (`closed/view/backslash-rendered-as-won/8db8c717`).
- 필수 UI 요소에는 `accessible-label`을 붙인다 (테스트에서 사용).
- 외부 프로세스 spawn 시 `CREATE_NO_WINDOW` 사용 (release GUI에서 console 창 깜빡임 방지).

## 테스트

- 공개 API 검증·handover §29 시나리오는 `cores/rust/tests/`. private 함수 단위 테스트만 해당 파일 내 `#[cfg(test)]` 허용.
- 테스트 파일 첫 줄에 대상 issue를 적는다. (예: `//! issue: view/main-ui-components-missing/521ce549`)
- UI 테스트는 `i-slint-backend-testing` + `accessible-label`로 요소를 찾는다. `app/build.rs`의 `with_debug_info(true)` 필수.
- 실행 결과는 `ref/actual/logs/`에 저장한다:
  - `cargo test 2>&1 | tee ref/actual/logs/cargo-test.log`
  - `cargo clippy --all-targets 2>&1 | tee ref/actual/logs/cargo-clippy.log`

## 로그

- 런타임 증거는 `common::log::write(event, detail)`로 남긴다 (AGENTS.md 9). event는 function family 이름 또는 `app`/`test`.
- 로그 실패가 앱 동작에 영향을 주면 안 된다.
- 경로와 override는 `environment.md#환경변수`.
