//! Rust model of the navigation-service "interface shape" from
//! `com.xj.common.service` (e.g. `IHomeService`, `IMainNavService`,
//! `ILandscapeLauncherNavService`).
//!
//! In the app those are TheRouter service-locator interfaces carrying many
//! Android-typed methods (`Intent`, `Activity`, `FragmentActivity`, `View`).
//! The portable contract is small: a nav service reports whether it is ready
//! and navigates to one of the seven deep-link page types. [`dispatch`]
//! mirrors `GameHubDeepLinkManager`: a link is deferred while the main page is
//! not ready, otherwise it is handed to the service.

use super::deeplink::{DeepLink, DeepLinkPage};

/// One navigation request, derived from a parsed deep link.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NavRequest {
    pub page: DeepLinkPage,
    pub params: Vec<(String, String)>,
}

impl NavRequest {
    pub fn new(page: DeepLinkPage, params: Vec<(String, String)>) -> Self {
        Self { page, params }
    }

    pub fn param(&self, key: &str) -> Option<&str> {
        self.params
            .iter()
            .find(|(k, _)| k == key)
            .map(|(_, v)| v.as_str())
    }
}

impl From<&DeepLink> for NavRequest {
    fn from(link: &DeepLink) -> Self {
        Self::new(link.page, link.params.clone())
    }
}

/// Result of trying to navigate.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NavOutcome {
    /// The target page was opened.
    Navigated,
    /// Service not ready yet; caller should retry once the main page is up
    /// (mirrors the manager caching a pending intent).
    Deferred,
    /// No handler for this page / navigation failed.
    Ignored,
}

/// A portable navigation error.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NavError(pub String);

/// Portable shape of a TheRouter `*NavService`.
///
/// `is_ready` mirrors `IHomeService.i()` ("home page ready"), used by
/// `GameHubDeepLinkManager` to decide whether a deep link can be handled now.
pub trait NavService {
    fn is_ready(&self) -> bool {
        true
    }

    fn navigate(&mut self, request: NavRequest) -> Result<NavOutcome, NavError>;
}

/// Dispatch a parsed deep link through a [`NavService`].
pub fn dispatch<S: NavService>(service: &mut S, link: &DeepLink) -> NavOutcome {
    if !service.is_ready() {
        return NavOutcome::Deferred;
    }
    match service.navigate(NavRequest::from(link)) {
        Ok(outcome) => outcome,
        Err(_) => NavOutcome::Ignored,
    }
}

/// Minimal in-memory implementation: a reference skeleton for concrete
/// services and a convenient test double.
#[derive(Debug)]
pub struct RecordingNavService {
    pub ready: bool,
    pub visited: Vec<NavRequest>,
}

impl RecordingNavService {
    pub fn new(ready: bool) -> Self {
        Self {
            ready,
            visited: Vec::new(),
        }
    }
}

impl Default for RecordingNavService {
    fn default() -> Self {
        Self::new(true)
    }
}

impl NavService for RecordingNavService {
    fn is_ready(&self) -> bool {
        self.ready
    }

    fn navigate(&mut self, request: NavRequest) -> Result<NavOutcome, NavError> {
        self.visited.push(request);
        Ok(NavOutcome::Navigated)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::super::deeplink::parse;

    #[test]
    fn request_from_link_carries_page_and_params() {
        let link = parse("gamesir://gamehub?page_type=game_detail&game_id=7").unwrap();
        let req = NavRequest::from(&link);
        assert_eq!(req.page, DeepLinkPage::GameDetail);
        assert_eq!(req.param("game_id"), Some("7"));
        assert_eq!(req, NavRequest::new(DeepLinkPage::GameDetail, link.params));
    }

    #[test]
    fn dispatch_navigates_when_ready() {
        let link = parse("gamesir://gamehub?page_type=main_page").unwrap();
        let mut service = RecordingNavService::default();
        assert_eq!(dispatch(&mut service, &link), NavOutcome::Navigated);
        assert_eq!(service.visited.len(), 1);
        assert_eq!(service.visited[0].page, DeepLinkPage::MainPage);
    }

    #[test]
    fn dispatch_defers_when_not_ready() {
        let link = parse("gamesir://gamehub?page_type=game_detail").unwrap();
        let mut service = RecordingNavService::new(false);
        assert_eq!(dispatch(&mut service, &link), NavOutcome::Deferred);
        assert!(service.visited.is_empty());
    }

    #[test]
    fn dispatch_ignores_on_error() {
        struct Failing;
        impl NavService for Failing {
            fn navigate(&mut self, _: NavRequest) -> Result<NavOutcome, NavError> {
                Err(NavError("no handler".into()))
            }
        }
        let link = parse("gamesir://gamehub?page_type=web_url").unwrap();
        let mut service = Failing;
        assert_eq!(dispatch(&mut service, &link), NavOutcome::Ignored);
    }

    #[test]
    fn all_seven_page_types_are_dispatchable() {
        let mut service = RecordingNavService::default();
        for page in DeepLinkPage::ALL {
            let link = DeepLink {
                page,
                params: Vec::new(),
            };
            assert_eq!(dispatch(&mut service, &link), NavOutcome::Navigated);
        }
        assert_eq!(service.visited.len(), 7);
        assert_eq!(service.visited[6].page, DeepLinkPage::UserInfo);
    }
}
