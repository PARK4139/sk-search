//! User settings (handover §12 configured path, §20 editor, §5 recent root).
//!
//! Path: `%APPDATA%\skim-search\settings.json`, override with `SKIM_SEARCH_SETTINGS`.
//! A missing or unreadable file yields defaults; the reason is logged.

use std::fs;
use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use crate::log;

pub const MAX_RECENT_ROOTS: usize = 10;

#[derive(Debug, Default, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct Settings {
    pub rg_path: Option<PathBuf>,
    pub sk_path: Option<PathBuf>,
    /// File open target: `"vscode"`, `"cursor"` or `"system"` (default, handover §20).
    pub editor: Option<String>,
    /// Most recent first; the first entry is restored on start (handover §5).
    pub recent_roots: Vec<String>,
}

impl Settings {
    pub fn editor(&self) -> &str {
        self.editor.as_deref().unwrap_or("system")
    }

    pub fn last_root(&self) -> Option<&str> {
        self.recent_roots.first().map(String::as_str)
    }

    /// Moves `root` to the front of the recent list (case-insensitive dedupe).
    pub fn remember_root(&mut self, root: &str) {
        self.recent_roots.retain(|r| !r.eq_ignore_ascii_case(root));
        self.recent_roots.insert(0, root.to_owned());
        self.recent_roots.truncate(MAX_RECENT_ROOTS);
    }
}

pub fn settings_path() -> PathBuf {
    if let Some(p) = std::env::var_os("SKIM_SEARCH_SETTINGS") {
        return PathBuf::from(p);
    }
    let base = std::env::var_os("APPDATA").map(PathBuf::from).unwrap_or_default();
    base.join("skim-search").join("settings.json")
}

pub fn load() -> Settings {
    let path = settings_path();
    let text = match fs::read_to_string(&path) {
        Ok(t) => t,
        Err(e) => {
            log::write("settings", &format!("load default path={} reason={e}", path.display()));
            return Settings::default();
        }
    };
    match serde_json::from_str::<Settings>(&text) {
        Ok(s) => {
            log::write("settings", &format!("loaded path={} {s:?}", path.display()));
            s
        }
        Err(e) => {
            log::write("settings", &format!("load default path={} parse_error={e}", path.display()));
            Settings::default()
        }
    }
}

pub fn save(settings: &Settings) {
    let path = settings_path();
    let result = path
        .parent()
        .map_or(Ok(()), fs::create_dir_all)
        .and_then(|_| fs::write(&path, serde_json::to_string_pretty(settings).unwrap_or_default()));
    match result {
        Ok(()) => log::write("settings", &format!("saved path={} {settings:?}", path.display())),
        Err(e) => log::write("settings", &format!("save failed path={} error={e}", path.display())),
    }
}
