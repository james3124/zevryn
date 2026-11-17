# REPORT — `com.xj.game.entity` + `com.xj.game.repository` (Rust reimplementation)

Target: GameHub Lite v5.1.8. Read-only reference root:

```
/home/Project/Zevryn/reference/gamehub-lite-v518-decompiled/gamehub-lite-v518-decompiled/
```

Both areas are **std-only**, use plain `pub` structs/enums, and carry `#[cfg(test)]`
unit tests. Write areas: `app/sources/com/xj/game/entity/` and
`app/sources/com/xj/game/repository/`. No other files were touched.

## Files

### `app/sources/com/xj/game/entity/`
- `mod.rs` — module root; re-exports all public types.
- `game_type.rs` — `GameType` enum (exact codes `0/1/2`) + tests.
- `steam_game_info.rs` — dependency-free `SteamGameInfo` stand-in (mirrors the
  decompiled `SteamGameInfo.java` field names) + tests.
- `imported_game.rs` — `ImportedGame`, faithful mirror of the eleven fields of
  `ImportedGameEntity.java`; `new()` uses the Java constructor's argument order
  + tests.
- `storage_entity.rs` — field-compatible mirror of the app's existing
  `GameLibraryTable` (same field names/types as
  `app/sources/com/xj/common/data/game_library_table.rs`). Re-declared rather
  than `#[path]`-imported because the real file needs `super::json`, which would
  tie this area to a particular wiring in `app/src/lib.rs`. See note below.
- `conversion.rs` — `From<ImportedGame> for GameLibraryTable` and
  `LAUNCH_TYPE_PC_EMULATOR`; builds the `data` JSON with a tiny std-only escaper
  + tests.
- `REPORT.md` — pointer back to this file.

### `app/sources/com/xj/game/repository/`
- `mod.rs` — module root; re-exports store, outcome enum and model types.
- `game_library_store.rs` — `GameLibraryStore` (HashMap keyed by id),
  `ImportResult`, `is_local_game_id`, `is_pc_game_installed`, and CRUD/dedupe
  tests. Pulls in the entity model via a relative `#[path = "../entity/mod.rs"]`
  so this area also compiles standalone.
- `REPORT.md` — this file.

### Note on `GameLibraryTable` identity
`entity::storage_entity::GameLibraryTable` intentionally mirrors the existing
`com::xj::common::data::GameLibraryTable` field-for-field but is a separate Rust
type in a different module. This keeps each area self-contained (the real file
depends on its sibling `json` module). Because `repository` re-exports the type
from its own bundled `entity_model`, consumers of a single module get one
consistent set of types. If the orchestrator wires both areas into the crate,
use one area's re-exports consistently when calling `GameLibraryStore::add`.

## `ImportedGame` → `GameLibraryTable` field mapping

Sources: `ImportedGameEntity.java` (fields / synthetic ctor default),
`GameLibraryRepository.java` `addImportGame` (method `f`, lines ~853–921) and
`addPcGame` (method `h`, lines ~932–1144), and
`GameLibraryTable.java` (ctor lines 68–83 / synthetic ctor 270–271).

| `ImportedGame` field (Java) | `GameLibraryTable` column | Rust field | Mapping rule / source |
|---|---|---|---|
| `id` (String) | `id` | `id: i64` | `Integer.parseInt(id)`, failure → `0`, widened to long. Java `f`: `Integer.parseInt(importedGameEntity.getId())` in try/catch. |
| `id` (String) | `game_id` | `game_id: Option<i32>` | `Some(parsed_id)` (non-null `Integer` passed to ctor in `h`). |
| `id` (String) | `data` (JSON) | `data` | `"id"` key of the portable `data` JSON. |
| `filePath` | `package_name` | `package_name: String` | `addPcGame` passes `gameDetailEntity.getFilePath()` as `packageName`. |
| `filePath` | `data` (JSON) | `data` | `"filePath"` key. |
| `localGameId` | `local_game_id` | `local_game_id: Option<String>` | `Some(localGameId)` (copied verbatim by `addImportGame`). |
| `localGameId` | `data` (JSON) | `data` | `"localGameId"` key. |
| `gameType` | `game_type` | `game_type: i32` | Raw code copied from the entity. **Note:** the Java `addPcGame` ctor call actually passes `gameType = 0` (a const), ignoring the entity's value; we preserve the entity code as the task requires. |
| `gameType` | `data` (JSON) | `data` | `"gameType"` key. |
| `cover` | `data` (JSON) | `data` | `"cover"` key (also `GameDetailEntity.cover`; no dedicated column). |
| `description` | `data` (JSON) | `data` | `"description"` key. |
| `exeFileBgType` | `data` (JSON) | `data` | `"exeFileBgType"` key. |
| `name` | `data` (JSON) | `data` | `"name"` key. |
| `size` | `data` (JSON) | `data` | `"size"` key. |
| `steamAppId` | `data` (JSON) | `data` | `"steamAppId"` key; Java maps `null` → `""`. |
| `steamGameInfo` | `data` (JSON) | `data` | When present, Java calls `setSteamInfo(...)`; here we add a `"steamGameInfo"` key with the app id. |

