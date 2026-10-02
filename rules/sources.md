# sources — requirement sources

## Source material

| Material | Path | Notes |
|----------|------|-------|
| Requirements | `ref/handover.md` | FR-100~133, AC-100~114, handover §33 prohibitions. Migrated sections are stubs → issues (`#handover-migration`) |
| Requirements (migrated) | `issues/*/{family}/{sub family}/` | Issues a handover stub points to |
| Screens / flow | `ref/showreel/issue-full-flow.mp4` | 18 s, 1536x1024 |
| showreel frames | `ref/showreel/frames/fNN.png` | 1 fps extraction. Command in `environment.md#external-tools` |

- showreel reference: `showreel 00:SS` (`fNN.png` = showreel 00:(NN-1) s).
- `ref/closed/` is not a source. It is the history of handover text removed after migration to issues (`#handover-migration`).
- `ref/actual/` is not a source. It stores measured results (screenshots / logs) and is never cited as an issue source.

## Precedence

- `handover.md` (remaining sections) = issues of migrated sections > showreel. The source of a migrated section is its issue (`#handover-migration`).
- On conflict follow handover and record the conflict in the issue body.

## Handover migration

Requirements moved into issues are removed from handover so a source never exists in two places.

- Unit: a handover section (`# N.`) or subsection (`## N.M`).
- Condition: every normative sentence of the section (targets, prohibitions, required behavior, syntax / command examples) is contained in the `steps` / `expected result` of one or more issues.
  - If only part is covered, do not migrate. Add the missing content to an issue first.
  - Sections with only explanation or recommended implementation (recommended options etc.) are not issue material and stay.
- Never migrated: global principles, constraints, indexes — handover §1, §2, §25~§28, §31~§34, final request.
- Method: delete the body and keep the title and number as a stub. Keeping numbers keeps existing `handover §N` and FR/AC references valid.
  ```markdown
  # 17. Preview

  > 이관됨 (YYYY-MM-DD) → issues: `preview_editor/preview-content-mismatch`, `preview_editor/match-scroll-highlight`
  ```
  - A stub names `{function family}/{sub family}`, not the UUID file (unaffected by state moves or renames). Stubs live in `ref/handover.md` and keep its original language.
- `handover §N` references inside issues stay as they are. The stub links to the issue.
- History: keep the deleted text per section in `ref/closed/handover-YYYY-MM-DD.md`, with `> 이관 대상 issues: …` under each section.
  - `ref/closed/` is history. Never cite it as a source (the source is the issue).
  - Confirm every deleted line exists in the history file.
- git: commit the pre-migration state, then commit the migration result (`ref/handover.md`, `ref/closed/`, logs) separately. Full original: `git show <pre-migration commit>:ref/handover.md`.
- Record: per-section migration status, target issues and reasoning in `ref/actual/logs/handover-migration.log` (local evidence, not committed).
- When new issues cover all content of a remaining section, migrate it with the same procedure.

## Assumption marking

- Elements that appear only in the showreel and not in handover (e.g. favorites, terminal/Git tabs) are marked `Assumption:` and get priority Low (`issue.md#priority`).
- No implementation instruction before the requirement is confirmed (`issue.md#state-transitions-and-start`).
- Requirements without a source are not turned into issues.
