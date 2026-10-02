# title
Implement the one-shot commit → CI → CD → push pipeline

# pre-condition
- Version policy at the time: per the user's resume instruction, automatic classification is implemented in `diagnostics/one-shot-agent-classification/f88f1527`. Actual push is blocked per user instruction.
- Git repository, Rust build tools, Python and PowerShell runtime
- Settings needed: commit scope, CI commands, deploy target / method / verification, push remote / branch

# steps
Source: user request (2026-10-01): one-shot call structure and the commit → CI → CD on pass → push pipeline; rules/one-shot.md
1. Implement the root entry point `one-shot.cmd` → `one-shot.ps1` → `one_shot.py`.
2. Implement `.cmd` → `.ps1` → `.py` run paths for commit, ci, cd and push in `cores/scripts/one_shot/`.
3. Keep `.cmd` thinnest and `.ps1` thin; write the complex logic in Python. Use Rust only where measurements show extreme speed is needed.
4. The Python pipeline runs commit → CI (build, test, checks) → CD (deploy) on success → push on success.
5. Confirm per-stage failure, timeout, argument passing and exit code propagation, and that verification, deployment and push use the same commit.
6. Inside one-shot an agent classifies changes as major/minor/patch; Python assigns and keeps a version per full SHA and injects it into the build.
7. Publish the CI-verified artifacts to the shared `3rd_party/skim-search/releases/{full_SHA}/` as `skim-search-{version}-{sha8}-windows-x64.zip`, manifest and checksums.

# actual result
PASS (2026-10-02).
- Implementation: root `one-shot.cmd` → `.ps1` → `one_shot.py`, `cores/scripts/one_shot/{commit,ci,cd,security,push}.cmd/.ps1/.py` (through launch.ps1), `pipeline.py`. The deploy location is `3rd_party/skim-search/{full_SHA}/` per the efcb006b decision (not `releases/` as in step 7). The push block was lifted by user instruction (2026-10-01 "push via one-shot"), and a security stage was added (aea6b525).
- Real run PASS: run `ref/actual/logs/one-shot/20261002-013512-7acafd6a` — commit→ci→cd→security→push all passed, SHA 76e944e, version 0.2.4 (`--bump patch`), exe `--version` = `skim-search 0.2.4 76e944e…`, package / manifest / SHA256SUMS published and checksum verified, origin/main == 76e944e.
- Automated verification `test_pipeline` 21/21 PASS (`ref/actual/logs/one-shot-pipeline-tests.log`; path with spaces `repo with spaces`, wrappers run from another cwd):
  - Success path and same-SHA artifacts: full wrappers push, stop-after security.
  - Versions: resets per bump level, same-SHA reuse and conflict, unique concurrent assignment, agent classification / reuse / invalid response / timeout stop before CI, no agent call with --bump.
  - Failure propagation: CI failure → no CD; CD failure → no security / push, remote unchanged (new); timeout kills child processes; no changes → HEAD used and `commit_unchanged` recorded (new); missing required settings fail before the run (new); HEAD / package tamper guard; republishing keeps the existing package.
  - Push gate: blocked by the security policy, blocked when the remote changed, blocked by a result for another SHA, standalone push without `--run-dir` refused.
- Found and fixed during real runs (closed): one-shot-concurrent-build-overwrites-exe, scope-test-depends-on-fixture-content, e2e-common-missing-os-import, ui-flow-root-rerun-intermittent, e2e-ui-stats-after-fixture-change.
- Docs: `rules/one-shot.md#usage` gained prerequisites, settings, full / partial / independent stage runs and self-test commands.

# expected result
- Meets the call structure, stage order and language responsibilities of `rules/one-shot.md`; each stage also runs independently.
- Arguments and exit codes are preserved at every layer with paths containing spaces and from other working directories.
- The success path runs commit → CI → CD → push and uses the sources and artifacts of the same SHA.
- User-confirmed requirement (2026-10-01): the SHA is the SSOT and every build and user release uses the version+SHA format. Agent classification and Python version calculation meet `rules/one-shot.md#versioning`.
- Every new full SHA gets a new version; the same SHA keeps its version on reruns and retries after failures. major/minor resets, concurrent assignment, classification failures and SHA/base mismatches are verified.
- A verified release package is published under the shared 3rd_party per `rules/one-shot.md#shared-folder-deployment`. No rebuild, no user settings, no overwriting of existing deployments; checksum verification is logged.
- If commit or CI fails, later stages do not run. If CD fails, push does not run. A push failure is reported as a pipeline failure.
- With nothing to commit the current HEAD is used and this is distinguished in the log. If sources or HEAD change after CI, CD/push stop.
- Missing required settings fail before the run, and an unconfigured CD is never reported as success.
- Verification logs for success, failure and timeout paths are saved as UTF-8 in `ref/actual/logs/one-shot/`, including run ID, SHA, stage commands, times, exit codes, output and deploy/push results.
- Started child processes and temporary resources are cleaned up. Installation, settings, independent and full run commands are documented.

# label
SQA_sk_0_0_0

# environment
OS: Windows 10 Pro
hostname: TBD

# assignee
