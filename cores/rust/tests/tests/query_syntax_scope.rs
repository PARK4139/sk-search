//! issue: query_syntax/scope-token-separation/ef0463d0

use std::collections::BTreeSet;
use std::path::{Path, PathBuf};
use std::process::Command;

use common::log;
use i_slint_backend_testing::ElementHandle;
use skim_search::query_syntax::{parse, Scope};
use skim_search::{app, MainWindow};

fn s(v: &[&str]) -> Vec<String> {
    v.iter().map(|x| x.to_string()).collect()
}

#[test]
fn issue_case_1_skim_and_scope_split() {
    let p = parse("login !test path:src ext:ts");
    log::write(
        "test",
        &format!("case1 {p:?} globs={:?}", p.scope.rg_globs()),
    );
    assert_eq!(p.skim, "login !test");
    assert_eq!(
        p.scope,
        Scope {
            include_paths: s(&["src"]),
            include_exts: s(&["ts"]),
            ..Default::default()
        }
    );
}

#[test]
fn issue_case_2_or_expression_with_ext() {
    let p = parse("login | logout ext:md");
    log::write(
        "test",
        &format!("case2 {p:?} globs={:?}", p.scope.rg_globs()),
    );
    assert_eq!(p.skim, "login | logout");
    assert_eq!(
        p.scope,
        Scope {
            include_exts: s(&["md"]),
            ..Default::default()
        }
    );
}

#[test]
fn negated_scope_tokens() {
    let p = parse("login !path:test !ext:json");
    assert_eq!(p.skim, "login");
    assert_eq!(p.scope.exclude_paths, s(&["test"]));
    assert_eq!(p.scope.exclude_exts, s(&["json"]));
}

#[test]
fn non_scope_tokens_stay_in_skim() {
    // exact-quoted and non-prefix tokens are content conditions
    let p = parse("'path:src mypath:x !login");
    assert_eq!(p.skim, "'path:src mypath:x !login");
    assert!(p.scope.is_empty());
}

#[test]
fn empty_scope_value_is_dropped() {
    // user is still typing `path:`
    let p = parse("login path:");
    assert_eq!(p.skim, "login");
    assert!(p.scope.is_empty());
}

#[test]
fn scope_never_reaches_skim_expression() {
    for q in [
        "login !test path:src ext:ts",
        "TODO ext:md",
        "path:src",
        "!path:test",
        "ext:ts",
        "!ext:json",
        "login | logout ext:md",
    ] {
        let p = parse(q);
        let leaked = p.skim.split_whitespace().any(|t| {
            let t = t.trim_start_matches('!');
            t.starts_with("path:") || t.starts_with("ext:")
        });
        log::write("test", &format!("no_leak raw={q:?} skim={:?}", p.skim));
        assert!(!leaked, "{q} -> {}", p.skim);
    }
}

#[test]
fn rg_globs_combine_includes_as_and() {
    assert_eq!(
        parse("x path:src ext:ts").scope.rg_globs(),
        s(&["**/src/**/*.ts"])
    );
    assert_eq!(parse("x path:src").scope.rg_globs(), s(&["**/src/**"]));
    assert_eq!(parse("x ext:md").scope.rg_globs(), s(&["*.md"]));
    assert_eq!(
        parse("x path:src path:docs ext:ts !path:test !ext:json")
            .scope
            .rg_globs(),
        s(&[
            "**/src/**/*.ts",
            "**/docs/**/*.ts",
            "!**/test/**",
            "!*.json"
        ])
    );
}

fn rg_path() -> PathBuf {
    tests::rg()
}

/// Common committed fixture + scope-specific path traps.
fn workspace() -> PathBuf {
    let root = std::env::temp_dir().join(format!("skim-search-scope-{}", std::process::id()));
    tests::copy_sample_tree(&root);
    for f in ["src/readme.md", "srcfile.ts"] {
        let p = root.join(f);
        std::fs::create_dir_all(p.parent().unwrap()).unwrap();
        std::fs::write(p, "login logout TODO\n").unwrap();
    }
    root
}

fn rg_files(root: &Path, scope: &Scope) -> BTreeSet<String> {
    let mut cmd = Command::new(rg_path());
    // Scope only: list files, no content match (fixture contents vary).
    cmd.arg("--files");
    for g in scope.rg_globs() {
        cmd.args(["--glob", &g]);
    }
    let out = cmd.current_dir(root).output().unwrap();
    String::from_utf8_lossy(&out.stdout)
        .lines()
        .map(|l| l.replace('\\', "/"))
        .collect()
}

#[test]
fn rg_applies_scope() {
    let root = workspace();
    let cases: [(&str, &[&str]); 4] = [
        (
            "login !test path:src ext:ts",
            &["src/api/auth.ts", "src/auth/login.ts", "src/auth/logout.ts", "src/fuzzy.ts"],
        ),
        (
            "login | logout ext:md",
            &[
                "README.md",
                "docs/install.md",
                "docs/login-guide.md",
                "docs/sample file.md",
                "src/readme.md",
            ],
        ),
        (
            "login !path:test !ext:json",
            &[
                "README.md",
                "docs/install.md",
                "docs/login-guide.md",
                "docs/sample file.md",
                "src/api/auth.ts",
                "src/auth/login.ts",
                "src/auth/logout.ts",
                "src/fuzzy.ts",
                "src/readme.md",
                "srcfile.ts",
            ],
        ),
        (
            "login path:src/auth",
            &["src/auth/login.ts", "src/auth/logout.ts"],
        ),
    ];
    for (q, expected) in cases {
        let p = parse(q);
        let got = rg_files(&root, &p.scope);
        log::write(
            "test",
            &format!(
                "rg_scope raw={q:?} globs={:?} files={got:?}",
                p.scope.rg_globs()
            ),
        );
        let expected: BTreeSet<String> = expected.iter().map(|x| x.to_string()).collect();
        assert_eq!(got, expected, "{q}");
    }
    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn ui_query_edit_logs_parse() {
    i_slint_backend_testing::init_no_event_loop();
    let window = MainWindow::new().unwrap();
    app::wire(&window);

    let marker = format!("ui-{} path:src ext:ts", std::process::id());
    let input = ElementHandle::find_by_accessible_label(&window, "query-input")
        .next()
        .expect("query-input");
    input.set_accessible_value(marker.clone());

    let text = std::fs::read_to_string(log::log_path()).unwrap();
    let line = text
        .lines()
        .rev()
        .find(|l| l.contains("[query_syntax]") && l.contains(&marker))
        .unwrap_or_else(|| panic!("no [query_syntax] log line for {marker}"));
    log::write("test", &format!("ui_query_edit found log line: {line}"));
    assert!(line.contains(&format!("skim=\"ui-{}\"", std::process::id())));
    assert!(line.contains("rg_globs=[\"**/src/**/*.ts\"]"));
}
