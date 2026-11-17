# Report — Rust reimplementation of the launcher ViewModel state

Write area: `app/sources/com/xj/landscape/launcher/vm/`
Read-only reference: `reference/gamehub-lite-v518-decompiled/gamehub-lite-v518-decompiled/sources/com/xj/landscape/launcher/vm/`

Only `std` is used. No `Cargo.toml`, `app/src/lib.rs` or any other agent's
directory was touched. All modules are `pub` plain structs/enums with a
`mod.rs` that re-exports everything.

## Files written

| File | Purpose | Mirrors |
|---|---|---|
| `mod.rs` | Area root: declares and re-exports all submodules. | (wiring) |
| `async_state.rs` | `AsyncState<T> { Idle, Loading, Success(T), Error(String) }` with `is_loading`, `is_idle`, `is_success`, `is_error`, `value`, `value_mut`, `error`, `unwrap_or`, `map`, `begin`, `succeed`, `fail`, `reset`, `take_value`, `into_value`, plus `Default`. | LiveData/`UnPeekLiveData` + boolean loading flags used across all studied VMs (e.g. `GameDetailVM.f30419a`, `NewLandscapeLauncherMainVM.f30447e`) |
| `types.rs` | Self-contained value types: `PlayButtonState`, `StartType` (with real `LauncherTypeEnum` codes and `is_mobile_game`/`is_pc_emu`/`needs_decorator`), `GameCircle`, `TabItem`, `ConnectedDevice`, `GameDetail`, `UserSteamGameState`, `SteamPrice`. | `com.xj.common.data.launch.LauncherTypeEnum.java`, `com.xj.common.data.gameinfo.PlayButtonState.java`, entity shapes referenced by the VMs |
| `main_view_model.rs` | `MainViewModel` (tabs, game-circle list, selected tab, `(index, needRefresh)` selection, network flag), `RefreshState { Idle, Loading, Ready, Failed }`, pure `initial_tab_for_circles(...)`. | `NewLandscapeLauncherMainVM.java` (fields `f30443a`–`f30450h`, `n/r/t/u/x`, `AnonymousClass1.invokeSuspend`), `LandscapeLauncherMainVM.java` (deprecated list/tab LiveData, `K/z/O/v/y`) |
| `game_detail_view_model.rs` | `GameDetailViewModel`: detail fetch state, download-status refresh + play-button resolver, BuyNow→Steam-price side effect, Steam-vs-native start-type switch (`apply_steam_game_state`, `switch_start_type`), `is_valid_steam_app_id`. | `GameDetailVM.java` (`f30419a`–`f30425g`, `H/J/L/N/O/P/r/y/z/A`, `refreshDownloadStatusUI$2`, `fetchGameDetailInfo$1$1`) |
| `device_manager_view_model.rs` | `DeviceManagerViewModel`: connected-device list `AsyncState<Vec<_>>`, selected device id, selection/lookup, connected filter, disconnect clearing. | `DeviceManagerVM.java` (bare `BaseViewModel`), `MineDeviceVM.java` (`f30439a` item LiveData, `k(devicesName)` → `MineDeviceRepository.a(...)` → post) |
| `misc_vms.rs` | Lightweight portable state for the other studied VMs: `AlbumDetailViewModel` (loading guard + pagination), `FeedbackViewModel` (one-shot event), `HightlightsViewModel` (keyword search), `Pager` + `SearchMainViewModel`, `GuideLoginViewModel` (login result). | `AlbumDetailVM.java`, `FeedbackVM.java`, `HightlightsVM.java`, `SearchMainVM.java`, `com.xj.common.utils.pager.PagerImpl.java`/`Pager.java`, `GuideLoginVM.java` |

## Key behaviors modeled

- `AsyncState` transition set (`Idle → Loading → Success/Error`, `take_value`
  returning to `Idle`) with accessor tests for every helper.
- Main VM's locale-dependent initial tab: empty game-circle list **and** zh
  locale **and** no app flag → tab `2`; all other cases → tab `0`; posted
  `needRefresh` is always `true` (`NewLandscapeLauncherMainVM.AnonymousClass1`).
- `select_tab` returns/clears the one-shot `needRefresh` flag posted by
  `n(int, boolean)`; failed home loads surface as `RefreshState::Failed`.
- `PlayButtonState`/`StartType` predicates use the exact decompiled codes:
  mobile start types `1301/1302/1502/10/7`, PC-emulator types `1403/1407`,
  `PCLINK = 1402`, Steam-version type `1407`.
- Download-status resolver mirrors `refreshDownloadStatusUI`: decorator types
  and mobile types use the installed proxy, `PCLINK` or config-allowed types
  are playable, everything else is `GetTheGame`. `BuyNow` queues a Steam price
  query (the `refreshDownloadStatusUI$2` side effect).
- Steam-vs-native switch mirrors `changeSteamPcGameStartUpParamsIfUseSteamVersion`:
  `Downloaded`/`ImportedByUser`/`ImportedByTool`/`InDownloadQueue` → Steam
  version (`start_type = 1407`); otherwise the recorded native start type is
  restored. No Steam service/protocol is invoked.
- `Pager` mirrors `PagerImpl` exactly: page starts at `1`, page size is
  `i32::MAX`, `d()` is `page == 1`, and `e(max)` sets `page = min(max, page+1)`.
- Device selection: first loaded device auto-selected (matching the single
  `DeviceSubMenuItemEntity` posted by `MineDeviceVM.k`), disconnecting the
  selected device clears the selection.

## Verification

Standalone, dependency-free compile + test run (no Cargo changes):

```
rustc --edition 2021 --test sources/com/xj/landscape/launcher/vm/mod.rs
# test result: ok. 31 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
```

Zero warnings.

## Not implemented (out of scope / non-portable)

- **Coroutines / Flow / LiveData**: `ScopeKt.d`, `SuspendLambda`, `AndroidScope`,
  `SafeContinuation`, `UnPeekLiveData`/`MutableLiveData` observers. State is
  modeled as plain values; no reactive streams or lifecycle ownership.
- **Repository / network I/O**: `HomeInfoRepository`, `LandscapeLauncherRepository`,
  `GameDetailRepository`, `MineDeviceRepository`, `HightlightsRepository`,
  `SearchGameRepository`, Steam service calls, and all `Post`/`Get` extension
  functions. Results enter through `on_*` methods only.
- **Android lifecycle / platform**: `BaseViewModel.onCleared`, `Application`/
  `Context`, `Looper`/`MessageQueue.IdleHandler`, `NetworkUtils` listeners,
  `Utils.a()`, resource strings and `GHLocaleManager` internals. Locale/app
  flags are plain `bool` inputs; network state is a plain `bool` field.
- **Game start / Steam protocol**: `AppLauncher`, `LauncherHelper`,
  `SteamGameManager`, `IGameHeadEntityDecorator` implementations,
  `LauncherConfig`, `UninstallGameHelper`, `TheRouter`, download engines
  (Aria). Only the start-type *choice* is kept, never the launch itself.
- **Rich data models**: the real DTOs live in `../data` and the game-model
  agents' areas. This module deliberately uses minimal local stand-ins
  (`GameDetail`, `GameCircle`, `ConnectedDevice`, …) and does not import them.
- **UI / views**: fragments, adapters, DataBinding, focus handling and all
  launcher screens are owned by other agents.

## Wiring note for the orchestrator

`app/src/lib.rs` is intentionally untouched. To compile this area, add:

```rust
#[path = "sources/com/xj/landscape/launcher/vm/mod.rs"]
pub mod launcher_vm;
```
