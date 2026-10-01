//! issue: search_engine/rg-sk-executable-detection/5ea570b8
//! Cases: A configured path, B PATH only, fallback, C missing; detected tools can search.

use std::ffi::OsString;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

use common::log;
use i_slint_backend_testing::ElementHandle;
use skim_search::app::{apply_tools, Tools};
use skim_search::search_engine::executable::{
    detect, fallback_candidates, Missing, Source, RG, SK,
};
use skim_search::MainWindow;

/// `CavemanDrive/3rd_party` found by walking up from this crate.
fn third_party() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .map(|d| d.join("3rd_party"))
        .find(|d| d.join("ripgrep").join("rg.exe").is_file())
        .expect("3rd_party/ripgrep/rg.exe not found above cores/tests")
}

fn temp_dir(name: &str) -> PathBuf {
    let d = std::env::temp_dir().join(format!("skim-search-test-{}-{name}", std::process::id()));
    std::fs::create_dir_all(&d).unwrap();
    d
}

fn path_env(dirs: &[&Path]) -> OsString {
    std::env::join_paths(dirs).unwrap()
}

#[test]
fn case_a_configured_path_wins() {
    let rg = third_party().join("ripgrep").join("rg.exe");
    // PATH also has rg, configured must still win.
    let env = path_env(&[&third_party().join("ripgrep")]);
    let tool = detect(&RG, Some(&rg), Some(&env), &[]).unwrap();
    log::write(
        "test",
        &format!("case_a source={:?} version={}", tool.source, tool.version),
    );
    assert_eq!(tool.source, Source::Configured);
    assert_eq!(tool.path, rg);
    assert!(!tool.version.is_empty());
}

#[test]
fn case_b_path_used_when_configured_missing() {
    let empty = temp_dir("b-empty");
    let env = path_env(&[&empty, &third_party().join("skim")]);
    let tool = detect(
        &SK,
        Some(Path::new(r"C:\does-not-exist\sk.exe")),
        Some(&env),
        &[],
    )
    .unwrap();
    log::write(
        "test",
        &format!("case_b source={:?} version={}", tool.source, tool.version),
    );
    assert_eq!(tool.source, Source::Path);
    assert!(!tool.version.is_empty());
}

#[test]
fn fallback_found_from_exe_ancestors() {
    let empty = temp_dir("fb-empty");
    // exe path under the repo (file itself need not exist)
    let exe = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("target")
        .join("debug")
        .join("skim-search.exe");
    let tool = detect(
        &RG,
        None,
        Some(&path_env(&[&empty])),
        &fallback_candidates(&RG, &exe),
    )
    .unwrap();
    log::write(
        "test",
        &format!(
            "fallback source={:?} path={}",
            tool.source,
            tool.path.display()
        ),
    );
    assert_eq!(tool.source, Source::Fallback);
}

#[test]
fn case_c_missing_reports_name() {
    let empty = temp_dir("c-empty");
    let err = detect(
        &RG,
        Some(Path::new(r"C:\does-not-exist\rg.exe")),
        Some(&path_env(&[&empty])),
        &[],
    )
    .unwrap_err();
    log::write("test", &format!("case_c reason={}", err.reason));
    assert_eq!(err.name, "rg");
    assert!(err.reason.contains("rg.exe"));
}

#[test]
fn case_c_ui_error_toast_and_status() {
    i_slint_backend_testing::init_no_event_loop();
    let window = MainWindow::new().unwrap();
    let tools = Tools {
        rg: Err(Missing {
            name: "rg",
            reason: "test".into(),
        }),
        sk: Err(Missing {
            name: "sk",
            reason: "test".into(),
        }),
    };
    apply_tools(&window, &tools);

    let status = window.get_status().to_string();
    let toasts: Vec<String> = ElementHandle::find_by_accessible_label(&window, "toast")
        .map(|t| {
            t.accessible_description()
                .map(|s| s.to_string())
                .unwrap_or_default()
        })
        .collect();
    log::write(
        "test",
        &format!("case_c_ui status={status} toasts={toasts:?}"),
    );
    assert!(status.starts_with("rg 없음 · sk 없음"));
    assert_eq!(toasts.len(), 2);
    assert!(toasts[0].starts_with("error:") && toasts[0].contains("rg.exe"));
    assert!(toasts[1].starts_with("error:") && toasts[1].contains("sk.exe"));
}

#[test]
fn detected_tools_can_search() {
    let tp = third_party();
    let rg = detect(&RG, Some(&tp.join("ripgrep").join("rg.exe")), None, &[]).unwrap();
    let sk = detect(&SK, Some(&tp.join("skim").join("sk.exe")), None, &[]).unwrap();

    let dir = temp_dir("search");
    std::fs::write(
        dir.join("a.ts"),
        "function login() {}\nfunction logout() {}\n",
    )
    .unwrap();

    let rg_out = Command::new(&rg.path)
        .args([
            "--line-number",
            "--column",
            "--no-heading",
            "--color",
            "never",
            "log",
        ])
        .arg(&dir)
        .output()
        .unwrap();
    let rg_text = String::from_utf8_lossy(&rg_out.stdout).to_string();

    let mut child = Command::new(&sk.path)
        .args(["--filter", "'login"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .unwrap();
    child
        .stdin
        .take()
        .unwrap()
        .write_all(rg_text.as_bytes())
        .unwrap();
    let sk_text = String::from_utf8_lossy(&child.wait_with_output().unwrap().stdout).to_string();

    log::write(
        "test",
        &format!(
            "search rg_lines={} sk_lines={}",
            rg_text.lines().count(),
            sk_text.lines().count()
        ),
    );
    assert_eq!(rg_text.lines().count(), 2);
    assert_eq!(sk_text.lines().count(), 1);
    assert!(sk_text.contains("login()"));
}
