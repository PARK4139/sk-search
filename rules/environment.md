# environment — development and verification environment

`CavemanDrive/` in paths = `%USERPROFILE%\Downloads\CavemanDrive\`.

## External tools

| Tool | Path | Version | Origin / notes |
|------|------|---------|----------------|
| rg | `CavemanDrive/3rd_party/ripgrep/rg.exe` | 15.2.0 (+pcre2) | Copy of the Codex bundle (OpenAI signed). Not on PATH (`closed/build_env/rg-executable-missing/5797d05d`) |
| sk | `CavemanDrive/3rd_party/skim/sk.exe` | 5.6.6 | Copy of winget `skim-rs.skim`. The winget original is on PATH |
| ffmpeg | `CavemanDrive/3rd_party/LosslessCut-win-x64/resources/ffmpeg.exe` | — | showreel frame extraction only |
| Rust | stable-x86_64-pc-windows-msvc | cargo 1.98.1 | |
| uv | `CavemanDrive/3rd_party/pk_system/uv.exe` | 0.12.9 | Not on PATH. Python 3.13 managed by uv (`cores/python/pyproject.toml` requires-python) |

- App rg/sk detection order: configured path → PATH → fallback (`<tool>.exe` in the exe folder and its ancestors, `3rd_party/{ripgrep,skim}/<tool>.exe`). During development the fallback finds `3rd_party`.
- Version record: `ref/actual/logs/env-tools.log`.
- showreel frame extraction:
  `3rd_party/LosslessCut-win-x64/resources/ffmpeg.exe -i ref/showreel/issue-full-flow.mp4 -vf fps=1 ref/showreel/frames/f%02d.png`
- System Python is not used. Python logic and tests run with `uv run` in the uv project `cores/python/` (Python 3.13, `uv.lock`). Add dependencies with `uv add` (updates pyproject.toml + uv.lock).
- Build output is the root `target/` (`.cargo/config.toml`). Run cargo in `cores/rust/`.

## cargo settings

- Corporate network schannel `CRYPT_E_NO_REVOCATION_CHECK` workaround: `[http] check-revoke = false` in `%USERPROFILE%\.cargo\config.toml` (this PC only, never in the repo).
- Remove it once the corporate certificate revocation list (CRL) is reachable (`closed/build_env/crates-io-ssl-revocation-check-failure/6d33b85a`).

## Evidence paths

| Kind | Path |
|------|------|
| Runtime log | `ref/actual/logs/skim-search.log` (append, `<unix_ms> [<event>] <detail>`, UTF-8). Items per handover §30 (debug build) |
| Build / test logs | `ref/actual/logs/cargo-{build,test,clippy}.log` |
| Tool versions | `ref/actual/logs/env-tools.log` |
| Screen evidence | `ref/actual/screenshot/frames/{sub family}_{priority}[_{case}].png` |

- `ref/actual/` is not a source (`sources.md`).
- `ref/actual/logs/` is local evidence. It is not committed so local paths and PIDs never reach the public repository (`ref/actual/logs/.gitignore`). Screenshots show only the skim-search window and are committed.

## Environment variables

| Variable | Purpose | Default |
|----------|---------|---------|
| `SKIM_SEARCH_LOG` | Runtime log path override | exe inside the repository: `ref/actual/logs/skim-search.log` (root found at runtime above the exe via `cores/common/paths.ini`). exe outside the repository (deployed, e2e case C copy): `skim-search.log` next to the exe. Tests set this variable explicitly |
| `SKIM_SEARCH_SETTINGS` | Settings file path override | `%APPDATA%\skim-search\settings.json` |

Settings fields: `rg_path`, `sk_path` (missing or unparsable → defaults, reason logged).

## Test scripts

All tests run through `scripts\test.cmd` (works from any cwd).

```text
scripts\test.cmd          build (debug/release) → cargo test → clippy (-D warnings) → path SSOT check → Python tool tests → e2e detection → e2e UI → bench rg/sk/paths
scripts\test.cmd quick    build → cargo test → clippy only
```

- Exit code 0 = all passed, 1 = failures (failing step names printed). Logs in `ref/actual/logs/`.
- The e2e steps open skim-search windows and send Ctrl+Shift+F. Do not type during the run (external input in the query is retried once, then fails).
- Latency targets (search start ≤25 ms, first result ≤50 ms, p50) WARN by default. Use `tests.e2e.ui --strict-latency` to fail.

uv project `cores/python/` (package `skim_search`, dependencies `uiautomation`, `Pillow`). Run individually from `cores/python`:

```text
uv run --locked python -m <module> [options]
uv run --locked python -m unittest discover -s tests -t . -p "test_*.py"
```

| module | Purpose | Log |
|--------|---------|-----|
| `tests.support.capture --exe <exe> --out <png>` | Launch the app → `PrintWindow` capture of this process's `skim-search` window only → WM_CLOSE → print exit code | — |
| `tests.e2e.detection [--no-build]` | rg/sk detection case A (configured) / B (PATH) / C (missing): run, capture, judge | `e2e-detection.log` |
| `tests.e2e.ui [--no-build] [--latency-runs N] [--strict-latency] [--system-open]` | Drive the release exe with UI Automation: search, preview, save, toasts, open, shortcuts, folder pick, settings save/restore, search error, latency distribution. Records `CHECK PASS/WARN/FAIL [issue]`. Workspace `%PUBLIC%\skim-search-e2e` | `e2e-ui.log` |
| `benchmarks.rg [--runs N]` | rg alone (app arguments + option variants) | `bench-rg.log` |
| `benchmarks.sk [--runs N]` | Processing time of a pre-spawned sk after input ends | `bench-sk.log` |
| `benchmarks.paths [--runs N] [--limit-us 50]` | Import cost of the generated path constants (generated `__init__` vs empty, alternating fresh processes) | `paths-bench.log` |
| `skim_search.gen_paths [--check]` | `cores/common/paths.ini` → generated region of `skim_search/__init__.py` (check: exit 1 on mismatch) | `paths-check.log` |

- Shared module `tests.support.common`: paths, UTF-8 log, Checker (exit code), app log waits, Win32 (ctypes), `App` that only closes its own process.
- Text values on screen (stats, status, preview path) are not readable via UIA. Judge from app log lines `[status_bar] stats=… status=…`, `[preview_editor] show path=… line=… column=…`.

## Script writing cautions

Preventions from incident issues:

- Test scripts close or kill only processes / windows they started (identified by PID or by HWND before/after comparison). Never pick targets by title or name. Opening files in the user's default app is opt-in (`tests.e2e.ui --system-open`) (`closed/process/e2e-closed-user-vscode`).
- Inject key input (`keybd_event`) only while skim-search is in the foreground (never into user apps). Global hotkey checks are the exception (the registered hotkey intercepts).
- (PowerShell scripts) PowerShell 5.1 scripts containing Korean must be saved as UTF-8 **with BOM** (without BOM they are read as ANSI and patterns break).
- In bulk text replacement `awk -v` and perl substitutions interpret backslash escapes. Pass bodies via file/stdin and check for control characters (`\x07` etc.) afterwards (`closed/process/issue-text-escape-mangled`).
- Changes affecting latency are compared before/after with 10 release e2e runs (`closed/search_engine/next-sk-prespawn-slowed-first-result`).
- Always `cargo build` before e2e. `cargo test` does not refresh the root `target\debug\skim-search.exe` (`closed/diagnostics/e2e-uses-stale-binary/70b8d276`).
- Screen capture uses only `PrintWindow`. `CopyFromScreen` captures overlapping windows (the user's screen) (`closed/diagnostics/capture-includes-overlapping-windows/03e79abe`).
- In PowerShell pass null strings to Win32 APIs as `[NullString]::Value` (`$null` becomes `""`).
- In PowerShell 5.1 read logs with `Get-Content -Encoding UTF8` (`closed/diagnostics/log-read-mojibake-in-powershell/9dd13927`).
- Edit files containing Windows paths with the Edit tool, not sed/awk (`closed/process/script-edit-backslash-mangled/fd494e04`). Inline Python in a bash heredoc also interprets `\b`, `\n` etc. in normal strings, so replacements containing backslashes use a raw-string script file or the Edit tool and check `count == 1` (`closed/process/heredoc-python-escape-mangled/51cebecc`).
- Verify scripts from a cwd outside the repository (e.g. `%TEMP%`).
- bash folder loops use `shopt -s nullglob` (`closed/process/rename-script-empty-glob-noise/48d4b36b`).
- Multi-step PowerShell e2e is written as a script file; unset env with `$env:X = $null` (`closed/process/powershell-command-blocked-by-safety-check/42fdc1e3`).
