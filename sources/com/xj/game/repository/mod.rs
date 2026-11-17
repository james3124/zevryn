//! Rust model of `com.xj.game.repository` (GameHub Lite v5.1.8) — the
//! in-memory game-library store and its import outcome.
//!
//! Self-contained (std only) and independent of a specific `#[path]` wiring in
//! `app/src/lib.rs`: the entity model is pulled in from a sibling area with a
//! relative `#[path]` inside `game_library_store.rs`.
//!
//! ```ignore
//! #[path = "sources/com/xj/game/repository/mod.rs"]
//! pub mod game_repository;
//! ```

pub mod game_library_store;

pub use game_library_store::{is_local_game_id, is_pc_game_installed, GameLibraryStore, ImportResult};
// Re-export the model types from the store's bundled entity view so callers of
// this module use one consistent set of types.
pub use game_library_store::entity_model::{
    GameLibraryTable, GameType, ImportedGame, SteamGameInfo, LAUNCH_TYPE_PC_EMULATOR,
};
