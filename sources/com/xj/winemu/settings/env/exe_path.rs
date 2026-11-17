//! Local PC executable-path model.
//!
//! `GameSettingViewModel.getLocalPcExePath` reads the stored path from the
//! game library repository; `updateLocalPcExePath` writes a new one and only
//! publishes it to the LiveData when the repository reports success
//! (otherwise it logs "更新exepath失败"). This module keeps the portable
//! validation of that path — absolute path, `.exe` extension — and a pure
//! update outcome model. Repository calls themselves are out of scope.

use std::path::Path;

/// Why a candidate executable path was rejected.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExePathError {
    /// Empty or whitespace-only path.
    Empty,
    /// Not an absolute path (no root / drive prefix).
    NotAbsolute,
    /// Does not end in `.exe` (case-insensitive).
    WrongExtension,
    /// Passed local validation but the repository update failed.
    RepositoryRejected,
}

/// A validated local PC executable path.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LocalExePath {
    pub path: String,
}

impl LocalExePath {
    pub fn new(path: impl Into<String>) -> Self {
        Self { path: path.into() }
    }

    /// Accept Unix-style roots (`/...`), Windows roots (`\\...`) and drive
    /// prefixes (`C:\...`, `D:/...`).
    pub fn is_absolute(path: &str) -> bool {
        if path.starts_with('/') || path.starts_with('\\') {
            return true;
        }
        let bytes = path.as_bytes();
        bytes.len() >= 3
            && bytes[0].is_ascii_alphabetic()
            && bytes[1] == b':'
            && (bytes[2] == b'\\' || bytes[2] == b'/')
    }

    /// Case-insensitive `.exe` extension check.
    pub fn has_exe_extension(path: &str) -> bool {
        Path::new(path)
            .extension()
            .and_then(|ext| ext.to_str())
            .map(|ext| ext.eq_ignore_ascii_case("exe"))
            .unwrap_or(false)
    }

    pub fn validate(&self) -> Result<(), ExePathError> {
        let path = self.path.trim();
        if path.is_empty() {
            return Err(ExePathError::Empty);
        }
        if !Self::is_absolute(path) {
            return Err(ExePathError::NotAbsolute);
        }
        if !Self::has_exe_extension(path) {
            return Err(ExePathError::WrongExtension);
        }
        Ok(())
    }

    pub fn is_valid(&self) -> bool {
        self.validate().is_ok()
    }
}

/// Pure outcome of `updateLocalPcExePath`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ExePathUpdate {
    /// Validation and the (simulated) repository write succeeded.
    Applied(LocalExePath),
    /// The path was rejected locally, or the repository write failed.
    Rejected {
        attempted: LocalExePath,
        error: ExePathError,
        repository_ok: bool,
    },
}

impl ExePathUpdate {
    pub fn is_applied(&self) -> bool {
        matches!(self, ExePathUpdate::Applied(_))
    }

    pub fn applied_path(&self) -> Option<&LocalExePath> {
        match self {
            ExePathUpdate::Applied(path) => Some(path),
            ExePathUpdate::Rejected { .. } => None,
        }
    }
}

/// Simulate an update: `repository_ok` stands in for the boolean returned by
/// the game-library repository. An unchanged, already-valid path is a no-op
/// success. No I/O is performed.
pub fn update(current: &LocalExePath, candidate: &str, repository_ok: bool) -> ExePathUpdate {
    let attempted = LocalExePath::new(candidate);
    match attempted.validate() {
        Err(error) => ExePathUpdate::Rejected {
            attempted,
            error,
            repository_ok,
        },
        Ok(()) => {
            if !repository_ok {
                return ExePathUpdate::Rejected {
                    attempted,
                    error: ExePathError::RepositoryRejected,
                    repository_ok: false,
                };
            }
            if attempted == *current {
                ExePathUpdate::Applied(current.clone())
            } else {
                ExePathUpdate::Applied(attempted)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn validates_unix_absolute_exe_paths() {
        assert!(LocalExePath::new("/home/games/Game/game.exe").is_valid());
        assert!(LocalExePath::new("/opt/Game.Launch.GAME.EXE").is_valid());
    }

    #[test]
    fn validates_windows_absolute_exe_paths() {
        assert!(LocalExePath::new(r"C:\Games\game.exe").is_valid());
        assert!(LocalExePath::new("D:/Games/game.exe").is_valid());
        assert!(LocalExePath::new(r"\\server\share\game.exe").is_valid());
    }

    #[test]
    fn rejects_empty_relative_and_wrong_extension() {
        assert_eq!(LocalExePath::new("").validate(), Err(ExePathError::Empty));
        assert_eq!(
            LocalExePath::new("   ").validate(),
            Err(ExePathError::Empty)
        );
        assert_eq!(
            LocalExePath::new("games/game.exe").validate(),
            Err(ExePathError::NotAbsolute)
        );
        assert_eq!(
            LocalExePath::new("game.exe").validate(),
            Err(ExePathError::NotAbsolute)
        );
        assert_eq!(
            LocalExePath::new("/home/games/game.zip").validate(),
            Err(ExePathError::WrongExtension)
        );
    }

    #[test]
    fn update_applies_only_valid_repo_confirmed_paths() {
        let current = LocalExePath::new("/old/game.exe");

        let applied = update(&current, "/new/game.exe", true);
        assert!(applied.is_applied());
        assert_eq!(
            applied.applied_path(),
            Some(&LocalExePath::new("/new/game.exe"))
        );

        let repo_fail = update(&current, "/new/game.exe", false);
        assert_eq!(
            repo_fail,
            ExePathUpdate::Rejected {
                attempted: LocalExePath::new("/new/game.exe"),
                error: ExePathError::RepositoryRejected,
                repository_ok: false,
            }
        );

        let invalid = update(&current, "relative/game.exe", true);
        assert!(!invalid.is_applied());
        assert!(invalid.applied_path().is_none());
    }
}
