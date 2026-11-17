# Area report — OTA / APK update / device management / krly (Rust port)

Master report for the four write areas of this agent. Reference (read-only):
`sources/com/xj/ota/` (92 files), `sources/com/xj/apk/` (26),
`sources/com/xj/devicemanagement/` (28), `sources/com/krly/` (24) of
GameHub Lite v5.1.8. All modules are **std-only, dependency-free**, with
`#[cfg(test)]` unit tests. Layout follows `PLAN.md`: `mod.rs` + sibling `.rs`
at the mirrored Java package path, wired via `#[path]` in `app/src/lib.rs`.

*Per-area summaries:* `../apk/REPORT.md`, `../devicemanagement/REPORT.md`,
`../../krly/REPORT.md`.

## Files created

| File | Reference class(es) | Portable content |
|---|---|---|
| `ota/state.rs` | `set.BaseOTA`, `set.IUpListener`, `set.OtaUpState` | `OtaState` (Idle/Checking/Downloading/Verifying/Installing/Completed/Failed/Cancelled), `OtaEvent`, `OtaFailure` (with `IUpListener.b(int)` codes), `OtaEffect`, pure `transition()` / `can_transition()`, stateful `OtaStateMachine` |
| `ota/download.rs` | `http.HttpHandle`, `http.IDownloadListener`, `apk.update.config.DownloadStatus`, `apk.update.listener.OnDownloadListener` | `DownloadTask { id, url, destination, expected_md5, content_length, downloaded, progress, state }`, `DownloadTaskState`, `DownloadStatus` sealed-family enum, `DownloadEvent` |
| `ota/info.rs` | `OTAInfor`, `GamepadInfor`, `data.entity.GameSirersionsState`, `data.model.DeviceStatusInfo`, `data.model.RadiatorInfo` | `OtaInfo`, `GamepadInfo`, `FirmwareInfo`, `FirmwareEntry`, `FirmwareDataEntry`, `DeviceStatusInfo`, `RadiatorInfo`, firmware-type constants, X4A version joining, cached-file/MD5 rule |
| `ota/check.rs` | `check.FirmwareUpdateCheckUtil`, `DefaultUpdateCheck`, `G8UpdateCheck`, `VerCompareUtil` | `compare_versions`, `UpdateCheckStrategy`, `needs_upgrade`, `is_update_available`, `version_from_archive_path` |
| `ota/listener.rs` | `set.IUpListener` | `UpgradeEvent`, observed error-code constants (7, 3, 110, 711, 79, 71) |
| `ota/mod.rs` | package root | module declarations + re-exports |
| `apk/update.rs` | `apk.update.config.Constant`, `manager.DownloadManager` (+ `Builder`) | `ApkUpdateConfig`, validation from `a()`/`b()`, `UpdateDecision` from `download()`, `UpdatePaths`, cache/authority constants |
| `apk/mod.rs` | package root | includes shared `download.rs`, re-exports |
| `devicemanagement/device_info.rs` | `bussiness.devicemanagement.entity.DeviceInfo` (+ nested `FirmwareDataBean`) | 14-field `DeviceInfo`, defaults from the Kotlin no-arg constructor, `FirmwareDataBean` |
| `devicemanagement/byte_utils.rs` | `utils.StringUtil`, `ota.set.HexTools`, `krly.platform.util.Utils` | hex formatters, `bytes_to_u32_be`, `u16_to_le_bytes`, `byte_sum`, `crc16_ccitt` |
| `devicemanagement/mod.rs` | package root | module declarations + re-exports |
| `krly/command.rs` | `controller.KrCmdController`, `kr.request.*`, `kr.response.*`, `util.Utils` | `KrCommand` (codes 80–83), `KrRequest` + `BaseReq.c()` frame encode/decode, ready payload builder, `ReqFactory`, `SendBytesNumberRsp`, `CommandAck` |
| `krly/firmware_ota.rs` | `controller.FirmwareOtaController`, `controller.UpgradeCallBack` | `FirmwareOtaError`, `UpgradeEvent`, `chunk_firmware` (`x()`), `packet_index` (ceil), `FirmwareOtaPhase`, `FirmwareOtaSession` |
| `krly/mod.rs` | package root | module declarations + re-exports |

