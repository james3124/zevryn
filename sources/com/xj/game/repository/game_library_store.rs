//! In-memory model of the DAO-backed game library
//! (`com.xj.game.repository.GameLibraryRepository` / `GameLibraryDao`).
//!
//! The Java repository talks to Room (`GameSirUxDB.gameLibraryDao()`) using
//! `insertOrUpdate`, `_insertIfNotExists`, `findByPkgAndLaunchType`,
//! `findByGameIdOrLocalGameId` and `updateGameDataById`. Here those operations
//! are represented with a plain `HashMap<i64, GameLibraryTable>` so the area
//! is std-only and compiles standalone. No SQL is executed.

#[path = "../entity/mod.rs"]
pub mod entity_model;

use std::collections::HashMap;

use entity_model::{GameLibraryTable, ImportedGame};

/// Outcome of importing/adding a game to the library.
#[derive(Debug, Clone, PartialEq)]
pub enum ImportResult {
    /// Row was inserted or updated (mirrors a successful `insertOrUpdate`).
    Success(GameLibraryTable),
    /// A row with the same package/launch type already exists and would not be
    /// replaced (mirrors the `replace == false` branch of `addPcGame`,
    /// "已经存在该游戏，且不替换").
    AlreadyExists,
    /// The imported game lacked fields the store needs to key/identify it.
    Invalid,
}

/// An in-memory stand-in for the Room-backed `t_game_library` table.
#[derive(Debug, Clone, Default)]
pub struct GameLibraryStore {
    rows: HashMap<i64, GameLibraryTable>,
}

impl GameLibraryStore {
    pub fn new() -> Self {
        Self::default()
    }

    /// Mirror of `addImportGame` with its default `replace = true` flag:
    /// convert the imported game and **insert or update by id** (Room's
    /// `insertOrUpdate`). Always returns `Success` for a valid game.
    pub fn add(&mut self, game: ImportedGame) -> ImportResult {
        self.add_import(game, true)
    }

    /// Full mirror of `addImportGame` / `addPcGame`.
    ///
    /// `replace` is the boolean argument (`z3`) of the decompiled method:
    /// - `true` → `insertOrUpdate` by id (replace an existing row).
    /// - `false` → check `findByPkgAndLaunchType`; if such a row exists,
    ///   return `AlreadyExists` without replacing; otherwise
    ///   `_insertIfNotExists`.
    pub fn add_import(&mut self, game: ImportedGame, replace: bool) -> ImportResult {
        // Portable validation guard (Room would accept empty strings, but a
        // row without an id or an exe path is not identifiable in this store).
        if game.id.is_empty() || game.file_path.is_empty() {
            return ImportResult::Invalid;
        }

        let row = GameLibraryTable::from(game);

        if !replace
            && self
                .find_by_pkg_and_launch_type(&row.package_name, row.launch_type)
                .is_some()
        {
            return ImportResult::AlreadyExists;
        }

        let id = row.id;
        self.rows.insert(id, row.clone());
        ImportResult::Success(row)
    }

    /// Update an existing row by id (mirrors `updateGameDataById` at the row
    /// level). Returns `true` when a row with that id existed and was replaced.
    pub fn update(&mut self, row: GameLibraryTable) -> bool {
        match self.rows.get_mut(&row.id) {
            Some(slot) => {
                *slot = row;
                true
            }
            None => false,
        }
    }

    /// Remove and return a row by id (mirrors a delete-by-id DAO helper).
    pub fn remove(&mut self, id: i64) -> Option<GameLibraryTable> {
        self.rows.remove(&id)
    }

    pub fn get(&self, id: i64) -> Option<&GameLibraryTable> {
        self.rows.get(&id)
    }

    /// All rows, sorted by id for deterministic iteration over the HashMap.
    pub fn all(&self) -> Vec<&GameLibraryTable> {
        let mut rows: Vec<_> = self.rows.values().collect();
        rows.sort_by_key(|r| r.id);
        rows
    }

    /// Mirrors `findByLocalGameId`.
    pub fn find_by_local_id(&self, local_id: &str) -> Option<&GameLibraryTable> {
        self.rows
            .values()
            .find(|r| r.local_game_id.as_deref() == Some(local_id))
    }

    /// Mirrors `findByGameId`.
    pub fn find_by_game_id(&self, game_id: i32) -> Option<&GameLibraryTable> {
        self.rows.values().find(|r| r.game_id == Some(game_id))
    }

    /// Mirrors `findByPkgAndLaunchType(packageName, launchType)`.
    pub fn find_by_pkg_and_launch_type(
        &self,
        package_name: &str,
        launch_type: i32,
    ) -> Option<&GameLibraryTable> {
        self.rows
            .values()
            .find(|r| r.package_name == package_name && r.launch_type == launch_type)
    }

    pub fn len(&self) -> usize {
        self.rows.len()
    }

    pub fn is_empty(&self) -> bool {
        self.rows.is_empty()
    }
}

/// `GameLibraryRepository.Companion.a(String)`: `str != null && str.startsWith("local_")`.
pub fn is_local_game_id(id: &str) -> bool {
    id.starts_with("local_")
}

