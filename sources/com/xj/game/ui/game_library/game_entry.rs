//! Minimal, self-contained game entry plus pure sort/filter helpers.
//!
//! Models the portable shape of a row in the game library grid/list
//! (`GamesFragment` / `GamesAdapter` / `MyGamesItemData` in GameHub Lite v5.1.8).
//! No Android types, no database, no I/O — just the `(title, time, game_type)`
//! triple the UI sorts and filters by.
//!
//! [`GameType`] codes mirror the decompiled
//! `com.xj.common.data.model.GameType`:
//! `MOBILE_INSTALL_APP(1)`, `PC_STEAM_IMPORT(2)`. `Imported` (3) is a local
//! modelling extension for games added from a file rather than synced from an
//! account, so that the `Imported` library filter has a matching category.

use std::cmp::Ordering;

/// Category of a game row. Codes match the decompiled `GameType` enum where
/// one exists; `Imported` is a modelling-only extension.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum GameType {
    /// `MOBILE_INSTALL_APP` = 1 — installed mobile/APK game.
    Mobile = 1,
    /// `PC_STEAM_IMPORT` = 2 — PC / Steam-imported game.
    Pc = 2,
    /// Modelling extension = 3 — locally imported game file (exe/txt/APK).
    Imported = 3,
}

impl GameType {
    /// All variants, decompiled-enum `values()` style.
    pub const ALL: [Self; 3] = [Self::Mobile, Self::Pc, Self::Imported];

    /// Numeric code (`getValue()` in the Kotlin enum).
    pub fn code(self) -> i32 {
        self as i32
    }

    /// Look up by numeric code; unknown codes return `None`.
    pub fn from_code(code: i32) -> Option<Self> {
        Self::ALL.iter().copied().find(|t| t.code() == code)
    }
}

/// A single library row. Deliberately tiny: only the fields the portable
/// sort/filter logic needs.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GameEntry {
    /// Stable identifier (maps to `GameDetailEntity.id` / `localGameId`).
    pub id: String,
    /// Display title (maps to `GameDetailEntity.name`).
    pub title: String,
    /// Category (maps to `GameDetailEntity.gameType`).
    pub game_type: GameType,
    /// Timestamp used by the "time" sort. Larger = more recent.
    pub time: u64,
}

impl GameEntry {
    pub fn new(id: impl Into<String>, title: impl Into<String>, game_type: GameType, time: u64) -> Self {
        Self {
            id: id.into(),
            title: title.into(),
            game_type,
            time,
        }
    }

    /// Case-insensitive substring match against the title.
    pub fn title_contains(&self, query: &str) -> bool {
        if query.trim().is_empty() {
            return true;
        }
        self.title.to_lowercase().contains(&query.trim().to_lowercase())
    }
}

/// Order used by the "sort" control of the library.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SortOrder {
    /// Alphabetical by title (A→Z, case-insensitive).
    Name,
    /// Most recently used/added first (descending `time`).
    Time,
    /// Grouped by [`GameType`], then alphabetical by title.
    Type,
}

impl Default for SortOrder {
    fn default() -> Self {
        Self::Name
    }
}

/// Category tab / filter of the library.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum LibraryFilter {
    /// No category restriction.
    All,
    /// Only [`GameType::Pc`].
    Pc,
    /// Only [`GameType::Mobile`].
    Mobile,
    /// Only [`GameType::Imported`].
    Imported,
}

impl Default for LibraryFilter {
    fn default() -> Self {
        Self::All
    }
}

impl LibraryFilter {
    /// Whether `game_type` passes this filter.
    pub fn matches(self, game_type: GameType) -> bool {
        match self {
            Self::All => true,
            Self::Pc => game_type == GameType::Pc,
            Self::Mobile => game_type == GameType::Mobile,
            Self::Imported => game_type == GameType::Imported,
        }
    }
}

/// Sort the given slice into a fresh `Vec` by `order`. Pure: the input is
/// never mutated. Ties are broken deterministically by `time` then `id` so
/// the result is stable and independent of the input order.
pub fn sort(games: &[GameEntry], order: SortOrder) -> Vec<GameEntry> {
    let mut out = games.to_vec();
    out.sort_by(|a, b| compare_entries(a, b, order));
    out
}

