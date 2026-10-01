# title
one-shot으로 commit → CI → CD → push 파이프라인 구현

# pre-condition
- 현재 버전 정책: 사용자 재개 지시에 따라 `diagnostics/one-shot-agent-classification/f88f1527`에서 자동 분류를 구현한다. 실제 push는 사용자 지시에 따라 차단한다.
- Git 저장소, Rust 빌드 도구, Python 및 PowerShell 실행 환경
- commit 범위, CI 명령, 배포 대상·방법·확인 절차, push 원격·브랜치 설정 필요

# steps
근거: 사용자 요청 (2026-10-01): one-shot 호출 구조와 commit → CI → 통과 시 CD → push 파이프라인; rules/one-shot.md
1. 루트에 `one-shot.cmd` → `one-shot.ps1` → `one_shot.py` 진입점을 구현한다.
2. `cores/scripts/one_shot/`에 commit, ci, cd, push 각각의 `.cmd` → `.ps1` → `.py` 실행 경로를 구현한다.
3. `.cmd`는 가장 얇게, `.ps1`은 얇게 유지하고 복잡한 로직은 Python에 작성한다. 극초고속 처리가 필요한 부분만 실측 근거로 Rust를 사용한다.
4. Python pipeline이 commit → CI(빌드·테스트·검사) → 성공 시 CD(배포) → 성공 시 push를 실행하도록 구성한다.
5. 단계별 실패·시간 초과·인자 전달·종료 코드 전파와 동일 커밋의 검증·배포·push를 확인한다.
6. one-shot 내부에서 에이전트가 변경을 major/minor/patch로 분류하고, Python이 전체 SHA에 버전을 배정·보존한 뒤 빌드에 주입하도록 구현한다.
7. CI에서 검증한 산출물을 공유 `3rd_party/skim-search/releases/{전체_SHA}/`에 `skim-search-{version}-{sha8}-windows-x64.zip`, manifest, 체크섬으로 게시한다.

# actual result
미구현. one-shot 파이프라인과 각 단계의 공통 호출 구조가 없다.

# expected result
- `rules/one-shot.md`의 호출 구조, 단계 순서, 언어별 책임을 충족하며 각 단계도 독립 실행 가능하다.
- 공백이 있는 경로와 다른 작업 디렉터리에서 인자 및 종료 코드가 계층마다 보존된다.
- 성공 경로는 commit → CI → CD → push 순서로 실행되며 동일 SHA의 소스와 산출물을 사용한다.
- 사용자 확정 요구(2026-10-01): SHA가 SSOT이며 모든 빌드·사용자 배포는 버전+SHA 형식을 사용한다. 에이전트 분류와 Python 버전 계산은 `rules/one-shot.md#버전-관리`를 충족한다.
- 새 전체 SHA마다 버전이 바뀌며 같은 SHA는 재실행·실패 재시도에서도 같은 버전을 유지한다. major/minor 초기화, 동시 배정, 분류 실패와 SHA/기준 불일치를 검증한다.
- `rules/one-shot.md#공유-폴더-배포`에 따라 공유 3rd_party 하위에 검증한 Release 패키지가 게시된다. 재빌드·사용자 설정 포함·기존 배포 덮어쓰기가 없으며 체크섬 확인 결과를 로그로 남긴다.
- commit 또는 CI 실패 시 후속 단계가 실행되지 않는다. CD 실패 시 push가 실행되지 않는다. push 실패는 pipeline 실패로 보고된다.
- commit할 변경이 없으면 현재 HEAD를 사용하고 로그로 구분한다. CI 이후 소스 또는 HEAD가 변경되면 CD/push를 중단한다.
- 필수 실행 설정이 없으면 실행 전에 실패하며, 미설정 CD를 성공으로 보고하지 않는다.
- 성공·실패·시간 초과 경로에 대한 검증 로그가 `ref/actual/logs/one-shot/`에 UTF-8로 저장된다. 실행 ID, SHA, 단계 명령·시각·종료 코드·출력과 배포/push 결과를 포함한다.
- 실행한 자식 프로세스와 임시 자원을 정리한다. 설치·설정·독립 실행·전체 실행 명령을 문서화한다.

# label
SQA_sk_0_0_0

# environment
OS: Windows 10 Pro
hostname: TBD

# 담당자
