//! End-to-end UI flow with the real event loop, rg and sk (single test: the
//! testing backend can be initialised once per process).
//!
//! issues: search_engine/auto-search-on-input/2042b048,
//! stale-result-overwrites-latest/c910da32, stale-rg-sk-process-accumulation/42f62b22,
//! coalescing-20ms/96a87b9b, result_panel/full-panel-refresh/8dc99b0c,
//! group-by-file/84b81dce, status_bar/result-stats-not-updated/a2b0d185,
//! preview_editor/preview-content-mismatch/c854479d, ctrl-s-save/a982cbf4,
//! external-change-conflict/61a0bd37, match-scroll-highlight/b787fe7e,
//! duplicate-line-numbers/6cba8468, breadcrumb-not-updated/097089a9,
//! shortcut/arrow-key-navigation/450ba569, enter-ctrl-enter-actions/65041078,
//! esc-hide/c38bab5d, toast/exceeds-max-three/fe59e0e6, auto-dismiss/42a68a5c

use std::cell::RefCell;
use std::path::{Path, PathBuf};
use std::rc::Rc;
use std::time::{Duration, Instant};

use common::log;
use slint::platform::{Key, WindowEvent};
use slint::{ComponentHandle, Model, SharedString};
use skim_search::app::{self, Tools};
use skim_search::search_engine::engine::Event;
use skim_search::search_engine::executable::{detect, RG, SK};
use skim_search::search_engine::ripgrep::Hit;
use skim_search::MainWindow;

fn press(w: &MainWindow, text: SharedString) {
    w.window().dispatch_event(WindowEvent::KeyPressed { text: text.clone() });
    w.window().dispatch_event(WindowEvent::KeyReleased { text });
}

fn with_ctrl(w: &MainWindow, text: SharedString) {
    w.window().dispatch_event(WindowEvent::KeyPressed { text: Key::Control.into() });
    press(w, text);
    w.window().dispatch_event(WindowEvent::KeyReleased { text: Key::Control.into() });
}

fn toasts(w: &MainWindow) -> Vec<(String, String, String)> {
    let m = w.get_toasts();
    (0..m.row_count())
        .filter_map(|i| m.row_data(i))
        .map(|t| (t.kind.to_string(), t.title.to_string(), t.body.to_string()))
        .collect()
}

/// Highest toast id currently in the model.
fn max_toast_id(w: &MainWindow) -> i32 {
    let m = w.get_toasts();
    (0..m.row_count()).filter_map(|i| m.row_data(i)).map(|t| t.id).max().unwrap_or(0)
}

/// A "검색 완료" toast newer than `mark` exists (the search started after `mark` finished).
fn new_done(w: &MainWindow, mark: i32) -> bool {
    let m = w.get_toasts();
    (0..m.row_count()).filter_map(|i| m.row_data(i)).any(|t| t.id > mark && t.title == "검색 완료")
}

fn type_query(w: &MainWindow, q: &str) {
    w.set_query(q.into());
    w.invoke_query_edited(q.into());
}

fn fake_bin(dir: &Path) {
    std::fs::create_dir_all(dir).unwrap();
    std::fs::write(dir.join("code.cmd"), "@echo %* > \"%~dp0code-args.txt\"\r\n").unwrap();
    std::fs::write(dir.join("explorer.cmd"), "@echo %* > \"%~dp0explorer-args.txt\"\r\n").unwrap();
}

#[derive(Default)]
struct Probe {
    max_stale_procs: usize,
    max_live_procs: usize,
    max_toasts: usize,
    samples: usize,
}

type Step = Box<dyn FnMut(&MainWindow, &mut Ctx) -> bool>;

struct Ctx {
    root: PathBuf,
    bin: PathBuf,
    started: Instant,
    mark: Instant,
    gens: Vec<u64>,
    note: Vec<String>,
    /// highest toast id seen before an action; used to wait for a *new* completion toast
    toast_mark: i32,
}

