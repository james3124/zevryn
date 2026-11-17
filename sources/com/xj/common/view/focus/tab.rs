//! Tab focus model: `AutoScrollMode` and the portable focus-index logic of
//! `FocusTabLayout`.
//!
//! Only the index/selection state machine is modelled. `FocusTabLayout` itself
//! is an Android `HorizontalScrollView` that measures and draws `ShapeTextView`
//! children; none of that is portable and it is intentionally left out.

use super::FocusDirection;

/// `com.xj.common.view.focus.focus.view.AutoScrollMode`.
///
/// The Java enum consults the RecyclerView to decide whether a position is
/// already visible enough. Here the visibility answer is supplied by the
/// caller as a [`Visibility`] value so this module stays View-free.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AutoScrollMode {
    CompletelyOrMaxVisible,
    Completely,
    MaxVisible,
}

impl AutoScrollMode {
    /// Mirrors `shouldScrollToPosition`: scroll only when the mode's
    /// visibility condition is *not* satisfied.
    pub fn should_scroll(self, visibility: Visibility) -> bool {
        match self {
            Self::CompletelyOrMaxVisible => {
                !(visibility.completely_visible || visibility.max_visible)
            }
            Self::Completely => !visibility.completely_visible,
            Self::MaxVisible => !visibility.max_visible,
        }
    }
}

/// Portable visibility facts used by [`AutoScrollMode::should_scroll`].
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Visibility {
    pub completely_visible: bool,
    pub max_visible: bool,
}

/// Focus-index model of `FocusTabLayout`.
///
/// Java uses `-1` as the "nothing selected / use default" sentinel for
/// `f24786j` (default index), `f24787k` (last selected) and `f24788l`
/// (selected tab). The same sentinel is kept here for fidelity.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FocusTabModel {
    tabs: Vec<String>,
    default_index: isize,
    last_selected: isize,
    selected: isize,
}

impl FocusTabModel {
    pub fn new() -> Self {
        Self {
            tabs: Vec::new(),
            default_index: -1,
            last_selected: -1,
            selected: -1,
        }
    }

    /// Build a model with tab ids and an optional default index.
    pub fn with_tabs<I, S>(tabs: I, default_index: Option<usize>) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        let mut model = Self::new();
        model.set_tabs(tabs, default_index);
        model
    }

    /// Replace the tab list (`setTabs`). A `None` default maps to `-1`.
    pub fn set_tabs<I, S>(&mut self, tabs: I, default_index: Option<usize>)
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        self.tabs = tabs.into_iter().map(Into::into).collect();
        self.default_index = default_index.map_or(-1, |i| i as isize);
        // FocusTabLayout.J resets the selected index when an explicit index
        // is supplied; we keep the current selection unless it is now invalid.
        if self.selected >= self.tabs.len() as isize {
            self.selected = -1;
        }
        if self.last_selected >= self.tabs.len() as isize {
            self.last_selected = -1;
        }
    }

    pub fn tabs(&self) -> &[String] {
        &self.tabs
    }

    pub fn tab_count(&self) -> usize {
        self.tabs.len()
    }

    pub fn selected(&self) -> isize {
        self.selected
    }

    pub fn last_selected(&self) -> isize {
        self.last_selected
    }

    pub fn default_index(&self) -> isize {
        self.default_index
    }

    /// Id of the currently selected tab, if any.
    pub fn selected_tab_id(&self) -> Option<&str> {
        usize::try_from(self.selected)
            .ok()
            .and_then(|i| self.tabs.get(i))
            .map(String::as_str)
    }

    /// Mirror of `FocusTabLayout.setSelectedTab(int)`.
    ///
    /// `-1` means "use the default index"; out-of-range values are clamped to
    /// `[0, len - 1]`. Returns `true` when the selection actually changed.
    pub fn set_selected_tab(&mut self, raw: isize) -> bool {
        let mut index = if raw == -1 { self.default_index } else { raw };
        if index < 0 {
            index = 0;
        } else if self.tabs.is_empty() || index >= self.tabs.len() as isize {
            index = self.tabs.len() as isize - 1;
        }
        // With no tabs, Java's method returns without changing the selection.
        if index < 0 || index >= self.tabs.len() as isize {
            return false;
        }
        if index == self.selected {
            return false;
        }
        self.last_selected = self.selected;
        self.selected = index;
        true
    }

    /// Select an absolute index (convenience wrapper).
    pub fn select(&mut self, index: usize) -> bool {
        self.set_selected_tab(index as isize)
    }

    /// Mirror of `FocusTabLayout.y(direction)`: LEFT goes one tab back when
    /// possible, RIGHT one tab forward; UP/DOWN are unhandled.
    pub fn next_index(&self, direction: FocusDirection) -> Option<usize> {
        let len = self.tabs.len() as isize;
        match direction {
            FocusDirection::Left if len > 0 && self.selected - 1 >= 0 => {
                Some((self.selected - 1) as usize)
            }
            FocusDirection::Right if len > 0 && self.selected + 1 <= len - 1 => {
                Some((self.selected + 1) as usize)
            }
            _ => None,
        }
    }

    /// Mirror of `FocusTabLayout.M(direction)`: move selection in a direction
    /// and "request focus" on the resulting tab. Returns `true` on success.
    pub fn request_focus(&mut self, direction: FocusDirection) -> bool {
        match self.next_index(direction) {
            Some(index) => self.set_selected_tab(index as isize),
            None => false,
        }
    }
}

