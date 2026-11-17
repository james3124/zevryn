//! Rust model of `com.winemu.core.regedit.Architecture` (GameHub Lite v5.1.8).
//!
//! The reference type is a plain Kotlin enum with three entries: `UNKNOWN`,
//! `WIN32`, `WIN64`. The `.reg` parser reads it from a `#arch=win32|win64`
//! directive and the writer emits it back as the same directive. No Android or
//! JNI state is involved, so the port is dependency-free.

use std::fmt;

/// Target architecture recorded in a Wine `.reg` file.
///
/// Mirrors the entries of the Java/Kotlin `Architecture` enum. `Unknown` is
/// the default used by `RegistryParser` when no `#arch=` line is present.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum Architecture {
    #[default]
    Unknown,
    Win32,
    Win64,
}

impl Architecture {
    /// Lowercase token written after `#arch=` by `RegistryWriter`.
    pub fn as_str(self) -> &'static str {
        match self {
            Architecture::Unknown => "unknown",
            Architecture::Win32 => "win32",
            Architecture::Win64 => "win64",
        }
    }

    /// Parse the token following `#arch=`. Matching is case-insensitive and
    /// tolerates surrounding whitespace, mirroring the lenient decompiled
    /// comparison (`win32` / `win64`, otherwise an error).
    pub fn parse(token: &str) -> Option<Self> {
        match token.trim().to_ascii_lowercase().as_str() {
            "win32" => Some(Architecture::Win32),
            "win64" => Some(Architecture::Win64),
            "unknown" => Some(Architecture::Unknown),
            _ => None,
        }
    }

    /// Word size implied by the architecture (`Unknown` assumes 64-bit, the
    /// same default Wine uses when no explicit arch is given).
    pub fn pointer_width(self) -> u32 {
        match self {
            Architecture::Win32 | Architecture::Unknown => 4,
            Architecture::Win64 => 8,
        }
    }
}

impl fmt::Display for Architecture {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

#[cfg(test)]
mod tests {
    use super::Architecture;

    #[test]
    fn default_is_unknown() {
        assert_eq!(Architecture::default(), Architecture::Unknown);
    }

    #[test]
    fn parse_tokens() {
        assert_eq!(Architecture::parse("win32"), Some(Architecture::Win32));
        assert_eq!(Architecture::parse(" WIN64 "), Some(Architecture::Win64));
        assert_eq!(Architecture::parse("unknown"), Some(Architecture::Unknown));
        assert_eq!(Architecture::parse("arm64"), None);
    }

    #[test]
    fn as_str_and_width() {
        assert_eq!(Architecture::Win32.as_str(), "win32");
        assert_eq!(Architecture::Win64.to_string(), "win64");
        assert_eq!(Architecture::Win32.pointer_width(), 4);
        assert_eq!(Architecture::Win64.pointer_width(), 8);
    }
}
