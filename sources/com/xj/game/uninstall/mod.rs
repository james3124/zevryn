//! Rust port of GameHub Lite v5.1.8's "uninstall game / remove from library"
//! flow triggered from the game long-press menu.
//!
//! Reference (read-only):
//! * `sources/com/xj/game/UninstallGameHelper.java` (+ its `$uninstallGame$2`,
//!   `$uninstallPcDemoGame$2`, `$getGameFrom$1`, `$isNeed2UninstallGame$1`
//!   continuations)
//! * `sources/com/xj/game/UninstallGameConfirmHelper.java` (+ its
//!   `$showRemoveGameConfirmDialog$1`, `$showRemovePcGameConfirmDialog$1`,
//!   `$toUninstallPcGame$1` continuations)
//! * `sources/com/xj/common/utils/RemoveOrUninstallGameListener.java`
//! * `sources/com/xj/common/dialog/CommUninstallConfirmDialog.java`
//! * `resources/res/values/strings.xml` — `winemu_uninstall_game_title`
//!
//! * [`game_from`] — [`GameFrom`], the four exact enum constants of
//!   `UninstallGameHelper.GameFrom`.
//! * [`uninstall_step`] — [`UninstallStep`], each action with its source method.
//! * [`uninstall_plan`] — [`Game`], [`UninstallFlags`] and [`UninstallPlan`],
//!   the ordered plan builder mirroring the helper's decision tree.
//! * [`uninstall_confirm`] — [`UninstallConfirm`] and the listener trait, the
//!   `(boolean, boolean, String)` callback model.
//! * [`executor`] — [`RemoveLibrary`], [`UninstallResult`], [`Execution`] and
//!   [`execute`], a pure state-transition executor.
//!
//! Real file deletion, Android dialogs/coroutines and native (Steam/emulator)
//! calls are intentionally out of scope — see `REPORT.md`.

pub mod executor;
pub mod game_from;
pub mod uninstall_confirm;
pub mod uninstall_plan;
pub mod uninstall_step;

pub use executor::{
    execute, execute_confirmed, Execution, RemoveLibrary, UninstallResult,
};
pub use game_from::GameFrom;
pub use uninstall_confirm::{
    NoopConfirmListener, UninstallConfirm, UninstallConfirmListener,
};
pub use uninstall_plan::{Game, UninstallFlags, UninstallPlan};
pub use uninstall_step::UninstallStep;

/// String resource key of the confirmation dialog title.
///
/// `resources/res/values/strings.xml:3391` — "Uninstall game"
/// (`values-zh`: "卸载游戏", `values-ja`: "ゲームをアンインストール").
pub const UNINSTALL_GAME_TITLE_KEY: &str = "winemu_uninstall_game_title";
pub const UNINSTALL_GAME_TITLE_EN: &str = "Uninstall game";
pub const UNINSTALL_GAME_TITLE_ZH: &str = "卸载游戏";
