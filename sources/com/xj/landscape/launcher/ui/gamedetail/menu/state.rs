//! Open/closed and selection state for the long-press menu popup.
//!
//! The decompiled UI shows the options through `OptionsPopup` (an XPopup
//! subclass) and reports dismissal through a `Function0` callback
//! (`GameDetailSettingMenu.m0(..., dismiss)` / `MyGamesOperationPopupKt.k`).
//! None of that UI is portable, so this module keeps only the two pieces of
//! state that matter to logic: whether the popup is open and which row is
//! selected.

/// State of one menu popup instance.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct MenuState {
    open: bool,
    selected: Option<usize>,
}

impl MenuState {
    /// A closed popup with no selection.
    pub fn new() -> Self {
        Self::default()
    }

    /// Mark the popup as open and clear any previous selection.
    pub fn open(&mut self) {
        self.open = true;
        self.selected = None;
    }

    /// Close the popup and clear the selection.
    ///
    /// This is the portable counterpart of the decompiled `dismiss` callback.
    pub fn dismiss(&mut self) {
        self.open = false;
        self.selected = None;
    }

    /// Select a row by index.
    ///
    /// Returns `true` only when the popup is open and `index` is within
    /// `option_count`. An invalid selection leaves the state untouched.
    pub fn select(&mut self, index: usize, option_count: usize) -> bool {
        if !self.open || index >= option_count {
            return false;
        }
        self.selected = Some(index);
        true
    }

    /// Clear the current selection without closing the popup.
    pub fn clear_selection(&mut self) {
        self.selected = None;
    }

    pub fn is_open(&self) -> bool {
        self.open
    }

    pub fn is_closed(&self) -> bool {
        !self.open
    }

    pub fn selected(&self) -> Option<usize> {
        self.selected
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_closed_without_selection() {
        let state = MenuState::new();
        assert!(state.is_closed());
        assert!(!state.is_open());
        assert_eq!(state.selected(), None);
        assert_eq!(MenuState::new(), MenuState::default());
    }

    #[test]
    fn select_requires_open_state_and_valid_index() {
        let mut state = MenuState::new();
        assert!(!state.select(0, 3));
        assert_eq!(state.selected(), None);

        state.open();
        assert!(state.is_open());
        assert!(state.select(2, 3));
        assert_eq!(state.selected(), Some(2));

        // Out-of-range selections are rejected and keep the old selection.
        assert!(!state.select(3, 3));
        assert!(!state.select(usize::MAX, 3));
        assert_eq!(state.selected(), Some(2));
    }

    #[test]
    fn selecting_again_replaces_the_row() {
        let mut state = MenuState::new();
        state.open();
        assert!(state.select(0, 4));
        assert!(state.select(1, 4));
        assert_eq!(state.selected(), Some(1));
    }

    #[test]
    fn reopen_clears_selection() {
        let mut state = MenuState::new();
        state.open();
        state.select(1, 4);
        state.open();
        assert_eq!(state.selected(), None);
        assert!(state.is_open());
    }

    #[test]
    fn dismiss_closes_and_clears() {
        let mut state = MenuState::new();
        state.open();
        state.select(1, 4);
        state.dismiss();
        assert!(state.is_closed());
        assert_eq!(state.selected(), None);
        assert!(!state.select(0, 4));
    }

    #[test]
    fn clear_selection_keeps_popup_open() {
        let mut state = MenuState::new();
        state.open();
        state.select(0, 2);
        state.clear_selection();
        assert!(state.is_open());
        assert_eq!(state.selected(), None);
    }
}
