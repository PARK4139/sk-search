//! Runtime evidence log (rules/environment.md#증거-경로).
//!
//! Default path: `paths::APP_LOG` under the repository root found above the running exe
//! (dev builds, tests); outside a repository (deployed exe) `skim-search.log` next to the exe.
//! Override with the `SKIM_SEARCH_LOG` environment variable.

use std::fs::{self, File, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::{Mutex, OnceLock};
use std::time::{SystemTime, UNIX_EPOCH};

use crate::paths;

static SINK: OnceLock<Mutex<Option<File>>> = OnceLock::new();

pub fn log_path() -> PathBuf {
    if let Some(p) = std::env::var_os("SKIM_SEARCH_LOG") {
        return PathBuf::from(p);
    }
    // found at runtime: no build-machine path is compiled in (build_env/paths-ssot)
    let exe = std::env::current_exe().unwrap_or_default();
    let dir = exe.parent().map(Path::to_path_buf).unwrap_or_default();
    match paths::repo_root(&dir) {
        Some(root) => paths::join(&root, paths::APP_LOG),
        None => dir.join("skim-search.log"),
    }
}

fn open() -> Option<File> {
    let path = log_path();
    if let Some(dir) = path.parent() {
        fs::create_dir_all(dir).ok()?;
    }
    OpenOptions::new().create(true).append(true).open(path).ok()
}

/// Appends one line: `<unix_ms> [<event>] <detail>`.
/// Logging failures never affect the app.
pub fn write(event: &str, detail: &str) {
    let sink = SINK.get_or_init(|| Mutex::new(open()));
    let Ok(mut guard) = sink.lock() else { return };
    if let Some(file) = guard.as_mut() {
        let ms = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_millis())
            .unwrap_or(0);
        let _ = writeln!(file, "{ms} [{event}] {detail}");
        let _ = file.flush();
    }
}
