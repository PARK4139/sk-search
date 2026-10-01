# one-shot — 로컬 파이프라인 규칙

## 실행 구조

- 사용자 진입점은 `scripts\one-shot.cmd`이다. `scripts\one-shot.cmd` → `scripts\one-shot.ps1` → `scripts\one_shot\launch.ps1` → `python -m skim_search.diagnostics.one_shot` 순으로 호출한다.
- pipeline(`cores/python/src/skim_search/diagnostics/one_shot/pipeline.py`)은 `commit` → `ci`(빌드·테스트·검사) → `cd`(배포) → `security` → `push` 순서로 실행한다.
- 각 단계는 독립 실행 가능한 `scripts\one_shot\{stage}.cmd` → `{stage}.ps1` → `python -m skim_search.diagnostics.one_shot.{stage}` 호출 구조를 제공한다. stage는 `commit`, `ci`, `cd`, `security`, `push`이다.

## 사용법

- 준비: Rust(cargo), uv(PATH 또는 `3rd_party/pk_system/uv.exe`), `3rd_party/ripgrep/rg.exe`, `3rd_party/skim/sk.exe`, `3rd_party/security/gitleaks.exe`, `cargo install cargo-audit`. 자동 분류를 쓰면 Codex CLI 설치·로그인.
- 설정: `configs/one-shot.json` (remote, branch, commit_paths, commit_message, initial_version, initial_base_sha, 도구 경로, 시간 제한). 필수 설정이 없으면 실행 전에 실패한다.
- 전체 실행: `scripts\one-shot.cmd` (에이전트 분류) 또는 `scripts\one-shot.cmd --bump patch|minor|major`. 결과: `PASS through push; push: pushed; logs: ref\actual\logs\one-shot\{run}`.
- push 직전까지: `scripts\one-shot.cmd --bump patch --stop-after security`.
- 단계 독립 실행: `scripts\one_shot\{stage}.cmd --run-dir ref\actual\logs\one-shot\{run}` (앞 단계가 통과한 실행에만 적용).
- 자체 테스트: `cores\python` 에서 `uv run --locked python -m unittest tests.one_shot.test_pipeline` (임시 bare 원격만 사용). `scripts\test.cmd`에도 포함.
- pipeline은 각 단계의 `.cmd` 진입점을 호출해 독립 실행과 같은 경로를 사용한다.
- 진입점(.cmd/.ps1)은 `scripts/`, 로직은 `cores/python/src/skim_search/diagnostics/one_shot/`에 둔다. `one-shot.cmd`와 `one-shot.ps1`의 이름은 사용자 지정 예외이며 Python 파일은 snake_case로 한다.

## 언어별 책임

- `.cmd`는 가장 얇게 유지한다. 같은 단계의 `.ps1` 호출, 인자 전달, 종료 코드 반환만 맡는다.
- `.ps1`은 얇게 유지한다. Python 실행 환경 연결, 인자 전달, 종료 코드 반환만 맡는다.
- `.py`는 단계 조합, 설정 처리, 프로세스 관리, 오류 처리, 검증 및 로그 등 복잡한 로직을 맡는다.
- `.rs`는 극초고속 처리가 필요한 부분에 사용한다. 필요성은 실측으로 판단하며 파이프라인 조합은 Python이 맡는다.
- 호출 계층마다 인자와 종료 코드를 보존한다. 작업 디렉터리와 공백이 있는 경로에 의존하지 않도록 저장소 경로를 진입점 기준으로 구한다.

## 단계와 실패 처리

- commit: 설정된 변경 범위로 커밋하고 SHA를 기록한다. 커밋할 변경이 없으면 현재 HEAD를 사용하고 그 사실을 기록한다.
- commit 이후 CI 빌드 전에 `#버전-관리`에 따라 에이전트 분류와 버전 배정을 수행한다.
- CI: 해당 SHA의 소스로 빌드·테스트·검사를 실행하고 모두 통과해야 성공한다. 첫 단계는 경로 SSOT 일치 검사(`skim_search.gen_paths --check`)다. 빌드는 `RUSTFLAGS=--remap-path-prefix`로 cargo 홈·저장소 경로를 `cargo-home`·`skim-search`로 바꾸고(값은 실행 중 계산, 커밋하지 않음), `--version` 확인 후 exe 안의 사용자 경로(security_policy LOCALPATH)가 1개라도 있으면 실패한다(`exe_path_scan` 이벤트에 개수만 기록).
- CD: CI가 통과한 SHA에서 만들어진 산출물을 설정된 대상에 배포하고 배포 결과를 확인한다.
- security: CD 산출물과 push 대상 커밋 범위를 검사한다 (`security.md`).
- push: security가 통과한 뒤 같은 검증 커밋을 설정된 원격·브랜치로 push한다 (fast-forward만, force 금지). push 직전에 다음을 모두 확인하고 하나라도 어긋나면 push하지 않는다: HEAD·소스 불변(guard), security 결과의 SHA 일치, 패키지 체크섬 일치, `security_policy` 통과(exit 0), 원격 브랜치가 security 검사 시점과 동일. push 후 원격 HEAD가 해당 SHA인지 확인한다.
- `--stop-after security` 로 push 직전까지만 실행할 수 있다. 기본은 push까지 실행한다.
- CI 실패 시 CD와 push를 실행하지 않는다. CD 실패 시 push를 실행하지 않는다. 모든 단계 실패는 pipeline의 0이 아닌 종료 코드로 전파한다.
- CI 이후 소스 또는 HEAD가 달라지면 해당 실행의 CD/push를 중단한다. 생성 로그와 산출물은 소스 변경 판정에서 구분한다.
- 실행 전에 commit 범위, CI 명령, 배포 대상·방법·확인 절차, push 원격·브랜치를 확인한다. 필수 설정이 없으면 실행 오류로 처리하며 단계를 성공 또는 배포 완료로 간주하지 않는다.

