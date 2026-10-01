@echo off
rem skim-search: run every test (build, cargo test, clippy, e2e, benchmarks).
rem Usage: test.cmd [quick]     quick = build + cargo test + clippy only
rem Exit code: 0 = all passed, 1 = something failed. Logs: ref\actual\logs\
rem Python checks run from the uv project cores\tests\py (uv: PATH, else 3rd_party\pk_system\uv.exe).
rem The e2e steps launch skim-search windows and send Ctrl+Shift+F; do not type during the run.
setlocal
set "ROOT=%~dp0"
set "LOGS=%ROOT%ref\actual\logs"
set "PYPROJ=%ROOT%cores\tests\py"
set "FAILED="
if not exist "%LOGS%" mkdir "%LOGS%"

set "UV="
for /f "delims=" %%i in ('where uv 2^>nul') do if not defined UV set "UV=%%i"
if not defined UV if exist "%ROOT%..\..\3rd_party\pk_system\uv.exe" set "UV=%ROOT%..\..\3rd_party\pk_system\uv.exe"
set "PY="%UV%" run --project "%PYPROJ%" python -m"

pushd "%ROOT%cores"

echo [run ] cargo build (debug + release)
cargo build > "%LOGS%\cargo-build.log" 2>&1 || set "FAILED=%FAILED% [cargo build]"
cargo build --release >> "%LOGS%\cargo-build.log" 2>&1 || set "FAILED=%FAILED% [cargo build --release]"

echo [run ] cargo test
cargo test > "%LOGS%\cargo-test.log" 2>&1 || set "FAILED=%FAILED% [cargo test]"

echo [run ] cargo clippy
cargo clippy --all-targets -- -D warnings > "%LOGS%\cargo-clippy.log" 2>&1 || set "FAILED=%FAILED% [cargo clippy]"

if /i "%~1"=="quick" goto summary

if not defined UV (
  echo [FAIL] uv not found ^(PATH or 3rd_party\pk_system\uv.exe^)
  set "FAILED=%FAILED% [uv missing]"
  goto summary
)

echo [run ] e2e rg/sk detection
%PY% skim_tests.e2e_detection --no-build > "%LOGS%\e2e-detection.out.log" 2>&1 || set "FAILED=%FAILED% [e2e detection]"

echo [run ] e2e UI (release exe, UI Automation)
%PY% skim_tests.e2e_ui --no-build > "%LOGS%\e2e-ui.out.log" 2>&1 || set "FAILED=%FAILED% [e2e UI]"

echo [run ] bench rg / sk
%PY% skim_tests.bench_rg > "%LOGS%\bench-rg.out.log" 2>&1 || set "FAILED=%FAILED% [bench rg]"
%PY% skim_tests.bench_sk > "%LOGS%\bench-sk.out.log" 2>&1 || set "FAILED=%FAILED% [bench sk]"

:summary
popd
echo.
if defined FAILED (
  echo FAILED:%FAILED%
  echo logs: %LOGS%
  exit /b 1
)
echo ALL PASSED  (logs: %LOGS%)
exit /b 0
