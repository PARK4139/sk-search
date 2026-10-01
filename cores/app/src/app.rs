//! UI model ↔ module wiring.
//!
//! All UI state lives on the Slint event-loop thread (`thread_local`). Search
//! workers post events back with `invoke_from_event_loop`; events of a
//! non-current generation are discarded (Latest Query Wins, handover §3.2).

use std::cell::RefCell;
use std::path::PathBuf;
use std::rc::Rc;
use std::time::{Duration, Instant};

use common::settings::{self, Settings};
use common::log;
use slint::{ComponentHandle, Model, ModelRc, SharedString, VecModel};

use crate::preview_editor::{self, Doc, SaveOutcome};
use crate::query_syntax;
use crate::result_panel::{RowKind, Results};
use crate::search_engine::engine::{Engine, Event, Request};
use crate::search_engine::executable::{self, Missing, Tool, RG, SK};
use crate::{external_open, shortcut, toast, view, MainWindow, ResultRow, ToastData};

/// Coalescing window between the last query change and search start (handover §3.1).
pub const COALESCE: Duration = Duration::from_millis(20);
pub const WINDOW_TITLE: &str = "skim-search";

pub struct Tools {
    pub rg: Result<Tool, Missing>,
    pub sk: Result<Tool, Missing>,
}

/// Detects rg/sk using settings, PATH and fallback locations next to the exe.
pub fn detect_tools() -> Tools {
    let settings = settings::load();
    let exe = std::env::current_exe().unwrap_or_default();
    let path_env = std::env::var_os("PATH");
    let rg = executable::detect(
        &RG,
        settings.rg_path.as_deref(),
        path_env.as_deref(),
        &executable::fallback_candidates(&RG, &exe),
    );
    let sk = executable::detect(
        &SK,
        settings.sk_path.as_deref(),
        path_env.as_deref(),
        &executable::fallback_candidates(&SK, &exe),
    );
    Tools { rg, sk }
}

pub fn tool_label(name: &str, tool: &Result<Tool, Missing>) -> String {
    match tool {
        Ok(t) => format!("{name} {}", t.version),
        Err(_) => format!("{name} 없음"),
    }
}

fn status_text(tools: &Tools, results: usize, secs: f64) -> String {
    format!(
        "{} · {} · {results} results · {secs:.2} sec",
        tool_label("rg", &tools.rg),
        tool_label("sk", &tools.sk)
    )
}

fn toast_model(window: &MainWindow) -> Rc<VecModel<ToastData>> {
    let current = window.get_toasts();
    thread_local! {
        static MODEL: RefCell<Option<Rc<VecModel<ToastData>>>> = const { RefCell::new(None) };
    }
    MODEL.with(|cell| {
        let mut cell = cell.borrow_mut();
        let model = cell.get_or_insert_with(|| Rc::new(VecModel::default())).clone();
        let is_same = current
            .as_any()
            .downcast_ref::<VecModel<ToastData>>()
            .is_some_and(|m| std::ptr::eq(m, Rc::as_ptr(&model)));
        if !is_same {
            window.set_toasts(ModelRc::from(model.clone()));
        }
        model
    })
}

pub fn push_toast(window: &MainWindow, kind: &str, title: &str, body: &str) {
    toast::push(&toast_model(window), kind, title, body);
}

/// Status bar versions + error toast per missing executable (FR-132, FR-133).
pub fn apply_tools(window: &MainWindow, tools: &Tools) {
    window.set_status(status_text(tools, 0, 0.0).into());
    for missing in [&tools.rg, &tools.sk].into_iter().filter_map(|t| t.as_ref().err()) {
        push_toast(
            window,
            "error",
            "실행파일 없음",
            &format!("{}.exe — 설정 경로, PATH, 기본 위치에서 찾지 못함", missing.name),
        );
    }
}

struct App {
    window: slint::Weak<MainWindow>,
    engine: Engine,
    tools: Tools,
    settings: Settings,
    root: String,
    query: String,
    coalesce: slint::Timer,
    changed_at: Instant,
    results: Results,
    results_gen: u64,
    rows: Rc<VecModel<ResultRow>>,
    doc: Option<Doc>,
    preview_seq: i32,
    last_root_error: Option<String>,
}

thread_local! {
    static APP: RefCell<Option<App>> = const { RefCell::new(None) };
}

fn with_app<R>(f: impl FnOnce(&mut App) -> R) -> Option<R> {
    APP.with(|a| a.borrow_mut().as_mut().map(f))
}

