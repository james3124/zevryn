//! Extract display information from an executable file name.
//!
//! `ImportGameViewModel.m(exePath)` (`ImportGameViewModel.java:49-57`) forwards
//! the exe path to `GameLibraryRepository.k(exePath)` via
//! `fetchGameInfoByExeFileName`; that repository call is the only step in the
//! reference that depends on a name derived from an exe. The model here keeps
//! the portable part: the file name and the game name (the stem, i.e. the file
//! name without `.exe`).

use super::file_detector::{basename, is_exe_file, EXE_SUFFIX};

/// Name information derived from an executable path.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExeInfo {
    /// Original path as supplied to the pipeline.
    pub path: String,
    /// File name component (with extension).
    pub file_name: String,
    /// Game name: the file name with the trailing `.exe` removed.
    pub game_name: String,
}

impl ExeInfo {
    /// Build exe info from a path. Returns `None` if the path's file name is
    /// not an `.exe` (case-insensitive).
    pub fn from_path(path: &str) -> Option<Self> {
        let file_name = basename(path);
        if !is_exe_file(file_name) {
            return None;
        }
        let game_name = file_name[..file_name.len() - EXE_SUFFIX.len()].to_owned();
        Some(Self {
            path: path.to_owned(),
            file_name: file_name.to_owned(),
            game_name,
        })
    }

    /// The name passed to `fetchGameInfoByExeFileName` for lookup.
    pub fn lookup_name(&self) -> &str {
        &self.game_name
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn game_name_is_stem_without_exe() {
        let info = ExeInfo::from_path(r"C:\Games\Dota 2\dota2.exe").unwrap();
        assert_eq!(info.file_name, "dota2.exe");
        assert_eq!(info.game_name, "dota2");
        assert_eq!(info.lookup_name(), "dota2");
    }

    #[test]
    fn uppercase_extension_is_stripped() {
        let info = ExeInfo::from_path("/g/GAME.EXE").unwrap();
        assert_eq!(info.game_name, "GAME");
    }

    #[test]
    fn non_exe_paths_are_rejected() {
        assert!(ExeInfo::from_path("/g/game.zip").is_none());
        assert!(ExeInfo::from_path("/g/").is_none());
    }
}
