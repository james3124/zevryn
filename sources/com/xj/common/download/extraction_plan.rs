//! Ordered post-download pipeline (download -> unzip -> merge -> verify) and
//! compression-format detection.
//!
//! Behaviour is drawn from the reference:
//! * `com.xj.winemu.download.explorer.UnZipComponent` and
//!   `com.xj.winemu.download.service.UnzipService` unzip a downloaded archive
//!   into a destination directory (8 KiB buffer, `UnzipServiceConfig` default
//!   of 3 retries), reporting progress via `OnUnZipProgressListener`.
//! * `com.xj.standalone.steam.contentdownloader.FileMerger` moves/overwrites
//!   staged files atomically, then verifies them.
//! * `net.lingala.zip4j.headers.HeaderSignature` supplies the ZIP magic values.
//! * Strings `comm_unzipping` ("正在解压中") and `llauncher_pc_game_file_merge`
//!   ("Files are merging") are the resource keys shown for the unzip and merge
//!   steps (`resources/res/values/strings.xml`).
//!
//! No actual I/O, decompression or Android `Service` is implemented here — the
//! plan is a pure description of the ordered stages and the detected format.

use std::fmt;

/// A stage of the environment/game installation pipeline, in execution order.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ExtractionStep {
    /// Download the archive (Aria task).
    Download,
    /// Extract the archive into a staging directory (`comm_unzipping`).
    Unzip,
    /// Move staged files into their final location (`llauncher_pc_game_file_merge`).
    Merge,
    /// Verify names/sizes/hashes against a manifest.
    Verify,
}

impl ExtractionStep {
    /// The canonical order of the pipeline.
    pub const ORDER: [ExtractionStep; 4] = [
        ExtractionStep::Download,
        ExtractionStep::Unzip,
        ExtractionStep::Merge,
        ExtractionStep::Verify,
    ];

    pub const fn as_str(self) -> &'static str {
        match self {
            ExtractionStep::Download => "download",
            ExtractionStep::Unzip => "unzip",
            ExtractionStep::Merge => "merge",
            ExtractionStep::Verify => "verify",
        }
    }

    /// Android string resource key displayed for this stage in the launcher.
    ///
    /// Only unzip and merge have dedicated strings in the reference; the other
    /// stages have no user-facing label and return `None`.
    pub const fn string_key(self) -> Option<&'static str> {
        match self {
            ExtractionStep::Download => None,
            ExtractionStep::Unzip => Some("comm_unzipping"),
            ExtractionStep::Merge => Some("llauncher_pc_game_file_merge"),
            ExtractionStep::Verify => None,
        }
    }

    /// Zero-based position of this stage in [`ExtractionStep::ORDER`].
    pub const fn index(self) -> usize {
        match self {
            ExtractionStep::Download => 0,
            ExtractionStep::Unzip => 1,
            ExtractionStep::Merge => 2,
            ExtractionStep::Verify => 3,
        }
    }

    /// The stage that follows this one, or `None` for the final stage.
    pub const fn next(self) -> Option<ExtractionStep> {
        match self {
            ExtractionStep::Download => Some(ExtractionStep::Unzip),
            ExtractionStep::Unzip => Some(ExtractionStep::Merge),
            ExtractionStep::Merge => Some(ExtractionStep::Verify),
            ExtractionStep::Verify => None,
        }
    }
}

impl fmt::Display for ExtractionStep {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

/// Container/codec formats the downloader may need to unpack.
///
/// A local redefinition (rather than `crate::compression::CompressionFormat`)
/// keeps this module self-contained and adds the magic-byte detection that the
/// inventory enum does not provide.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ArchiveFormat {
    /// ZIP (including encrypted/zip4j and APK/JAR artifacts).
    Zip,
    /// 7-Zip (`.7z`).
    SevenZip,
    /// RAR (`.rar`).
    Rar,
    /// POSIX/ustar tape archive (`.tar`).
    Tar,
    /// gzip (`.gz`, `.tgz`).
    Gzip,
    /// XZ / LZMA2 (`.xz`, `.txz`, `.lzma`).
    Xz,
    /// Zstandard (`.zst`).
    Zstd,
    /// bzip2 (`.bz2`).
    Bzip2,
    /// LZ4 (`.lz4`).
    Lz4,
    /// The format could not be determined from the available data.
    Unknown,
}

impl ArchiveFormat {
    pub const ALL: [ArchiveFormat; 10] = [
        ArchiveFormat::Zip,
        ArchiveFormat::SevenZip,
        ArchiveFormat::Rar,
        ArchiveFormat::Tar,
        ArchiveFormat::Gzip,
        ArchiveFormat::Xz,
        ArchiveFormat::Zstd,
        ArchiveFormat::Bzip2,
        ArchiveFormat::Lz4,
        ArchiveFormat::Unknown,
    ];

