//! RouteRequest builder: turn a deep-link page into a concrete route path.
//!
//! Mirrors the portion of GameHub's deeplink handling that picks a target
//! activity for a `page_type` and appends the remaining query parameters to
//! the route string passed to `TheRouter.withRoutePath(...)`.
//!
//! The seven `DeepLinkPage` values are re-declared here (rather than imported
//! from the sibling deeplink module) so this area stays self-contained.
//! They mirror `com.xj.common.deeplink.GhDeepLinkPageType` exactly.

/// The seven page types accepted by a `gamesir://gamehub` deep link.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
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

    /// Parse a wire value such as `"game_detail"` (mirrors `Companion.a`).
    pub fn from_str(value: &str) -> Option<Self> {
        Self::ALL.iter().find(|page| page.as_str() == value).copied()
    }

    /// The wire value for this page (mirrors `getType()`).
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

/// The real route path registered for each deep-link page, taken from the
/// generated `RouterMap__TheRouter__*.java` tables.
pub fn route_path_for(page: DeepLinkPage) -> &'static str {
    match page {
        DeepLinkPage::MainPage => "/landscapeLauncher/ui/LandscapeLauncherMainActivity",
        DeepLinkPage::NewsDetail => "/launcher/ui/gamedetail/NewsDetailActivity",
        DeepLinkPage::GameDetail => "/launcher/ui/gamedetail/GameDetailActivity",
        // "play pc emu game" lands on the game library / winemu home.
        DeepLinkPage::PlayPcEmuGame => "/game/ui/gameli_brary/GameLibraryActivity",
        DeepLinkPage::PlayCloudGame => "/cloud/ui/LauncherCloudGameActivity",
        DeepLinkPage::WebUrl => "/landscape/launcher/ui/WebActivity",
        DeepLinkPage::UserInfo => "/landscapeLauncher/ui/menu/MenuUserInfoActivity",
    }
}

/// A route request assembled from a deep-link page and its parameters.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RouteRequest {
    pub page: DeepLinkPage,
    pub path: String,
    pub params: Vec<(String, String)>,
}

impl RouteRequest {
    /// Start from the default route path for `page`.
    pub fn new(page: DeepLinkPage) -> Self {
        Self {
            page,
            path: route_path_for(page).to_string(),
            params: Vec::new(),
        }
    }

    /// Override the path (e.g. a handler-specific target).
    pub fn with_path(mut self, path: impl Into<String>) -> Self {
        self.path = path.into();
        self
    }

    /// Append a query parameter.
    pub fn param(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.params.push((key.into(), value.into()));
        self
    }

    /// Render the path with its `?key=value` query, ready for route resolution.
    pub fn url(&self) -> String {
        if self.params.is_empty() {
            return self.path.clone();
        }
        let mut url = self.path.clone();
        url.push('?');
        let pairs: Vec<String> = self
            .params
            .iter()
            .map(|(key, value)| format!("{key}={value}"))
            .collect();
        url.push_str(&pairs.join("&"));
        url
    }
}

impl core::fmt::Display for RouteRequest {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_str(&self.url())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn page_type_strings_round_trip() {
        assert_eq!(DeepLinkPage::ALL.len(), 7);
        for page in DeepLinkPage::ALL {
            assert_eq!(DeepLinkPage::from_str(page.as_str()), Some(page));
        }
        assert_eq!(DeepLinkPage::from_str("game_detail"), Some(DeepLinkPage::GameDetail));
        assert_eq!(DeepLinkPage::from_str("unknown"), None);
    }

    #[test]
    fn builds_route_path_with_params() {
        let request = RouteRequest::new(DeepLinkPage::GameDetail)
            .param("id", "42")
            .param("type", "1");
        assert_eq!(
            request.url(),
            "/launcher/ui/gamedetail/GameDetailActivity?id=42&type=1"
        );
    }
}
