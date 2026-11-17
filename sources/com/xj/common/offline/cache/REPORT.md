# Area report — Offline cache model (Rust port)

Read-only reference: `reference/gamehub-lite-v518-decompiled/gamehub-lite-v518-decompiled/`.
All modules are **std-only**: plain `pub` structs/enums, a `HashMap`-backed
store with an injectable clock, and a byte-capacity LRU policy. No external
crates, no real disk/network I/O. Layout follows the project convention — one
`mod.rs` with re-exports plus sibling `.rs` files at the mirrored Java package
path (`com/xj/common/offline/cache/`).

## Files created

| File | Contents |
|---|---|
| `mod.rs` | Package root: submodule declarations + re-exports (`CacheEntry`, `CacheKind`, `CacheStore`, `StringCacheStore`, `LruEvictor`, capacity constants) and 14 `#[cfg(test)]` tests. |
| `cache_entry.rs` | `CacheKind` (`GameDetail`/`GameIcon`/`GameConfig`/`ApiResponse`) and `CacheEntry<K = String, V = String>` with `key`, `kind`, `stored_at: u64`, `ttl_secs: u64`, `payload: V` (raw), plus `expires_at`, `is_expired_at`, `is_fresh_at`, `remaining_secs`, `refresh`; `system_now_secs()` default clock. |
| `store.rs` | `CacheStore<V = String>`: `new()` / `with_clock(...)`, `put`, `insert`, `get` (fresh-only), `get_entry`, `peek`, `get_with_policy` / `entry_with_policy` (stale-while-revalidate), `invalidate`, `expire`, `purge_expired`, `contains_key`/`len`/`is_empty`/`clear`/`iter`. |
| `lru.rs` | Evidence constants and `LruEvictor` (access-ordered, byte-counted capacity with `put`/`touch`/`remove`/`evict_to_capacity`), constructors `media3()` and `okhttp()`. |
| `REPORT.md` | This file. |

## Evidence note — the real offline caches in the reference

1. **Media3 `SimpleCache` — 1 GB LRU (offline video).**
   `sources/defpackage/Media3CacheUtils.java:40-50` builds, in a static block:
   - directory `<external-files-or-files>/<package>/egg_media3_cache`
     (`new File(base, "egg_media3_cache")`, line 43),
   - `new LeastRecentlyUsedCacheEvictor(1048576000L)` (line 44) — exactly
     **1,048,576,000 bytes = 1000 MiB (~1 GB)**, modelled as `MEDIA3_MAX_BYTES`,
   - `new SimpleCache(file, evictor, new EggExoplayerDatabaseProvider(app, null, 0, 6, null))`
     (line 47) — an SQLite index of cached spans,
   - `new CacheDataSource.Factory().setCache(...).setUpstreamDataSourceFactory(
     new DefaultHttpDataSource.Factory().setAllowCrossProtocolRedirects(true)).setFlags(2)`
     (line 49). `setFlags(2)` is Media3's `FLAG_IGNORE_CACHE_ON_ERROR`: cache
     errors fall back to the network upstream. The 1 GB LRU is why cached video
     keeps working offline and refreshes when connectivity returns.

   Supporting framework: `sources/androidx/media3/datasource/cache/{SimpleCache,
   LeastRecentlyUsedCacheEvictor,SimpleCacheSpan,CachedContent}.java`.
2. **OkHttp disk cache — 128 MB.**
   `sources/com/xj/common/http/EggGameHttpConfig.java:68-70` installs
   `initialize.cache(new Cache(context.getCacheDir(), 134217728L))` — exactly
   **134,217,728 bytes = 128 MiB**, modelled as `OKHTTP_MAX_BYTES`. The base URL
   is `https://gamehub-lite-api.emuready.workers.dev/` (line 55), so cached API
   responses (the `@SerializedName` DTOs) are the `GameDetail` / `ApiResponse`
   entries in this model.
