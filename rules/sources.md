# sources — 요구사항 근거

## 근거 자료

| 자료 | 경로 | 비고 |
|------|------|------|
| 요구사항 | `ref/handover.md` | FR-100~133, AC-100~114, handover §33 금지사항. 이관된 절은 stub → issue (`#handover-이관`) |
| 요구사항 (이관분) | `issues/*/{family}/{sub family}/` | handover stub이 가리키는 issue |
| 화면/흐름 | `ref/showreel/issue-full-flow.mp4` | 18초, 1536x1024 |
| showreel 프레임 | `ref/showreel/frames/fNN.png` | 1fps 추출본. 추출 명령은 `environment.md#외부-도구` |

- showreel 근거 표기: `showreel 00:SS` (`fNN.png` = showreel 00:(NN-1)초).
- `ref/closed/`는 근거가 아니다. issue로 이관되어 handover에서 삭제된 원문의 이력이다 (`#handover-이관`).
- `ref/actual/`은 근거가 아니다. 구현 실측 결과(스크린샷/로그) 저장 위치이며 issue 근거로 인용하지 않는다.

## 우선순위

- `handover.md`(남은 절) = 이관된 절의 issue > showreel. 이관된 절의 요구사항 근거는 해당 issue다 (`#handover-이관`).
- 충돌 시 handover를 따르고, 충돌 사실을 issue 본문에 기록한다.

## handover 이관

issue로 옮겨진 요구사항은 handover에서 삭제해 근거가 두 곳에 존재하지 않게 한다.

- 이관 단위: handover 절(`# N.`) 또는 하위 절(`## N.M`).
- 이관 조건: 그 절의 규범 문장(목표값, 금지, 필수 동작, 문법·명령 예시)이 모두 하나 이상의 issue `steps`/`expected result`에 들어 있어야 한다.
  - 일부만 들어 있으면 이관하지 않는다. 빠진 내용을 issue에 먼저 추가한 뒤 이관한다.
  - 설명·권장 구현(권장 옵션 등)만 있는 절은 issue 대상이 아니므로 남긴다.
- 이관하지 않는 절: 전역 원칙·제약·색인 — handover §1, §2, §25~§28, §31~§34, 최종 요청.
- 이관 방법: 본문을 삭제하고 제목과 번호는 stub으로 남긴다. 번호를 유지해야 기존 `handover §N`, FR/AC 표기가 계속 유효하다.
  ```markdown
  # 17. Preview

  > 이관됨 (YYYY-MM-DD) → issues: `preview_editor/preview-content-mismatch`, `preview_editor/match-scroll-highlight`
  ```
  - stub에는 UUID 파일명이 아니라 `{function family}/{sub family}`를 쓴다 (상태 이동·파일명 변경에 영향 없음).
- issue 본문의 `handover §N` 표기는 그대로 둔다. stub이 issue로 연결한다.
- 이력: 삭제한 절의 원문을 `ref/closed/handover-YYYY-MM-DD.md`에 절별로 남긴다. 각 절 아래에 `> 이관 대상 issues: …`를 적는다.
  - `ref/closed/`는 이력이다. 근거로 인용하지 않는다 (근거는 issue).
  - 삭제한 모든 줄이 이력 파일에 있는지 확인한다.
- git: 이관 전 상태를 commit한 뒤, 이관 결과(`ref/handover.md`, `ref/closed/`, 로그)를 별도 commit으로 남긴다. 원본 전체는 `git show <이관 전 commit>:ref/handover.md`.
- 기록: 절별 이관 여부·대상 issue·판단 근거를 `ref/actual/logs/handover-migration.log`에 남기고 이관 commit에 포함한다.
- 새 issue를 만들어 남은 절의 내용이 모두 반영되면 같은 절차로 추가 이관한다.

## 추정 표기

- showreel에만 있고 handover에 없는 요소(예: 즐겨찾기, 터미널/Git 탭)는 `추정:`으로 표기하고 우선순위 `300` 이하로 둔다.
- 요구 확정 전 구현 지시 금지 (`issue.md#상태-전이와-착수`).
- 근거 없는 요구사항은 issue로 만들지 않는다.
