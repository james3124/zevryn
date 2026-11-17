//! Portable, framework-free UI state for the game library screen.
//!
//! Mirrors the *state* held by `GamesFragment` (`page_type` tab index,
//! grid adapter), `GameLibraryActivity` (`textTabLayout` tabs) and the
//! `MyFragment` dashboard, without any Android `Fragment`/`Adapter`/`View`
//! types. The sub-views themselves stay on the Android side; this module only
//! models what can be reasoned about in plain Rust.
//!
//! NOTE on the "tab" field: the decompiled `GamesFragment` uses a numeric
//! `page_type` integer (1 = PC games tab) rather than a named enum. Here it
//! is kept as `tab_index: usize` to stay faithful to that representation.

use super::game_entry::{filter, sort, GameEntry, LibraryFilter, SortOrder};

/// Grid vs. list presentation of the library rows.
///
/// The decompiled `GamesFragment` always sets a 3-column
/// `FocusGridLayoutManager`; `ListView` is a modelling extension representing
/// a single-column presentation so the portable UI can switch view modes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ViewMode {
    Grid,
    List,
}

impl Default for ViewMode {
    fn default() -> Self {
        Self::Grid
    }
}

/// Complete portable state of the game-library screen.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GameLibraryUiState {
    /// Active tab index (`page_type` in `GamesFragment`, default 1 = PC games).
    pub tab_index: usize,
    /// Grid/list presentation.
    pub view_mode: ViewMode,
    /// Active sort order.
    pub sort_order: SortOrder,
    /// Active category filter.
    pub library_filter: LibraryFilter,
    /// Current search query (free text, matched against game titles).
    pub search_query: String,
    /// Currently selected game id, if any.
    pub selected_game_id: Option<String>,
}

impl Default for GameLibraryUiState {
    fn default() -> Self {
        Self {
            tab_index: 1,
            view_mode: ViewMode::Grid,
            sort_order: SortOrder::default(),
            library_filter: LibraryFilter::default(),
            search_query: String::new(),
            selected_game_id: None,
        }
    }
}

impl GameLibraryUiState {
    pub fn new(tab_index: usize) -> Self {
        Self {
            tab_index,
            ..Self::default()
        }
    }

    /// Apply this state's filter and sort to `games`, producing the rows the
    /// view would render. Filter first, then sort. Pure.
    pub fn visible_games(&self, games: &[GameEntry]) -> Vec<GameEntry> {
        let filtered = filter(games, self.library_filter, &self.search_query);
        sort(&filtered, self.sort_order)
    }

    /// Select a game by id.
    pub fn select(&mut self, id: impl Into<String>) {
        self.selected_game_id = Some(id.into());
    }

    /// Clear the current selection (e.g. on tab/filter change).
    pub fn clear_selection(&mut self) {
        self.selected_game_id = None;
    }

    /// True if `id` is the selected game.
    pub fn is_selected(&self, id: &str) -> bool {
        self.selected_game_id.as_deref() == Some(id)
    }

    /// Set the sort order.
    pub fn set_sort_order(&mut self, order: SortOrder) {
        self.sort_order = order;
    }

    /// Set the category filter and clear the selection (rows may disappear).
    pub fn set_filter(&mut self, filter: LibraryFilter) {
        self.library_filter = filter;
        self.clear_selection();
    }

    /// Set the search query and clear the selection (rows may disappear).
    pub fn set_search_query(&mut self, query: impl Into<String>) {
        self.search_query = query.into();
        self.clear_selection();
    }

    /// Switch tab and clear the selection, matching `GameLibraryActivity`'s
    /// page switching behavior.
    pub fn set_tab(&mut self, tab_index: usize) {
        self.tab_index = tab_index;
        self.clear_selection();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::super::game_entry::GameType;

    fn games() -> Vec<GameEntry> {
        vec![
            GameEntry::new("1", "Zelda", GameType::Pc, 300),
            GameEntry::new("2", "alpha", GameType::Mobile, 100),
            GameEntry::new("3", "Beta", GameType::Imported, 200),
        ]
    }

    #[test]
    fn default_state() {
        let s = GameLibraryUiState::default();
        assert_eq!(s.tab_index, 1);
        assert_eq!(s.view_mode, ViewMode::Grid);
        assert_eq!(s.sort_order, SortOrder::Name);
        assert_eq!(s.library_filter, LibraryFilter::All);
        assert!(s.search_query.is_empty());
        assert_eq!(s.selected_game_id, None);
    }

    #[test]
    fn visible_games_filters_then_sorts() {
        let mut s = GameLibraryUiState::default();
        s.set_filter(LibraryFilter::Pc);
        s.set_sort_order(SortOrder::Time);
        let visible = s.visible_games(&games());
        let ids: Vec<&str> = visible.iter().map(|g| g.id.as_str()).collect();
        assert_eq!(ids, vec!["1"]);
    }

    #[test]
    fn visible_games_applies_search_query() {
        let mut s = GameLibraryUiState::default();
        s.set_search_query("bet");
        let visible = s.visible_games(&games());
        let ids: Vec<&str> = visible.iter().map(|g| g.id.as_str()).collect();
        assert_eq!(ids, vec!["3"]);
    }

    #[test]
    fn selection_lifecycle() {
        let mut s = GameLibraryUiState::default();
        s.select("2");
        assert!(s.is_selected("2"));
        assert!(!s.is_selected("1"));
        s.set_tab(0);
        assert_eq!(s.selected_game_id, None);
        s.select("1");
        s.set_search_query("alpha");
        assert_eq!(s.selected_game_id, None);
    }
}
