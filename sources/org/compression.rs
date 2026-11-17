//! Portable model of the compression / archive formats shipped by GameHub Lite
//! v5.1.8 through `org.apache.commons.compress`, `org.tukaani` (XZ / LZMA) and
//! `net.lingala.zip4j` (ZIP).
//!
//! These are third-party Java libraries. They are **not reimplemented** here;
//! this module only records the format inventory and the Rust crates that
//! would be used to replace them. See `apktool/lib/REPORT.md`.

use std::fmt;

/// Archive / compression formats present in the reference source tree.
///
/// Kept as a closed enum so callers can match exhaustively on the exact set of
/// formats the original application supports.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum CompressionFormat {
    /// ZIP containers (also encrypted ZIP via zip4j).
    Zip,
    /// 7-Zip containers (`.7z`).
    SevenZip,
    /// POSIX / ustar tape archives (`.tar`).
    Tar,
    /// gzip streams (`.gz`).
    Gzip,
    /// XZ / LZMA2 streams (`.xz`, `.lzma`) from the tukaani implementation.
    Xz,
    /// Zstandard streams (`.zst`) — also shipped natively as `libzstd-jni`.
    Zstd,
    /// bzip2 streams (`.bz2`).
    Bzip2,
}

impl CompressionFormat {
    /// Every format, in a stable order.
    pub const ALL: [CompressionFormat; 7] = [
        CompressionFormat::Zip,
        CompressionFormat::SevenZip,
        CompressionFormat::Tar,
        CompressionFormat::Gzip,
        CompressionFormat::Xz,
        CompressionFormat::Zstd,
        CompressionFormat::Bzip2,
    ];

    /// Canonical short name of the format.
    pub const fn name(self) -> &'static str {
        match self {
            CompressionFormat::Zip => "zip",
            CompressionFormat::SevenZip => "7z",
            CompressionFormat::Tar => "tar",
            CompressionFormat::Gzip => "gzip",
            CompressionFormat::Xz => "xz",
            CompressionFormat::Zstd => "zstd",
            CompressionFormat::Bzip2 => "bzip2",
        }
    }

    /// Suggested Rust crate that replaces the corresponding Java implementation.
    pub const fn suggested_crate(self) -> &'static str {
        match self {
            CompressionFormat::Zip => "zip",
            CompressionFormat::SevenZip => "sevenz-rust",
            CompressionFormat::Tar => "tar",
            CompressionFormat::Gzip => "flate2",
            CompressionFormat::Xz => "xz2",
            CompressionFormat::Zstd => "zstd",
            CompressionFormat::Bzip2 => "bzip2",
        }
    }

    /// Original Java package(s) that provided this format in the reference.
    pub const fn origin(self) -> &'static str {
        match self {
            CompressionFormat::Zip => "net.lingala.zip4j / org.apache.commons.compress.archivers.zip",
            CompressionFormat::SevenZip => "org.apache.commons.compress.archivers.sevenz",
            CompressionFormat::Tar => "org.apache.commons.compress.archivers.tar",
            CompressionFormat::Gzip => "org.apache.commons.compress.compressors.gzip",
            CompressionFormat::Xz => "org.tukaani.xz / org.apache.commons.compress.compressors.xz",
            CompressionFormat::Zstd => "org.apache.commons.compress.compressors.zstandard",
            CompressionFormat::Bzip2 => "org.apache.commons.compress.compressors.bzip2",
        }
    }
}

impl fmt::Display for CompressionFormat {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.name())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn models_all_seven_formats() {
        assert_eq!(CompressionFormat::ALL.len(), 7);
        let mut unique = std::collections::BTreeSet::new();
        for fmt in CompressionFormat::ALL {
            assert!(unique.insert(fmt));
            assert!(!fmt.name().is_empty());
            assert!(!fmt.suggested_crate().is_empty());
            assert!(!fmt.origin().is_empty());
        }
    }

    #[test]
    fn crate_mapping_matches_expected_crates() {
        let expected = [
            "zip",
            "sevenz-rust",
            "tar",
            "flate2",
            "xz2",
            "zstd",
            "bzip2",
        ];
        for (fmt, want) in CompressionFormat::ALL.iter().zip(expected.iter()) {
            assert_eq!(fmt.suggested_crate(), *want);
        }
    }

    #[test]
    fn display_matches_name() {
        for fmt in CompressionFormat::ALL {
            assert_eq!(fmt.to_string(), fmt.name());
        }
        assert_eq!(CompressionFormat::SevenZip.to_string(), "7z");
    }
}
