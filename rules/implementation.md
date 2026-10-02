# implementation — code structure (`cores/`)

## Locations

Repository root (`build_env/repo-layout/d9be0edb`):

```text
.cargo/config.toml   ← [build] target-dir = "target" (cargo fixes this location, so it is at the root)
configs/             ← settings read by the project: one-shot.json, security-exceptions.json
scripts/             ← entry points only (.cmd → .ps1 → Python module). No logic, no subfolders
cores/rust/          ← Cargo workspace (Rust conventions)
cores/python/        ← uv project (Python conventions, src layout, package skim_search)
cores/common/        ← language-neutral SSOT (build_env/paths-ssot)
target/              ← cargo output (not committed). one-shot isolated build: target/one-shot/
```

- All implementation code lives under `cores/`. No logic in `issues/`, `ref/`, `rules/`, `scripts/`, `configs/`. No executables at the root.
- Entry point locations and language responsibilities follow `one-shot.md#execution-structure` and `one-shot.md#language-responsibilities`.
- Tests live inside each language project: Rust `cores/rust/tests/`, Python `cores/python/tests/` (benchmarks in `cores/python/benchmarks/`).
- `cores/rust/` = Cargo workspace with three members: `common` (lib), `app` (lib+bin), `tests` (integration tests).
- Feature code is a module tree inside `cores/rust/app/src/` named after the function family (locations: `families.md`).
- Rust code used by two or more function families goes to `cores/rust/common/`.
- All folder / file / module names are English (ASCII, snake_case).
- Code comments, docstrings and generated text (logs, issues) are English. Product UI strings follow handover.

## Target tree (Rust)

```text
cores/rust/
├─ Cargo.toml                  ← [workspace] members = ["common", "app", "tests"]
├─ common/                     ← shared lib crate
│  ├─ Cargo.toml
│  ├─ build.rs                 ← paths.ini → common::paths constants
│  └─ src/
│     ├─ lib.rs
│     ├─ log.rs                ← runtime evidence log (environment.md#evidence-paths)
│     ├─ paths.rs              ← path SSOT constants and root discovery
│     ├─ settings.rs           ← settings load/save
│     ├─ process.rs            ← external process spawn/kill
│     └─ path.rs               ← Windows path handling
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
│     ├─ main.rs               ← bin entry (keep thin)
│     ├─ lib.rs                ← exposes modules (used by the tests crate)
│     ├─ app.rs                ← UI model ↔ modules
│     ├─ query_syntax/         ← query parser
│     ├─ search_engine/        ← rg, sk, generation, cancel, streaming
│     ├─ result_panel/         ← result grouping model
│     ├─ preview_editor/       ← load/save/conflict
│     ├─ external_open/        ← editor launch, explorer /select
│     ├─ shortcut/             ← global hotkey
│     └─ toast/                ← toast stack model
└─ tests/                      ← integration test crate (diagnostics, handover §29)
   ├─ Cargo.toml               ← depends on common, app
   ├─ fixtures/sample/tree/    ← golden sample tree (issue.md common test workspace)
   └─ tests/{function family}_*.rs
```

## Python tree

```text
cores/python/
├─ pyproject.toml, uv.lock     ← project skim-search (uv_build, src layout)
├─ src/skim_search/
│  ├─ __init__.py              ← generated path constants (gen_paths)
│  ├─ gen_paths.py
│  └─ diagnostics/
│     ├─ one_shot/             ← pipeline, stage modules, failure_issue (python -m skim_search.diagnostics.one_shot)
│     ├─ security_policy.py
│     └─ issue_ids.py
├─ tests/                      ← test_*.py (unittest), e2e scripts, shared helpers
│  ├─ support/                 ← common, capture
│  ├─ e2e/                     ← detection, ui
│  ├─ one_shot/ security/ issue_ids/ paths/
└─ benchmarks/                 ← rg, sk, paths
```

## Structure rules

- The trees above are the target shape. Create only the files/modules an issue needs when it starts. No empty modules in advance.
- `common/` holds only code actually used by two or more function families. No speculative sharing.
- Dependency direction: `tests` → `app` → `common`. No reverse references. No cycles between `app` modules.
- Repository and 3rd_party structure paths are defined only in `cores/common/paths.ini` (`build_env/paths-ssot`). No structure path strings or `parents[N]` arithmetic in code.
  - Rust: `common::paths` (`common/build.rs` generates `const &str`). Roots are found at runtime with `paths::repo_root(start)` / `paths::third_party(start)` and joined with `paths::join`. No absolute path built from `env!("CARGO_MANIFEST_DIR")` in app code (allowed only as a search start in tests).
  - Python: `from skim_search import KEY` or `skim_search.REL["KEY"]` (relative). Generate with `python -m skim_search.gen_paths` (`--check` runs in test.cmd and as the first one-shot CI step). The generated region holds string constants only, no imports (measured by `benchmarks.paths`, at most 50 µs).
  - To add a key, edit `paths.ini` and run the generator. Above 200 keys a warning asks to derive paths by rule instead of listing them.
- Do not create the layers prohibited by handover §25 (`manager/`, `repository/`, `service/`, `adapter/`, `domain/`).
- Update the `families.md` table first when adding a function family.
- This differs from the structure recommended in handover §25 (single crate). The user instruction (`cores/` tree, per-language split) takes precedence.

## UI

- Dark theme: style `fluent-dark` in `app/build.rs` (handover §24).
- Default font `Segoe UI`. `Malgun Gothic` renders `\` as `₩` and breaks Windows paths (`closed/view/backslash-rendered-as-won/8db8c717`).
- Required UI elements carry an `accessible-label` (used by tests).
- Spawn external processes with `CREATE_NO_WINDOW` (prevents console flashes in the release GUI).

## Tests

- Public API checks and handover §29 scenarios go to `cores/rust/tests/`. Only private-function unit tests may use `#[cfg(test)]` in the source file.
- The first line of a test file names its issue (e.g. `//! issue: view/main-ui-components-missing/521ce549`).
- UI tests find elements with `i-slint-backend-testing` + `accessible-label`. `with_debug_info(true)` in `app/build.rs` is required.
- Save results in `ref/actual/logs/`:
  - `cargo test 2>&1 | tee ref/actual/logs/cargo-test.log`
  - `cargo clippy --all-targets 2>&1 | tee ref/actual/logs/cargo-clippy.log`

## Logs

- Runtime evidence is written with `common::log::write(event, detail)` (AGENTS.md 9). event is a function family name or `app`/`test`.
- Log failures must not affect app behavior.
- Path and override: `environment.md#environment-variables`.
