//! Portable state for the launcher's main ViewModels.
//!
//! Sources: `NewLandscapeLauncherMainVM` (game-circle list, the `(index,
//! needRefresh)` tab-selection pair posted by `n(int, boolean)`, the `t()`
//! loading flag, network status) and the deprecated `LandscapeLauncherMainVM`
//! (tab/list LiveData and the `getGameCircleList` loading callback).
//!
//! Network I/O, coroutine scopes, `MessageQueue.IdleHandler` and
//! `NetworkUtils` listeners are not modeled — their results are fed into the
//! state machine through the `on_*` methods.

use super::types::{GameCircle, TabItem};

/// Coarse data-refresh state of the home screen.
///
/// Combines the boolean loading flag (`NewLandscapeLauncherMainVM.f30447e`)
/// with the outcome of the `getGameCircleList` coroutine.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RefreshState {
    Idle,
    Loading,
    Ready,
    Failed(String),
}

impl RefreshState {
    pub fn is_loading(&self) -> bool {
        matches!(self, RefreshState::Loading)
    }

    pub fn is_ready(&self) -> bool {
        matches!(self, RefreshState::Ready)
    }

    pub fn error(&self) -> Option<&str> {
        match self {
            RefreshState::Failed(e) => Some(e.as_str()),
            _ => None,
        }
    }

    pub fn begin(&mut self) {
        *self = RefreshState::Loading;
    }

    pub fn ready(&mut self) {
        *self = RefreshState::Ready;
    }

    pub fn fail<S: Into<String>>(&mut self, message: S) {
        *self = RefreshState::Failed(message.into());
    }
}

impl Default for RefreshState {
    fn default() -> Self {
        RefreshState::Idle
    }
}

/// State of the landscape launcher's main screen.
#[derive(Debug, Clone, Default)]
pub struct MainViewModel {
    /// Tabs shown by the launcher (Dashboard / Instant Play in the original).
    pub tabs: Vec<TabItem>,
    /// Game-circle list returned by `getGameCircleList` / stored via `x(list)`.
    pub game_circles: Vec<GameCircle>,
    /// Currently selected tab index (`n(index, ...)`).
    pub selected_tab: usize,
    /// Whether the last tab selection requested a forced refresh — the
    /// `boolean` half of the pair posted to `r()`.
    pub pending_refresh: bool,
    /// Data-refresh state.
    pub refresh: RefreshState,
    /// Mirrors the `NetworkUtils.k()` value posted to `u()`.
    pub network_available: bool,
}

impl MainViewModel {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_tabs(tabs: Vec<TabItem>) -> Self {
        let mut vm = Self::new();
        vm.set_tabs(tabs);
        vm
    }

    /// Replaces the tab list and clamps the selected index into range.
    pub fn set_tabs(&mut self, tabs: Vec<TabItem>) {
        self.tabs = tabs;
        if !self.tabs.is_empty() && self.selected_tab >= self.tabs.len() {
            self.selected_tab = self.tabs.len() - 1;
        } else if self.tabs.is_empty() {
            self.selected_tab = 0;
        }
    }

    /// Mirrors `n(int, boolean)`: publish a tab selection and return the
    /// `needRefresh` flag so the caller can decide whether to reload.
    pub fn select_tab(&mut self, index: usize, need_refresh: bool) -> bool {
        self.selected_tab = index;
        self.pending_refresh = need_refresh;
        need_refresh
    }

    /// Starts a home-data refresh (the `t()` flag goes true).
    pub fn begin_refresh(&mut self) {
        self.refresh.begin();
    }

    /// Stores the game-circle list (`x(list)`) without changing tab state.
    pub fn set_game_circles(&mut self, list: Vec<GameCircle>) {
        self.game_circles = list;
    }

    /// Completion of `getGameCircleList`: store the list, mark data ready and
    /// apply the locale-dependent initial tab choice.
    pub fn on_circles_loaded(&mut self, list: Vec<GameCircle>, locale_is_zh: bool, app_flag: bool) {
        let count = list.len();
        self.game_circles = list;
        self.refresh.ready();
        let (tab, need_refresh) = initial_tab_for_circles(count, locale_is_zh, app_flag);
        self.select_tab(tab, need_refresh);
    }

    /// Failure of the home-data request.
    pub fn on_circles_failed<S: Into<String>>(&mut self, message: S) {
        self.refresh.fail(message);
    }

    /// Mirrors the network-status listener: `f30446d.m(NetworkUtils.k())`.
    pub fn set_network_available(&mut self, available: bool) {
        self.network_available = available;
    }

