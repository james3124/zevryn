//! Portable model of `com.xj.common.data.gameinfo.GameDetailEntity`.
//!
//! Only the JSON wire fields (those annotated with Gson `@SerializedName`)
//! and a few constructor fields used by the portable predicates are modeled.
//! Nested/complex objects (`card_line_data`, `friend_list`, startup params,
//! compatibility params, ...) are kept as raw `Value` nodes so the DTO stays
//! dependency-free and parseable by the crate's JSON reader; see `REPORT.md`.

use super::json::Value;

#[derive(Debug, Clone, PartialEq)]
pub struct GameDetailEntity {
    // @SerializedName scalar/string fields
    pub age_rating: String,
    pub ai_desc: String,
    pub back_image: String,
    pub channel_icon: String,
    pub company: String,
    pub cover_image: String,
    pub description: String,
    pub file_md5: Option<String>,
    pub file_size: Option<String>,
    pub game_category: String,
    pub game_file_size: i64,
    pub game_mode: String,
    pub id: i32,
    pub jump_type: i32,
    pub jump_url: String,
    pub logo: String,
    pub name: String,
    pub other_desc: String,
    pub package_str: String,
    pub release_date: String,
    pub remark: String,
    pub share_link: String,
    pub steam_appid: Option<String>,
    pub steam_game_info: Option<String>,
    pub title_desc: String,
    pub video_url: String,

    // String-array fields
    pub game_lang: Vec<String>,
    pub screenshot: Vec<String>,

    // Nested objects kept as raw JSON nodes
    pub card_line_data: Option<Value>,
    pub cst_data: Option<Value>,
    pub friend_list: Option<Value>,
    pub game_channel_params: Option<Value>,
    pub game_open_param: Option<Value>,
    pub game_startup_params: Option<Value>,
    pub game_video_list: Option<Value>,
    pub not_fount_cts: Option<Value>,
    pub recommend_game: Option<Value>,

    // Constructor fields used by the portable predicates
    pub local_game_id: String,
    pub local_mobile_app_id: Option<String>,
    pub launch_type: i32,
    pub game_type: i32,
}

impl GameDetailEntity {
    pub fn from_value(v: &Value) -> Self {
        Self {
            age_rating: str(v, "age_rating"),
            ai_desc: str(v, "ai_desc"),
            back_image: str(v, "back_image"),
            channel_icon: str(v, "channel_icon"),
            company: str(v, "company"),
            cover_image: str(v, "cover_image"),
            description: str(v, "description"),
            file_md5: opt_str(v, "file_md5"),
            file_size: opt_str(v, "file_size"),
            game_category: str(v, "game_category"),
            game_file_size: num(v, "game_file_size", 0.0) as i64,
            game_mode: str(v, "game_mode"),
            id: num(v, "id", -1.0) as i32,
            jump_type: num(v, "jump_type", 0.0) as i32,
            jump_url: str(v, "jump_url"),
            logo: str(v, "logo"),
            name: str(v, "name"),
            other_desc: str(v, "other_desc"),
            package_str: str(v, "package_str"),
            release_date: str(v, "release_date"),
            remark: str(v, "remark"),
            share_link: str(v, "share_link"),
            steam_appid: opt_str(v, "steam_appid"),
            steam_game_info: opt_str(v, "steam_game_info"),
            title_desc: str(v, "title_desc"),
            video_url: str(v, "video_url"),
            game_lang: str_array(v, "game_lang"),
            screenshot: str_array(v, "screenshot"),
            card_line_data: opt_value(v, "card_line_data"),
            cst_data: opt_value(v, "cst_data"),
            friend_list: opt_value(v, "friend_list"),
            game_channel_params: opt_value(v, "game_channel_params"),
            game_open_param: opt_value(v, "game_open_param"),
            game_startup_params: opt_value(v, "game_startup_params"),
            game_video_list: opt_value(v, "game_video_list"),
            not_fount_cts: opt_value(v, "not_fount_cts"),
            recommend_game: opt_value(v, "recommend_game"),
            local_game_id: str(v, "localGameId"),
            local_mobile_app_id: opt_str(v, "localMobileAppId"),
            launch_type: num(v, "launchType", 0.0) as i32,
            game_type: num(v, "gameType", 0.0) as i32,
        }
    }

    pub fn parse(input: &str) -> Result<Self, super::json::ParseError> {
        super::json::Parser::new(input).parse().map(|v| Self::from_value(&v))
    }

