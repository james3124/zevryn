//! Rust model of the portable part of `com.winemu.core.Container`
//! (GameHub Lite v5.1.8).
//!
//! A container is a path plus a `BottleMetadata` loaded from `metadata.json`.
//! The Gson JSON (de)serialization and the `patch_version.txt` file I/O need a
//! filesystem and a JSON library; only the plain data shape and the pure
//! string-parsing of the patch version are modeled here.

/// Portable subset of `BottleMetadata` referenced by `Container`.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct BottleMetadata {
    /// `getWinePath()` used by `Container.d()`.
    pub wine_path: Option<String>,
}

/// A Wine container, as a plain data struct.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Container {
    /// `f20201a` — the container directory path (`a()`).
    pub path: String,
    /// `f20202b` — bottle metadata (`b()`).
    pub metadata: BottleMetadata,
}

impl Container {
    pub fn new(path: impl Into<String>) -> Self {
        Self {
            path: path.into(),
            metadata: BottleMetadata::default(),
        }
    }

    /// `d()` — the Wine path recorded in the metadata.
    pub fn wine_path(&self) -> Option<&str> {
        self.metadata.wine_path.as_deref()
    }

    /// Names of the two files a container persists, relative to `path`.
    pub const METADATA_FILE: &'static str = "metadata.json";
    pub const PATCH_VERSION_FILE: &'static str = "patch_version.txt";

    /// Pure equivalent of `Container.c()`: parse the trimmed content of
    /// `patch_version.txt` into an integer (Kotlin `toIntOrNull`), returning
    /// `None` for missing/invalid content.
    pub fn parse_patch_version(text: &str) -> Option<i32> {
        text.trim().parse::<i32>().ok()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn container_data_shape() {
        let mut c = Container::new("/containers/game");
        assert_eq!(c.path, "/containers/game");
        assert_eq!(c.wine_path(), None);
        c.metadata.wine_path = Some("/wine".into());
        assert_eq!(c.wine_path(), Some("/wine"));
        assert_eq!(Container::METADATA_FILE, "metadata.json");
        assert_eq!(Container::PATCH_VERSION_FILE, "patch_version.txt");
    }

    #[test]
    fn patch_version_parsing() {
        assert_eq!(Container::parse_patch_version("42"), Some(42));
        assert_eq!(Container::parse_patch_version(" 7 \n"), Some(7));
        assert_eq!(Container::parse_patch_version("abc"), None);
        assert_eq!(Container::parse_patch_version(""), None);
    }
}
