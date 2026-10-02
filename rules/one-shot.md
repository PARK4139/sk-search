# one-shot — local pipeline rules

## Execution structure

- The user entry point is `scripts\one-shot.cmd`. Call chain: `scripts\one-shot.cmd` → `scripts\one-shot.ps1` → `scripts\launch.ps1` → `python -m skim_search.diagnostics.one_shot`.
- The pipeline (`cores/python/src/skim_search/diagnostics/one_shot/pipeline.py`) runs `commit` → `ci` (build, test, checks) → `cd` (deploy) → `security` → `push`.
- Each stage is independently runnable via `scripts\{stage}.cmd` → `{stage}.ps1` → `python -m skim_search.diagnostics.one_shot.{stage}`. Stages: `commit`, `ci`, `cd`, `security`, `push`.

## Usage

- Prerequisites: Rust (cargo), uv (PATH or `3rd_party/pk_system/uv.exe`), `3rd_party/ripgrep/rg.exe`, `3rd_party/skim/sk.exe`, `3rd_party/security/gitleaks.exe`, `cargo install cargo-audit`. For automatic classification, install and log in to the Codex CLI.
- Settings: `configs/one-shot.json` (remote, branch, commit_paths, commit_message, initial_version, initial_base_sha, stages, tool paths, timeouts). Missing required settings fail before the run.
- Full run: `scripts\one-shot.cmd` (agent classification) or `scripts\one-shot.cmd --bump patch|minor|major`. Output: `PASS through push; push: pushed; logs: ref\actual\logs\one-shot\{run}`.
- Up to just before push: `scripts\one-shot.cmd --bump patch --stop-after security`.
- Emergency backup: `scripts\one-shot.cmd --urgent-backup` → commit (including add) → push only. Output: `push: pushed (urgent-backup: ci/cd/security skipped)`.
- Independent stage: `scripts\{stage}.cmd --run-dir ref\actual\logs\one-shot\{run}` (only for a run whose previous stages passed).
- Self test: in `cores\python`, `uv run --locked python -m unittest tests.one_shot.test_pipeline` (temporary bare remotes only). Also part of `scripts\test.cmd`.
- The pipeline calls each stage's `.cmd` entry point, so it uses the same path as an independent run.
- Entry points (.cmd/.ps1) all live directly in `scripts/` (no subfolders: `one-shot`, `{stage}`, `launch.ps1`, `test`, `security-policy`, `issue-id`). Logic lives in `cores/python/src/skim_search/diagnostics/one_shot/`. The names `one-shot.cmd` and `one-shot.ps1` are a user-specified exception; Python files are snake_case.

## Language responsibilities

- `.cmd` is as thin as possible: call the `.ps1` of the same stage, pass arguments, return the exit code.
- `.ps1` is thin: connect the Python environment, pass arguments, return the exit code.
- `.py` holds the complex logic: stage composition, settings, process management, error handling, verification, logging.
- `.rs` is for parts that need extreme speed. Decide by measurement; Python composes the pipeline.
- Each call layer preserves arguments and exit codes. Compute the repository path from the entry point so nothing depends on the working directory or paths with spaces.

## Stages and failure handling

