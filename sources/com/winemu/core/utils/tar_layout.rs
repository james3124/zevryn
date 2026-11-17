//! Rust model of the portable part of `com.winemu.core.utils.TarCompressorUtils`
//! (GameHub Lite v5.1.8).
//!
//! Java `TarCompressorUtils` walks a tar stream via
//! `TarArchiveInputStream.getNextEntry()` and, for each `TarArchiveEntry`,
//! reads `p()` (name), `s()` (size), `u()` (isDirectory), `H()`
//! (isSymbolicLink) and `k()` (link name). This module models that as an
//! **ordered list of entries** (a `TarLayout`) so callers can plan an
//! extraction without any I/O.
//!
//! Compression (`Type.XZ/ZSTD/BZ2` → Commons Compressor streams), actual tar
//! parsing, filesystem extraction, symlink creation (`FileUtils.p`) and
//! permissions (`FileUtils.a(file, 505)`) are all out of scope. Only the
//! `Type` tag and the ordered-entry shape are kept.

/// `TarCompressorUtils.Type` — the compression wrapper around a tar stream.
/// `None` is a plain `.tar`; the others correspond to XZ/ZSTD/BZip2
/// compressor input streams in `f(Type, InputStream)`.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum CompressionType {
    #[default]
    None,
    Xz,
    Zstd,
    Bz2,
}

impl CompressionType {
    /// Infer the compression type from a filename's extension, matching the
    /// conventional suffixes handled by the Java callers.
    pub fn from_filename(name: &str) -> Self {
        let lower = name.to_ascii_lowercase();
        if lower.ends_with(".xz") || lower.ends_with(".tar.xz") {
            Self::Xz
        } else if lower.ends_with(".zst") || lower.ends_with(".zstd") || lower.ends_with(".tar.zst") {
            Self::Zstd
        } else if lower.ends_with(".bz2") || lower.ends_with(".tar.bz2") {
            Self::Bz2
        } else {
            Self::None
        }
    }

    /// Whether a compressor stream is needed (i.e. not `None`).
    pub fn is_compressed(self) -> bool {
        self != Self::None
    }
}

/// Kind of a tar entry, from `TarArchiveEntry.u()` / `.H()`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TarEntryKind {
    File,
    Directory,
    Symlink,
}

/// A single modeled `TarArchiveEntry`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TarEntry {
    /// `p()` — entry path/name.
    pub name: String,
    /// `s()` — uncompressed size in bytes (0 for dirs/symlinks).
    pub size: u64,
    /// Unix mode bits; `FileUtils.a(file, 505)` uses `0771` in Java.
    pub mode: u32,
    /// `k()` — symlink target (only meaningful for [`TarEntryKind::Symlink`]).
    pub link_name: String,
    /// Entry kind.
    pub kind: TarEntryKind,
}

impl TarEntry {
    pub fn file(name: impl Into<String>, size: u64) -> Self {
        Self {
            name: name.into(),
            size,
            mode: 0o644,
            link_name: String::new(),
            kind: TarEntryKind::File,
        }
    }

    pub fn directory(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            size: 0,
            mode: 0o755,
            link_name: String::new(),
            kind: TarEntryKind::Directory,
        }
    }

    pub fn symlink(name: impl Into<String>, target: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            size: 0,
            mode: 0o777,
            link_name: target.into(),
            kind: TarEntryKind::Symlink,
        }
    }

    /// `u()` — is this a directory entry?
    pub fn is_directory(&self) -> bool {
        self.kind == TarEntryKind::Directory
    }

    /// `H()` — is this a symbolic link?
    pub fn is_symlink(&self) -> bool {
        self.kind == TarEntryKind::Symlink
    }

    /// `p()` — the entry name.
    pub fn name(&self) -> &str {
        &self.name
    }

    /// `s()` — the entry size.
    pub fn size(&self) -> u64 {
        self.size
    }

    /// `k()` — the link name.
    pub fn link_name(&self) -> &str {
        &self.link_name
    }
}

/// An ordered sequence of tar entries, mirroring the order returned by
/// repeated `getNextEntry()` calls.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct TarLayout {
    pub entries: Vec<TarEntry>,
    pub compression: CompressionType,
}

impl TarLayout {
    pub fn new(compression: CompressionType) -> Self {
        Self {
            entries: Vec::new(),
            compression,
        }
    }

    /// Append an entry, preserving order.
    pub fn push(&mut self, entry: TarEntry) {
        self.entries.push(entry);
    }

    /// Convenience: append a file entry.
    pub fn add_file(&mut self, name: impl Into<String>, size: u64) {
        self.push(TarEntry::file(name, size));
    }

    /// Convenience: append a directory entry.
    pub fn add_dir(&mut self, name: impl Into<String>) {
        self.push(TarEntry::directory(name));
    }

    /// Convenience: append a symlink entry.
    pub fn add_symlink(&mut self, name: impl Into<String>, target: impl Into<String>) {
        self.push(TarEntry::symlink(name, target));
    }

    pub fn iter(&self) -> impl Iterator<Item = &TarEntry> {
        self.entries.iter()
    }

    pub fn len(&self) -> usize {
        self.entries.len()
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// Total uncompressed payload size of regular file entries.
    pub fn total_file_size(&self) -> u64 {
        self.entries
            .iter()
            .filter(|e| e.kind == TarEntryKind::File)
            .map(|e| e.size)
            .sum()
    }

    /// Names of all entries in order (the `getNextEntry().p()` sequence).
    pub fn names(&self) -> Vec<&str> {
        self.entries.iter().map(|e| e.name.as_str()).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn compression_type_from_filename() {
        assert_eq!(CompressionType::from_filename("a.tar"), CompressionType::None);
        assert_eq!(CompressionType::from_filename("a.tar.xz"), CompressionType::Xz);
        assert_eq!(CompressionType::from_filename("a.zst"), CompressionType::Zstd);
        assert_eq!(CompressionType::from_filename("a.tar.bz2"), CompressionType::Bz2);
        assert_eq!(CompressionType::from_filename("A.TAR.XZ"), CompressionType::Xz);
        assert!(CompressionType::Zstd.is_compressed());
        assert!(!CompressionType::None.is_compressed());
    }

    #[test]
    fn entry_accessors() {
        let f = TarEntry::file("dir/a.txt", 123);
        assert_eq!(f.name(), "dir/a.txt");
        assert_eq!(f.size(), 123);
        assert!(!f.is_directory());
        assert!(!f.is_symlink());

        let d = TarEntry::directory("dir/");
        assert!(d.is_directory());
        assert_eq!(d.size(), 0);

        let s = TarEntry::symlink("dir/link", "a.txt");
        assert!(s.is_symlink());
        assert_eq!(s.link_name(), "a.txt");
    }

    #[test]
    fn layout_preserves_entry_order() {
        let mut layout = TarLayout::new(CompressionType::Xz);
        layout.add_dir("prefix/");
        layout.add_file("prefix/a", 10);
        layout.add_file("prefix/b", 20);
        layout.add_symlink("prefix/l", "a");
        assert_eq!(
            layout.names(),
            vec!["prefix/", "prefix/a", "prefix/b", "prefix/l"]
        );
        assert_eq!(layout.len(), 4);
        assert_eq!(layout.total_file_size(), 30);
        assert_eq!(layout.compression, CompressionType::Xz);
        assert_eq!(layout.iter().filter(|e| e.is_directory()).count(), 1);
    }
}
