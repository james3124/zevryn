//! Conversion `ImportedGame -> GameLibraryTable` (the storage entity).
//!
//! Mirrors `GameLibraryRepository.addImportGame` / `addPcGame`: the Java code
//! first wraps the imported game in a `GameDetailEntity` (Gson-serialized into
//! the `data` column) and then inserts a `GameLibraryTable` row. See
//! REPORT.md for the full field-by-field mapping table and source citations.

use super::{GameLibraryTable, ImportedGame};

/// `LauncherTypeEnum.START_TYPE_PC_EMULATOR` (from the decompiled
/// `com/xj/common/data/launch/LauncherTypeEnum.java`), used as `launch_type`
/// by `addPcGame`.
pub const LAUNCH_TYPE_PC_EMULATOR: i32 = 1403;

impl From<ImportedGame> for GameLibraryTable {
    fn from(game: ImportedGame) -> Self {
        // Java: Integer.parseInt(id) with a catch-all fallback of 0.
        let parsed_id = game.parsed_id();

        // Java: GsonConverter.toJson(gameDetailEntity) -> `data`. We emit a
        // portable JSON of the ImportedGame fields instead (stdlib only).
        let data = imported_game_json(&game);

        Self {
            id: i64::from(parsed_id),
            // The real repository fills this with the current user's uid;
            // no user context is available in this self-contained model.
            user_id: String::new(),
            launch_type: LAUNCH_TYPE_PC_EMULATOR,
            // addPcGame stores the exe path in the `package_name` column.
            package_name: game.file_path.clone(),
            // Java passes a non-null Integer parsed from the imported id.
            game_id: Some(parsed_id),
            // addImportGame copies the ImportedGame.localGameId verbatim.
            local_game_id: Some(game.local_game_id.clone()),
            // The constructed GameDetailEntity has localMobileAppId = null.
            local_mobile_app_id: None,
            data,
            // Java stamps new Date(now); timestamping is left to the caller.
            time: None,
            is_last_launched: false,
            // The entity's gameType code is preserved here. (Note: the Java
            // addPcGame path hardcodes gameType = 0; see REPORT.md.)
            game_type: game.game_type,
        }
    }
}

/// Build a small JSON object for the `data` column without any dependency.
fn imported_game_json(game: &ImportedGame) -> String {
    let steam_app_id = game.steam_app_id.as_deref().unwrap_or("");
    let mut out = String::from("{");
    out.push_str(&format!("\"cover\":{}", json_str(&game.cover)));
    out.push_str(&format!(",\"description\":{}", json_str(&game.description)));
    out.push_str(&format!(",\"exeFileBgType\":{}", game.exe_file_bg_type));
    out.push_str(&format!(",\"filePath\":{}", json_str(&game.file_path)));
    out.push_str(&format!(",\"gameType\":{}", game.game_type));
    out.push_str(&format!(",\"id\":{}", json_str(&game.id)));
    out.push_str(&format!(",\"localGameId\":{}", json_str(&game.local_game_id)));
    out.push_str(&format!(",\"name\":{}", json_str(&game.name)));
    out.push_str(&format!(",\"size\":{}", game.size));
    out.push_str(&format!(",\"steamAppId\":{}", json_str(steam_app_id)));
    if let Some(info) = &game.steam_game_info {
        out.push_str(&format!(",\"steamGameInfo\":{}", json_str(&info.steam_app_id.to_string())));
    }
    out.push('}');
    out
}

fn json_str(s: &str) -> String {
    let mut out = String::with_capacity(s.len() + 2);
    out.push('"');
    for c in s.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            c if (c as u32) < 0x20 => out.push_str(&format!("\\u{:04x}", c as u32)),
            c => out.push(c),
        }
    }
    out.push('"');
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::super::SteamGameInfo;

    fn sample() -> ImportedGame {
        ImportedGame::new(
            "42".into(),
            "local_abcd".into(),
            "My \"Game\"".into(),
            "/games/my/game.exe".into(),
            "/cover.png".into(),
            "desc\nline".into(),
            12_345,
            3,
            Some(SteamGameInfo::new(42)),
            Some("42".into()),
            2,
        )
    }

    #[test]
    fn maps_all_storage_columns() {
        let row = GameLibraryTable::from(sample());
        assert_eq!(row.id, 42);
        assert_eq!(row.user_id, "");
        assert_eq!(row.launch_type, LAUNCH_TYPE_PC_EMULATOR);
        assert_eq!(row.package_name, "/games/my/game.exe");
        assert_eq!(row.game_id, Some(42));
        assert_eq!(row.local_game_id.as_deref(), Some("local_abcd"));
        assert_eq!(row.local_mobile_app_id, None);
        assert_eq!(row.time, None);
        assert!(!row.is_last_launched);
        assert_eq!(row.game_type, 2);
        assert!(row.has_server_game());
    }

    #[test]
    fn data_json_escapes_and_carries_imported_fields() {
        let row = GameLibraryTable::from(sample());
        assert!(row.data.starts_with('{') && row.data.ends_with('}'));
        assert!(row.data.contains("\"name\":\"My \\\"Game\\\"\""));
        assert!(row.data.contains("\"description\":\"desc\\nline\""));
        assert!(row.data.contains("\"filePath\":\"/games/my/game.exe\""));
        assert!(row.data.contains("\"gameType\":2"));
        assert!(row.data.contains("\"steamGameInfo\":\"42\""));
    }

    #[test]
    fn unparseable_id_falls_back_to_zero() {
        let mut game = sample();
        game.id = "oops".into();
        let row = GameLibraryTable::from(game);
        assert_eq!(row.id, 0);
        assert_eq!(row.game_id, Some(0));
        assert!(!row.has_server_game());
    }

    #[test]
    fn json_str_escapes_control_characters() {
        assert_eq!(json_str("a\tb\r\nc"), "\"a\\tb\\r\\nc\"");
        assert_eq!(json_str("q\"\\"), "\"q\\\"\\\\\"");
    }
}
