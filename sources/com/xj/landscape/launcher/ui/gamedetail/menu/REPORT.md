# Report — game-card long-press menu (Rust port)

## Scope

Port of the portable menu logic found in GameHub Lite v5.1.8's
`com.xj.landscape.launcher.ui.gamedetail.GameDetailSettingMenu` and the
long-press entry point
`com.xj.landscape.launcher.ui.main.me.MyGamesOperationPopupKt`. Only `std` is
used; no Android types, no external crates.

## Files written (all inside this directory)

| File | Contents |
|---|---|
| `mod.rs` | Module declarations and re-exports of the public API. |
| `option.rs` | `MenuOptionId` (exact option identities + labels + R.string names + decompiled `Option.id` values), `MenuOption { id, label, enabled }`. |
| `assembly.rs` | `MenuContext` flags, `build_menu()`, `menu_option_ids()`, documented assembly rules. |
| `action.rs` | `MenuAction`, `resolve(MenuOptionId) -> MenuAction`. |
| `state.rs` | `MenuState` (open/closed + selected index) with `open()`, `select()`, `dismiss()`. |
| `REPORT.md` | This file. |

All four code files contain `#[cfg(test)]` tests: **18 tests, all passing**
(verified with `rustc --test --edition 2021` via a temporary harness outside
the project tree).

## Exact option labels and sources

English values are from `resources/res/values/strings.xml` (the default
`values/` configuration). The two update rows are hard-coded Chinese literals
in the Java, so they have no `R.string` entry.

| `MenuOptionId` | Label | `R.string` | Java source line | strings.xml line |
|---|---|---|---|---|
| `AddToLibrary` | `Add Game Library` | `llauncher_add_game_to_library` | `GameDetailSettingMenu.java:363` (`A`) | 1027 |
| `RemoveFromLibrary` | `Remove Library` | `llauncher_remove_game_from_library` | `GameDetailSettingMenu.java:363` (`A`), `:718` (`a0`) | 1400 |
| `CloudGameSetting` | `Cloud Game Settings` | `llauncher_cloud_game_setting` | `GameDetailSettingMenu.java:379` (`C`) | 1045 |
| `CheckCompatibility` | `Check Compatibility` | `llauncher_check_compatibility` | `GameDetailSettingMenu.java:394` (`E`) | 1043 |
| `PcGameSetting` | `PC Game Settings` | `llauncher_pcgame_setting` | `GameDetailSettingMenu.java:609, :660` (`Y`) | 1348 |
| `UninstallGame` | `Uninstall game` | `winemu_uninstall_game_title` | `GameDetailSettingMenu.java:727` (`a0`) | 3391 |
| `IgnoreUpdate` | `忽略此版本更新` | — (literal) | `GameDetailSettingMenu.java:523, :550` (`V`) | — |
| `CancelIgnoreUpdate` | `取消忽略此版本更新` | — (literal) | `GameDetailSettingMenu.java:497` (`V`) | — |
| `GameSettings` | `Game Settings` | `llauncher_game_setting` | string defined at `strings.xml:1138` (generic settings label) | 1138 |

`Option` field mapping (`sources/com/xj/common/view/popup/Option.java`):
`text` = field `a`/`d()` (lines 23, 58–60); `id` = field `d`/`a()` (lines 31,
46–48); `enabled` is represented by the `isChecked`/`f24949b` boolean (lines
25–26, 62–64) with the synthetic constructor mask semantics at lines 87–89.

Decompiled `Option.id` values (set via the constructor mask at the call sites;
see `apktool/smali_classes5/.../GameDetailSettingMenu.smali`):
`-1` = generic rows (mask `0xe`, e.g. cloud setting at smali 1450–1458,
compatibility at 1514–1522, PC setting / update rows); `1` = uninstall (mask
`0x6`, smali `a0`); `2` = add/remove library (`A`, smali 1393–1405); `3` =
remove-library variant from `a0`. These are exposed by
`MenuOptionId::option_id()` and drive de-duplication and the final stable sort.

## Assembly rules

