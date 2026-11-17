# Rust reimplementation report — `com.winemu.core.gamepad` (Wine-side gamepad server)

Pure `std` Rust model of the gamepad **server** state used by the Wine side of
GameHub Lite v5.1.8. Write area: `app/sources/com/winemu/core/gamepad_server/`.
Reference (read-only): `reference/gamehub-lite-v518-decompiled/gamehub-lite-v518-decompiled/sources/com/winemu/core/gamepad/`.
No `Cargo.toml`, `app/src/lib.rs` or any other agent's directory was touched.
In particular the simpler BLE gamepad model in `app/sources/com/xj/device/ble/`
was **not** modified — this is the separate Wine-side server model.

Wiring (left to the parent crate):
```rust
#[path = "../sources/com/winemu/core/gamepad_server/mod.rs"]
pub mod gamepad_server;
```
All cross-module imports inside the area are relative (`super::…`), so the
parent may name the module anything it likes. A test harness compiled both
write areas under arbitrary module names with `rustc --test -D warnings`:
**51 tests passed, 0 warnings** (32 in this area, 19 in `core/input/`).

## Files written

| File | Contents |
|---|---|
| `mod.rs` | Module declarations + `pub use` re-exports; source index and wiring example. |
| `gamepad_state.rs` | `GamepadState` buffer snapshot (2 sticks, 2 triggers, 15-button mask, hat), `GamepadButton`/`GamepadButtons` bitmask, `GamepadHat` + D-pad mapping, `GamepadStatePatch` with field-level `apply`/`merge`, and the `e`/`f`/`p`/`z` conversion helpers from the router. |
| `player_slot.rs` | `PlayerSlot`, `SlotState` (plus `SlotOccupancy` classifier), and the `GamepadDevice` / `PhysicalDevice` / `VirtualDevice` identities used by slot logic. Implements `SlotState.b()` display naming (`"Empty"`, `"A + Virtual Gamepad"`). |
| `slot_manager.rs` | `SlotManager` with four slots, `assign` (descriptor grouping + first free slot), `first_free_slot`, `find_by_descriptor`, `slot_of_device_id`, `compact`, `remove`, `disconnect`, `swap`, `devices_in`, `player_slots`, `active_count`, `has_virtual`; `SlotError`. |
| `virtual_controller.rs` | `VirtualGamepadController` state (active virtual device + resolved slot + enabled flag), `VirtualGamepadEvent { slot, patch }` and all `j/l/n/p/r/t/v` operations as pure patch producers; `VirtualControllerError`. |
| `server_manager.rs` | `GamepadServerManager`: four state buffers, gamepad count validation, `onRumble` queue + descriptor-deduplicated rumble targets, intensity blend, `close`; `RumbleCommand`, `ServerError`. |
| `device_config.rs` | `DeviceConfig` disabled-device (blacklist) set with `disable`/`enable`/`is_disabled` and `apply_to`. |

## Sources consulted (read-only)

- `GamepadState.java`: buffer layout — sticks at offsets 0/2/4/6, triggers at
  8/10, 15 buttons at 12..26, hat at 27. `a()` = reset: sticks 0, triggers
  `-32767`, hat 0, all buttons cleared. `b(int, boolean)` ignores indices
  outside 0..=14.
- `GamepadInputRouter.java`:
  - `z(int)`: Android key code -> button index (96=A, 97=B, 99=X, 100=Y,
    4/109=Select, 110=Mode, 82/108=Start, 106=ThumbL, 107=ThumbR, 102=L1,
    103=R1, 19/20/21/22=D-pad; 82 and 4 are aliases; -1 = ignore).
  - `A(...)`: D-pad from axes 15/16 at ±0.5 (`b(11..14, …)`).
  - `e(...)` / `VirtualGamepadController.g(...)`: `(short) clamp(v * 0xffff,
    -32768, 32767)` (the obfuscated `DNSRecordClass.CLASS_MASK` is 0xffff).
  - `f(...)`: trigger square-law (`<=0 -> -32767`, `>=1 -> 32767`, else
    `sqrt(v)*65534 - 32767`); `p(...)`: discard values <= 0.01, clamp 0..1.
  - `u`/`w`: device id -> slot via `GamepadSlotManager.k`, then apply patch.