fn compare_entries(a: &GameEntry, b: &GameEntry, order: SortOrder) -> Ordering {
    let primary = match order {
        SortOrder::Name => a.title.to_lowercase().cmp(&b.title.to_lowercase()),
        // Newest first: reverse the natural u64 order.
        SortOrder::Time => b.time.cmp(&a.time),
        SortOrder::Type => a.game_type.code().cmp(&b.game_type.code()),
    };
    primary
        .then_with(|| a.title.to_lowercase().cmp(&b.title.to_lowercase()))
        .then_with(|| b.time.cmp(&a.time))
        .then_with(|| a.id.cmp(&b.id))
}

/// Filter the given slice into a fresh `Vec` by category and search query.
/// Both criteria are ANDed. An empty/whitespace query matches all titles.
/// Pure: the input is never mutated.
pub fn filter(games: &[GameEntry], filter: LibraryFilter, query: &str) -> Vec<GameEntry> {
    games
        .iter()
        .filter(|g| filter.matches(g.game_type) && g.title_contains(query))
        .cloned()
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample() -> Vec<GameEntry> {
        vec![
            GameEntry::new("1", "Zelda", GameType::Pc, 300),
            GameEntry::new("2", "alpha", GameType::Mobile, 100),
            GameEntry::new("3", "Beta", GameType::Imported, 200),
            GameEntry::new("4", "beta two", GameType::Pc, 200),
        ]
    }

    #[test]
    fn game_type_codes_round_trip() {
        for t in GameType::ALL {
            assert_eq!(GameType::from_code(t.code()), Some(t));
        }
        assert_eq!(GameType::from_code(0), None);
        assert_eq!(GameType::Mobile.code(), 1);
        assert_eq!(GameType::Pc.code(), 2);
        assert_eq!(GameType::Imported.code(), 3);
    }

    #[test]
    fn sort_by_name_is_case_insensitive() {
        let sorted = sort(&sample(), SortOrder::Name);
        let titles: Vec<&str> = sorted.iter().map(|g| g.title.as_str()).collect();
        assert_eq!(titles, vec!["alpha", "Beta", "beta two", "Zelda"]);
    }

    #[test]
    fn sort_by_time_is_newest_first() {
        let sorted = sort(&sample(), SortOrder::Time);
        let ids: Vec<&str> = sorted.iter().map(|g| g.id.as_str()).collect();
        // 300 first; the two 200s tie and break by title then id ("3" < "4").
        assert_eq!(ids, vec!["1", "3", "4", "2"]);
    }

    #[test]
    fn sort_by_type_groups_then_alphabetizes() {
        let sorted = sort(&sample(), SortOrder::Type);
        let types: Vec<GameType> = sorted.iter().map(|g| g.game_type).collect();
        assert_eq!(types, vec![GameType::Mobile, GameType::Pc, GameType::Pc, GameType::Imported]);
        // Within the Pc group: "beta two" before "Zelda".
        assert_eq!(sorted[1].title, "beta two");
        assert_eq!(sorted[2].title, "Zelda");
    }

    #[test]
    fn sort_does_not_mutate_input() {
        let games = sample();
        let original: Vec<String> = games.iter().map(|g| g.id.clone()).collect();
        let _ = sort(&games, SortOrder::Time);
        let after: Vec<String> = games.iter().map(|g| g.id.clone()).collect();
        assert_eq!(original, after);
    }

    #[test]
    fn filter_by_category() {
        assert_eq!(filter(&sample(), LibraryFilter::All, "").len(), 4);
        assert_eq!(filter(&sample(), LibraryFilter::Pc, "").len(), 2);
        assert_eq!(filter(&sample(), LibraryFilter::Mobile, "").len(), 1);
        assert_eq!(filter(&sample(), LibraryFilter::Imported, "").len(), 1);
    }

    #[test]
    fn filter_by_query_is_case_insensitive() {
        let matched = filter(&sample(), LibraryFilter::All, "beta");
        let ids: Vec<&str> = matched.iter().map(|g| g.id.as_str()).collect();
        assert_eq!(ids, vec!["3", "4"]);
        assert!(filter(&sample(), LibraryFilter::All, "   ").len() == 4);
    }

    #[test]
    fn filter_combines_category_and_query() {
        let matched = filter(&sample(), LibraryFilter::Pc, "zel");
        let ids: Vec<&str> = matched.iter().map(|g| g.id.as_str()).collect();
        assert_eq!(ids, vec!["1"]);
    }
}
