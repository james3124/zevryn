//! Portable, dependency-free Rust model of the GameHub Lite v5.1.8 game
//! library UI package (`sources/com/xj/game/ui/game_library/`).
//!
//! Mirrors the *state and logic* of `GamesFragment`, `GameLibraryActivity`,
//! the adapters/viewholders and `SelectImportGameTypeDialog` — sorting,
//! filtering, search, view mode, dialog selection and event→action mapping —
//! without any Android `Fragment`/`Adapter`/`View`/file-picker types. Only
//! `std` is used. See `REPORT.md` for file inventory, exact decompiled strings
//! and out-of-scope items.

#[path = "game_entry.rs"]
pub mod game_entry;

#[path = "ui_state.rs"]
pub mod ui_state;

#[path = "import_dialog.rs"]
pub mod import_dialog;

#[path = "import_event.rs"]
pub mod import_event;

// Re-exports (flat, plain pub API).
pub use self::game_entry::{
    filter, sort, GameEntry, GameType, LibraryFilter, SortOrder,
};
pub use self::ui_state::{GameLibraryUiState, ViewMode};
pub use self::import_dialog::{ImportChoice, ImportDialogModel};
pub use self::import_event::{
    action_from_dialog, process_import_request, request_from_dialog, ImportRequest, UiAction,
};