/// Mirror of `GameLibraryRepository.A(GameDetailEntity)`: a PC game counts as
/// installed when its server game id is in `installed_ids` (the external
/// `GameStateMgr`) **or** its local id carries the `local_` prefix.
pub fn is_pc_game_installed(local_game_id: &str, game_id: i32, installed_ids: &[i32]) -> bool {
    installed_ids.contains(&game_id) || is_local_game_id(local_game_id)
}

#[cfg(test)]
mod tests {
    use super::*;
    use entity_model::{LAUNCH_TYPE_PC_EMULATOR, SteamGameInfo};

    fn game(id: &str, local: &str, path: &str, game_type: i32) -> ImportedGame {
        ImportedGame::new(
            id.into(),
            local.into(),
            format!("Game {id}"),
            path.into(),
            "cover".into(),
            "desc".into(),
            100,
            0,
            Some(SteamGameInfo::new(id.parse::<i64>().unwrap_or(0))),
            Some(id.into()),
            game_type,
        )
    }

    #[test]
    fn add_inserts_and_maps_row() {
        let mut store = GameLibraryStore::new();
        match store.add(game("11", "local_a", "/exe/a.exe", 2)) {
            ImportResult::Success(row) => {
                assert_eq!(row.id, 11);
                assert_eq!(row.launch_type, LAUNCH_TYPE_PC_EMULATOR);
                assert_eq!(row.local_game_id.as_deref(), Some("local_a"));
            }
            other => panic!("expected Success, got {:?}", other),
        }
        assert_eq!(store.len(), 1);
        assert!(store.get(11).is_some());
    }

    #[test]
    fn add_is_insert_or_update_by_id() {
        let mut store = GameLibraryStore::new();
        assert!(matches!(
            store.add(game("11", "local_a", "/exe/a.exe", 0)),
            ImportResult::Success(_)
        ));
        // Same id, new payload -> update, not a second row.
        let result = store.add(game("11", "local_a", "/exe/a_v2.exe", 1));
        match result {
            ImportResult::Success(row) => {
                assert_eq!(row.package_name, "/exe/a_v2.exe");
                assert_eq!(row.game_type, 1);
            }
            other => panic!("expected Success, got {:?}", other),
        }
        assert_eq!(store.len(), 1);
        assert_eq!(store.get(11).unwrap().package_name, "/exe/a_v2.exe");
    }

    #[test]
    fn add_import_dedupes_when_not_replacing() {
        let mut store = GameLibraryStore::new();
        // First insert with replace = false succeeds.
        assert!(matches!(
            store.add_import(game("11", "local_a", "/exe/same.exe", 0), false),
            ImportResult::Success(_)
        ));
        // Same package + launch type, different id -> AlreadyExists.
        let dup = store.add_import(game("22", "local_b", "/exe/same.exe", 0), false);
        assert_eq!(dup, ImportResult::AlreadyExists);
        assert_eq!(store.len(), 1);

        // With replace = true the by-id path ignores the pkg dedupe.
        let replaced = store.add_import(game("22", "local_b", "/exe/same.exe", 0), true);
        assert!(matches!(replaced, ImportResult::Success(_)));
        assert_eq!(store.len(), 2);
    }

    #[test]
    fn invalid_game_is_rejected() {
        let mut store = GameLibraryStore::new();
        assert_eq!(
            store.add(game("", "local_a", "/exe/a.exe", 0)),
            ImportResult::Invalid
        );
        assert_eq!(
            store.add(game("11", "local_a", "", 0)),
            ImportResult::Invalid
        );
        assert!(store.is_empty());
    }

    #[test]
    fn crud_and_finders() {
        let mut store = GameLibraryStore::new();
        let _ = store.add(game("11", "local_a", "/exe/a.exe", 0));
        let _ = store.add(game("22", "local_b", "/exe/b.exe", 2));
        let _ = store.add_import(game("33", "steam_c", "/exe/c.exe", 2), true);

        assert_eq!(store.len(), 3);
        assert_eq!(store.all().len(), 3);
        // Sorted by id.
        assert_eq!(store.all()[0].id, 11);
        assert_eq!(store.all()[2].id, 33);

        assert_eq!(store.find_by_local_id("local_b").unwrap().id, 22);
        assert!(store.find_by_local_id("missing").is_none());
        assert_eq!(store.find_by_game_id(33).unwrap().id, 33);
        assert_eq!(
            store
                .find_by_pkg_and_launch_type("/exe/a.exe", LAUNCH_TYPE_PC_EMULATOR)
                .unwrap()
                .id,
            11
        );

        // update requires an existing id.
        let mut row = store.get(11).unwrap().clone();
        row.package_name = "/exe/renamed.exe".into();
        assert!(store.update(row));
        assert_eq!(store.get(11).unwrap().package_name, "/exe/renamed.exe");
        let absent = GameLibraryTable {
            id: 999,
            ..store.get(11).unwrap().clone()
        };
        assert!(!store.update(absent));

        assert!(store.remove(22).is_some());
        assert!(store.get(22).is_none());
        assert_eq!(store.len(), 2);
    }

    #[test]
    fn is_pc_game_installed_matches_java_conditions() {
        assert!(is_local_game_id("local_123"));
        assert!(!is_local_game_id("steam_local123"));
        assert!(is_pc_game_installed("local_x", -1, &[]));
        assert!(is_pc_game_installed("other", 42, &[7, 42]));
        assert!(!is_pc_game_installed("other", -1, &[7, 42]));
    }
}