/// Builds the app state and connects UI callbacks to modules.
pub fn init(window: &MainWindow, tools: Tools) {
    apply_tools(window, &tools);
    let settings = settings::load();
    let root = settings.last_root().unwrap_or_default().to_owned();
    window.set_search_root(root.clone().into());
    let rows = Rc::new(VecModel::<ResultRow>::default());
    window.set_rows(ModelRc::from(rows.clone()));
    log::write("view", &format!("init root={root:?} editor={}", settings.editor()));

    APP.with(|a| {
        *a.borrow_mut() = Some(App {
            window: window.as_weak(),
            engine: Engine::new(),
            tools,
            settings,
            root,
            query: String::new(),
            coalesce: slint::Timer::default(),
            changed_at: Instant::now(),
            results: Results::default(),
            results_gen: 0,
            rows,
            doc: None,
            preview_seq: 0,
            last_root_error: None,
        })
    });

    window.on_query_edited(|text| {
        // Query parse happens immediately (handover §3.1); search starts after coalescing.
        query_syntax::parse_logged(&text);
        with_app(|app| {
            app.query = text.to_string();
            schedule(app, "query");
        });
    });
    window.on_root_edited(|text| {
        with_app(|app| {
            app.root = text.trim().to_string();
            schedule(app, "root");
        });
    });
    window.on_browse(|| {
        if let Some(path) = view::pick_folder(WINDOW_TITLE) {
            with_app(|app| {
                if let Some(w) = app.window.upgrade() {
                    w.set_search_root(path.clone().into());
                }
                app.root = path;
                schedule(app, "browse");
            });
        }
    });
    window.on_row_clicked(|group, index| {
        with_app(|app| {
            if index < 0 {
                app.results.toggle(group as usize);
                log::write("result_panel", &format!("toggle group={group}"));
            } else if app.results.select(group as usize, index as usize) {
                load_preview(app);
            }
            refresh_rows(app);
        });
    });
    window.on_nav(|delta| {
        with_app(|app| {
            if app.results.step(delta) {
                refresh_rows(app);
                load_preview(app);
            }
        });
    });
    window.on_preview_edited(|text| {
        with_app(|app| {
            let Some(w) = app.window.upgrade() else { return };
            let dirty = app.doc.as_ref().is_some_and(|d| d.text != text.as_str());
            w.set_preview_dirty(dirty);
            let lines = preview_editor::line_count(&text);
            if lines != preview_editor::line_count(&w.get_preview_gutter()) {
                set_gutter(&w, &text);
            }
        });
    });
    window.on_save_requested(|text| {
        with_app(|app| save(app, &text));
    });
    window.on_open_file(|| {
        with_app(open_file);
    });
    window.on_open_parent(|| {
        with_app(open_parent);
    });
    window.on_hide_requested(|| {
        with_app(|app| {
            if let Some(w) = app.window.upgrade() {
                log::write("shortcut", "Esc hide window");
                let _ = w.hide();
            }
        });
    });

    // initial search for the restored root is triggered by the first query edit
}

/// Kept for tests: init with auto-detected tools.
pub fn wire(window: &MainWindow) {
    init(window, detect_tools());
}

/// Global hotkey: show + foreground + focus query (FR-102, AC-101).
pub fn register_hotkey() -> bool {
    shortcut::register(|| {
        let _ = slint::invoke_from_event_loop(|| {
            with_app(|app| {
                if let Some(w) = app.window.upgrade() {
                    let _ = w.show();
                    shortcut::bring_to_front(WINDOW_TITLE);
                    w.invoke_focus_query();
                    log::write("shortcut", "window shown and query focused");
                }
            });
        });
    })
}

fn schedule(app: &mut App, reason: &str) {
    let gen = app.engine.bump();
    app.changed_at = Instant::now();
    log::write(
        "search_engine",
        &format!("change detected reason={reason} generation={gen} raw={:?} root={:?}", app.query, app.root),
    );
    if let Some(w) = app.window.upgrade() {
        w.set_stats("검색 중…".into());
    }
    app.coalesce.start(slint::TimerMode::SingleShot, COALESCE, move || {
        with_app(|app| start_search(app, gen));
    });
}

fn clear_results(app: &mut App, gen: u64, stats: &str) {
    app.results = Results::new(&app.root);
    app.results_gen = gen;
    refresh_rows(app);
    if let Some(w) = app.window.upgrade() {
        w.set_stats(stats.into());
        w.set_status(status_text(&app.tools, 0, 0.0).into());
    }
}

