//! Rust model of the portable DTOs in `com.xj.common.data.gameinfo`
//! (GameHub Lite v5.1.8): game-compatibility presets and per-game records.
//! Field names follow the decompiled `@SerializedName` annotations exactly.

pub mod compatibility;

pub use compatibility::{
    find_preset_by_level, CompatibilityCatalog, CompatibilityItem, CompatibilityPreset,
    SimpleGameCompatibility,
};
