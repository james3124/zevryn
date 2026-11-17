//! Resolution of a [`MenuOptionId`] to the portable action it should trigger.
//!
//! In the decompiled app every `Option` carries a Kotlin `Function1` click
//! lambda (`Option.f24952e` / `Option.b()`). Those lambdas perform Android
//! navigation and service calls (e.g. `PageRouterUtils.o`,
//! `UninstallGameConfirmHelper.h`, `ISteamGameDownloadService.c/l`) and are
//! intentionally *not* reproduced here. Instead, [`resolve`] maps the option
//! identity to a pure [`MenuAction`] that a platform layer can dispatch.

use super::option::MenuOptionId;

/// The portable outcome of activating one menu row.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MenuAction {
    /// Add the game to the local library (`llauncher_add_game_to_library`).
    AddToLibrary,
    /// Remove the game from the local library (`llauncher_remove_game_from_library`).
    RemoveFromLibrary,
    /// Open the PC-game settings page (`llauncher_pcgame_setting`).
    OpenPcGameSettings,
    /// Open the cloud-game settings page (`llauncher_cloud_game_setting`).
    OpenCloudSettings,
    /// Open the compatibility dialog (`llauncher_check_compatibility`).
    CheckCompatibility,
    /// Run the uninstall / remove confirmation (`winemu_uninstall_game_title`).
    Uninstall,
    /// Ignore the current Steam version update (`忽略此版本更新`).
    IgnoreUpdate,
    /// Cancel ignoring the Steam version update (`取消忽略此版本更新`).
    CancelIgnoreUpdate,
    /// Open the generic game settings page (`llauncher_game_setting`).
    OpenGameSettings,
    /// Close the popup without performing an action (back / outside tap).
    Dismiss,
}

/// Map a menu option to the action it represents.
///
/// Every [`MenuOptionId`] has a total mapping; `Dismiss` is not produced by
/// `resolve` (there is no "dismiss" row in the decompiled menu) but exists for
/// the platform layer to report a cancelled popup. It is returned by
/// [`super::state::MenuState::dismiss`] flows.
pub fn resolve(option_id: MenuOptionId) -> MenuAction {
    match option_id {
        MenuOptionId::AddToLibrary => MenuAction::AddToLibrary,
        MenuOptionId::RemoveFromLibrary => MenuAction::RemoveFromLibrary,
        MenuOptionId::PcGameSetting => MenuAction::OpenPcGameSettings,
        MenuOptionId::CloudGameSetting => MenuAction::OpenCloudSettings,
        MenuOptionId::CheckCompatibility => MenuAction::CheckCompatibility,
        MenuOptionId::UninstallGame => MenuAction::Uninstall,
        MenuOptionId::IgnoreUpdate => MenuAction::IgnoreUpdate,
        MenuOptionId::CancelIgnoreUpdate => MenuAction::CancelIgnoreUpdate,
        MenuOptionId::GameSettings => MenuAction::OpenGameSettings,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_option_resolves_to_its_action() {
        let cases = [
            (MenuOptionId::AddToLibrary, MenuAction::AddToLibrary),
            (MenuOptionId::RemoveFromLibrary, MenuAction::RemoveFromLibrary),
            (MenuOptionId::PcGameSetting, MenuAction::OpenPcGameSettings),
            (MenuOptionId::CloudGameSetting, MenuAction::OpenCloudSettings),
            (MenuOptionId::CheckCompatibility, MenuAction::CheckCompatibility),
            (MenuOptionId::UninstallGame, MenuAction::Uninstall),
            (MenuOptionId::IgnoreUpdate, MenuAction::IgnoreUpdate),
            (MenuOptionId::CancelIgnoreUpdate, MenuAction::CancelIgnoreUpdate),
            (MenuOptionId::GameSettings, MenuAction::OpenGameSettings),
        ];
        for (id, expected) in cases {
            assert_eq!(resolve(id), expected);
            assert_eq!(super::super::option::MenuOption::new(id).action(), expected);
        }
    }

    #[test]
    fn dismiss_is_not_an_option_action() {
        // Dismiss is produced by popup state, never by an option row.
        assert_ne!(resolve(MenuOptionId::GameSettings), MenuAction::Dismiss);
    }
}