Decompiled `getMenuOptions` (method `G`) is present only in smali
(`GameDetailSettingMenu.smali:1531–2288`) because JADX skipped the Java body
(`GameDetailSettingMenu.java:448–454`). It iterates the game's startup params
and dispatches per `start_type`:

* `f0` (mobile) at smali 1810 → `U()` (library option only).
* `e0` (`1406`, cloud) at smali 1831 → `C()` (library option when in library + cloud setting).
* `g0` (`1403/1407/1501`, PC game) at smali 1854, gated by `!AppConfig.l()` → `Y()` (uninstall/remove + compatibility + PC settings).
* `j0` (`1202`/`1`) at smali 1949 → `d0()` (library option when in library).
* Otherwise → `A()` at smali 1970.

After the loop it removes duplicate library/uninstall rows by `Option.id`
(smali 1980–2219), optionally appends `add_to_homescreen` when in library
(smali 2236–2274), and **stably sorts by `Option.id` ascending**
(smali 2279–2287; comparator
`GameDetailSettingMenu$getMenuOptions$$inlined$sortedBy$1.smali:47–75`).
`build_menu` reproduces the same de-duplication and sort with the flags in
`MenuContext`; the full ordering for all flags true is:
`CloudGameSetting, CheckCompatibility, PcGameSetting, GameSettings,
IgnoreUpdate, UninstallGame, RemoveFromLibrary`.

The long-press entry point `MyGamesOperationPopupKt.i` calls
`gameDetailSettingMenu.m0(..., isInGameLib = true, ...)` at line 75, so the
real menu always passes `is_in_library = true`; `false` is still modelled so
that the `AddToLibrary` branch is representable. Note the decompiled `A()`
early-returns `null` when `isInGameLib == false` (Java 358–360), making its
`llauncher_add_game_to_library` branch unreachable in this build; the port
restores that branch intentionally and documents it in `assembly.rs`.

`GameDetailSettingMenu$V` (`getPcGameUpdateOption`, Java 464–558) is
technically invoked from the detail header rather than `getMenuOptions`, but
its two options belong to the same class and are included here behind the
`is_steam_game` / `has_update` / `is_update_ignored` flags. It requires a Steam
AppID > 0 (Java 489–492); `IgnoreUpdate` calls
`ISteamGameDownloadService.l(appId, buildId)` (Java 295–302) and
`CancelIgnoreUpdate` calls `c(appId)` (Java 287–293).

## Out of scope

* **Android XPopup UI**: `OptionsPopup`, `MenuFloatView`, view anchoring and
  offsets (`GameDetailSettingMenu.o0/p0`, `MyGamesOperationPopupKt.j/k/l`),
  focus borders, and the `LinkedHashMap` de-duplication-by-label in
  `GameDetailSettingMenu$show$3` (lines 99–104).
* **Click callbacks**: the Kotlin `Function1` lambdas `d1, h1, g1, b1, c1,
  u0, z0, a1, m1` and their effects — `PageRouterUtils.o/y`,
  `CompatibleDialog`, `UninstallGameConfirmHelper.h`, `GameDetailShortCuter.c`,
  `ISteamGameDownloadService.c/l`; also coroutine dispatch in `m0`/`G`/`Y`.
  These are replaced by the pure `MenuAction` from `resolve()`.
* **Android resource resolution**: `Context.getString(R.string.*)`; labels
  are stored as static `&'static str` values verified against `strings.xml`.
* **Async / device-state queries** supplied as flags instead: `AppConfig.l()`
  (lite-mode suppression), `cst_data` presence, `GameStateMgr` /
  `SteamGameManager` installed checks (`i0`/`h0`), and the Steam download
  service queries `b/k/j/q`.
* **`add_to_homescreen`** (`strings.xml:84`, appended by decompiled
  `getMenuOptions`) and **`llauncher_change_start_game_type`**
  (`strings.xml:1042`, the `GameOperationPopup` fallback at
  `MyGamesOperationPopupKt.java:101–109`): neither is in the requested option
  list, so they are not modelled.
* Real Android `Option` objects, `FragmentActivity`, and any launcher/router
  integration (handled by other agent write areas).
