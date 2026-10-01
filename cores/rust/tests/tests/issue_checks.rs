//! Non-UI checks for:
//! - diagnostics/query-parser-tests (handover §29 input list)
//! - preview_editor/ctrl-s-save (CRLF / encoding preserved)
//! - view/search-root-validation-missing
//! - toast/search-error-toast (engine error path with a failing rg)
//! - settings/recent-search-root-persistence, settings/open-editor-selection (settings file)
//! - result_panel/group-by-file (toggle / navigation model)

use common::log;
use common::settings::{self, Settings};
use skim_search::preview_editor::{self, SaveOutcome};
use skim_search::query_syntax::parse;
use skim_search::result_panel::{badge, Results};
use skim_search::search_engine::engine::{Engine, Event, Request};
use skim_search::search_engine::ripgrep::Hit;
use skim_search::view::validate_root;

/// (raw, skim expression, include paths, exclude paths, include exts, exclude exts)
type ParseCase = (&'static str, &'static str, &'static [&'static str], &'static [&'static str], &'static [&'static str], &'static [&'static str]);

#[test]
fn parser_handover_inputs() {
    let cases: [ParseCase; 12] = [
        ("login", "login", &[], &[], &[], &[]),
        ("login error", "login error", &[], &[], &[], &[]),
        ("'login", "'login", &[], &[], &[], &[]),
        ("!login", "!login", &[], &[], &[], &[]),
        ("login !test", "login !test", &[], &[], &[], &[]),
        ("login | logout", "login | logout", &[], &[], &[], &[]),
        ("path:src", "", &["src"], &[], &[], &[]),
        ("!path:test", "", &[], &["test"], &[], &[]),
        ("ext:ts", "", &[], &[], &["ts"], &[]),
        ("!ext:json", "", &[], &[], &[], &["json"]),
        ("login !test path:src ext:ts", "login !test", &["src"], &[], &["ts"], &[]),
        ("TODO ext:md", "TODO", &[], &[], &["md"], &[]),
    ];
    for (raw, skim, ip, ep, ie, ee) in cases {
        let p = parse(raw);
        log::write("test", &format!("parser raw={raw:?} skim={:?} scope={:?}", p.skim, p.scope));
        assert_eq!(p.skim, skim, "{raw}");
        assert_eq!(p.scope.include_paths, ip, "{raw}");
        assert_eq!(p.scope.exclude_paths, ep, "{raw}");
        assert_eq!(p.scope.include_exts, ie, "{raw}");
        assert_eq!(p.scope.exclude_exts, ee, "{raw}");
    }
}

#[test]
fn save_preserves_crlf_and_utf8() {
    let dir = tests::temp_dir("crlf");
    let path = dir.join("a.md");
    std::fs::write(&path, "한글 line1\r\nline2\r\n").unwrap();
    let mut doc = preview_editor::load(&path).unwrap();
    assert!(doc.crlf);
    assert_eq!(doc.text, "한글 line1\nline2\n");
    let edited = "한글 line1\nline2 edited\n";
    assert_eq!(preview_editor::save(&mut doc, edited).unwrap(), SaveOutcome::Saved);
    let disk = std::fs::read(&path).unwrap();
    log::write("test", &format!("crlf save bytes={:?}", String::from_utf8_lossy(&disk)));
    assert_eq!(disk, "한글 line1\r\nline2 edited\r\n".as_bytes());
    // unchanged save does not touch the file
    assert_eq!(preview_editor::save(&mut doc, edited).unwrap(), SaveOutcome::Unchanged);
    // non UTF-8 file is refused for editing
    std::fs::write(dir.join("cp949.txt"), [0xB0u8, 0xA1, 0x0A]).unwrap();
    assert!(preview_editor::load(&dir.join("cp949.txt")).is_err());
}

#[test]
fn root_validation() {
    let dir = tests::temp_dir("root");
    std::fs::write(dir.join("README.md"), "x").unwrap();
    let file = dir.join("README.md");
    let missing = dir.join("not-exist");
    let cases = [
        (dir.display().to_string(), true),
        (file.display().to_string(), false),
        (missing.display().to_string(), false),
        ("relative\\path".to_string(), false),
        (String::new(), false),
    ];
    for (root, ok) in cases {
        let r = validate_root(&root);
        log::write("test", &format!("root_validation root={root:?} result={r:?}"));
        assert_eq!(r.is_ok(), ok, "{root}");
    }
}

#[test]
fn rg_error_becomes_error_event() {
    // fake rg: writes to stderr and exits 2 without output
    let dir = tests::temp_dir("rg-error");
    let fake = dir.join("rg.cmd");
    std::fs::write(&fake, "@echo rg: permission denied 1>&2\r\n@exit /b 2\r\n").unwrap();
    let engine = Engine::new();
    let generation = engine.bump();
    let (tx, rx) = std::sync::mpsc::channel();
    engine.spawn(
        Request {
            generation,
            query: parse("login"),
            root: dir.clone(),
            rg: fake,
            sk: tests::sk(),
            changed_at: std::time::Instant::now(),
        },
        move |ev| {
            let _ = tx.send(ev);
        },
    );
    let ev = rx.recv_timeout(std::time::Duration::from_secs(10)).unwrap();
    log::write("test", &format!("rg_error event={ev:?}"));
    match ev {
        Event::Error { message, .. } => assert!(message.contains("permission denied"), "{message}"),
        other => panic!("expected error, got {other:?}"),
    }
}

