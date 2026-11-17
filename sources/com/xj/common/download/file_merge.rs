//! File-merge manifest and integrity verification over precomputed hashes.
//!
//! Reference sources:
//! * `com.xj.common.download.bean.SubData` — `{sub_file_name, sub_download_url,
//!   sub_file_md5}` describes a sub-file with its expected name and MD5.
//! * `com.xj.common.download.bean.AriaDownloadArgs` — `fileMd5` / `fileSize`
//!   are the expected hash and size recorded on a download task.
//! * `com.xj.standalone.steam.contentdownloader.FileMerger` — staged files are
//!   moved atomically (`.tmp` + `ATOMIC_MOVE`) and compared with `e(file1,
//!   file2)`, which checks equal lengths then equal bytes.
//!
//! No cryptographic dependency is used: MD5 strings are treated as opaque
//! precomputed hashes supplied by the caller (the manifest's expected values
//! vs. the observed values). A byte-for-byte comparison helper mirrors
//! `FileMerger.e` for callers that have raw bytes instead of hashes.

use std::collections::BTreeMap;
use std::fmt;

/// One expected file in a merge manifest: `{name, size, md5}`.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ManifestEntry {
    pub name: String,
    pub size: u64,
    pub md5: String,
}

impl ManifestEntry {
    pub fn new(name: impl Into<String>, size: u64, md5: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            size,
            md5: md5.into(),
        }
    }
}

/// An observed/precomputed file to check against a manifest.
///
/// The caller supplies `size` and `md5` from its own (dependency-free) source —
/// e.g. values read from a server response or an on-disk index. No hashing is
/// performed in this module.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ObservedFile {
    pub name: String,
    pub size: u64,
    pub md5: String,
}

impl ObservedFile {
    pub fn new(name: impl Into<String>, size: u64, md5: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            size,
            md5: md5.into(),
        }
    }
}

/// A single verification failure.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Mismatch {
    /// A manifest file has no observed counterpart.
    Missing { name: String },
    /// Sizes differ (`FileMerger.e` first compares `file.length()`).
    Size {
        name: String,
        expected: u64,
        actual: u64,
    },
    /// Precomputed hashes differ (hex compared case-insensitively).
    Hash {
        name: String,
        expected: String,
        actual: String,
    },
    /// An observed file is not described by the manifest.
    Unexpected { name: String },
}

impl fmt::Display for Mismatch {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Mismatch::Missing { name } => write!(f, "missing: {name}"),
            Mismatch::Size {
                name,
                expected,
                actual,
            } => write!(f, "size mismatch: {name} (expected {expected}, got {actual})"),
            Mismatch::Hash {
                name,
                expected,
                actual,
            } => write!(f, "hash mismatch: {name} (expected {expected}, got {actual})"),
            Mismatch::Unexpected { name } => write!(f, "unexpected file: {name}"),
        }
    }
}

/// Result of verifying an observed file set against a manifest.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VerifyReport {
    pub ok: bool,
    /// Number of manifest entries that were found and checked.
    pub checked: usize,
    pub mismatches: Vec<Mismatch>,
}

/// An ordered list of files to merge, with their expected size and hash.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct FileMerge {
    pub entries: Vec<ManifestEntry>,
}

impl FileMerge {
    pub fn new(entries: Vec<ManifestEntry>) -> Self {
        Self { entries }
    }

    /// Convenience constructor from `(name, size, md5)` triples.
    pub fn from_tuples<U: Into<String>, H: Into<String>>(items: Vec<(U, u64, H)>) -> Self {
        Self {
            entries: items
                .into_iter()
                .map(|(n, s, h)| ManifestEntry::new(n, s, h))
                .collect(),
        }
    }

    pub fn add(&mut self, name: impl Into<String>, size: u64, md5: impl Into<String>) {
        self.entries.push(ManifestEntry::new(name, size, md5));
    }

