# Rust reimplementation report — `com.xj.device.ble`

Rust reimplementation of the portable device/controller model of GameHub Lite
v5.1.8. Write area: `app/sources/com/xj/device/ble/`.
Reference (read-only): `reference/gamehub-lite-v518-decompiled/gamehub-lite-v518-decompiled/`.

Only `std` is used. No `Cargo.toml`, `app/src/lib.rs`, or other agents' areas were
touched. Wiring is left to the parent crate via a `#[path]` module declaration:

```rust
#[path = "../sources/com/xj/device/ble/mod.rs"]
pub mod device_ble;
```

## Files written

| File | Contents |
|---|---|
| `mod.rs` | Module declarations and re-exports (`pub use`) for all public types; module-level documentation with source index. |
| `gamepad_button.rs` | `GamepadButton` enum (37 buttons) + `ButtonMask` bitmask; GCM byte codes, Android key codes and labels; `from_gcm_code` / `from_android_keycode` lookups. |
| `gcm_packet.rs` | `GcmPacket` frame model, `HeaderForm`, `DecodeError`, `checksum`, `offsets`; `encode` / `decode` with length + checksum verification. |
| `device_connection_state.rs` | `DeviceConnectionState` + `ConnectionEvent` state machine (`Disconnected → Scanning → Connecting → Connected → Calibrating → Disconnected`). |
| `calibration.rs` | `CalibrationPoint` and `AxisRange` for calibration screens. |

Tests live inline under `#[cfg(test)]`: 21 unit tests total, all passing
(`RUSTFLAGS="-D warnings" cargo test`, zero warnings). Coverage:
frame round-trip (both header forms, empty payload), checksum/length/timeout
rejection, every state transition and invalid-transition rejection, code
round-trips and uniqueness, calibration point/range helpers.

## Sources consulted (and what was verified)

### Button codes — `GamepadButton`
- `sources/com/xiaoji/sdk/utils/KeyCodeUtil.java`:
  `Companion.a(int)` (Android key code → GCM byte), `Companion.b(byte)` (reverse),
  `Companion.d(int)` (label).
- `apktool/smali_classes4/com/xiaoji/sdk/utils/KeyCodeUtil$Companion.smali`,
  method `b(B)I`: literal immediates confirm every byte value, unaffected by
  JADX's constant substitution — e.g. A = `-0x4c`, B = `-0x4b`, … Home =
  `-0x3e`, Screenshot = `-0x2c`, Macro = `-0x01`. `-0x34` is absent (no button
  maps to it), and `-0x33` maps to Android key code 197 (`R4`).
- `sources/com/xj/devicesetting/gamepad/GamePadTestActivity.java#E1`:
  independently confirms Android key codes A=96, B=97, X=99, Y=100, L1=102,
  R1=103, L2=104, R2=105, L3=106, R3=107, Start=108, Select=109, Home=3,
  Screenshot=120.
- `sources/com/xiaoji/sdk/constants/GcmKeyCode.java`,
  `sources/com/xiaoji/sdk/constants/HidKeyCode.java`,
  `sources/com/xj/devicesetting/api/HidKeyCode.java`, `GcmScanCode.java`:
  these constant classes lost their `const val` initializers during JADX
  decompilation (malformed Kotlin metadata), and their smali confirms they
  declare **no** static fields. Hence no codes were taken from them; all codes
  come from the verified `KeyCodeUtil` mapping above. The `ButtonMask` bit
  positions are our own one-bit-per-button assignment (the reference has no such
  mask) and each bit maps to a real, verified code.

### Frame layout — `GcmPacket`
All fields documented with their source in `gcm_packet.rs` module docs:
- `GcmProtocol.E(...)` (`cmd=byte[] {b4,b5,5,b6}`) and `F(...)` (`{b4,b5,4}`):
  build `[COMMAND, TYPE, LENGTH, (ACTION), payload, 0x00]`; `LENGTH` is
  `payload.len() + 5` (with action) or `+ 4` (without).
- `GcmProtocol.Q` (sum of bytes except last) and `r0` (`command[last] = Q(...)`):
  checksum algorithm and placement.
