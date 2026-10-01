//! Query parser (handover §8, §9).
//!
//! Splits a raw query into the Skim content expression (passed to sk) and
//! filesystem scope filters (passed to rg as `--glob`).
//!
//! Scope tokens: `path:`, `!path:`, `ext:`, `!ext:`. Other tokens, including
//! quoted ones like `'path:src`, stay in the Skim expression.
//! A scope token with an empty value (`path:` while typing) is dropped.
//!
//! `path:X` semantics (handover does not define it): directory path `X` at any
//! depth below the search root, e.g. `path:src` → `**/src/**`.

use common::log;

/// Scope keys. Adding `file:` later (handover §8) means adding a variant here.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ScopeKey {
    Path,
    Ext,
}

const SCOPE_KEYS: [(&str, ScopeKey); 2] = [("path:", ScopeKey::Path), ("ext:", ScopeKey::Ext)];

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct Scope {
    pub include_paths: Vec<String>,
    pub exclude_paths: Vec<String>,
    pub include_exts: Vec<String>,
    pub exclude_exts: Vec<String>,
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
        let mut fields = line.rsplitn(4, ':');
        let Some(text) = fields.next() else {
            malformed += 1;
            continue;
        };
        let Some(column) = fields.next().filter(|v| v.parse::<u64>().is_ok()) else {
            malformed += 1;
            continue;
        };
        let Some(line_number) = fields.next().filter(|v| v.parse::<u64>().is_ok()) else {
            malformed += 1;
            continue;
        };
        let Some(path) = fields.next().filter(|v| !v.is_empty()) else {
            malformed += 1;
            continue;
        };
        records.push(format!("{path}\t{line_number}\t{column}\t{text}"));
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
    }

    /// rg `--glob` values.
    ///
    /// rg ORs multiple include globs, so includes are combined into one glob per
    /// (path × ext) pair: `path:src ext:ts` → `**/src/**/*.ts` (AND).
    /// Exclude globs (`!…`) always apply on top (AND).
    pub fn rg_globs(&self) -> Vec<String> {
        let mut globs = Vec::new();
        match (self.include_paths.is_empty(), self.include_exts.is_empty()) {
            (true, true) => {}
            (false, true) => globs.extend(self.include_paths.iter().map(|p| format!("**/{p}/**"))),
            (true, false) => globs.extend(self.include_exts.iter().map(|e| format!("*.{e}"))),
            (false, false) => {
                for p in &self.include_paths {
                    globs.extend(self.include_exts.iter().map(|e| format!("**/{p}/**/*.{e}")));
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
