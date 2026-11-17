# Game detail — portable Rust reimplementation report

Scope: dependency-free Rust (`std` only) under
`app/sources/com/xj/landscape/launcher/ui/gamedetail/`. This port covers the
portable *model and decision* logic around the game-detail screen and game
startup types, distilled from the decompiled GameHub Lite v5.1.8 sources.

## Files written

| File | Contents |
| --- | --- |
| `mod.rs` | Module declarations + `pub use` re-exports; scope notes. |
| `start_type.rs` | `GameStartType`, `GameStartCategory`, the exact `START_TYPE_*` code constants, `code`/`from_code` mapping, and the two classification helpers. Includes `#[cfg(test)]` tests. |
| `launch_params.rs` | `GameLaunchParams` (`game_id`, `valid_id`, `start_type`, extra args), `ExtraArgs`, `GameLaunchParamsBuilder`. Includes tests. |
| `ui_state.rs` | `GameDetail`, `GameDetailUiState { Idle, Loading, Loaded{detail}, Error }`, `GameDetailEvent`, `GameDetailStateMachine` and transitions. Includes tests. |
| `launch_decision.rs` | `LocalAvailability`, `LaunchAction { LaunchEmu, LaunchMobile, LaunchCloud, PromptInstall, Unsupported }`, `decide_launch` / `decide_launch_for_code`. Includes tests. |
| `REPORT.md` | This file. |

No changes were made to `Cargo.toml`, `app/src/lib.rs`, or any other agent's
directory.

## Exact `start_type` codes found (not invented)

Authoritative source:
`reference/.../sources/com/xj/common/data/launch/LauncherTypeEnum.java`,
lines 23–34 (`public static final int START_TYPE_*`). These are the only
`start_type` constants defined in the decompiled source; every value used in
the Rust code is taken verbatim from here.

| Constant | Code | Rust variant | Semantic category |
| --- | --- | --- | --- |
| `START_TYPE_LAUNCH_OTHER_APP` | `1001` | `LaunchOtherApp` | Other |
| `START_TYPE_OPEN_URL` | `1201` | `OpenUrl` | Other |
| `START_TYPE_XBOX_CLOUD_GAMING` | `1202` | `XboxCloud` | CLOUD |
| `START_TYPE_HID_GAME` | `1301` | `HidGame` | MOBILE |
| `START_TYPE_MOBILE_PLAY` | `1302` | `Mobile` | MOBILE |
| `START_TYPE_PSLINK` | `1401` | `Pslink` | Other |
| `START_TYPE_PCLINK` | `1402` | `Pclink` | Other |
| `START_TYPE_PC_EMULATOR` | `1403` | `LocalPcEmu` | LOCAL_PC_EMU |
| `START_TYPE_MOVING_CLOUD_GAMING` | `1406` | `Cloud` | CLOUD |
| `START_TYPE_STEAM_GAME_BY_PC_EMULATOR` | `1407` | `SteamNative` | STEAM_NATIVE ("Steam version") |
| `START_TYPE_STEAM_GAME_DETAIL_BY_WEB` | `1501` | `SteamWebDetail` | Other |
| `START_TYPE_MOBILE_INSTALL_APP` | `1502` | `MobileInstall` | MOBILE (install flow) |

There is no `1404`, `1405`, `1408`, etc. constant in the decompiled source;
unknown/raw codes map to `None` / `LaunchAction::Unsupported` rather than to a
made-up variant.

### Classification helpers (also from `LauncherTypeEnum.java`)

- `isMobileGameStartType(int)` (lines 56–58) returns true for
  `1301 || 1302 || 1502 || 10 || 7`. Note `10` and `7` are the raw game *TYPE*
  codes `TYPE_MOBILE_PLAY = 10` and `TYPE_HID = 7` (lines 40–41), not
  `START_TYPE_*` codes. Reproduced as `MOBILE_GAME_START_CODES` and
  `is_mobile_game_start_type(code)`.
- `isPcEmuGameLaunchType(int)` (lines 60–62) returns true for `1403 || 1407`.
- `getMobileGameTypes()` (lines 52–54) lists `10, 7, 1301, 1302, 1502`.
- `GameStartupParams.isPcEmuGame()` (`com/xj/common/data/gameinfo/GameStartupParams.java`,
  lines 305–308) also returns true for `1403 || 1407`, and
  `getDownTypeDesc()` (lines 195–205) labels `1403` as the "demo version"
  (when `isPcDemoGame()`) and `1407` as the "Steam version".

