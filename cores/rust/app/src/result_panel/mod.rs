//! Result grouping by file and the flattened row list shown in the left panel
//! (handover §14, §15).

use std::collections::HashMap;
use std::path::Path;

use crate::search_engine::ripgrep::Hit;

#[derive(Debug, Clone)]
pub struct Line {
    pub line: u32,
    pub column: u32,
    pub text: String,
}

#[derive(Debug, Clone)]
pub struct Group {
    /// Absolute path.
    pub path: String,
    /// Path relative to the search root, `/` separated, for display.
    pub rel: String,
    pub badge: String,
    pub lines: Vec<Line>,
    pub expanded: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RowKind {
    Group,
    Line,
}

#[derive(Debug, Clone)]
pub struct Row {
    pub kind: RowKind,
    pub group: usize,
    /// Line index inside the group; `None` for group headers.
    pub index: Option<usize>,
}

#[derive(Debug, Default)]
pub struct Results {
    pub root: String,
    pub groups: Vec<Group>,
    by_path: HashMap<String, usize>,
    pub total: usize,
    /// (group, line index)
    pub selected: Option<(usize, usize)>,
}

/// Short file-type badge from the extension: `ts` → `TS`, none → `TXT`.
pub fn badge(path: &str) -> String {
    Path::new(path)
        .extension()
        .and_then(|e| e.to_str())
        .map(|e| e.chars().take(4).collect::<String>().to_uppercase())
        .unwrap_or_else(|| "TXT".into())
}

pub fn relative(root: &str, path: &str) -> String {
    let root = root.trim_end_matches(['\\', '/']);
    let rel = path
        .get(..root.len())
        .filter(|p| p.eq_ignore_ascii_case(root))
        .map(|_| &path[root.len()..])
        .unwrap_or(path);
    rel.trim_start_matches(['\\', '/']).replace('\\', "/")
}

impl Results {
    pub fn new(root: &str) -> Self {
        Self { root: root.to_owned(), ..Default::default() }
    }

    /// Adds hits; the first group starts expanded, later groups collapsed (handover §14).
    /// The first hit becomes the selection when nothing is selected.
    pub fn add(&mut self, hits: Vec<Hit>) {
        for hit in hits {
            let gi = match self.by_path.get(&hit.path) {
                Some(&gi) => gi,
                None => {
                    let gi = self.groups.len();
                    self.groups.push(Group {
                        rel: relative(&self.root, &hit.path),
                        badge: badge(&hit.path),
                        path: hit.path.clone(),
                        lines: Vec::new(),
                        expanded: gi == 0,
                    });
                    self.by_path.insert(hit.path.clone(), gi);
                    gi
                }
            };
            self.groups[gi].lines.push(Line { line: hit.line, column: hit.column, text: hit.text });
            self.total += 1;
        }
        if self.selected.is_none() && !self.groups.is_empty() {
            self.selected = Some((0, 0));
        }
    }

    pub fn rows(&self) -> Vec<Row> {
        let mut rows = Vec::new();
        for (gi, g) in self.groups.iter().enumerate() {
            rows.push(Row { kind: RowKind::Group, group: gi, index: None });
            if g.expanded {
                rows.extend((0..g.lines.len()).map(|li| Row { kind: RowKind::Line, group: gi, index: Some(li) }));
            }
        }
        rows
    }

    pub fn toggle(&mut self, gi: usize) {
        if let Some(g) = self.groups.get_mut(gi) {
            g.expanded = !g.expanded;
        }
    }

    /// Selects a match line; its group is expanded so the selection is visible.
    pub fn select(&mut self, gi: usize, li: usize) -> bool {
        match self.groups.get_mut(gi) {
            Some(g) if li < g.lines.len() => {
                g.expanded = true;
                self.selected = Some((gi, li));
                true
            }
            _ => false,
        }
    }

    /// Moves the selection over all match lines across groups (↑/↓, handover §22).
    pub fn step(&mut self, delta: i32) -> bool {
        let flat: Vec<(usize, usize)> = self
            .groups
            .iter()
            .enumerate()
            .flat_map(|(gi, g)| (0..g.lines.len()).map(move |li| (gi, li)))
            .collect();
        if flat.is_empty() {
            return false;
        }
        let pos = self.selected.and_then(|s| flat.iter().position(|&p| p == s));
        let next = match pos {
            None => 0,
            Some(p) => (p as i64 + delta as i64).clamp(0, flat.len() as i64 - 1) as usize,
        };
        let (gi, li) = flat[next];
        self.select(gi, li)
    }

    pub fn selected_line(&self) -> Option<(&Group, &Line)> {
        let (gi, li) = self.selected?;
        let g = self.groups.get(gi)?;
        Some((g, g.lines.get(li)?))
    }
}
