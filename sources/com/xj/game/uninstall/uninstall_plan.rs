//! Pure construction of an ordered [`UninstallPlan`] from a game's origin and
//! the state flags the Java helpers check at runtime.
//!
//! The ordering mirrors `UninstallGameConfirmHelper.toUninstallPcGame$1`
//! (`invokeSuspend`): show the confirm dialog, run `UninstallGameHelper` for the
//! source-specific game removal, delete the cached cover
//! (`deleteIfGameCoverInLocal`), call `removeFromGameLibrary` and finally
//! refresh home state via `IHomeService.m`.

use super::game_from::GameFrom;
use super::uninstall_step::UninstallStep;

/// The minimal game identity the plan/executor need.
///
/// Maps to the fields used by `UninstallGameHelper.uninstallGame` and
/// `UninstallGameConfirmHelper`: `gameSvrId`/`validId`, `localGameId` and
/// `steamAppId` (`GameDetailEntity`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Game {
    /// Server/valid id used as the library key (`gameSvrId` / `validId`).
    pub id: String,
    /// Local import id (`localGameId`), used for user-imported games.
    pub local_game_id: String,
    /// Numeric Steam app id (`steamAppId`), when the game has one.
    pub steam_app_id: String,
    /// Display name (used when building the dialog text in the reference).
    pub name: String,
}

impl Game {
    pub fn new(id: impl Into<String>, steam_app_id: impl Into<String>) -> Self {
        Game {
            id: id.into(),
            local_game_id: String::new(),
            steam_app_id: steam_app_id.into(),
            name: String::new(),
        }
    }
}

/// Runtime state checks that decide which optional steps are added.
///
/// * `is_steam_downloaded` — `UninstallGameHelper.g()`
///   (`checkIsSteamGameDownloadBySteamService`): Steam reports the app as
///   `Downloaded` or `InDownloadQueue`.
/// * `is_in_library` — the game still has a row in `GameLibraryTable`; gates
///   `UninstallGameConfirmHelper.f()` (`removeFromGameLibrary`).
/// * `has_local_cover` — `cover_image` / `localGameIconPath` is a local file
///   that can be deleted (`deleteIfGameCoverInLocal`).
/// * `is_pc_demo` — the server game is a PC demo, so the emulator container is
///   uninstalled too (`uninstallPcDemoGame` -> `IWinEmuService.j`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct UninstallFlags {
    pub is_steam_downloaded: bool,
    pub is_in_library: bool,
    pub has_local_cover: bool,
    pub is_pc_demo: bool,
}

impl Default for UninstallFlags {
    fn default() -> Self {
        UninstallFlags {
            is_steam_downloaded: false,
            is_in_library: true,
            has_local_cover: false,
            is_pc_demo: false,
        }
    }
}

impl UninstallFlags {
    pub fn new(
        is_steam_downloaded: bool,
        is_in_library: bool,
        has_local_cover: bool,
        is_pc_demo: bool,
    ) -> Self {
        UninstallFlags {
            is_steam_downloaded,
            is_in_library,
            has_local_cover,
            is_pc_demo,
        }
    }
}

/// An ordered, purely descriptive uninstall plan.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UninstallPlan {
    /// Library key that a [`RemoveLibrary`](super::executor::RemoveLibrary)
    /// implementation should pass to `remove`.
    pub game_id: String,
    /// Classified origin of the game.
    pub from: GameFrom,
    /// Steps in execution order.
    pub steps: Vec<UninstallStep>,
}

impl UninstallPlan {
    /// Build the plan for `game` of origin `from` with the given state flags.
    pub fn new(game: &Game, from: GameFrom, flags: UninstallFlags) -> Self {
        let mut steps = Vec::new();

        // Confirmation is always shown first: showRemoveGameConfirmDialog /
        // showRemovePcGameConfirmDialog, titled `winemu_uninstall_game_title`.
        steps.push(UninstallStep::ConfirmDialog);

        // Source-specific game-file removal, from realUninstallGame()/r():
        // WhenMappings.f26492a -> 1..4.
        match from {
            GameFrom::DownloadFromSvr => {
                // r() case 1 -> v() -> uninstallPcDemoGame: the online
                // server download is deleted first.
                steps.push(UninstallStep::DeleteOnlineDownload);
                if flags.is_pc_demo {
                    steps.push(UninstallStep::UninstallPcDemo);
                }
                // uninstallPcDemoGame always calls GameStateMgr.j(gameId).
                steps.push(UninstallStep::ResetGameData);
            }
            GameFrom::ImportByUser => {
                // r() case 2 only deletes the local cover icon; no game-file
                // removal (isNeed2UninstallGame() is false for this source).
            }
            GameFrom::ImportBySteamTool => {
                // r() case 3 -> l(steamAppId) -> IWinEmuService.h: remove the
                // Steam-tool registration.
                steps.push(UninstallStep::DeleteSteamGame);
            }
            GameFrom::DownloadFromSteamSvr => {
                // r() case 4 -> m(steamAppId) -> ISteamGameService.s: only
                // meaningful when the Steam service reports it downloaded.
                if flags.is_steam_downloaded {
                    steps.push(UninstallStep::DeleteSteamGame);
                }
            }
        }

        // resetSingleGame()/toUninstallPcGame(): deleteIfGameCoverInLocal.
        if flags.has_local_cover {
            steps.push(UninstallStep::DeleteLocalCover);
        }

        // removeFromGameLibrary(), then IHomeService.m(validId) refresh.
        if flags.is_in_library {
            steps.push(UninstallStep::RemoveFromLibrary);
            // Avoid emitting ResetGameData twice for DOWNLOAD_FROM_SVR.
            if !steps.contains(&UninstallStep::ResetGameData) {
                steps.push(UninstallStep::ResetGameData);
            }
        }

        UninstallPlan {
            game_id: game.id.clone(),
            from,
            steps,
        }
    }