## Sources consulted (read-only)

All under
`reference/gamehub-lite-v518-decompiled/gamehub-lite-v518-decompiled/sources/`:

- `com/xj/common/data/launch/LauncherTypeEnum.java` — the authoritative
  `START_TYPE_*` / `TYPE_*` constants and classification helpers.
- `com/xj/common/data/gameinfo/GameStartupParams.java` — `start_type` field,
  synthetic-constructor default, `isPcEmuGame()`, `getDownTypeDesc()`.
- `com/xj/common/data/gameinfo/GameDetailEntity.java` — `id`, `valid_id`,
  `localGameId`, `steamAppIdInt`, `currentOpenType`, `isMobileGame()`; source
  of the `GameDetail` / `GameLaunchParams` fields.
- `com/xj/landscape/launcher/ui/gamedetail/GameDetailActivity.java` —
  lines 1966–1999: the launch branching used for `decide_launch`
  (`1406` -> cloud event; `1403` -> local-game/state check then start;
  `1407` -> Steam download check then start; otherwise
  `isMobileGameStartType(start_type) && i3(curStartupParams)` -> mobile start).
- `com/xj/landscape/launcher/ui/gamedetail/GameDetailSettingMenu.java` —
  lines 759–760: `1403 || 1407 || 1501` treated as PC/Steam-related types.
- `com/xj/landscape/launcher/holder/GameDetailHeadViewHolder.java` —
  lines 568–571: `1403` / `1407` head-view handling.
- `com/xj/landscape/launcher/vm/GameDetailVM.java` — detail fetch/observe flow
  and the decorator map keyed by `1407` / `1403`; source of the
  Idle/Loading/Loaded/Error state model.
- `com/xj/landscape/launcher/data/model/entity/StartTypeEntity.java`,
  `com/xj/mapping/play/entity/NewStartTypeModel.java` — mapping-input entities
  (`/vtouch/startType`), confirming start-type strings such as `"hid"` /
  mode `key_name` values exist but are not numeric `start_type` codes.
- `com/xj/landscape/launcher/ui/gamedetail/` directory listing —
  `GameDetailActivity`, `GameDetailMoreInfoActivity`, `GameDetailSettingMenu`,
  `GameVideoActivity`, `NewsDetailActivity`, `PreviewPictureActivity`, and the
  `handler/` adapters/decorators.

## Launch decision mapping rationale

`decide_launch` is a portable, I/O-free mirror of the decompiled branching:

- `1403` (`LocalPcEmu`): decompiled code verifies the local game is present
  (`GameLibraryRepository.a(localGameId)` or `GameStateMgr.a(id)`) and returns
  without starting otherwise. Rust emits `LaunchEmu` only when
  `game_installed && emu_installed`, else `PromptInstall`.
- `1407` (`SteamNative`): decompiled code verifies the Steam game is downloaded
  (`ISteamGameDownloadService.b(appId)`) and uses the
  `SteamGameByPcEmuLaunchStrategy`. Rust emits `LaunchEmu` when
  `steam_native_installed && emu_installed`, else `PromptInstall`.
- `1301`/`1302`/`1502` (`HidGame`/`Mobile`/`MobileInstall`): decompiled
  `isMobileGameStartType` + installed-app check -> `LaunchMobile`, else
  `PromptInstall`; `1502` is the install flow, so it maps to `PromptInstall`.
- `1202`/`1406` (`XboxCloud`/`Cloud`): decompiled `1406` sends the cloud start
  event directly. Rust emits `LaunchCloud` when `cloud_available`, else
  `Unsupported`.
- `1001`, `1201`, `1401`, `1402`, `1501` and unknown codes have no portable
  launch path in this scope -> `Unsupported`.

## Out of scope

- Android `GameDetailActivity`, fragments, adapters, ViewBinding, `LiveData`,
  coroutine scopes and lifecycle handling (UI rendering only modelled as pure
  state transitions).
- Real game launching (`AppLauncher`, `LauncherConfig`, `LaunchResult`,
  `SendClickStartGameEvent`) and emulator process/executable execution.
- Steam and cloud protocols, PC-stream/pc-link/ps-link/xbox implementations,
  keymapping (`/vtouch/startType`) network calls.
- Gson/`@SerializedName` serialization, icons/drawables, file sizes and other
  non-portable fields of `GameStartupParams` / `GameDetailEntity`.
