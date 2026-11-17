//! Compatibility mirror of the app's existing portable storage entity
//! `com.xj.common.data.game_library_table.GameLibraryTable`.
//!
//! The real definition lives at
//! `app/sources/com/xj/common/data/game_library_table.rs`. It is *not*
//! imported directly because it depends on `super::json` (the sibling
//! `com/xj/common/data/json.rs`), which would make this area depend on a
//! specific `#[path]` wiring in `app/src/lib.rs`. Re-declaring the struct with
//! identical field names and types keeps this area self-contained (std-only)
//! while remaining field-for-field compatible with the real model.
//!
//! Column names in Room are `id`, `user_id`, `launch_type`, `package_name`,
//! `game_id`, `local_game_id`, `local_mobile_app_id`, `data`, `time`,
//! `is_last_launched`, `game_type` (see `db/schema.rs`, table `t_game_library`).

#[derive(Debug, Clone, PartialEq)]
pub struct GameLibraryTable {
    pub id: i64, // `_id`
    pub user_id: String,
    pub launch_type: i32,
    pub package_name: String,
    pub game_id: Option<i32>,
    pub local_game_id: Option<String>,
    pub local_mobile_app_id: Option<String>,
    pub data: String,
    pub time: Option<String>, // java.sql.Date, kept as raw text
    pub is_last_launched: bool,
    pub game_type: i32,
}

impl GameLibraryTable {
    /// A row has a server game id when `game_id` is present and positive
    /// (same rule as the real model's `has_server_game`).
    pub fn has_server_game(&self) -> bool {
        self.game_id.map(|id| id > 0).unwrap_or(false)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mirror_has_expected_shape() {
        let row = GameLibraryTable {
            id: 7,
            user_id: "u".into(),
            launch_type: 1403,
            package_name: "p".into(),
            game_id: Some(42),
            local_game_id: Some("lg".into()),
            local_mobile_app_id: None,
            data: "{}".into(),
            time: None,
            is_last_launched: false,
            game_type: 2,
        };
        assert!(row.has_server_game());
        assert_eq!(row.local_mobile_app_id, None);
    }
}
