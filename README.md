# sk-search

Native Windows file-content search (Rust + Slint) using ripgrep and skim.

## Layout

```text
cores/rust/     Cargo workspace: app (skim-search.exe), common, tests
cores/python/   uv project: skim_search (one-shot pipeline, security policy, issue IDs), tests, benchmarks
scripts/        entry points (.cmd -> .ps1 -> Python module)
configs/        one-shot.json, security-exceptions.json
issues/ rules/ ref/
target/         build output (not committed)
```

## Commands

```text
scripts\test.cmd [quick]            build, cargo test, clippy (+ Python tests, e2e, benchmarks)
scripts\one-shot.cmd --bump patch   commit -> CI -> CD -> security -> push
scripts\security-policy.cmd         pre-push repository policy check
scripts\issue-id.cmd                reserve a new issue ID
```

Rules: `rules/README.md`.