    pub const fn name(self) -> &'static str {
        match self {
            ArchiveFormat::Zip => "zip",
            ArchiveFormat::SevenZip => "7z",
            ArchiveFormat::Rar => "rar",
            ArchiveFormat::Tar => "tar",
            ArchiveFormat::Gzip => "gzip",
            ArchiveFormat::Xz => "xz",
            ArchiveFormat::Zstd => "zstd",
            ArchiveFormat::Bzip2 => "bzip2",
            ArchiveFormat::Lz4 => "lz4",
            ArchiveFormat::Unknown => "unknown",
        }
    }

    /// Common file extensions for this format (lowercase, no leading dot).
    pub const fn extensions(self) -> &'static [&'static str] {
        match self {
            ArchiveFormat::Zip => &["zip", "apk", "jar"],
            ArchiveFormat::SevenZip => &["7z"],
            ArchiveFormat::Rar => &["rar"],
            ArchiveFormat::Tar => &["tar"],
            ArchiveFormat::Gzip => &["gz", "tgz"],
            ArchiveFormat::Xz => &["xz", "txz", "lzma"],
            ArchiveFormat::Zstd => &["zst"],
            ArchiveFormat::Bzip2 => &["bz2"],
            ArchiveFormat::Lz4 => &["lz4"],
            ArchiveFormat::Unknown => &[],
        }
    }

    /// Detect a format from a file name or URL extension.
    ///
    /// Query strings and fragment identifiers are ignored, so
    /// `https://x/y.zip?token=1` is detected as ZIP.
    pub fn from_extension(name: &str) -> Option<Self> {
        let lower = name.to_ascii_lowercase();
        let path = lower.split(['?', '#']).next().unwrap_or("");
        let ext = path.rsplit(['/', '\\']).next().unwrap_or("").rsplit('.').next()?;
        if ext.is_empty() || ext.len() == path.len() {
            return None;
        }
        Self::ALL.iter().copied().find(|f| {
            f.extensions().iter().any(|e| *e == ext)
        })
    }

    /// Detect a format from the leading bytes of a file (magic numbers).
    ///
    /// The ZIP signatures are the little-endian encodings of the values in
    /// zip4j's `HeaderSignature` (0x04034B50, 0x05064B50, 0x08074B50).
    /// TAR is identified by the `ustar` marker at offset 257.
    pub fn from_magic(bytes: &[u8]) -> Option<Self> {
        if bytes.starts_with(&[0x50, 0x4B, 0x03, 0x04])
            || bytes.starts_with(&[0x50, 0x4B, 0x05, 0x06])
            || bytes.starts_with(&[0x50, 0x4B, 0x07, 0x08])
        {
            return Some(ArchiveFormat::Zip);
        }
        if bytes.starts_with(&[0x37, 0x7A, 0xBC, 0xAF, 0x27, 0x1C]) {
            return Some(ArchiveFormat::SevenZip);
        }
        if bytes.starts_with(&[0x52, 0x61, 0x72, 0x21, 0x1A, 0x07]) {
            return Some(ArchiveFormat::Rar);
        }
        if bytes.starts_with(&[0x1F, 0x8B]) {
            return Some(ArchiveFormat::Gzip);
        }
        if bytes.starts_with(&[0xFD, 0x37, 0x7A, 0x58, 0x5A, 0x00]) {
            return Some(ArchiveFormat::Xz);
        }
        if bytes.starts_with(&[0x28, 0xB5, 0x2F, 0xFD]) {
            return Some(ArchiveFormat::Zstd);
        }
        if bytes.starts_with(b"BZh") {
            return Some(ArchiveFormat::Bzip2);
        }
        if bytes.starts_with(&[0x04, 0x22, 0x4D, 0x18]) {
            return Some(ArchiveFormat::Lz4);
        }
        if bytes.len() >= 262 && &bytes[257..262] == b"ustar" {
            return Some(ArchiveFormat::Tar);
        }
        None
    }

    /// Detect from magic bytes first (authoritative), falling back to the
    /// extension. Returns [`ArchiveFormat::Unknown`] if neither matches.
    pub fn detect(name: &str, magic: &[u8]) -> Self {
        Self::from_magic(magic)
            .or_else(|| Self::from_extension(name))
            .unwrap_or(ArchiveFormat::Unknown)
    }
}

impl fmt::Display for ArchiveFormat {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.name())
    }
}

/// Description of the work required to turn a downloaded archive into an
/// installed environment/game.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExtractionPlan {
    /// Archive file name or URL, used for extension-based detection.
    pub source_name: String,
    /// Destination directory passed to the unzip/merger stages.
    pub dest_dir: String,
    /// Detected container format.
    pub format: ArchiveFormat,
}

