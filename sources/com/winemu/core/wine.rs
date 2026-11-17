//! Rust model of `com.winemu.core.Wine` (GameHub Lite v5.1.8).
//!
//! `Wine` is a small value object: a path, a version string and a
//! [`WineArch`]. The only portable behavior is architecture-dependent path
//! selection (`a()`) and the arch predicates (`d()`/`e()`). Filesystem checks
//! (`c()` → `libarm64ecfex.dll` existence) require a real Wine tree and are out
//! of scope; the path they would inspect is exposed via `lib_dir()`.

/// Wine architecture selector (`Wine.WineArch`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum WineArch {
    X86,
    Arm64X,
}

/// Portable Wine descriptor.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Wine {
    pub path: String,
    pub version: String,
    pub arch: WineArch,
}

impl Wine {
    pub fn new(path: impl Into<String>, version: impl Into<String>, arch: WineArch) -> Self {
        Self {
            path: path.into(),
            version: version.into(),
            arch,
        }
    }

    /// `b()` — the Wine root path.
    pub fn root(&self) -> &str {
        &self.path
    }

    /// `d()` — true for `Arm64X`.
    pub fn is_arm64x(&self) -> bool {
        self.arch == WineArch::Arm64X
    }

    /// `e()` — true for `X86`.
    pub fn is_x86(&self) -> bool {
        self.arch == WineArch::X86
    }

    /// `a()` — the Wine library directory suffix for this architecture.
    pub fn lib_dir(&self) -> String {
        if self.is_arm64x() {
            format!("{}/arm64-v8a/lib/wine/aarch64-windows/", self.path)
        } else {
            format!("{}/x86_64/lib/wine/x86_64-windows/", self.path)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn arch_paths() {
        let arm = Wine::new("/wine", "9.0", WineArch::Arm64X);
        assert!(arm.is_arm64x());
        assert!(!arm.is_x86());
        assert_eq!(arm.root(), "/wine");
        assert_eq!(
            arm.lib_dir(),
            "/wine/arm64-v8a/lib/wine/aarch64-windows/"
        );

        let x86 = Wine::new("/wine", "9.0", WineArch::X86);
        assert!(x86.is_x86());
        assert_eq!(x86.lib_dir(), "/wine/x86_64/lib/wine/x86_64-windows/");
    }
}