Columns **not** sourced from `ImportedGame` (set by the surrounding repository):

| Column | Value here | Java source |
|---|---|---|
| `user_id` | `""` | `String.valueOf(UserManager.INSTANCE.getUid())` — no user context in this area. |
| `launch_type` | `1403` | `LauncherTypeEnum.START_TYPE_PC_EMULATOR`. |
| `local_mobile_app_id` | `None` | `GameDetailEntity`'s `localMobileAppId` is `null` in `addImportGame`. |
| `time` | `None` | `new Date(System.currentTimeMillis())`; timestamping left to caller. |
| `is_last_launched` | `false` | literal `false` in the `addPcGame` ctor call. |

`data` note: Java stores the full Gson serialization of `GameDetailEntity`. The
portable model emits a small, valid JSON object containing the ImportedGame
fields above (stdlib-only string escaping) — the subset needed for this
reimplementation.

### Add/dedupe semantics
- `add(game)` = `add_import(game, replace = true)`: convert and **insert or
  update by id** (Room `insertOrUpdate`). Valid input → `Success`.
- `add_import(game, replace = false)`: first call the equivalent of
  `findByPkgAndLaunchType(package_name, launch_type)`; if a row exists, return
  `AlreadyExists` without replacing (Java logs "已经存在该游戏，且不替换" and
  returns null); otherwise insert.
- Invalid input (empty `id` or empty `filePath`) → `Invalid`.
- Helpers `is_local_game_id` and `is_pc_game_installed` mirror
  `GameLibraryRepository.Companion.a` and method `A` respectively; external
  `GameStateMgr` installed ids are passed in as `&[i32]`.

## Game type codes (exact, not invented)

From `sources/com/xj/common/data/model/GameType.java` (lines 20–22):

| Enum constant | Code | Meaning |
|---|---|---|
| `DEFAULT` | `0` | default; also the synthetic-constructor default (`GameType.DEFAULT.getValue()`) |
| `MOBILE_INSTALL_APP` | `1` | installed mobile app |
| `PC_STEAM_IMPORT` | `2` | imported PC/Steam game |

`GameType::from_code` rejects any other value. Do **not** confuse `1537` in the
`addPcGame` constructor call (line 969) with a game type: it is Kotlin's
synthetic-constructor default-argument **bitmask** (`1 | 512 | 1024`) in
`GameLibraryTable.java` lines 270–271; the `gameType` argument there is the
preceding literal `0`.

Launch-type constant used for `launch_type`: `1403` =
`LauncherTypeEnum.START_TYPE_PC_EMULATOR`, from
`sources/com/xj/common/data/launch/LauncherTypeEnum.java` line 30.

## Out of scope
- Real Room/SQLite execution, SQL generation and DAO coroutine plumbing
  (`Continuation`, `Flow`, `LiveData`, `SuspendKt`).
- Network / service routing (`TheRouter`, `IWinEmuService`, `BodyRequest`) and
  channel events (`AddPcGameLibraryEvent`, `UpdatePcGameFilePathEvent`).
- Full Gson `GameDetailEntity` serialization/deserialization and Android
  `Parcelable`/`@Parcelize` machinery (only a portable `data` JSON subset is emitted).
- Icon/bitmap work: `saveIcon2LocalIfNeed`, `IWinEmuService.p/k`, `ImageUtils`,
  `FileUtils`, and local icon path generation (`getSaveIconPath`).
- Android file-system/process inspection (`GameStateMgr` is modeled as a slice
  of installed ids) and `UserManager` uid lookup (empty string here).
- `get`/`all` return owned or borrowed values sorted by id; HashMap iteration
  order is otherwise unspecified.
