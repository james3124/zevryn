# REPORT — Rust port of `PcGameSettingOperations` / `PcGameSettingDataHelper` / gameinfo compatibility

Subagent deliverable. Dependency-free Rust (`std` only), plain `pub` structs/enums,
`#[cfg(test)]` tests, and `mod.rs` re-exports. No changes outside the two
assigned write areas; `Cargo.toml`, `app/src/lib.rs` and other agents' trees
were not touched.

## Reference (read-only)

`/home/Project/Zevryn/reference/gamehub-lite-v518-decompiled/gamehub-lite-v518-decompiled/sources/com/xj/`

## Files written

Write area 1 — `app/sources/com/xj/winemu/settings/operations/`:

| File | Mirrors |
|---|---|
| `component_selection.rs` | `winemu/settings/PcGameSettingOperations.java` + its `$getSelect*OrDefault$1` / `$getDefault*ByDownload$1` classes, plus `winemu/settings/PcGameSettingDataHelper.java` (`getDefaultContainerEntityByDownload`, `getDefaultTranslatorByDownload`), `winemu/bean/PcSettingDataEntity.java`, `winemu/api/bean/ComponentType.java` |
| `translation_config.rs` | `common/data/table/WinEmuTranslationConfigEntity.java`, `common/data/dao/WinEmuTranslationConfigDao.java`, `winemu/settings/translations/data/ConfigModifyState.java` |
| `yml_config.rs` | `PcGameSettingOperations$getYmlList$listType$1.java` (TypeToken of `getYmlList`) |
| `mod.rs` | re-exports |

Write area 2 — `app/sources/com/xj/common/data/gameinfo/`:

| File | Mirrors |
|---|---|
| `compatibility.rs` | `common/data/gameinfo/GameCompatibilityParams.java`, `GameCompatibilityItemParams.java`, `SimpleGameCompatibility.java`; lookup shape from `landscape/launcher/ui/gamedetail/GameDetailSettingMenu.java` and `landscape/launcher/ui/compatible/CompatibleDialog.java` |
| `mod.rs` | re-exports |

## Exact types, fields and sources

### `component_selection.rs`
- `ComponentKind { Container, Gpu, Dxvk, Vkd3d, Translator }` derived from the five
  `getDefault*ByDownload` methods: `PcGameSettingDataHelper.k`
  (`getDefaultContainerEntityByDownload`), `PcGameSettingOperations.T/R/W`
  (gpu/dxvk/vkd3d), `PcGameSettingDataHelper.o` (`getDefaultTranslatorByDownload`).
  `component_type_name()` maps to `ComponentType.GENERAL/GPU/DXVK/VKD3D/TRANSLATOR`
  (`winemu/api/bean/ComponentType.java`: 5/GENERAL, 2/GPU, 3/DXVK, 4/VKD3D, 1/TRANSLATOR).
- `ComponentSelection` fields from `PcSettingDataEntity.java`: `id` (`getId`),
  `name` (`getName`), `version` (`getVersion`), `file_name` (`getFileName`),
  `file_md5` (`getFileMd5`). Added `kind`, `downloaded` (result of
  `WinEmuFileExplorer.q(..)` / `.v(..)`), `installed` (selection-dialog state).
- `SelectionResolver` / `default_by_download` / `select_or_default`: mirrors
  `getSelectXOrDefault`: read current selection (`Z()`/`H0()`/`f1()`/`x0()`),
  and only if null read default entity (`PcGameSettingDataHelper.l/m/q/j/p`,
  keys `pc_d_dxvk`, `pc_d_gpu`, `pc_d_vkd3d`, `pc_d_translator`), returning it
  only when the download check passes. `getDefaultXByDownload` = default entity
  filtered by the download check.
- `official_gpu_driver_selection()` — the `id = -1` sentinel returned by
  `getSelectGpuOrDefault` / `getDefaultGpuByDownload`.

### `translation_config.rs`
- `TranslationConfig` fields from `WinEmuTranslationConfigEntity.java`:
  `uid`, `id` (`@PrimaryKey`), `name`, `config_id` (`configId`),
  `create_time` (`createTime`), `last_update_time` (`lastUpdateTime`),
  `game_id` (`gameId`), `config`, `copy_from_id` (`copyFromId`),
  `copy_from_name` (`copyFromName`); const `ID_PREFIX = "local_"` and
  `create_local_id(sourceId)` from `Companion.createLocalId`.
- `ConfigModifyState` variants exactly as `ConfigModifyState.java`:
  `CreateSuccess, CreateFailure, DeleteSuccess, DeleteFailure, ApplySuccess,
  ApplyFailure, UpdateSuccess, UpdateFailure`.
- `ConfigChange` mirrors the `Pair<ConfigModifyState, config>` events of
  `TranslationConfigViewModel` / `UpdateTranslationConfigEvent`.
- DAO (`WinEmuTranslationConfigDao`) read only for field context; no DAO code ported.

### `yml_config.rs`
- `YmlConfig { name, list_type: YmlListType::StringList }` from `getYmlList`:
  `PcGameSettingOperations$getYmlList$listType$1 extends TypeToken<List<String>>`.

### `compatibility.rs`
- `CompatibilityPreset` fields from `GameCompatibilityParams.java`
  (`@SerializedName`): `title`, `icon`, `desc`, `level`, `data: Vec<CompatibilityItem>`.
- `CompatibilityItem` fields from `GameCompatibilityItemParams.java`:
  `title`, `icon`, `desc`, `level`, `is_check`.
- `SimpleGameCompatibility` fields from `SimpleGameCompatibility.java`:
  `game_id` (`game_id`), `steam_app_id` (`steam_appid`), `title`, `icon`, `desc`,
  `level`; `is_unknown_level()` = `level == 0` (`isUnknownLevel()`).
- `CompatibilityCatalog { found, not_found }` from `GameDetailSettingMenu.F`
  (`cst_data` / `notFountCts` passed to `CompatibleDialog`); `lookup` picks
  `not_found` when `isUnknownLevel()`, else `found`. Also `find_preset_by_level`.

## Out of scope (not ported)

- Network download / file I/O: `WinEmuFileExplorer.q/v`, `aria2`, dependency
  install and `checkAndDownloadComponent`.
- Room persistence: `WinEmuTranslationConfigDao` SQL/`@Query` methods and the
  `WinEmuTranslationConfigDao_Impl` implementation; all `Flow`/`LiveData`/`Pair`
  event plumbing.
- Kotlin/Suspension machinery: `Continuation` state machines, coroutines,
  `TypeToken`/Gson/YAML parsing, Android framework/resource-string lookups and
  UI dialogs/recycler adapters.

## Verification

Compiled each module in isolation with `rustc --test --edition 2021` (the new
modules are not yet wired into `app/src/lib.rs`, which is owned by another
agent). Results: component_selection 6/6, translation_config 4/4, yml_config
1/1, compatibility 3/3. All tests pass; `rustc --crate-type lib mod.rs` for the
operations module exits 0 with no warnings.
