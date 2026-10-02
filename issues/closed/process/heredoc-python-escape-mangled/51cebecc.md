# title
Backslashes in Python strings inside a bash heredoc are interpreted as escapes, replacements fail repeatedly

# pre-condition
- Running Python replacement scripts through a bash heredoc (`<<'PYEOF'`) with Windows paths and `\n` literals in strings

# steps
Source: rules/environment.md#script-writing-cautions / Origin: build_env/paths-ssot, diagnostics/one-shot-failure-issue
1. Run a replacement in heredoc Python containing `"...%LOGS%\bench-sk.out.log..."`, `"\n"` etc.

# actual result
- Occurred: three times on 2026-10-02 — `\b` interpreted as backspace failed an assert (test.cmd), a multi-line block did not match (test_pipeline), regex `\S` warning. The file was never changed (stopped by the assert).
- Cause: the heredoc passes text as is, but normal Python string literals treat `\b`, `\p`, `\c` etc. as escapes. The existing prevention (`closed/process/script-edit-backslash-mangled/fd494e04`: edit Windows paths with the Edit tool) was not applied to heredoc Python.
- Action: write a script file with raw strings (`r"..."`) using the Write tool and run it, or use the Edit tool. Every replacement asserts `count == 1` to prevent partial application.
- Result: PASS — later replacements applied correctly.
- Prevention: replacements containing backslashes use a Write-tool file (raw strings) or the Edit tool instead of inline heredoc Python (added to rules/environment.md).

# expected result
- Replacement scripts apply in one go with the strings exactly as written.

# label
SQA_sk_0_0_0

# environment
OS: Windows 11 Pro
hostname: TBD

# assignee
