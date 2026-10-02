# families — function family list

The first-level issue folder name and the `cores/rust/app/src/` module name.
Table order is also the start order for issues of equal priority (`issue.md#state-transitions-and-start`).

| function family | Meaning | Source FR / AC | Implementation (relative to `cores/rust/`; Python under `cores/python/`) |
|-----------------|---------|----------------|---------------------------|
| view            | Main view (main window, search root) | FR-100, 101, 103, 104 / AC-100, handover §5, §24 | `app/ui/view.slint`, `app/src/app.rs` |
| query_syntax    | Query syntax | FR-111~118 / AC-105 | `app/src/query_syntax/` |
| search_engine   | Search engine | FR-105~110, 120, 132 / AC-102~104, 114 | `app/src/search_engine/` |
| result_panel    | Result panel | FR-119, 121, 122 / AC-106, 107 | `app/src/result_panel/`, `app/ui/result_panel.slint` |
| preview_editor  | Preview editing | FR-123~126 / AC-108~110 | `app/src/preview_editor/`, `app/ui/preview_editor.slint` |
| external_open   | External open | FR-127, 128 / AC-111, 112 | `app/src/external_open/` |
| shortcut        | Shortcuts | FR-102, 130, handover §22 / AC-101 | `app/src/shortcut/` |
| toast           | Toasts | FR-129, 133 / AC-113 | `app/src/toast/`, `app/ui/toast.slint` |
| status_bar      | Status display | handover §24 status bar, §14 stats | `app/ui/status_bar.slint` |
| settings        | Settings | FR-131, handover §20 editor setting | `common/src/settings.rs` |
| diagnostics     | Diagnostics & performance | handover §30 logs, AC-103 measurement, handover §29 tests, verification tools | `common/src/log.rs`, `tests/`, `cores/python/` (`skim_search.diagnostics`, `tests/`, `benchmarks/`) |
| build_env       | Build & runtime environment | handover §26 dependencies, handover §12 rg/sk setup, dev environment constraints | `Cargo.toml`, root `.cargo/`, `scripts/`, `configs/`, path SSOT `cores/common/paths.ini` |
| process         | Work procedure | `rules/issue.md` violations and accidents (no product code) | (no code) |

## Rules

- Update this table first when adding a family.
- Names: snake_case English (`issue.md#names-and-uuid`).
- Locations are the target shape. Create only the files an issue needs when it starts (`implementation.md#structure-rules`).
