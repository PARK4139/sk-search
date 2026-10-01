# rules — skim-search 프로젝트 규칙 (진입점)

작업 전 이 파일을 먼저 읽고, 아래 표에서 해당 작업의 파일만 추가로 읽는다.
에이전트 행동 규칙(응답 언어, 보고 방식 등)은 `AGENTS.md`를 따른다.

## 작업별 읽을 파일

| 작업 | 읽을 파일 |
|------|-----------|
| issue 작성 | `sources.md`, `families.md`, `issue.md` |
| issue 착수 / 완료 | `issue.md`, `families.md` |
| 코드 구현 | `implementation.md`, `families.md`, `environment.md` |
| 검증 / 증거 수집 | `environment.md`, `issue.md#완료-조건과-보고` |
| one-shot 파이프라인 구현 / 실행 | `one-shot.md`, `environment.md` |
| 작업 중 생긴 일 기록 | `issue.md#작업-중-생긴-일`, `issue.md#파일-템플릿` |
| 규칙 수정 | 이 파일 `#규칙-수정` |

## 파일 목록

| 파일 | 내용 |
|------|------|
| `sources.md` | 요구사항 근거(handover, showreel), 근거 간 우선순위, `추정:` 표기 |
| `families.md` | function family 목록: 의미 · 근거 FR/AC · 구현 위치 |
| `issue.md` | issue 경로·UUID·우선순위·작성/착수/완료 절차·템플릿·근거·기록 규칙 |
| `implementation.md` | `cores/` 코드 구조, 의존 방향, 테스트·로그 규칙 |
| `environment.md` | 외부 도구 경로, cargo 설정, 증거 경로, 환경변수, 검증 스크립트 |
| `one-shot.md` | commit → CI → CD → push 파이프라인, 언어별 책임, 실행 증거 |

## 참조 표기

- handover 절: 항상 `handover §N`. 접두어 없는 `§N` 단독 사용 금지. 여러 절은 접두어 1회로 나열 (`handover §5, §24`).
- FR-xxx / AC-xxx: handover §27 (FR 목록), §28 (Acceptance Criteria).
- 규칙: `rules/<file>.md#<heading>` (예: `rules/issue.md#상태-전이와-착수`). 번호 참조 금지.
- issue: `{function family}/{sub family}/{uuid:8}` — 상태 폴더(`backlog`/`working`/`closed`) 생략.
- showreel 시각: `showreel 00:SS`.

## 용어

| 용어 | 의미 |
|------|------|
| function family | issue 1단계 폴더. 사용자 관점 기능 묶음 (`families.md`) |
| sub family | issue 2단계 폴더. 단일 문제/기능 단위 |
| 우선순위 | `issues/priority.md`에 기록하는 Critical / High / Normal / Low 등급 |
| UUID | issue 파일명에 쓰는 UUID 앞 8자리 소문자 16진수 |
| 상태 폴더 | `issues/backlog`, `issues/working`, `issues/closed` |
| 작업 issue | 요구사항을 구현·검증하는 issue |
| 기록 issue | 작업 중 생긴 일을 기록하는 issue. 항상 `closed` (`issue.md#작업-중-생긴-일`) |
| 근거 | 요구사항 출처 (`handover.md`, showreel) |
| 증거 | 구현 실측 결과 (`ref/actual/`의 로그·스크린샷) |
| `추정:` | 근거 없이 해석한 내용 표시 |

우선순위와 UUID 목록은 `issues/priority.md`에서 관리한다 (`issue.md#우선순위-목록`).

## 규칙 수정

- 규칙 하나는 한 파일에만 쓴다. 다른 파일에서는 링크로 참조한다 (중복 기술 금지).
- 파일을 추가·삭제하면 이 파일의 두 표를 갱신한다.
- 사용자 지시로 규칙을 바꾸면 해당 파일에 반영 후 변경 요약을 보고한다.