- `GamepadServerManager.java`: four `GamepadState` buffers, `c(int)` bounds,
  `o(int)` count validation (0..=4 modeled; the reference checks <5 but only
  allocates four buffers), `onRumble` slot guard 0..3 and `HashSet` dedupe by
  `device.a()` before `device.f(strong, weak)`.
- `GamepadSlotManager.java`: four slots, virtual gamepad forced into slot 0,
  physical devices grouped by descriptor (`h`), free-slot search (`g`),
  compaction (`f`), removal (`q`), disconnect (`n`, which only recompacts and
  does not delete a device), swap (`t`, virtual stays in slot 0), `k`/`j`/`l`.
- `PlayerSlot.java`, `SlotState.java`, `GamepadDevice.java`,
  `VirtualGamepadController.java`, `DeviceConfig.java`: field meanings,
  display names, "Virtual gamepad not assigned to any slot" error, blacklist
  by descriptor.

## Thresholds / constants found

| Constant | Value | Source |
|---|---|---|
| Gamepad buffer slots | 4 | `GamepadServerManager.f20451c = new GamepadState[4]`; `GamepadSlotManager.f20454b = new SlotState[4]` |
| Buttons per state | 15 | `GamepadState.a()` loops `i < 15` |
| Neutral trigger | -32767 | `GamepadState.a()` calls `g((short) -32767)` |
| Axis scale | 0xffff (65535) | `DNSRecordClass.CLASS_MASK` in `GamepadInputRouter.e` / `VirtualGamepadController.g` |
| Trigger deadzone | 0.01 | `GamepadInputRouter.p` (`if (fM > 0.01f)`) |
| Stick axis noise floor | 0.001 | `GamepadInputRouter.m` (axis selection), `j` uses `getFlat()` from Android `MotionRange` |
| Trigger axis noise floor | 0.01 | `GamepadInputRouter.m` for LEFT/RIGHT_TRIGGER |
| D-pad axis threshold | ±0.5 | `GamepadInputRouter.A`, axes 15/16 |
| Hat center | 0 | `GamepadState.a()` calls `c((byte) 0)` |
| Virtual slot | 0 | "Virtual gamepad assigned to slot 0" / "virtual gamepad stays in slot 0" |
| Rumble slot range | 0..3 | `GamepadServerManager.onRumble` guard `i3 < 0 || i3 >= 4` |
| Rumble dedupe | by `device.a()` descriptor | `GamepadServerManager.e` `HashSet.add(device.a())` |
| Rumble blend | 0.6·strong + 0.4·weak, 0..255 | `GamepadDevice.h` / `Virtual.h` (virbator intensity; VibrationEffect duration 250 ms / 1000 ms not modeled) |

Decompilation note: `GamepadSlotManager.g()` / `f()` retain JADX-mangled
control flow (empty inline loop bodies). The implementation follows the
observable behavior implied by the source log strings and the rest of the
class rather than the literal broken bytecode shape; this is documented in
`slot_manager.rs`.

## Out of scope (explicitly not implemented)

- **Native/JNI gamepad**: `System.loadLibrary("winemu")`, `nativeCreate`,
  `nativeDestroy`, `nativeGetGamepadBuffer`, `nativeUpdateGamepadCount`,
  `nativeSetRumbleCallback`, the native Unix socket and direct `ByteBuffer`
  writes. No FFI is present.
- **Android device I/O**: `InputDevice`/`MotionEvent`/`KeyEvent`, `Vibrator`
  and `VibratorManager` access, HID/GCM/BLE gamepad transports (those belong
  to the separate `com.xj.device.ble` model), USB `device_filter.xml`.
- **Runtime/lifecycle**: coroutines (`CoroutineScope`, `scheduleSyncToNative`),
  `Closeable` native handles, `Context`, logging, listener registration and
  `GamepadDeviceManager` scanning.
- **Real touch injection**: no `InputCallback`/`TouchEventDispatcher` calls;
  see the sibling `app/sources/com/winemu/core/input/` report for that model.
