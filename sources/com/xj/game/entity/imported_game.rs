//! Portable mirror of `com.xj.game.entity.ImportedGameEntity`
//! (GameHub Lite v5.1.8, read-only decompiled source:
//! `sources/com/xj/game/entity/ImportedGameEntity.java`).
//!
//! The Java data class declares exactly these eleven fields (constructor
//! order: id, localGameId, name, filePath, cover, description, size,
//! exeFileBgType, steamGameInfo, steamAppId, gameType). They are listed below
//! in the same order used by the task/REPORT mapping. Android `Parcelable`,
//! `@Parcelize` and Gson plumbing is intentionally omitted.

use super::{GameType, SteamGameInfo};

#[derive(Debug, Clone, PartialEq)]
pub struct ImportedGame {
    /// `@NotNull String cover`
    pub cover: String,
    /// `@NotNull String description`
    pub description: String,
    /// `int exeFileBgType`
    pub exe_file_bg_type: i32,
    /// `@NotNull String filePath`
    pub file_path: String,
    /// `int gameType` — one of the `GameType` codes (0/1/2)
    pub game_type: i32,
    /// `@NotNull String id` — parsed to an int by the Java repository
    pub id: String,
    /// `@NotNull String localGameId`
    pub local_game_id: String,
    /// `@NotNull String name`
    pub name: String,
    /// `long size`
    pub size: i64,
    /// `@Nullable String steamAppId`
    pub steam_app_id: Option<String>,
    /// `@Nullable SteamGameInfo steamGameInfo`
    pub steam_game_info: Option<SteamGameInfo>,
}

impl ImportedGame {
    /// Construct in the same parameter order as the decompiled Java
    /// constructor, for faithful call sites.
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        id: String,
        local_game_id: String,
        name: String,
        file_path: String,
        cover: String,
        description: String,
        size: i64,
        exe_file_bg_type: i32,
        steam_game_info: Option<SteamGameInfo>,
        steam_app_id: Option<String>,
        game_type: i32,
    ) -> Self {
        Self {
            cover,
            description,
            exe_file_bg_type,
            file_path,
            game_type,
            id,
            local_game_id,
            name,
            size,
            steam_app_id,
            steam_game_info,
        }
    }

    /// Parse `id` the way `addImportGame` does:
    /// `try { Integer.parseInt(id) } catch { 0 }`.
    pub fn parsed_id(&self) -> i32 {
        self.id.parse::<i32>().unwrap_or(0)
    }

    /// Interpret the raw `game_type` code as a known `GameType`.
    pub fn game_type_enum(&self) -> Option<GameType> {
        GameType::from_code(self.game_type)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample() -> ImportedGame {
        ImportedGame::new(
            "42".into(),
            "local_abcd".into(),
            "My Game".into(),
            "/games/my/game.exe".into(),
            "/cover.png".into(),
            "desc".into(),
            12_345,
            3,
            Some(SteamGameInfo::new(42)),
            Some("42".into()),
            2,
        )
    }

    #[test]
    fn holds_all_java_fields() {
        let g = sample();
        assert_eq!(g.cover, "/cover.png");
        assert_eq!(g.description, "desc");
        assert_eq!(g.exe_file_bg_type, 3);
        assert_eq!(g.file_path, "/games/my/game.exe");
        assert_eq!(g.game_type, 2);
        assert_eq!(g.id, "42");
        assert_eq!(g.local_game_id, "local_abcd");
        assert_eq!(g.name, "My Game");
        assert_eq!(g.size, 12_345);
        assert_eq!(g.steam_app_id.as_deref(), Some("42"));
        assert_eq!(g.steam_game_info.unwrap().steam_app_id, 42);
    }

    #[test]
    fn parses_id_like_java_and_reads_game_type() {
        let mut g = sample();
        assert_eq!(g.parsed_id(), 42);
        assert_eq!(g.game_type_enum(), Some(GameType::PcSteamImport));
        g.id = "not-a-number".into();
        assert_eq!(g.parsed_id(), 0); // Integer.parseInt failure -> 0
        g.game_type = 99;
        assert_eq!(g.game_type_enum(), None);
    }
}
