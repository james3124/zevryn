//! Rust port of the portable logic in `com.xj.ota` (GameHub Lite v5.1.8).
//!
//! Submodules mirror the Java packages/classes:
//! * [`state`] — OTA upgrade state machine (`BaseOTA.g()` flow).
//! * [`download`] — `DownloadTask` and the `DownloadStatus` sealed hierarchy.
//! * [`info`] — OTA / firmware / device bean structs.
//! * [`check`] — version comparison and update-check decisions.
//! * [`listener`] — `IUpListener` callback codes/events.
//!
//! Android-only code (OkHttp downloads, BLE/USB transports, per-device flash
//! routines, Activities, dialogs, notifications) is deliberately excluded.

pub mod check;
pub mod download;
pub mod info;
pub mod listener;
pub mod state;

pub use check::{compare_versions, compare_versions_i32, is_update_available, needs_upgrade, UpdateCheckStrategy};
pub use download::{DownloadEvent, DownloadStatus, DownloadTask, DownloadTaskState};
pub use info::{
    DeviceStatusInfo, FirmwareDataEntry, FirmwareEntry, FirmwareInfo, GamepadInfo, OtaInfo, RadiatorInfo,
};
pub use listener::{UpgradeEvent, UpgradeEvent as OtaUpgradeEvent};
pub use state::{
    can_transition, transition, InvalidTransition, OtaEffect, OtaEvent, OtaFailure, OtaState,
    OtaStateMachine,
};
