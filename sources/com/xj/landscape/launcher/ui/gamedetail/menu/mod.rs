//! Rust port of the portable game-card long-press menu logic from
//! `com.xj.landscape.launcher.ui.gamedetail.GameDetailSettingMenu` and
//! `com.xj.landscape.launcher.ui.main.me.MyGamesOperationPopupKt`
//! (GameHub Lite v5.1.8).
//!
//! * [`option`] — [`MenuOptionId`] / [`MenuOption`] with the exact labels and
//!   string-resource names found in the reference build.
//! * [`assembly`] — [`MenuContext`] flags and [`build_menu`], mirroring the
//!   order and de-duplication of `getMenuOptions`.
//! * [`action`] — [`MenuAction`] and [`resolve`], replacing the Android click
//!   lambdas.
//! * [`state`] — [`MenuState`] (open/closed + selected index).
//!
//! Android `XPopup` rendering, coroutine dispatch and all click callbacks are
//! intentionally out of scope (see `REPORT.md`).

pub mod action;
pub mod assembly;
pub mod option;
pub mod state;

pub use action::{resolve, MenuAction};
pub use assembly::{build_menu, menu_option_ids, MenuContext};
pub use option::{MenuOption, MenuOptionId};
pub use state::MenuState;
