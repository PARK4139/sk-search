# environment — 개발·검증 환경

경로의 `CavemanDrive/` = `%USERPROFILE%\Downloads\CavemanDrive\`.

## 외부 도구

| 도구 | 경로 | 버전 | 출처 / 비고 |
|------|------|------|-------------|
| rg | `CavemanDrive/3rd_party/ripgrep/rg.exe` | 15.2.0 (+pcre2) | Codex 번들 복사본 (OpenAI 서명). PATH 미등록 (`closed/build_env/rg-executable-missing/5797d05d`) |
| sk | `CavemanDrive/3rd_party/skim/sk.exe` | 5.6.6 | winget `skim-rs.skim` 복사본. winget 원본은 PATH 등록됨 |
| ffmpeg | `CavemanDrive/3rd_party/LosslessCut-win-x64/resources/ffmpeg.exe` | — | showreel 프레임 추출 전용 |
| Rust | stable-x86_64-pc-windows-msvc | cargo 1.98.1 | |

- 앱의 rg/sk 탐지 순서: 설정 경로 → PATH → fallback (exe 폴더와 상위 폴더의 `<tool>.exe`, `3rd_party/{ripgrep,skim}/<tool>.exe`). 개발 중에는 fallback으로 `3rd_party`를 찾는다.
- 버전 기록: `ref/actual/logs/env-tools.log`.
- showreel 프레임 추출:
  `3rd_party/LosslessCut-win-x64/resources/ffmpeg.exe -i ref/showreel/issue-full-flow.mp4 -vf fps=1 ref/showreel/frames/f%02d.png`
- Python 없음. 자동화 스크립트는 bash 또는 PowerShell로 작성한다.

## cargo 설정

- 사내망 schannel `CRYPT_E_NO_REVOCATION_CHECK` 회피: `%USERPROFILE%\.cargo\config.toml`에 `[http] check-revoke = false` (이 PC 한정, repo에는 두지 않음).
- 사내 인증서 폐기 목록(CRL) 접근이 열리면 이 설정을 제거한다 (`closed/build_env/crates-io-ssl-revocation-check-failure/6d33b85a`).

## 증거 경로

| 종류 | 경로 |
|------|------|
| 런타임 로그 | `ref/actual/logs/skim-search.log` (append, `<unix_ms> [<event>] <detail>`, UTF-8). 기록 항목은 handover §30 (debug build 기준) |
| 빌드·테스트 로그 | `ref/actual/logs/cargo-{build,test,clippy}.log` |
| 도구 버전 | `ref/actual/logs/env-tools.log` |
| 화면 증거 | `ref/actual/screenshot/frames/{sub family}_{우선순위}[_{case}].png` |

- `ref/actual/`은 근거가 아니다 (`sources.md`).

## 환경변수

| 변수 | 용도 | 기본값 |
|------|------|--------|
| `SKIM_SEARCH_LOG` | 런타임 로그 경로 override | `ref/actual/logs/skim-search.log` (`cores/common` 기준 상대 경로) |
| `SKIM_SEARCH_SETTINGS` | 설정 파일 경로 override | `%APPDATA%\skim-search\settings.json` |

설정 파일 필드: `rg_path`, `sk_path` (없거나 parse 실패 시 기본값, 사유는 로그에 기록).

## 검증 스크립트

`cores/tests/scripts/`:

| 스크립트 | 용도 |
|----------|------|
| `run_capture.ps1 -Exe <exe> -Out <png>` | 앱 실행 → 창 제목 `skim-search`로 찾기 → `PrintWindow`로 해당 창만 캡처 → WM_CLOSE 종료 → exit code 출력 |
| `e2e_detection.ps1` | `cargo build` 후 rg/sk 탐지 case A(설정)/B(PATH)/C(없음) 실행·캡처 |

## 스크립트 작성 주의

기록 issue의 재발 방지책:

- 테스트 스크립트는 자신이 시작한 프로세스·창(PID, 또는 실행 전후 HWND 비교로 식별)만 닫거나 종료한다. 제목·이름 일치로 대상을 고르지 않는다. 사용자 기본 앱으로 파일을 여는 검증은 opt-in (`e2e_ui.ps1 -SystemOpen`) (`closed/process/e2e-closed-user-vscode`).
- 키 입력 주입(`keybd_event`)은 skim-search가 foreground일 때만 보낸다 (사용자 앱으로 입력 금지). 전역 단축키 검증은 예외(등록된 hotkey가 가로챔).
- PowerShell 5.1 스크립트에 한글이 있으면 UTF-8 **BOM**으로 저장한다 (BOM 없으면 ANSI로 읽혀 패턴이 깨짐).
- 텍스트 일괄 치환 시 `awk -v`, perl 치환문은 백슬래시 escape를 해석한다. 본문은 파일/stdin으로 넘기고, 치환 후 제어문자(`\x07` 등)를 검사한다 (`closed/process/issue-text-escape-mangled`).
- latency에 영향을 주는 변경은 release e2e 10회 측정으로 전후를 비교한다 (`closed/search_engine/next-sk-prespawn-slowed-first-result`).

- e2e 전 항상 `cargo build`. `cargo test`는 `target\debug\skim-search.exe`를 갱신하지 않는다 (`closed/diagnostics/e2e-uses-stale-binary/70b8d276`).
- 화면 캡처는 `PrintWindow`만 사용. `CopyFromScreen`은 겹친 다른 창(사용자 화면)을 캡처한다 (`closed/diagnostics/capture-includes-overlapping-windows/03e79abe`).
- PowerShell에서 Win32 API에 null 문자열은 `[NullString]::Value` (`$null`은 `""`로 전달됨).
- PowerShell 5.1에서 로그 읽기는 `Get-Content -Encoding UTF8` (`closed/diagnostics/log-read-mojibake-in-powershell/9dd13927`).
- Windows 경로가 포함된 파일 수정은 sed/awk 대신 Edit 도구 사용 (`closed/process/script-edit-backslash-mangled/fd494e04`).
- 스크립트 검증은 저장소 밖 cwd(예: `%TEMP%`)에서 실행한다.
- bash 폴더 순회는 `shopt -s nullglob` (`closed/process/rename-script-empty-glob-noise/48d4b36b`).
- 여러 단계 PowerShell e2e는 스크립트 파일로 작성하고, env 해제는 `$env:X = $null` (`closed/process/powershell-command-blocked-by-safety-check/42fdc1e3`).
