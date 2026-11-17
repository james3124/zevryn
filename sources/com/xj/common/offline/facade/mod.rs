//! Rust model of the offline-first repository facade (GameHub Lite v5.1.8).
//!
//! Unifies the *local* data sources the crate already models so the launcher
//! can operate without a network: the game-library map (`GameLibraryStore`),
//! the settings KV (`InMemoryKv` / `Settings`), the `GameDetailEntity` cache,
//! the Room schema (`GameSirUxDB` / `SteamDB`) and the Media3 / OkHttp caches.
//! All of it is represented in memory here: no Room, MMKV, disk or network I/O.
//!
//! Self-contained (std only); wire from `app/src/lib.rs` with a single
//! `#[path]` attribute, e.g.
//!
//! ```ignore
//! #[path = "../sources/com/xj/common/offline/facade/mod.rs"]
//! pub mod offline_facade;
//! ```

pub mod offline_source;
pub mod outbox;
pub mod report;
pub mod sync;

pub use offline_source::{
    LocalDataSource, OfflineDataSource, OfflineMode, GameDetailEntity, GameLibraryTable,
    Settings, ERR_COMMON_NETWORK, ERR_NO_NETWORK, ERR_PC_OFFLINE, STR_STEAM_OFFLINE_MODE,
    STR_STEAM_OFFLINE_MODE_DESC,
};
pub use outbox::{Outbox, OutboxItem};
pub use report::OfflineReport;
pub use sync::{decide, CachePolicy, CacheState, SyncDecision};
