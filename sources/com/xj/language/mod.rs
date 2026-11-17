//! Rust model of the `com.xj.language` / `com.xj.base.language` resource layer.
//!
//! Mirrors the portable parts of `GHLocaleManager` plus the Android
//! `values/strings.xml` resources it reads. All modules are dependency-free
//! (std only) and live at the Java-mirrored path, to be wired from
//! `app/src/lib.rs` with a `#[path]` module declaration.

pub mod locale;
pub mod strings_catalog;
pub mod strings_xml;

pub use locale::{
    language_aliases, resolve, valid_storage, Locale, PrimaryLocale, SupportedLocales,
    BASE_LOCALES, DEFAULT_LOCALE, REGIONAL_LOCALES,
};
pub use strings_catalog::StringsCatalog;
pub use strings_xml::{decode as decode_xml_text, parse as parse_strings_xml, ParseError};
