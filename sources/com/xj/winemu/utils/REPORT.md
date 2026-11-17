# REPORT — Rust model of the portable PC-game import pipeline

All files below were written only in the assigned write area
`app/sources/com/xj/winemu/utils/`. The project `Cargo.toml`,
`app/src/lib.rs` and other agents' directories were not touched.

## Files written

| File | Contents |
| --- | --- |
| `mod.rs` | Module documentation and re-exports of all public types/functions. |
| `local_pc_game_manifest.rs` | `LocalPcGameManifest` struct with the 12 real Gson fields, getters/setters, `empty()`/`Default`, `handle_executable()`, `is_portable_v1()`, `zip_marker_file()`. |
| `file_detector.rs` | `FileKind::{Manifest, Exe, Ignored}`, `classify()`, `basename()`, `is_manifest_file()`, `is_exe_file()`, `manifest_steam_app_id()` and the recovered string constants. |
| `import_plan.rs` | `ImportPlan`, `ImportStep`, `ImportStepKind`; classify + dedupe + ordered steps from a list of path strings. |
| `acf_parser.rs` | `AcfDocument`, `VdfValue`, `AcfParseError`: dependency-free Valve VDF/ACF key-value parser with `get(["AppState","appid"])`-style lookup and `to_manifest()` projection. |
| `exe_info.rs` | `ExeInfo::from_path()` — file name and game name (stem without `.exe`). |
| `REPORT.md` | This document. |

Every source file uses only `std`, exposes plain `pub` structs/enums, and
carries `#[cfg(test)]` unit tests (20 tests total, all passing).

## Reference sources studied (read-only)

Root: `reference/gamehub-lite-v518-decompiled/gamehub-lite-v518-decompiled/`

- `sources/com/xj/winemu/bean/LocalPcGameManifest.java`
- `sources/com/xj/winemu/utils/ImportPcGameHelper.java`
- `sources/com/xj/winemu/utils/ImportPcGameHelper$readAppManifestFilesAndImportGame$1.java`
- `sources/com/xj/winemu/utils/ImportPcGameHelper$checkPermissionAndStartRead$2.java`
- `sources/com/xj/winemu/utils/ImportPcGameHelper$removeManifestFilesBySteamId$2.java`
- `sources/com/xj/winemu/vm/ImportGameViewModel.java`
- `sources/com/xj/winemu/vm/ImportGameViewModel$fetchGameInfoByExeFileName$1.java`

## Exact `LocalPcGameManifest` fields and getters

Field declarations and `@SerializedName` values are at
`LocalPcGameManifest.java:26-69`; getters at lines 156-216; setters at
224-278; `componentN()` at 76-130; primary constructor at 285-306; no-arg
constructor at 71-73.

| Java field | `@SerializedName` | Lines | Getter (lines) | Rust field / method |
| --- | --- | --- | --- | --- |
| `appId` (`long`) | `AppId` | 26-27 | `getAppId()` 156-158 | `app_id` / `get_app_id()` |
| `arguments` | `Arguments` | 29-31 | `getArguments()` 160-163 | `arguments` / `get_arguments()` |
| `executable` | `Executable` | 33-35 | `getExecutable()` 165-168 | `executable` / `get_executable()` |
| `installDir` | `InstallDir` | 37-39 | `getInstallDir()` 175-178 | `install_dir` / `get_install_dir()` |
| `manifestVersion` | `ManifestVersion` | 41-43 | `getManifestVersion()` 180-183 | `manifest_version` / `get_manifest_version()` |
| `name` | `Name` | 45-47 | `getName()` 185-188 | `name` / `get_name()` |
| `packaged` (`boolean`) | `Packaged` | 49-50 | `getPackaged()` 190-192 | `packaged` / `get_packaged()` |
| `packagedFiles` (`List<String>`) | `PackagedFiles` | 52-54 | `getPackagedFiles()` 194-197 | `packaged_files` / `get_packaged_files()` |
| `sizeOnDisk` (`long`) | `SizeOnDisk` | 56-57 | `getSizeOnDisk()` 199-202 | `size_on_disk` / `get_size_on_disk()` |
| `source` | `Source` | 59-61 | `getSource()` 203-206 | `source` / `get_source()` |
| `userId` | `UserId` | 63-65 | `getUserId()` 208-211 | `user_id` / `get_user_id()` |
| `userName` | `UserName` | 67-69 | `getUserName()` 213-216 | `user_name` / `get_user_name()` |

Additional getter: `getHandleExecutable()` (lines 170-173) does
`replace('\\', '/', ignoreCase = false)` — modeled as
`LocalPcGameManifest::get_handle_executable()`.

