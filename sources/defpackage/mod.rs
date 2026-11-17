//! Rust model of the `defpackage` types used by the emulator exec chain
//! (GameHub Lite v5.1.8).
//!
//! Mirrors `sources/defpackage/` from the reference. Only the portable pieces
//! of `State`, `EmbeddedFileManager` / `RunEmbedded` and `ComponentRepo` are
//! modeled; Android/Kotlin metadata, Base64 PE payloads and xj-specific beans
//! are intentionally excluded. Layout follows `PLAN.md`: a `mod.rs` package
//! root with sibling `.rs` files for each modeled Java class.

pub mod component_repo;
pub mod embedded_file_manager;
pub mod state;

pub use component_repo::ComponentRepo;
pub use embedded_file_manager::{is_cached_version, EmbeddedFile};
pub use state::ComponentState;
