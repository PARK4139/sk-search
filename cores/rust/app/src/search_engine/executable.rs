//! rg.exe / sk.exe detection (handover §12, FR-132).
//! Order: configured path → PATH → fallback.

use std::ffi::OsStr;
use std::os::windows::process::CommandExt;
use std::path::{Path, PathBuf};
use std::process::Command;

use common::log;

/// Prevents a console window flash when spawning from the GUI process.
pub const CREATE_NO_WINDOW: u32 = 0x0800_0000;

#[derive(Debug, Clone, Copy)]
pub struct ToolSpec {
    pub name: &'static str,
    pub file: &'static str,
    /// Directory under `3rd_party/` used by the fallback search.
    pub third_party_dir: &'static str,
}

pub const RG: ToolSpec = ToolSpec {
    name: "rg",
    file: "rg.exe",
    third_party_dir: "ripgrep",
};
pub const SK: ToolSpec = ToolSpec {
    name: "sk",
    file: "sk.exe",
    third_party_dir: "skim",
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Source {
    Configured,
    Path,
    Fallback,
}

#[derive(Debug, Clone)]
pub struct Tool {
    pub name: &'static str,
    pub path: PathBuf,
    pub source: Source,
    pub version: String,
}

#[derive(Debug, Clone)]
pub struct Missing {
    pub name: &'static str,
    pub reason: String,
}

/// Fallback locations: for the exe dir and each ancestor,
/// `<dir>\<file>` and `<dir>\3rd_party\<third_party_dir>\<file>`.
pub fn fallback_candidates(spec: &ToolSpec, exe: &Path) -> Vec<PathBuf> {
    let mut out = Vec::new();
    for dir in exe.parent().into_iter().flat_map(Path::ancestors) {
        out.push(dir.join(spec.file));
        out.push(
            dir.join("3rd_party")
                .join(spec.third_party_dir)
                .join(spec.file),
        );
    }
    out
}

pub fn locate(
    spec: &ToolSpec,
    configured: Option<&Path>,
    path_env: Option<&OsStr>,
    fallbacks: &[PathBuf],
) -> Result<(PathBuf, Source), String> {
    if let Some(p) = configured {
        if p.is_file() {
            return Ok((p.to_path_buf(), Source::Configured));
        }
        log::write(
            "search_engine",
            &format!("{} configured path not found: {}", spec.name, p.display()),
        );
    }
    if let Some(env) = path_env {
        if let Some(p) = std::env::split_paths(env)
            .map(|d| d.join(spec.file))
            .find(|p| p.is_file())
        {
            return Ok((p, Source::Path));
        }
    }
    if let Some(p) = fallbacks.iter().find(|p| p.is_file()) {
        return Ok((p.clone(), Source::Fallback));
    }
    Err(format!(
        "{} not found (configured={}, PATH, fallback {} candidates)",
        spec.file,
        configured
            .map(|p| p.display().to_string())
            .unwrap_or_else(|| "-".into()),
        fallbacks.len()
    ))
}

/// Runs `<tool> --version` and returns the version token (`ripgrep 15.2.0` → `15.2.0`).
pub fn version(path: &Path) -> Result<String, String> {
    let out = Command::new(path)
        .arg("--version")
        .creation_flags(CREATE_NO_WINDOW)
        .output()
        .map_err(|e| format!("spawn failed: {e}"))?;
    if !out.status.success() {
        return Err(format!("--version exit {}", out.status));
    }
    let text = String::from_utf8_lossy(&out.stdout);
    text.lines()
        .next()
        .and_then(|l| l.split_whitespace().nth(1))
        .map(str::to_owned)
        .ok_or_else(|| format!("unexpected --version output: {text:?}"))
}

pub fn detect(
    spec: &ToolSpec,
    configured: Option<&Path>,
    path_env: Option<&OsStr>,
    fallbacks: &[PathBuf],
) -> Result<Tool, Missing> {
    let result = locate(spec, configured, path_env, fallbacks).and_then(|(path, source)| {
        version(&path)
            .map(|v| (path, source, v))
            .map_err(|e| e.to_string())
    });
    match result {
        Ok((path, source, version)) => {
            log::write(
                "search_engine",
                &format!(
                    "{} detected source={source:?} version={version} path={}",
                    spec.name,
                    path.display()
                ),
            );
            Ok(Tool {
                name: spec.name,
                path,
                source,
                version,
            })
        }
        Err(reason) => {
            log::write(
                "search_engine",
                &format!("{} missing reason={reason}", spec.name),
            );
            Err(Missing {
                name: spec.name,
                reason,
            })
        }
    }
}