Behaviour recovered from `ImportPcGameHelper`:

- `n(LocalPcGameManifest)` (lines 232-238): version `"1.0.0"` selects the
  Downloads `/SteamGame` root; modeled as `is_portable_v1()`.
- `s(LocalPcGameManifest)` (lines 256-280): picks the first `packagedFiles`
  entry ending in `.zip` (case-sensitive) for the `.tmp` marker; modeled as
  `zip_marker_file()`.
- `m(String appId)` / `v(String appId)` (lines 217-230 / 288-299): manifest
  file name is `"appmanifest_" + appId + ".json"`.

`component1..component12` (lines 76-130) map, in order, to
`manifestVersion, appId, name, executable, arguments, installDir, sizeOnDisk,
source, userId, userName, packaged, packagedFiles` — this is the field order
used by the Rust `new(...)` constructor.

## Manifest-file / exe detector (exact recovered logic)

`ImportPcGameHelper$readAppManifestFilesAndImportGame$1.java:298-311` and
`ImportPcGameHelper$checkPermissionAndStartRead$2.java:137-150`:

```java
if (!file.isFile()) return false;
String name = file.getName();
if (!StringsKt__StringsJVMKt.X(name, "appmanifest_", false, 2, null)) return false;  // startsWith
return StringsKt__StringsJVMKt.H(file.getName(), ".json", false, 2, null);          // endsWith
```

`false` is the `ignoreCase` argument, so both checks are case-sensitive. The
model implements this as `is_manifest_file()` (plus a `.acf` variant, see the
note below) and applies `basename()` so a full path can also be passed.

The task requested `appmanifest_*.acf` detection; the decompiled GameHub Lite
code actually recognizes `appmanifest_*.json` (its own Gson manifests). The
detector intentionally accepts **both** extensions, with the discrepancy
documented here and in `file_detector.rs`. No `.exe` predicate was recoverable
(the `invokeSuspend` bodies are JADX dump stubs), so `is_exe_file()` uses a
case-insensitive `.exe` suffix as a pragmatic extension.

## Import plan and ViewModel mapping

- `ImportPcGameHelper.u(...)` (`ImportPcGameHelper.java:282-286`) launches
  `readAppManifestFilesAndImportGame`; the callback reports success
  (`"全部导入完成 "`, line 243) or failures such as `"无游戏可导入"` (line 198).
- `ImportGameViewModel.m(exePath)` (`ImportGameViewModel.java:49-57`) calls
  `fetchGameInfoByExeFileName`, which delegates to
  `GameLibraryRepository.k(exePath)`
  (`ImportGameViewModel$fetchGameInfoByExeFileName$1.java:60`) and treats a
  result with `gameId <= 0` as no match (lines 73-77).

`ImportPlan::from_paths()` models this as:
`ParseManifest*` -> `ReadExeInfo*` -> `FetchGameInfo*` -> `Complete`.
Paths are deduplicated by exact string (first occurrence kept); `Ignored`
paths are collected separately. `ExeInfo.game_name` is the exe file name
without its final `.exe`, i.e. the lookup name passed to the repository.

## Out of scope (deliberately not modeled)

- Real file I/O: directory listing, `File.isFile()`, reading/writing/deleting
  manifest files, ZIP extraction, progress monitors (`net.lingala.zip4j`).
- Android runtime: `FragmentActivity`, `ManageStoragePermissionUtil`,
  `PermissionUtils`, `NotificationManager` / channel `"steam-899"`,
  `Environment` paths, `Utils.a()` context access, resources/strings.
- Kotlin coroutines and scheduling (`GlobalScope`, `Dispatchers`,
  `SuspendLambda`, `CoroutineExceptionHandler`) and the `Function3` callback.
- Gson JSON serialization/deserialization of `LocalPcGameManifest`, and the
  `GameLibraryRepository` / `ImportedGameEvent` / database and network layer.
- Human-readable size formatting `h(long)` (`ImportPcGameHelper.java:139-173`).
- Full VDF feature set (comments, `@include`, conditionals, expansions) and
  duplicate-key merge rules; the ACF parser keeps the first matching key and
  handles only quoted strings, braces and the common escapes.
- Native code (`.so`, JNI) and any non-std Rust dependency.

## Verification

The module was compiled and tested with `rustc`/`cargo 1.100` (edition 2021)
through a temporary harness that included `utils/mod.rs` by path; all **20
unit tests passed**. The harness lives outside the project and the project's
`app/src/lib.rs` was not modified — wiring the new `mod utils;` declaration
into the crate is left to the integrating agent.