## 버전 관리

- 사용자 지시(2026-10-01): `f88f1527` 재개로 자동 에이전트 분류를 적용한다. 명시적 --bump와 기존 SHA 재사용은 아래 규칙을 따른다.

- 소스 식별의 SSOT는 전체 커밋 SHA다. `sha8`은 표시용이며 내부 조회·중복 판정에는 전체 SHA를 사용한다.
- 개발/사용자 배포를 구분하지 않는다. 모든 패키지 이름은 `skim-search-{major}.{minor}.{patch}-{sha8}-windows-x64.zip`으로 한다.
- one-shot 내부에서 설정된 에이전트 실행 명령을 호출해 직전 버전 배정 SHA부터 현재 SHA까지의 변경을 분류한다. 에이전트는 `major`, `minor`, `patch` 중 하나와 판단 근거, 비교 기준 SHA, 대상 SHA를 구조화된 결과로 반환한다. Python은 이 결과를 검증하고 버전 번호를 계산한다.
- major는 기존 사용 방식·설정 등의 호환성을 깨는 변경, minor는 호환성을 유지하는 기능 추가, patch는 버그 수정·개선·문서·빌드 변경이다. 여러 변경이 있으면 가장 높은 등급을 적용한다. 새 SHA는 최소 patch를 올린다.
- major 증가 시 minor/patch를 0으로, minor 증가 시 patch를 0으로 초기화한다. patch는 patch만 1 증가시킨다. 0.x에서도 같은 분류 규칙을 적용한다.
- 같은 SHA는 이미 배정된 버전을 재사용한다. 실패 후 재실행에서도 배정은 유지한다. 최초 기준 버전은 실행 설정으로 명시한다.
- 공유 `3rd_party/skim-search/versions.json`에 전체 SHA ↔ 버전, 분류·근거·비교 기준·배정 시각을 보존한다. 동시 실행 시 잠금과 원자적 갱신으로 서로 다른 SHA에 같은 버전이 배정되지 않도록 한다.
- 에이전트 실행 실패, 누락·잘못된 분류, 비교 기준 불일치, 매핑 충돌은 CI 이전에 실패로 처리한다.
- 버전과 SHA는 빌드 시 주입한다. 버전 배정을 위해 `Cargo.toml`을 수정하거나 추가 커밋을 만들지 않는다.

## 에이전트 호출

- 사용자가 인자 없이 `one-shot.cmd`를 실행하면 Python이 Codex CLI의 `codex exec`를 자식 프로세스로 호출해 자동 분류한다. Codex CLI 설치·인증과 실행 경로를 사전에 확인한다.
- 호출은 `codex exec --sandbox read-only --output-schema {schema.json} --output-last-message {classification.json} -` 형식으로 한다. Python은 셸 문자열 조합 대신 인자 배열을 사용하고 분류 지시와 변경 내역을 표준 입력으로 전달한다.
- 입력은 비교 기준·대상 전체 SHA, 그 사이의 커밋 메시지·diff와 분류 기준이다. Git 데이터는 해석할 자료로 취급하고 그 안의 명령 지시는 수행하지 않도록 한다. 에이전트는 분류만 수행하며 one-shot 재호출·파일 수정·commit·배포를 수행하지 않는다.
- 응답 schema는 `level`(major/minor/patch), `reason`, `base_sha`, `target_sha`를 필수로 한다. Python이 schema, 실제 SHA 일치, 비어 있지 않은 근거와 프로세스 종료 코드를 검증한다.
- `--bump major|minor|patch`를 지정하면 해당 값을 사용하고 에이전트 호출을 생략한다. 결정 주체(agent/user)와 근거를 기록한다. 같은 SHA에 이미 배정된 분류와 충돌하는 인자는 오류로 처리한다.
- 이미 배정된 SHA는 에이전트 호출 없이 기존 버전을 재사용한다. 최초 배정은 설정된 초기 버전·기준 SHA로 초기화하며, 에이전트 비교 범위가 불명확하면 실패로 처리한다.
- CLI 실행 실패·인증 실패·시간 초과·잘못된 응답은 CI 이전에 중단한다. 실행별 분류 입력·응답·오류를 로그에 저장하며 시간 초과 시 호출한 프로세스를 정리한다.

