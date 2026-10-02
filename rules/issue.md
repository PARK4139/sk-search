# issue — writing, tracking and closing issues

## Paths and states

```text
issues/
├─ backlog/   ← not started
│  └─ {function family}/{sub family}/{uuid:8}.md
├─ working/   ← actually in progress
│  └─ {function family}/{sub family}/{uuid:8}.md
├─ closed/    ← done, or a record of an incident
│  └─ {function family}/{sub family}/{uuid:8}.md
└─ priority.md
```

- State is shown by the `backlog` / `working` / `closed` folder; issue bodies have no status field.
- An issue path is `{function family}/{sub family}/{uuid:8}`. References omit the state folder.
- On a state change move only the one issue file to the same family/subfamily path. Never move whole folders. Delete folders left empty.
- All existing issues use UUID file names. State folder, family and subfamily are kept.

## Language

- Issues and `priority.md` are written in English (`README.md#language`). UI strings, log lines and other text that must match the product stay verbatim in backticks.

## Names and UUID

- All folder names are English ASCII. function family = snake_case name from `families.md`, sub family = kebab-case.
- A new issue file name is the first 8 lowercase hex characters of a UUID, e.g. `a1b2c3d4.md`. `{uuid:8}` is notation; braces and colon are not part of the file name.
- The single SSOT for ID allocation is `issue_ids.sqlite3` in the Git common directory. The only allocation entry point is `get_issue_id()` in `skim_search.diagnostics.issue_ids` (`cores/python/src/skim_search/diagnostics/issue_ids.py`). CLI: `scripts\issue-id.cmd`.
- Only the first initialization transaction imports existing IDs from `backlog` / `working` / `closed` and earlier reservation files into the DB. After that, unique keys and transactions resolve collisions without scanning folders, and the ID is returned after commit. Earlier reservation files are removed after the import commit. SQLite journals and runtime logs are for recovery/verification, not an ID SSOT.
- Every issue author uses this API and never deletes/recreates the DB or assigns IDs by hand. When importing external issues after initialization, stop allocation and register/deduplicate IDs first. Separate clones do not share the DB, so register/deduplicate before merging.
- The UUID stays the same across state moves.
- Never use Windows-forbidden characters `\\ / : * ? " < > |` in paths.

## Priority

| Grade | Criteria |
|-------|----------|
| Critical | Risk of data loss, wrong file shown / modified / saved, or a blocked core flow (handover §2) that must be handled immediately |
| High | A required feature or AC fails so a main flow is incomplete, but not Critical |
| Normal | Usability, display quality, diagnostics, performance target improvements. Core behavior works but quality is below requirement |
| Low | Optional features (`file:` etc.), showreel-only elements, unconfirmed `Assumption:` items |

## Priority list

`issues/priority.md` lists active UUID issues with exactly three columns.

| Priority | UUID | Source |
|---|---|---|
| Critical / High / Normal / Low | 8 lowercase hex | FR/AC, handover section, showreel time, etc. |

- No title or status column. The work content is the issue's `title`.
- Only `backlog` and `working` issues are listed. Remove the row when the issue moves to `closed`.
- Sort by grade (Critical → High → Normal → Low), then prerequisites, then `families.md` order.
- Each row's source must match the requirement source in that issue's `steps`.

## Splitting

- One issue covers one result that can be judged PASS/FAIL.
- At most one AC or 1–2 FRs. Split if more.
- Syntax tokens may be split per token (`!path:` and `!ext:` etc.).
- Separate UI composition from behavior.
- One issue per defect observed in the showreel.
- Cite only implementation constraints stated by handover; do not prescribe implementation.
- Never create issues that require handover §33 prohibitions.
- Check existing issues in `backlog` / `working` / `closed` for the same FR/AC to avoid duplicates.

## Writing procedure

1. Find verification units in handover FR/AC and the showreel per `sources.md`.
2. Choose the family in `families.md` and name the sub family.
3. Choose the grade by `#priority`.
4. Reserve a new ID with `get_issue_id()` (`#names-and-uuid`).
5. Report the proposal (path + title + grade + source) first; create files after approval (or when the user asks directly, e.g. "add issue").
6. Add a priority · UUID · source row to `issues/priority.md`.

