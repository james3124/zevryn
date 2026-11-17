//! Rust port of the portable logic in `com.xj.apk.update` (GameHub Lite v5.1.8).
//!
//! * [`update`] — `DownloadManager` configuration, validation and decisions.
//! * [`download`] — shared `DownloadStatus` / `DownloadTask` model (reused from
//!   the OTA area so both download subsystems use one definition).
//!
//! Android `DownloadService`, notifications, `FileProvider` and dialog
//! Activities are intentionally not reimplemented.

// The download model is source-identical to the OTA one; include it here so
// `apk` consumers see the same types without depending on crate-level wiring.
#[path = "../ota/download.rs"]
pub mod download;

pub mod update;

pub use download::{DownloadEvent, DownloadStatus, DownloadTask, DownloadTaskState};
pub use update::{
    ApkUpdateConfig, ConfigError, UpdateDecision, UpdatePaths, CACHE_DIR_TEMPLATE,
    FILE_PROVIDER_SUFFIX, UNSET_VERSION_CODE,
};
