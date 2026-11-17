//! Rust model of `defpackage.EmbeddedFileManager` / `defpackage.RunEmbedded`
//! (GameHub Lite v5.1.8).
//!
//! The Java class caches three Base64-backed PE executables (`run.exe`,
//! `tabtip.exe`, `winmonitor.exe`) in the cache dir under a
//! `<name>_v<version>` filename, deleting older cached versions on write.
//! The actual PE payloads (the Base64 strings in `RunEmbedded`,
//! `TabtipEmbedded`, `WinMonitorEmbedded`) are native Windows binaries and are
//! intentionally **not** embedded; only the portable naming/version logic and
//! the cache-eviction predicate are modeled here.
//!
//! Kept name-for-name compatible with the core crate's `src/emu.rs`.

/// The embedded PE executables written by `EmbeddedFileManager`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum EmbeddedFile {
    RunExe,
    TabtipExe,
    WinMonitorExe,
}

impl EmbeddedFile {
    /// All embedded files, in the order exposed by `EmbeddedFileManager`.
    pub const ALL: [Self; 3] = [Self::RunExe, Self::TabtipExe, Self::WinMonitorExe];

    /// Base filename passed to `EmbeddedFileManager.b()`.
    pub fn filename(self) -> &'static str {
        match self {
            Self::RunExe => "run.exe",
            Self::TabtipExe => "tabtip.exe",
            Self::WinMonitorExe => "winmonitor.exe",
        }
    }

    /// Version token passed as `str2` to `EmbeddedFileManager.b()`.
    pub fn version(self) -> &'static str {
        match self {
            Self::RunExe => "16045985",
            Self::TabtipExe => "19508713",
            Self::WinMonitorExe => "19585099",
        }
    }

    /// The `<name>_v<version>` cache filename
    /// (`str + "_v" + str2` in `EmbeddedFileManager.b()`).
    pub fn cached_name(self) -> String {
        format!("{}_v{}", self.filename(), self.version())
    }
}

/// True when a cache entry belongs to `base` and should be evicted by
/// `EmbeddedFileManager.a(String)` (its `name.startsWith(str + "_v", true)`).
pub fn is_cached_version(name: &str, base: &str) -> bool {
    let prefix = format!("{}_v", base);
    name.len() >= prefix.len() && name[..prefix.len()].eq_ignore_ascii_case(&prefix)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn embedded_file_names() {
        assert_eq!(EmbeddedFile::RunExe.cached_name(), "run.exe_v16045985");
        assert_eq!(EmbeddedFile::TabtipExe.cached_name(), "tabtip.exe_v19508713");
        assert_eq!(
            EmbeddedFile::WinMonitorExe.cached_name(),
            "winmonitor.exe_v19585099"
        );
    }

    #[test]
    fn versions_are_distinct_and_nonempty() {
        let versions: Vec<_> = EmbeddedFile::ALL.iter().map(|f| f.version()).collect();
        assert_eq!(versions.len(), 3);
        for v in &versions {
            assert!(!v.is_empty());
        }
        assert!(versions.windows(2).all(|w| w[0] != w[1]));
    }

    #[test]
    fn cache_eviction_predicate() {
        assert!(is_cached_version("run.exe_v16045985", "run.exe"));
        assert!(is_cached_version("run.exe_v1", "run.exe"));
        assert!(is_cached_version("TABTIP.EXE_v9", "tabtip.exe"));
        assert!(!is_cached_version("run.exe", "run.exe"));
        assert!(!is_cached_version("other.exe_v1", "run.exe"));
    }
}
