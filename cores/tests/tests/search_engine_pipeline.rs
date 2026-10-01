//! issues (real rg + sk through the engine):
//! - query_syntax/space-and/b964dd7b, exact/a34228c2, exclude/de8e3c3f, or/3c0b614e,
//!   path-include/7821beab, path-exclude/a26111e2, ext-include/b98e808f,
//!   ext-exclude/e2e26348, combined-query/78475d10 (AC-105)
//! - query_syntax/content-filter-path-contamination/208c79fb (engine payload)
//! - result_panel/markdown-results/b0eceafb (AC-106)
//! - search_engine/windows-path-with-spaces-parsing/49502d23
//! - search_engine/result-streaming/03669710

use common::log;
use skim_search::search_engine::ripgrep::parse_line;
use skim_search::search_engine::skim::prefilter_regex;
use tests::{big_workspace, files, keys, search, workspace};

fn run(root: &std::path::Path, q: &str) -> (Vec<String>, Vec<String>) {
    let out = search(root, q);
    assert!(out.error.is_none(), "{q}: {:?}", out.error);
    assert!(out.done.is_some(), "{q}: not done");
    let k = keys(root, &out.hits);
    let f = files(root, &out.hits);
    log::write("test", &format!("pipeline query={q:?} files={f:?} hits={k:?}"));
    (k, f)
}

#[test]
fn ac105_syntax_cases() {
    let root = workspace("ac105");

    // plain fuzzy: fuzzy-only line matches too
    let (k, _) = run(&root, "login");
    assert!(k.contains(&"src/auth/login.ts:2".into()));
    assert!(k.contains(&"src/fuzzy.ts:1".into()), "fuzzy l..o..g..i..n should match");

    // space = AND
    let (k, _) = run(&root, "login error");
    assert_eq!(k, vec!["src/auth/login.ts:5".to_string()]);

    // 'exact excludes fuzzy-only
    let (k, _) = run(&root, "'login");
    assert!(k.contains(&"src/auth/login.ts:2".into()));
    assert!(!k.contains(&"src/fuzzy.ts:1".into()));

    // !exclude alone: lines without "login" (case-insensitive)
    let (k, _) = run(&root, "!login");
    assert!(k.contains(&"docs/install.md:1".into()));
    assert!(!k.iter().any(|x| x == "src/auth/login.ts:2" || x == "docs/login-guide.md:1"));

    // login !test
    let (k, _) = run(&root, "login !test");
    assert!(k.contains(&"src/auth/login.ts:2".into()));

    // OR
    let (k, f) = run(&root, "login | logout");
    assert!(f.contains(&"src/auth/logout.ts".into()));
    assert!(k.contains(&"src/auth/login.ts:2".into()));

    // path / !path / ext / !ext
    let (_, f) = run(&root, "login path:src");
    assert!(f.iter().all(|p| p.starts_with("src/")), "{f:?}");
    let (_, f) = run(&root, "login !path:test");
    assert!(!f.iter().any(|p| p.starts_with("test/")), "{f:?}");
    assert!(f.contains(&"src/auth/login.ts".into()));
    let (_, f) = run(&root, "login ext:ts");
    assert!(f.iter().all(|p| p.ends_with(".ts")), "{f:?}");
    let (_, f) = run(&root, "login !ext:json");
    assert!(!f.iter().any(|p| p.ends_with(".json")), "{f:?}");
    assert!(f.contains(&"README.md".into()));

    // scope-only query: all lines in scope
    let (_, f) = run(&root, "path:src");
    assert!(f.iter().all(|p| p.starts_with("src/")) && f.len() == 4, "{f:?}");
    let (_, f) = run(&root, "ext:ts");
    assert!(f.iter().all(|p| p.ends_with(".ts")), "{f:?}");

    // combined
    let (k, f) = run(&root, "login !test path:src ext:ts");
    assert!(f.iter().all(|p| p.starts_with("src/") && p.ends_with(".ts")), "{f:?}");
    assert!(k.contains(&"src/auth/login.ts:2".into()));
    assert!(!f.contains(&"test/login.test.ts".into()));

    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn content_filter_not_polluted_by_path() {
    let root = workspace("contamination");
    // path contains "login" but the line does not → must be included
    let (k, _) = run(&root, "logout !login");
    log::write("test", &format!("contamination keys={k:?}"));
    assert!(k.contains(&"test/login.test.ts:2".into()), "{k:?}");
    assert!(k.contains(&"src/auth/logout.ts:1".into()));
    // a line containing login is excluded
    assert!(!k.contains(&"README.md:3".into()));
    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn markdown_todo() {
    let root = workspace("md");
    let (k, f) = run(&root, "TODO ext:md");
    assert!(f.iter().all(|p| p.ends_with(".md")));
    for want in ["docs/login-guide.md:5", "docs/install.md:2", "README.md:2", "docs/sample file.md:1"] {
        assert!(k.contains(&want.to_string()), "missing {want}: {k:?}");
    }
    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn windows_paths_with_spaces_and_drive_colon() {
    let h = parse_line(b"D:\\My Projects\\sample file.md\x0012:7:TODO: a:b:c\r\n").unwrap();
    assert_eq!(h.path, "D:\\My Projects\\sample file.md");
    assert_eq!((h.line, h.column), (12, 7));
    assert_eq!(h.text, "TODO: a:b:c");
    let h = parse_line(b"C:\\Projects\\test\\src\\main.rs\x003:1:fn main() {}\n").unwrap();
    assert_eq!(h.path, "C:\\Projects\\test\\src\\main.rs");

    // real directory with spaces
    let root = tests::temp_dir("My Projects");
    std::fs::write(root.join("sample file.md"), "x\nTODO: spaced: path\n").unwrap();
    let out = search(&root, "TODO");
    let k = keys(&root, &out.hits);
    log::write("test", &format!("spaces root={} keys={k:?}", root.display()));
    assert_eq!(k, vec!["sample file.md:2".to_string()]);
    assert_eq!(out.hits[0].column, 1);
    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn prefilter_regex_shapes() {
    assert_eq!(prefilter_regex("login"), "l.*o.*g.*i.*n");
    assert_eq!(prefilter_regex("'login"), "login");
    assert_eq!(prefilter_regex("!login"), "");
    assert_eq!(prefilter_regex("!test login"), "l.*o.*g.*i.*n");
    assert_eq!(prefilter_regex("'a.b | ^x"), "(?:a\\.b)|(?:^x)");
}

#[test]
fn streaming_sends_first_batch_before_done() {
    let root = big_workspace("stream", 400, 200); // 80k matching lines
    let out = search(&root, "login");
    log::write(
        "test",
        &format!(
            "streaming hits={} batches={} first_batch_ms={:?} done_ms={:?}",
            out.hits.len(),
            out.batches,
            out.first_batch.map(|d| d.as_secs_f64() * 1000.0),
            out.done.map(|d| d.as_secs_f64() * 1000.0)
        ),
    );
    assert_eq!(out.hits.len(), 80_000);
    assert!(out.batches > 1, "expected several batches");
    assert!(out.first_batch.unwrap() < out.done.unwrap());
    let _ = std::fs::remove_dir_all(&root);
}
