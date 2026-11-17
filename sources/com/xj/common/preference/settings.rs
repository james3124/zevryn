//! Typed settings view over a [`KvStore`], with Java initializer defaults.
//!
//! `Settings::default()` reproduces the values in the Kotlin field delegates
//! of `AppPreferences` plus the inline defaults in `GHLocaleManager` and
//! `SplashActivity`. `from_store` reads each key with the same default so a
//! fresh store yields exactly `Settings::default()`.

use super::keys::SettingsKey;
use super::kv::KvStore;

/// Portable snapshot of the app's persisted scalar settings.
#[derive(Debug, Clone, PartialEq)]
pub struct Settings {
    /// `deviceUniqueId`, key `uniqueId`, default `""`.
    pub device_unique_id: String,
    /// `lastScrollPositionByMyGames`, key `lastMyGamesScrollPosition`, default `-1`.
    pub last_scroll_position_by_my_games: i64,
    /// `firstInstallTime`, default `-1`.
    pub first_install_time: i64,
    /// `lastVisibleExploreTime`, default `-1`.
    pub last_visible_explore_time: i64,
    /// `isFirstShowPlayInSecondsTab`, default `true`.
    pub is_first_show_play_in_seconds_tab: bool,
    /// `wineScreenBrightnessSwitch` (MMKV), default `false`.
    pub wine_screen_brightness_switch: bool,
    /// `wineScreenBrightness` (MMKV), default `-1.0`.
    pub wine_screen_brightness: f64,
    /// `gh-language` / `language`, default `""` (format `language$country`).
    pub language: String,
    /// `uminit` (`spUtils`), default `0`.
    pub um_init: i64,
    /// `intro_video_prefs` / `intro_video_played`, default `false`.
    pub intro_video_played: bool,
    /// `intro_video_prefs` / `gif_loaded`, default `false`.
    pub gif_loaded: bool,
}

impl Default for Settings {
    fn default() -> Self {
        Settings {
            device_unique_id: String::new(),
            last_scroll_position_by_my_games: -1,
            first_install_time: -1,
            last_visible_explore_time: -1,
            is_first_show_play_in_seconds_tab: true,
            wine_screen_brightness_switch: false,
            wine_screen_brightness: -1.0,
            language: String::new(),
            um_init: 0,
            intro_video_played: false,
            gif_loaded: false,
        }
    }
}

impl Settings {
    /// Read every setting from `store`, falling back to Java defaults.
    pub fn from_store<S: KvStore>(store: &S) -> Self {
        let d = Settings::default();
        Settings {
            device_unique_id: store.get_string(SettingsKey::UniqueId.as_str(), &d.device_unique_id),
            last_scroll_position_by_my_games: store.get_i64(
                SettingsKey::LastMyGamesScrollPosition.as_str(),
                d.last_scroll_position_by_my_games,
            ),
            first_install_time: store.get_i64(
                SettingsKey::FirstInstallTime.as_str(),
                d.first_install_time,
            ),
            last_visible_explore_time: store.get_i64(
                SettingsKey::LastVisibleExploreTime.as_str(),
                d.last_visible_explore_time,
            ),
            is_first_show_play_in_seconds_tab: store.get_bool(
                SettingsKey::IsFirstShowPlayInSecondsTab.as_str(),
                d.is_first_show_play_in_seconds_tab,
            ),
            wine_screen_brightness_switch: store.get_bool(
                SettingsKey::WineScreenBrightnessSwitch.as_str(),
                d.wine_screen_brightness_switch,
            ),
            wine_screen_brightness: store.get_f64(
                SettingsKey::WineScreenBrightness.as_str(),
                d.wine_screen_brightness,
            ),
            language: store.get_string(SettingsKey::Language.as_str(), &d.language),
            um_init: store.get_i64(SettingsKey::UmInit.as_str(), d.um_init),
            intro_video_played: store.get_bool(
                SettingsKey::IntroVideoPlayed.as_str(),
                d.intro_video_played,
            ),
            gif_loaded: store.get_bool(SettingsKey::GifLoaded.as_str(), d.gif_loaded),
        }
    }

    /// Write every setting to `store` under its exact key.
    pub fn to_store<S: KvStore>(&self, store: &mut S) {
        store.set_string(SettingsKey::UniqueId.as_str(), &self.device_unique_id);
        store.set_i64(
            SettingsKey::LastMyGamesScrollPosition.as_str(),
            self.last_scroll_position_by_my_games,
        );
        store.set_i64(SettingsKey::FirstInstallTime.as_str(), self.first_install_time);
        store.set_i64(
            SettingsKey::LastVisibleExploreTime.as_str(),
            self.last_visible_explore_time,
        );
        store.set_bool(
            SettingsKey::IsFirstShowPlayInSecondsTab.as_str(),
            self.is_first_show_play_in_seconds_tab,
        );
        store.set_bool(
            SettingsKey::WineScreenBrightnessSwitch.as_str(),
            self.wine_screen_brightness_switch,
        );
        store.set_f64(
            SettingsKey::WineScreenBrightness.as_str(),
            self.wine_screen_brightness,
        );
        store.set_string(SettingsKey::Language.as_str(), &self.language);
        store.set_i64(SettingsKey::UmInit.as_str(), self.um_init);
        store.set_bool(
            SettingsKey::IntroVideoPlayed.as_str(),
            self.intro_video_played,
        );
        store.set_bool(SettingsKey::GifLoaded.as_str(), self.gif_loaded);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::super::InMemoryKv;

    #[test]
    fn java_defaults() {
        let d = Settings::default();
        assert_eq!(d.device_unique_id, "");
        assert_eq!(d.last_scroll_position_by_my_games, -1);
        assert_eq!(d.first_install_time, -1);
        assert_eq!(d.last_visible_explore_time, -1);
        assert!(d.is_first_show_play_in_seconds_tab);
        assert!(!d.wine_screen_brightness_switch);
        assert_eq!(d.wine_screen_brightness, -1.0);
        assert_eq!(d.language, "");
        assert_eq!(d.um_init, 0);
        assert!(!d.intro_video_played);
        assert!(!d.gif_loaded);
    }

    #[test]
    fn empty_store_yields_defaults() {
        let kv = InMemoryKv::new();
        assert_eq!(Settings::from_store(&kv), Settings::default());
    }

    #[test]
    fn round_trip_all_fields() {
        let mut expected = Settings::default();
        expected.device_unique_id = "abc-123".into();
        expected.last_scroll_position_by_my_games = 12;
        expected.first_install_time = 1_700_000_000_000;
        expected.last_visible_explore_time = 1_700_000_050_000;
        expected.is_first_show_play_in_seconds_tab = false;
        expected.wine_screen_brightness_switch = true;
        expected.wine_screen_brightness = 0.6;
        expected.language = "zh$CN".into();
        expected.um_init = 1;
        expected.intro_video_played = true;
        expected.gif_loaded = true;

        let mut kv = InMemoryKv::new();
        expected.to_store(&mut kv);
        assert_eq!(kv.len(), SettingsKey::all().len());
        assert_eq!(Settings::from_store(&kv), expected);
    }
}