## Field sources (notable mappings)

- **OTAInfor** (`f3357x` obfuscated fields): names inferred from getters + call
  sites — `d()`→`download_url`, `e()`→`save_path`, `f()`→`expected_md5`,
  `n()`→`mode` (0 = verify MD5, non-zero = existence only), `l()`→`version`,
  `k()`→`manifest`.
- **DeviceStatusInfo / DeviceInfo**: defaults copied verbatim (e.g.
  `ble_ver`/`firmware`/`gcm_version` = `"0.0"`, `mode` ignores `<= 0`); Android
  `Bundle mExtras` → `BTreeMap<String, String>`.
- **DownloadStatus**: Kotlin sealed class variants `Start`, `Downloading(content,
  max, progress)`, `Done(File)`, `Error(Throwable)`, `Cancel` → enum; `File` and
  `Throwable` reduced to path/reason strings.
- **KrCommand codes**: from `BaseReq(int)` constructor calls in
  `DeviceOTAReadyReq` (80), `SendBytesNumberReq` (81), `SetDataSendReq` (82),
  `DeviceOTACompleteReq` (83). `DeviceOTAVerifyReq` and `DonglePairingReq` have
  **no constructor/code** in the decompiled output and are not invented.
- **Firmware OTA sequencing**: derived from `FirmwareOtaController.z/w/A/u/t/x/v/p`,
  `KrCmdController.d/e/g`, `OtaMessageCallback` codes (`100` retry, `-1`
  disconnect, `-2` notify, `-3` write), and `UpgradeCallBack.a/b/c`.
- **CRC-16**: `Utils.b()` polynomial `0x1021`, initial `0x0000`
  (`"123456789"` → `0x31C3`); implemented locally in each area that needs it to
  keep areas self-contained.

## Intentionally not implemented

- **Android DownloadService** and the whole APK install flow: `DownloadService`,
  `HttpDownloadManager`, notification channels, `FileProvider`, Activity
  lifecycle callbacks, `UpdateDialogActivity`, `NotificationUtil`, `ApkUtil`
  package-install calls. Only config validation / decisions and the download
  data model are portable.
- **BLE**: `BLEScanner`, `BLEService`, `BLEOtaManager`, all `devicemanagement`
  BLE receivers/listeners, `BluetoothStateListener`, GATT connect/read/write/
  notify, RSSI scanning and coroutine continuations.
- **USB**: `USBMonitor`, `USBFinder`, `UsbOTGService`, `USBPermissionUtil`,
  permission broadcasts and all USB OTA transports
  (`ZikWayUsbOTA`, `G8TypeCOTA`, etc.).
- **Android UI/platform**: Activities (`BaseBleOTAActivity`, `BaseUSBOTAActivity`,
  per-device OTA activities), databinding, dialogs, views, `Context`, OkHttp,
  SSL/`XDns`, `ConditionVariable`, threads/`Thread.sleep`, handler timing.
- **Device-specific flashing**: the concrete `BaseOTA.j()` implementations
  (JieLi, T4n, X3Pro, G8 Type-C, Nova2Lite, etc.) and BLE/USB packet protocols;
  only their control-flow ordering and error codes are represented.
- **Known decompiler gaps**: `DownloadStatus` / `Constant` are Kotlin classes
  whose JADX metadata failed to decompile; field names/values were recovered from
  the smali and call sites.

## Verification

Compiled and tested standalone with `rustc --test` using a temporary `#[path]`
harness that wires the four `mod.rs` files exactly as `lib.rs` is expected to:
**70 passed, 0 failed, 0 warnings**. No external crates used.