    /// Convenience wrapper using only an id/steam id.
    pub fn for_ids(
        game_id: impl Into<String>,
        steam_app_id: impl Into<String>,
        from: GameFrom,
        flags: UninstallFlags,
    ) -> Self {
        UninstallPlan::new(&Game::new(game_id, steam_app_id), from, flags)
    }

    pub fn game_id(&self) -> &str {
        &self.game_id
    }

    pub fn steps(&self) -> &[UninstallStep] {
        &self.steps
    }

    pub fn needs_library_removal(&self) -> bool {
        self.steps.contains(&UninstallStep::RemoveFromLibrary)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn game() -> Game {
        Game::new("42", "620")
    }

    fn ids(plan: &UninstallPlan) -> Vec<UninstallStep> {
        plan.steps.clone()
    }

    #[test]
    fn all_four_game_from_values_have_distinct_core_plans() {
        // No optional flags: only library removal is kept.
        let f = UninstallFlags::default();
        let svr = UninstallPlan::new(&game(), GameFrom::DownloadFromSvr, f);
        assert_eq!(
            ids(&svr),
            [
                UninstallStep::ConfirmDialog,
                UninstallStep::DeleteOnlineDownload,
                UninstallStep::ResetGameData,
                UninstallStep::RemoveFromLibrary,
            ]
        );

        let user = UninstallPlan::new(&game(), GameFrom::ImportByUser, f);
        assert_eq!(
            ids(&user),
            [
                UninstallStep::ConfirmDialog,
                UninstallStep::RemoveFromLibrary,
                UninstallStep::ResetGameData,
            ]
        );

        let tool = UninstallPlan::new(&game(), GameFrom::ImportBySteamTool, f);
        assert_eq!(
            ids(&tool),
            [
                UninstallStep::ConfirmDialog,
                UninstallStep::DeleteSteamGame,
                UninstallStep::RemoveFromLibrary,
                UninstallStep::ResetGameData,
            ]
        );

        // Steam download without the "downloaded" flag: no Steam deletion.
        let steam = UninstallPlan::new(&game(), GameFrom::DownloadFromSteamSvr, f);
        assert_eq!(
            ids(&steam),
            [
                UninstallStep::ConfirmDialog,
                UninstallStep::RemoveFromLibrary,
                UninstallStep::ResetGameData,
            ]
        );
    }

    #[test]
    fn steam_downloaded_flag_enables_steam_deletion() {
        let flags = UninstallFlags::new(true, true, false, false);
        let plan = UninstallPlan::new(&game(), GameFrom::DownloadFromSteamSvr, flags);
        assert_eq!(
            ids(&plan),
            [
                UninstallStep::ConfirmDialog,
                UninstallStep::DeleteSteamGame,
                UninstallStep::RemoveFromLibrary,
                UninstallStep::ResetGameData,
            ]
        );
    }

    #[test]
    fn pc_demo_and_cover_flags_add_their_steps_in_order() {
        let flags = UninstallFlags::new(false, true, true, true);
        let plan = UninstallPlan::new(&game(), GameFrom::DownloadFromSvr, flags);
        // Confirm -> online download -> pc demo -> reset data -> cover ->
        // library removal (ResetGameData is not duplicated at the end).
        assert_eq!(
            ids(&plan),
            [
                UninstallStep::ConfirmDialog,
                UninstallStep::DeleteOnlineDownload,
                UninstallStep::UninstallPcDemo,
                UninstallStep::ResetGameData,
                UninstallStep::DeleteLocalCover,
                UninstallStep::RemoveFromLibrary,
            ]
        );
        assert_eq!(
            plan.steps
                .iter()
                .filter(|s| **s == UninstallStep::ResetGameData)
                .count(),
            1
        );
    }

    #[test]
    fn not_in_library_drops_library_step_and_trailing_reset() {
        let flags = UninstallFlags::new(true, false, true, false);
        let plan = UninstallPlan::new(&game(), GameFrom::ImportBySteamTool, flags);
        assert_eq!(
            ids(&plan),
            [
                UninstallStep::ConfirmDialog,
                UninstallStep::DeleteSteamGame,
                UninstallStep::DeleteLocalCover,
            ]
        );
        assert!(!plan.needs_library_removal());
        assert_eq!(plan.game_id(), "42");
    }
}
