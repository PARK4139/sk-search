//! File open / parent folder open (handover §20, §21).
//!
//! - VSCode: `code --goto "file:line:column"`, Cursor: `cursor --goto …`
//! - System Default: Windows Shell open (`ShellExecuteW "open"`)
//! - Parent: `explorer.exe /select,"<path>"`

use std::ffi::OsStr;
use std::os::windows::ffi::OsStrExt;
use std::os::windows::process::CommandExt;
use std::path::{Path, PathBuf};
use std::process::Command;

use common::log;
use windows::core::PCWSTR;
use windows::Win32::UI::Shell::ShellExecuteW;
use windows::Win32::UI::WindowsAndMessaging::SW_SHOWNORMAL;

use crate::search_engine::executable::CREATE_NO_WINDOW;

/// Finds `<name>.cmd` / `<name>.exe` on PATH (VS Code and Cursor install `.cmd` launchers).
fn find_on_path(name: &str) -> Option<PathBuf> {
    let path = std::env::var_os("PATH")?;
    std::env::split_paths(&path).find_map(|dir| {
        ["cmd", "exe"].iter().map(|ext| dir.join(format!("{name}.{ext}"))).find(|p| p.is_file())
    })
}

fn wide(s: &OsStr) -> Vec<u16> {
    s.encode_wide().chain(std::iter::once(0)).collect()
}

/// Opens `path` at `line:column` with the configured editor. Returns a description for logs/toasts.
pub fn open_file(editor: &str, path: &Path, line: u32, column: u32) -> Result<String, String> {
    let result = match editor {
        "vscode" | "cursor" => {
            let name = if editor == "vscode" { "code" } else { "cursor" };
            let exe = find_on_path(name).ok_or_else(|| format!("{name} 실행 파일을 PATH에서 찾지 못함"))?;
            let target = format!("{}:{line}:{column}", path.display());
            Command::new(&exe)
                .arg("--goto")
                .arg(&target)
                .creation_flags(CREATE_NO_WINDOW)
                .spawn()
                .map(|_| format!("{} --goto {target}", exe.display()))
                .map_err(|e| format!("{name} 실행 실패: {e}"))
        }
        _ => {
            let file = wide(path.as_os_str());
            let verb = wide(OsStr::new("open"));
            // SAFETY: both strings are NUL-terminated UTF-16 buffers that outlive the call.
            let h = unsafe {
                ShellExecuteW(None, PCWSTR(verb.as_ptr()), PCWSTR(file.as_ptr()), PCWSTR::null(), PCWSTR::null(), SW_SHOWNORMAL)
            };
            // ShellExecuteW returns a value > 32 on success.
            if h.0 as isize > 32 {
                Ok(format!("ShellExecuteW open {}", path.display()))
            } else {
                Err(format!("기본 프로그램 실행 실패 (code {})", h.0 as isize))
            }
        }
    };
    log::write("external_open", &format!("open_file editor={editor} line={line} column={column} result={result:?}"));
    result
}

/// Opens Explorer with `path` selected.
/// `SKIM_SEARCH_EXPLORER` overrides the executable (test seam).
pub fn open_parent(path: &Path) -> Result<String, String> {
    let arg = format!("/select,\"{}\"", path.display());
    let exe = std::env::var_os("SKIM_SEARCH_EXPLORER").unwrap_or_else(|| "explorer.exe".into());
    let result = Command::new(&exe)
        .raw_arg(&arg)
        .spawn()
        .map(|_| format!("explorer.exe {arg}"))
        .map_err(|e| format!("explorer 실행 실패: {e}"));
    log::write("external_open", &format!("open_parent result={result:?}"));
    result
}
