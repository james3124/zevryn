//! Rust model of `com.xj.common.deeplink.GhDeepLinkPageType`.
//!
//! The Java enum has seven values, each carrying the wire string used by the
//! `page_type` query parameter of a `gamesir://gamehub` deep link. The mapping
//! and declaration order mirror the reference enum exactly.

/// The seven page types accepted by `page_type` (mirrors `GhDeepLinkPageType`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DeepLinkPage {
    MainPage,
    NewsDetail,
    GameDetail,
    PlayPcEmuGame,
    PlayCloudGame,
    WebUrl,
    UserInfo,
}

impl DeepLinkPage {
    /// Every page type, in declaration order (mirrors `getEntries()`).
    pub const ALL: [Self; 7] = [
        Self::MainPage,
        Self::NewsDetail,
        Self::GameDetail,
        Self::PlayPcEmuGame,
        Self::PlayCloudGame,
        Self::WebUrl,
        Self::UserInfo,
    ];

    /// Parse a wire value (e.g. `"game_detail"`) into a page type.
    ///
    /// Mirrors `GhDeepLinkPageType.Companion.a(String)`.
    pub fn from_str(s: &str) -> Option<Self> {
        Self::ALL.iter().find(|p| p.as_str() == s).copied()
    }

    /// The wire value for this page type (mirrors `getType()`).
    pub fn as_str(self) -> &'static str {
        match self {
            Self::MainPage => "main_page",
            Self::NewsDetail => "news_detail",
            Self::GameDetail => "game_detail",
            Self::PlayPcEmuGame => "play_pc_emu_game",
            Self::PlayCloudGame => "play_cloud_game",
            Self::WebUrl => "web_url",
            Self::UserInfo => "user_info",
        }
    }
}

impl core::fmt::Display for DeepLinkPage {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_str(self.as_str())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn has_exactly_seven_page_types() {
        assert_eq!(DeepLinkPage::ALL.len(), 7);
    }

    #[test]
    fn round_trips_all_wire_values() {
        for page in DeepLinkPage::ALL {
            assert_eq!(DeepLinkPage::from_str(page.as_str()), Some(page));
            assert_eq!(page.to_string(), page.as_str());
        }
    }

    #[test]
    fn rejects_unknown_wire_value() {
        assert_eq!(DeepLinkPage::from_str("nope"), None);
        assert_eq!(DeepLinkPage::from_str(""), None);
    }
}
