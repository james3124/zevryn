//! Offline support for the launcher (GameHub Lite v5.1.8).
//!
//! Models how the app works without constant network connectivity, grounded
//! in the reference: Room `GameSirUxDB`/`SteamDB`, MMKV KV, imported game
//! files, the 1 GB LRU Media3 cache (`egg_media3_cache`), the 128 MB OkHttp
//! disk cache, and the explicit Steam "Offline Mode".

pub mod capability;
pub mod cache;
pub mod facade;

pub use capability::*;
pub use cache::*;
pub use facade::*;
