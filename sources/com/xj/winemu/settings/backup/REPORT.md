# REPORT — settings backup / env models (Rust port of GameHub Lite v5.1.8)

## Scope

Dependency-free Rust models for the portable logic of the GameHub Lite v5.1.8
settings package. Only `std` is used. No real I/O, dialogs, or native calls.
This report covers both assigned write areas (`settings/backup/` and
`settings/env/`) so the full inventory is in one place.

## Files written

### `settings/backup/`
- `mod.rs` — package root; `pub mod` declarations + `pub use` re-exports.
- `plan.rs` — `BackupKind`, `BackupEntry`, `BackupPlan`, `BackupReport`.
  - `BackupPlan::with_defaults()` gives a stable ordered entry list
    (settings → translation_config → env_params → exe_path → container).
  - `backup(&BTreeSet<String>)` / `recover(&BTreeSet<String>)` are pure
    path-set transformations producing an ordered `BackupReport`
    (`requested`, `found`, `missing`, `is_success()`).
- `reset.rs` — `ResetTarget`, `ResetScope` (`SettingsOnly`,
  `SettingsAndContainer`, `Full`), and `ResetPlan` = scope-derived ordered
  target list with `with_target` / `without_target` builder methods.
- `container.rs` — `VirtualContainerCheck` with `found(path)` /
  `missing(message)` constructors modeling the `Pair<File?, String>` result.
- `REPORT.md` — this file.

### `settings/env/`
- `mod.rs` — package root; `pub mod` declarations + `pub use` re-exports.
- `params.rs` — `EditType`, `EnvParam`, `EnvParams`, `EnvParseError`,
  `validate_env_string`.
  - `to_env_string()`: skip fully-blank rows, de-dup by key (first wins),
    join `key=value` with `,`.
  - `parse()` / `parse_with_edit_type()`: faithful port of
    `PcGameSettingDataHelper.R` (split on `,`, split on `=` with limit 2,
    trim, concatenate comma-continuation values, preserve order).
- `exe_path.rs` — `LocalExePath`, `ExePathError`, `ExePathUpdate`, and
  `update()` for local PC exe-path validation (absolute path + `.exe`).
- `REPORT.md` — copy of this report for the env write area.

## Sources studied (reference, read-only)

Path prefix:
`reference/gamehub-lite-v518-decompiled/gamehub-lite-v518-decompiled/sources/com/xj/winemu/`

- `settings/PcGameDataHandler.java`
  - `backupData2Local` (method `a`): `SAFUtils` uri `{name}_{timestamp}.zip`,
    `WinAPI.N(absPath, out)` ZIPs the container; returns Pair(success, message).
  - `recoverDataFromLocal` (method `c`): `WinAPI.L(in, absPath)` unzips into
    the container; returns Pair(success, message).
  - `checkGameVirtualContainerExist` (method `b`): resolves `IEmuContainer`
    via `IWinEmuService.l(gameId)`; null → `(null, install_dep_tips)`,
    else `(File(container.b()), "")`.
  - `resetGameData` (method `d`): delegates to `IWinEmuService.c(gameId)`.
- `settings/PcGameDataHandler$backupData2Local$isSuccess$1.java` — confirms the
  container directory is the single ZIP payload (`WinAPI.N`).
- `settings/PcGameDataHandler$recoverDataFromLocal$isSuccess$1.java` — confirms
  restore target (`WinAPI.L`).
- `settings/PcGameDataHandler$resetGameData$2.java` — returns the service
  boolean.
- `settings/PcGameSettingOperations.java` / `...$resetGameData$1.java` —
  `B1()` clears game settings then translation config for the game.
- `settings/PcGameSettingDataHelper.java` / `...$resetAllData$1.java` —
  `J()` clears all `SPUtils` stores plus the translation-config repository;
  `R(String)` (lines 523–554) is the env-string parser; `V1(String)` writes it.
- `settings/PcGameSettingsActivity$onDataBackUpOrRecover$1$1.java` — confirms
  the backup (document tree) / recover (open document) flow around the handler.
- `settings/InputEnvParamsDialog.java` — `EditType`-driven input; rejects empty
  input and invalid env strings with `pc_cc_environment_variable_input_tips`.
- `settings/NewInputEnvParamsDialog.java` — `p(List)` builds the comma-joined
  `key=value` string (filters blank rows, de-dups by key via `HashSet`);
  `q(entity)` = `key + "=" + value`; populates rows from `R(initValue)`.
- `settings/EditType.java` — `EditGameVars`, `EditTranslationConfigName`.
- `bean/PcSettingEnvEntity.java` — `key` / `value` data class.
- `settings/GameSettingViewModel.java` and its
  `$getLocalPcExePath$1` / `$updateLocalPcExePath$1` inner classes — get/update
  flows and the repository-boolean success gate.

## Out of scope (intentionally not modeled)

- Real filesystem access, SAF `Uri` streams, ZIP compression/decompression
  (`WinAPI.N` / `WinAPI.L` in native `libwinemu.so`).
- Android UI: dialogs, focus engine, RecyclerView adapters, bindings, toasts.
- Coroutines / `Continuation` state machines, `TheRouter`, `IWinEmuService`
  IPC, `GameLibraryRepository` network/storage calls.
- Resource-ID → localized-string resolution (messages stored as plain strings).
- LiveData / ViewModel lifecycle and event delivery.
- `mod.rs` files of other agents' directories and `app/src/lib.rs` /
  `Cargo.toml` (not touched).

## Tests

`#[cfg(test)]` unit tests are colocated in each module and cover:
backup plan ordering and pure path-set transforms; all three reset scopes and
builder edits; env-param parse/serialize round-trips (including commas, `=`,
de-dup, edit types); exe-path validation and update outcomes; container-check
constructors. Run with `cargo test` once the crate's module tree is wired up.
