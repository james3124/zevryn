//! Rust model of the `com.xj.common.data.dao` package (portable operations only).
//!
//! The Room `@Dao` interfaces are modeled as an enum of DAO kinds and an enum
//! of CRUD operations, since their concrete signatures depend on Android
//! `Flow`/`LiveData`/coroutine types that are not portable.

pub mod operations;

pub use operations::*;
