//! In-memory catalog of Android string resources with locale fallback.
//!
//! Resource lookup follows Android's usual chain: the most specific locale
//! that was loaded, then its base language, then the base (`values/`) catalog.
//! All parsing is done up front by [`strings_xml::parse`].

use std::collections::HashMap;

use super::locale::{language_aliases, Locale};
use super::strings_xml::{self, ParseError};

/// A set of string resources: one base catalog plus optional locale catalogs.
#[derive(Debug, Default, Clone)]
pub struct StringsCatalog {
    base: HashMap<String, String>,
    locales: HashMap<String, HashMap<String, String>>,
}

impl StringsCatalog {
    pub fn new() -> Self {
        Self::default()
    }

    /// Build a catalog from the base (`values/strings.xml`) document.
    pub fn from_base(xml: &str) -> Result<Self, ParseError> {
        Ok(Self {
            base: strings_xml::parse(xml)?,
            locales: HashMap::new(),
        })
    }

    /// Register (or replace) a locale's catalog.
    ///
    /// `tag` is the resource qualifier, e.g. `"zh-rCN"` or `"en"`. The same
    /// map is also reachable through the locale's base language so that a
    /// request for `"zh-rTW"` falls back to a loaded `"zh"` catalog.
    pub fn add_locale(&mut self, tag: &str, xml: &str) -> Result<(), ParseError> {
        let map = strings_xml::parse(xml)?;
        self.locales.insert(tag.to_string(), map);
        Ok(())
    }

    /// Builder-style variant of [`StringsCatalog::add_locale`].
    pub fn with_locale(mut self, tag: &str, xml: &str) -> Result<Self, ParseError> {
        self.add_locale(tag, xml)?;
        Ok(self)
    }

    /// Look up `key` for an optional locale, falling back to the base catalog.
    pub fn lookup<'a>(&'a self, key: &str, locale: Option<&str>) -> Option<&'a str> {
        let Some(tag) = locale else {
            return self.base.get(key).map(String::as_str);
        };
        if let Some(value) = self.locales.get(tag).and_then(|m| m.get(key)) {
            return Some(value.as_str());
        }
        let parsed = Locale::parse_tag(tag);
        let base_language = parsed.as_ref().map(|l| l.language.as_str());
        if let Some(lang) = base_language {
            if let Some(value) = self.locales.get(lang).and_then(|m| m.get(key)) {
                return Some(value.as_str());
            }
            for alias in language_aliases(lang) {
                if let Some(value) = self.locales.get(*alias).and_then(|m| m.get(key)) {
                    return Some(value.as_str());
                }
            }
        }
        self.base.get(key).map(String::as_str)
    }

    /// Look up a key in the base catalog only.
    pub fn get(&self, key: &str) -> Option<&str> {
        self.base.get(key).map(String::as_str)
    }

    pub fn locale_count(&self) -> usize {
        self.locales.len()
    }

    pub fn len(&self) -> usize {
        self.base.len()
    }

    pub fn is_empty(&self) -> bool {
        self.base.is_empty()
    }

    pub fn has_locale(&self, tag: &str) -> bool {
        self.locales.contains_key(tag)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const BASE: &str = r#"
<resources>
    <string name="app_name">GameHub Lite</string>
    <string name="action_settings">Settings</string>
    <string name="only_in_base">base value</string>
</resources>"#;

    const ZH: &str = r#"
<resources>
    <string name="app_name">游戏中心</string>
    <string name="action_settings">设置</string>
</resources>"#;

    const ZH_CN: &str = r#"
<resources>
    <string name="app_name">游戏中心 简体</string>
</resources>"#;

    #[test]
    fn base_lookup_and_missing_key() {
        let catalog = StringsCatalog::from_base(BASE).unwrap();
        assert_eq!(catalog.lookup("app_name", None), Some("GameHub Lite"));
        assert_eq!(catalog.get("action_settings"), Some("Settings"));
        assert_eq!(catalog.lookup("nope", None), None);
        assert_eq!(catalog.len(), 3);
        assert!(catalog.is_empty() == false);
    }

    #[test]
    fn locale_lookup_falls_back_to_base() {
        let catalog = StringsCatalog::from_base(BASE)
            .unwrap()
            .with_locale("zh", ZH)
            .unwrap();

        assert_eq!(catalog.lookup("app_name", Some("zh")), Some("游戏中心"));
        // key absent from zh -> base
        assert_eq!(
            catalog.lookup("only_in_base", Some("zh")),
            Some("base value")
        );
        // unknown locale -> base
        assert_eq!(catalog.lookup("app_name", Some("de")), Some("GameHub Lite"));
    }

    #[test]
    fn region_falls_back_to_base_language() {
        let catalog = StringsCatalog::from_base(BASE)
            .unwrap()
            .with_locale("zh", ZH)
            .unwrap()
            .with_locale("zh-rCN", ZH_CN)
            .unwrap();

        assert_eq!(catalog.lookup("app_name", Some("zh-rCN")), Some("游戏中心 简体"));
        // zh-rCN only overrides app_name: setting comes from the zh catalog.
        assert_eq!(catalog.lookup("action_settings", Some("zh-rCN")), Some("设置"));
        // A different region falls back to the base zh catalog.
        assert_eq!(catalog.lookup("app_name", Some("zh-rTW")), Some("游戏中心"));
        assert_eq!(catalog.locale_count(), 2);
        assert!(catalog.has_locale("zh-rCN"));
    }

    #[test]
    fn legacy_language_alias_is_respected() {
        let catalog = StringsCatalog::from_base(BASE)
            .unwrap()
            .with_locale("in", "<resources><string name=\"app_name\">Indonesia</string></resources>")
            .unwrap();
        assert_eq!(catalog.lookup("app_name", Some("id")), Some("Indonesia"));
    }

    #[test]
    fn reads_real_base_strings_when_available() {
        const PATHS: [&str; 2] = [
            "/home/Project/Zevryn/reference/gamehub-lite-v518-decompiled/gamehub-lite-v518-decompiled/resources/res/values/strings.xml",
            "/home/Project/Zevryn/app/resources/res/values/strings.xml",
        ];
        let xml = PATHS
            .iter()
            .find_map(|p| std::fs::read_to_string(p).ok());
        let Some(xml) = xml else {
            eprintln!("skip: real strings.xml not present; graceful fallback");
            return;
        };
        let catalog = StringsCatalog::from_base(&xml).unwrap();
        assert!(catalog.len() > 3000);
        assert_eq!(catalog.get("app_name"), Some("GameHub Lite"));
        assert!(catalog.lookup("action_settings", Some("zh-rCN")).is_some());
    }
}
