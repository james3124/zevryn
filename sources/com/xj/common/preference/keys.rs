//! Exact string keys used by the app's persisted key/value stores.
//!
//! Every variant's `as_str` is a byte-for-byte copy of a string literal found
//! in the read-only decompiled sources. See `REPORT.md` for the source file and
//! line of each key.

/// Every exact key string found in the reference stores.
///
/// The enum is intentionally flat: the same key name can appear in more than
/// one store (e.g. a SharedPreferences file and an MMKV instance), and the
/// store is chosen by the caller, not by the key.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SettingsKey {
    // --- AppPreferences (SharedPreferences `<pkg>_app_global_preferences`) ---
    /// `AppPreferences.deviceUniqueId` — stored key `uniqueId` (Java `str6`).
    UniqueId,
    /// `AppPreferences.lastScrollPositionByMyGames` — stored key
    /// `lastMyGamesScrollPosition` (Java `str8`).
    LastMyGamesScrollPosition,
    /// `AppPreferences.firstInstallTime` — Java `str`.
    FirstInstallTime,
    /// `AppPreferences.lastVisibleExploreTime` — Java `str2`.
    LastVisibleExploreTime,
    /// `AppPreferences.isFirstShowPlayInSecondsTab` — Java `str3`.
    IsFirstShowPlayInSecondsTab,

    // --- AppPreferences (MMKV `_mmkv_app_global_preferences`) ---
    /// `AppPreferences.wineScreenBrightnessSwitch` — Java `str4` (MMKV).
    WineScreenBrightnessSwitch,
    /// `AppPreferences.wineScreenBrightness` — Java `str5` (MMKV).
    WineScreenBrightness,

    // --- GHLocaleManager (SharedPreferences `gh-language`) ---
    /// `IjkMediaMeta.IJKM_KEY_LANGUAGE`, value `"language"`.
    Language,

    // --- SplashActivity, SPUtils.f() (SharedPreferences `spUtils`) ---
    /// `SPUtils.f().i("uminit", 0)` / `.m("uminit", 1)`.
    UmInit,

    // --- SplashActivity (SharedPreferences `intro_video_prefs`) ---
    /// `getBoolean("intro_video_played", false)`.
    IntroVideoPlayed,
    /// `getBoolean("gif_loaded", false)`; written as `true`.
    GifLoaded,
}

impl SettingsKey {
    /// The exact persisted key string.
    pub fn as_str(self) -> &'static str {
        match self {
            SettingsKey::UniqueId => "uniqueId",
            SettingsKey::LastMyGamesScrollPosition => "lastMyGamesScrollPosition",
            SettingsKey::FirstInstallTime => "firstInstallTime",
            SettingsKey::LastVisibleExploreTime => "lastVisibleExploreTime",
            SettingsKey::IsFirstShowPlayInSecondsTab => "isFirstShowPlayInSecondsTab",
            SettingsKey::WineScreenBrightnessSwitch => "wineScreenBrightnessSwitch",
            SettingsKey::WineScreenBrightness => "wineScreenBrightness",
            SettingsKey::Language => "language",
            SettingsKey::UmInit => "uminit",
            SettingsKey::IntroVideoPlayed => "intro_video_played",
            SettingsKey::GifLoaded => "gif_loaded",
        }
    }

    /// All keys, in declaration order (useful for enumerating defaults).
    pub fn all() -> &'static [SettingsKey] {
        &[
            SettingsKey::UniqueId,
            SettingsKey::LastMyGamesScrollPosition,
            SettingsKey::FirstInstallTime,
            SettingsKey::LastVisibleExploreTime,
            SettingsKey::IsFirstShowPlayInSecondsTab,
            SettingsKey::WineScreenBrightnessSwitch,
            SettingsKey::WineScreenBrightness,
            SettingsKey::Language,
            SettingsKey::UmInit,
            SettingsKey::IntroVideoPlayed,
            SettingsKey::GifLoaded,
        ]
    }
}

impl core::fmt::Display for SettingsKey {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_str(self.as_str())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn exact_strings() {
        let expected = [
            "uniqueId",
            "lastMyGamesScrollPosition",
            "firstInstallTime",
            "lastVisibleExploreTime",
            "isFirstShowPlayInSecondsTab",
            "wineScreenBrightnessSwitch",
            "wineScreenBrightness",
            "language",
            "uminit",
            "intro_video_played",
            "gif_loaded",
        ];
        let got: Vec<&str> = SettingsKey::all().iter().map(|k| k.as_str()).collect();
        assert_eq!(got, expected);
    }

    #[test]
    fn display_matches_key() {
        assert_eq!(SettingsKey::UmInit.to_string(), "uminit");
    }
}
