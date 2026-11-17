//! Offline-first data access for the launcher.
//!
//! The launcher normally reads from Room (`t_game_library`), MMKV /
//! SharedPreferences, and the `GameDetailEntity` cache. All three are local
//! sources: none of them require the network to be read or written. This
//! module unifies them behind one small, dependency-free surface — the
//! [`OfflineDataSource`] trait — and supplies a fully in-memory implementation,
//! [`LocalDataSource`], so the launcher can operate with no network, no disk
//! and no Android runtime.
//!
//! Nothing here performs I/O. The types mirror the *shapes* of the crate's
//! existing models:
//!   * `GameLibraryStore` (`com::xj::game::repository`) for the game library map,
//!   * `InMemoryKv` / `Settings` (`com::xj::common::preference`) for settings,
//!   * `GameDetailEntity` (`com::xj::common::data`) for cached details.
//!
//! They are re-declared as minimal, field-compatible types so this area stays
//! self-contained (see `REPORT.md`). Convert to/from the crate's real types at
//! the wiring boundary if desired — the shapes match field-for-field.

use std::collections::HashMap;

// --- Verified string identifiers from GameHub Lite v5.1.8 -------------------
// Kept as named constants so the facade documents the exact resources it
// models. They are inert data here: no Android resource manager is involved.

/// `pc_cc_steam_offline_mode` — user-facing label "Offline Mode".
pub const STR_STEAM_OFFLINE_MODE: &str = "pc_cc_steam_offline_mode";
/// `pc_cc_steam_offline_mode_desc` — "Allow running verified Steam games
/// without network connection".
pub const STR_STEAM_OFFLINE_MODE_DESC: &str = "pc_cc_steam_offline_mode_desc";
/// `comm_network_err` — generic network-error string.
pub const ERR_COMMON_NETWORK: &str = "comm_network_err";
/// `no_network` — "no network connection" string.
pub const ERR_NO_NETWORK: &str = "no_network";
/// `error_pc_offline` — "PC game offline" error string.
pub const ERR_PC_OFFLINE: &str = "error_pc_offline";

/// The user's Steam offline-mode toggle. When [`OfflineMode::Enabled`], verified
/// Steam games may be launched with no network connection (the
/// `pc_cc_steam_offline_mode` / `..._desc` setting).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OfflineMode {
    /// Verified Steam games require a network connection (default).
    Disabled,
    /// Verified Steam games are runnable offline.
    Enabled,
}

impl Default for OfflineMode {
    fn default() -> Self {
        OfflineMode::Disabled
    }
}

impl OfflineMode {
    pub fn is_enabled(self) -> bool {
        self == OfflineMode::Enabled
    }
}

/// A single local game-library row (field-compatible with the crate's
/// `GameLibraryTable` / Room `t_game_library` entity).
#[derive(Debug, Clone, PartialEq)]
pub struct GameLibraryTable {
    pub id: i64,
    pub user_id: String,
    pub launch_type: i32,
    pub package_name: String,
    pub game_id: Option<i32>,
    pub local_game_id: Option<String>,
    pub local_mobile_app_id: Option<String>,
    pub data: String,
    pub time: Option<String>,
    pub is_last_launched: bool,
    pub game_type: i32,
}

impl GameLibraryTable {
    /// A row has a server game id when `game_id` is present and positive.
    pub fn has_server_game(&self) -> bool {
        self.game_id.map(|id| id > 0).unwrap_or(false)
    }
}

/// Minimal snapshot of `GameDetailEntity` (only the fields the offline facade
/// needs to display and key a cached detail). Extra wire fields are ignored at
/// the conversion boundary; the key fields below are authoritative locally.
#[derive(Debug, Clone, PartialEq)]
pub struct GameDetailEntity {
    pub id: i32,
    pub name: String,
    pub company: String,
    pub cover_image: String,
    pub description: String,
    pub local_game_id: String,
    pub launch_type: i32,
    pub game_type: i32,
}

