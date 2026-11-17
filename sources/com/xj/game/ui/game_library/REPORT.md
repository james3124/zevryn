# REPORT — Portable Rust model of the Game Library UI

Rust reimplementation of the **portable UI state** for GameHub Lite v5.1.8.
Only `std` is used. All code lives in my write area:

```
app/sources/com/xj/game/ui/game_library/
```

## Files written

| File | Contents |
|------|----------|
| `mod.rs` | Module declarations + flat `pub use` re-exports. |
| `game_entry.rs` | `GameEntry`, `GameType`, `SortOrder`, `LibraryFilter`, pure `sort()` / `filter()`. |
| `ui_state.rs` | `GameLibraryUiState`, `ViewMode` + state mutators / `visible_games()`. |
| `import_dialog.rs` | `ImportChoice`, `ImportDialogModel` (selection state). |
| `import_event.rs` | `ImportRequest`, `UiAction`, `process_import_request()` / dialog→action helpers. |
| `REPORT.md` | This file. |

## Reference sources studied (read-only)

- `sources/com/xj/game/ui/game_library/fragment/GamesFragment.java`
  — `page_type` tab index (field `f26549l`, default `1` = PC games), `GamesAdapter`,
  grid `FocusGridLayoutManager(context, 3, ...)`, imported-game / remove-game events.
- `sources/com/xj/game/ui/game_library/GameLibraryActivity.java`
  — `textTabLayout` tabs from `GameLibraryTab` (`z1()` → `game_pc_games_tab`),
  `gl_page_main_index` intent extra, page switching.
- `sources/com/xj/landscape/launcher/ui/main/me/MyFragment.java` (+ `MyGamesView.java`,
  `MyGamesItemData.java`, `MyGamesItemViewHolder.java`)
  — dashboard / "My Games" listing state; no extra persistent sort/filter fields
  were found there beyond the list data itself, so the modeled state is driven
  by the game-library screens.
- `sources/com/xj/landscape/launcher/event/Navigate2ImportPcGameFileEvent.java`
  — empty marker event consumed by `LandscapeLauncherMainActivity.AnonymousClass8`,
  which checks storage permission and launches a file picker for `exe` (and `txt`
  when `AppConfig.j()` / logitech flag is set).
- `sources/com/xj/landscape/launcher/ui/dialog/SelectImportGameTypeDialog.java`
  — two clickable rows (`llImportPcGame`, `llImportMobileGame`), close icon
  (`ivClose`), focus init coroutine (just `requestFocus()` after a 50 ms delay).
- `sources/com/xj/common/router/PageRouterUtils.java` — `j(Activity)` routes the
  mobile choice to `/launcher/ui/mobilegame/MobileGameV4Activity`.
- `sources/com/xj/common/data/model/GameType.java` — `MOBILE_INSTALL_APP(1)`,
  `PC_STEAM_IMPORT(2)`.

## Exact import-type choices (from decompiled strings)

The dialog offers **two** choices (plus a close button). Labels come from
`resources/res/values/strings.xml`; layout ids from
`resources/res/layout/llauncher_dialog_select_import_game_type.xml`:
dialog title `@string/import_game` = **"Import Game"** (strings.xml line 879).

| Choice (model) | Layout id | Label string resource | English label | Behavior |
|---|---|---|---|---|
| `ImportChoice::PcGame` | `llImportPcGame` | `game_type_pc_game` (strings.xml line 800) | **"PC Game"** | posts `Navigate2ImportPcGameFileEvent` → file picker for `exe` (`txt` also when logitech flag set) → modeled as `UiAction::OpenFilePicker` |
| `ImportChoice::MobileGame` | `llImportMobileGame` | `game_type_mobile_game_title` (strings.xml line 799) | **"Mobile Game"** | `PageRouterUtils.j(activity)` → `MobileGameV4Activity` (APK import) → modeled as `UiAction::OpenFilePicker` |

Close: `ivClose` → dismiss only → modeled as `UiAction::Cancel`.

There is **no "from Steam" choice in `SelectImportGameTypeDialog`**. The
`OpenSteamScan` action is part of the required portable action set and is
modeled as reachable through the separate `ImportRequest::ScanSteamLibrary`
path (Steam import exists elsewhere in the app, e.g. the `winemu_import_steam_*`
resources). Likewise the task's "from APK / from Steam / from exe" example is
not literal in the decompiled dialog; only the two choices above are real.

## Modeling notes / decisions

- `GameLibraryUiState.tab_index` is a `usize` because the decompiled code uses a
  numeric `page_type` / `gl_page_main_index`, not a named tab enum. Default is `1`.
- `ViewMode::{Grid, List}` — the decompiled fragment always uses a 3-column
  grid; `List` is a modeling extension so the portable UI can switch modes.
- `GameType` codes mirror the decompiled enum (`Mobile=1`, `Pc=2`); `Imported=3`
  is a modeling extension giving the `LibraryFilter::Imported` filter a category
  (locally added game files).
- `sort()` semantics: `Name` = A→Z case-insensitive; `Time` = newest first
  (descending timestamp); `Type` = grouped by `GameType` code then alphabetical.
  Ties break deterministically (title, time, id) so results are stable. Both
  `sort()` and `filter()` are pure (return a fresh `Vec`, never mutate input).
- Search query is matched case-insensitively as a substring of the title and is
  ANDed with the category filter in `filter()` / `visible_games()`.
- Selection is cleared when the tab, filter, or search query changes (rows may
  disappear), matching the reactive fragment behavior.
- `Navigate2ImportPcGameFileEvent` carries no data (empty marker class), so the
  modeled `ImportRequest` variants are purely categorical and the translation
  to `UiAction` is a total match with a `Cancel` default.

## Out of scope (intentionally not modeled)

- Android `Fragment` / `Activity` / `Adapter` / `ViewHolder` / `View` /
  `FocusGridLayoutManager` / coroutine / `Channel` bus plumbing.
- Real file picker, permission (`ManageStoragePermissionUtil`) checks, and
  `ActivityResultLauncher` for `exe`/`txt`/APK.
- Router / `TheRouter` / `Navigator` navigation and all `PageRouterUtils` side effects.
- Steam account login, import progress/extraction, download and game-launch logic.
- UI rendering, focus borders, animation, localization of labels (English labels
  are only documented as reference constants).

## Verification

Standalone harness (only these files, `std` only), compiled with
`rustc --edition 2021 --test`, report **18 tests passed, 0 failed**:
sort (name/time/type + purity), filter (category/query/combined),
dialog selection + dismiss, and event→action (PC/mobile/Steam/cancel) paths.
The `#[path]`-based crate wiring (mirroring `app/src/lib.rs`) was also verified.
