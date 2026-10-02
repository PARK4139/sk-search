# title
Rules link check mistakes the code notation `#[cfg(test)]` for an anchor

# pre-condition
- A bash script that checks `` `#...` `` notations in rules/*.md as same-file heading anchors

# steps
Source: rules/README.md#reference-notation / Origin: rules structure refactoring
1. Run the same-file anchor check

# actual result
- Occurred: 1 hit `NOANCHOR rules/implementation.md#[cfg(test)]`.
- Cause: the Rust attribute `` `#[cfg(test)]` `` matches the anchor pattern.
- Action: judged a false positive by manual review. All 24 real anchors (17 cross-file, 7 same-file) are valid.
- Result: PASS
- Prevention: exclude notations starting with `#[` from the anchor pattern.

# expected result
- The anchor check reports only real broken links.

# label
SQA_sk_0_0_0

# environment
OS: windows 10 pro
hostname: TBD

# assignee
