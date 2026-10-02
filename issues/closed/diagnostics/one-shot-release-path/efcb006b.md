# title
Change the one-shot CD deploy location to 3rd_party/skim-search/{SHA}/ (drop the releases level)

# pre-condition
- Previous deploy location `3rd_party/skim-search/releases/{SHA}/`

# steps
Source: user request (2026-10-01): "fix path 3rd_party/skim-search/releases/ > 3rd_party/skim-search/" / rules/one-shot.md#shared-folder-deployment
1. Run `one-shot.cmd --bump patch` and check the deploy folder

# actual result
PASS (2026-10-02, one-shot run `ref/actual/logs/one-shot/20261002-013512-7acafd6a`).
- Published to `3rd_party/skim-search/76e944e9f66a343a02a5514665b39131406e9349/` — `skim-search-0.2.4-76e944e9-windows-x64.zip`, `manifest.json`, `SHA256SUMS`, security report. package_sha256 dbe78379….
- No `3rd_party/skim-search/releases/`. Test: `test_full_wrappers_push_verified_sha_to_temporary_remote` (release parent = `shared/skim-search`) PASS.

# expected result
- The package ZIP, `manifest.json` and `SHA256SUMS` are published to `3rd_party/skim-search/{full SHA}/`. No `releases` folder is created.

# label
SQA_sk_0_0_0

# environment
OS: windows 10 pro
hostname: TBD

# assignee