impl ExtractionPlan {
    /// Build a plan, detecting the format from the source name.
    pub fn new(source_name: impl Into<String>, dest_dir: impl Into<String>) -> Self {
        let source_name = source_name.into();
        let format = ArchiveFormat::from_extension(&source_name).unwrap_or(ArchiveFormat::Unknown);
        Self {
            source_name,
            dest_dir: dest_dir.into(),
            format,
        }
    }

    /// Build a plan, detecting the format from leading bytes with an extension
    /// fallback (see [`ArchiveFormat::detect`]).
    pub fn from_magic(
        source_name: impl Into<String>,
        dest_dir: impl Into<String>,
        magic: &[u8],
    ) -> Self {
        let source_name = source_name.into();
        let format = ArchiveFormat::detect(&source_name, magic);
        Self {
            source_name,
            dest_dir: dest_dir.into(),
            format,
        }
    }

    /// The fixed, ordered list of pipeline stages.
    pub const fn steps(&self) -> &'static [ExtractionStep; 4] {
        &ExtractionStep::ORDER
    }

    /// Whether this plan's format is one the unzip stage can actually handle.
    pub const fn is_supported(&self) -> bool {
        !matches!(self.format, ArchiveFormat::Unknown)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn steps_are_ordered_and_linked() {
        assert_eq!(
            ExtractionStep::ORDER,
            [
                ExtractionStep::Download,
                ExtractionStep::Unzip,
                ExtractionStep::Merge,
                ExtractionStep::Verify,
            ]
        );
        for (i, step) in ExtractionStep::ORDER.iter().enumerate() {
            assert_eq!(step.index(), i);
        }
        assert_eq!(ExtractionStep::Download.next(), Some(ExtractionStep::Unzip));
        assert_eq!(ExtractionStep::Merge.next(), Some(ExtractionStep::Verify));
        assert_eq!(ExtractionStep::Verify.next(), None);
    }

    #[test]
    fn string_keys_match_reference_resources() {
        assert_eq!(ExtractionStep::Unzip.string_key(), Some("comm_unzipping"));
        assert_eq!(
            ExtractionStep::Merge.string_key(),
            Some("llauncher_pc_game_file_merge")
        );
        assert_eq!(ExtractionStep::Download.string_key(), None);
    }

    #[test]
    fn detects_formats_from_extension() {
        assert_eq!(ArchiveFormat::from_extension("env.zip"), Some(ArchiveFormat::Zip));
        assert_eq!(
            ArchiveFormat::from_extension("https://x/y.ZIP?token=1"),
            Some(ArchiveFormat::Zip)
        );
        assert_eq!(ArchiveFormat::from_extension("game.7z"), Some(ArchiveFormat::SevenZip));
        assert_eq!(ArchiveFormat::from_extension("a.tar.gz"), Some(ArchiveFormat::Gzip));
        assert_eq!(ArchiveFormat::from_extension("a.firmware"), None);
        assert_eq!(ArchiveFormat::from_extension("noext"), None);
    }

    #[test]
    fn detects_formats_from_magic_bytes() {
        assert_eq!(
            ArchiveFormat::from_magic(b"PK\x03\x04rest"),
            Some(ArchiveFormat::Zip)
        );
        assert_eq!(
            ArchiveFormat::from_magic(&[0x1F, 0x8B, 0x08]),
            Some(ArchiveFormat::Gzip)
        );
        assert_eq!(
            ArchiveFormat::from_magic(&[0x37, 0x7A, 0xBC, 0xAF, 0x27, 0x1C]),
            Some(ArchiveFormat::SevenZip)
        );
        assert_eq!(ArchiveFormat::from_magic(b"BZh91..."), Some(ArchiveFormat::Bzip2));
        assert_eq!(ArchiveFormat::from_magic(b"not an archive"), None);
    }

    #[test]
    fn tar_uses_ustar_marker_at_offset_257() {
        let mut bytes = vec![0u8; 512];
        bytes[257..262].copy_from_slice(b"ustar");
        assert_eq!(ArchiveFormat::from_magic(&bytes), Some(ArchiveFormat::Tar));
    }

    #[test]
    fn magic_takes_priority_over_extension() {
        // A mislabeled .zip whose bytes are actually gzip.
        let plan = ExtractionPlan::from_magic("mislabeled.zip", "/d", &[0x1F, 0x8B]);
        assert_eq!(plan.format, ArchiveFormat::Gzip);
        assert!(plan.is_supported());
        assert_eq!(plan.steps().len(), 4);

        let unknown = ExtractionPlan::new("mystery.bin", "/d");
        assert_eq!(unknown.format, ArchiveFormat::Unknown);
        assert!(!unknown.is_supported());
    }
}
