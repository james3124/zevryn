//! Rust model of the portable PC-game import pipeline from GameHub Lite v5.1.8.
//!
//! Mirrors `sources/com/xj/winemu/utils/ImportPcGameHelper.java`,
//! `sources/com/xj/winemu/bean/LocalPcGameManifest.java` and
//! `sources/com/xj/winemu/vm/ImportGameViewModel.java`. Android-specific
//! coroutines, notifications, storage permissions and real file I/O are left
//! out; this module only models the strings, data structures and ordering of
//! the import flow. See `REPORT.md` for exact source references.

pub mod acf_parser;
pub mod exe_info;
pub mod file_detector;
pub mod import_plan;
pub mod local_pc_game_manifest;

pub use self::acf_parser::{AcfDocument, AcfParseError, VdfValue};
pub use self::exe_info::ExeInfo;
pub use self::file_detector::{
    basename, classify, is_exe_file, is_manifest_file, manifest_steam_app_id, FileKind,
    EXE_SUFFIX, MANIFEST_ACF_SUFFIX, MANIFEST_JSON_SUFFIX, MANIFEST_PREFIX,
};
pub use self::import_plan::{ImportPlan, ImportStep, ImportStepKind};
pub use self::local_pc_game_manifest::LocalPcGameManifest;
