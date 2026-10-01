# title
one-shot CD 배포 위치를 3rd_party/skim-search/{SHA}/ 로 변경 (releases 단계 제거)

# pre-condition
- 기존 배포 위치 `3rd_party/skim-search/releases/{SHA}/`

# steps
근거: 사용자 요청 (2026-10-01): "fix path 3rd_party/skim-search/releases/ > 3rd_party/skim-search/" / rules/one-shot.md#공유-폴더-배포
1. `one-shot.cmd --bump patch` 실행 후 배포 폴더 확인

# actual result
PASS (2026-10-02, one-shot run `ref/actual/logs/one-shot/20261002-013512-7acafd6a`).
- 게시 위치: `3rd_party/skim-search/76e944e9f66a343a02a5514665b39131406e9349/` — `skim-search-0.2.4-76e944e9-windows-x64.zip`, `manifest.json`, `SHA256SUMS`, security 보고서. package_sha256 dbe78379….
- `3rd_party/skim-search/releases/` 없음. 테스트: `test_full_wrappers_push_verified_sha_to_temporary_remote`(릴리스 부모 = `shared/skim-search`) PASS.

# expected result
- 패키지 ZIP, `manifest.json`, `SHA256SUMS` 가 `3rd_party/skim-search/{전체 SHA}/` 에 게시된다. `releases` 폴더를 만들지 않는다.

# label
SQA_sk_0_0_0

# environment
OS: windows 10 pro
hostname: TBD

# 담당자
