# title
Assumption: confirm the requirement for the showreel `[파일별 그룹화]` / `[정렬: 관련도]` controls

# pre-condition
- None (requirement not defined)

# steps
Source: showreel 00:01 (not defined by handover)
1. Check `파일별 그룹화` and `정렬: 관련도` in the result panel header at showreel 00:01

# actual result
Rejected (2026-10-01): group-by-file toggle and sort (relevance) controls are showreel-only elements not in handover. Per rules/sources.md they are not implemented before the requirement is confirmed.
- Current state: not implemented (based on the final UI of handover §24).
- Resume condition: when the user confirms the requirement, move this issue back to backlog or write a new issue.

# expected result
- Implementation on hold until the requirement is confirmed
- If confirmed, write the behavior (ungrouped mode, sort criteria list) as a separate issue

# label
SQA_sk_0_0_0

# environment
OS: windows 10 pro
hostname: TBD

# assignee
