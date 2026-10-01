//! rg.exe invocation and output parsing (handover §11).
//!
//! Output format: `--null` puts a NUL after the file path, so Windows paths with
//! drive colons or spaces never collide with the `line:column:text` separators.

use std::ffi::OsString;
use std::path::Path;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Hit {
    /// Path as printed by rg (absolute, because the search root is absolute).
    pub path: String,
    pub line: u32,
    /// 1-based byte column of the first prefilter match (rg `--column`).
    pub column: u32,
    pub text: String,
}

pub fn args(pattern: &str, globs: &[String], root: &Path) -> Vec<OsString> {
    let mut a: Vec<OsString> = [
        "--null",
        "--line-number",
        "--column",
        "--no-heading",
        "--with-filename",
        "--color",
        "never",
        "--ignore-case",
        "--max-columns",
        "500",
        "--max-columns-preview",
    ]
    .iter()
    .map(OsString::from)
    .collect();
    for g in globs {
        a.push("--glob".into());
        a.push(g.into());
    }
    a.push("--regexp".into());
    a.push(pattern.into());
    a.push("--".into());
    a.push(root.as_os_str().to_owned());
    a
}

/// Parses one rg output line: `<path>\0<line>:<column>:<text>`.
pub fn parse_line(raw: &[u8]) -> Option<Hit> {
    let raw = raw.strip_suffix(b"\n").unwrap_or(raw);
    let raw = raw.strip_suffix(b"\r").unwrap_or(raw);
    let nul = raw.iter().position(|&b| b == 0)?;
    let path = String::from_utf8_lossy(&raw[..nul]).into_owned();
    let rest = &raw[nul + 1..];
    let c1 = rest.iter().position(|&b| b == b':')?;
    let c2 = c1 + 1 + rest[c1 + 1..].iter().position(|&b| b == b':')?;
    let line = std::str::from_utf8(&rest[..c1]).ok()?.parse().ok()?;
    let column = std::str::from_utf8(&rest[c1 + 1..c2]).ok()?.parse().ok()?;
    let text = String::from_utf8_lossy(&rest[c2 + 1..]).into_owned();
    Some(Hit { path, line, column, text })
}
