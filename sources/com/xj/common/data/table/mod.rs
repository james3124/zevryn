//! Rust model of the `com.xj.common.data.table` package (and the Room
//! `@Entity` model classes registered with the app's two Room databases).
//!
//! Mirrors `sources/com/xj/common/data/table/` from GameHub Lite v5.1.8.
//! Entities are plain data structs: no Android/Room types, no logic.

pub mod entities;

pub use entities::*;
