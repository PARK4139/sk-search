# title
one-shot CD 배포 위치를 3rd_party/skim-search/{SHA}/ 로 변경 (releases 단계 제거)

# pre-condition
- 기존 배포 위치 `3rd_party/skim-search/releases/{SHA}/`

# steps
근거: 사용자 요청 (2026-10-01): "fix path 3rd_party/skim-search/releases/ > 3rd_party/skim-search/" / rules/one-shot.md#공유-폴더-배포
1. `one-shot.cmd --bump patch` 실행 후 배포 폴더 확인

# actual result
미검증 — 구현 완료(`cd()`), 테스트 `test_full_wrappers_push_verified_sha_to_temporary_remote` 에서 배포 폴더 부모 = `skim-search` PASS.

# expected result
- 패키지 ZIP, `manifest.json`, `SHA256SUMS` 가 `3rd_party/skim-search/{전체 SHA}/` 에 게시된다. `releases` 폴더를 만들지 않는다.

# label
SQA_sk_0_0_0

# environment
OS: windows 10 pro
hostname: TBD

# 담당자
