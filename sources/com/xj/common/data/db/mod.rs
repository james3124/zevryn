//! Rust model of the `com.xj.common.data.db` package (schema description only).
//!
//! Mirrors the Room databases of GameHub Lite v5.1.8 (`GameSirUxDB`, `SteamDB`)
//! and the legacy `com.xj.mapping.db.DBHelper`. Only the *schema* is modeled:
//! table/column metadata plus a generator that emits SQLite `CREATE TABLE`
//! text. There is no SQLite dependency — executing the SQL is future work.

pub mod schema;

pub use schema::*;
