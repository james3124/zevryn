# Area report — SharedPreferences / MMKV settings (Rust port)

Reference (read-only): `reference/gamehub-lite-v518-decompiled/gamehub-lite-v518-decompiled/sources/`
— specifically:

- `com/xj/common/data/preferences/AppPreferences.java` (the central settings holder)
- `com/xj/common/data/preferences/MMKVExtensionsKt.java`, `PreferencesExtensionsKt.java`
  (generic `field`/`getValue`/`putValue` delegates — no keys of their own)
- `com/xj/base/language/GHLocaleManager.java` (locale preference)
- `com/xj/app/SplashActivity.java` (`uminit`, `intro_video_prefs`)
- `com/blankj/utilcode/util/SPUtils.java` (default file name for `SPUtils.f()`)
- `tv/danmaku/ijk/media/player/IjkMediaMeta.java` (resolves `IJKM_KEY_LANGUAGE`)

All modules are **dependency-free (std only)**: plain `pub` structs/enums plus a
small `KvStore` trait with an in-memory `HashMap` implementation and `#[cfg(test)]`
tests. Layout follows the project convention — a `mod.rs` with re-exports plus
sibling `.rs` files at the mirrored Java package path, to be wired from
`app/src/lib.rs` with a single `#[path]` attribute (as `pub mod preference;`).

## Files created

| File | Contents |
|---|---|
| `mod.rs` | Package root: four `pub mod` declarations and re-exports (`SettingsKey`, `KvStore`, `Value`, `InMemoryKv`, `Settings`, `migrate`, `migrate_all`, `LEGACY_KEY_MAP`). |
| `keys.rs` | `SettingsKey` enum — one variant per exact persisted key, `as_str()`, `all()`, `Display`. |
| `kv.rs` | `Value` enum (`Str`/`Bool`/`Int`), `KvStore` trait (raw `get`/`set`/`remove`/`contains`, typed `get_string`/`get_bool`/`get_i64` + `set_*`, `get_f64`/`set_f64` for the Java float), `InMemoryKv` (`HashMap`). |
| `settings.rs` | `Settings` struct whose `Default` reproduces the Java initializer values; `from_store` / `to_store` over any `KvStore`. |
| `migration.rs` | Generic `migrate(old, new)` and `migrate_all`, plus `LEGACY_KEY_MAP` with the two documented property-name → stored-key pairs. |
| `REPORT.md` | This file. |

## Exact keys found (byte-for-byte from the decompiled literals)

### `AppPreferences` — two separate stores

Store names (not keys):
- SharedPreferences: `context.getPackageName() + "_app_global_preferences"`
  (`AppPreferences.java:1529-1533`)
- MMKV: `"_mmkv_app_global_preferences"` (`AppPreferences.java:1544-1546`)

| Stored key | Kotlin property / Java local | Type | Java default | Store | Source |
|---|---|---|---|---|---|
| `uniqueId` | `deviceUniqueId` (`str6`, line 94) | String | `""` (`str7`, line 98) | SharedPreferences | lines 94-98, 1556-1559, 1585-1588 |
| `lastMyGamesScrollPosition` | `lastScrollPositionByMyGames` (`str8`, line 301) | int | `-1` (`i3`, line 300) | SharedPreferences | lines 300-305, 1565-1567, 1598-1600 |
| `firstInstallTime` | `firstInstallTime` (`str`, line 71) | long | `-1L` (`j3`, line 505) | SharedPreferences | lines 71, 505-509, 1561-1563, 1590-1592 |
| `lastVisibleExploreTime` | `lastVisibleExploreTime` (`str2`, line 72) | long | `-1L` (`j4`, line 709) | SharedPreferences | lines 72, 709-713, 1569-1571, 1602-1604 |
| `isFirstShowPlayInSecondsTab` | `isFirstShowPlayInSecondsTab` (`str3`, line 73) | boolean | `true` (`Boolean.TRUE`, line 913) | SharedPreferences | lines 73, 913-917, 1581-1595 |
| `wineScreenBrightnessSwitch` | `wineScreenBrightnessSwitch` (`str4`, line 74) | boolean | `false` (`Boolean.FALSE`, line 1115) | MMKV | lines 74, 1114-1119, 1577-1579, 1610-1612 |
| `wineScreenBrightness` | `wineScreenBrightness` (`str5`, line 75) | float | `-1.0f` (`Float.valueOf(-1.0f)`, line 1317) | MMKV | lines 75, 1316-1321, 1573-1575, 1606-1608 |

Notable: the first two properties do **not** use their property names as keys —
`deviceUniqueId` is persisted as `uniqueId`, and `lastScrollPositionByMyGames`
as `lastMyGamesScrollPosition`. `PreferencesExtensionsKt.checkPreferenceFieldKey`
is called with the *stored* key strings, confirming them.

### `GHLocaleManager` — SharedPreferences file `gh-language`