    pub fn len(&self) -> usize {
        self.entries.len()
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    pub fn get(&self, name: &str) -> Option<&ManifestEntry> {
        self.entries.iter().find(|e| e.name == name)
    }

    /// Verify the observed files against this manifest.
    ///
    /// Every manifest entry must be present with a matching size and a
    /// case-insensitive MD5 string. Extra observed files are reported as
    /// `Unexpected`. `ok` is true only when there are no mismatches.
    pub fn verify(&self, observed: &[ObservedFile]) -> VerifyReport {
        let observed_by_name: BTreeMap<&str, &ObservedFile> = observed
            .iter()
            .map(|f| (f.name.as_str(), f))
            .collect();
        let mut checked = 0usize;
        let mut mismatches = Vec::new();

        for expected in &self.entries {
            match observed_by_name.get(expected.name.as_str()) {
                None => mismatches.push(Mismatch::Missing {
                    name: expected.name.clone(),
                }),
                Some(found) => {
                    checked += 1;
                    if found.size != expected.size {
                        mismatches.push(Mismatch::Size {
                            name: expected.name.clone(),
                            expected: expected.size,
                            actual: found.size,
                        });
                    }
                    if !hashes_equal(&expected.md5, &found.md5) {
                        mismatches.push(Mismatch::Hash {
                            name: expected.name.clone(),
                            expected: expected.md5.clone(),
                            actual: found.md5.clone(),
                        });
                    }
                }
            }
        }

        for name in observed_by_name.keys() {
            if self.get(name).is_none() {
                mismatches.push(Mismatch::Unexpected {
                    name: (*name).to_owned(),
                });
            }
        }

        VerifyReport {
            ok: mismatches.is_empty(),
            checked,
            mismatches,
        }
    }
}

/// Compare two hex hash strings without depending on a crypto crate.
/// Comparison is ASCII case-insensitive; surrounding whitespace is ignored.
fn hashes_equal(expected: &str, actual: &str) -> bool {
    expected.trim().eq_ignore_ascii_case(actual.trim())
}

/// Byte-for-byte comparison mirroring `FileMerger.e(File, File)`: equal length
/// first, then equal contents. Provided for callers that hold raw bytes rather
/// than precomputed hashes.
pub fn content_equal(a: &[u8], b: &[u8]) -> bool {
    a.len() == b.len() && a == b
}

#[cfg(test)]
mod tests {
    use super::*;

    fn manifest() -> FileMerge {
        FileMerge::from_tuples(vec![
            ("env.zip", 100u64, "abc123"),
            ("run.exe", 50u64, "deadbeef"),
        ])
    }

    #[test]
    fn verifies_matching_manifest_case_insensitively() {
        let observed = [
            ObservedFile::new("env.zip", 100, "ABC123"),
            ObservedFile::new("run.exe", 50, "DEADBEEF"),
        ];
        let report = manifest().verify(&observed);
        assert!(report.ok);
        assert_eq!(report.checked, 2);
        assert!(report.mismatches.is_empty());
    }

    #[test]
    fn reports_missing_size_and_hash_mismatches() {
        let observed = [
            ObservedFile::new("env.zip", 99, "abc123"), // size mismatch
            ObservedFile::new("run.exe", 50, "bad"),    // hash mismatch
            // "extra.dat" would be Unexpected, "ghost" is not supplied -> Missing
        ];
        let mut m = manifest();
        m.add("ghost.bin", 1, "0");
        let report = m.verify(&observed);
        assert!(!report.ok);
        assert_eq!(report.checked, 2);
        assert!(report.mismatches.contains(&Mismatch::Size {
            name: "env.zip".into(),
            expected: 100,
            actual: 99,
        }));
        assert!(report
            .mismatches
            .iter()
            .any(|x| matches!(x, Mismatch::Hash { name, .. } if name == "run.exe")));
        assert!(report
            .mismatches
            .contains(&Mismatch::Missing { name: "ghost.bin".into() }));
    }

    #[test]
    fn reports_unexpected_files() {
        let observed = [
            ObservedFile::new("env.zip", 100, "abc123"),
            ObservedFile::new("run.exe", 50, "deadbeef"),
            ObservedFile::new("extra.dat", 1, "0"),
        ];
        let report = manifest().verify(&observed);
        assert!(!report.ok);
        assert!(report
            .mismatches
            .contains(&Mismatch::Unexpected { name: "extra.dat".into() }));
    }

    #[test]
    fn empty_manifest_vs_no_files_is_ok() {
        let report = FileMerge::default().verify(&[]);
        assert!(report.ok);
        assert_eq!(report.checked, 0);
    }

    #[test]
    fn content_equal_matches_file_merger_semantics() {
        assert!(content_equal(b"abc", b"abc"));
        assert!(!content_equal(b"abc", b"ab"));
        assert!(!content_equal(b"abc", b"abd"));
    }

    #[test]
    fn get_and_len_work() {
        let m = manifest();
        assert_eq!(m.len(), 2);
        assert!(!m.is_empty());
        assert_eq!(m.get("run.exe").map(|e| e.size), Some(50));
        assert!(m.get("nope").is_none());
    }
}
