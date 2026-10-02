# security — push 전 보안 위험 검사

모든 push(수동, one-shot pipeline)에 적용한다. 원격 `PARK4139/sk-search`는 **public**이다.

> **필수 (예외 없음)**: 모든 push는 직전에 `security_policy` exit 0 이어야 한다. 비상 백업(`--urgent-backup`)·단계 스위치(`configs/one-shot.json` `stages`)·어떤 인자로도 끌 수 없으며, 끄는 설정·옵션을 만들지 않는다. 비상 모드가 건너뛰는 것은 CI·CD·security 단계(빌드·테스트·gitleaks·의존성 감사)뿐이다 (사용자 결정 2026-10-02, `diagnostics/one-shot-urgent-backup`).
구현·자동화: `diagnostics/one-shot-security-gate`. pipeline 연결은 `one-shot.md#단계와-실패-처리`.

## 검사 범위

- push 대상 커밋 = `<remote>/<branch>..<push할 ref>` 의 모든 커밋과 그 커밋들이 추가·변경한 blob, 커밋 메타데이터(작성자·커미터 이름/이메일, 메시지).
  - 원격 기준을 확보하지 못하면(fetch 실패, 원격 ref 없음) 통과로 처리하지 않는다. 신규 저장소는 push할 ref의 전체 이력을 검사한다.
  - `--all` 로 검사하지 않는다. 로컬 전용 ref(stash, 재작성 전 이력)는 push되지 않으며 오탐을 만든다.
- 배포 패키지(ZIP)를 함께 올리거나 게시하면 패키지 내용도 검사한다.
- 작업 폴더의 commit되지 않은 변경은 push 대상이 아니다. 다른 작업자의 진행 중 파일을 push 범위에 섞지 않는다.

## 차단 항목 (탐지 시 push 금지)

| ID | 항목 | 예 |
|----|------|----|
| SEC-SECRET | 비밀정보 | 토큰(`ghp_`, `github_pat_`, `xox?-`, `AKIA…`, `AIza…`), 개인키 블록, 비밀번호·API key 대입 |
| SEC-LOCALPATH | 로컬 사용자 경로·계정명 | `C:\Users\<name>\`, `C:/Users/<name>/`, `/c/Users/<name>/` → 문서에는 `%USERPROFILE%` 사용. Windows 공용 프로필(`Public`, `Default`, `Default User`, `All Users`)은 계정이 아니므로 허용 |
| SEC-EMAIL | 개인 이메일 | 커밋 메타데이터·추가된 줄의 이메일 중 허용 목록(`*@users.noreply.github.com`, `noreply@github.com`, `noreply@anthropic.com`(Co-Authored-By), 문서·테스트 예약 도메인 `example.*`, `*.invalid`, `*.test`, `*.example`) 외 |
| SEC-PATH | 공개 금지 경로 | `ref/actual/logs/`, `.venv/`, `target/`, `__pycache__/`, `*.env`, 사용자 설정 파일(`settings.json` 실사용본) |
| SEC-VULN | 의존성 취약점 | `cores/rust/Cargo.lock`, `cores/python/uv.lock` 에서 설정 심각도 이상 |
| SEC-TOOL | 검사 실패 | 검사 미실행, 도구 오류, 시간 초과, 취약점 데이터 미확보 |

- 경고(차단 아님): 5MB 초과 파일, 사용자 작업공간·호스트명 등 개인 식별 가능 문자열.
- 화면 캡처는 skim-search 창만 담는다 (`environment.md#스크립트-작성-주의`). 다른 창이 섞일 수 있는 캡처는 commit하지 않는다.
- 이미지 속 글자는 자동 검사(security_policy, gitleaks) 범위 밖이다. 캡처를 commit하기 전에 화면에 보이는 경로(검색 경로 입력란, 결과·토스트)에 계정명이 없는지 눈으로 확인한다. e2e workspace는 `%PUBLIC%\skim-search-e2e`(골든 샘플 트리 복사본)를 쓰고 `%TEMP%`·`%USERPROFILE%` 아래를 화면에 띄우지 않는다 (`diagnostics/screenshot-local-user-path`).

## 실행

| 검사 | 진입점 | 담당 항목 |
|------|--------|-----------|
| 저장소 정책 검사 | `scripts\security-policy.cmd [--remote origin] [--branch main] [--ref HEAD]` | SEC-SECRET(기본 패턴), SEC-LOCALPATH, SEC-EMAIL, SEC-PATH, SEC-TOOL, WARN-LARGE |
| 비밀정보·의존성 취약점 | pipeline `security` 단계 (`cores/python/src/skim_search/diagnostics/one_shot/pipeline.py`) | SEC-SECRET(gitleaks), SEC-VULN(cargo-audit, pip-audit), SEC-TOOL |

- 수동 push 전: `scripts\security-policy.cmd` 실행 → 종료 코드 0일 때만 push. 1 = 차단 항목 탐지, 2 = 검사 실행 불가.
- 자체 테스트: `scripts\security-policy.cmd --self-test` (또는 `cores/python/tests/security/test_policy.py`) (임시 저장소만 사용, 실제 origin에 push하지 않음).
- one-shot pipeline 의 push 단계는 push 직전에 `security_policy` 를 호출하고 exit 0 일 때만 push한다 (`one-shot.md#단계와-실패-처리`). `--urgent-backup`·`stages` 설정과 무관하게 항상 실행된다.

## 예외

- 예외 목록 `configs/security-exceptions.json` 에 ID·rule·대상 경로(glob)·사유·만료일(`expires`, YYYY-MM-DD)을 적는다. 만료된 예외는 무시한다.
- 적용된 예외는 검사 결과에 남긴다.

## 결과와 기록

- 결과는 전체 SHA(검사 대상 범위)와 묶는다. 검사 후 대상 SHA·패키지가 바뀌면 결과를 무효화한다.
- 로그·보고서: `ref/actual/logs/security/` (로컬, git 미포함). 탐지 위치·ID·심각도·차단 여부를 남기고 **비밀정보 원문은 기록하지 않는다** (마스킹).
- 통과는 "설정된 검사 항목에서 탐지 없음"을 뜻한다. 검사 범위와 미지원 항목을 함께 보고하며 "보안 위험 없음"으로 표기하지 않는다.
- 탐지되면 수정 후 재검사한다. 이미 커밋된 이력에 있으면 push 전에 이력에서 제거한다(재작성은 push 전 로컬 커밋에 한함). 이미 원격에 올라간 비밀정보는 즉시 폐기·교체한다.