#[test]
fn ui_flow() {
    i_slint_backend_testing::init_integration_test_with_system_time();

    let root = tests::workspace("ui-flow");
    // bulk files make rg run long enough for cancellation to matter
    let bulk = tests::big_workspace("ui-flow-bulk", 300, 200);
    std::fs::rename(&bulk, root.join("bulk")).unwrap();
    let bin = tests::temp_dir("ui-flow-bin");
    fake_bin(&bin);
    let settings_file = tests::temp_dir("ui-flow-settings").join("settings.json");
    std::fs::write(&settings_file, r#"{ "editor": "vscode" }"#).unwrap();
    std::env::set_var("SKIM_SEARCH_SETTINGS", &settings_file);
    std::env::set_var("SKIM_SEARCH_EXPLORER", bin.join("explorer.cmd"));
    std::env::set_var("PATH", format!("{};{}", bin.display(), std::env::var("PATH").unwrap_or_default()));

    let window = MainWindow::new().unwrap();
    let tools = Tools {
        rg: detect(&RG, Some(&tests::rg()), None, &[]),
        sk: detect(&SK, Some(&tests::sk()), None, &[]),
    };
    app::init(&window, tools);
    window.show().unwrap();

    let probe = Rc::new(RefCell::new(Probe::default()));
    let ctx = Ctx {
        root: root.clone(),
        bin: bin.clone(),
        started: Instant::now(),
        mark: Instant::now(),
        gens: Vec::new(),
        note: Vec::new(),
        toast_mark: 0,
    };

    let done_for = |w: &MainWindow, q: &str| toasts(w).iter().any(|t| t.1 == "검색 완료" && t.2.starts_with(&format!("{q} ·")));

    let mut steps: Vec<(&str, Step)> = vec![
        (
            "set root",
            Box::new(|w, c| {
                let r = c.root.display().to_string();
                w.set_search_root(r.clone().into());
                w.invoke_root_edited(r.into());
                c.mark = Instant::now();
                true
            }),
        ),
        (
            "rapid typing l→login (4ms apart)",
            Box::new({
                let mut i = 0;
                move |w, c| {
                    let seq = ["l", "lo", "log", "logi", "login"];
                    if c.mark.elapsed() >= Duration::from_millis(4 * i as u64) {
                        type_query(w, seq[i]);
                        c.gens.push(app::engine_snapshot().0);
                        i += 1;
                    }
                    i == seq.len()
                }
            }),
        ),
        ("wait login done", Box::new(move |w, _| done_for(w, "login"))),
        (
            "check login results (LQW)",
            Box::new(|w, c| {
                let rows = app::rows_snapshot(w);
                let lines: Vec<_> = rows.iter().filter(|r| !r.is_group).collect();
                assert!(!lines.is_empty());
                let all_login = rows.iter().filter(|r| !r.is_group).all(|r| {
                    let t = r.text.to_lowercase();
                    let mut it = t.chars();
                    "login".chars().all(|c| it.any(|x| x == c))
                });
                assert!(all_login, "non-login line in final rows");
                let stats = w.get_stats().to_string();
                let status = w.get_status().to_string();
                c.note.push(format!("login stats={stats:?} status={status:?} rows={}", rows.len()));
                assert!(stats.contains("결과") && stats.contains("파일"), "{stats}");
                assert!(status.contains("rg 15") && status.contains("results"), "{status}");
                assert!(!status.contains(" 0 results"), "{status}");
                // group headers carry badge + relative path + count; first expanded, others collapsed
                let groups: Vec<_> = rows.iter().filter(|r| r.is_group).collect();
                assert!(groups[0].expanded && groups.iter().skip(1).all(|g| !g.expanded));
                assert!(groups.iter().all(|g| !g.badge.is_empty() && g.count > 0 && !g.path.contains('\\')));
                true
            }),
        ),
        (
            "stale event injection is discarded",
            Box::new(|w, c| {
                let before = app::rows_snapshot(w).len();
                let old = c.gens[0];
                app::deliver(Event::Batch {
                    generation: old,
                    hits: vec![Hit { path: c.root.join("STALE.ts").display().to_string(), line: 1, column: 1, text: "STALE login".into() }],
                });
                let rows = app::rows_snapshot(w);
                assert_eq!(rows.len(), before);
                assert!(!rows.iter().any(|r| r.path.contains("STALE") || r.text.contains("STALE")));
                c.note.push(format!("stale generation={old} discarded"));
                true
            }),
        ),
        (
            "preview shows selected file",
            Box::new(|w, c| {
                let rows = app::rows_snapshot(w);
                let sel = rows.iter().find(|r| r.selected).expect("a selected row").clone();
                let group = rows.iter().find(|r| r.is_group && r.group == sel.group).unwrap();
                let path = c.root.join(group.path.as_str());
                let disk = std::fs::read_to_string(&path).unwrap().replace("\r\n", "\n");
                assert_eq!(w.get_preview_path(), group.path, "breadcrumb/header path");
                assert_eq!(w.get_preview_text().as_str(), disk, "preview content = selected file");
                assert_eq!(w.get_preview_line(), sel.line);
                assert!(w.get_preview_loc().starts_with(&format!("{}:", sel.line)));
                let gutter = w.get_preview_gutter().to_string();
                let expected: Vec<String> = (1..=disk.split('\n').count()).map(|n| n.to_string()).collect();
                assert_eq!(gutter, expected.join("\n"), "line numbers 1..N without duplicates");
                c.note.push(format!("preview path={} line={}", group.path, sel.line));
                true
            }),
        ),
        (
            "arrow down moves selection + preview",
            Box::new(|w, c| {
                let before = (w.get_selected_row(), w.get_preview_loc().to_string());
                press(w, Key::DownArrow.into());
                press(w, Key::DownArrow.into());
                let after = (w.get_selected_row(), w.get_preview_loc().to_string());
                assert!(after.0 > before.0, "{before:?} -> {after:?}");
                press(w, Key::UpArrow.into());
                assert!(w.get_selected_row() < after.0);
                c.note.push(format!("nav {before:?} -> {after:?} -> up {}", w.get_selected_row()));
                true
            }),
        ),
        (
            "Enter opens file in editor with line:column",
            Box::new(|w, c| {
                let _ = std::fs::remove_file(c.bin.join("code-args.txt"));
                press(w, Key::Return.into());
                c.mark = Instant::now();
                true
            }),
        ),
        (
            "wait code args",
            Box::new(|w, c| {
                let Ok(args) = std::fs::read_to_string(c.bin.join("code-args.txt")) else {
                    return false;
                };
                let loc = w.get_preview_loc().to_string();
                c.note.push(format!("code args={:?}", args.trim()));
                assert!(args.contains("--goto") && args.trim_end().ends_with(&format!(":{loc}")), "{args} / {loc}");
                true
            }),
        ),
        (
            "Ctrl+Enter opens parent with /select",
            Box::new(|w, c| {
                let _ = std::fs::remove_file(c.bin.join("explorer-args.txt"));
                with_ctrl(w, Key::Return.into());
                true
            }),
        ),
        (
            "wait explorer args",
            Box::new(|w, c| {
                let Ok(args) = std::fs::read_to_string(c.bin.join("explorer-args.txt")) else {
                    return false;
                };
                let rel = w.get_preview_path().to_string().replace('/', "\\");
                c.note.push(format!("explorer args={:?}", args.trim()));
                assert!(args.starts_with("/select,\"") && args.contains(&rel), "{args}");
                true
            }),
        ),
        (
            "edit + Ctrl+S saves",
            Box::new(|w, c| {
                let path = c.root.join(w.get_preview_path().as_str());
                let edited = format!("{}// edited by ui_flow\n", w.get_preview_text());
                w.set_preview_text(edited.clone().into());
                w.invoke_preview_edited(edited.clone().into());
                assert!(w.get_preview_dirty());
                with_ctrl(w, "s".into());
                let disk = std::fs::read_to_string(&path).unwrap();
                assert_eq!(disk, edited);
                assert!(toasts(w).iter().any(|t| t.0 == "success" && t.1 == "저장 완료"));
                assert!(!w.get_preview_dirty());
                c.note.push(format!("saved {}", path.display()));
                true
            }),
        ),
        (
            "external change blocks save",
            Box::new(|w, c| {
                let path = c.root.join(w.get_preview_path().as_str());
                std::fs::write(&path, "changed outside\n").unwrap();
                let edited = format!("{}// second edit\n", w.get_preview_text());
                w.set_preview_text(edited.clone().into());
                with_ctrl(w, "s".into());
                assert_eq!(std::fs::read_to_string(&path).unwrap(), "changed outside\n");
                assert!(toasts(w).iter().any(|t| t.0 == "warning" && t.1 == "파일이 외부에서 변경됨"));
                c.note.push("conflict detected, file untouched".into());
                true
            }),
        ),
        (
            "new query replaces whole panel",
            Box::new(|w, _| {
                type_query(w, "TODO ext:md");
                true
            }),
        ),
        ("wait md done", Box::new(move |w, _| done_for(w, "TODO ext:md"))),
        (
            "md results only",
            Box::new(|w, c| {
                let rows = app::rows_snapshot(w);
                let groups: Vec<_> = rows.iter().filter(|r| r.is_group).map(|r| r.path.to_string()).collect();
                assert!(!groups.is_empty() && groups.iter().all(|p| p.ends_with(".md")), "{groups:?}");
                assert!(rows.iter().filter(|r| r.is_group).all(|g| g.badge == "MD"));
                assert!(w.get_preview_path().ends_with(".md"));
                assert!(w.get_stats().starts_with(&format!("{} 결과", rows.iter().filter(|r| !r.is_group).count()))
                    || rows.iter().any(|r| r.is_group && !r.expanded));
                c.note.push(format!("md groups={groups:?} stats={}", w.get_stats()));
                true
            }),
        ),
        (
            "group header click toggles; spaced path opens in preview",
            Box::new(|w, c| {
                let rows = app::rows_snapshot(w);
                let g = rows.iter().find(|r| r.is_group && r.path.as_str() == "docs/sample file.md").expect("spaced file group").clone();
                let expanded_of = |w: &MainWindow| app::rows_snapshot(w).iter().find(|r| r.is_group && r.group == g.group).unwrap().expanded;
                let start = g.expanded;
                w.invoke_row_clicked(g.group, -1);
                assert_eq!(expanded_of(w), !start, "toggled");
                w.invoke_row_clicked(g.group, -1);
                assert_eq!(expanded_of(w), start, "toggled back");
                if !start {
                    w.invoke_row_clicked(g.group, -1);
                }
                let rows = app::rows_snapshot(w);
                let line = rows.iter().find(|r| !r.is_group && r.group == g.group).expect("line row").clone();
                w.invoke_row_clicked(line.group, line.index);
                assert_eq!(w.get_preview_path().as_str(), "docs/sample file.md");
                let disk = std::fs::read_to_string(c.root.join("docs").join("sample file.md")).unwrap();
                assert_eq!(w.get_preview_text().as_str(), disk);
                c.note.push("toggle + spaced path preview ok".into());
                true
            }),
        ),
        (
            "invalid root: no search, warning toast",
            Box::new(|w, c| {
                let bad = c.root.join("README.md").display().to_string();
                w.set_search_root(bad.clone().into());
                w.invoke_root_edited(bad.into());
                c.mark = Instant::now();
                true
            }),
        ),
        (
            "wait invalid root handled",
            Box::new(|w, c| {
                if c.mark.elapsed() < Duration::from_millis(60) {
                    return false;
                }
                assert!(w.get_stats().starts_with("검색 경로 오류"), "{}", w.get_stats());
                assert!(app::rows_snapshot(w).is_empty());
                assert!(toasts(w).iter().any(|t| t.0 == "warning" && t.1 == "검색 경로 오류"));
                c.note.push(format!("invalid root stats={}", w.get_stats()));
                true
            }),
        ),
        (
            "root change reruns current query",
            Box::new(|w, c| {
                c.toast_mark = max_toast_id(w);
                let docs = c.root.join("docs").display().to_string();
                w.set_search_root(docs.clone().into());
                w.invoke_root_edited(docs.into());
                true
            }),
        ),
        // a completion toast newer than the root change (the previous "TODO ext:md" toast may still be visible)
        ("wait rerun done", Box::new(|w, c| new_done(w, c.toast_mark))),
        (
            "rerun results are from new root only",
            Box::new(|w, c| {
                let groups: Vec<String> = app::rows_snapshot(w).iter().filter(|r| r.is_group).map(|r| r.path.to_string()).collect();
                assert!(groups.iter().all(|p| !p.contains('/') || !p.starts_with("docs/")), "paths relative to new root: {groups:?}");
                assert!(groups.contains(&"login-guide.md".to_string()) && !groups.contains(&"README.md".to_string()), "{groups:?}");
                c.note.push(format!("rerun groups={groups:?}"));
                true
            }),
        ),
        (
            "search error shows error toast",
            Box::new(|w, c| {
                let (current, _) = app::engine_snapshot();
                app::deliver(Event::Error { generation: current, message: "rg: permission denied".into() });
                assert!(toasts(w).iter().any(|t| t.0 == "error" && t.1 == "검색 오류" && t.2.contains("permission denied")));
                c.note.push("error toast ok".into());
                c.mark = Instant::now();
                true
            }),
        ),
        (
            "toasts auto dismiss",
            Box::new(|w, c| {
                if !toasts(w).is_empty() {
                    assert!(c.mark.elapsed() < Duration::from_secs(10), "toasts not dismissed: {:?}", toasts(w));
                    return false;
                }
                c.note.push(format!("toasts dismissed after {:?}", c.mark.elapsed()));
                true
            }),
        ),
        (
            "Esc hides window",
            Box::new(|w, c| {
                press(w, Key::Escape.into());
                assert!(!w.window().is_visible());
                c.note.push("esc hid window".into());
                true
            }),
        ),
    ];

    let timer = slint::Timer::default();
    let weak = window.as_weak();
    let probe2 = probe.clone();
    let ctx = Rc::new(RefCell::new(ctx));
    let ctx2 = ctx.clone();
    let mut idx = 0usize;
    timer.start(slint::TimerMode::Repeated, Duration::from_millis(1), move || {
        if idx >= steps.len() {
            return;
        }
        let w = weak.upgrade().unwrap();
        let mut c = ctx2.borrow_mut();
        {
            let (current, live) = app::engine_snapshot();
            let mut p = probe2.borrow_mut();
            p.samples += 1;
            p.max_live_procs = p.max_live_procs.max(live.len());
            p.max_stale_procs = p.max_stale_procs.max(live.iter().filter(|&&g| g < current).count());
            p.max_toasts = p.max_toasts.max(w.get_toasts().row_count());
        }
        assert!(c.started.elapsed() < Duration::from_secs(60), "timeout at step {}", steps[idx].0);
        if (steps[idx].1)(&w, &mut c) {
            log::write("test", &format!("ui_flow step ok: {} at {:?}", steps[idx].0, c.started.elapsed()));
            idx += 1;
            if idx == steps.len() {
                slint::quit_event_loop().unwrap();
            }
        }
    });
    slint::run_event_loop_until_quit().unwrap();

    let p = probe.borrow();
    let c = ctx.borrow();
    for n in &c.note {
        log::write("test", &format!("ui_flow {n}"));
    }
    log::write(
        "test",
        &format!(
            "ui_flow probe samples={} max_live_procs={} max_stale_procs={} max_toasts={} typed_generations={:?}",
            p.samples, p.max_live_procs, p.max_stale_procs, p.max_toasts, c.gens
        ),
    );
    assert_eq!(p.max_stale_procs, 0, "old-generation rg/sk alive after bump");
    assert!(p.max_toasts <= 3, "more than 3 toasts");
    let _ = std::fs::remove_dir_all(&root);
}
