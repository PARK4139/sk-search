# title
e2e_ui Ctrl+S check assumes the first md result is README.md and fails intermittently

# pre-condition
- `skim_tests.e2e_ui` step E: search `TODO ext:md`, edit the first result (preview), press Ctrl+S

# steps
Source: AC-109 save (handover §18) / Origin: diagnostics/screenshot-local-user-path
1. `uv run --locked --project cores/tests/py python -m skim_tests.e2e_ui`

# actual result
- Occurred: `CHECK FAIL [preview_editor/ctrl-s-save] keyboard Ctrl+S wrote README.md` (2026-10-02 02:02, 20 PASS / 1 FAIL).
- Cause: the first md result depends on rg output order (this time `docs/sample file.md`). Saving worked (`[preview_editor] saved path=...\docs\sample file.md`) but the check always read `README.md`.
- Action: read the saved path from `saved path=` in the app log and check that file's content.
- Result: PASS — rerun 21/21 (`ctrl-s-save ... wrote sample file.md`).

# expected result
- The Ctrl+S check targets the file that was actually saved.

# label
SQA_sk_0_0_0

# environment
OS: Windows 11 Pro
hostname: TBD

# assignee
