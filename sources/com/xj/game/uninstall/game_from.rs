//! `GameFrom` — where a game in the library came from.
//!
//! Exact port of `com.xj.game.UninstallGameHelper.GameFrom`
//! (`UninstallGameHelper.java`, lines 61-65). The four enum constants and their
//! declaration order are identical to the decompiled Java, so [`GameFrom::ordinal`]
//! matches the JVM `Enum.ordinal()` and [`GameFrom::when_mapping`] matches the
//! `UninstallGameHelper.WhenMappings.f26492a` lookup table (lines 78-103).
//!
//! The value is normally produced by `UninstallGameHelper.o()`/`getGameFrom()`
//! (lines 311-379):
//! * `DOWNLOAD_FROM_STEAM_SVR` — Steam service reports the app as downloaded
//!   or in the download queue (`g()` / `checkIsSteamGameDownloadBySteamService`).
//! * `IMPORT_BY_USER` — a local game (`GameLibraryRepository.a(localGameId)`)
//!   without `steam_game_info`.
//! * `IMPORT_BY_STEAM_TOOL` — a local game that has `steam_game_info` (imported
//!   through the Steam tool).
//! * `DOWNLOAD_FROM_SVR` — a server game tracked by `GameStateMgr`
//!   (`f(id)`, `d(id)` or the async `e(id)` check).
//!
//! When none of those checks match, the Java helper returns `null` and
//! `realUninstallGame()` refuses to uninstall ("未识别的游戏来源，不进行卸载操作").

use std::fmt;

/// The four exact constants of `UninstallGameHelper.GameFrom`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum GameFrom {
    /// `DOWNLOAD_FROM_SVR` — game downloaded from GameHub's own server.
    DownloadFromSvr,
    /// `IMPORT_BY_USER` — game imported manually by the user (local files).
    ImportByUser,
    /// `IMPORT_BY_STEAM_TOOL` — local game imported via the Steam tool.
    ImportBySteamTool,
    /// `DOWNLOAD_FROM_STEAM_SVR` — game downloaded through the Steam service.
    DownloadFromSteamSvr,
}

impl GameFrom {
    /// All variants in Java declaration (ordinal) order.
    pub const ALL: [GameFrom; 4] = [
        GameFrom::DownloadFromSvr,
        GameFrom::ImportByUser,
        GameFrom::ImportBySteamTool,
        GameFrom::DownloadFromSteamSvr,
    ];

    /// The exact Java enum constant name.
    pub const fn java_name(self) -> &'static str {
        match self {
            GameFrom::DownloadFromSvr => "DOWNLOAD_FROM_SVR",
            GameFrom::ImportByUser => "IMPORT_BY_USER",
            GameFrom::ImportBySteamTool => "IMPORT_BY_STEAM_TOOL",
            GameFrom::DownloadFromSteamSvr => "DOWNLOAD_FROM_STEAM_SVR",
        }
    }

    /// Zero-based JVM ordinal (declaration order).
    pub const fn ordinal(self) -> usize {
        match self {
            GameFrom::DownloadFromSvr => 0,
            GameFrom::ImportByUser => 1,
            GameFrom::ImportBySteamTool => 2,
            GameFrom::DownloadFromSteamSvr => 3,
        }
    }

    /// The 1-based index assigned by `WhenMappings.f26492a`
    /// (`UninstallGameHelper.java`, lines 84-101). This is the value switched
    /// on by `realUninstallGame()`/`r()`.
    pub const fn when_mapping(self) -> usize {
        self.ordinal() + 1
    }

    /// `isNeed2UninstallGame()` (`q()`, lines 386-449) is false only for a
    /// user-imported game — its files belong to the user and must not be
    /// deleted, only the library entry is removed.
    pub const fn needs_game_uninstall(self) -> bool {
        !matches!(self, GameFrom::ImportByUser)
    }

    /// Recover a variant from its Java ordinal, if valid.
    pub const fn from_ordinal(ordinal: usize) -> Option<GameFrom> {
        match ordinal {
            0 => Some(GameFrom::DownloadFromSvr),
            1 => Some(GameFrom::ImportByUser),
            2 => Some(GameFrom::ImportBySteamTool),
            3 => Some(GameFrom::DownloadFromSteamSvr),
            _ => None,
        }
    }
}

impl fmt::Display for GameFrom {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.java_name())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn exact_java_constants_and_order() {
        let names: Vec<&str> = GameFrom::ALL.iter().map(|v| v.java_name()).collect();
        assert_eq!(
            names,
            [
                "DOWNLOAD_FROM_SVR",
                "IMPORT_BY_USER",
                "IMPORT_BY_STEAM_TOOL",
                "DOWNLOAD_FROM_STEAM_SVR",
            ]
        );
        for (i, v) in GameFrom::ALL.iter().enumerate() {
            assert_eq!(v.ordinal(), i);
            assert_eq!(v.when_mapping(), i + 1);
            assert_eq!(GameFrom::from_ordinal(i), Some(*v));
        }
        assert_eq!(GameFrom::from_ordinal(4), None);
        assert_eq!(GameFrom::DownloadFromSteamSvr.to_string(), "DOWNLOAD_FROM_STEAM_SVR");
    }

    #[test]
    fn only_user_import_skips_game_uninstall() {
        assert!(!GameFrom::ImportByUser.needs_game_uninstall());
        for v in [GameFrom::DownloadFromSvr, GameFrom::ImportBySteamTool, GameFrom::DownloadFromSteamSvr] {
            assert!(v.needs_game_uninstall());
        }
    }
}