impl GameDetailEntity {
    /// `getValidId()`: prefer a positive server id, then local id.
    pub fn valid_id(&self) -> String {
        if self.id > 0 {
            self.id.to_string()
        } else {
            self.local_game_id.clone()
        }
    }
}

/// Portable snapshot of the app's persisted scalar settings (field-compatible
/// with `preference::Settings`).
#[derive(Debug, Clone, PartialEq)]
pub struct Settings {
    pub device_unique_id: String,
    pub language: String,
    pub first_install_time: i64,
    pub last_scroll_position_by_my_games: i64,
    pub last_visible_explore_time: i64,
    pub is_first_show_play_in_seconds_tab: bool,
    pub wine_screen_brightness_switch: bool,
    pub wine_screen_brightness: f64,
    pub um_init: i64,
    pub intro_video_played: bool,
    pub gif_loaded: bool,
    /// Steam offline-mode toggle (`pc_cc_steam_offline_mode`): verified Steam
    /// games runnable without network when [`OfflineMode::Enabled`].
    pub steam_offline_mode: OfflineMode,
}

impl Default for Settings {
    fn default() -> Self {
        Settings {
            device_unique_id: String::new(),
            language: String::new(),
            first_install_time: -1,
            last_scroll_position_by_my_games: -1,
            last_visible_explore_time: -1,
            is_first_show_play_in_seconds_tab: true,
            wine_screen_brightness_switch: false,
            wine_screen_brightness: -1.0,
            um_init: 0,
            intro_video_played: false,
            gif_loaded: false,
            steam_offline_mode: OfflineMode::Disabled,
        }
    }
}

impl Settings {
    /// Convenience accessor mirroring the `pc_cc_steam_offline_mode` toggle.
    pub fn steam_offline_mode_enabled(&self) -> bool {
        self.steam_offline_mode.is_enabled()
    }
}

/// The local-only data surface the launcher needs to render and mutate state
/// while offline. All operations touch in-memory local sources only; none of
/// them reach the network.
pub trait OfflineDataSource {
    /// All locally known games, ordered by row id for deterministic output.
    fn list_games(&self) -> Vec<GameLibraryTable>;

    /// The locally cached detail for `key` (a server id or `localGameId`).
    fn get_game_detail(&self, key: &str) -> Option<GameDetailEntity>;

    /// Cache/replace a game detail under `key`.
    fn put_game_detail(&mut self, key: String, detail: GameDetailEntity);

    /// Current settings snapshot.
    fn get_settings(&self) -> Settings;

    /// Persist a settings snapshot (in memory here).
    fn save_settings(&mut self, settings: Settings);
}

/// In-memory composition of the three local sources:
///   * a `GameLibraryStore`-like `HashMap<i64, GameLibraryTable>`,
///   * a `Settings` value standing in for the KV store,
///   * a `HashMap<String, GameDetailEntity>` game-detail cache.
///
/// `settings_loaded` records whether `get_settings` has been called at least
/// once (mirroring a real store being read into memory). It is exposed to
/// [`crate::report::OfflineReport`] via [`LocalDataSource::settings_loaded`].
#[derive(Debug, Clone, Default)]
pub struct LocalDataSource {
    games: HashMap<i64, GameLibraryTable>,
    details: HashMap<String, GameDetailEntity>,
    settings: Settings,
    settings_loaded: bool,
}

impl LocalDataSource {
    pub fn new() -> Self {
        Self::default()
    }

    /// Insert or replace a game row by id (Room `insertOrUpdate`).
    pub fn upsert_game(&mut self, row: GameLibraryTable) {
        self.games.insert(row.id, row);
    }

    /// Remove a game row by id. Returns the removed row, if any.
    pub fn remove_game(&mut self, id: i64) -> Option<GameLibraryTable> {
        self.games.remove(&id)
    }

    /// Borrow a single game row by id.
    pub fn game(&self, id: i64) -> Option<&GameLibraryTable> {
        self.games.get(&id)
    }

    /// Number of cached game details.
    pub fn cached_detail_count(&self) -> usize {
        self.details.len()
    }

