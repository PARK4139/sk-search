//! Runtime evidence log (rules/environment.md#증거-경로).
//!
//! Default path: `ref/actual/logs/skim-search.log` relative to the repository root.
//! Override with the `SKIM_SEARCH_LOG` environment variable.

use std::fs::{self, File, OpenOptions};
use std::io::Write;
use std::path::PathBuf;
use std::sync::{Mutex, OnceLock};
use std::time::{SystemTime, UNIX_EPOCH};

static SINK: OnceLock<Mutex<Option<File>>> = OnceLock::new();

pub fn log_path() -> PathBuf {
    if let Some(p) = std::env::var_os("SKIM_SEARCH_LOG") {
        return PathBuf::from(p);
    }
    // cores/rust/common -> repository root
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..")
        .join("..")
        .join("ref")
        .join("actual")
        .join("logs")
        .join("skim-search.log")
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
