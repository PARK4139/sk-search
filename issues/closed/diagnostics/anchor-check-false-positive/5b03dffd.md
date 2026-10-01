# title
rules 링크 검사에서 코드 표기 `#[cfg(test)]`를 anchor로 오인

# pre-condition
- rules/*.md 안의 `` `#...` `` 표기를 같은 파일 제목 anchor로 검사하는 bash 스크립트

# steps
근거: rules/README.md#참조-표기 / 발생: rules 구조 리팩터링
1. same-file anchor 검사 실행

# actual result
- 발생: `NOANCHOR rules/implementation.md#[cfg(test)]` 1건.
- 원인: Rust attribute `` `#[cfg(test)]` `` 가 anchor 패턴과 일치.
- 조치: 수동 확인으로 false positive 판정. 실제 anchor 24개(파일 간 17, 같은 파일 7) 모두 유효.
- 결과: PASS
- 재발 방지: anchor 검사 패턴에서 `#[` 로 시작하는 표기 제외.

# expected result
- anchor 검사 결과에 실제 깨진 링크만 나타난다.

# label
SQA_sk_0_0_0

# environment
OS: windows 10 pro
hostname: TBD

# 담당자
