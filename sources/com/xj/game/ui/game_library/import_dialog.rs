//! Portable model of `SelectImportGameTypeDialog` (GameHub Lite v5.1.8).
//!
//! The decompiled dialog offers exactly **two** import choices, found in
//! `resources/res/layout/llauncher_dialog_select_import_game_type.xml` and the
//! labels in `resources/res/values/strings.xml`:
//!
//! | Choice        | Layout id          | Label string                        | English label |
//! |---------------|--------------------|-------------------------------------|---------------|
//! | [`ImportChoice::PcGame`]   | `llImportPcGame`   | `@string/game_type_pc_game`         | "PC Game"     |
//! | [`ImportChoice::MobileGame`]| `llImportMobileGame` | `@string/game_type_mobile_game_title` | "Mobile Game" |
//!
//! The dialog title is `@string/import_game` ("Import Game"). There is **no**
//! "from Steam" choice in this dialog; the `OpenSteamScan` UI action (required
//! by the portable action set) is reachable only through the separate
//! `ImportRequest::ScanSteamLibrary` path, documented in `import_event.rs`.
//!
//! Selection behavior: clicking a choice invokes its callback and dismisses
//! the dialog (`SelectImportGameTypeDialog.y0`); clicking `ivClose` just
//! dismisses (`w0`). We model that as `Option<ImportChoice>` where `None` means
//! nothing chosen / dismissed.

/// One of the import types offered by `SelectImportGameTypeDialog`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ImportChoice {
    /// `llImportPcGame` → emits `Navigate2ImportPcGameFileEvent` → file picker
    /// for `exe` (and `txt` when the logitech build flag is set).
    PcGame,
    /// `llImportMobileGame` → `PageRouterUtils.j(activity)` →
    /// `/launcher/ui/mobilegame/MobileGameV4Activity` (APK import).
    MobileGame,
}

impl ImportChoice {
    /// The decompiled layout id of the clickable row.
    pub fn layout_id(self) -> &'static str {
        match self {
            Self::PcGame => "llImportPcGame",
            Self::MobileGame => "llImportMobileGame",
        }
    }

    /// The Android string resource name used as the visible label.
    pub fn label_res(self) -> &'static str {
        match self {
            Self::PcGame => "game_type_pc_game",
            Self::MobileGame => "game_type_mobile_game_title",
        }
    }

    /// English label from `resources/res/values/strings.xml`.
    pub fn label(self) -> &'static str {
        match self {
            Self::PcGame => "PC Game",
            Self::MobileGame => "Mobile Game",
        }
    }

    /// All choices, in the order they appear in the layout (PC left, mobile right).
    pub const ALL: [Self; 2] = [Self::PcGame, Self::MobileGame];
}

impl Default for ImportChoice {
    fn default() -> Self {
        Self::PcGame
    }
}

/// Selectable state of the import-type dialog. Pure: no Android dialog, no
/// focus handling, no coroutine delay.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ImportDialogModel {
    /// The currently selected choice, or `None` when dismissed/unselected.
    pub selected: Option<ImportChoice>,
}

impl Default for ImportDialogModel {
    fn default() -> Self {
        Self { selected: None }
    }
}

impl ImportDialogModel {
    pub fn new() -> Self {
        Self::default()
    }

    /// Record a choice (mirrors the row click callback + dismiss).
    pub fn select(&mut self, choice: ImportChoice) {
        self.selected = Some(choice);
    }

    /// Dismiss without choosing (mirrors `ivClose` / `dismissAllowingStateLoss`).
    pub fn dismiss(&mut self) {
        self.selected = None;
    }

    pub fn selected(&self) -> Option<ImportChoice> {
        self.selected
    }

    pub fn is_open_with_choice(&self, choice: ImportChoice) -> bool {
        self.selected == Some(choice)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn choices_match_decompiled_strings() {
        assert_eq!(ImportChoice::PcGame.layout_id(), "llImportPcGame");
        assert_eq!(ImportChoice::PcGame.label_res(), "game_type_pc_game");
        assert_eq!(ImportChoice::PcGame.label(), "PC Game");
        assert_eq!(ImportChoice::MobileGame.layout_id(), "llImportMobileGame");
        assert_eq!(ImportChoice::MobileGame.label_res(), "game_type_mobile_game_title");
        assert_eq!(ImportChoice::MobileGame.label(), "Mobile Game");
        assert_eq!(ImportChoice::ALL.len(), 2);
    }

    #[test]
    fn dialog_selection_round_trip() {
        let mut d = ImportDialogModel::new();
        assert_eq!(d.selected(), None);
        d.select(ImportChoice::PcGame);
        assert_eq!(d.selected(), Some(ImportChoice::PcGame));
        assert!(d.is_open_with_choice(ImportChoice::PcGame));
        d.select(ImportChoice::MobileGame);
        assert_eq!(d.selected(), Some(ImportChoice::MobileGame));
    }

    #[test]
    fn dismiss_clears_selection() {
        let mut d = ImportDialogModel::new();
        d.select(ImportChoice::PcGame);
        d.dismiss();
        assert_eq!(d.selected(), None);
    }
}
