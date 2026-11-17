//! Locale model for the Rust port of `com.xj.base.language.GHLocaleManager`.
//!
//! The reference app stores the chosen language in `SharedPreferences` under
//! the key `"language"` as `"<language>$<region>"` (see `GHLocaleManager.n`,
//! `.l`, `.r`, `.q`). It only branches on four "primary" locales — Simplified
//! Chinese, Japanese, Russian and everything else (`GHLocaleManager.f`, `.h`)
//! — while resource lookup itself falls back through a much larger set of
//! `values-*` directories. This module models both layers.

use std::fmt;

/// A locale reduced to the pieces Android resource resolution cares about.
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord, Default)]
pub struct Locale {
    /// Lower-case ISO language code, e.g. `"zh"` (legacy `in`/`iw` are kept).
    pub language: String,
    /// Upper-case region/country code, e.g. `"CN"`. Empty when unspecified.
    pub region: String,
    /// Script code, e.g. `"Latn"`. Empty when unspecified (`b+sr+Latn`).
    pub script: String,
}

impl Locale {
    pub fn new(language: impl Into<String>, region: impl Into<String>) -> Self {
        Self {
            language: language.into(),
            region: region.into(),
            script: String::new(),
        }
    }

    /// A base locale with no region or script, e.g. `Locale::base("en")`.
    pub fn base(language: impl Into<String>) -> Self {
        Self::new(language, "")
    }

    pub fn is_base(&self) -> bool {
        self.region.is_empty() && self.script.is_empty()
    }

    /// Parse an Android resource qualifier or BCP-47-style language tag.
    ///
    /// Recognises `zh-rCN`, `zh_CN`, `en-US`, `b+es+419` and `b+sr+Latn`.
    pub fn parse_tag(tag: &str) -> Option<Self> {
        let tag = tag.trim();
        if tag.is_empty() {
            return None;
        }

        if let Some(rest) = tag.strip_prefix("b+") {
            // Android's BCP-47 syntax: b+<language>+<region|script>[+...].
            let parts: Vec<&str> = rest.split('+').collect();
            let language = parts.first()?.to_ascii_lowercase();
            if language.is_empty() {
                return None;
            }
            let mut locale = Self::base(language);
            if let Some(extra) = parts.get(1) {
                if extra.len() == 4 && extra.chars().all(|c| c.is_ascii_alphabetic()) {
                    locale.script = title_case(extra);
                } else {
                    locale.region = normalize_region(extra);
                }
            }
            return Some(locale);
        }

        let normalized = tag.replace('_', "-");
        let parts: Vec<&str> = normalized.split('-').collect();
        let language = parts.first()?.to_ascii_lowercase();
        if language.is_empty() || !language.chars().all(|c| c.is_ascii_alphanumeric()) {
            return None;
        }
        let mut locale = Self::base(language);
        for part in parts.iter().skip(1) {
            if part.is_empty() {
                continue;
            }
            if let Some(region) = part.strip_prefix('r').filter(|_| part.len() > 2) {
                locale.region = region.to_ascii_uppercase();
            } else if part.len() == 4 && part.chars().all(|c| c.is_ascii_alphabetic()) {
                locale.script = title_case(part);
            } else if part.len() == 2 || part.len() == 3 {
                locale.region = normalize_region(part);
            }
        }
        Some(locale)
    }

    /// BCP-47-like tag, e.g. `"zh-CN"` or `"zh"` (mirrors `toLanguageTag()`).
    pub fn to_language_tag(&self) -> String {
        let mut tag = self.language.clone();
        if !self.script.is_empty() {
            tag.push('-');
            tag.push_str(&self.script);
        }
        if !self.region.is_empty() {
            tag.push('-');
            tag.push_str(&self.region);
        }
        tag
    }

    /// Storage form used by `GHLocaleManager.n()`: `"language$region"`.
    pub fn to_storage(&self) -> String {
        format!("{}${}", self.language, self.region)
    }

    /// Inverse of [`Locale::to_storage`] (mirrors `GHLocaleManager.r()` / `.q()`).
    ///
    /// The reference accepts exactly one `'$'`; without it the saved value is
    /// treated as "follow system" and yields `None`.
    pub fn from_storage(s: &str) -> Option<Self> {
        if !valid_storage(s) {
            return None;
        }
        let (language, region) = s.split_once('$')?;
        if language.is_empty() {
            return None;
        }
        Some(Self::new(language, region))
    }

