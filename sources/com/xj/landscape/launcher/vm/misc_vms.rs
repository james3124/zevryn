//! Lightweight portable state for the remaining studied ViewModels.
//!
//! These are intentionally small: the decompiled sources expose only a
//! `LiveData` field plus a couple of state-update methods, so only that state
//! is modeled here. Networking, coroutine scopes and Android callbacks are
//! represented through `on_*` / `post_*` methods.

use super::async_state::AsyncState;

/// `AlbumDetailVM`: paginated album game list with a loading guard and a
/// "can load more" flag.
#[derive(Debug, Clone)]
pub struct AlbumDetailViewModel {
    pub loading: bool,
    pub items: Vec<String>,
    pub has_more: bool,
    pub current_page: i32,
    pub total_count: i32,
}

impl Default for AlbumDetailViewModel {
    fn default() -> Self {
        Self {
            loading: false,
            items: Vec::new(),
            has_more: true,
            current_page: 1,
            total_count: 0,
        }
    }
}

impl AlbumDetailViewModel {
    pub fn new() -> Self {
        Self::default()
    }

    /// Mirrors `AlbumDetailVM.l(page)`: ignore while loading, set the flag
    /// when refreshing page 1.
    pub fn load(&mut self, page: i32) -> bool {
        if self.loading {
            return false;
        }
        if page == 1 {
            self.loading = true;
        }
        self.current_page = page;
        true
    }

    /// Mirrors `AlbumDetailVM.s()`: only advance when more pages exist.
    pub fn load_more(&mut self) -> bool {
        if !self.has_more {
            return false;
        }
        self.load(self.current_page + 1)
    }

    /// Mirrors `AlbumDetailVM.t()`.
    pub fn refresh(&mut self) -> bool {
        self.load(1)
    }

    /// Publishes a page result: page 1 replaces the list, later pages append.
    pub fn on_loaded(&mut self, items: Vec<String>, has_more: bool) {
        if self.current_page == 1 {
            self.items = items;
        } else {
            self.items.extend(items);
        }
        self.total_count = self.items.len() as i32;
        self.has_more = has_more;
        self.loading = false;
    }
}

/// `FeedbackVM`: a single one-shot LiveData event slot.
#[derive(Debug, Clone, Default)]
pub struct FeedbackViewModel {
    pub event: Option<String>,
}

impl FeedbackViewModel {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn post_event(&mut self, event: impl Into<String>) {
        self.event = Some(event.into());
    }

    pub fn take_event(&mut self) -> Option<String> {
        self.event.take()
    }
}

/// `HightlightsVM`: keyword search result list.
#[derive(Debug, Clone, Default)]
pub struct HightlightsViewModel {
    pub results: AsyncState<Vec<String>>,
    pub last_keyword: Option<String>,
}

impl HightlightsViewModel {
    pub fn new() -> Self {
        Self::default()
    }

    /// Mirrors `HightlightsVM.k(keyWord)`.
    pub fn search(&mut self, keyword: impl Into<String>) {
        self.last_keyword = Some(keyword.into());
        self.results.begin();
    }

    pub fn on_results(&mut self, results: Vec<String>) {
        self.results.succeed(results);
    }

    pub fn on_failed<S: Into<String>>(&mut self, message: S) {
        self.results.fail(message);
    }
}

/// Portable pager matching `com.xj.common.utils.pager.PagerImpl`:
/// `a()` → page, `b(i)` → setPage, `d()` → `page == 1`,
/// `e(max)` → `page = min(max, page + 1)`, `g()` → page size.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Pager {
    pub page: i32,
    pub page_size: i32,
}

impl Default for Pager {
    fn default() -> Self {
        Self { page: 1, page_size: i32::MAX }
    }
}

impl Pager {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn set_page(&mut self, page: i32) {
        self.page = page;
    }

    pub fn is_first_page(&self) -> bool {
        self.page == 1
    }

    pub fn increase_page(&mut self, max_page: i32) {
        self.page = max_page.min(self.page + 1);
    }
}

/// `SearchMainVM`: pager + async search-result state.
#[derive(Debug, Clone, Default)]
pub struct SearchMainViewModel {
    pub pager: Pager,
    pub results: AsyncState<Vec<String>>,
    pub has_more: bool,
}

impl SearchMainViewModel {
    pub fn new() -> Self {
        Self::default()
    }

    /// Starts a fresh search at page 1.
    pub fn refresh(&mut self) {
        self.pager.set_page(1);
        self.has_more = true;
        self.results.begin();
    }

