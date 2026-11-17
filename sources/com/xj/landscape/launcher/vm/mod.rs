//! Rust reimplementation of the *portable viewmodel state* under
//! `com.xj.landscape.launcher.vm` in GameHub Lite v5.1.8.
//!
//! Submodules:
//! * `async_state`             — generic `AsyncState<T>` async state container
//! * `types`                   — self-contained, plain value types referenced by
//!                               the viewmodels (no dependency on `../data` etc.)
//! * `main_view_model`         — `NewLandscapeLauncherMainVM` /
//!                               `LandscapeLauncherMainVM` → tabs, game-circle
//!                               list, selected tab, data-refresh state
//! * `game_detail_view_model`  — `GameDetailVM` → detail fetch, download-status
//!                               refresh, steam-vs-native start-type choice
//! * `device_manager_view_model` — `DeviceManagerVM` / `MineDeviceVM` →
//!                               connected-device list + selected device
//! * `misc_vms`                — lightweight portable state for the remaining
//!                               studied VMs (album, feedback, highlights,
//!                               search, login guide)
//!
//! Coroutines/Flow/LiveData, repository network I/O and the Android lifecycle
//! are deliberately not modeled; see `REPORT.md`.

mod async_state;
mod device_manager_view_model;
mod game_detail_view_model;
mod main_view_model;
mod misc_vms;
mod types;

pub use async_state::*;
pub use device_manager_view_model::*;
pub use game_detail_view_model::*;
pub use main_view_model::*;
pub use misc_vms::*;
pub use types::*;
