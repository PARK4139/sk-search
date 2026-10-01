//! Path constants generated from `cores/common/paths.ini` (build_env/paths-ssot).
//!
//! `[repo]` values are relative to the repository root, `[third_party]` values to `CavemanDrive/3rd_party`.
//! Roots are found at runtime by walking up from a start directory (exe or crate), so no absolute
//! path is compiled into the binary.

use std::path::{Path, PathBuf};

include!(concat!(env!("OUT_DIR"), "/paths.rs"));

/// `root` joined with a `/`-separated relative SSOT value.
pub fn join(root: &Path, rel: &str) -> PathBuf {
    rel.split('/').fold(root.to_path_buf(), |p, c| p.join(c))
}

/// Repository root: the nearest ancestor of `start` that contains `PATHS_INI`.
pub fn repo_root(start: &Path) -> Option<PathBuf> {
    start.ancestors().find(|d| d.join(PATHS_INI).is_file()).map(Path::to_path_buf)
}

/// `CavemanDrive/3rd_party`: the nearest `THIRD_PARTY` directory above `start` that contains `RG`.
pub fn third_party(start: &Path) -> Option<PathBuf> {
    start.ancestors().map(|d| d.join(THIRD_PARTY)).find(|d| d.join(RG).is_file())
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Every generated constant equals the SSOT value (the Python side checks the same file).
    #[test]
    fn constants_match_paths_ini() {
        let ini = include_str!("../../../common/paths.ini");
        let mut section = "";
        let mut expected = Vec::new();
        for line in ini.lines().map(str::trim).filter(|l| !l.is_empty() && !l.starts_with('#')) {
            if let Some(s) = line.strip_prefix('[').and_then(|s| s.strip_suffix(']')) {
                section = s;
            } else if let Some((k, v)) = line.split_once('=') {
                expected.push((section, k.trim(), v.trim()));
            }
        }
        assert_eq!(ALL.to_vec(), expected);
        assert!(ALL.iter().all(|(_, _, v)| !v.contains(':') && !v.starts_with('/')));
    }

    #[test]
    fn roots_found_from_crate_dir() {
        let here = Path::new(env!("CARGO_MANIFEST_DIR"));
        let root = repo_root(here).expect("repository root");
        assert!(root.join(CARGO_WORKSPACE).join("Cargo.toml").is_file());
        assert!(third_party(here).expect("3rd_party").join(SK).is_file());
    }
}
