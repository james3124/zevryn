//! The portable option model for the game-card long-press menu
//! (`GameDetailSettingMenu` / `MyGamesOperationPopupKt`).
//!
//! Each [`MenuOptionId`] corresponds to exactly one `Option` that the
//! decompiled `com.xj.landscape.launcher.ui.gamedetail.GameDetailSettingMenu`
//! can construct. The label text is the *verified* English value from
//! `resources/res/values/strings.xml` (or the literal Chinese string that is
//! hard-coded in the Java for the Steam "ignore update" entries). See
//! `REPORT.md` in this directory for the exact source lines.
//!
//! No Android dependency is used: the labels are stored as `&'static str`
//! instead of being resolved through `Context.getString(R.string.*)`.

/// Identity of a menu row, one per distinct option built by
/// `GameDetailSettingMenu`.
///
/// The `option_id()` helper returns the integer stored in the decompiled
/// `Option.id` field (`Option.f24951d`, exposed by `Option.a()`), which the
/// original `getMenuOptions` uses to de-duplicate and finally sort the rows.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum MenuOptionId {
    /// `llauncher_add_game_to_library` — "Add Game Library".
    AddToLibrary,
    /// `llauncher_remove_game_from_library` — "Remove Library".
    RemoveFromLibrary,
    /// `llauncher_cloud_game_setting` — "Cloud Game Settings".
    CloudGameSetting,
    /// `llauncher_check_compatibility` — "Check Compatibility".
    CheckCompatibility,
    /// `llauncher_pcgame_setting` — "PC Game Settings".
    PcGameSetting,
    /// `winemu_uninstall_game_title` — "Uninstall game".
    UninstallGame,
    /// Hard-coded `"忽略此版本更新"` (ignore this version update).
    IgnoreUpdate,
    /// Hard-coded `"取消忽略此版本更新"` (cancel ignoring this version update).
    CancelIgnoreUpdate,
    /// `llauncher_game_setting` — "Game Settings" (generic settings entry).
    GameSettings,
}

impl MenuOptionId {
    /// The user-visible label, exactly as found in the reference build.
    pub fn label(self) -> &'static str {
        match self {
            MenuOptionId::AddToLibrary => "Add Game Library",
            MenuOptionId::RemoveFromLibrary => "Remove Library",
            MenuOptionId::CloudGameSetting => "Cloud Game Settings",
            MenuOptionId::CheckCompatibility => "Check Compatibility",
            MenuOptionId::PcGameSetting => "PC Game Settings",
            MenuOptionId::UninstallGame => "Uninstall game",
            MenuOptionId::IgnoreUpdate => "忽略此版本更新",
            MenuOptionId::CancelIgnoreUpdate => "取消忽略此版本更新",
            MenuOptionId::GameSettings => "Game Settings",
        }
    }

    /// The `R.string` symbol whose value is [`MenuOptionId::label`].
    ///
    /// Returns `None` for the two update options: the decompiled Java passes
    /// those Chinese strings as literal arguments to the `Option` constructor
    /// (`GameDetailSettingMenu.V`), so no string resource exists for them.
    pub fn string_resource(self) -> Option<&'static str> {
        match self {
            MenuOptionId::AddToLibrary => Some("llauncher_add_game_to_library"),
            MenuOptionId::RemoveFromLibrary => Some("llauncher_remove_game_from_library"),
            MenuOptionId::CloudGameSetting => Some("llauncher_cloud_game_setting"),
            MenuOptionId::CheckCompatibility => Some("llauncher_check_compatibility"),
            MenuOptionId::PcGameSetting => Some("llauncher_pcgame_setting"),
            MenuOptionId::UninstallGame => Some("winemu_uninstall_game_title"),
            MenuOptionId::IgnoreUpdate | MenuOptionId::CancelIgnoreUpdate => None,
            MenuOptionId::GameSettings => Some("llauncher_game_setting"),
        }
    }

    /// Value of the decompiled `Option.id` field (`Option.a()`), used by
    /// `getMenuOptions` to remove duplicate library/uninstall rows and to
    /// stably sort the final list in ascending order.
    ///
    /// * `-1` — generic rows built with the `0xe` synthetic-constructor mask
    ///   (cloud setting, compatibility, PC setting, update rows, ...).
    /// * `1` — uninstall (`getPcUninstallOption`).
    /// * `2` — add/remove library (`getAddOrRemoveGameLibOption`).
    /// * `3` — remove-library variant emitted by `getPcUninstallOption` when
    ///   the PC game has no local download. Modelled here by the same
    ///   [`MenuOptionId::RemoveFromLibrary`] variant.
    pub fn option_id(self) -> i32 {
        match self {
            MenuOptionId::UninstallGame => 1,
            MenuOptionId::AddToLibrary | MenuOptionId::RemoveFromLibrary => 2,
            MenuOptionId::CloudGameSetting
            | MenuOptionId::CheckCompatibility
            | MenuOptionId::PcGameSetting
            | MenuOptionId::IgnoreUpdate
            | MenuOptionId::CancelIgnoreUpdate
            | MenuOptionId::GameSettings => -1,
        }
    }
}

