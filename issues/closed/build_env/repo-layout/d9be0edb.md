# title
저장소 구조 정리: 언어별 cores, 루트 scripts·configs·target 분리

# pre-condition
- 다른 작업자의 진행 중 변경(`cores/tests/`, `skim_tests`, `cores/scripts/one_shot/`, `issue_ids.py`)이 먼저 커밋되어 있다.
- 동시에 실행 중인 빌드·테스트가 없다 (`cores/target/` 삭제·이동 시 파일 잠금).

# steps
근거: 사용자 요청 (2026-10-01): 빌드 출력은 루트 `target/`(Rust 관례 이름), 실행 스크립트는 루트 `scripts/`, 중요 설정은 `configs/`, `cores/`에는 로직만, Rust·Python은 각 언어 관례(Rust `src/`, Python src layout), 테스트는 언어 프로젝트 안, `.cargo`는 A안(루트 `.cargo/config.toml`), Rust 테스트는 1안(`cores/rust/tests/` 유지)
목표 구조:
```text
.cargo/config.toml          [build] target-dir = "target"
configs/                    one-shot.json, security-exceptions.json
scripts/                    test.cmd/.ps1, one-shot.cmd/.ps1, security-policy.cmd/.ps1, issue-id.cmd, one_shot/(단계별 .cmd/.ps1, launch.ps1)
cores/common/               언어 중립 SSOT (build_env/paths-ssot)
cores/rust/                 Cargo.toml, Cargo.lock, common/, app/(build.rs, src/{family}/, ui/), tests/
cores/python/               pyproject.toml, uv.lock, src/skim_search/(diagnostics/: one_shot/, security_policy.py, issue_ids.py), tests/(support/, e2e/, one_shot/, security/, issue_ids/), benchmarks/
target/                     cargo 출력 (git 제외), one-shot 격리 빌드는 target/one-shot/
```
1. `cores/target/`을 삭제한다 (사용자 결정: 이동하지 않고 삭제).
2. 루트 `.cargo/config.toml`을 만들고 Cargo 워크스페이스를 `cores/rust/`로 옮긴다.
3. Python 코드를 `cores/python/`(src layout, 패키지 `skim_search`)으로 옮기고, 벤치마크를 `benchmarks/`로 분리한다.
4. `.cmd`/`.ps1` 진입점을 `scripts/`로, 설정 파일을 `configs/`로 옮긴다. 진입점은 `%~dp0..`로 루트를 계산한다.
5. 경로 참조(스크립트, `pipeline.py`, `launch.ps1`, `test_pipeline.py`, `.gitignore`, `security_policy.py`)와 `rules/*.md`(`families.md` 구현 위치 열 포함), `README.md`를 갱신한다.
6. 전체 검증을 실행한다.

# actual result
PASS (2026-10-02, commit 0841838).
- 이동: `cores/{Cargo.*,app,common,tests}` → `cores/rust/`; `cores/tests/py` + `cores/scripts` → `cores/python/` (src layout, 패키지 `skim_search`: `diagnostics/one_shot/`, `security_policy.py`, `issue_ids.py`; `tests/{support,e2e,one_shot,security,issue_ids}`, `benchmarks/{rg,sk}`); 진입점 `.cmd/.ps1` → `scripts/` (`test.cmd`, `one-shot`, `security-policy`, `issue-id`, `one_shot/{stage}`, launch.ps1는 `python -m` 모듈 실행); `config.json` → `configs/one-shot.json`, `security-exceptions.json` → `configs/`; 루트 `one_shot.py` 제거(`skim_search.diagnostics.one_shot.__main__`). `cores/target/`(14GB) 삭제, 루트 `.cargo/config.toml` `target-dir = "target"`. pyproject 이름 `skim-search`, `uv lock` 갱신.
- 경로 조정: 앱 로그 `CARGO_MANIFEST_DIR/../../..`, pipeline(ROOT·단계 스크립트·CI cwd `cores/rust`·모듈 실행 cwd `cores/python`·one-shot 빌드 `target/one-shot`·cargo-audit/pip-audit 경로), security_policy·issue_ids ROOT/예외 파일, 테스트 경로 SSOT `tests/support/paths.py`, test_pipeline 임시 저장소 구성(단독 push 진입 테스트는 PYTHONPATH로 임시 복사본 import → 실제 저장소에 이슈 생성 없음).
- 검증:
  - `cores/rust`에서 `cargo build` → 루트 `target/debug/skim-search.exe`, `cores/target` 없음.
  - `%TEMP%`에서 `scripts\test.cmd` 전체 `ALL PASSED` exit 0 (`ref/actual/logs/test-cmd-full.log`): cargo build/test/clippy, Python 33개(pipeline 25, issue_ids 7, security policy 1, `python-unittest.log`), e2e_detection 11/11, e2e_ui 21/21, bench.
  - `%TEMP%`에서 `scripts\security-policy.cmd` exit 0, `scripts\issue-id.cmd --help`, `scripts\test.cmd quick` PASS.
  - `%TEMP%`에서 `scripts\one-shot.cmd --bump minor` → run `20261002-024248-5e7cfc0e` 전 단계 passed, `target/one-shot/release/skim-search.exe --version` = `skim-search 0.5.0 0841838…`, origin/main == 0841838.
  - `rules/*.md`, `README.md`, `.gitignore`에 옛 경로 없음 (`git grep`, closed 이슈·`ref/closed` 과거 기록 제외).
- 결정: e2e 스크립트 이름은 테스트 프레임워크 대상이 아니라 실행 스크립트라 `tests/e2e/detection.py`, `ui.py` (test_ 접두사 없음). security self-test는 모듈에 두고 `tests/security/test_policy.py`가 호출.

이전 상태: 워크스페이스 `cores/`, 빌드 출력 `cores/target/`(14GB), 스크립트 `cores/scripts/`와 루트(`test.cmd`, `one-shot.*`, `one_shot.py`), Python 프로젝트 `cores/tests/py`. 경로 계산이 5곳에 흩어져 있음(`parents[N]` 깊이 각각 다름).
배경: one-shot CI가 공유 `cores/target/`을 쓰다가 동시 빌드에 exe가 덮어써짐 (`closed/diagnostics/one-shot-concurrent-build-overwrites-exe`).

# expected result
- `cores/rust/`에서 cargo를 실행하면 루트 `target/`에 출력된다. `cores/target/`이 없다.
- one-shot은 `target/one-shot/`에서 빌드하고, `--version` 검사가 PASS한다.
- 루트에 실행 파일(`.cmd`/`.ps1`/`.py`)이 없다. `scripts/`에는 진입점만 있고, 다른 폴더에서 실행해도 동작한다.
- `cores/`에는 `common/`, `rust/`, `python/`만 있다. Rust는 `src/`, Python은 `src/skim_search/` 관례를 따르고, 테스트는 각 언어 프로젝트 안에 있다.
- `configs/`에 one-shot 설정과 보안 예외 목록이 있고, 코드가 이 위치에서 읽는다.
- `scripts\test.cmd` 전체, `test_pipeline`, security policy 테스트, `issue_ids` 테스트가 모두 PASS한다. 결과를 `ref/actual/logs/`에 남긴다.
- `rules/*.md`, `README.md`, `.gitignore`에 옛 경로(`cores/target`, `cores/scripts`, `cores/tests/py`, 루트 `test.cmd`)가 남아 있지 않다 (`ref/closed/`, closed 이슈의 과거 기록은 제외).
- `security_policy` 검사가 exit 0이다.

# label
SQA_sk_0_0_0

# environment
OS: Windows 11 Pro
hostname: TBD

# 담당자
