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
미착수. 현재: 워크스페이스 `cores/`, 빌드 출력 `cores/target/`(14GB), 스크립트 `cores/scripts/`와 루트(`test.cmd`, `one-shot.*`, `one_shot.py`), Python 프로젝트 `cores/tests/py`. 경로 계산이 5곳에 흩어져 있음(`parents[N]` 깊이 각각 다름).
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
