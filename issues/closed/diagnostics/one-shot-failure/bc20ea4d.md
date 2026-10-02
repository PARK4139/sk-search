# title
one-shot ci stage failed: command failed: uv.exe (exit 1); command

# pre-condition
- Target full SHA: fadb554f92bbb64bc7aef74cf9b218135f9573b9
- Run ID: `20261002-131635-3158b004`

# steps
Source: rules/one-shot.md#stages-and-failure-handling / Origin: recorded automatically by one-shot
1. `scripts\one-shot.cmd --bump patch`

# actual result
- Failure match: `sha=fadb554f92bbb64bc7aef74cf9b218135f9573b9 | stage=ci | item=command failed: uv.exe (exit 1); command`
- Failed stage / item: ci / command failed: C:\Users\<user>\Downloads\CavemanDrive\3rd_party\pk_system\uv.exe (exit 1); command-ff4527531a98
- Error summary (masked):
```text
13:17:55.262 latency summary rg_spawn_ms: n=10 avg=22.9 p50=22.8 p90=23.4 min=22.4 max=23.4
13:17:55.263 CHECK PASS [search_engine/coalescing-20ms] search start p50=22.8ms (target <=25)
13:17:55.263 latency summary first_result_ms: n=10 avg=45.2 p50=44.6 p90=47.0 min=43.7 max=47.9
13:17:55.263 CHECK PASS [diagnostics/first-result-latency] first result p50=44.6ms p90=47.0ms (target <=50)
13:17:55.263 latency summary completed_ms: n=10 avg=69.0 p50=68.9 p90=70.6 min=65.6 max=72.2
13:17:55.770 set query-input = 'TODO ext:md'
13:17:55.771 done 'TODO ext:md': 1790914675344 [toast] push id=20 kind=info title=검색 완료 body=TODO ext:md · 4 results count=3
13:17:56.493 set preview-editor = '# my-project\nTODO: write readme\nlogin and logout supported\nTODO: edited in e2e\n'
13:17:56.795 CHECK PASS [result_panel/markdown-results] 1790914675316 [preview_editor] show path=C:\Users\Public\skim-search-e2e\README.md line=2 column=1 same_file=false | 1790914675344 [status_bar] generation=31 stats="4 결과 · 4 파일 · 0.07초" status="rg 15.2.0 · sk 5.6.6 · 4 results · 0.07 sec"
13:17:56.807 CHECK PASS [preview_editor/edit-text-overlap] edited text kept in single editor (screenshot e2e_md_edit)
13:17:56.836 screenshot e2e_md_edit.png
13:17:57.050 keys (17, 83)
13:17:57.102 CHECK PASS [preview_editor/ctrl-s-save] keyboard Ctrl+S wrote README.md
13:17:57.341 screenshot e2e_toast_success.png
13:17:57.847 set query-input = 'logout'
13:17:57.848 done 'logout': 1790914677416 [toast] push id=22 kind=info title=검색 완료 body=logout · 8 results count=3
13:17:58.049 CHECK FAIL [e2e_ui] aborted: COMError(-2147220991, '이벤트에서 가입자를 불러낼 수 없습니다.', (None, None, None, 0, None))
13:17:58.049 killed own skim-search pid=18252
13:17:58.052 removed workspace C:\Users\Public\skim-search-e2e exists=False
13:17:58.053 e2e_ui summary: pass=8 warn=0 fail=1
```
- Cause: no product or pipeline defect. The user needed the keyboard and interrupted this normal run during the CI UI e2e (`tests.e2e.ui`); the e2e module exited 1, so CI failed. Everything before it passed (cargo build/test/clippy, Python tests, e2e detection; latency within target in the summary above).
- Action: the same commit fadb554 was pushed with `scripts\one-shot.cmd --urgent-backup` (run `20261002-131822-9ebdc3ec`, security_policy passed, origin/main == 35604c0). Screenshot changes left by the interrupted e2e were reverted.
- Re-verification: PASS for the pipeline behavior — the interrupted run's version reservation behaved as designed: `version_superseded` 0.7.1 of the earlier failed SHA 7adc027, then a new reservation for fadb554 (diagnostics/one-shot-version-on-failure). A full normal run (including UI e2e) is deferred to a time when the keyboard is free.
- Occurred 2026-10-02T13:17:58+09:00: run `20261002-131635-3158b004`, SHA fadb554f92bbb64bc7aef74cf9b218135f9573b9, exit code 1, evidence `ref/actual/logs/one-shot/20261002-131635-3158b004/command-ff4527531a98.stderr.log, command-ff4527531a98.stdout.log`

# expected result
- The one-shot ci stage passes for the same SHA. After the fix is verified, move this file to closed and remove its priority row.

# label
SQA_sk_0_0_0

# environment
OS: win32
hostname: TBD

# assignee
