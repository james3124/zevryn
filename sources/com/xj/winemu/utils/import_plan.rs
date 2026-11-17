//! Ordered, deduplicated import plan for a batch of selected file paths.
//!
//! This models the portable shape of
//! `ImportPcGameHelper.readAppManifestFilesAndImportGame(callback)`: the
//! coroutine scans the selected files, filters manifest files via the
//! `appmanifest_*` predicate (`readAppManifestFilesAndImportGame$1.java:298-311`),
//! parses each one into a `LocalPcGameManifest`, resolves the executable name
//! and calls `fetchGameInfoByExeFileName` (`ImportGameViewModel.java:49-57`)
//! before importing. The plan produced here describes that order but performs
//! no I/O, permission checks, notifications or game-library writes.

use super::exe_info::ExeInfo;
use super::file_detector::{basename, classify, manifest_steam_app_id, FileKind};

/// The coarse kind of an [`ImportStep`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ImportStepKind {
    ParseManifest,
    ReadExeInfo,
    FetchGameInfo,
    Complete,
}

/// One action in the import pipeline.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ImportStep {
    /// Parse an `appmanifest_*` file and extract its Steam app id from the
    /// name (the Steam VDF/ACF body can be parsed with `AcfDocument`).
    ParseManifest {
        path: String,
        steam_app_id: String,
    },
    /// Derive the game name from an executable path.
    ReadExeInfo {
        path: String,
        game_name: String,
    },
    /// Look up a game by exe name (`fetchGameInfoByExeFileName`).
    FetchGameInfo {
        exe_name: String,
    },
    /// Final summary step, mirroring the callback that reports completion.
    Complete {
        manifest_count: usize,
        exe_count: usize,
    },
}

impl ImportStep {
    pub fn kind(&self) -> ImportStepKind {
        match self {
            ImportStep::ParseManifest { .. } => ImportStepKind::ParseManifest,
            ImportStep::ReadExeInfo { .. } => ImportStepKind::ReadExeInfo,
            ImportStep::FetchGameInfo { .. } => ImportStepKind::FetchGameInfo,
            ImportStep::Complete { .. } => ImportStepKind::Complete,
        }
    }
}

/// A deduplicated, ordered description of the work for a batch of paths.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ImportPlan {
    /// Pipeline steps in execution order.
    pub steps: Vec<ImportStep>,
    /// Paths classified as [`FileKind::Ignored`], in first-seen order.
    pub ignored: Vec<String>,
}

impl ImportPlan {
    /// Classify every path, drop duplicate paths (keeping the first
    /// occurrence), and build the ordered step list:
    ///
    /// 1. `ParseManifest` for each manifest, in input order.
    /// 2. `ReadExeInfo` for each exe, in input order.
    /// 3. `FetchGameInfo` for each distinct exe game name, in first-seen order.
    /// 4. A single `Complete` step with the manifest/exe counts.
    pub fn from_paths<'a, I>(paths: I) -> Self
    where
        I: IntoIterator<Item = &'a str>,
    {
        let mut seen = Vec::new();
        let mut manifests: Vec<(String, String)> = Vec::new();
        let mut exes: Vec<(String, String)> = Vec::new();
        let mut ignored = Vec::new();

        for path in paths {
            if seen.iter().any(|p| *p == path) {
                continue;
            }
            seen.push(path);
            match classify(path) {
                FileKind::Manifest => {
                    let name = basename(path);
                    let steam_app_id =
                        manifest_steam_app_id(name).unwrap_or_default().to_owned();
                    manifests.push((path.to_owned(), steam_app_id));
                }
                FileKind::Exe => {
                    if let Some(info) = ExeInfo::from_path(path) {
                        exes.push((info.path, info.game_name));
                    }
                }
                FileKind::Ignored => ignored.push(path.to_owned()),
            }
        }

        let manifest_count = manifests.len();
        let exe_count = exes.len();
        let mut steps: Vec<ImportStep> = manifests
            .into_iter()
            .map(|(path, steam_app_id)| ImportStep::ParseManifest { path, steam_app_id })
            .collect();
        steps.extend(exes.iter().cloned().map(|(path, game_name)| ImportStep::ReadExeInfo { path, game_name }));

        let mut looked_up = Vec::new();
        for (_, game_name) in &exes {
            if !looked_up.iter().any(|n: &String| n == game_name) {
                looked_up.push(game_name.clone());
                steps.push(ImportStep::FetchGameInfo { exe_name: game_name.clone() });
            }
        }
        steps.push(ImportStep::Complete { manifest_count, exe_count });

        Self { steps, ignored }
    }

    /// Number of steps in the plan.
    pub fn len(&self) -> usize {
        self.steps.len()
    }

    pub fn is_empty(&self) -> bool {
        self.steps.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn classifies_dedupes_and_orders() {
        let paths = [
            "/s/steamapps/appmanifest_570.acf",
            r"C:\Games\dota2.exe",
            "/s/steamapps/libraryfolders.vdf",
            "/s/steamapps/appmanifest_570.acf",
            "/other/appmanifest_10.json",
            r"D:\Games\dota2.EXE",
        ];
        let plan = ImportPlan::from_paths(paths.iter().copied());
        assert_eq!(
            plan.steps,
            vec![
                ImportStep::ParseManifest {
                    path: "/s/steamapps/appmanifest_570.acf".into(),
                    steam_app_id: "570".into(),
                },
                ImportStep::ParseManifest {
                    path: "/other/appmanifest_10.json".into(),
                    steam_app_id: "10".into(),
                },
                ImportStep::ReadExeInfo {
                    path: r"C:\Games\dota2.exe".into(),
                    game_name: "dota2".into(),
                },
                ImportStep::ReadExeInfo {
                    path: r"D:\Games\dota2.EXE".into(),
                    game_name: "dota2".into(),
                },
                ImportStep::FetchGameInfo { exe_name: "dota2".into() },
                ImportStep::Complete { manifest_count: 2, exe_count: 2 },
            ]
        );
        assert_eq!(plan.ignored, vec!["/s/steamapps/libraryfolders.vdf".to_owned()]);
        assert_eq!(plan.len(), 6);
    }

    #[test]
    fn step_kinds_follow_pipeline_order() {
        let plan = ImportPlan::from_paths(["appmanifest_1.acf", "game.exe"].into_iter());
        let kinds: Vec<_> = plan.steps.iter().map(ImportStep::kind).collect();
        assert_eq!(
            kinds,
            vec![
                ImportStepKind::ParseManifest,
                ImportStepKind::ReadExeInfo,
                ImportStepKind::FetchGameInfo,
                ImportStepKind::Complete,
            ]
        );
    }

    #[test]
    fn empty_batch_still_has_complete_step() {
        let plan = ImportPlan::from_paths(std::iter::empty::<&str>());
        assert_eq!(plan.steps, vec![ImportStep::Complete { manifest_count: 0, exe_count: 0 }]);
        assert!(!plan.is_empty());
        assert!(plan.ignored.is_empty());
    }
}