fn start_search(app: &mut App, gen: u64) {
    if !app.engine.is_current(gen) {
        return;
    }
    log::write(
        "search_engine",
        &format!("generation={gen} coalesced_ms={:.1}", app.changed_at.elapsed().as_secs_f64() * 1000.0),
    );
    if let Err(reason) = view::validate_root(&app.root) {
        log::write("view", &format!("generation={gen} root rejected: {reason}"));
        clear_results(app, gen, &format!("검색 경로 오류: {reason}"));
        if app.last_root_error.as_deref() != Some(reason.as_str()) && !app.root.is_empty() {
            if let Some(w) = app.window.upgrade() {
                push_toast(&w, "warning", "검색 경로 오류", &reason);
            }
        }
        app.last_root_error = Some(reason);
        return;
    }
    app.last_root_error = None;
    if app.settings.last_root() != Some(app.root.as_str()) {
        app.settings.remember_root(&app.root);
        settings::save(&app.settings);
    }
    if app.query.trim().is_empty() {
        clear_results(app, gen, "0 결과 · 0 파일 · 0.00초");
        return;
    }
    let (Ok(rg), Ok(sk)) = (&app.tools.rg, &app.tools.sk) else {
        clear_results(app, gen, "rg/sk 실행파일 없음");
        if let Some(w) = app.window.upgrade() {
            push_toast(&w, "error", "검색 불가", "rg.exe 또는 sk.exe 를 찾지 못함");
        }
        return;
    };
    let req = Request {
        generation: gen,
        query: query_syntax::parse(&app.query),
        root: PathBuf::from(&app.root),
        rg: rg.path.clone(),
        sk: sk.path.clone(),
        changed_at: app.changed_at,
    };
    app.engine.spawn(req, |ev| {
        let _ = slint::invoke_from_event_loop(move || {
            with_app(|app| on_event(app, ev));
        });
    });
}

fn on_event(app: &mut App, ev: Event) {
    let gen = match &ev {
        Event::Batch { generation, .. } | Event::Done { generation, .. } | Event::Error { generation, .. } => *generation,
    };
    if !app.engine.is_current(gen) {
        log::write("search_engine", &format!("discard stale event generation={gen} current={}", app.engine.current()));
        return;
    }
    if app.results_gen != gen {
        // first event of a new generation replaces the whole panel (handover §15)
        app.results = Results::new(&app.root);
        app.results_gen = gen;
    }
    let elapsed = app.changed_at.elapsed().as_secs_f64();
    match ev {
        Event::Batch { hits, .. } => {
            let had_selection = app.results.selected.is_some();
            app.results.add(hits);
            refresh_rows(app);
            set_stats(app, elapsed);
            if !had_selection {
                load_preview(app);
            }
        }
        Event::Done { total, elapsed, .. } => {
            refresh_rows(app);
            set_stats(app, elapsed.as_secs_f64());
            if app.results.total == 0 {
                clear_preview(app);
            }
            if let Some(w) = app.window.upgrade() {
                push_toast(&w, "info", "검색 완료", &format!("{} · {total} results", app.query.trim()));
            }
        }
        Event::Error { message, .. } => {
            refresh_rows(app);
            set_stats(app, elapsed);
            if let Some(w) = app.window.upgrade() {
                push_toast(&w, "error", "검색 오류", &message);
            }
        }
    }
}

fn set_stats(app: &App, secs: f64) {
    let Some(w) = app.window.upgrade() else { return };
    w.set_stats(format!("{} 결과 · {} 파일 · {secs:.2}초", app.results.total, app.results.groups.len()).into());
    w.set_status(status_text(&app.tools, app.results.total, secs).into());
}

fn refresh_rows(app: &App) {
    let mut selected_row = -1;
    let rows: Vec<ResultRow> = app
        .results
        .rows()
        .into_iter()
        .enumerate()
        .map(|(i, r)| {
            let g = &app.results.groups[r.group];
            match (r.kind, r.index) {
                (RowKind::Line, Some(li)) => {
                    let l = &g.lines[li];
                    let selected = app.results.selected == Some((r.group, li));
                    if selected {
                        selected_row = i as i32;
                    }
                    ResultRow {
                        is_group: false,
                        group: r.group as i32,
                        index: li as i32,
                        line: l.line as i32,
                        text: SharedString::from(l.text.trim()),
                        selected,
                        ..Default::default()
                    }
                }
                _ => ResultRow {
                    is_group: true,
                    group: r.group as i32,
                    index: -1,
                    badge: g.badge.clone().into(),
                    path: g.rel.clone().into(),
                    count: g.lines.len() as i32,
                    expanded: g.expanded,
                    ..Default::default()
                },
            }
        })
        .collect();
    app.rows.set_vec(rows);
    if let Some(w) = app.window.upgrade() {
        w.set_selected_row(selected_row);
    }
}

/// Gutter numbers and the line count used for the editor line height.
fn set_gutter(w: &MainWindow, text: &str) {
    w.set_preview_gutter(preview_editor::gutter(text).into());
    w.set_preview_line_count(preview_editor::line_count(text) as i32);
}

