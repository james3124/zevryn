# REPORT — `com.xj.common.offline.facade` (Rust reimplementation)

Target: GameHub Lite v5.1.8. Read-only reference root:

```
/home/Project/Zevryn/reference/gamehub-lite-v518-decompiled/gamehub-lite-v518-decompiled/
```

Write area (only files created here): `app/sources/com/xj/common/offline/facade/`.
No other files were touched — `Cargo.toml`, `app/src/lib.rs` and other agents'
areas were left untouched.

## Goal

An **offline-first repository facade** that unifies the local data sources our
crate already models, so the launcher can operate with no network. All modules
are **std-only**, use plain `pub` structs/enums plus one small trait, and carry
`#[cfg(test)]` unit tests (10 tests, all passing). Nothing here performs I/O.

## Files

| File | Contents |
|---|---|
| `mod.rs` | Module root: four `pub mod` declarations and re-exports of every public type/constant. |
| `offline_source.rs` | `OfflineDataSource` trait + in-memory `LocalDataSource` (composes the game-library map, a settings snapshot and a game-detail cache). Re-declares minimal field-compatible `GameLibraryTable`, `GameDetailEntity`, `Settings`; adds the `OfflineMode` setting and verified string constants. CRUD/details/settings tests. |
| `sync.rs` | `SyncDecision` (`NetworkFirst` / `CacheFirst` / `LocalOnly` / `NeedSync`), `CacheState`, `CachePolicy`, `decide(network_available, cache_state)` and `error_key_for`. Decision tests. |
| `outbox.rs` | `OutboxItem` (`AddGame` / `RemoveGame` / `UpdateSetting`) + FIFO `Outbox` with `enqueue` / `drain` / `pending` and convenience enqueue helpers. Drain tests. |
| `report.rs` | `OfflineReport` (game count, cached details, pending outbox items, settings loaded) with `summary()` / `Display`. State/format tests. |
| `REPORT.md` | This file. |

## Local sources composed

The facade presents one local-only surface over the data sources that make
offline operation real in v5.1.8:

1. **Game library store** — the crate's `com::xj::game::repository::GameLibraryStore`
   (Room `GameSirUxDB.gameLibraryDao()`, table `t_game_library`). Modeled as a
   `HashMap<i64, GameLibraryTable>` keyed by row id; `list_games()` returns rows
   sorted by id. `GameSirUxDB` also owns the `ScreenRecord`, `StarterGame`,
   `WinEmuFileOpRecord` and `win_emu_trans_config` tables — our crate carries
   their byte-exact DDL in `common/data/db/schema.rs` (the facade composes the
   library table; the others remain in that area).
2. **Settings KV** — the crate's `com::xj::common::preference` (`InMemoryKv`,
   `KvStore`, `Settings`), i.e. Android SharedPreferences and Tencent **MMKV**
   (`libmmkv.so`). Modeled as one `Settings` snapshot with `get_settings` /
   `save_settings`; `settings_loaded` mirrors reading the store into memory.
3. **Game-detail cache** — the crate's `GameDetailEntity` (`common/data`), keyed
   by a stable id (server id or `localGameId`), with `get_game_detail` /
   `put_game_detail`. This is the locally cached payload a `CacheFirst` hit reads.
4. **Room `SteamDB`** and **imported game files on disk** are the other local
   game sources in the app; they are named here for completeness but their
   modeling lives in `common/data/db` and the game areas, not in this facade.

### Verified Steam offline-mode setting

Steam offline mode is an explicit feature in v5.1.8:

- `pc_cc_steam_offline_mode` = **"Offline Mode"**
- `pc_cc_steam_offline_mode_desc` =
  **"Allow running verified Steam games without network connection"**

Modeled as `OfflineMode { Disabled, Enabled }` and the `Settings.steam_offline_mode`
field (default `Disabled`), with the two resource keys exported as
`STR_STEAM_OFFLINE_MODE` / `STR_STEAM_OFFLINE_MODE_DESC`. `Enabled` means verified
Steam games are runnable offline. The change is recorded through the outbox.

### Caches behind `SyncDecision`

`CacheFirst` / `NetworkFirst` mirror real caching layers in the app:

- **OkHttp 128 MB disk cache** and **drake `ForceCache` (`DiskLruCache`)**, with
  cache-control overrides `noCache` (ignore cache → `NetworkFirst`) and `noStore`
  (don't write the cache). Modeled by `CachePolicy { NoCache, NoStore, ForceCache }`
  (default `ForceCache` → `CacheFirst` on a fresh hit).
- **Room rows + Media3 offline cache** supply the persisted payloads that
  `CacheState::{Fresh, Stale, Empty}` describes.

### Network error keys

Exported as constants and surfaced by decisions/callers when offline work cannot
be satisfied:

- `comm_network_err` (`ERR_COMMON_NETWORK`) — generic network error; used for
  `NeedSync`.
- `no_network` (`ERR_NO_NETWORK`) — used for `LocalOnly`.
- `error_pc_offline` (`ERR_PC_OFFLINE`) — PC-game offline error (available to
  callers launching offline PC games).

## Decision rules (`decide`)

| Network | Cache state | Decision |
|---|---|---|
| unavailable | any | `LocalOnly` |
| available | `Empty` | `NetworkFirst` |
| available | `Stale` | `NeedSync` |
| available | `Fresh` | `CacheFirst` |

The function is total and deterministic over its input domain. No fetch is
performed — callers interpret the decision.

## Outbox

Mutations made while offline are appended to a FIFO `Outbox` as `AddGame`,
`RemoveGame` or `UpdateSetting` items. `drain()` returns all pending items in
enqueue order and empties the queue for the next session — this models
"changes made offline, synced when online" without any real network. The
convenience `enqueue_settings_snapshot` records the Steam offline-mode toggle
change as an `UpdateSetting` item.

## Verification

Compiled standalone with `rustc --edition 2021 --test` via a temporary harness
that wires `mod.rs` as `pub mod offline_facade;`: **10 tests pass, 0 fail**, and
no warnings (also clean under `RUSTFLAGS="-D warnings"`). Tests cover local
game CRUD (insert/update/remove/ordering), detail + settings CRUD (including
the Steam offline-mode toggle), every `decide` branch and cache/policy helpers,
network-error keys, FIFO outbox enqueue/drain + settings-note, and report
state/summary after mutation.

## Wiring note for the parent session

Add to `app/src/lib.rs`:

```rust
#[path = "../sources/com/xj/common/offline/facade/mod.rs"]
pub mod offline_facade;
```

No other files need changing; the module has no cross-area dependencies. The
re-declared model types are field-compatible with the crate's real types —
convert at the wiring boundary if a caller needs the canonical types.

## Out of scope

- **No real Room/SQLite**: no SQL execution, DAO coroutine plumbing
  (`Flow`/`LiveData`/`Continuation`) or migration logic — only the in-memory
  map mirroring `t_game_library`; the schema itself lives in `common/data/db`.
- **No real MMKV / SharedPreferences**: no `libmmkv.so`, mmap/protobuf files,
  XML persistence or `Editor.apply()` — only the in-memory settings snapshot.
- **No network**: no OkHttp, Retrofit, drake downloader, TheRouter service calls
  or actual HTTP fetches; `SyncDecision` only *decides*, it never connects.
- **No disk I/O**: no DiskLruCache/Media3 cache files, no imported-game file
  access — all state is in memory for the lifetime of `LocalDataSource`.
- **No Android runtime**: no Context, Parcelable, resources manager or strings
  XML; the verified string keys are carried as inert `const`s.
- No persistence across restarts; a fresh `LocalDataSource` starts empty with
  default settings.