#[test]
fn settings_roundtrip() {
    let dir = tests::temp_dir("settings");
    let file = dir.join("settings.json");
    std::env::set_var("SKIM_SEARCH_SETTINGS", &file);
    let mut s = Settings::default();
    assert_eq!(s.editor(), "system");
    s.editor = Some("cursor".into());
    s.remember_root(r"D:\Projects\a");
    s.remember_root(r"D:\Projects\b");
    s.remember_root(r"d:\projects\A"); // case-insensitive dedupe, moves to front
    settings::save(&s);
    let loaded = settings::load();
    log::write("test", &format!("settings_roundtrip loaded={loaded:?}"));
    assert_eq!(loaded, s);
    assert_eq!(loaded.last_root(), Some(r"d:\projects\A"));
    assert_eq!(loaded.recent_roots.len(), 2);
    assert_eq!(loaded.editor(), "cursor");
}

fn hit(path: &str, line: u32) -> Hit {
    Hit { path: path.into(), line, column: 1, text: format!("t{line}") }
}

#[test]
fn result_model_groups_toggle_and_step() {
    let mut r = Results::new(r"D:\p");
    r.add(vec![hit(r"D:\p\src\a.ts", 1), hit(r"D:\p\src\a.ts", 5), hit(r"D:\p\docs\b.md", 2)]);
    assert_eq!(r.groups.len(), 2);
    assert_eq!(r.groups[0].rel, "src/a.ts");
    assert_eq!((r.groups[0].badge.as_str(), r.groups[1].badge.as_str()), ("TS", "MD"));
    assert!(r.groups[0].expanded && !r.groups[1].expanded);
    assert_eq!(r.rows().len(), 4); // 2 headers + 2 lines of expanded group 0
    r.toggle(0);
    assert_eq!(r.rows().len(), 2);
    assert_eq!(r.selected, Some((0, 0)));
    assert!(r.step(1)); // (0,1)
    assert!(r.step(1)); // (1,0) expands group 1
    assert_eq!(r.selected, Some((1, 0)));
    assert!(r.groups[1].expanded);
    assert!(r.step(1)); // clamped at last
    assert_eq!(r.selected, Some((1, 0)));
    assert_eq!(badge("Makefile"), "TXT");
    log::write("test", "result_model groups/toggle/step ok");
}

/// query_syntax/file-token: `file:` include via rg glob, `!file:` via file-name filter.
#[test]
fn file_token() {
    let p = parse("login file:auth !file:test ext:ts path:src");
    assert_eq!(p.skim, "login");
    assert_eq!(p.scope.include_files, vec!["auth"]);
    assert_eq!(p.scope.exclude_files, vec!["test"]);
    assert_eq!(p.scope.rg_globs(), vec!["**/src/**/*auth*.ts".to_string()]);
    assert_eq!(parse("x file:auth").scope.rg_globs(), vec!["*auth*".to_string()]);
    assert!(p.scope.excludes_file(r"D:\p\src\login.test.ts"));
    assert!(!p.scope.excludes_file(r"D:\p\test\auth.ts"), "only the file name counts, not directories");

    let root = tests::workspace("file-token");
    std::fs::create_dir_all(root.join("src").join("test")).unwrap();
    std::fs::write(root.join("src").join("test").join("auth_helper.ts"), "login helper\n").unwrap();
    std::fs::write(root.join("src").join("auth.test.ts"), "login spec\n").unwrap();
    let out = tests::search(&root, "login file:auth !file:test");
    let f = tests::files(&root, &out.hits);
    log::write("test", &format!("file_token files={f:?}"));
    // name contains "auth", name does not contain "test"; a directory named test is fine
    assert!(f.contains(&"src/api/auth.ts".to_string()), "{f:?}");
    assert!(f.contains(&"src/test/auth_helper.ts".to_string()), "{f:?}");
    assert!(!f.contains(&"src/auth.test.ts".to_string()), "{f:?}");
    assert!(!f.contains(&"src/auth/login.ts".to_string()), "directory name must not count: {f:?}");
    let _ = std::fs::remove_dir_all(&root);
}

/// query_syntax/skim-input-from-rg-colon-fix
#[test]
fn skim_input_keeps_colons_in_text() {
    let out = skim_search::query_syntax::skim_input_from_rg(concat!(r"C:\a.ts", ":12:3:foo: bar\nsrc/b.ts:1:1:x\n"));
    log::write("test", &format!("skim_input colon out={out:?}"));
    assert_eq!(out, concat!(r"C:\a.ts", "\t12\t3\tfoo: bar\nsrc/b.ts\t1\t1\tx"));
}
