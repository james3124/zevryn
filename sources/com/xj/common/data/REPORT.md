# Area report — Room databases / tables / DAOs (Rust port)

Reference (read-only): `sources/com/xj/common/data/{db,table,dao}/`,
`GameSirUxDB_Impl.java`, `SteamDB_Impl.java`, `com/xj/mapping/db/DBHelper.java`
and the Room `@Entity` model classes under `sources/com/xj/common/data/model/`
and `sources/com/xj/standalone/steam/data/{db/tables,bean}/`
of GameHub Lite v5.1.8.

All modules are **dependency-free (std only)**: plain structs/enums plus SQL
text generation, with `#[cfg(test)]` unit tests. Layout follows the project
convention — a Rust `mod.rs` plus sibling `.rs` files at the mirrored Java
package path, to be wired from `app/src/lib.rs` with `#[path]` attributes
(each `mod.rs` is self-contained; no cross-area references).

## Files created

| File | Contents |
|---|---|
| `table/entities.rs` | Plain Rust structs mirroring every Room `@Entity`: `GameLibraryTable`, `WinEmuFileOpRecord`, `WinEmuTranslationConfigEntity` (+ `ComponentInstallInContainer{,List}`, `DepComponentFileOp{,Ext,List}` value classes) and the entity model classes `StarterGame`, `EmulatorInfo`, `KernelInfo`, `GameMainTagEntity`, `ScreenCaptureEntity`, `ScreenRecordEntity`, `SteamAccountTable`, `SteamDownloadEntity`, `SteamUserTable`, `SteamUserGameTable`, `SteamModuleDownloadEntity`. Java nullable/boxed fields -> `Option<_>`; converter-backed fields kept as serialized `TEXT` (`String`); `TimestampMillis = i64`. |
| `table/mod.rs` | Package root: submodule declaration + re-exports. |
| `db/schema.rs` | Schema description + SQLite DDL text generator. `SqlType`, `SqlStyle` (`Room`/`Legacy`), `Column`, `Table`, `Schema`. `Schema::create_table_sql`, `create_all_sql`, `drop_all_sql`, `table_names`. Schema constructors `game_sir_ux_db()` (11 entity tables, v5), `steam_db()` (3 entity tables, v2), `legacy_game_info_db()` (`gameinfo.db` v7, 5 tables), plus `all_schemas()`. Room identity hashes, `ROOM_MASTER_TABLE_SQL` and the legacy `mygame` migration `ALTER TABLE` statements recorded as constants. |
| `db/mod.rs` | Package root: submodule declaration + re-exports. |
| `dao/operations.rs` | Portable DAO operation enums: `DaoAnnotation` (Room `@Insert`/`@Update`/`@Delete`/`@Query`, with `from_annotation`), `DaoOperation` (CRUD operation shapes, with `annotation()`), `DaoKind` (the 11 `@Dao` interfaces, with `operations()`). |
| `dao/mod.rs` | Package root: submodule declaration + re-exports. |

## Verification

- Compiled standalone with `rustc --edition 2021 --test` via a temporary harness
  wiring the three `mod.rs` files; **17 tests pass, 0 fail**.
- The generated `CREATE TABLE` statements were diffed byte-for-byte against the
  strings in `GameSirUxDB_Impl.createAllTables()` and
  `SteamDB_Impl.createAllTables()` (read-only reference): **all 16 statements
  match exactly**, including column order, `AUTOINCREMENT`, `NOT NULL`,
  `DEFAULT` literals and table-level `PRIMARY KEY(...)` constraints.

## Intentionally out of scope

- **SQLite execution needs a future dependency.** This area only *generates*
  SQL text; it does not open a database or run statements. Executing the emitted
  DDL/DML requires adding a driver such as `rusqlite` (bundled SQLite) or
  `sqlx` to `app/Cargo.toml` — deliberately not done here (no external deps).
- **Room runtime behavior**: `RoomDatabase`, `RoomOpenDelegate`, migrations,
  `Migration`/`AutoMigration` classes, `InvalidationTracker`, type converters,
  schema validation/hash checks and the singleton `Mutex`/coroutine `get()`
  builders are not modeled — they are Android/Room framework plumbing.
- **DAO implementations**: `*_Impl` classes, SQL binding, `Flow`/`LiveData`,
  `Continuation`/suspend bridges and @Query SQL strings are abstracted away;
  only the portable operation categories are kept.
- **TypeConverter details** (`DateConverter`, `StringListConverter`, etc.) are
  not implemented; converter-backed fields are represented as their stored
  `TEXT` form.

## Wiring note for the parent session

Add to `app/src/lib.rs` (or a `data` parent module) three `#[path]` declarations
pointing at `sources/com/xj/common/data/{db,table,dao}/mod.rs`. No `data/mod.rs`
is provided by this area to avoid overlapping the three assigned directories;
each of the three `mod.rs` files can be included directly and independently.
