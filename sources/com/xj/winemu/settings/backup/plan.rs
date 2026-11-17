//! Ordered backup plan and pure path-set transformations.
//!
//! `PcGameDataHandler.backupData2Local` / `recoverDataFromLocal` ultimately
//! archive a single virtual-container directory into one `.zip`. The model
//! here generalizes that into an ordered list of logical entries so the
//! ordering is observable and testable. `backup` / `recover` never touch the
//! filesystem: they map a set of available paths onto a [`BackupReport`].

use std::collections::BTreeSet;

/// Logical category of a backed-up path.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum BackupKind {
    /// The game's virtual container directory (the primary ZIP payload).
    Container,
    /// Per-game settings store.
    Settings,
    /// Translation / configuration repository data.
    TranslationConfig,
    /// Environment-variable string (`KEY=VALUE,KEY2=VALUE2`).
    EnvParams,
    /// Locally selected PC executable path.
    ExePath,
}

impl BackupKind {
    /// Stable lowercase label, useful for reports and logging.
    pub fn as_str(self) -> &'static str {
        match self {
            BackupKind::Container => "container",
            BackupKind::Settings => "settings",
            BackupKind::TranslationConfig => "translation_config",
            BackupKind::EnvParams => "env_params",
            BackupKind::ExePath => "exe_path",
        }
    }
}

/// One entry in a [`BackupPlan`]: a path relative to the game's data root and
/// the logical kind of data it holds.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BackupEntry {
    pub relative_path: String,
    pub kind: BackupKind,
}

impl BackupEntry {
    pub fn new(relative_path: impl Into<String>, kind: BackupKind) -> Self {
        Self {
            relative_path: relative_path.into(),
            kind,
        }
    }

    pub fn is_container(&self) -> bool {
        self.kind == BackupKind::Container
    }
}

/// The result of a `backup` or `recover` path-set transformation.
///
/// Both found and missing paths are reported in the order of the source
/// [`BackupPlan`], so the report is deterministic.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct BackupReport {
    /// Number of entries the plan asked for.
    pub requested: usize,
    /// Paths present in the supplied set (the transformation's output set).
    pub found: Vec<String>,
    /// Paths absent from the supplied set.
    pub missing: Vec<String>,
}

impl BackupReport {
    /// True when every requested path was present.
    pub fn is_success(&self) -> bool {
        self.missing.is_empty()
    }

    pub fn found_count(&self) -> usize {
        self.found.len()
    }

    pub fn missing_count(&self) -> usize {
        self.missing.len()
    }
}

/// An ordered list of paths to back up or recover.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct BackupPlan {
    pub entries: Vec<BackupEntry>,
}

impl BackupPlan {
    pub fn new() -> Self {
        Self::default()
    }

    /// A plan mirroring the data touched by the app: settings first, then the
    /// virtual container (the ZIP payload written by `backupData2Local`).
    pub fn with_defaults() -> Self {
        Self {
            entries: vec![
                BackupEntry::new("settings", BackupKind::Settings),
                BackupEntry::new("translation_config", BackupKind::TranslationConfig),
                BackupEntry::new("env_params", BackupKind::EnvParams),
                BackupEntry::new("exe_path", BackupKind::ExePath),
                BackupEntry::new("container", BackupKind::Container),
            ],
        }
    }

    /// Append an entry, preserving insertion order.
    pub fn push(&mut self, entry: BackupEntry) {
        self.entries.push(entry);
    }

    /// Convenience builder that returns `self` for chaining.
    pub fn with_entry(mut self, relative_path: impl Into<String>, kind: BackupKind) -> Self {
        self.push(BackupEntry::new(relative_path, kind));
        self
    }

    pub fn len(&self) -> usize {
        self.entries.len()
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    pub fn iter(&self) -> std::slice::Iter<'_, BackupEntry> {
        self.entries.iter()
    }

    /// All relative paths, in plan order.
    pub fn paths(&self) -> Vec<&str> {
        self.entries
            .iter()
            .map(|entry| entry.relative_path.as_str())
            .collect()
    }

    /// Pure backup transformation: from the set of source paths that exist,
    /// derive which plan entries are covered and which are missing.
    pub fn backup(&self, available_sources: &BTreeSet<String>) -> BackupReport {
        self.transform(available_sources)
    }

    /// Pure recover transformation: from the set of paths present in an
    /// archive, derive which plan entries can be restored and which cannot.
    pub fn recover(&self, available_in_archive: &BTreeSet<String>) -> BackupReport {
        self.transform(available_in_archive)
    }

    fn transform(&self, available: &BTreeSet<String>) -> BackupReport {
        let mut report = BackupReport {
            requested: self.entries.len(),
            ..BackupReport::default()
        };
        for entry in &self.entries {
            if available.contains(&entry.relative_path) {
                report.found.push(entry.relative_path.clone());
            } else {
                report.missing.push(entry.relative_path.clone());
            }
        }
        report
    }
}

impl<'a> IntoIterator for &'a BackupPlan {
    type Item = &'a BackupEntry;
    type IntoIter = std::slice::Iter<'a, BackupEntry>;

    fn into_iter(self) -> Self::IntoIter {
        self.entries.iter()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn set(paths: &[&str]) -> BTreeSet<String> {
        paths.iter().map(|s| s.to_string()).collect()
    }

    #[test]
    fn default_plan_order_is_stable() {
        let plan = BackupPlan::with_defaults();
        assert_eq!(
            plan.paths(),
            vec![
                "settings",
                "translation_config",
                "env_params",
                "exe_path",
                "container",
            ]
        );
        assert_eq!(plan.len(), 5);
        assert!(plan.iter().last().unwrap().is_container());
    }

    #[test]
    fn plan_preserves_insertion_order() {
        let plan = BackupPlan::new()
            .with_entry("z", BackupKind::Settings)
            .with_entry("a", BackupKind::Container)
            .with_entry("m", BackupKind::EnvParams);
        assert_eq!(plan.paths(), vec!["z", "a", "m"]);
    }

    #[test]
    fn backup_is_a_pure_ordered_path_set_transform() {
        let plan = BackupPlan::with_defaults();
        let sources = set(&["container", "settings", "env_params"]);
        let report = plan.backup(&sources);
        assert_eq!(report.requested, 5);
        assert_eq!(report.found, vec!["settings", "env_params", "container"]);
        assert_eq!(report.missing, vec!["translation_config", "exe_path"]);
        assert!(!report.is_success());

        // The input set is untouched (no I/O, no mutation).
        assert_eq!(sources.len(), 3);
    }

    #[test]
    fn recover_reports_success_when_archive_is_complete() {
        let plan = BackupPlan::with_defaults();
        let all: BTreeSet<String> = plan.paths().into_iter().map(str::to_string).collect();
        let report = plan.recover(&all);
        assert!(report.is_success());
        assert_eq!(report.missing_count(), 0);
        assert_eq!(report.found_count(), 5);
        assert_eq!(report.found, plan.paths());
    }

    #[test]
    fn empty_plan_is_always_successful() {
        let plan = BackupPlan::new();
        let report = plan.backup(&BTreeSet::new());
        assert!(plan.is_empty());
        assert!(report.is_success());
        assert_eq!(report.requested, 0);
    }
}
