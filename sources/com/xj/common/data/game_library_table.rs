//! Portable model of `com.xj.common.data.table.GameLibraryTable`.
//!
//! A Room entity (primary key `_id`) representing one local game-library row.
//! Column names equal the Java field names, so those are used as the JSON
//! keys. Nullable columns (`gameId`, `localGameId`, `localMobileAppId`, `time`)
//! map to `Option`.

use super::json::Value;

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
    pub fn from_value(v: &Value) -> Self {
        Self {
            id: num(v, "_id", 0.0) as i64,
            user_id: str(v, "userId"),
            launch_type: num(v, "launchType", 0.0) as i32,
            package_name: str(v, "packageName"),
            game_id: v
                .get("gameId")
                .and_then(Value::as_f64)
                .map(|n| n as i32),
            local_game_id: v.get("localGameId").and_then(Value::as_str).map(str::to_owned),
            local_mobile_app_id: v
                .get("localMobileAppId")
                .and_then(Value::as_str)
                .map(str::to_owned),
            data: str(v, "data"),
            time: v.get("time").and_then(Value::as_str).map(str::to_owned),
            is_last_launched: v
                .get("isLastLaunched")
                .and_then(Value::as_bool)
                .unwrap_or(false),
            game_type: num(v, "gameType", 0.0) as i32,
        }
    }

    pub fn parse(input: &str) -> Result<Self, super::json::ParseError> {
        super::json::Parser::new(input).parse().map(|v| Self::from_value(&v))
    }

    /// A row has a server game id when `gameId` is present and positive.
    pub fn has_server_game(&self) -> bool {
        self.game_id.map(|id| id > 0).unwrap_or(false)
    }
}

fn str(v: &Value, key: &str) -> String {
    v.get(key).and_then(Value::as_str).unwrap_or("").to_owned()
}

fn num(v: &Value, key: &str, default: f64) -> f64 {
    v.get(key).and_then(Value::as_f64).unwrap_or(default)
}

#[cfg(test)]
mod tests {
    use super::*;

    const ROW: &str = r#"{
        "_id": 12, "userId": "u1", "launchType": 1403, "packageName": "com.x.game",
        "gameId": 42, "localGameId": "lg1", "localMobileAppId": null,
        "data": "{}", "time": "2026-01-01", "isLastLaunched": true, "gameType": 2
    }"#;

    #[test]
    fn parses_row() {
        let row = GameLibraryTable::parse(ROW).unwrap();
        assert_eq!(row.id, 12);
        assert_eq!(row.user_id, "u1");
        assert_eq!(row.launch_type, 1403);
        assert_eq!(row.package_name, "com.x.game");
        assert_eq!(row.game_id, Some(42));
        assert_eq!(row.local_game_id.as_deref(), Some("lg1"));
        assert_eq!(row.local_mobile_app_id, None);
        assert_eq!(row.time.as_deref(), Some("2026-01-01"));
        assert!(row.is_last_launched);
        assert_eq!(row.game_type, 2);
        assert!(row.has_server_game());
    }

    #[test]
    fn nullable_and_missing_fields() {
        let row = GameLibraryTable::parse(r#"{"_id":1,"data":"","packageName":"p","userId":"u"}"#).unwrap();
        assert_eq!(row.game_id, None);
        assert_eq!(row.local_game_id, None);
        assert!(!row.is_last_launched);
        assert!(!row.has_server_game());
    }
}