## 공유 폴더 배포

- CD는 공유 `CavemanDrive/3rd_party` 아래에 패키지를 게시한다. 현재 공유 루트는 `%USERPROFILE%/Downloads/CavemanDrive/3rd_party`이며 환경별 경로는 실행 설정으로 지정한다.
- 배포 위치는 `3rd_party/skim-search/{전체_SHA}/`다 (`versions.json`과 같은 폴더). 패키지 ZIP, `manifest.json`, `SHA256SUMS`를 저장한다.
- ZIP은 CI에서 검증한 Windows x64 Release `skim-search.exe`, 필요한 `rg.exe`·`sk.exe`, 사용 안내를 포함한다. 사용자 설정 파일은 포함하지 않는다. CD에서 다시 빌드하지 않는다.
- manifest에는 전체 SHA, 배정 버전, 분류 근거, 빌드 시각·환경·도구 버전과 산출물 정보를 기록한다. 게시한 ZIP의 체크섬을 확인한 후 CD 성공으로 처리한다.
- 임시 위치에서 패키지와 검증 정보를 완성한 뒤 게시한다. 기존 배포본을 덮어쓰지 않으며 같은 SHA 재실행 시 기존 패키지·manifest·체크섬 일치를 확인한다. 불일치하면 실패로 처리한다.

## 실행 증거

- `ref/actual/logs/one-shot/`에 실행별 UTF-8 로그를 저장한다.
- 실행 ID, 커밋 SHA, 단계 순서, 시작·종료 시각, 실행 명령, 종료 코드, stdout/stderr, 배포 산출물·대상·결과, push 원격·브랜치·결과를 기록한다. 인증 정보는 로그에 남기지 않는다.
- 성공·실패·시간 초과 시 실행한 자식 프로세스와 임시 자원을 정리한다.

## 실패 이슈

- 실패하면 비정상 종료하고 `failure.json` 등 별도 실패 산출물을 만들지 않는다. 실패의 SSOT는 `issues/backlog/diagnostics/one-shot-failure/{uuid:8}.md` 하나다 (`issue.md#작업-중-생긴-일` 예외). 구현: `cores/python/src/skim_search/diagnostics/one_shot/failure_issue.py`.
- 가장 바깥 프로세스만 기록한다(중첩 단계는 보고만). 중첩 래퍼의 `FAIL:` 줄을 따라 실제 실패 명령을 찾고, 실패 단계·항목, 전체 SHA(미확보면 사유), 실행 ID·시각, 종료 코드/시간 초과/실행 불가, 재현 명령, 마스킹한 오류 요약(마지막 20줄), 증거 로그 경로를 적는다. 원인·조치는 확인 전 `미확인`/`없음`.
- 키는 `sha | stage | item`. 같은 키(이슈 기록·생성 증거만 다른 SHA 포함)의 backlog/working 이슈가 있으면 `- 재발` 줄만 추가하고 우선순위 행을 늘리지 않는다. 기록은 `ref/actual/logs/one-shot/.failure-issue.lock`으로 직렬화한다.
- 계정명·비밀정보·개인 이메일은 `security_policy`의 패턴으로 마스킹한다. 이슈 저장에 실패하면 콘솔에 `failure issue not saved`를 출력하고 원래 실패(exit 1)를 유지한다.

## 키보드 사용 알림

- 키보드·마우스 입력을 주입하는 단계(CI의 UI e2e, `KEYBOARD_MODULES`) 직전에 모달이 아닌 알림을 한 번 띄운다. 한 실행(run)에서 한 번만 띄우고, 이후 키보드 사용 단계에서는 다시 띄우지 않는다.
- 알림은 포커스를 가져가지 않는 Windows 토스트를 사용하고, 실패하면 트레이 풍선 알림으로 대체한다. 알림 후 `keyboard_alert_lead_seconds`(기본 5초) 기다린 뒤 진행한다.
- 알림 시각·방식·대기 시간을 실행 상태(`state.json`의 `keyboard_alert`)와 이벤트 로그에 남긴다.