    /// Which of the four branched locales this maps to (`GHLocaleManager.f`).
    pub fn primary(&self) -> PrimaryLocale {
        PrimaryLocale::from_language(&self.language)
    }
}

impl fmt::Display for Locale {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.to_language_tag())
    }
}

/// Mirrors `GHLocaleManager.l()`: exactly one `'$'` separator.
pub fn valid_storage(s: &str) -> bool {
    s.matches('$').count() == 1
}

fn title_case(s: &str) -> String {
    let mut chars = s.chars();
    match chars.next() {
        Some(first) => first.to_uppercase().collect::<String>() + &chars.as_str().to_ascii_lowercase(),
        None => String::new(),
    }
}

fn normalize_region(s: &str) -> String {
    let s = s.trim();
    if s.len() == 2 && s.chars().all(|c| c.is_ascii_alphabetic()) {
        s.to_ascii_uppercase()
    } else {
        s.to_string()
    }
}

/// Alternate codes for legacy Android language tags, used as extra catalog
/// lookup keys (e.g. Indonesian `in`/`id`, Hebrew `iw`/`he`).
pub fn language_aliases(language: &str) -> &'static [&'static str] {
    match language {
        "in" | "id" => &["in", "id"],
        "iw" | "he" => &["iw", "he"],
        "ji" | "yi" => &["ji", "yi"],
        _ => &[],
    }
}

/// The four locales the reference branches on in `GHLocaleManager.f()` / `.h()`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PrimaryLocale {
    Chinese,
    Japanese,
    Russian,
    Other,
}

impl PrimaryLocale {
    pub fn from_language(language: &str) -> Self {
        match language {
            "zh" => Self::Chinese,
            "ja" => Self::Japanese,
            "ru" => Self::Russian,
            _ => Self::Other,
        }
    }

    pub fn from_tag(tag: &str) -> Self {
        Locale::parse_tag(tag)
            .map(|l| Self::from_language(&l.language))
            .unwrap_or(Self::Other)
    }

    /// Display names from `GHLocaleManager.h()`. `Other` has no built-in name
    /// (the app falls back to the localized `follow_system` string resource).
    pub fn display_name(&self) -> Option<&'static str> {
        match self {
            Self::Chinese => Some("简体中文"),
            Self::Japanese => Some("にほんご"),
            Self::Russian => Some("русский"),
            Self::Other => None,
        }
    }
}

/// Base languages that ship `values-<lang>/` directories in the reference.
pub static BASE_LOCALES: &[&str] = &[
    "af", "am", "ar", "as", "az", "be", "bg", "bn", "bs", "ca", "cs", "da", "de", "el", "en",
    "es", "et", "eu", "fa", "fi", "fr", "gl", "gu", "hi", "hr", "hu", "hy", "in", "is", "it",
    "iw", "ja", "ka", "kk", "km", "kn", "ko", "ky", "lo", "lt", "lv", "mk", "ml", "mn", "mr",
    "ms", "my", "nb", "ne", "nl", "or", "pa", "pl", "pt", "ro", "ru", "si", "sk", "sl", "sq",
    "sr", "sv", "sw", "ta", "te", "th", "tl", "tr", "uk", "ur", "uz", "vi", "zh", "zu",
];

/// Region/script-specific `values-<qualifier>/` directories in the reference.
pub static REGIONAL_LOCALES: &[&str] = &[
    "ar-rAE",
    "b+es+419",
    "b+sr+Latn",
    "cs-rCZ",
    "de-rDE",
    "en-rAU",
    "en-rCA",
    "en-rGB",
    "en-rIN",
    "en-rUS",
    "en-rXC",
    "es-rES",
    "es-rUS",
    "fr-rCA",
    "fr-rFR",
    "ja-rJP",
    "kk-rKZ",
    "ko-rKR",
    "pt-rBR",
    "pt-rPT",
    "ru-rRU",
    "vi-rVN",
    "zh-rCN",
    "zh-rHK",
    "zh-rMO",
    "zh-rTW",
];

/// The default/fallback locale (`values/` with no qualifier is English).
pub const DEFAULT_LOCALE: &str = "en";

/// Query the app's supported locale set.
pub struct SupportedLocales;

