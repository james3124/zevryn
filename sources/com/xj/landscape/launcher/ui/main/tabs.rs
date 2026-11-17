//! Portable model of the launcher's two-tab layout.
//!
//! Mirrors `LandscapeLauncherMainActivity.initView()` (the `f28611g` tab list),
//! `TabItemData` and the `GHLocaleManager.m()` locale gate. In this build the
//! host exposes exactly two tabs:
//!
//! * `llauncher_main_page_title_my`  -> "Dashboard"     -> `MyFragment`
//! * `llauncher_play_in_second`      -> "Instant Play"  -> `CloudPlayFragment`
//!
//! The `find games` (`llauncher_main_page_title_platform`) tab referenced by
//! `P2()` belongs to other configurations and is not part of this build's
//! two-tab layout, so it is intentionally absent.

/// The two tabs present in this build.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TabId {
    Dashboard,
    InstantPlay,
}

impl TabId {
    pub fn title(self) -> &'static str {
        match self {
            Self::Dashboard => "Dashboard",
            Self::InstantPlay => "Instant Play",
        }
    }
}

/// The fragment a tab creates, from the `Function0` factories in
/// `LandscapeLauncherMainActivity` (`M2()/I2()` -> MyFragment,
/// `N2()/J2()` -> CloudPlayFragment).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum FragmentKind {
    MyFragment,
    CloudPlayFragment,
}

/// Portable equivalent of `TabItemData`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Tab {
    pub id: TabId,
    pub title: String,
    pub fragment: FragmentKind,
    /// `TabItemData.f28632d`, set from
    /// `AppPreferences.INSTANCE.isFirstShowPlayInSecondsTab()` on the gated
    /// Instant Play tab.
    pub first_show: bool,
    /// `TabItemData.f28632d` is also used as a red-dot flag; exposed here for
    /// callers that need the raw value.
    pub red_dot: bool,
}

/// `GHLocaleManager.m(context)`: returns `true` iff the (lower-cased) locale
/// language is `"zh"`. This gates the Instant Play tab in the non-AppConfig
/// branch of `initView()`.
pub fn locale_gate_allows_instant_play(locale_language: &str) -> bool {
    locale_language.eq_ignore_ascii_case("zh")
}

/// Build the tab list exactly as `LandscapeLauncherMainActivity.initView()`
/// does:
///
/// * when `app_config_enabled` (the `AppConfig.l()` flag) is true, both tabs
///   are appended unconditionally with `first_show = false`;
/// * otherwise Dashboard is always appended, and Instant Play is appended
///   only when the locale gate passes, carrying the
///   `isFirstShowPlayInSecondsTab()` value.
pub fn build_tabs(
    app_config_enabled: bool,
    locale_language: &str,
    first_show_play_in_seconds: bool,
) -> Vec<Tab> {
    let mk = |id: TabId, first_show: bool| Tab {
        id,
        title: id.title().to_owned(),
        fragment: match id {
            TabId::Dashboard => FragmentKind::MyFragment,
            TabId::InstantPlay => FragmentKind::CloudPlayFragment,
        },
        first_show,
        red_dot: first_show,
    };

    let mut tabs = Vec::with_capacity(2);
    tabs.push(mk(TabId::Dashboard, false));
    if app_config_enabled {
        tabs.push(mk(TabId::InstantPlay, false));
    } else if locale_gate_allows_instant_play(locale_language) {
        tabs.push(mk(TabId::InstantPlay, first_show_play_in_seconds));
    }
    tabs
}

/// Index of a tab by id, if present (the gate can remove Instant Play).
pub fn index_of(tabs: &[Tab], id: TabId) -> Option<usize> {
    tabs.iter().position(|t| t.id == id)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn two_tabs_when_app_config_enabled() {
        let tabs = build_tabs(true, "en", false);
        assert_eq!(tabs.len(), 2);
        assert_eq!(tabs[0].title, "Dashboard");
        assert_eq!(tabs[0].fragment, FragmentKind::MyFragment);
        assert_eq!(tabs[1].title, "Instant Play");
        assert_eq!(tabs[1].fragment, FragmentKind::CloudPlayFragment);
        assert!(!tabs[1].first_show);
    }

    #[test]
    fn locale_gate_hides_instant_play_for_non_zh() {
        let tabs = build_tabs(false, "en", true);
        assert_eq!(tabs.len(), 1);
        assert_eq!(tabs[0].id, TabId::Dashboard);
        assert_eq!(index_of(&tabs, TabId::InstantPlay), None);
    }

    #[test]
    fn locale_gate_shows_instant_play_for_zh() {
        let tabs = build_tabs(false, "zh", true);
        assert_eq!(tabs.len(), 2);
        let instant = &tabs[index_of(&tabs, TabId::InstantPlay).unwrap()];
        assert!(instant.first_show);
        assert!(instant.red_dot);
    }

    #[test]
    fn locale_gate_is_case_insensitive() {
        // The gate consumes LanguageUtils.getLanguage() ("zh", "en", ...),
        // not a full locale tag.
        assert!(locale_gate_allows_instant_play("ZH"));
        assert!(locale_gate_allows_instant_play("zh"));
        assert!(!locale_gate_allows_instant_play("zh-CN"));
        assert!(!locale_gate_allows_instant_play("ja"));
    }
}
