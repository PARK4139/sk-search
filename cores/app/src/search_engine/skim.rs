//! sk.exe filtering (handover §12) and the rg prefilter derived from the Skim expression.
//!
//! sk receives `<id>\t<content>` records and matches only the content field
//! (`--delimiter \t --nth 2..`), so path/line/column metadata can never affect
//! Skim conditions such as `!login` (handover §10, option 3: id mapping in Rust).

use std::io::{BufRead, BufReader, Write};
use std::os::windows::process::CommandExt;
use std::path::Path;
use std::process::{Child, ChildStdin, ChildStdout, Command, Stdio};

use super::executable::CREATE_NO_WINDOW;

pub const CONTENT_ONLY_ARGS: [&str; 5] = ["--no-sort", "--delimiter", "\t", "--nth", "2.."];

/// Spawns `sk --filter <expr>` with content-only matching.
pub fn spawn_filter(sk: &Path, expr: &str) -> std::io::Result<Child> {
    Command::new(sk)
        .arg("--filter")
        .arg(expr)
        .args(CONTENT_ONLY_ARGS)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .creation_flags(CREATE_NO_WINDOW)
        .spawn()
}

/// Feeds `(id, content)` records to a spawned sk and returns matching ids in input order.
/// Returns early (partial ids) if the process is killed.
pub fn exchange(stdin: ChildStdin, stdout: ChildStdout, records: &[(usize, &str)]) -> std::io::Result<Vec<usize>> {
    let mut stdin = stdin;
    let mut payload = String::new();
    for (id, text) in records {
        payload.push_str(&id.to_string());
        payload.push('\t');
        payload.push_str(text);
        payload.push('\n');
    }
    let writer = std::thread::spawn(move || {
        let _ = stdin.write_all(payload.as_bytes());
    });
    let mut ids = Vec::new();
    for line in BufReader::new(stdout).lines() {
        let Ok(line) = line else { break };
        if let Some(id) = line.split('\t').next().and_then(|s| s.parse().ok()) {
            ids.push(id);
        }
    }
    let _ = writer.join();
    Ok(ids)
}

/// Convenience for tests and one-off use: spawn + exchange + wait.
pub fn filter(sk: &Path, expr: &str, records: &[(usize, &str)]) -> std::io::Result<Vec<usize>> {
    let mut child = spawn_filter(sk, expr)?;
    let ids = exchange(child.stdin.take().expect("sk stdin"), child.stdout.take().expect("sk stdout"), records);
    let _ = child.wait();
    ids
}

fn regex_escape(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for c in s.chars() {
        if "\\.+*?()|[]{}^$-".contains(c) {
            out.push('\\');
        }
        out.push(c);
    }
    out
}

/// rg regex for one positive Skim term, matching a superset of what sk accepts.
fn term_regex(term: &str) -> Option<String> {
    if term.starts_with('!') || term.is_empty() {
        return None;
    }
    if let Some(exact) = term.strip_prefix('\'') {
        let exact = exact.strip_suffix('\'').unwrap_or(exact);
        return (!exact.is_empty()).then(|| regex_escape(exact));
    }
    let (prefix, body) = match term.strip_prefix('^') {
        Some(b) => ("^", b),
        None => ("", term),
    };
    let (body, suffix) = match body.strip_suffix('$') {
        Some(b) => (b, "$"),
        None => (body, ""),
    };
    if body.is_empty() {
        return None;
    }
    if !prefix.is_empty() || !suffix.is_empty() {
        return Some(format!("{prefix}{}{suffix}", regex_escape(body)));
    }
    // fuzzy: characters in order
    Some(body.chars().map(|c| regex_escape(&c.to_string())).collect::<Vec<_>>().join(".*"))
}

/// Picks the first AND-group whose OR-alternatives are all positive and builds an
/// rg regex for it. Returns `""` (match every line) when no such group exists,
/// e.g. `!login`.
pub fn prefilter_regex(expr: &str) -> String {
    let mut groups: Vec<Vec<&str>> = Vec::new();
    let mut join_next = false;
    for token in expr.split_whitespace() {
        if token == "|" {
            join_next = true;
            continue;
        }
        match (join_next, groups.last_mut()) {
            (true, Some(g)) => g.push(token),
            _ => groups.push(vec![token]),
        }
        join_next = false;
    }
    for group in groups {
        let parts: Option<Vec<String>> = group.iter().map(|t| term_regex(t)).collect();
        if let Some(parts) = parts {
            return if parts.len() == 1 {
                parts.into_iter().next().unwrap()
            } else {
                parts.iter().map(|p| format!("(?:{p})")).collect::<Vec<_>>().join("|")
            };
        }
    }
    String::new()
}