impl SupportedLocales {
    pub fn base_locales() -> &'static [&'static str] {
        BASE_LOCALES
    }

    pub fn regional_locales() -> &'static [&'static str] {
        REGIONAL_LOCALES
    }

    /// `true` when the app has resources for `tag`, either directly or via
    /// fallback to a supported base language.
    pub fn contains(tag: &str) -> bool {
        let tag = tag.trim();
        if BASE_LOCALES.contains(&tag) || REGIONAL_LOCALES.contains(&tag) {
            return true;
        }
        match Locale::parse_tag(tag) {
            Some(locale) => {
                let lang = locale.language.as_str();
                BASE_LOCALES.contains(&lang) || language_aliases(lang).iter().any(|a| BASE_LOCALES.contains(a))
            }
            None => false,
        }
    }

    /// Resolve a full tag to its base language, e.g. `"zh-rCN" -> "zh"`.
    /// Returns `None` for malformed tags.
    pub fn resolve_base(tag: &str) -> Option<String> {
        Locale::parse_tag(tag).map(|l| l.language)
    }
}

/// Convenience wrapper around [`SupportedLocales::resolve_base`].
pub fn resolve(tag: &str) -> Option<String> {
    SupportedLocales::resolve_base(tag)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_regional_and_bcp47_tags() {
        let zh = Locale::parse_tag("zh-rCN").unwrap();
        assert_eq!(zh.language, "zh");
        assert_eq!(zh.region, "CN");
        assert!(zh.script.is_empty());

        let en = Locale::parse_tag("en-rUS").unwrap();
        assert_eq!((en.language.as_str(), en.region.as_str()), ("en", "US"));
        assert_eq!(Locale::parse_tag("zh_CN").unwrap(), zh);
        assert_eq!(Locale::parse_tag("EN-us").unwrap().to_language_tag(), "en-US");

        let bcp = Locale::parse_tag("b+es+419").unwrap();
        assert_eq!((bcp.language.as_str(), bcp.region.as_str()), ("es", "419"));

        let sr = Locale::parse_tag("b+sr+Latn").unwrap();
        assert_eq!((sr.language.as_str(), sr.script.as_str()), ("sr", "Latn"));
        assert_eq!(sr.to_language_tag(), "sr-Latn");
    }

    #[test]
    fn resolve_base_maps_region_to_language() {
        assert_eq!(resolve("zh-rCN").as_deref(), Some("zh"));
        assert_eq!(resolve("en-rUS").as_deref(), Some("en"));
        assert_eq!(resolve("pt-BR").as_deref(), Some("pt"));
        assert_eq!(resolve("b+sr+Latn").as_deref(), Some("sr"));
        assert_eq!(resolve("").as_deref(), None);
    }

    #[test]
    fn supported_set_contains_key_locales() {
        for tag in ["en", "zh", "ja", "ru", "de", "fr", "es", "ko", "pt", "zh-rCN", "en-rUS"] {
            assert!(SupportedLocales::contains(tag), "expected support for {tag}");
        }
        assert!(!SupportedLocales::contains("xx"));
        assert!(!SupportedLocales::contains("hdpi")); // density qualifier
    }

    #[test]
    fn storage_format_round_trips() {
        let loc = Locale::new("zh", "CN");
        assert_eq!(loc.to_storage(), "zh$CN");
        assert_eq!(Locale::from_storage("zh$CN"), Some(loc));
        assert_eq!(Locale::from_storage("ru$"), Some(Locale::base("ru")));
        assert_eq!(Locale::from_storage(""), None);
        assert_eq!(Locale::from_storage("zh$CN$TW"), None);
        assert!(valid_storage("en$US"));
        assert!(!valid_storage("en-US"));
    }

    #[test]
    fn primary_locale_matches_reference_branches() {
        assert_eq!(PrimaryLocale::from_tag("zh-rCN"), PrimaryLocale::Chinese);
        assert_eq!(PrimaryLocale::from_tag("ja"), PrimaryLocale::Japanese);
        assert_eq!(PrimaryLocale::from_tag("ru"), PrimaryLocale::Russian);
        assert_eq!(PrimaryLocale::from_tag("de"), PrimaryLocale::Other);
        assert_eq!(PrimaryLocale::Chinese.display_name(), Some("简体中文"));
        assert_eq!(PrimaryLocale::Other.display_name(), None);
    }
}
