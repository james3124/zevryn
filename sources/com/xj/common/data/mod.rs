//! Rust model of the portable DTOs in `com.xj.common.data` (GameHub Lite
//! v5.1.8). Field names follow the decompiled `@SerializedName` / Room column
//! names exactly; parsing uses the local, dependency-free JSON reader in
//! `json.rs` (a mirror of the core crate's reader).

#[path = "json.rs"]
pub mod json;

pub mod common_download_task_state;
pub mod game_detail_entity;
pub mod game_library_table;

pub use common_download_task_state::CommonDownloadTaskState;
pub use game_detail_entity::GameDetailEntity;
pub use game_library_table::GameLibraryTable;
