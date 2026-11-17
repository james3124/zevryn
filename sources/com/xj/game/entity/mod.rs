//! Rust model of `com.xj.game.entity` (GameHub Lite v5.1.8).
//!
//! Portable, std-only reimplementation of `ImportedGameEntity` plus the
//! game-type codes and a field-compatible mirror of the app's existing
//! storage entity (`GameLibraryTable`). The `ImportedGame -> GameLibraryTable`
//! conversion lives here so it sits next to both types.
//!
//! Self-contained: no external crates and no reliance on a particular
//! `#[path]` wiring in `app/src/lib.rs`. It can be compiled on its own, e.g.
//!
//! ```ignore
//! #[path = "sources/com/xj/game/entity/mod.rs"]
//! pub mod game_entity;
//! ```

mod conversion;
mod game_type;
mod imported_game;
mod steam_game_info;
mod storage_entity;

pub use conversion::LAUNCH_TYPE_PC_EMULATOR;
pub use game_type::GameType;
pub use imported_game::ImportedGame;
pub use steam_game_info::SteamGameInfo;
pub use storage_entity::GameLibraryTable;