    /// Advances the page (capped by `total_pages`) and starts a load.
    pub fn load_more(&mut self, total_pages: i32) -> bool {
        if !self.has_more {
            return false;
        }
        self.pager.increase_page(total_pages);
        self.results.begin();
        true
    }

    pub fn on_results(&mut self, items: Vec<String>, has_more: bool) {
        self.has_more = has_more;
        self.results.succeed(items);
    }

    pub fn on_failed<S: Into<String>>(&mut self, message: S) {
        self.results.fail(message);
    }
}

/// `GuideLoginVM`: login result state (the three `catch` callbacks all
/// delivered `null` to their `block`, modeled as an error).
#[derive(Debug, Clone, Default)]
pub struct GuideLoginViewModel {
    pub login: AsyncState<String>,
    pub last_openid: Option<String>,
}

impl GuideLoginViewModel {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn login(&mut self) {
        self.login.begin();
    }

    pub fn login_qq(&mut self, openid: impl Into<String>) {
        self.last_openid = Some(openid.into());
        self.login.begin();
    }

    pub fn login_wechat(&mut self, openid: impl Into<String>) {
        self.last_openid = Some(openid.into());
        self.login.begin();
    }

    pub fn on_login_success(&mut self, user_id: impl Into<String>) {
        self.login.succeed(user_id.into());
    }

    pub fn on_login_failure<S: Into<String>>(&mut self, message: S) {
        self.login.fail(message);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn album_loading_guard_and_pagination() {
        let mut vm = AlbumDetailViewModel::new();
        assert!(vm.has_more);
        assert_eq!(vm.current_page, 1);
        assert!(vm.load(1));
        assert!(vm.loading);
        assert!(!vm.load(2)); // guarded while loading

        vm.on_loaded(vec!["a".into(), "b".into()], true);
        assert!(!vm.loading);
        assert_eq!(vm.items.len(), 2);
        assert_eq!(vm.total_count, 2);

        assert!(vm.load_more());
        assert_eq!(vm.current_page, 2);
        vm.on_loaded(vec!["c".into()], false);
        assert_eq!(vm.items, vec!["a", "b", "c"]);
        assert!(!vm.has_more);
        assert!(!vm.load_more());
    }

    #[test]
    fn feedback_event_is_one_shot() {
        let mut vm = FeedbackViewModel::new();
        vm.post_event("submitted");
        assert_eq!(vm.take_event().as_deref(), Some("submitted"));
        assert_eq!(vm.take_event(), None);
    }

    #[test]
    fn highlights_search_state() {
        let mut vm = HightlightsViewModel::new();
        vm.search("knight");
        assert!(vm.results.is_loading());
        assert_eq!(vm.last_keyword.as_deref(), Some("knight"));
        vm.on_results(vec!["r1".into()]);
        assert_eq!(vm.results.value().unwrap().len(), 1);
        vm.search("x");
        vm.on_failed("network");
        assert_eq!(vm.results.error(), Some("network"));
    }

    #[test]
    fn pager_matches_pager_impl() {
        let mut p = Pager::new();
        assert_eq!(p.page, 1);
        assert_eq!(p.page_size, i32::MAX);
        assert!(p.is_first_page());
        p.increase_page(3);
        assert_eq!(p.page, 2);
        p.increase_page(2);
        assert_eq!(p.page, 2); // capped at max
        p.set_page(1);
        assert!(p.is_first_page());
    }

    #[test]
    fn search_refresh_and_load_more() {
        let mut vm = SearchMainViewModel::new();
        vm.pager.set_page(4);
        vm.refresh();
        assert_eq!(vm.pager.page, 1);
        assert!(vm.results.is_loading());
        vm.on_results(vec!["a".into()], true);
        assert!(vm.results.is_success());

        assert!(vm.load_more(1)); // capped to page 1
        assert_eq!(vm.pager.page, 1);
        vm.on_results(vec!["b".into()], false);
        assert!(!vm.has_more);
        assert!(!vm.load_more(5));
        vm.on_failed("err");
        assert_eq!(vm.results.error(), Some("err"));
    }

    #[test]
    fn guide_login_state() {
        let mut vm = GuideLoginViewModel::new();
        vm.login_qq("openid-1");
        assert!(vm.login.is_loading());
        assert_eq!(vm.last_openid.as_deref(), Some("openid-1"));
        vm.on_login_success("user-9");
        assert_eq!(vm.login.value().map(String::as_str), Some("user-9"));

        vm.login_wechat("openid-2");
        vm.on_login_failure("cancelled");
        assert_eq!(vm.last_openid.as_deref(), Some("openid-2"));
        assert_eq!(vm.login.error(), Some("cancelled"));
    }
}
