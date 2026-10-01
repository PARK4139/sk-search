@echo off
rem skim-search: run every test (build, cargo test, clippy, Python tool tests, e2e, benchmarks).
rem Usage: scripts\test.cmd [quick]     quick = build + cargo test + clippy only
rem Exit code: 0 = all passed, 1 = something failed. Logs: ref\actual\logs\
rem Rust: cargo workspace cores\rust (output: target\ via .cargo\config.toml).
rem Python: uv project cores\python (uv: PATH, else 3rd_party\pk_system\uv.exe), run from that folder.
rem The e2e steps launch skim-search windows and send Ctrl+Shift+F; do not type during the run.
setlocal
for %%i in ("%~dp0..") do set "ROOT=%%~fi"
set "LOGS=%ROOT%\ref\actual\logs"
set "FAILED="
if not exist "%LOGS%" mkdir "%LOGS%"

set "UV="
for /f "delims=" %%i in ('where uv 2^>nul') do if not defined UV set "UV=%%i"
if not defined UV if exist "%ROOT%\..\..\3rd_party\pk_system\uv.exe" set "UV=%ROOT%\..\..\3rd_party\pk_system\uv.exe"
set "PY="%UV%" run --locked python -m"

pushd "%ROOT%\cores\rust"

echo [run ] cargo build (debug + release)
cargo build > "%LOGS%\cargo-build.log" 2>&1 || set "FAILED=%FAILED% [cargo build]"
cargo build --release >> "%LOGS%\cargo-build.log" 2>&1 || set "FAILED=%FAILED% [cargo build --release]"

echo [run ] cargo test
cargo test > "%LOGS%\cargo-test.log" 2>&1 || set "FAILED=%FAILED% [cargo test]"

echo [run ] cargo clippy
cargo clippy --all-targets -- -D warnings > "%LOGS%\cargo-clippy.log" 2>&1 || set "FAILED=%FAILED% [cargo clippy]"

popd
if /i "%~1"=="quick" goto summary

if not defined UV (
  echo [FAIL] uv not found ^(PATH or 3rd_party\pk_system\uv.exe^)
  set "FAILED=%FAILED% [uv missing]"
  goto summary
)

pushd "%ROOT%\cores\python"

echo [run ] path SSOT check (cores\common\paths.ini -^> skim_search constants)
%PY% skim_search.gen_paths --check > "%LOGS%\paths-check.log" 2>&1 || set "FAILED=%FAILED% [paths check]"

echo [run ] python tool tests (one-shot pipeline, issue IDs, security policy, paths)
%PY% unittest discover -s tests -t . -p "test_*.py" > "%LOGS%\python-unittest.log" 2>&1 || set "FAILED=%FAILED% [python unittest]"

echo [run ] e2e rg/sk detection
%PY% tests.e2e.detection --no-build > "%LOGS%\e2e-detection.out.log" 2>&1 || set "FAILED=%FAILED% [e2e detection]"

echo [run ] e2e UI (release exe, UI Automation)
%PY% tests.e2e.ui --no-build > "%LOGS%\e2e-ui.out.log" 2>&1 || set "FAILED=%FAILED% [e2e UI]"

echo [run ] bench rg / sk / paths
%PY% benchmarks.rg > "%LOGS%\bench-rg.out.log" 2>&1 || set "FAILED=%FAILED% [bench rg]"
%PY% benchmarks.sk > "%LOGS%\bench-sk.out.log" 2>&1 || set "FAILED=%FAILED% [bench sk]"
rem regression guard only (process-noise margin); the 50us target is measured with benchmarks.paths defaults
%PY% benchmarks.paths --limit-us 100 > "%LOGS%\bench-paths.out.log" 2>&1 || set "FAILED=%FAILED% [bench paths]"

popd

:summary
echo.
if defined FAILED (
  echo FAILED:%FAILED%
  echo logs: %LOGS%
  exit /b 1
)
echo ALL PASSED  (logs: %LOGS%)
exit /b 0
