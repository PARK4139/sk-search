# rules — skim-search project rules (entry point)

Read this file before any task, then read only the files listed for that task below.
Agent behavior rules (reply language, reporting style, etc.) follow `AGENTS.md`.

## Files to read per task

| Task | Files |
|------|-------|
| Write an issue | `sources.md`, `families.md`, `issue.md` |
| Start / finish an issue | `issue.md`, `families.md` |
| Implement code | `implementation.md`, `families.md`, `environment.md` |
| Verify / collect evidence | `environment.md`, `issue.md#completion-and-report` |
| Implement / run the one-shot pipeline | `one-shot.md`, `environment.md`, `security.md` |
| Push (including manual) | `security.md` |
| Record an incident | `issue.md#incidents`, `issue.md#file-template` |
| Edit rules | this file, `#editing-rules` |

## File list

| File | Content |
|------|---------|
| `sources.md` | Requirement sources (handover, showreel), precedence between sources, `Assumption:` marking |
| `families.md` | Function family list: meaning · source FR/AC · implementation location |
| `issue.md` | Issue paths · UUID · priority · write/start/finish procedure · template · language · incident rules |
| `implementation.md` | Code structure under `cores/`, dependency direction, test and log rules |
| `environment.md` | External tool paths, cargo settings, evidence paths, environment variables, test scripts |
| `one-shot.md` | commit → CI → CD → security → push pipeline, language responsibilities, run evidence |
| `security.md` | Pre-push security checks: scope, blocking items (SEC-*), exceptions, result records |

## Language

- Rules (`rules/*.md`) and issues (`issues/**`, including `priority.md`) are written in **English** (user decision 2026-10-02).
- Keep verbatim, in backticks, text that must match something outside the document: product UI strings (e.g. toast title `검색 완료`), log lines, command output, quoted file content. Do not translate them.
- Chat replies to the user follow `AGENTS.md` (Korean). The requirement source `ref/handover.md` and its history `ref/closed/` stay in their original language.

## Reference notation

- handover sections: always `handover §N`. Never a bare `§N`. Several sections share one prefix (`handover §5, §24`).
- FR-xxx / AC-xxx: handover §27 (FR list), §28 (Acceptance Criteria).
- Rules: `rules/<file>.md#<heading>` (e.g. `rules/issue.md#state-transitions-and-start`). No numbered references.
- Issues: `{function family}/{sub family}/{uuid:8}` — omit the state folder (`backlog`/`working`/`closed`).
- showreel time: `showreel 00:SS`.

## Terms

| Term | Meaning |
|------|---------|
| function family | First-level issue folder. A user-facing feature group (`families.md`) |
| sub family | Second-level issue folder. One problem / feature unit |
| priority | Critical / High / Normal / Low grade recorded in `issues/priority.md` |
| UUID | First 8 lowercase hex characters of a UUID, used as the issue file name |
| state folder | `issues/backlog`, `issues/working`, `issues/closed` |
| work issue | An issue that implements and verifies a requirement |
| incident issue | An issue recording something that happened during work. Always `closed` (`issue.md#incidents`) |
| source | Origin of a requirement (`handover.md`, showreel) |
| evidence | Measured implementation result (logs and screenshots in `ref/actual/`) |
| `Assumption:` | Marks an interpretation that has no source |

Priority and UUID list: `issues/priority.md` (`issue.md#priority-list`).

## Editing rules

- Each rule lives in exactly one file. Other files reference it by link (no duplication).
- When adding or removing a file, update both tables in this file.
- When a rule changes on user instruction, apply it to the owning file and report a summary of the change.