    /// `getValidId()`: prefer a positive server `id`, then `localGameId`, then
    /// `localMobileAppId`, else empty.
    pub fn valid_id(&self) -> String {
        if self.id > 0 {
            return self.id.to_string();
        }
        if !self.local_game_id.is_empty() {
            return self.local_game_id.clone();
        }
        self.local_mobile_app_id.clone().unwrap_or_default()
    }

    /// `isPcGame()` — `launchType == 1403` (`START_TYPE_PC_EMULATOR`).
    pub fn is_pc_game(&self) -> bool {
        self.launch_type == 1403
    }

    /// `isMobileGame(startType)`: mobile start types from `LauncherTypeEnum`
    /// (1301, 1302, 1502, 10, 7) or `GameType.MOBILE_INSTALL_APP` (1).
    pub fn is_mobile_game(&self, start_type: i32) -> bool {
        matches!(start_type, 1301 | 1302 | 1502 | 10 | 7) || self.game_type == 1
    }

    /// `isValidGameInfo()`: false only when there is no server id and both
    /// local ids are blank.
    pub fn is_valid_game_info(&self) -> bool {
        !(self.id <= 0 && self.local_game_id.is_empty() && self.local_mobile_app_id.as_deref().unwrap_or("").is_empty())
    }
}

fn str(v: &Value, key: &str) -> String {
    v.get(key).and_then(Value::as_str).unwrap_or("").to_owned()
}

fn opt_str(v: &Value, key: &str) -> Option<String> {
    v.get(key).and_then(Value::as_str).map(str::to_owned)
}

fn num(v: &Value, key: &str, default: f64) -> f64 {
    v.get(key).and_then(Value::as_f64).unwrap_or(default)
}

fn str_array(v: &Value, key: &str) -> Vec<String> {
    match v.get(key) {
        Some(Value::Array(items)) => items
            .iter()
            .filter_map(Value::as_str)
            .map(str::to_owned)
            .collect(),
        _ => Vec::new(),
    }
}

fn opt_value(v: &Value, key: &str) -> Option<Value> {
    v.get(key).cloned()
}

#[cfg(test)]
mod tests {
    use super::*;

    const DETAIL: &str = r#"{
      "id": 481516, "name": "Hollow Knight", "company": "Team Cherry",
      "game_category": "Action", "game_mode": "Single", "launchType": 1403,
      "game_lang": ["zh", "en"], "screenshot": ["a.webp", "b.webp"],
      "localGameId": "lg-9", "localMobileAppId": null,
      "game_channel_params": [{"channel":"x"}], "card_line_data": {"id":"row"}
    }"#;

    #[test]
    fn parses_detail() {
        let d = GameDetailEntity::parse(DETAIL).unwrap();
        assert_eq!(d.id, 481516);
        assert_eq!(d.name, "Hollow Knight");
        assert_eq!(d.company, "Team Cherry");
        assert_eq!(d.game_lang, vec!["zh", "en"]);
        assert_eq!(d.screenshot.len(), 2);
        assert!(d.game_channel_params.is_some());
        assert!(d.card_line_data.is_some());
        assert_eq!(d.steam_appid, None);
        assert_eq!(d.valid_id(), "481516");
    }

    #[test]
    fn valid_id_falls_back_to_local_ids() {
        let d = GameDetailEntity::parse(r#"{"id":0,"localGameId":"lg-1"}"#).unwrap();
        assert_eq!(d.valid_id(), "lg-1");
        assert!(d.is_valid_game_info());

        let d2 = GameDetailEntity::parse(r#"{"id":-1,"localGameId":"","localMobileAppId":"m1"}"#).unwrap();
        assert_eq!(d2.valid_id(), "m1");

        let d3 = GameDetailEntity::parse(r#"{"id":0,"localGameId":""}"#).unwrap();
        assert_eq!(d3.valid_id(), "");
        assert!(!d3.is_valid_game_info());
    }

    #[test]
    fn game_type_predicates() {
        let d = GameDetailEntity::parse(DETAIL).unwrap();
        assert!(d.is_pc_game());
        assert!(!d.is_mobile_game(1403));
        assert!(d.is_mobile_game(1302));

        let d2 = GameDetailEntity::parse(r#"{"gameType":1}"#).unwrap();
        assert!(d2.is_mobile_game(999));
    }
}