3. **`com.drake.net` `ForceCache` — forced offline HTTP.**
   `sources/com/drake/net/NetConfig.java:152-160` obtains a `DiskLruCache` via
   `OkHttpUtils.diskLruCache(cache)` and wraps it in `ForceCache`;
   `sources/com/drake/net/cache/ForceCache.java` serves cached responses even
   when stale (a request is tagged with `ForceCache` in
   `NetOkHttpInterceptor.java:96`). This is the app's explicit
   stale-while-revalidate / forced-offline read path, modelled by
   `CacheStore::get_with_policy()`.
4. **Room structured local cache.**
   `GameSirUxDB_Impl` (v5, 11 tables) and `SteamDB_Impl` (v2, 3 tables) persist
   game/config DTOs locally — e.g. `t_game_library.data` (serialized JSON),
   `win_emu_trans_config_table.config`, `StarterGame.hasDownloaded`. The
   portable schema is in `../data/db/schema.rs` in this app.
5. **Mapping-report corroboration.**
   - `reference/mapping/04-androidx-framework.md:108-109,146,152`: media3
     `datasource/cache` (offline caching), `database` (ExoPlayer offline
     download cache), and Room as "Local structured DB / offline cache".
   - `reference/mapping/06-networking.md:12,37,68`: OkHttp 4.12.0 with
     `internal/cache/DiskLruCache`, and the `com.drake.net` HTTP stack.

## Modelled API

- `CacheEntry<K,V>` carries provenance (`kind`), `stored_at`/`ttl_secs` and the
  raw `payload`. Expiry is exclusive (`now >= stored_at + ttl_secs`) and
  overflow-safe (saturating add; `ttl_secs == 0` expires immediately).
- `CacheStore::get_with_policy(key) -> Option<(&V, bool)>` returns the value and
  an `is_stale` flag: `false` = within TTL, `true` = past TTL but still served
  so callers can display cached data while triggering a refresh. `get()` is the
  strict fresh-only variant; `peek()` ignores expiry; `expire()` removes a key
  only when stale; `invalidate()` removes unconditionally.
- `LruEvictor` tracks byte sizes and access order (monotonic sequence, like
  `LeastRecentlyUsedCacheEvictor`), evicting oldest-first until usage fits the
  capacity. `media3()` and `okhttp()` instantiate the exact byte limits above.

## Verification

- Compiled standalone with `rustc --edition 2021 --test` via a temporary
  harness wiring both offline `mod.rs` files; **24 tests pass, 0 fail** (14 in
  this area, 10 in `../capability/`). Clean under `RUSTFLAGS="-D warnings"`.
- Tests cover: exact byte constants and `setFlags(2)`, exclusive/overflow-safe
  expiry, `refresh`, all four `CacheKind` tokens, put/get/invalidate/expire with
  an injected clock (`Rc<Cell<u64>>`), purge-expired, the full
  fresh→stale→absent stale-while-revalidate sequence, LRU eviction order and
  touch-based victim selection, entry replacement/size accounting, and the
  oversized-entry case.

## Intentionally out of scope

- **No real disk or network I/O.** `SimpleCache`, `DiskLruCache`, the SQLite
  cache index (`EggExoplayerDatabaseProvider`), Room and OkHttp are *not*
  instantiated; only their policy constants and in-memory behaviour are
  modelled. Nothing is written to a file system and no HTTP request is made.
- **No serialization / codec**: payloads stay raw `String` (or a caller-chosen
  `V`). Parsing `@SerializedName` DTO JSON, Media3 span files, bitmaps, and
  protobuf/SQLite binary formats is separate.
- **No Android/ExoPlayer runtime**: `CacheDataSource` flags beyond the recorded
  constant, upstream data sources, background download scheduling, file locking
  (`SimpleCache.isCacheFolderLocked`), and actual eviction-on-disk are out of
  scope.
- **No wall-clock dependence in tests**: the default `system_now_secs()` is a
  plain clock read (not I/O); all expiry/SWR tests inject time explicitly.

## Wiring note for the parent session

Add to `app/src/lib.rs` (or an `offline` parent module):

```rust
#[path = "../sources/com/xj/common/offline/cache/mod.rs"]
pub mod cache;
```

No other files need changing; the module has no cross-area dependencies and
does not overlap the sibling `capability/` or `facade/` write areas.
