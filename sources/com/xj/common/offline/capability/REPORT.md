# Area report — Offline capability & availability (Rust port)

Read-only reference: `reference/gamehub-lite-v518-decompiled/gamehub-lite-v518-decompiled/`.
All modules are **std-only**: plain `pub` structs/enums, no external crates,
no disk/network I/O, no Android APIs. Layout follows the project convention —
one `mod.rs` with re-exports plus sibling `.rs` files at the mirrored Java
package path (`com/xj/common/offline/capability/`), to be wired from
`app/src/lib.rs` with a single `#[path]` attribute.

## Files created

| File | Contents |
|---|---|
| `mod.rs` | Package root: submodule declarations + re-exports (`OfflineCapability`, `OfflineLevel`, `DataSource`, `OfflinePolicy`, `is_available`, `availability_status`, `is_capability_available`, `AvailabilityStatus`, offline string constants) and 10 `#[cfg(test)]` tests. |
| `capability.rs` | `OfflineLevel` (`Full`/`Partial`/`OnlineOnly`), `DataSource` (Room/MMKV/files/Media3/OkHttp/network), `OfflineCapability` (9 variants), with `as_str()`, `level()`, `data_source()`, `rationale()` and `ALL` constants. |
| `policy.rs` | `OfflinePolicy`: static capability → level + primary data-source mapping, `capabilities_for_level`, free-function forms `default_level`/`default_data_source`, and a policy-aware `is_available` wrapper. |
| `availability.rs` | Pure `is_available(level, has_local_data, network_online, cache_fresh) -> bool` with the documented truth table, plus `availability_status(...)` (fresh / stale / online / unavailable) and `is_capability_available(...)`. |
| `strings.rs` | Byte-exact English offline strings from `res/values/strings.xml` and the `(resource name, value)` table `OFFLINE_STRINGS`. |
| `REPORT.md` | This file. |

## Exact local data sources (why each capability gets its level)

| Capability | Level | Primary source | Reference evidence (read-only) |
|---|---|---|---|
| `LocalGameLibrary` | Full | Room | `sources/com/xj/common/data/db/GameSirUxDB_Impl.java` (v5, 11 tables) — `t_game_library`, `StarterGame`; `sources/com/xj/common/data/table/GameLibraryTable.java` / `sources/com/xj/common/data/game_library_table.rs` (this app) stores the serialized game data in the `data` TEXT column. |
| `ImportGame` | Full | Files → Room | `GameLibraryTable.isAddFromLocal`, `filePath`, `local_game_id`; imported APK/ROM paths are local files, the record is written to Room `t_game_library`. Metadata enrichment via the signed API is optional and not required to import/launch. |
| `LaunchPcEmu` | Full | Room + files | `GameSirUxDB` tables `winemu_file_op_record`, `win_emu_trans_config_table`, `Emulator`, `Kernel` (`sources/com/xj/common/data/db/schema.rs` in this app); emulator components/versions are local. The first-time component download needs network, but launching an installed game does not (modelled by `has_local_data`). |
| `GamepadMapping` | Full | Files + Room + MMKV | `sources/com/xj/mapping/` (`*.icp` profiles, `*.xj` payloads), keymap config rows (Room `win_emu_trans_config_table.config`), and MMKV keys held by `InputControlsManager` (dynamic, game-ID keyed). |
| `Settings` | Full | MMKV / SharedPreferences | `sources/com/xj/common/data/preferences/AppPreferences.java`: SharedPreferences store `<pkg>_app_global_preferences`, MMKV store `_mmkv_app_global_preferences`, plus `gh-language` and `spUtils` (`SPUtils.f()`). Exact keys/defaults are catalogued in `../preference/REPORT.md`. |
| `GameDetailCached` | Partial | OkHttp/ForceCache (+ Room) | `@SerializedName` DTOs (e.g. `GameDetailEntity`, `GameLibraryTable.data`) persisted by `sources/com/xj/common/http/EggGameHttpConfig.java:68-70` (OkHttp `Cache`, 128 MB) and `sources/com/drake/net/cache/ForceCache.java` (OkHttp `DiskLruCache`, forced offline reads). Fresh values need `https://gamehub-lite-api.emuready.workers.dev/`, so stale-while-revalidate applies — this is the capability the `cache/` area's `get_with_policy()` serves. |
| `CloudPlay` | OnlineOnly | Network | `com.xj.cloud` / `com.streaming` (Limelight/Moonlight-derived) needs a live host. `res/values/strings.xml:689` `error_pc_offline` = "Computer is offline" when the host is unreachable. No local substitute. |
| `SocialStore` | OnlineOnly | Network | Explore / free-games / news / social feeds from the signed GameHub API (`EggGameHttpConfig.f23888b`, base URL above); no offline fallback. |
| `SteamOfflineMode` | Partial | Room (SteamDB) + network | `res/values/strings.xml:2100-2101`: `pc_cc_steam_offline_mode` = "Offline Mode", `pc_cc_steam_offline_mode_desc` = "Allow running verified Steam games without network connection". Requires prior online verification and cached rows in `SteamDB` (v2: `steam_account`, `t_steam_user_games`, `steam_download_entity`) — see `../data/db/schema.rs`. |