## State transitions and start

```text
issues/backlog/… ──start──▶ issues/working/… ──PASS──▶ issues/closed/…
```

- Before starting, confirm the issue is not an `Assumption:` issue. Do not start unconfirmed requirements.
- By default move one issue at a time from `backlog` to `working`. If bundling code changes is more efficient, report the issue list and reason first.
- `working` holds only issues actually in progress.
- Start order follows `issues/priority.md`. Within a grade, prerequisites first, then `families.md` order.
- If the user names an issue that is already `closed`, do not reopen or redo it. Skip it in `priority.md` and take the next startable `backlog` issue; if all following items are `closed`, take the next highest-priority `backlog` issue, and report completion only when no active `backlog` issue remains.
- Report target files and reasons before editing.

## Completion and report

- An issue is done only when every expected result is PASS with evidence. If anything FAILs, keep it in `working`.
- If a dependency prevents direct verification, record the alternative verification and the dependency issue, and ask for a decision.
- On completion save build, `cargo test` and `cargo clippy` results in `ref/actual/logs/` and collect needed e2e / screen evidence.
- Update the issue's `actual result` with measured results and evidence paths, move it to `closed`, and remove its `priority.md` row.
- The completion report contains: conclusion, verdict and evidence per expected result, changed files, unverified items and why, incident issues, next issue to start.

## File template

Do not change section names or order.

```markdown
# title
{one line, at most 80 characters}

# pre-condition
{run state, search root, test data, etc.}

# steps
Source: FR-xxx, AC-xxx (handover §n) / showreel 00:SS
1. ...

# actual result
{current behavior. "Not implemented" if not implemented}

# expected result
{PASS criteria. Give values and where they are checked when measurements are needed}

# label
SQA_sk_0_0_0

# environment
OS: Windows 11 Pro
hostname: TBD

# assignee
```

### Fields and sources

- The first line of `steps` gives the FR/AC/handover section/showreel time source. Incident issues also give `Origin: {issue path}`.
- Write showreel observations as observed; separate interpretations with `Assumption:`.
- When a work issue is done, `actual result` becomes measured results and evidence paths.
- Write expected results so runtime evidence (logs etc.) can verify them. For performance targets give the target and where measurements are recorded, and report misses too.
- Without a specific pre-condition use the common test workspace below.
- `label` default is `SQA_sk_0_0_0`.

### Common test workspace

The common search fixture is committed in `cores/rust/tests/fixtures/sample/tree/`. Rust and Python tests copy it to a temporary path and never modify the original. Scope-specific extra files are created by the test code.

## Incidents

Product defects, accidents, environment constraints, procedure mistakes, tool errors and build/test failures are always recorded as incident issues right away, at the latest before the work's completion report. Incident issues always live in `closed`.

- family: the feature for product defects, `diagnostics` for verification tools, `build_env` for environment, `process` for procedure mistakes.
- `steps` of an incident issue contains `Origin: {issue path}`.
- `actual result` is written as `Occurred` / `Cause` / `Action` / `Result` (PASS or `Unresolved`) / `Prevention`, in that order.
- expected result describes normal behavior had the incident not happened.
- If unresolved, set `Result: Unresolved`, write a separate fix issue in `backlog`, and record each other's path.
- If the prevention is a rule, add it to the owning rules file.
- Exception — one-shot failures: the pipeline creates one `backlog/diagnostics/one-shot-failure/{uuid:8}.md` automatically and registers it in `priority.md`. That issue is the failure SSOT (no `failure.json`); no separate closed record is written at failure time. Active issues with the same SHA (including SHAs that differ only in issue records / generated evidence), stage and item only get a recurrence entry. After the fix is verified, move the same file to closed and remove the row. Closed issues are never reopened; a recurrence gets a new issue that links them (`one-shot.md#failure-issues`).
