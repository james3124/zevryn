//! Pure string path helpers modeled on `com.winemu.core.utils.PathUtils`,
//! `WinUtils` and `StringUtils` (GameHub Lite v5.1.8).
//!
//! * `PathUtils.a(path)` is Kotlin `path.startsWith("/")` → [`is_absolute`].
//! * `WinUtils.b(drive, path)` is
//!   `drive + "\\" + path.replace("/", "\\")` followed by trimming trailing
//!   backslashes → [`join_windows`].
//! * `StringUtils.a(path)` strips all trailing `/` or `\` →
//!   [`strip_trailing_separators`].
//!
//! The filesystem-dependent `WinUtils.a()` (which calls `toRealPath()` and
//! probes `dosdevices` drive symlinks) needs a real Wine prefix and is out of
//! scope; only the pure string transforms are kept.

/// `PathUtils.a(path)` — true when `path` starts with `/`.
pub fn is_absolute(path: &str) -> bool {
    path.starts_with('/')
}

/// `WinUtils.b(str, str2)`: convert a Unix path to a backslash Windows path.
/// Forward slashes become backslashes; leading and trailing separators are
/// removed (in Java `str2` is already a substring without a leading `/`, so
/// this normalization keeps the drive-relative form clean).
pub fn to_windows_path(path: &str) -> String {
    trim_backslashes(&path.trim_start_matches(|c| c == '/' || c == '\\').replace('/', "\\"))
}

/// The inverse of [`to_windows_path`]: backslashes become forward slashes.
pub fn to_unix_path(path: &str) -> String {
    path.replace('\\', "/")
}

/// Join a drive prefix (e.g. `c:`) and a Unix path exactly as
/// `WinUtils.b(drive, path)` does: `drive + "\\" + path(backslash-converted)`,
/// then strip trailing backslashes.
///
/// ```text
/// assert!(is_absolute("/home/user/games"));
/// assert_eq!(to_windows_path("/a/b/"), "a\\b");
/// assert_eq!(to_unix_path("c:\\games\\game.exe"), "c:/games/game.exe");
/// assert_eq!(join_windows("c:", "/Games/doom/"), "c:\\Games\\doom");
/// ```
pub fn join_windows(drive: &str, path: &str) -> String {
    let converted = path.trim_start_matches(|c| c == '/' || c == '\\').replace('/', "\\");
    let joined = format!("{}\\{}", drive, converted);
    trim_backslashes(&joined)
}

/// Generic [`join`]: append `child` to `base` using `/` for Unix-like bases and
/// `\` for Windows-style bases, collapsing a redundant separator at the seam.
/// Each input has its trailing separators stripped first (`StringUtils.a`).
pub fn join(base: &str, child: &str) -> String {
    let base = strip_trailing_separators(base);
    let child = child.trim_start_matches(|c| c == '/' || c == '\\');
    if base.is_empty() {
        return child.to_owned();
    }
    // A drive letter or any backslash marks a Windows-style base.
    let sep = if base.contains('\\')
        || (base.len() == 2 && base.ends_with(':'))
    {
        '\\'
    } else {
        '/'
    };
    format!("{}{}{}", base, sep, child)
}

/// `StringUtils.a(str)` — repeatedly drop trailing `/` or `\`.
pub fn strip_trailing_separators(s: &str) -> &str {
    s.trim_end_matches(|c| c == '/' || c == '\\')
}

fn trim_backslashes(s: &str) -> String {
    s.trim_end_matches('\\').to_owned()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn path_utils_is_absolute() {
        assert!(is_absolute("/"));
        assert!(is_absolute("/home/user"));
        assert!(!is_absolute("home/user"));
        assert!(!is_absolute("c:\\games"));
        assert!(!is_absolute(""));
    }

    #[test]
    fn to_windows_and_unix_paths() {
        assert_eq!(to_windows_path("/a/b/c"), "a\\b\\c");
        assert_eq!(to_windows_path("/a/b/c///"), "a\\b\\c");
        assert_eq!(to_windows_path("a/b\\c"), "a\\b\\c");
        assert_eq!(to_unix_path("c:\\games\\game.exe"), "c:/games/game.exe");
        assert_eq!(to_unix_path("/mnt/c/games"), "/mnt/c/games");
        // round trip
        assert_eq!(to_unix_path(&to_windows_path("/a/b/c")), "a/b/c");
    }

    #[test]
    fn win_utils_join_drive_and_path() {
        assert_eq!(join_windows("c:", "/Games/doom/"), "c:\\Games\\doom");
        assert_eq!(join_windows("d:", "/apps/foo.exe"), "d:\\apps\\foo.exe");
        assert_eq!(join_windows("z:", "///"), "z:");
    }

    #[test]
    fn generic_join_handles_separators() {
        assert_eq!(join("/home/user/", "/games"), "/home/user/games");
        assert_eq!(join("c:\\games\\", "doom.exe"), "c:\\games\\doom.exe");
        assert_eq!(join("c:", "/Games"), "c:\\Games");
        assert_eq!(join("", "child"), "child");
    }

    #[test]
    fn strip_trailing_separators_string_utils() {
        assert_eq!(strip_trailing_separators("/a/b///"), "/a/b");
        assert_eq!(strip_trailing_separators("c:\\a\\b\\"), "c:\\a\\b");
        assert_eq!(strip_trailing_separators("plain"), "plain");
        assert_eq!(strip_trailing_separators("///"), "");
    }
}
