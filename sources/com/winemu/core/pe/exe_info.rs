//! Rust model of `com.winemu.core.ExeFileInfo` and its `Info` bean
//! (GameHub Lite v5.1.8).
//!
//! The decompiled class reads a PE file with pecoff4j, extracts the largest
//! <=32px icon from RT_GROUP_ICON/RT_ICON resources and parses the RT_VERSION
//! string table. Its `b()` method maps seven `StringPair` keys into the `Info`
//! bean:
//! `ProductVersion`, `FileDescription`, `FileVersion`, `InternalName`,
//! `ProductName`, `CompanyName`, `OriginalFilename`.
//!
//! Here only the data model and the pure key-mapping are reproduced. PE parsing,
//! resource extraction and Android `Bitmap` creation are out of scope.

use super::icon_image::PeIcon;

/// Projection of `ExeFileInfo.Info` — version-resource string fields.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ExeInfo {
    /// `FileDescription` (`f20245a`, `d()` / `a()`).
    pub file_description: Option<String>,
    /// `CompanyName` (`f20246b`, `c()`).
    pub company: Option<String>,
    /// `FileVersion` (`f20247c`, `e()`).
    pub file_version: Option<String>,
    /// `InternalName` (`f20248d`, `f()`).
    pub internal_name: Option<String>,
    /// `OriginalFilename` (`f20249e`, `g()`).
    pub original_filename: Option<String>,
    /// `ProductName` (`f20250f`, `h()` / `b()`).
    pub product_name: Option<String>,
    /// `ProductVersion` (`f20251g`, `i()`).
    pub product_version: Option<String>,
}

impl ExeInfo {
    /// Port of the switch in `ExeFileInfo.b()`: unknown keys are ignored.
    pub fn from_version_pairs<'a, I>(pairs: I) -> Self
    where
        I: IntoIterator<Item = (&'a str, &'a str)>,
    {
        let mut info = ExeInfo::default();
        for (key, value) in pairs {
            match key {
                "FileDescription" => info.file_description = Some(value.to_owned()),
                "CompanyName" => info.company = Some(value.to_owned()),
                "FileVersion" => info.file_version = Some(value.to_owned()),
                "InternalName" => info.internal_name = Some(value.to_owned()),
                "OriginalFilename" => info.original_filename = Some(value.to_owned()),
                "ProductName" => info.product_name = Some(value.to_owned()),
                "ProductVersion" => info.product_version = Some(value.to_owned()),
                _ => {}
            }
        }
        info
    }

    pub fn is_empty(&self) -> bool {
        self.file_description.is_none()
            && self.company.is_none()
            && self.file_version.is_none()
            && self.internal_name.is_none()
            && self.original_filename.is_none()
            && self.product_name.is_none()
            && self.product_version.is_none()
    }
}

/// Projection of `ExeFileInfo`: fields `f20241a` (path), `f20242b` (icon
/// bitmap), `f20243c` (`peA.i()`), `f20244d` (VersionInfo).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExeFileInfo {
    pub path: String,
    /// `c()` — the boolean returned by `PE.i()` (PE32+ vs PE32).
    pub is_64bit: bool,
    /// `a()` — a `Bitmap` in Java; here the header-derived icon model.
    pub icon: Option<PeIcon>,
    /// `b()` — the mapped version-resource bean.
    pub info: Option<ExeInfo>,
}

impl ExeFileInfo {
    pub fn new(path: impl Into<String>) -> Self {
        Self { path: path.into(), is_64bit: false, icon: None, info: None }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn maps_all_seven_version_keys() {
        let pairs = [
            ("FileDescription", "Game executable"),
            ("CompanyName", "Example Co"),
            ("FileVersion", "1.2.3.4"),
            ("InternalName", "game"),
            ("OriginalFilename", "game.exe"),
            ("ProductName", "Example Game"),
            ("ProductVersion", "1.2"),
            ("UnsupportedKey", "ignored"),
        ];
        let info = ExeInfo::from_version_pairs(pairs);
        assert_eq!(info.file_description.as_deref(), Some("Game executable"));
        assert_eq!(info.company.as_deref(), Some("Example Co"));
        assert_eq!(info.file_version.as_deref(), Some("1.2.3.4"));
        assert_eq!(info.internal_name.as_deref(), Some("game"));
        assert_eq!(info.original_filename.as_deref(), Some("game.exe"));
        assert_eq!(info.product_name.as_deref(), Some("Example Game"));
        assert_eq!(info.product_version.as_deref(), Some("1.2"));
        assert!(!info.is_empty());
    }

    #[test]
    fn unknown_keys_leave_default_info() {
        let info = ExeInfo::from_version_pairs([("Foo", "bar")]);
        assert!(info.is_empty());
    }

    #[test]
    fn exe_file_info_defaults() {
        let exe = ExeFileInfo::new("/tmp/game.exe");
        assert_eq!(exe.path, "/tmp/game.exe");
        assert!(!exe.is_64bit);
        assert!(exe.icon.is_none());
        assert!(exe.info.is_none());
    }
}
