# security — pre-push security risk checks

Applies to every push (manual and one-shot pipeline). The remote `PARK4139/sk-search` is **public**.

> **Mandatory (no exceptions)**: every push must be immediately preceded by `security_policy` exit 0. It cannot be turned off by emergency backup (`--urgent-backup`), stage switches (`configs/one-shot.json` `stages`) or any argument, and no setting or option to turn it off may be created. Emergency mode skips only the CI, CD and security stages (build, tests, gitleaks, dependency audit) (user decision 2026-10-02, `diagnostics/one-shot-urgent-backup`).

Implementation / automation: `diagnostics/one-shot-security-gate`. Pipeline wiring: `one-shot.md#stages-and-failure-handling`.

## Scope

- Commits to push = all commits in `<remote>/<branch>..<ref to push>`, the blobs they add or change, and commit metadata (author/committer name and e-mail, message).
  - If the remote base cannot be determined (fetch failure, no remote ref) the check does not pass. A new repository checks the full history of the ref to push.
  - Never check `--all`. Local-only refs (stash, pre-rewrite history) are not pushed and cause false positives.
- When a deploy package (ZIP) is uploaded or published as well, its content is checked too.
- Uncommitted changes in the working tree are not pushed. Never mix other contributors' in-progress files into the push range.

## Blocking items (no push when found)

| ID | Item | Examples |
|----|------|----------|
| SEC-SECRET | Secrets | tokens (`ghp_`, `github_pat_`, `xox?-`, `AKIA…`, `AIza…`), private key blocks, password / API key assignments |
| SEC-LOCALPATH | Local user paths / account names | `C:\Users\<name>\`, `C:/Users/<name>/`, `/c/Users/<name>/` → docs use `%USERPROFILE%`. Windows shared profiles (`Public`, `Default`, `Default User`, `All Users`) are not accounts and are allowed |
| SEC-EMAIL | Personal e-mail | e-mail in commit metadata or added lines outside the allow list (`*@users.noreply.github.com`, `noreply@github.com`, `noreply@anthropic.com` (Co-Authored-By), reserved documentation/test domains `example.*`, `*.invalid`, `*.test`, `*.example`) |
| SEC-PATH | Forbidden paths | `ref/actual/logs/`, `.venv/`, `target/`, `__pycache__/`, `*.env`, user settings files (real `settings.json`) |
| SEC-VULN | Dependency vulnerabilities | at or above the configured severity in `cores/rust/Cargo.lock`, `cores/python/uv.lock` |
| SEC-TOOL | Check failure | check not run, tool error, timeout, vulnerability data unavailable |

- Warnings (not blocking): files over 5 MB, personally identifying strings such as user workspace or host names.
- Screen captures show only the skim-search window (`environment.md#script-writing-cautions`). Never commit captures that may include other windows.
- Text inside images is outside automatic checks (security_policy, gitleaks). Before committing captures, visually confirm that paths on screen (search root box, results, toasts) show no account name. The e2e workspace is `%PUBLIC%\skim-search-e2e` (copy of the golden sample tree); never show paths under `%TEMP%` or `%USERPROFILE%` on screen (`diagnostics/screenshot-local-user-path`).

## Running

| Check | Entry point | Covers |
|-------|-------------|--------|
| Repository policy check | `scripts\security-policy.cmd [--remote origin] [--branch main] [--ref HEAD]` | SEC-SECRET (base patterns), SEC-LOCALPATH, SEC-EMAIL, SEC-PATH, SEC-TOOL, WARN-LARGE |
| Secrets and dependency vulnerabilities | pipeline `security` stage (`cores/python/src/skim_search/diagnostics/one_shot/pipeline.py`) | SEC-SECRET (gitleaks), SEC-VULN (cargo-audit, pip-audit), SEC-TOOL |

- Before a manual push: run `scripts\security-policy.cmd` → push only on exit 0. 1 = blocking item found, 2 = check could not run.
- Self test: `scripts\security-policy.cmd --self-test` (or `cores/python/tests/security/test_policy.py`) (temporary repositories only, never pushes to the real origin).
- The one-shot push stage calls `security_policy` right before pushing and pushes only on exit 0 (`one-shot.md#stages-and-failure-handling`). It always runs, regardless of `--urgent-backup` and `stages`.

## Exceptions

- The exception list `configs/security-exceptions.json` gives ID, rule, target path (glob), reason and expiry (`expires`, YYYY-MM-DD). Expired exceptions are ignored.
- Applied exceptions are recorded in the check result.

## Results and records

- Results are bound to the full SHA (checked range). If the target SHA or package changes after the check, the result is invalid.
- Logs and reports: `ref/actual/logs/security/` (local, not committed). Record location, ID, severity and blocking status, and **never the raw secret** (masked).
- Passing means "nothing found by the configured checks". Report the scope and unsupported items with it; never state "no security risk".
- When something is found, fix and recheck. If it is in committed history, remove it from history before pushing (rewrites only for local unpushed commits). A secret already on the remote is revoked and replaced immediately.
