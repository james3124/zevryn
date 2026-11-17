//! General Aria / WinEmu download-and-install pipeline.
//!
//! Dependency-free models for the download job, the ordered extract/merge
//! pipeline, the merge manifest and the Aria task configuration. See
//! `REPORT.md` in this directory for sources and out-of-scope notes.
//!
//! Wire from `lib.rs` with:
//!
//! ```text
//! #[path = "../sources/com/xj/common/download/mod.rs"]
//! pub mod download;
//! ```

mod aria_task_config;
mod download_job;
mod extraction_plan;
mod file_merge;

pub use aria_task_config::{AriaTaskConfig, QueueMod};
pub use download_job::{DownloadJob, JobState};
pub use extraction_plan::{ArchiveFormat, ExtractionPlan, ExtractionStep};
pub use file_merge::{content_equal, FileMerge, ManifestEntry, Mismatch, ObservedFile, VerifyReport};