    /// Whether settings have been read into memory yet.
    pub fn settings_loaded(&self) -> bool {
        self.settings_loaded
    }
}

impl OfflineDataSource for LocalDataSource {
    fn list_games(&self) -> Vec<GameLibraryTable> {
        let mut rows: Vec<_> = self.games.values().cloned().collect();
        rows.sort_by_key(|r| r.id);
        rows
    }

    fn get_game_detail(&self, key: &str) -> Option<GameDetailEntity> {
        self.details.get(key).cloned()
    }

    fn put_game_detail(&mut self, key: String, detail: GameDetailEntity) {
        self.details.insert(key, detail);
    }

    fn get_settings(&self) -> Settings {
        // Note: `settings_loaded` is observable only through a &self borrow, so
        // the "loaded" flag is set by `load_settings` below. `get_settings`
        // returns the current snapshot regardless.
        self.settings.clone()
    }

    fn save_settings(&mut self, settings: Settings) {
        self.settings = settings;
        self.settings_loaded = true;
    }
}

impl LocalDataSource {
    /// Explicitly read settings into memory, marking the local settings source
    /// as loaded (used by `OfflineReport::settings_loaded`).
    pub fn load_settings(&mut self) -> Settings {
        self.settings_loaded = true;
        self.settings.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn row(id: i64, name: &str) -> GameLibraryTable {
        GameLibraryTable {
            id,
            user_id: String::new(),
            launch_type: 1403,
            package_name: name.to_owned(),
            game_id: Some(id as i32),
            local_game_id: Some(format!("local_{id}")),
            local_mobile_app_id: None,
            data: String::new(),
            time: None,
            is_last_launched: false,
            game_type: 2,
        }
    }

    fn detail(id: i32) -> GameDetailEntity {
        GameDetailEntity {
            id,
            name: "Hollow Knight".into(),
            company: "Team Cherry".into(),
            cover_image: "cover.webp".into(),
            description: "desc".into(),
            local_game_id: String::new(),
            launch_type: 1403,
            game_type: 2,
        }
    }

    #[test]
    fn local_game_crud() {
        let mut ds = LocalDataSource::new();
        assert!(ds.list_games().is_empty());
        ds.upsert_game(row(2, "b"));
        ds.upsert_game(row(1, "a"));
        // Sorted by id, not insertion order.
        assert_eq!(ds.list_games()[0].id, 1);
        assert_eq!(ds.list_games()[1].id, 2);
        // Upsert replaces by id.
        ds.upsert_game(row(1, "a-v2"));
        assert_eq!(ds.list_games().len(), 2);
        assert_eq!(ds.game(1).unwrap().package_name, "a-v2");
        // Remove.
        assert!(ds.remove_game(2).is_some());
        assert!(ds.remove_game(2).is_none());
        assert_eq!(ds.list_games().len(), 1);
    }

    #[test]
    fn detail_and_settings_crud() {
        let mut ds = LocalDataSource::new();
        assert!(ds.get_game_detail("481516").is_none());
        ds.put_game_detail("481516".into(), detail(481516));
        assert_eq!(ds.get_game_detail("481516").unwrap().name, "Hollow Knight");
        assert_eq!(ds.cached_detail_count(), 1);

        // Settings default / save / reload.
        assert!(!ds.settings_loaded());
        assert_eq!(ds.get_settings(), Settings::default());
        let mut s = Settings::default();
        s.language = "zh$CN".into();
        s.um_init = 1;
        s.steam_offline_mode = OfflineMode::Enabled;
        ds.save_settings(s);
        assert!(ds.settings_loaded());
        assert_eq!(ds.get_settings().language, "zh$CN");
        assert_eq!(ds.get_settings().um_init, 1);
        assert!(ds.get_settings().steam_offline_mode_enabled());
        assert_eq!(STR_STEAM_OFFLINE_MODE, "pc_cc_steam_offline_mode");
        assert_eq!(ERR_NO_NETWORK, "no_network");
    }
}
