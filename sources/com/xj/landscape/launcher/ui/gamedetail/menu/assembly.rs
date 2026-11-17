//! Assembly of the game-card long-press menu.
//!
//! This is the portable equivalent of `GameDetailSettingMenu.getMenuOptions`
//! (decompiled as method `G`, smali lines 1531-2288) together with the option
//! builders it dispatches to:
//!
//! * `getAddOrRemoveGameLibOption` (`A`) — library option;
//! * `getCloudGameOptions` (`C`) — library option + cloud setting;
//! * `getPcGamesOptions` (`Y`) — uninstall/remove + compatibility + PC setting;
//! * `getPcUninstallOption` (`a0`) — uninstall or remove-from-library;
//! * `getPcGameUpdateOption` (`V`) — ignore / cancel-ignore update (this
//!   builder is invoked from the detail header in the original app, but its
//!   options are part of the same class and are modelled here).
//!
//! # Assembly rules (mirroring the decompiled `getMenuOptions`)
//!
//! The Java iterates the game's startup params and, for each one, dispatches
//! on `start_type`:
//!
//! 1. `isMobileGameStartType(...)` (mobile) -> `U()` -> only `A`.
//! 2. `start_type == 1406` (cloud, `e0`) -> `C()` -> `A(true)` when in library
//!    plus `CloudGameSetting`.
//! 3. `start_type in {1403, 1407, 1501}` (PC game, `g0`) and
//!    `!AppConfig.l()` -> `Y()` -> `a0()` (uninstall when the game has a local
//!    download and is in the library; `null` when not in the library),
//!    `E()` (compatibility when compatibility data exists), and
//!    `PcGameSetting` when the PC game is installed and in the library.
//! 4. `start_type in {1202, 1}` (`j0`) -> `d0()` -> `A(true)` when in library.
//! 5. Any other type -> `A()` directly.
//!
//! After the loop the Java removes duplicate library/uninstall rows (it looks
//! for `Option.id` 1, 2 and 3 and keeps the most specific entry), appends an
//! extra option when the list is non-empty and the game is in the library,
//! and finally **stably sorts by `Option.id` ascending**. Generic rows carry
//! id `-1`, uninstall `1`, add/remove-library `2`, remove-library `3`; hence
//! the final order is: generic rows (insertion order), uninstall, library.
//!
//! Our flag-based API collapses the startup-type loop into booleans:
//!
//! * [`MenuContext::is_in_library`] selects the library row that `A` builds —
//!   `RemoveFromLibrary` when true, `AddToLibrary` when false. (The
//!   decompiled `A` early-returns `null` for `false`, so the "Add" branch is
//!   unreachable in that build; we restore the intended behaviour.)
//! * [`MenuContext::is_cloud_game`] enables the `C` branch (cloud setting).
//! * [`MenuContext::is_pc_game`] enables the `Y` branch (uninstall,
//!   compatibility, PC-game setting).
//! * [`MenuContext::is_steam_game`] exposes the generic `GameSettings` entry
//!   and gates the Steam update option (`getPcGameUpdateOption` requires a
//!   Steam AppID > 0).
//! * [`MenuContext::has_update`] + [`MenuContext::is_update_ignored`] choose
//!   between `IgnoreUpdate` and `CancelIgnoreUpdate` (`k(appId)` in the
//!   decompiled service call).
//!
//! The generated rows are de-duplicated by [`MenuOptionId`] and stably sorted
//! by [`MenuOptionId::option_id`], matching the Java post-processing.

use super::option::{MenuOption, MenuOptionId};

