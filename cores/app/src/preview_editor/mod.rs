//! Preview document: load, edit, save with external-change detection
//! (handover §17, §18, §19).

use std::collections::hash_map::DefaultHasher;
use std::fs;
use std::hash::{Hash, Hasher};
use std::path::{Path, PathBuf};
use std::time::SystemTime;

use common::log;

/// File identity at load time. Save is refused when the file on disk differs.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Stamp {
    pub len: u64,
    pub modified: Option<SystemTime>,
    pub hash: u64,
}

#[derive(Debug, Clone)]
pub struct Doc {
    pub path: PathBuf,
    pub crlf: bool,
    pub stamp: Stamp,
    /// Text shown in the editor (`\n` line endings).
    pub text: String,
}

#[derive(Debug, PartialEq, Eq)]
pub enum SaveOutcome {
    Saved,
    Unchanged,
    Conflict,
}

fn hash(bytes: &[u8]) -> u64 {
    let mut h = DefaultHasher::new();
    bytes.hash(&mut h);
    h.finish()
}

pub fn stamp_of(path: &Path) -> std::io::Result<Stamp> {
    let bytes = fs::read(path)?;
    let meta = fs::metadata(path)?;
    Ok(Stamp { len: meta.len(), modified: meta.modified().ok(), hash: hash(&bytes) })
}

pub fn load(path: &Path) -> Result<Doc, String> {
    let bytes = fs::read(path).map_err(|e| format!("읽기 실패: {e}"))?;
    let meta = fs::metadata(path).map_err(|e| format!("읽기 실패: {e}"))?;
    let text = String::from_utf8(bytes.clone()).map_err(|_| "UTF-8 파일이 아님 (편집 불가)".to_string())?;
    let crlf = text.contains("\r\n");
    let doc = Doc {
        path: path.to_path_buf(),
        crlf,
        stamp: Stamp { len: meta.len(), modified: meta.modified().ok(), hash: hash(&bytes) },
        text: if crlf { text.replace("\r\n", "\n") } else { text },
    };
    log::write(
        "preview_editor",
        &format!("load path={} bytes={} crlf={crlf} lines={}", path.display(), doc.stamp.len, line_count(&doc.text)),
    );
    Ok(doc)
}

/// Saves `edited` unless the file changed on disk since load (size, mtime or content hash).
pub fn save(doc: &mut Doc, edited: &str) -> Result<SaveOutcome, String> {
    let now = stamp_of(&doc.path).map_err(|e| format!("확인 실패: {e}"))?;
    if now != doc.stamp {
        log::write(
            "preview_editor",
            &format!("save conflict path={} loaded={:?} disk={:?}", doc.path.display(), doc.stamp, now),
        );
        return Ok(SaveOutcome::Conflict);
    }
    if edited == doc.text {
        log::write("preview_editor", &format!("save unchanged path={}", doc.path.display()));
        return Ok(SaveOutcome::Unchanged);
    }
    let out = if doc.crlf { edited.replace('\n', "\r\n") } else { edited.to_owned() };
    fs::write(&doc.path, out.as_bytes()).map_err(|e| format!("저장 실패: {e}"))?;
    doc.stamp = stamp_of(&doc.path).map_err(|e| format!("확인 실패: {e}"))?;
    doc.text = edited.to_owned();
    log::write(
        "preview_editor",
        &format!("saved path={} bytes={} crlf={} stamp={:?}", doc.path.display(), out.len(), doc.crlf, doc.stamp),
    );
    Ok(SaveOutcome::Saved)
}

pub fn line_count(text: &str) -> usize {
    text.split('\n').count()
}

/// Gutter text: `1\n2\n…\nN`.
pub fn gutter(text: &str) -> String {
    (1..=line_count(text)).map(|n| n.to_string()).collect::<Vec<_>>().join("\n")
}

/// Byte offset of (1-based line, 1-based byte column) in `text`, clamped to the line.
pub fn byte_offset(text: &str, line: u32, column: u32) -> usize {
    let mut start = 0usize;
    for (i, l) in text.split('\n').enumerate() {
        if i + 1 == line as usize {
            let col = (column.max(1) as usize - 1).min(l.len());
            let mut off = start + col;
            while !text.is_char_boundary(off) {
                off -= 1;
            }
            return off;
        }
        start += l.len() + 1;
    }
    text.len()
}

/// Byte range of the given 1-based line (without the newline).
pub fn line_range(text: &str, line: u32) -> (usize, usize) {
    let start = byte_offset(text, line, 1);
    let end = text[start..].find('\n').map(|i| start + i).unwrap_or(text.len());
    (start, end)
}