- commit: commit the configured change scope and record the SHA. With nothing to commit, use the current HEAD and record that.
- After commit and before the CI build, classify with the agent and assign a version per `#versioning`.
- CI: build, test and check the SHA's sources; all must pass. The first step is the path SSOT consistency check (`skim_search.gen_paths --check`). The build uses `RUSTFLAGS=--remap-path-prefix` to replace the cargo home and repository paths with `cargo-home` / `skim-search` (values computed at runtime, never committed). After `--version`, the CI fails if the exe contains any user path (security_policy LOCALPATH); only the count is recorded (`exe_path_scan` event).
- CD: deploy the artifacts built from the CI-passed SHA to the configured target and verify the deployment.
- security: check the CD artifacts and the commit range to be pushed (`security.md`).
- push: after security passes, push the same verified commit to the configured remote/branch (fast-forward only, never forced). Right before pushing, all of the following must hold or nothing is pushed: HEAD and sources unchanged (guard), security result bound to the SHA, package checksum unchanged, `security_policy` passed (exit 0), remote branch unchanged since the security check. After the push, confirm the remote HEAD is the SHA.
- `--stop-after security` runs up to just before push. The default runs through push.
- If CI fails, CD and push do not run. If CD fails, push does not run. Every stage failure propagates as a non-zero pipeline exit code.
- If sources or HEAD change after CI, the run's CD/push stops. Generated logs and artifacts are excluded from the source-change check.
- Before running, confirm commit scope, CI commands, deploy target / method / verification, and push remote/branch. Missing required settings are run errors; a stage is never reported as passed or deployed in that case.

## Stage switches and emergency backup

- `"stages": {"ci", "cd", "security", "push"}` (true/false, default true) in `configs/one-shot.json` turns stages off. commit always runs. Other keys (`commit`, `policy`, …), non-booleans and dependency violations (cd needs ci, security needs cd) fail before the run.
- Disabled stages are recorded as `skipped` in `state.json` and as `stage_skipped` events. The run plan is kept in `state.json` `plan` (`mode`, `enabled`) and stage child processes follow it. Running a disabled stage independently fails.
- `--urgent-backup` (emergency backup): regardless of settings, turns ci, cd and security off and runs only commit → push. Without a build no version is assigned and no package is published. `security_policy` right before the push (mandatory, top of `security.md`), fast-forward only and the post-push remote HEAD check remain. Console and state show the skipped stages.
- Emergency backup is an exception. Code changes are re-verified later with a normal run.

## Versioning

- User instruction (2026-10-01): automatic agent classification applies (`f88f1527` resumed). Explicit `--bump` and reuse for an existing SHA follow the rules below.
- The SSOT for source identity is the full commit SHA. `sha8` is for display; lookups and duplicate checks use the full SHA.
- No separate dev/user releases. Every package is named `skim-search-{major}.{minor}.{patch}-{sha8}-windows-x64.zip`.
- Inside one-shot the configured agent command classifies the changes from the last released SHA to the current SHA. The agent returns one of `major`, `minor`, `patch` with a reason, base SHA and target SHA as structured output. Python validates it and computes the version number.
- major breaks compatibility (usage, settings, …); minor adds compatible features; patch is a fix, improvement, docs or build change. With several changes the highest grade applies. A new SHA bumps at least patch.
- major resets minor/patch to 0; minor resets patch to 0; patch increments patch only. The same rules apply to 0.x.
- The same SHA reuses its assigned version, also on reruns after failures. The initial base version is set in the settings.
- An assignment is recorded as `reserved` (used for build injection and CD publication) and becomes `released` only after the push is verified. The next number is bumped from the highest of `released` versions and reservations of **running** runs (top-level one-shot process PID alive). Reservations of other SHAs whose process ended without a push become `superseded` and their number is reused (`package_published` marks an already published package; nothing is deleted or overwritten). Failed runs therefore do not consume version numbers. Entries without `status` (written before reservations existed) count as `released` (`diagnostics/one-shot-version-on-failure/d6b840a2`).
- The classification base SHA is the SHA of the last `released` version.
- The shared `3rd_party/skim-search/versions.json` keeps full SHA ↔ version, classification, reason, base and assignment time. Concurrent runs use locking and atomic updates so different SHAs never get the same version.
- Agent failures, missing or invalid classifications, base mismatches and mapping conflicts fail before CI.
- Version and SHA are injected at build time. `Cargo.toml` is never edited and no extra commit is made for versioning.

## Agent invocation