/// One row of the game-card long-press menu.
///
/// Mirrors the portable fields of `com.xj.common.view.popup.Option`:
/// `id` (`Option.a()`), `text` (`Option.d()`) and the "checked/disabled"
/// boolean (`Option.e()`). The Android click lambda (`Option.b()`) is
/// deliberately not carried over; see [`super::action::resolve`] instead.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct MenuOption {
    pub id: MenuOptionId,
    pub label: &'static str,
    pub enabled: bool,
}

impl MenuOption {
    /// Build an enabled option with the canonical label for `id`.
    pub fn new(id: MenuOptionId) -> Self {
        Self {
            id,
            label: id.label(),
            enabled: true,
        }
    }

    /// Build an option with an explicit label (used when a caller overrides
    /// the resource text) and explicit enabled state.
    pub fn with_label(id: MenuOptionId, label: &'static str, enabled: bool) -> Self {
        Self { id, label, enabled }
    }

    /// Action produced when this option is activated.
    pub fn action(self) -> super::action::MenuAction {
        super::action::resolve(self.id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn labels_and_resources_match_reference() {
        let cases = [
            (MenuOptionId::AddToLibrary, "Add Game Library", "llauncher_add_game_to_library"),
            (MenuOptionId::RemoveFromLibrary, "Remove Library", "llauncher_remove_game_from_library"),
            (MenuOptionId::CloudGameSetting, "Cloud Game Settings", "llauncher_cloud_game_setting"),
            (MenuOptionId::CheckCompatibility, "Check Compatibility", "llauncher_check_compatibility"),
            (MenuOptionId::PcGameSetting, "PC Game Settings", "llauncher_pcgame_setting"),
            (MenuOptionId::UninstallGame, "Uninstall game", "winemu_uninstall_game_title"),
            (MenuOptionId::IgnoreUpdate, "忽略此版本更新", ""),
            (MenuOptionId::CancelIgnoreUpdate, "取消忽略此版本更新", ""),
            (MenuOptionId::GameSettings, "Game Settings", "llauncher_game_setting"),
        ];
        for (id, label, resource) in cases {
            assert_eq!(id.label(), label);
            assert_eq!(id.string_resource().unwrap_or(""), resource);
            let option = MenuOption::new(id);
            assert_eq!(option.label, label);
            assert!(option.enabled);
        }
    }

    #[test]
    fn decompiled_option_ids() {
        assert_eq!(MenuOptionId::UninstallGame.option_id(), 1);
        assert_eq!(MenuOptionId::AddToLibrary.option_id(), 2);
        assert_eq!(MenuOptionId::RemoveFromLibrary.option_id(), 2);
        for id in [
            MenuOptionId::CloudGameSetting,
            MenuOptionId::CheckCompatibility,
            MenuOptionId::PcGameSetting,
            MenuOptionId::IgnoreUpdate,
            MenuOptionId::CancelIgnoreUpdate,
            MenuOptionId::GameSettings,
        ] {
            assert_eq!(id.option_id(), -1);
        }
    }

    #[test]
    fn explicit_label_override() {
        let option = MenuOption::with_label(MenuOptionId::GameSettings, "Custom", false);
        assert_eq!(option.label, "Custom");
        assert!(!option.enabled);
    }
}
