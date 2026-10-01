//! Query parser (handover §8, §9).
//!
//! Splits a raw query into the Skim content expression (passed to sk) and
//! filesystem scope filters (passed to rg as `--glob`).
//!
//! Scope tokens: `path:`, `!path:`, `ext:`, `!ext:`, `file:`, `!file:`. Other tokens, including
//! quoted ones like `'path:src`, stay in the Skim expression.
//! A scope token with an empty value (`path:` while typing) is dropped.
//!
//! `path:X` semantics (handover does not define it): directory path `X` at any
//! depth below the search root, e.g. `path:src` → `**/src/**`.
//!
//! `file:X` (handover §8, optional in v1): file name contains `X` (rg glob `*X*`, files only).
//! `!file:X` is applied to hit paths in Rust: an rg exclude glob would also skip
//! directories whose name contains `X`.

use common::log;

/// Scope keys (handover §8).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ScopeKey {
    Path,
    Ext,
    File,
}

const SCOPE_KEYS: [(&str, ScopeKey); 3] = [("path:", ScopeKey::Path), ("ext:", ScopeKey::Ext), ("file:", ScopeKey::File)];

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct Scope {
    pub include_paths: Vec<String>,
    pub exclude_paths: Vec<String>,
    pub include_exts: Vec<String>,
    pub exclude_exts: Vec<String>,
    pub include_files: Vec<String>,
    pub exclude_files: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParsedQuery {
    pub raw: String,
    /// Skim content expression for `sk --filter`. Scope tokens removed.
    pub skim: String,
    pub scope: Scope,
}

/// Skim must evaluate content expressions against the matched line only.
/// Keep ripgrep's path/line/column as tab-separated metadata and restrict
/// Skim's matcher to field four onward (`--nth 4..`).
pub const SKIM_CONTENT_FILTER_ARGS: [&str; 4] = ["--delimiter", "\t", "--nth", "4.."];

/// Converts ripgrep's `path:line:column:text` records to Skim records while
/// preserving metadata outside the fields used for matching.
pub fn skim_input_from_rg(rg_output: &str) -> String {
    let mut records = Vec::new();
    let mut malformed = 0usize;
    for line in rg_output.lines() {
        match split_rg_record(line) {
            Some((path, line_number, column, text)) => {
                records.push(format!("{path}\t{line_number}\t{column}\t{text}"))
            }
            None => malformed += 1,
        }
    }
    log::write(
        "query_syntax",
        &format!(
            "skim_input records={} malformed={} delimiter=tab match_fields=4.. shape=path\\tline\\tcolumn\\tcontent",
            records.len(), malformed
        ),
    );
    records.join("\n")
}

/// Splits `path:line:column:text` left to right. A Windows drive prefix (`C:`) is part of
/// the path, and the text may contain `:` (closed/query_syntax/skim-input-from-rg-colon-in-text).
fn split_rg_record(line: &str) -> Option<(&str, &str, &str, &str)> {
    let b = line.as_bytes();
    let skip = if b.len() > 2 && b[0].is_ascii_alphabetic() && b[1] == b':' { 2 } else { 0 };
    let p_end = skip + line[skip..].find(':')?;
    let rest = &line[p_end + 1..];
    let (line_number, rest) = rest.split_once(':')?;
    let (column, text) = rest.split_once(':')?;
    let numeric = |s: &str| !s.is_empty() && s.bytes().all(|c| c.is_ascii_digit());
    (p_end > 0 && numeric(line_number) && numeric(column)).then_some((&line[..p_end], line_number, column, text))
}

fn scope_token(token: &str) -> Option<(bool, ScopeKey, &str)> {
    let (negated, body) = match token.strip_prefix('!') {
        Some(rest) => (true, rest),
        None => (false, token),
    };
    SCOPE_KEYS.iter().find_map(|(prefix, key)| {
        body.strip_prefix(prefix)
            .map(|value| (negated, *key, value))
    })
}

pub fn parse(raw: &str) -> ParsedQuery {
    let mut scope = Scope::default();
    let mut skim = Vec::new();
    for token in raw.split_whitespace() {
        let Some((negated, key, value)) = scope_token(token) else {
            skim.push(token);
            continue;
        };
        if value.is_empty() {
            continue;
        }
        let list = match (key, negated) {
            (ScopeKey::Path, false) => &mut scope.include_paths,
            (ScopeKey::Path, true) => &mut scope.exclude_paths,
            (ScopeKey::Ext, false) => &mut scope.include_exts,
            (ScopeKey::Ext, true) => &mut scope.exclude_exts,
            (ScopeKey::File, false) => &mut scope.include_files,
            (ScopeKey::File, true) => &mut scope.exclude_files,
        };
        list.push(
            value
                .trim_matches(|c| c == '/' || c == '\\')
                .replace('\\', "/"),
        );
    }
    ParsedQuery {
        raw: raw.to_owned(),
        skim: skim.join(" "),
        scope,
    }
}

impl Scope {
    pub fn is_empty(&self) -> bool {
        self.include_paths.is_empty()
            && self.exclude_paths.is_empty()
            && self.include_exts.is_empty()
            && self.exclude_exts.is_empty()
            && self.include_files.is_empty()
            && self.exclude_files.is_empty()
    }

    /// `!file:` filter: true if the file name of `path` contains an excluded fragment.
    pub fn excludes_file(&self, path: &str) -> bool {
        let name = path.rsplit(['\\', '/']).next().unwrap_or(path);
        self.exclude_files.iter().any(|f| name.contains(f.as_str()))
    }

    /// File-name patterns from `file:` × `ext:` includes (`*auth*.ts`); empty = no name constraint.
    fn name_patterns(&self) -> Vec<String> {
        match (self.include_files.is_empty(), self.include_exts.is_empty()) {
            (true, true) => Vec::new(),
            (true, false) => self.include_exts.iter().map(|e| format!("*.{e}")).collect(),
            (false, true) => self.include_files.iter().map(|f| format!("*{f}*")).collect(),
            (false, false) => self
                .include_files
                .iter()
                .flat_map(|f| self.include_exts.iter().map(move |e| format!("*{f}*.{e}")))
                .collect(),
        }
    }

    /// rg `--glob` values.
    ///
    /// rg ORs multiple include globs, so includes are combined into one glob per
    /// (path × name) pair: `path:src ext:ts` → `**/src/**/*.ts` (AND).
    /// Exclude globs (`!…`) always apply on top (AND).
    pub fn rg_globs(&self) -> Vec<String> {
        let mut globs = Vec::new();
        let names = self.name_patterns();
        if self.include_paths.is_empty() {
            globs.extend(names.iter().cloned());
        } else {
            for p in &self.include_paths {
                if names.is_empty() {
                    globs.push(format!("**/{p}/**"));
                } else {
                    globs.extend(names.iter().map(|n| format!("**/{p}/**/{n}")));
                }
            }
        }
        globs.extend(self.exclude_paths.iter().map(|p| format!("!**/{p}/**")));
        globs.extend(self.exclude_exts.iter().map(|e| format!("!*.{e}")));
        globs
    }
}

/// Parses and writes the runtime evidence line (handover §30: raw query, parsed skim query, scope filters).
pub fn parse_logged(raw: &str) -> ParsedQuery {
    let parsed = parse(raw);
    log::write(
        "query_syntax",
        &format!(
            "raw={:?} skim={:?} scope={:?} rg_globs={:?}",
            parsed.raw,
            parsed.skim,
            parsed.scope,
            parsed.scope.rg_globs()
        ),
    );
    parsed
}
