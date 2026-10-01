//! Search root validation and folder picker (handover §5, FR-103, FR-104).

use std::path::Path;

use common::log;
use windows::Win32::System::Com::{
    CoCreateInstance, CoInitializeEx, CoTaskMemFree, CLSCTX_INPROC_SERVER, COINIT_APARTMENTTHREADED,
};
use windows::Win32::UI::Shell::{FileOpenDialog, IFileOpenDialog, FOS_FORCEFILESYSTEM, FOS_PICKFOLDERS, SIGDN_FILESYSPATH};

use crate::shortcut;

/// `Ok(())` if `root` is an existing directory; otherwise the reason shown to the user.
pub fn validate_root(root: &str) -> Result<(), String> {
    let root = root.trim();
    if root.is_empty() {
        return Err("검색 경로를 입력하세요".into());
    }
    let p = Path::new(root);
    if !p.is_absolute() {
        return Err(format!("절대 경로가 아님: {root}"));
    }
    match std::fs::metadata(p) {
        Ok(m) if m.is_dir() => Ok(()),
        Ok(_) => Err(format!("폴더가 아닌 파일 경로: {root}")),
        Err(_) => Err(format!("존재하지 않는 경로: {root}")),
    }
}

/// Modal Windows folder picker. `None` when cancelled or on error.
pub fn pick_folder(owner_title: &str) -> Option<String> {
    // SAFETY: COM calls on the UI thread; every returned interface is used within its lifetime
    // and the path buffer is freed with CoTaskMemFree.
    let result = unsafe {
        let _ = CoInitializeEx(None, COINIT_APARTMENTTHREADED);
        (|| -> windows::core::Result<String> {
            let dialog: IFileOpenDialog = CoCreateInstance(&FileOpenDialog, None, CLSCTX_INPROC_SERVER)?;
            dialog.SetOptions(dialog.GetOptions()? | FOS_PICKFOLDERS | FOS_FORCEFILESYSTEM)?;
            dialog.Show(shortcut::find_window(owner_title))?;
            let item = dialog.GetResult()?;
            let pw = item.GetDisplayName(SIGDN_FILESYSPATH)?;
            let s = pw.to_string().unwrap_or_default();
            CoTaskMemFree(Some(pw.0 as _));
            Ok(s)
        })()
    };
    log::write("view", &format!("folder picker result={result:?}"));
    result.ok()
}