/// Context flags that drive menu assembly.
///
/// The five first fields are the ones required by the port; `is_update_ignored`
/// is an optional extra that distinguishes the two Steam update rows
/// (`ISteamGameDownloadService.k(appId)` in the reference). It defaults to
/// `false`, i.e. an available update is offered as `IgnoreUpdate`.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct MenuContext {
    /// Whether the game is already in the local library (`z3` / `isInGameLib`).
    pub is_in_library: bool,
    /// PC-game startup params are present (`g0`: 1403 / 1407 / 1501).
    pub is_pc_game: bool,
    /// Cloud startup params are present (`e0`: 1406).
    pub is_cloud_game: bool,
    /// The game has a Steam AppID (`getPcGameUpdateOption` precondition).
    pub is_steam_game: bool,
    /// A newer Steam version is available (`hasUpdate`).
    pub has_update: bool,
    /// Whether the current version is already ignored (`k(appId)`).
    pub is_update_ignored: bool,
}

impl MenuContext {
    /// Construct a context from the five primary flags.
    pub fn new(
        is_in_library: bool,
        is_pc_game: bool,
        is_cloud_game: bool,
        is_steam_game: bool,
        has_update: bool,
    ) -> Self {
        Self {
            is_in_library,
            is_pc_game,
            is_cloud_game,
            is_steam_game,
            has_update,
            is_update_ignored: false,
        }
    }

    /// Set the ignored-update state (builder-style, mirrors `k(appId)`).
    pub fn ignored(mut self, is_update_ignored: bool) -> Self {
        self.is_update_ignored = is_update_ignored;
        self
    }
}

/// Build the ordered list of menu rows for the given context.
pub fn build_menu(ctx: MenuContext) -> Vec<MenuOption> {
    let mut options: Vec<MenuOption> = Vec::new();
    let mut push = |id: MenuOptionId| {
        // De-duplicate by identity, keeping the first entry. This is the
        // portable form of the `removeIf` calls on Option ids 1/2/3 in
        // `getMenuOptions` (smali lines 1980-2219).
        if !options.iter().any(|option| option.id == id) {
            options.push(MenuOption::new(id));
        }
    };

    // Step 1 — library option `A` (always present in our model).
    if ctx.is_in_library {
        push(MenuOptionId::RemoveFromLibrary);
    } else {
        push(MenuOptionId::AddToLibrary);
    }

    // Step 2 — cloud branch `C`, used when startup type is 1406.
    if ctx.is_cloud_game {
        if ctx.is_in_library {
            push(MenuOptionId::RemoveFromLibrary);
        }
        push(MenuOptionId::CloudGameSetting);
    }

    // Step 3 — PC-game branch `Y` / `a0` / `E`, used for 1403/1407/1501.
    if ctx.is_pc_game {
        // `a0` (getPcUninstallOption) returns null entirely when the game is
        // not in the library: the uninstall label is built only for
        // `isInGameLib == true`, and its remove-library fallback also
        // short-circuits to null in that case.
        if ctx.is_in_library {
            push(MenuOptionId::UninstallGame);
        }
        push(MenuOptionId::CheckCompatibility);
        if ctx.is_in_library {
            push(MenuOptionId::PcGameSetting);
        }
    }

    // Step 4 — generic game settings (`llauncher_game_setting`) for Steam games.
    if ctx.is_steam_game {
        push(MenuOptionId::GameSettings);
    }

    // Step 5 — Steam update option `V`; requires a Steam AppID and an update.
    if ctx.is_steam_game && ctx.has_update {
        if ctx.is_update_ignored {
            push(MenuOptionId::CancelIgnoreUpdate);
        } else {
            push(MenuOptionId::IgnoreUpdate);
        }
    }

    // Final step — stable ascending sort by the decompiled `Option.id`.
    // `sort_by_key` is stable, so equal-id rows preserve insertion order.
    options.sort_by_key(|option| option.id.option_id());
    options
}

