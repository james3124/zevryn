//! The individual actions that make up an [`UninstallPlan`](super::UninstallPlan).
//!
//! Each variant is paired with the decompiled method it represents, so the
//! pure executor never pretends to do Android/file I/O — it only records that
//! the corresponding source-side action was selected.

use std::fmt;

/// One actionable step of an uninstall / remove-from-library flow, in the
/// order in which the Java helpers perform them.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum UninstallStep {
    /// User confirmation gate.
    ///
    /// Source: `UninstallGameConfirmHelper.h()` / `i()` ->
    /// `CommUninstallConfirmDialog` (`showRemoveGameConfirmDialog$1` /
    /// `showRemovePcGameConfirmDialog$1`). Title key
    /// `winemu_uninstall_game_title` ("Uninstall game" / "卸载游戏").
    ConfirmDialog,

    /// Delete a game downloaded from GameHub's online server.
    ///
    /// Source: `UninstallGameHelper.h()` -> `deleteDownloadOnlineGame`
    /// (`IWinEmuService.u(...)`, lines 156-192).
    DeleteOnlineDownload,

    /// Delete the game's Steam installation / Steam-tool registration.
    ///
    /// Source: `UninstallGameHelper.m()` -> `deleteSteamGameByDownload`
    /// (`ISteamGameService.s(appId)`, lines 267-303) for
    /// `DOWNLOAD_FROM_STEAM_SVR`, and `UninstallGameHelper.l()` ->
    /// `IWinEmuService.h(steamAppId)` (lines 260-264) for
    /// `IMPORT_BY_STEAM_TOOL`.
    DeleteSteamGame,

    /// Delete the locally cached cover image of the game.
    ///
    /// Source: `UninstallGameHelper.j()`/`k()` -> `deleteIfGameCoverInLocal`
    /// (lines 200-258) using `cover_image`, plus `realUninstallGame()`/`r()`
    /// lines 471-481 deleting `localGameIconPath` for `IMPORT_BY_USER`.
    DeleteLocalCover,

    /// Clear per-game state / refresh the launcher home data.
    ///
    /// Source: `UninstallGameHelper$uninstallPcDemoGame$2` ->
    /// `GameStateMgr.j(gameId)`, and `UninstallGameConfirmHelper`
    /// `resetSingleGame()`/`toUninstallPcGame$1` -> `IHomeService.m(validId)`.
    ResetGameData,

    /// Remove the PC-demo (emulator container) files for a server game.
    ///
    /// Source: `UninstallGameHelper.v()` -> `uninstallPcDemoGame`
    /// (`IWinEmuService.j(gameId)`, `UninstallGameHelper$uninstallPcDemoGame$2`).
    UninstallPcDemo,

    /// Remove the library row and broadcast `RemoveGameLibraryEvent`.
    ///
    /// Source: `UninstallGameConfirmHelper.f()` -> `removeFromGameLibrary`
    /// (`GameLibraryTableHelper.l(...)`, lines 65-263).
    RemoveFromLibrary,
}

impl UninstallStep {
    /// Stable, lowercase identifier for logging/tests.
    pub const fn as_str(self) -> &'static str {
        match self {
            UninstallStep::ConfirmDialog => "confirm_dialog",
            UninstallStep::DeleteOnlineDownload => "delete_online_download",
            UninstallStep::DeleteSteamGame => "delete_steam_game",
            UninstallStep::DeleteLocalCover => "delete_local_cover",
            UninstallStep::ResetGameData => "reset_game_data",
            UninstallStep::UninstallPcDemo => "uninstall_pc_demo",
            UninstallStep::RemoveFromLibrary => "remove_from_library",
        }
    }

    /// Short reference to the decompiled source that produces this step.
    pub const fn source(self) -> &'static str {
        match self {
            UninstallStep::ConfirmDialog => "UninstallGameConfirmHelper.h/i + CommUninstallConfirmDialog",
            UninstallStep::DeleteOnlineDownload => "UninstallGameHelper.h (deleteDownloadOnlineGame)",
            UninstallStep::DeleteSteamGame => "UninstallGameHelper.m / l (deleteSteamGameByDownload / IWinEmuService.h)",
            UninstallStep::DeleteLocalCover => "UninstallGameHelper.j/k + r (deleteIfGameCoverInLocal)",
            UninstallStep::ResetGameData => "GameStateMgr.j + IHomeService.m (uninstallPcDemoGame / resetSingleGame)",
            UninstallStep::UninstallPcDemo => "UninstallGameHelper.v (uninstallPcDemoGame)",
            UninstallStep::RemoveFromLibrary => "UninstallGameConfirmHelper.f (removeFromGameLibrary)",
        }
    }
}

impl fmt::Display for UninstallStep {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn step_identifiers_unique() {
        let all = [
            UninstallStep::ConfirmDialog,
            UninstallStep::DeleteOnlineDownload,
            UninstallStep::DeleteSteamGame,
            UninstallStep::DeleteLocalCover,
            UninstallStep::ResetGameData,
            UninstallStep::UninstallPcDemo,
            UninstallStep::RemoveFromLibrary,
        ];
        for (i, a) in all.iter().enumerate() {
            for b in all.iter().skip(i + 1) {
                assert_ne!(a.as_str(), b.as_str());
            }
            assert!(!a.source().is_empty());
        }
    }
}
