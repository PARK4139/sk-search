# title
The preview current-line highlight is shifted down (line 81 selected → line 85 highlighted)

# pre-condition
- src/long.ts (163 lines), query `loginTarget` → line 81 selected

# steps
Source: handover §17 (match line highlight) / Origin: first batch incl. search_engine/auto-search-on-input (e2e screenshot)
1. Search `loginTarget` in the release exe and check the preview

# actual result
- Occurred: the selection is on line 81 but the highlight bar is on line 85 (`ref/actual/screenshot/frames/e2e_scroll_highlight.png`, first capture).
- Cause: the line height came from the preferred-height of a separate probe Text → about 5% off from the TextInput's real line spacing, accumulating with the line number.
- Action: line-h = editor.preferred-height / preview-line-count (based on the editor's own layout), line-count set from Rust.
- Result: PASS — highlight on line 81, scrolled to the top third (recaptured).
- Prevention: line position calculations use measurements of the TextInput itself.

# expected result
- The highlight bar and the selected line match.

# label
SQA_sk_0_0_0

# environment
OS: windows 10 pro
hostname: TBD

# assignee