impl Default for FocusTabModel {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn auto_scroll_modes() {
        let none = Visibility::default();
        assert!(AutoScrollMode::CompletelyOrMaxVisible.should_scroll(none));
        assert!(AutoScrollMode::Completely.should_scroll(none));
        assert!(AutoScrollMode::MaxVisible.should_scroll(none));

        let complete = Visibility { completely_visible: true, max_visible: false };
        assert!(!AutoScrollMode::CompletelyOrMaxVisible.should_scroll(complete));
        assert!(!AutoScrollMode::Completely.should_scroll(complete));
        assert!(AutoScrollMode::MaxVisible.should_scroll(complete));

        let max = Visibility { completely_visible: false, max_visible: true };
        assert!(!AutoScrollMode::CompletelyOrMaxVisible.should_scroll(max));
        assert!(AutoScrollMode::Completely.should_scroll(max));
        assert!(!AutoScrollMode::MaxVisible.should_scroll(max));
    }

    #[test]
    fn set_selected_clamps_and_tracks_last() {
        let mut model = FocusTabModel::with_tabs(["home", "games", "settings"], Some(1));
        assert_eq!(model.selected(), -1);
        assert!(model.set_selected_tab(-1)); // default -> 1
        assert_eq!(model.selected_tab_id(), Some("games"));
        assert!(model.select(2));
        assert_eq!(model.selected(), 2);
        assert_eq!(model.last_selected(), 1);
        assert!(!model.set_selected_tab(99)); // clamps to 2, already selected
        assert_eq!(model.selected(), 2);
        assert!(!model.set_selected_tab(2));
    }

    #[test]
    fn request_focus_moves_and_clamps_at_edges() {
        let mut model = FocusTabModel::with_tabs(["a", "b", "c"], None);
        assert!(model.set_selected_tab(0));
        assert!(!model.request_focus(FocusDirection::Left));
        assert!(model.request_focus(FocusDirection::Right));
        assert_eq!(model.selected_tab_id(), Some("b"));
        assert!(model.request_focus(FocusDirection::Right));
        assert_eq!(model.selected_tab_id(), Some("c"));
        assert!(!model.request_focus(FocusDirection::Right));
        assert!(model.request_focus(FocusDirection::Left));
        assert_eq!(model.selected_tab_id(), Some("b"));
        assert!(!model.request_focus(FocusDirection::Up));
        assert!(!model.request_focus(FocusDirection::Down));
    }

    #[test]
    fn right_from_unselected_goes_to_first() {
        let mut model = FocusTabModel::with_tabs(["a", "b"], None);
        assert_eq!(model.selected(), -1);
        assert_eq!(model.next_index(FocusDirection::Right), Some(0));
        assert!(model.request_focus(FocusDirection::Right));
        assert_eq!(model.selected_tab_id(), Some("a"));
    }

    #[test]
    fn empty_tabs_never_select() {
        let mut model = FocusTabModel::new();
        assert!(!model.set_selected_tab(0));
        assert!(!model.request_focus(FocusDirection::Right));
        assert_eq!(model.selected_tab_id(), None);
    }
}
