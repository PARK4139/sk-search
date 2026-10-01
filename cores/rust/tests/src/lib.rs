//! Shared fixtures for integration tests in `tests/`.

use std::path::{Path, PathBuf};
use std::sync::mpsc;
use std::time::{Duration, Instant};

use common::paths;
use skim_search::query_syntax;
use skim_search::search_engine::engine::{Engine, Event, Request};
use skim_search::search_engine::ripgrep::Hit;

/// `CavemanDrive/3rd_party` (paths::THIRD_PARTY) found by walking up from this crate.
pub fn third_party() -> PathBuf {
    paths::third_party(Path::new(env!("CARGO_MANIFEST_DIR"))).expect("3rd_party with ripgrep/rg.exe above the repository")
}

/// A `[third_party]` SSOT value (e.g. `paths::RG`) as an absolute path.
pub fn tool(rel: &str) -> PathBuf {
    paths::join(&third_party(), rel)
}

pub fn rg() -> PathBuf {
    tool(paths::RG)
}

pub fn sk() -> PathBuf {
    tool(paths::SK)
}

/// Repository root (directory containing paths::PATHS_INI).
pub fn repo_root() -> PathBuf {
    paths::repo_root(Path::new(env!("CARGO_MANIFEST_DIR"))).expect("repository root")
}

/// Fresh temp directory unique to this process + name.
pub fn temp_dir(name: &str) -> PathBuf {
    let d = std::env::temp_dir().join(format!("skim-search-{}-{name}", std::process::id()));
    let _ = std::fs::remove_dir_all(&d);
    std::fs::create_dir_all(&d).unwrap();
    d
}

pub fn workspace(name: &str) -> PathBuf {
    let root = temp_dir(name);
    copy_sample_tree(&root);
    root
}

pub fn copy_sample_tree(destination: &Path) {
    copy_fixture(paths::join(&repo_root(), paths::SAMPLE_TREE), destination);
}

fn copy_fixture(source: PathBuf, destination: &Path) {
    std::fs::create_dir_all(destination).unwrap();
    for item in std::fs::read_dir(source).unwrap() {
        let item = item.unwrap();
        let target = destination.join(item.file_name());
        if item.file_type().unwrap().is_dir() {
            copy_fixture(item.path(), &target);
        } else {
            std::fs::copy(item.path(), target).unwrap();
        }
    }
}

/// Large workspace for streaming / cancellation tests.
pub fn big_workspace(name: &str, files: usize, lines: usize) -> PathBuf {
    let root = temp_dir(name);
    let body: String = (0..lines).map(|i| format!("line {i} login value {i}\n")).collect();
    for f in 0..files {
        let p = root.join(format!("d{}", f % 20)).join(format!("f{f}.ts"));
        std::fs::create_dir_all(p.parent().unwrap()).unwrap();
        std::fs::write(p, &body).unwrap();
    }
    root
}

pub struct Outcome {
    pub hits: Vec<Hit>,
    pub batches: usize,
    pub first_batch: Option<Duration>,
    pub done: Option<Duration>,
    pub error: Option<String>,
}

/// Runs one search to completion through the real engine (rg + sk).
pub fn search(root: &Path, raw: &str) -> Outcome {
    let engine = Engine::new();
    let generation = engine.bump();
    let (tx, rx) = mpsc::channel();
    let start = Instant::now();
    engine.spawn(
        Request {
            generation,
            query: query_syntax::parse(raw),
            root: root.to_path_buf(),
            rg: rg(),
            sk: sk(),
            changed_at: start,
        },
        move |ev| {
            let _ = tx.send((Instant::now(), ev));
        },
    );
    let mut out = Outcome { hits: Vec::new(), batches: 0, first_batch: None, done: None, error: None };
    while let Ok((at, ev)) = rx.recv_timeout(Duration::from_secs(30)) {
        match ev {
            Event::Batch { hits, .. } => {
                out.batches += 1;
                out.first_batch.get_or_insert(at - start);
                out.hits.extend(hits);
            }
            Event::Done { .. } => {
                out.done = Some(at - start);
                break;
            }
            Event::Error { message, .. } => {
                out.error = Some(message);
                break;
            }
        }
    }
    out
}

/// `rel/path:line` keys of hits, relative to `root`, `/` separated.
pub fn keys(root: &Path, hits: &[Hit]) -> Vec<String> {
    let root = root.to_string_lossy().to_string();
    let mut v: Vec<String> = hits
        .iter()
        .map(|h| format!("{}:{}", skim_search::result_panel::relative(&root, &h.path), h.line))
        .collect();
    v.sort();
    v
}

pub fn files(root: &Path, hits: &[Hit]) -> Vec<String> {
    let mut v: Vec<String> = keys(root, hits).into_iter().map(|k| k.rsplit_once(':').unwrap().0.to_string()).collect();
    v.dedup();
    v
}