/// Return the ids of the built menu, a compact form handy for tests and for
/// platform code that only needs the identities.
pub fn menu_option_ids(ctx: MenuContext) -> Vec<MenuOptionId> {
    build_menu(ctx).into_iter().map(|option| option.id).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ids(ctx: MenuContext) -> Vec<MenuOptionId> {
        menu_option_ids(ctx)
    }

    #[test]
    fn empty_context_only_offers_add_to_library() {
        // The decompiled `A` would return null here; our model restores the
        // intended Add branch (see module docs).
        assert_eq!(ids(MenuContext::default()), vec![MenuOptionId::AddToLibrary]);
    }

    #[test]
    fn in_library_only_offers_remove() {
        let ctx = MenuContext::new(true, false, false, false, false);
        assert_eq!(ids(ctx), vec![MenuOptionId::RemoveFromLibrary]);
    }

    #[test]
    fn cloud_branch_adds_cloud_settings() {
        let ctx = MenuContext::new(false, false, true, false, false);
        assert_eq!(
            ids(ctx),
            vec![MenuOptionId::CloudGameSetting, MenuOptionId::AddToLibrary]
        );
        let ctx = MenuContext::new(true, false, true, false, false);
        assert_eq!(
            ids(ctx),
            vec![MenuOptionId::CloudGameSetting, MenuOptionId::RemoveFromLibrary]
        );
    }

    #[test]
    fn pc_branch_adds_uninstall_compatibility_and_pc_settings() {
        let ctx = MenuContext::new(true, true, false, false, false);
        // Generic rows (-1) come first in decompiled id order, then
        // uninstall (1), then library (2).
        assert_eq!(
            ids(ctx),
            vec![
                MenuOptionId::CheckCompatibility,
                MenuOptionId::PcGameSetting,
                MenuOptionId::UninstallGame,
                MenuOptionId::RemoveFromLibrary,
            ]
        );
        // When not in the library `a0` contributes nothing: no uninstall and
        // no remove-library row, and the installed-only PC-settings row is
        // dropped too.
        let ctx = MenuContext::new(false, true, false, false, false);
        assert_eq!(
            ids(ctx),
            vec![
                MenuOptionId::CheckCompatibility,
                MenuOptionId::AddToLibrary,
            ]
        );
    }

    #[test]
    fn steam_update_choice() {
        let ctx = MenuContext::new(true, false, false, true, true);
        assert_eq!(
            ids(ctx),
            vec![
                MenuOptionId::GameSettings,
                MenuOptionId::IgnoreUpdate,
                MenuOptionId::RemoveFromLibrary,
            ]
        );
        let ctx = ctx.ignored(true);
        assert_eq!(
            ids(ctx),
            vec![
                MenuOptionId::GameSettings,
                MenuOptionId::CancelIgnoreUpdate,
                MenuOptionId::RemoveFromLibrary,
            ]
        );
        // No update -> no update row.
        let ctx = MenuContext::new(true, false, false, true, false);
        assert_eq!(
            ids(ctx),
            vec![MenuOptionId::GameSettings, MenuOptionId::RemoveFromLibrary]
        );
    }

    #[test]
    fn full_context_is_deduplicated_and_sorted() {
        let ctx = MenuContext::new(true, true, true, true, true);
        assert_eq!(
            ids(ctx),
            vec![
                MenuOptionId::CloudGameSetting,
                MenuOptionId::CheckCompatibility,
                MenuOptionId::PcGameSetting,
                MenuOptionId::GameSettings,
                MenuOptionId::IgnoreUpdate,
                MenuOptionId::UninstallGame,
                MenuOptionId::RemoveFromLibrary,
            ]
        );
        // Only one library row despite three branches producing one.
        assert_eq!(
            ids(ctx)
                .iter()
                .filter(|id| **id == MenuOptionId::RemoveFromLibrary)
                .count(),
            1
        );
    }

    #[test]
    fn built_options_have_canonical_labels_and_are_enabled() {
        let options = build_menu(MenuContext::new(true, true, true, true, true));
        assert!(!options.is_empty());
        for option in options {
            assert_eq!(option.label, option.id.label());
            assert!(option.enabled);
        }
    }
}
