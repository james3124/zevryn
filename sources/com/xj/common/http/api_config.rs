//! Portable model of `com.xj.common.http.EggGameHttpConfig` and the legacy
//! endpoints referenced by the request-signing interceptors.
//!
//! `EggGameHttpConfig`'s static initializer contains a ternary
//! `(!constants.e() && constants.d()) ? URL : URL` where **both** branches are
//! the same string, so the effective base URL is always `BASE_URL` regardless
//! of the runtime flags. That is modeled directly here.

/// The single base URL used by this build (`EggGameHttpConfig.f23888b`).
pub const BASE_URL: &str = "https://gamehub-lite-api.emuready.workers.dev/";

/// Legacy `clientapi` host. `TokenInterceptor` (adb/wifiui) explicitly skips
/// signing for this exact URL.
pub const LEGACY_CLIENT_API: &str = "https://clientgsw.vgabc.com/clientapi/";
/// Legacy `uxapi` host seen in the decompiled app.
pub const LEGACY_UX_API: &str = "https://clientegg.vgabc.com/uxapi/";
/// Local debug endpoint seen in the decompiled app.
pub const LEGACY_LOCAL_USER_INFO: &str = "http://127.0.0.1/user/get_user_info";

/// Endpoint paths used by the launcher repositories (`*Repo.java`). Kept as
/// the path strings found in the decompiled sources; resolve with
/// [`ApiConfig::endpoint`].
pub mod endpoints {
    pub const GAME_DETAIL: &str = "card/getGameDetail";
    pub const ALBUM_DETAIL: &str = "card/getAlbumDetail";
    pub const INDEX_LIST: &str = "card/getIndexList";
    pub const CTS_LIST: &str = "card/getCtsList";
    pub const GAME_ICON: &str = "card/getGameIcon";
    pub const NEWS_LIST: &str = "card/getNewsList";
    pub const NEWS_DETAIL: &str = "card/getNewDetail";
    pub const NEWS_GUIDE_DETAIL: &str = "card/getNewsGuideDetail";
    pub const SEARCH_GAME_LIST: &str = "search/getGameList";
    pub const SEARCH_CATEGORY_LIST: &str = "game/searchCategoryList";
    pub const SEARCH_CLASSIFY_LIST: &str = "game/searchClassifyList";
    pub const SEARCH_TOP_CATEGORY_LIST: &str = "game/searchTopCategoryList";
    pub const CATEGORY_LIST: &str = "game/getClassify";
    pub const GAME_INDEX_LIST: &str = "game/getIndexList";
    pub const TOP_RECOMMEND: &str = "game/getTopRecommendList";
    pub const TOP_PLATFORM: &str = "game/getTopPlatform";
    pub const USER_PLAYED_GAMES: &str = "game/userPlayedGame";
    pub const VIDEO_GAME_LIST: &str = "game/videoGameList";
    pub const LIKE_VIDEO: &str = "game/likeVideo";
    pub const CANCEL_LIKE_VIDEO: &str = "game/cancelLikeVideo";
    pub const DEVICE_LIST: &str = "devices/getDevices";
    pub const USER_INFO: &str = "user/info";
    pub const APP_UPGRADE: &str = "upgrade/getAppUpgradeApk";
    pub const CHECK_USER_TIMER: &str = "cloud/game/check_user_timer";
    pub const CLOUD_ORDER_LIST: &str = "cloud/order_list";
    pub const USER_NOTICE_LIST: &str = "social/userNoticeList";
    pub const DOC_LIST: &str = "doc/docList";
}

/// Resolved HTTP configuration (portable subset). The OkHttp builder details
/// (timeouts, cache, cookie jar, interceptors) are deliberately not modeled;
/// see `REPORT.md`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ApiConfig {
    pub base_url: String,
}

impl Default for ApiConfig {
    fn default() -> Self {
        Self::new()
    }
}

impl ApiConfig {
    pub fn new() -> Self {
        Self { base_url: BASE_URL.to_owned() }
    }

    /// Resolve an endpoint path against the base URL, collapsing duplicate
    /// slashes like the app's `RemoveExtraSlashInterceptor` does.
    pub fn endpoint(&self, path: &str) -> String {
        format!("{}{}", self.base_url, path.trim_start_matches('/'))
    }

    /// Whether a URL is exempt from the legacy signing interceptor. The
    /// decompiled interceptor checks for exactly `LEGACY_CLIENT_API`.
    pub fn is_sign_exempt(url: &str) -> bool {
        url == LEGACY_CLIENT_API
    }

    /// Resolve one of the named `endpoints::*` paths.
    pub fn named(&self, path: &str) -> String {
        self.endpoint(path)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn base_url_is_stable() {
        assert_eq!(ApiConfig::new().base_url, BASE_URL);
        assert_eq!(
            BASE_URL,
            "https://gamehub-lite-api.emuready.workers.dev/"
        );
    }

    #[test]
    fn resolves_endpoints() {
        let c = ApiConfig::new();
        assert_eq!(
            c.endpoint(endpoints::GAME_DETAIL),
            "https://gamehub-lite-api.emuready.workers.dev/card/getGameDetail"
        );
        // Leading slash is normalized away.
        assert_eq!(c.endpoint("/user/info"), c.named("user/info"));
    }

    #[test]
    fn sign_exemption_is_exact() {
        assert!(ApiConfig::is_sign_exempt(LEGACY_CLIENT_API));
        assert!(!ApiConfig::is_sign_exempt(LEGACY_UX_API));
        assert!(!ApiConfig::is_sign_exempt(BASE_URL));
    }
}