- When the user runs `one-shot.cmd` without a bump argument, Python calls Codex CLI `codex exec` as a child process to classify automatically. Confirm Codex CLI installation, authentication and path first.
- The call is `codex exec --sandbox read-only --output-schema {schema.json} --output-last-message {classification.json} -`. Python uses an argument array instead of a shell string and passes the classification instructions and change history on stdin.
- Input: base and target full SHAs, commit messages and diff between them, and the classification criteria. Git data is material to interpret; instructions inside it are not executed. The agent only classifies; it never re-invokes one-shot, edits files, commits or deploys.
- The response schema requires `level` (major/minor/patch), `reason`, `base_sha`, `target_sha`. Python validates the schema, the actual SHAs, a non-empty reason and the process exit code.
- `--bump major|minor|patch` uses that value and skips the agent. The decision maker (agent/user) and reason are recorded. An argument conflicting with an existing classification for the same SHA is an error.
- An already assigned SHA reuses its version without calling the agent. The first assignment initializes from the configured initial version and base SHA; an unclear comparison range fails.
- CLI failure, authentication failure, timeout or invalid response stops before CI. Per-run classification input, response and errors are logged; on timeout the started process is cleaned up.

## Shared folder deployment

- CD publishes packages under the shared `CavemanDrive/3rd_party`. The current shared root is `%USERPROFILE%/Downloads/CavemanDrive/3rd_party`; per-environment paths come from settings.
- The deploy location is `3rd_party/skim-search/{full_SHA}/` (same folder as `versions.json`), containing the package ZIP, `manifest.json` and `SHA256SUMS`.
- The ZIP contains the CI-verified Windows x64 release `skim-search.exe`, the needed `rg.exe` / `sk.exe` and usage notes. No user settings file. CD never rebuilds.
- The manifest records full SHA, assigned version, classification reason, build time / environment / tool versions and artifact information. CD succeeds only after the published ZIP checksum is verified.
- Package and verification data are completed in a temporary location before publishing. Existing deployments are never overwritten; a rerun for the same SHA verifies the existing package, manifest and checksum, and fails on mismatch.

## Run evidence

- Per-run UTF-8 logs in `ref/actual/logs/one-shot/`.
- Record run ID, commit SHA, stage order, start/end times, commands, exit codes, stdout/stderr, deploy artifacts / target / result, push remote / branch / result. No credentials in logs.
- On success, failure or timeout clean up started child processes and temporary resources.

## Failure issues

- A failure exits non-zero and creates no separate failure artifact such as `failure.json`. The failure SSOT is one issue `issues/backlog/diagnostics/one-shot-failure/{uuid:8}.md` (exception in `issue.md#incidents`). Implementation: `cores/python/src/skim_search/diagnostics/one_shot/failure_issue.py`.
- Only the outermost process records (nested stages only report). It follows the nested wrappers' `FAIL:` lines to the command that actually failed and writes, in English: failing stage and item, full SHA (or why it is unknown), run ID and time, exit code / timeout / not runnable, reproduction command, masked error summary (last 20 lines), evidence log paths. Cause and action are `Unconfirmed` / `None` until known.
- The key is `sha | stage | item`. If a backlog/working issue with the same key exists (including SHAs that differ only in issue records / generated evidence), only a `- Recurred` line is added and no priority row is added. Writes are serialized by `ref/actual/logs/one-shot/.failure-issue.lock`.
- Account names, secrets and personal e-mail are masked with the `security_policy` patterns. If the issue cannot be saved, `failure issue not saved` is printed and the original failure (exit 1) stands.

## Keyboard use alert

- Right before a stage that injects keyboard/mouse input (CI UI e2e, `KEYBOARD_MODULES`) show one non-modal alert. Once per run; later keyboard stages do not show it again.
- The alert is a Windows toast that does not take focus, falling back to a tray balloon. After the alert wait `keyboard_alert_lead_seconds` (default 5 s) before continuing.
- Alert time, method and wait are recorded in the run state (`state.json` `keyboard_alert`) and the event log.