fn clear_preview(app: &mut App) {
    app.doc = None;
    if let Some(w) = app.window.upgrade() {
        w.set_preview_path("".into());
        w.set_preview_loc("".into());
        w.set_preview_text("".into());
        set_gutter(&w, "");
        w.set_preview_line(0);
        w.set_preview_dirty(false);
    }
}

fn load_preview(app: &mut App) {
    let Some(w) = app.window.upgrade() else { return };
    let Some((group, line)) = app.results.selected_line() else { return };
    let path = PathBuf::from(&group.path);
    let (rel, line_no, column) = (group.rel.clone(), line.line, line.column);

    let same_file = app.doc.as_ref().is_some_and(|d| d.path == path);
    if !same_file {
        if w.get_preview_dirty() {
            push_toast(&w, "warning", "저장하지 않은 변경 폐기", &w.get_preview_path());
        }
        match preview_editor::load(&path) {
            Ok(doc) => {
                w.set_preview_text(doc.text.clone().into());
                set_gutter(&w, &doc.text);
                w.set_preview_editable(true);
                app.doc = Some(doc);
            }
            Err(reason) => {
                let lossy = String::from_utf8_lossy(&std::fs::read(&path).unwrap_or_default()).replace("\r\n", "\n");
                set_gutter(&w, &lossy);
                w.set_preview_text(lossy.into());
                w.set_preview_editable(false);
                app.doc = None;
                log::write("preview_editor", &format!("load failed path={} reason={reason}", path.display()));
                push_toast(&w, "warning", "읽기 전용", &reason);
            }
        }
        w.set_preview_dirty(false);
    }
    w.set_preview_path(rel.into());
    w.set_preview_loc(format!("{line_no}:{column}").into());
    w.set_preview_line(line_no as i32);
    app.preview_seq += 1;
    w.set_preview_scroll_seq(app.preview_seq);
    let text = w.get_preview_text();
    let (start, end) = preview_editor::line_range(&text, line_no);
    let col = preview_editor::byte_offset(&text, line_no, column).clamp(start, end);
    w.invoke_select_preview(col as i32, end as i32);
    log::write(
        "preview_editor",
        &format!("show path={} line={line_no} column={column} same_file={same_file}", path.display()),
    );
}

fn save(app: &mut App, text: &str) {
    let Some(w) = app.window.upgrade() else { return };
    let Some(doc) = app.doc.as_mut() else {
        push_toast(&w, "warning", "저장 불가", "편집 가능한 파일이 없음");
        return;
    };
    let name = w.get_preview_path().to_string();
    match preview_editor::save(doc, text) {
        Ok(SaveOutcome::Saved) => {
            w.set_preview_dirty(false);
            push_toast(&w, "success", "저장 완료", &name);
        }
        Ok(SaveOutcome::Unchanged) => push_toast(&w, "info", "변경 사항 없음", &name),
        Ok(SaveOutcome::Conflict) => push_toast(&w, "warning", "파일이 외부에서 변경됨", &format!("{name} — 저장하지 않음")),
        Err(e) => push_toast(&w, "error", "저장 실패", &e),
    }
}

fn open_file(app: &mut App) {
    let Some(w) = app.window.upgrade() else { return };
    let Some((group, line)) = app.results.selected_line() else { return };
    let name = format!("{}:{}", group.rel.rsplit('/').next().unwrap_or(&group.rel), line.line);
    match external_open::open_file(app.settings.editor(), &PathBuf::from(&group.path), line.line, line.column) {
        Ok(_) => push_toast(&w, "info", "파일 열기", &name),
        Err(e) => push_toast(&w, "error", "파일 열기 실패", &e),
    }
}

fn open_parent(app: &mut App) {
    let Some(w) = app.window.upgrade() else { return };
    let Some((group, _)) = app.results.selected_line() else { return };
    match external_open::open_parent(&PathBuf::from(&group.path)) {
        Ok(_) => push_toast(&w, "info", "부모경로 열기", "Explorer에서 현재 파일 선택"),
        Err(e) => push_toast(&w, "error", "부모경로 열기 실패", &e),
    }
}

/// Test seam: delivers an engine event as if a worker had posted it.
pub fn deliver(ev: Event) {
    with_app(|app| on_event(app, ev));
}

/// (current generation, generations of live rg/sk processes)
pub fn engine_snapshot() -> (u64, Vec<u64>) {
    with_app(|app| (app.engine.current(), app.engine.live_generations())).unwrap_or_default()
}

/// Test helper: current row model snapshot.
pub fn rows_snapshot(window: &MainWindow) -> Vec<ResultRow> {
    let rows = window.get_rows();
    (0..rows.row_count()).filter_map(|i| rows.row_data(i)).collect()
}
