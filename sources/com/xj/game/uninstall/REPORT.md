# Uninstall game / remove-from-library flow — Rust port report

Write area: `app/sources/com/xj/game/uninstall/`
Reference (read-only): `reference/gamehub-lite-v518-decompiled/gamehub-lite-v518-decompiled/`
Standard library only (`std`); no external crates. 13 unit tests pass in isolation
(verified with a temporary standalone harness that `#[path]`-includes `mod.rs`;
the real `Cargo.toml` / `app/src/lib.rs` were not modified).

## Files

| File | Contents |
|---|---|
| `mod.rs` | Module docs, `pub mod` declarations, re-exports, dialog-title constants. |
| `game_from.rs` | `GameFrom` enum + Java-name/ordinal/WhenMappings helpers, tests. |
| `uninstall_step.rs` | `UninstallStep` enum with per-variant Java source, `as_str`, `source`, `Display`, tests. |
| `uninstall_plan.rs` | `Game`, `UninstallFlags`, `UninstallPlan` builder, tests across all `GameFrom` values. |
| `uninstall_confirm.rs` | `UninstallConfirm` callback model, `UninstallConfirmListener` trait + noop/closure impls, tests. |
| `executor.rs` | `RemoveLibrary` trait, `UninstallResult`, `Execution`, `execute` / `execute_confirmed`, tests. |
| `REPORT.md` | This report. |

## Exact `GameFrom` values and source

`sources/com/xj/game/UninstallGameHelper.java`, lines 61-65:

```java
public enum GameFrom {
    DOWNLOAD_FROM_SVR,
    IMPORT_BY_USER,
    IMPORT_BY_STEAM_TOOL,
    DOWNLOAD_FROM_STEAM_SVR;
}
```

| Rust variant | Java constant | Ordinal | `WhenMappings.f26492a` (lines 84-101) |
|---|---|---|---|
| `DownloadFromSvr` | `DOWNLOAD_FROM_SVR` | 0 | 1 |
| `ImportByUser` | `IMPORT_BY_USER` | 1 | 2 |
| `ImportBySteamTool` | `IMPORT_BY_STEAM_TOOL` | 2 | 3 |
| `DownloadFromSteamSvr` | `DOWNLOAD_FROM_STEAM_SVR` | 3 | 4 |

Classification happens in `o()`/`getGameFrom()` (lines 311-379) using
`g()`/`checkIsSteamGameDownloadBySteamService` (Steam `Downloaded` /
`InDownloadQueue`), `GameLibraryRepository.a(localGameId)` plus presence of
`steam_game_info`, and `GameStateMgr.f/d/e(id)`, or is `null` when unrecognized.
`q()`/`isNeed2UninstallGame` (lines 386-449) is false only for `IMPORT_BY_USER`,
which is why that source never deletes game files.

## `UninstallStep` rules and sources

Plan order mirrors `UninstallGameConfirmHelper$toUninstallPcGame$1`:
confirm -> source-specific game removal -> cover -> library removal -> home reset.

| Step (`UninstallStep`) | Emitted when | Decompiled source |
|---|---|---|
| `ConfirmDialog` | Always, first. | `UninstallGameConfirmHelper.h()`/`i()` (lines 368-377) -> `CommUninstallConfirmDialog` (`showRemoveGameConfirmDialog$1`, `showRemovePcGameConfirmDialog$1`). Title key `winemu_uninstall_game_title`. |
| `DeleteOnlineDownload` | `GameFrom::DownloadFromSvr`. | `UninstallGameHelper.r()` case 1 -> `v()` -> `h()` / `deleteDownloadOnlineGame` (`IWinEmuService.u`, lines 156-192). |
| `UninstallPcDemo` | `DownloadFromSvr` + `is_pc_demo`. | `v()` -> `uninstallPcDemoGame` (`IWinEmuService.j`, `$uninstallPcDemoGame$2`). |
| `DeleteSteamGame` | `ImportBySteamTool`; or `DownloadFromSteamSvr` + `is_steam_downloaded`. | `r()` case 3 -> `l()` (`IWinEmuService.h`, lines 260-264); `r()` case 4 -> `m()` / `deleteSteamGameByDownload` (`ISteamGameService.s`, lines 267-303). |
| `DeleteLocalCover` | `has_local_cover`. | `resetSingleGame()`/`toUninstallPcGame$1` -> `uninstallGameHelper.j(cover_image)` -> `deleteIfGameCoverInLocal` (`j()`/`k()`, lines 200-258); also `r()` case 2 deletes `localGameIconPath` (lines 471-481). |
| `ResetGameData` | `DownloadFromSvr`; and again after library removal when `is_in_library` (emitted only once per plan). | `uninstallPcDemoGame` -> `GameStateMgr.j(gameId)`; `resetSingleGame()`/`toUninstallPcGame$1` -> `IHomeService.m(validId)`. |
| `RemoveFromLibrary` | `is_in_library`. | `UninstallGameConfirmHelper.f()` / `removeFromGameLibrary` (`GameLibraryTableHelper.l`, lines 65-263). |

Flags map to runtime checks: `is_steam_downloaded` = `checkIsSteamGameDownloadBySteamService`;
`is_in_library` = a `GameLibraryTable` row exists; `has_local_cover` = local
`cover_image`/`localGameIconPath` file; `is_pc_demo` = PC-demo game.

## Confirm result / executor

* `RemoveOrUninstallGameListener.a(boolean z3, boolean z4, String str)`
  (`RemoveOrUninstallGameListener.java:17`) -> `UninstallConfirm { success, removed,
  message }`. `""` on the normal path, `"remove failure"` on the exception/
  empty-result path, `"cancelled"` for a declined dialog. `UninstallConfirmListener`
  mirrors the interface; closures implement it automatically.
* `RemoveLibrary::remove(id) -> bool` is the only I/O-shaped hook; `false`
  corresponds to `removeFromGameLibrary`'s empty affected list. It can be
  implemented by the built-in game-library storage.
* `execute(plan, store, confirmed)` walks the plan, records `completed_steps`,
  and returns `UninstallResult` (converted with `to_confirm()` into the callback
  model). Non-library steps are pure state transitions; declining confirmation
  stops before any destructive step.

## Out of scope

* Real file/directory deletion (`FileUtils.m`, `deleteIfGameCoverInLocal`
  filesystem work) and any Android storage access.
* Android UI: dialogs (`CommUninstallConfirmDialog`, `LoadingProgressDialog`),
  `Activity`/`FragmentActivity` lifecycle, focus/toast/event bus
  (`RemoveGameLibraryEvent`, `ChannelKt`), coroutine dispatchers.
* Native / service calls: Steam (`ISteamGameService`, `SteamGameManager`),
  WineMu emulator (`IWinEmuService`, `IWinEmuService.j/u/h`), `TheRouter`
  lookups, `GameStateMgr` persistence, `IHomeService` refresh, Room/DAO writes.
* Wiring into `app/src/lib.rs` and editing `Cargo.toml` (owned by other agents).
  To integrate, add `#[path = "../sources/com/xj/game/uninstall/mod.rs"] pub mod
  uninstall;` to the crate root.