| Stored key | Type | Default | Source |
|---|---|---|---|
| `language` (`IjkMediaMeta.IJKM_KEY_LANGUAGE`, defined as `"language"` in `IjkMediaMeta.java:91`) | String | `""` | read `GHLocaleManager.java:153`, write `:217` |

The stored value is built by `GHLocaleManager.n(Locale)` as
`language + "$" + country` (e.g. `zh$CN`, `en$US`, `ja$JP`, `ru$RU`) and parsed
back by splitting on a single `$` (`GHLocaleManager.java:169-183, 198-203,
220-240`). Rust stores it verbatim as a string; locale → `Locale` object
conversion is out of scope.

### `SplashActivity` / `SPUtils`

| Stored key | SharedPreferences file | Type | Default | Source |
|---|---|---|---|---|
| `uminit` | `spUtils` (the default name chosen by `SPUtils.f()` → `h("", 0)` → `"spUtils"`, `SPUtils.java:22-33`) | int (`SPUtils.m` = putInt, `SPUtils.i` = getInt) | `0` on read (`i("uminit", 0)`); written `1` on the first-time dialog callback | `SplashActivity.java:208` (write 1), `:285` (read 0) |
| `intro_video_played` | `intro_video_prefs` | boolean | `false` | `SplashActivity.java:209, 212, 335` |
| `gif_loaded` | `intro_video_prefs` | boolean | `false` (read), written `true` | `SplashActivity.java:336, 340` |

### Other MMKV keys in `com/xj/`

MMKV is initialized in `com/xj/app/App.java:44` (`MMKV.x(this)`). The `MMKV`
fields in `GameUseTimeTracker`, `WineInGameSettings`, `WineGameUsageTracker`,
`MobileManagerDataHelper`, `InputControlsManager`, etc. use **dynamic** keys
(mostly game IDs / package names / serialized JSON), not fixed string
literals, so they add no new constant keys. The only fixed MMKV keys in the
reference are `wineScreenBrightnessSwitch` and `wineScreenBrightness` above
(both in `_mmkv_app_global_preferences`). Intent/Bundle extras such as
`video_path`, `video_like`, `video_id`, `video_is_notice`, `is_intro_video`
(`SplashActivity.java:215-219`) are **not** preferences and are excluded.

## Migration helper

No explicit runtime migration routine exists in the decompiled sources. The
only old/new name relationship discovered is the Kotlin property name vs.
persisted key divergence in `AppPreferences`, recorded in `LEGACY_KEY_MAP`:

```
deviceUniqueId                -> uniqueId
lastScrollPositionByMyGames   -> lastMyGamesScrollPosition
```

`migrate(store, old, new)` copies a value to the new key (only when the new key
is absent), then removes the old key; `migrate_all` applies the map and is
idempotent. The helper is generic over `&str`, so callers may supply any
additional mapping.

## Verification

- Compiled standalone with `rustc --edition 2021 --test` via a temporary
  harness wiring `mod.rs` as `pub mod preference;`; **14 tests pass, 0 fail**,
  and no warnings (also clean under `RUSTFLAGS="-D warnings"`).
- Tests cover: exact key strings, all Java initializer defaults, empty-store →
  defaults, full round-trip of every field through `InMemoryKv`, typed
  get/set/remove including the float-as-string case, type-mismatch fallbacks,
  and all migration paths (copy, absent, existing-new-wins, identical keys,
  idempotence).

## Intentionally out of scope

- **MMKV native**: `libmmkv.so`, `MMKV.initialize`, the `mmkv$delegate` lazy
  singleton, `MMKV.C(...)` instance construction, mmap files, protobuf/binary
  encoding, encryption, multi-process mode, and `MMKVExtensionsKt`'s generic
  Kotlin `ReadWriteProperty` machinery (only its *effects* on typed values are
  modeled). Porting these would require the native library or a full binary
  format implementation — a separate, non-portable task.
- **Android `SharedPreferences`**: `Context.getSharedPreferences`, the
  `Editor.apply()`/`commit()` semantics, file-backed XML persistence,
  `OnSharedPreferenceChangeListener`, and `SPUtils`' singleton cache. The
  in-memory store captures only the get/set/remove semantics used here.
- **Kotlin delegates / reflection**: `ReadWriteProperty`, `Lazy`,
  `$$delegatedProperties`, `GsonUtils`-based object serialization, and the
  `checkPreferenceFieldKey` debug validation are not modeled.
- **`GHLocaleManager` Android behavior**: `LanguageUtils`, `Configuration`
  updates, process-name lookup, `Locale` encoding/parsing beyond the raw
  `language$country` string.
- **Room / SQLite persistence** (e.g. `GameSirUxDB`) is handled by a separate
  area (`../data/REPORT.md`), not here.

## Wiring note for the parent session

Add to `app/src/lib.rs`:

```rust
#[path = "../sources/com/xj/common/preference/mod.rs"]
pub mod preference;
```

No other files need changing; the module has no cross-area dependencies.