Supporting offline-state strings (byte-exact, in `strings.rs`, from
`res/values/strings.xml` lines 417, 689, 1893, 2100, 2101, 2255):
`comm_network_err` "Network error", `no_network` "Network Error.",
`error_pc_offline` "Computer is offline", `pc_cc_steam_offline_mode`
"Offline Mode", `pc_cc_steam_offline_mode_desc` "Allow running verified Steam
games without network connection", `pcview_menu_header_offline` "Offline".
All six also appear in `res/values/public.xml` with their resource IDs.

## Availability decision rules

`is_available(level, has_local_data, network_online, cache_fresh)`:

1. **Full** → `has_local_data`. Room/MMKV/files need no network; if nothing is
   stored yet the feature has nothing to act on even while online.
2. **Partial** → `has_local_data || network_online`. A stored copy (even stale,
   i.e. `cache_fresh == false`) is served via stale-while-revalidate; otherwise
   a live fetch is used. `cache_fresh` deliberately does **not** gate the
   boolean — `availability_status()` uses it to report `AvailableFresh` vs
   `AvailableStale`.
3. **OnlineOnly** → `network_online`. Local data and cache freshness are
   ignored.

The decision is pure and deterministic. The availability matrix test exercises
all 2³ = 8 state combinations for each of the 3 levels (24 assertions) and
cross-checks all three public entry points.

## Verification

- Compiled standalone with `rustc --edition 2021 --test` via a temporary
  harness wiring both `mod.rs` files; **24 tests pass, 0 fail** (10 in this
  area, 14 in `../cache/`). Clean under `RUSTFLAGS="-D warnings"`.
- Tests cover: static level/source mapping for all 9 capabilities, level/source
  consistency invariants, stable `as_str` tokens, the full availability matrix,
  Full/OnlineOnly edge cases, Partial stale-while-revalidate/offline cases, the
  Steam-offline prior-data rule, and byte-exact offline strings.

## Intentionally out of scope

- **No real disk or network I/O.** This area classifies *where* data lives and
  decides availability from booleans supplied by the caller; it never opens a
  Room/SQLite database, MMKV mmap file, media cache, or socket.
- **No Android/Room/MMKV runtime**: SQLite execution, migrations, MMKV native
  (`libmmkv.so`), `SharedPreferences` file persistence, lifecycle/ConnectivityManager
  callbacks and network-state detection are not modelled. The caller supplies
  `network_online`/`has_local_data`/`cache_fresh`.
- **No Steam/cloud implementation**: `SteamOfflineMode`, `CloudPlay` and
  `SocialStore` are classified and named only; the actual Steam DRM, streaming
  protocol and API client are separate (and, per `STRUCTURE.md`, out of rebuild
  scope).
- **No UI/policy loading**: levels are static Rust data, not a runtime
  configuration or server-provided feature flag.

## Wiring note for the parent session

Add to `app/src/lib.rs` (or an `offline` parent module):

```rust
#[path = "../sources/com/xj/common/offline/capability/mod.rs"]
pub mod capability;
```

No other files need changing; the module has no cross-area dependencies and
does not overlap the sibling `cache/` or `facade/` write areas.
