//! Portable dashboard model for the GameHub Lite launcher — mirror of
//! `com.xj.landscape.launcher.ui.main.me` (`MyFragment`, `MyVM`,
//! `MyContainerView`, `MyGamesData`, `MyGamesItemData`) and the row DTOs in
//! `com.xj.common.data.list` (`CardLineData`, `CardLineType`).
//!
//! Field names are taken verbatim from the decompiled DTOs, not invented:
//! `card_line_data`, `card_real_type`, `card_list`, `fixed_card_size`,
//! `aspect_ratio`, `is_vertical`, ... (`CardLineData.java`) and `id`, `title`,
//! `content_img`, `game_cover_image`, `game_back_image`, `steam_app_id`, ...
//! (`MyGamesItemData.java`).
//!
//! Only portable logic is modeled here. Android views, DataBinding,
//! RecyclerView adapters and coroutine/network code (MyVM/Repository) are out
//! of scope — see `../REPORT.md`.

#[path = "json.rs"]
mod json;

use json::{Parser, Value};

/// Row-kind enum from `com.xj.common.data.list.CardLineType`.
///
/// `CardLineType.fromCode()` falls back to `UNKNOWN` (code `-1`) for codes it
/// does not recognize; here `Unknown(code)` retains the offending value.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CardLineType {
    Album,       // 1
    AlbumGames,  // 2
    FixedImage,  // 3
    BannerImage, // 4
    Platform,    // 5
    Unknown(i32),
}

impl CardLineType {
    pub fn from_code(code: i32) -> Self {
        match code {
            1 => Self::Album,
            2 => Self::AlbumGames,
            3 => Self::FixedImage,
            4 => Self::BannerImage,
            5 => Self::Platform,
            other => Self::Unknown(other),
        }
    }

    pub fn code(self) -> i32 {
        match self {
            Self::Album => 1,
            Self::AlbumGames => 2,
            Self::FixedImage => 3,
            Self::BannerImage => 4,
            Self::Platform => 5,
            Self::Unknown(c) => c,
        }
    }

    pub fn is_known(self) -> bool {
        !matches!(self, Self::Unknown(_))
    }
}

/// Entry-point ids from `CardLineData.Companion`, used to classify a
/// top-platform card (`MyGamesItemData.topPlatformType`).
pub mod entry_id {
    pub const PS_LINK: i32 = 999;
    pub const PC_LINK: i32 = 1000;
    pub const GAME_LIB: i32 = 4;
    pub const STEAM: i32 = 5;
    pub const MOBILE_GAME: i32 = 6;
    pub const PC_EMULATOR: i32 = 7;
    pub const IMPORT_GAME: i32 = 8;
}

/// `TopPlatFormType` from `MyGamesItemData.Companion.a(topPlatformType)`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TopPlatform {
    Steam,
    GameLib,
    MobileGame,
    PcEmulator,
    ImportGame,
    Other,
}

/// Classify a `topPlatformType` string (the app compares it to
/// `String.valueOf(entryId)`) the same way `MyGamesItemData.Companion.a` does.
/// Note: the decompiled logic deliberately collapses PC_LINK and PS_LINK to
/// `Other` (they are only tested to exclude them from the later branches),
/// which is preserved here.
pub fn top_platform_from_str(s: &str) -> TopPlatform {
    let code = match s.parse::<i32>() {
        Ok(n) => n,
        Err(_) => return TopPlatform::Other,
    };
    if code == entry_id::STEAM {
        return TopPlatform::Steam;
    }
    if code == entry_id::GAME_LIB {
        return TopPlatform::GameLib;
    }
    if code == entry_id::PC_LINK || code == entry_id::PS_LINK {
        return TopPlatform::Other;
    }
    match code {
        entry_id::MOBILE_GAME => TopPlatform::MobileGame,
        entry_id::PC_EMULATOR => TopPlatform::PcEmulator,
        entry_id::IMPORT_GAME => TopPlatform::ImportGame,
        _ => TopPlatform::Other,
    }
}

/// One game card (`MyGamesItemData` / API card objects).
#[derive(Debug, Clone, PartialEq)]
pub struct GameCard {
    pub id: String,
    pub game_id: String,
    pub title: String,
    pub subtitle: String,
    pub content_img: String,
    pub game_cover_image: String,
    pub game_back_image: String,
    pub start_type: String,
    pub start_name: String,
    pub start_icon: String,
    pub steam_app_id: String,
    pub card_tag: String,
    pub topic_name: String,
    /// `topPlatformType` from `MyGamesItemData`, kept as text so callers can
    /// use `top_platform_from_str` on it.
    pub top_platform_type: String,
}

