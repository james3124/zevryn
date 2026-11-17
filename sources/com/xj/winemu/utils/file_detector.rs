//! Pure string classification of files encountered by the portable import
//! pipeline.
//!
//! The manifest rule is transcribed from the file filters recovered in
//! `ImportPcGameHelper$readAppManifestFilesAndImportGame$1.java:298-311` and
//! `ImportPcGameHelper$checkPermissionAndStartRead$2.java:137-150`:
//!
//! ```text
//! if (!file.isFile()) return false;
//! name = file.getName();
//! if (!name.startsWith("appmanifest_", ignoreCase = false)) return false;
//! return name.endsWith(".json", ignoreCase = false);
//! ```
//!
//! (`StringsKt__StringsJVMKt.X` is `startsWith`, `H` is `endsWith`; the `false`
//! argument is `ignoreCase`, so both checks are case-sensitive.)
//!
//! GameHub Lite stores its own manifests as `appmanifest_<id>.json`, but the
//! pipeline also accepts native Steam `appmanifest_<id>.acf` files; both
//! extensions are recognised here. The `isFile()` check needs real filesystem
//! access and is deliberately out of scope (see `REPORT.md`).

/// How a single path is treated by the import pipeline.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum FileKind {
    /// A Steam/GameHub app manifest (`appmanifest_*`).
    Manifest,
    /// A Windows executable (`*.exe`).
    Exe,
    /// Anything else (directories, other file types, unknown names).
    Ignored,
}

impl FileKind {
    pub const fn as_str(self) -> &'static str {
        match self {
            FileKind::Manifest => "Manifest",
            FileKind::Exe => "Exe",
            FileKind::Ignored => "Ignored",
        }
    }
}

impl std::fmt::Display for FileKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// Exact prefix from `ImportPcGameHelper.m/v` and the recovered filters.
pub const MANIFEST_PREFIX: &str = "appmanifest_";
/// GameHub Lite's own manifest extension (recovered from the decompiled code).
pub const MANIFEST_JSON_SUFFIX: &str = ".json";
/// Native Steam manifest extension (also accepted by this model).
pub const MANIFEST_ACF_SUFFIX: &str = ".acf";
/// Windows executable extension.
pub const EXE_SUFFIX: &str = ".exe";

/// Extract the file name from a path, accepting both `/` and `\` separators.
///
/// A trailing separator yields an empty name (there is no file component).
pub fn basename(path: &str) -> &str {
    path.rsplit(['/', '\\']).next().unwrap_or(path)
}

/// `name.startsWith("appmanifest_")` (case-sensitive).
pub fn has_manifest_prefix(name: &str) -> bool {
    name.starts_with(MANIFEST_PREFIX)
}

/// `name.endsWith(".json") || name.endsWith(".acf")` (case-sensitive).
pub fn has_manifest_suffix(name: &str) -> bool {
    name.ends_with(MANIFEST_JSON_SUFFIX) || name.ends_with(MANIFEST_ACF_SUFFIX)
}

/// The exact manifest-file predicate recovered from
/// `readAppManifestFilesAndImportGame$1.j(...)` (minus the `isFile()` call).
///
/// The reference applies this to `file.getName()`; [`basename`] is applied
/// here so a full path is accepted too. Like the reference, this is a pure
/// prefix/suffix check — it does not require a non-empty app id, so
/// `appmanifest_.acf` still passes the filter.
pub fn is_manifest_file(name: &str) -> bool {
    let name = basename(name);
    has_manifest_prefix(name) && has_manifest_suffix(name)
}

/// Case-insensitive `.exe` suffix test (Windows executables are commonly
/// spelled `GAME.EXE`). No equivalent check was recoverable from the
/// decompiled body of `readAppManifestFilesAndImportGame` (its `invokeSuspend`
/// is a JADX dump stub), so this is a pragmatic extension of the string rule.
pub fn is_exe_file(name: &str) -> bool {
    name.len() >= EXE_SUFFIX.len()
        && name.get(name.len() - EXE_SUFFIX.len()..).is_some_and(|s| s.eq_ignore_ascii_case(EXE_SUFFIX))
}

/// Extract the Steam app id from a manifest file name.
///
/// `appmanifest_440.acf` -> `Some("440")`; `notes.json` -> `None`.
/// Mirrors `ImportPcGameHelper.m(str)`, which builds the name as
/// `"appmanifest_" + appId + ".json"` (line 221).
pub fn manifest_steam_app_id(name: &str) -> Option<&str> {
    let name = basename(name);
    if !is_manifest_file(name) {
        return None;
    }
    let body = name.strip_prefix(MANIFEST_PREFIX)?;
    let id = body
        .strip_suffix(MANIFEST_JSON_SUFFIX)
        .or_else(|| body.strip_suffix(MANIFEST_ACF_SUFFIX))?;
    if id.is_empty() {
        None
    } else {
        Some(id)
    }
}

/// Classify a path by its file name. Manifest names take precedence over exe
/// names; unrecognised names are [`FileKind::Ignored`].
pub fn classify(path: &str) -> FileKind {
    let name = basename(path);
    if is_manifest_file(name) {
        FileKind::Manifest
    } else if is_exe_file(name) {
        FileKind::Exe
    } else {
        FileKind::Ignored
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn basename_handles_both_separators() {
        assert_eq!(basename("/steam/steamapps/appmanifest_440.acf"), "appmanifest_440.acf");
        assert_eq!(basename(r"C:\Games\GAME.EXE"), "GAME.EXE");
        assert_eq!(basename("appmanifest_10.json"), "appmanifest_10.json");
        assert_eq!(basename("dir/"), "");
    }

    #[test]
    fn manifest_detection_is_case_sensitive_and_prefix_based() {
        assert!(is_manifest_file("appmanifest_440.acf"));
        assert!(is_manifest_file("appmanifest_440.json"));
        assert!(!is_manifest_file("AppManifest_440.acf"));
        assert!(!is_manifest_file("appmanifest_440.ACF"));
        assert!(!is_manifest_file("my_appmanifest_440.acf"));
        assert!(!is_manifest_file("appmanifest_440.txt"));
        // The recovered filter is only startsWith + endsWith, so an empty id
        // still passes; the id extractor below is stricter.
        assert!(is_manifest_file("appmanifest_.acf"));
        assert!(is_manifest_file("/x/appmanifest_440.acf"));
    }

    #[test]
    fn exe_detection_is_extension_case_insensitive() {
        assert!(is_exe_file("game.exe"));
        assert!(is_exe_file("GAME.EXE"));
        assert!(!is_exe_file("game.exe.bak"));
        assert!(!is_exe_file("exe"));
        assert!(!is_exe_file(""));
    }

    #[test]
    fn classifies_paths() {
        assert_eq!(classify("/s/steamapps/appmanifest_570.acf"), FileKind::Manifest);
        assert_eq!(classify(r"D:\Games\game.EXE"), FileKind::Exe);
        assert_eq!(classify("/s/steamapps/libraryfolders.vdf"), FileKind::Ignored);
        assert_eq!(classify("/s/Games/"), FileKind::Ignored);
    }

    #[test]
    fn extracts_steam_app_id() {
        assert_eq!(manifest_steam_app_id("appmanifest_570.acf"), Some("570"));
        assert_eq!(manifest_steam_app_id("/x/appmanifest_10.json"), Some("10"));
        assert_eq!(manifest_steam_app_id("appmanifest_.json"), None);
        assert_eq!(manifest_steam_app_id("readme.txt"), None);
    }
}
