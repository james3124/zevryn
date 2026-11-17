//! Rust reimplementation of the portable logic under
//! `com.xj.landscape.launcher.ui.main` in GameHub Lite v5.1.8.
//!
//! Submodules mirror the decompiled Java files:
//! * `me`               — dashboard DTOs/parsing (`MyFragment`, `MyVM`,
//!                         `MyContainerView`, `MyGamesData`,
//!                         `MyGamesItemData`, `CardLineData`, `CardLineType`)
//! * `main_adapter`     — `MainAdapterKt` row-to-holder dispatch rules
//! * `focus`            — D-pad focus navigation (`FocusableActivity`)
//! * `tabs`             — the two launcher tabs and the locale gate
//!                         (`TabItemData`, `LandscapeLauncherMainActivity`)
//!
//! Android views, DataBinding and RecyclerView are deliberately not modeled;
//! see `REPORT.md`.

pub mod me;

mod focus;
mod tabs;
#[path = "MainAdapterKt.rs"]
mod main_adapter;

pub use focus::*;
pub use main_adapter::*;
pub use me::*;
pub use tabs::*;