    pub fn selected_tab_item(&self) -> Option<&TabItem> {
        self.tabs.get(self.selected_tab)
    }

    pub fn game_circle(&self, id: i32) -> Option<&GameCircle> {
        self.game_circles.iter().find(|c| c.id == id)
    }

    /// Consumes the pending refresh flag, mirroring a one-shot `UnPeekLiveData`.
    pub fn take_pending_refresh(&mut self) -> bool {
        std::mem::replace(&mut self.pending_refresh, false)
    }
}

/// Pure decision from `NewLandscapeLauncherMainVM.AnonymousClass1.invokeSuspend`:
///
/// * empty circle list **and** zh locale **and** no app flag → tab `2`;
/// * every other case → tab `0`;
/// * the posted `needRefresh` flag is `true` in both branches.
pub fn initial_tab_for_circles(
    circle_count: usize,
    locale_is_zh: bool,
    app_flag: bool,
) -> (usize, bool) {
    if circle_count == 0 && locale_is_zh && !app_flag {
        (2, true)
    } else {
        (0, true)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_tabs() -> Vec<TabItem> {
        vec![TabItem::new(0, "Dashboard"), TabItem::new(2, "Instant Play")]
    }

    #[test]
    fn refresh_state_transitions() {
        let mut r = RefreshState::default();
        assert!(matches!(r, RefreshState::Idle));
        r.begin();
        assert!(r.is_loading());
        r.ready();
        assert!(r.is_ready());
        r.fail("offline");
        assert_eq!(r.error(), Some("offline"));
        assert!(!r.is_ready());
    }

    #[test]
    fn tab_selection_tracks_need_refresh_flag() {
        let mut vm = MainViewModel::with_tabs(sample_tabs());
        assert_eq!(vm.selected_tab, 0);
        assert!(vm.select_tab(2, true));
        assert_eq!(vm.selected_tab, 2);
        assert!(vm.pending_refresh);
        assert!(vm.take_pending_refresh());
        assert!(!vm.take_pending_refresh());

        assert!(!vm.select_tab(0, false));
        assert_eq!(vm.selected_tab_item(), Some(&TabItem::new(0, "Dashboard")));
    }

    #[test]
    fn set_tabs_clamps_selection() {
        let mut vm = MainViewModel::with_tabs(sample_tabs());
        vm.select_tab(1, false);
        vm.set_tabs(vec![TabItem::new(0, "Only")]);
        assert_eq!(vm.selected_tab, 0);
        vm.set_tabs(vec![]);
        assert_eq!(vm.selected_tab, 0);
        assert!(vm.selected_tab_item().is_none());
    }

    #[test]
    fn circles_loaded_sets_ready_and_initial_tab() {
        let mut vm = MainViewModel::with_tabs(sample_tabs());
        vm.begin_refresh();
        assert!(vm.refresh.is_loading());

        // Non-empty list always selects tab 0.
        vm.on_circles_loaded(vec![GameCircle::new(1, "Circles")], true, false);
        assert_eq!(vm.game_circles.len(), 1);
        assert!(vm.refresh.is_ready());
        assert_eq!(vm.selected_tab, 0);
        assert!(vm.pending_refresh);
        assert_eq!(vm.game_circle(1).map(|c| c.name.as_str()), Some("Circles"));

        // Empty + zh locale + no app flag selects tab 2.
        vm.on_circles_loaded(vec![], true, false);
        assert_eq!(vm.selected_tab, 2);

        // Empty + non-zh locale selects tab 0.
        vm.on_circles_loaded(vec![], false, false);
        assert_eq!(vm.selected_tab, 0);

        // Empty + zh locale but app flag set selects tab 0.
        vm.on_circles_loaded(vec![], true, true);
        assert_eq!(vm.selected_tab, 0);
    }

    #[test]
    fn failed_load_and_network_status() {
        let mut vm = MainViewModel::new();
        vm.set_network_available(true);
        assert!(vm.network_available);
        vm.on_circles_failed("timeout");
        assert_eq!(vm.refresh.error(), Some("timeout"));
        assert!(!vm.refresh.is_ready());
    }

    #[test]
    fn initial_tab_helper_matches_original_condition() {
        assert_eq!(initial_tab_for_circles(0, true, false), (2, true));
        assert_eq!(initial_tab_for_circles(0, false, false), (0, true));
        assert_eq!(initial_tab_for_circles(0, true, true), (0, true));
        assert_eq!(initial_tab_for_circles(3, true, false), (0, true));
    }
}
