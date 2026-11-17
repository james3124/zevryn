//! Rust model of the portable backup / reset / container-check logic of
//! `com.xj.winemu.settings` (GameHub Lite v5.1.8).
//!
//! Mirrors `sources/com/xj/winemu/settings/PcGameDataHandler.java`
//! (`backupData2Local`, `recoverDataFromLocal`, `checkGameVirtualContainerExist`,
//! `resetGameData`) together with the reset orchestration in
//! `PcGameSettingOperations` / `PcGameSettingDataHelper`.
//!
//! Only the pure data model is kept here: an ordered backup plan and its
//! path-set transformations, reset scopes/targets, and a container-check
//! result. Real SAF streams, ZIP (de)compression via native `WinAPI`,
//! coroutines and Android UI are intentionally out of scope — see `REPORT.md`.

pub mod container;
pub mod plan;
pub mod reset;

pub use self::container::VirtualContainerCheck;
pub use self::plan::{BackupEntry, BackupKind, BackupPlan, BackupReport};
pub use self::reset::{ResetPlan, ResetScope, ResetTarget};