- `GcmProtocol.B(int)` error table: codes 3/4/5/7/8 name COMMAND / TYPE /
  LENGTH / ACTION / CHECKSUM, code 6 is minimum data length.
- `gcm/check/BaseCheck.java`: `b` = COMMAND equality (code 3), `c` = minimum
  length `>= 2 + min` (code 6), `d`/`e` = checksum equality (code 8).
- `gcm/check/NormalCommandWithActionCheck.java`: `h` = TYPE (code 4), `g` =
  `len == (b[2] & 0xFF)` (code 5), `f` = ACTION at byte 3 (code 7).
- `gcm/check/NormalCommandWithoutActionCheck.java`: `g` = TYPE (code 4),
  `f` = LENGTH (code 5), no ACTION field.
- `GcmProtocol.D(byte[])` selects which validator applies based on the command
  byte (and sometimes sub-fields), which is why the caller passes `HeaderForm`
  to `decode` rather than us guessing ACTION presence from the bytes.

### Connection lifecycle — `DeviceConnectionState`
- `sources/com/xj/bussiness/devicemanagement/utils/DeviceConnectionMonitor.java`:
  `IConnectStateListener.onConnectState(boolean)` and the
  `android.bluetooth.device.action.ACL_DISCONNECTED` broadcast that drives the
  disconnect notification.
- `sources/com/xj/devicesetting/gamepad/GamePadTestActivity.java`: connect
  callbacks (`onConnectState(true/false)`) around the gamepad test flow.
- `sources/com/xj/devicesetting/gamepad/GamePadCalibrationActivity.java`:
  the calibration step flow motivating the `Calibrating` state.
- The state names and the exact transition sequence were specified by the
  rebuild brief; the reference has no single named state enum.

### Calibration — `CalibrationPoint` / `AxisRange`
- `sources/com/xj/devicesetting/gamepad/GamePadCalibrationActivity.java`:
  multi-step calibration screens (drawables `..._calibration_1/2/3`).
- `sources/com/xj/devicesetting/bean/JoystickSettingEntity.java`: inner/outer
  dead-zone fields (`dead_zone_ljs_*`, `dead_zone_rjs_*`) — the reference uses
  no fixed coordinate range, hence caller-supplied `AxisRange`.
- `sources/com/xiaoji/sdk/bluetooth/le/Gamesir.java`: native point helpers
  (`setFirstPoint`, `touchpadTouchPoint`, `translatePoint`) confirming that
  2-D `(x, y)` point values are the calibration primitive.

### Device identity — `device_filter.xml` (context only)
- `resources/res/xml/device_filter.xml` declares three USB VID/PID pairs:
  `1356/3302`, `13623/264`, `13623/4374`. These match the mode checks in
  `GcmProtocol.S()` (`3302/1356` → DS5) and `V()` (`8201/1406` → Switch). No
  transport code was written; values noted for completeness only.

## Out of scope (explicitly not implemented)

- **BLE / USB transports**: GATT connect/write/notify, `WriteDataInterface`,
  `BleScanException` handling, USB device attachment, `device_filter.xml`
  filtering logic, and all Android `BluetoothDevice`/`UsbManager` APIs.
- **Native `.so` libraries**: `libGamesir.so` (loaded by
  `com.xiaoji.sdk.bluetooth.le.Gamesir`) and every other arm64-only native
  engine (Wine/X server/VFS, streaming, etc.). Only the portable point/range
  values those functions consume are modeled.
- **Android UI / runtime**: Activities, DataBinding layouts, coroutines,
  `Intent` broadcasts, the `com.xj.ota` / device-management flows beyond the
  abstract lifecycle, and any JNI.
- **Steam / cloud gaming / remote play**: excluded from the rebuild entirely.
- **Constant-class reconstruction**: the missing `const val` values in
  `GcmKeyCode`/`HidKeyCode` could only be recovered via `KeyCodeUtil`; no values
  were invented, and commands whose constants are unknown were left as raw
  `u8` in `GcmPacket` rather than guessed.