impl GameCard {
    /// `MyGamesItemData.P(str)`: false for null/blank, "0", "-1" and "-2".
    pub fn is_valid_id(s: &str) -> bool {
        !(s.trim().is_empty() || s == "0" || s == "-1" || s == "-2")
    }

    /// `MyGamesItemData.x(str)` — case-sensitive prefix check for "http".
    pub fn starts_with_http(s: &str) -> bool {
        s.starts_with("http")
    }
}

/// One row of the dashboard (`CardLineData`).
#[derive(Debug, Clone, PartialEq)]
pub struct CardLine {
    pub id: String,
    pub title: String,
    pub desc: String,
    pub card_type: Option<CardLineType>,
    pub bg_img: String,
    pub aspect_ratio: Option<f64>,
    pub fixed_card_size: Option<i32>,
    pub is_vertical: bool,
    pub is_text_outside: bool,
    pub is_play_video: bool,
    pub is_more: bool,
    pub page: i32,
    pub page_size: i32,
    pub total: i32,
    pub cards: Vec<GameCard>,
}

/// Parsed dashboard response (`card_line_data` from the API).
#[derive(Debug, Clone, PartialEq)]
pub struct Dashboard {
    pub lines: Vec<CardLine>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DashboardError {
    pub message: String,
}

impl DashboardError {
    fn new(msg: impl Into<String>) -> Self {
        Self { message: msg.into() }
    }
}

fn str_field(v: &Value, key: &str) -> String {
    v.get(key).and_then(Value::as_str).unwrap_or("").to_owned()
}

fn num_field(v: &Value, key: &str) -> Option<f64> {
    v.get(key).and_then(Value::as_f64)
}

fn int_field(v: &Value, key: &str, default: i32) -> i32 {
    num_field(v, key).unwrap_or(default as f64) as i32
}

fn bool_field(v: &Value, key: &str) -> bool {
    v.get(key).and_then(Value::as_bool).unwrap_or(false)
}

fn parse_card(v: &Value) -> GameCard {
    GameCard {
        id: str_field(v, "id"),
        game_id: str_field(v, "game_id"),
        title: str_field(v, "title"),
        subtitle: str_field(v, "subtitle"),
        content_img: str_field(v, "content_img"),
        game_cover_image: str_field(v, "game_cover_image"),
        game_back_image: str_field(v, "game_back_image"),
        start_type: str_field(v, "start_type"),
        start_name: str_field(v, "start_name"),
        start_icon: str_field(v, "start_icon"),
        steam_app_id: str_field(v, "steam_app_id"),
        card_tag: str_field(v, "card_tag"),
        topic_name: str_field(v, "topic_name"),
        top_platform_type: str_field(v, "topPlatformType"),
    }
}

fn parse_line(v: &Value) -> CardLine {
    let cards = v
        .get("card_list")
        .and_then(Value::as_array)
        .map(|items| items.iter().map(parse_card).collect())
        .unwrap_or_default();
    CardLine {
        id: str_field(v, "id"),
        title: str_field(v, "title"),
        desc: str_field(v, "desc"),
        card_type: num_field(v, "card_real_type").map(|n| CardLineType::from_code(n as i32)),
        bg_img: str_field(v, "bg_img"),
        aspect_ratio: num_field(v, "aspect_ratio"),
        fixed_card_size: num_field(v, "fixed_card_size").map(|n| n as i32),
        is_vertical: bool_field(v, "is_vertical"),
        is_text_outside: bool_field(v, "is_text_outside"),
        is_play_video: bool_field(v, "is_play_video"),
        is_more: bool_field(v, "is_more"),
        page: int_field(v, "page", 0),
        page_size: int_field(v, "page_size", 0),
        total: int_field(v, "total", 0),
        cards,
    }
}

impl Dashboard {
    /// Parse a dashboard response. Accepts either a bare `card_line_data`
    /// array or an object containing it, matching the two shapes seen in the
    /// app DTOs.
    pub fn parse(input: &str) -> Result<Self, DashboardError> {
        let root = Parser::new(input)
            .parse()
            .map_err(|e| DashboardError::new(format!("JSON error at {}: {}", e.offset, e.message)))?;

        let lines_value = root.get("card_line_data").unwrap_or(&root);
        let lines = lines_value
            .as_array()
            .map(|items| items.iter().map(parse_line).collect())
            .ok_or_else(|| DashboardError::new("expected card_line_data array"))?;
        Ok(Self { lines })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const FIXTURE: &str = r#"{
      "card_line_data": [
        {
          "id": "row1", "title": "Recently Played", "card_real_type": 5,
          "fixed_card_size": 4, "aspect_ratio": 1.5, "is_vertical": true,
          "page": 1, "page_size": 20, "total": 2,
          "card_list": [
            {"id":"a","game_id":"g1","title":"Game One","game_cover_image":"cover1.webp","start_type":"PC_EMU","topPlatformType":"7"},
            {"id":"b","game_id":"g2","title":"Game Two","content_img":"img2.webp","steam_app_id":"123"}
          ]
        },
        {"id":"row2","title":"Banners","card_real_type":4,"card_list":[]}
      ]
    }"#;

