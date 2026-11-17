//! Rust port of the portable logic under the decompiled
//! `com.xj.landscape.launcher.ui.gamedetail` package (GameHub Lite v5.1.8).
//!
//! * [`start_type`] — exact `start_type` codes from `LauncherTypeEnum` mapped
//!   to [`GameStartType`] / [`GameStartCategory`].
//! * [`launch_params`] — [`GameLaunchParams`] and its builder.
//! * [`ui_state`] — [`GameDetailUiState`] and its transitions.
//! * [`launch_decision`] — pure [`LaunchAction`] decision logic.
//!
//! Android `GameDetailActivity` / fragments / adapters, real launch execution,
//! and Steam/cloud protocols are intentionally out of scope (see `REPORT.md`).

pub mod launch_decision;
pub mod launch_params;
pub mod menu;
pub mod start_type;
pub mod ui_state;

pub use launch_decision::{
    decide_launch, decide_launch_for_code, is_cloud_start_code, LaunchAction, LocalAvailability,
};
pub use launch_params::{ExtraArgs, GameLaunchParams, GameLaunchParamsBuilder};
pub use start_type::{
    is_mobile_game_start_type, is_pc_emu_game, GameStartCategory, GameStartType,
};
pub use ui_state::{
    GameDetail, GameDetailEvent, GameDetailStateMachine, GameDetailUiState,
};
