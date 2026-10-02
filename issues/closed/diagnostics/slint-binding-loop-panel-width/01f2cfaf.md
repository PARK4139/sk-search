# title
Slint binding loop warning when panel width is a ratio of parent.width

# pre-condition
- `width: (parent.width - 8px) * 0.45` on the result/preview panels

# steps
Source: rules/implementation.md#ui / Origin: while fixing view/panel-width-shifts-with-content
1. `cargo build`

# actual result
- Occurred: warning `binding loop (root.layoutinfo-h -> keys.width -> ...)` with "may panic at runtime".
- Cause: a layout child referenced the parent width → cycle with layout info computation.
- Action: changed to preferred-width 0 + horizontal-stretch 45:55.
- Result: PASS — 0 warnings.

# expected result
- No layout binding loop warning.

# label
SQA_sk_0_0_0

# environment
OS: windows 10 pro
hostname: TBD

# assignee