    #[test]
    fn parses_dashboard() {
        let d = Dashboard::parse(FIXTURE).unwrap();
        assert_eq!(d.lines.len(), 2);
        let row = &d.lines[0];
        assert_eq!(row.title, "Recently Played");
        assert_eq!(row.card_type, Some(CardLineType::Platform));
        assert_eq!(row.fixed_card_size, Some(4));
        assert_eq!(row.aspect_ratio, Some(1.5));
        assert!(row.is_vertical);
        assert_eq!(row.page, 1);
        assert_eq!(row.total, 2);
        assert_eq!(row.cards.len(), 2);
        assert_eq!(row.cards[0].start_type, "PC_EMU");
        assert_eq!(row.cards[1].steam_app_id, "123");
    }

    #[test]
    fn parses_bare_array() {
        let d = Dashboard::parse(r#"[{"id":"r","title":"T","card_real_type":3}]"#).unwrap();
        assert_eq!(d.lines.len(), 1);
        assert_eq!(d.lines[0].card_type, Some(CardLineType::FixedImage));
    }

    #[test]
    fn card_type_mapping() {
        assert_eq!(CardLineType::from_code(1), CardLineType::Album);
        assert_eq!(CardLineType::from_code(2), CardLineType::AlbumGames);
        assert_eq!(CardLineType::from_code(3), CardLineType::FixedImage);
        assert_eq!(CardLineType::from_code(4), CardLineType::BannerImage);
        assert_eq!(CardLineType::from_code(5), CardLineType::Platform);
        assert_eq!(CardLineType::from_code(99), CardLineType::Unknown(99));
        assert_eq!(CardLineType::Platform.code(), 5);
        assert!(!CardLineType::Unknown(-1).is_known());
        assert!(CardLineType::Album.is_known());
    }

    #[test]
    fn top_platform_mapping() {
        assert_eq!(top_platform_from_str("5"), TopPlatform::Steam);
        assert_eq!(top_platform_from_str("4"), TopPlatform::GameLib);
        assert_eq!(top_platform_from_str("6"), TopPlatform::MobileGame);
        assert_eq!(top_platform_from_str("7"), TopPlatform::PcEmulator);
        assert_eq!(top_platform_from_str("8"), TopPlatform::ImportGame);
        // PC_LINK and PS_LINK collapse to Other in the decompiled logic.
        assert_eq!(top_platform_from_str("1000"), TopPlatform::Other);
        assert_eq!(top_platform_from_str("999"), TopPlatform::Other);
        assert_eq!(top_platform_from_str("nonsense"), TopPlatform::Other);
    }

    #[test]
    fn card_predicates() {
        assert!(!GameCard::is_valid_id(""));
        assert!(!GameCard::is_valid_id("0"));
        assert!(!GameCard::is_valid_id("-1"));
        assert!(!GameCard::is_valid_id("  "));
        assert!(GameCard::is_valid_id("g1"));
        assert!(GameCard::starts_with_http("https://x"));
        assert!(!GameCard::starts_with_http("HTTPS://x"));
    }

    #[test]
    fn rejects_non_array() {
        assert!(Dashboard::parse(r#"{"card_line_data":{}}"#).is_err());
    }

    #[test]
    fn reports_json_errors() {
        let e = Dashboard::parse("{bad json").unwrap_err();
        assert!(e.message.contains("JSON error"));
    }
}
