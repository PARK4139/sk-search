//! issue: query_syntax/content-filter-path-contamination/208c79fb

use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

use common::log;
use skim_search::query_syntax::{skim_input_from_rg, SKIM_CONTENT_FILTER_ARGS};

fn tool_path(name: &str, file: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .map(|d| d.join("3rd_party").join(name).join(file))
        .find(|p| p.is_file())
        .unwrap_or_else(|| panic!("3rd_party/{name}/{file} not found"))
}

#[test]
fn skim_content_filter_ignores_path_and_line_metadata() {
    let root =
        std::env::temp_dir().join(format!("skim-search-content-filter-{}", std::process::id()));
    let path_match_only = root.join("src").join("login").join("logout.ts");
    let content_match_and_exclude = root.join("src").join("auth.ts");
    std::fs::create_dir_all(path_match_only.parent().unwrap()).unwrap();
    std::fs::create_dir_all(content_match_and_exclude.parent().unwrap()).unwrap();
    std::fs::write(&path_match_only, "logout();\n").unwrap();
    std::fs::write(&content_match_and_exclude, "logout(); // login\n").unwrap();

    let rg_output = Command::new(tool_path("ripgrep", "rg.exe"))
        .args([
            "--line-number",
            "--column",
            "--no-heading",
            "--color",
            "never",
            "logout",
        ])
        .arg(&root)
        .output()
        .unwrap();
    assert!(rg_output.status.success());
    let rg_records = String::from_utf8_lossy(&rg_output.stdout);
    let skim_input = skim_input_from_rg(&rg_records);
    assert_eq!(skim_input.lines().count(), 2);
    assert!(skim_input.lines().all(|line| line.split('\t').count() >= 4));

    let mut sk = Command::new(tool_path("skim", "sk.exe"))
        .arg("--filter")
        .arg("logout !login")
        .args(SKIM_CONTENT_FILTER_ARGS)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .unwrap();
    sk.stdin
        .take()
        .unwrap()
        .write_all(skim_input.as_bytes())
        .unwrap();
    let output = sk.wait_with_output().unwrap();
    let filtered = String::from_utf8_lossy(&output.stdout);
    assert_eq!(filtered.lines().count(), 1, "{filtered}");
    assert!(filtered.contains("login\\logout.ts"), "{filtered}");
    assert!(filtered.contains("\tlogout();"), "{filtered}");
    log::write(
        "test",
        &format!(
            "content_filter rg_records={} skim_input_records={} match_fields=4.. filtered_records={} selected_path_contains_login=true selected_content=logout_only",
            rg_records.lines().count(),
            skim_input.lines().count(),
            filtered.lines().count()
        ),
    );

    let _ = std::fs::remove_dir_all(root);
}
