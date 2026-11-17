//! Portable endpoint catalogue for the launcher repositories
//! (`sources/com/xj/landscape/launcher/data/repository/*Repo.java`).
//!
//! Coroutine bodies, flows, ViewModel callbacks and the OkHttp `Post`/`Get`
//! extension calls are intentionally not modeled. What is modeled is the set
//! of endpoint paths the repositories target and the act of wrapping a
//! request body in the signing interceptor's shape.

use std::collections::BTreeMap;

use super::http::api_config::{endpoints, ApiConfig};
use super::http::{HttpMethod, SignedRequest};

/// A named repository endpoint and the HTTP verb the decompiled repo uses.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RepoEndpoint {
    pub repo: &'static str,
    pub endpoint: &'static str,
    pub method: HttpMethod,
}

/// Endpoint catalogue assembled from the `*Repo.java` string constants.
pub const REPO_ENDPOINTS: &[RepoEndpoint] = &[
    RepoEndpoint { repo: "GameDetailRepository", endpoint: endpoints::GAME_DETAIL, method: HttpMethod::Post },
    RepoEndpoint { repo: "AlbumRepo", endpoint: endpoints::ALBUM_DETAIL, method: HttpMethod::Post },
    RepoEndpoint { repo: "HomeInfoRepository", endpoint: endpoints::INDEX_LIST, method: HttpMethod::Post },
    RepoEndpoint { repo: "GameIconsRepo", endpoint: endpoints::GAME_ICON, method: HttpMethod::Post },
    RepoEndpoint { repo: "NewsRepo", endpoint: endpoints::NEWS_LIST, method: HttpMethod::Post },
    RepoEndpoint { repo: "NewsDetailRepository", endpoint: endpoints::NEWS_DETAIL, method: HttpMethod::Post },
    RepoEndpoint { repo: "NewsRepo", endpoint: endpoints::NEWS_GUIDE_DETAIL, method: HttpMethod::Post },
    RepoEndpoint { repo: "SearchGameRepositoryV4", endpoint: endpoints::SEARCH_GAME_LIST, method: HttpMethod::Post },
    RepoEndpoint { repo: "SearchGameRepository", endpoint: endpoints::SEARCH_CATEGORY_LIST, method: HttpMethod::Post },
    RepoEndpoint { repo: "SearchGameRepository", endpoint: endpoints::SEARCH_CLASSIFY_LIST, method: HttpMethod::Post },
    RepoEndpoint { repo: "SearchGameRepository", endpoint: endpoints::SEARCH_TOP_CATEGORY_LIST, method: HttpMethod::Post },
    RepoEndpoint { repo: "GameCompatibilityRepository", endpoint: endpoints::CTS_LIST, method: HttpMethod::Post },
    RepoEndpoint { repo: "HightlightsRepository", endpoint: endpoints::VIDEO_GAME_LIST, method: HttpMethod::Post },
    RepoEndpoint { repo: "HomeInfoRepository", endpoint: endpoints::TOP_PLATFORM, method: HttpMethod::Post },
    RepoEndpoint { repo: "HomeInfoRepository", endpoint: endpoints::TOP_RECOMMEND, method: HttpMethod::Post },
    RepoEndpoint { repo: "MineDeviceRepository", endpoint: endpoints::DEVICE_LIST, method: HttpMethod::Post },
    RepoEndpoint { repo: "AppUpgradeRepo", endpoint: endpoints::APP_UPGRADE, method: HttpMethod::Post },
    RepoEndpoint { repo: "HomeInfoRepository", endpoint: endpoints::USER_NOTICE_LIST, method: HttpMethod::Post },
    RepoEndpoint { repo: "CloudGameOrderListRepository", endpoint: endpoints::CHECK_USER_TIMER, method: HttpMethod::Post },
    RepoEndpoint { repo: "CloudGameOrderListRepository", endpoint: endpoints::CLOUD_ORDER_LIST, method: HttpMethod::Post },
];

/// Build a signed request for a repository endpoint.
///
/// Returns `None` only for a sign-exempt URL (none of the catalogue endpoints
/// are), and resolves the endpoint against the configured base URL.
pub fn build_repo_request(
    endpoint: &RepoEndpoint,
    body: BTreeMap<String, String>,
    clientparams: &str,
    time_ms: i64,
) -> Option<SignedRequest> {
    if endpoint.method != HttpMethod::Post {
        return None;
    }
    let url = ApiConfig::new().endpoint(endpoint.endpoint);
    SignedRequest::build_post(url, body, clientparams, time_ms)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn catalogue_is_populated() {
        assert!(REPO_ENDPOINTS.len() >= 15);
        assert!(REPO_ENDPOINTS.iter().all(|e| e.method == HttpMethod::Post));
        assert!(REPO_ENDPOINTS
            .iter()
            .any(|e| e.endpoint == endpoints::GAME_DETAIL));
    }

    #[test]
    fn builds_signed_repo_request() {
        let ep = REPO_ENDPOINTS
            .iter()
            .find(|e| e.endpoint == endpoints::GAME_DETAIL)
            .unwrap();
        let mut body = BTreeMap::new();
        body.insert("game_name".into(), "Hollow Knight".into());
        let req = build_repo_request(ep, body, "cp", 123).unwrap();
        assert!(req.url.ends_with("/card/getGameDetail"));
        assert_eq!(req.field("game_name"), Some("Hollow Knight"));
        assert_eq!(req.field("sign"), Some(req.sign.as_str()));
    }
}
