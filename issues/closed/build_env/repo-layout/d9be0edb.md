# title
Repository layout: per-language cores, root scripts / configs / target

# pre-condition
- Other contributors' in-progress changes (`cores/tests/`, `skim_tests`, `cores/scripts/one_shot/`, `issue_ids.py`) are committed first.
- No build or test runs at the same time (file locks when deleting/moving `cores/target/`).

# steps
Source: user request (2026-10-01): build output in the root `target/` (Rust conventional name), entry scripts in the root `scripts/`, important settings in `configs/`, only logic in `cores/`, Rust and Python follow their conventions (Rust `src/`, Python src layout), tests inside each language project, `.cargo` option A (root `.cargo/config.toml`), Rust tests option 1 (keep `cores/rust/tests/`)
Target layout:
```text
.cargo/config.toml          [build] target-dir = "target"
configs/                    one-shot.json, security-exceptions.json
scripts/                    test.cmd/.ps1, one-shot.cmd/.ps1, security-policy.cmd/.ps1, issue-id.cmd, one_shot/(per-stage .cmd/.ps1, launch.ps1)
cores/common/               language-neutral SSOT (build_env/paths-ssot)
cores/rust/                 Cargo.toml, Cargo.lock, common/, app/(build.rs, src/{family}/, ui/), tests/
cores/python/               pyproject.toml, uv.lock, src/skim_search/(diagnostics/: one_shot/, security_policy.py, issue_ids.py), tests/(support/, e2e/, one_shot/, security/, issue_ids/), benchmarks/
target/                     cargo output (not committed), one-shot isolated build in target/one-shot/
```
1. Delete `cores/target/` (user decision: delete, do not move).
2. Create the root `.cargo/config.toml` and move the Cargo workspace to `cores/rust/`.
3. Move Python code to `cores/python/` (src layout, package `skim_search`) and split benchmarks into `benchmarks/`.
4. Move `.cmd`/`.ps1` entry points to `scripts/` and settings files to `configs/`. Entry points compute the root with `%~dp0..`.
5. Update path references (scripts, `pipeline.py`, `launch.ps1`, `test_pipeline.py`, `.gitignore`, `security_policy.py`), `rules/*.md` (including the location column of `families.md`) and `README.md`.
6. Run the full verification.

# actual result
PASS (2026-10-02, commit 0841838).
- Moves: `cores/{Cargo.*,app,common,tests}` → `cores/rust/`; `cores/tests/py` + `cores/scripts` → `cores/python/` (src layout, package `skim_search`: `diagnostics/one_shot/`, `security_policy.py`, `issue_ids.py`; `tests/{support,e2e,one_shot,security,issue_ids}`, `benchmarks/{rg,sk}`); `.cmd/.ps1` entry points → `scripts/` (`test.cmd`, `one-shot`, `security-policy`, `issue-id`, `one_shot/{stage}`; launch.ps1 runs `python -m` modules); `config.json` → `configs/one-shot.json`, `security-exceptions.json` → `configs/`; root `one_shot.py` removed (`skim_search.diagnostics.one_shot.__main__`). `cores/target/` (14 GB) deleted, root `.cargo/config.toml` `target-dir = "target"`. pyproject name `skim-search`, `uv lock` refreshed.
- Path updates: app log `CARGO_MANIFEST_DIR/../../..`; pipeline (ROOT, stage scripts, CI cwd `cores/rust`, module cwd `cores/python`, one-shot build `target/one-shot`, cargo-audit/pip-audit paths); security_policy and issue_ids ROOT / exceptions file; test path SSOT `tests/support/paths.py`; test_pipeline temporary repository setup (the standalone push entry test imports the temporary copy via PYTHONPATH → no issue is created in the real repository).
- Verification:
  - `cargo build` in `cores/rust` → root `target/debug/skim-search.exe`; no `cores/target`.
  - Full `scripts\test.cmd` from `%TEMP%`: `ALL PASSED`, exit 0 (`ref/actual/logs/test-cmd-full.log`): cargo build/test/clippy, 33 Python tests (pipeline 25, issue_ids 7, security policy 1, `python-unittest.log`), e2e_detection 11/11, e2e_ui 21/21, benchmarks.
  - From `%TEMP%`: `scripts\security-policy.cmd` exit 0, `scripts\issue-id.cmd --help`, `scripts\test.cmd quick` PASS.
  - From `%TEMP%`: `scripts\one-shot.cmd --bump minor` → run `20261002-024248-5e7cfc0e` all stages passed, `target/one-shot/release/skim-search.exe --version` = `skim-search 0.5.0 0841838…`, origin/main == 0841838.
  - No old paths in `rules/*.md`, `README.md`, `.gitignore` (`git grep`, excluding history in closed issues and `ref/closed`).
- Decisions: the e2e scripts are runnable scripts, not test-framework targets, so they are `tests/e2e/detection.py`, `ui.py` (no `test_` prefix). The security self-test stays in the module and `tests/security/test_policy.py` calls it.

Previous state: workspace `cores/`, build output `cores/target/` (14 GB), scripts in `cores/scripts/` and the root (`test.cmd`, `one-shot.*`, `one_shot.py`), Python project `cores/tests/py`. Path computation spread over 5 places (each with a different `parents[N]` depth).
Background: one-shot CI used the shared `cores/target/` and a concurrent build overwrote the exe (`closed/diagnostics/one-shot-concurrent-build-overwrites-exe`).

# expected result
- Running cargo in `cores/rust/` outputs to the root `target/`. There is no `cores/target/`.
- one-shot builds in `target/one-shot/` and the `--version` check passes.
- No executables (`.cmd`/`.ps1`/`.py`) at the root. `scripts/` holds only entry points and they work from other folders.
- `cores/` contains only `common/`, `rust/`, `python/`. Rust follows `src/`, Python `src/skim_search/`, and tests live inside each language project.
- `configs/` holds the one-shot settings and the security exception list, and code reads them from there.
- Full `scripts\test.cmd`, `test_pipeline`, the security policy test and the `issue_ids` tests all pass, with results in `ref/actual/logs/`.
- No old paths (`cores/target`, `cores/scripts`, `cores/tests/py`, root `test.cmd`) remain in `rules/*.md`, `README.md`, `.gitignore` (excluding history in `ref/closed/` and closed issues).
- The `security_policy` check exits 0.

# label
SQA_sk_0_0_0

# environment
OS: Windows 11 Pro
hostname: TBD

# assignee
