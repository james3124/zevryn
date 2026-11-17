# Rust reimplementation report — `com.winemu.core.input` (touch/gesture model)

Pure `std` Rust model of the touch-input layer of GameHub Lite v5.1.8.
Write area: `app/sources/com/winemu/core/input/`.
Reference (read-only): `reference/gamehub-lite-v518-decompiled/gamehub-lite-v518-decompiled/sources/com/winemu/core/input/`.
No `Cargo.toml`, `app/src/lib.rs` or any other agent's directory was touched.

Wiring (left to the parent crate):
```rust
#[path = "../sources/com/winemu/core/input/mod.rs"]
pub mod gamepad_input;
```
All cross-module imports inside the area are relative (`super::…`), so the
parent may name the module anything it likes. A test harness compiled both
`core/gamepad_server/` and this area with `rustc --test -D warnings`:
**51 tests passed, 0 warnings** (19 in this area, 32 in `core/gamepad_server/`).

## Files written

| File | Contents |
|---|---|
| `mod.rs` | Module declarations + `pub use` re-exports; source index and wiring example. |
| `touch_data.rs` | `TouchAction` (Android action codes 0/1/2/3/5/6/7/8/10), `TouchData` (x, y, pointer_id, action, time, screen bounds, scale, last point) with `set_last` and `scaled_clamped`. |
| `gesture.rs` | `Gesture` (`Tap` / `Swipe` + `Direction`), `GestureRecognizer` (stateful sequence consumer), plus free helpers `moved_beyond_slop`, `parallel_swipe`, `is_double_tap` and `DoubleTapConfig`. All reference thresholds documented at the top of the file. |
| `input_callback.rs` | `InputCallbackEvent` enum mirroring the three `InputCallback` methods, `TouchActionCode` (18/19/20), `InputCallback` handler trait and `RecordingCallback`. |
| `input_interface.rs` | `InputInterfaceEvent`, `NullInput`, `SimulatedTouchInput` (double-tap tracking) and `TrackpadInput` state. |
| `touch_dispatcher.rs` | `TouchEventDispatcher` model with defaults 100.0 / 0.65, supported buttons `[0,1,2,3]`, 10 pointer slots, 50 ms tap-release delay; touch translation plus virtual-button/scroll event builders. |
| `trackpad.rs` | `TrackpadHandler` state: single tracked pointer, reversed relative deltas (`last - current`). |

## Sources consulted (read-only)

- `TouchData.java`: `f20534a` (`d()`, scale `PointF`), `f20539f` (`a()`/`g()`,
  last point), `f20535b`/`f20536c` (`f()`/`c()`, screen width/height),
  `f20538e` (`b()`/`h()`, action). `g(x,y)` updates the last point only on
  change and reports whether it changed.
- `SwipeHandler.java`: exactly two pointers; on down/pointer-down seed one
  baseline pair; on move compare each pointer's squared displacement against
  `scaledTouchSlop²`; if **both** exceeded it, set `swiped = dot(d0,d1) > 0`.
  The Java decompilation's nested condition (lines 79–85) evaluates to exactly
  that. Direction extraction is our own addition; the reference only asks
  whether the two-pointer motion is parallel.
- `TapHandler.java`: `e()` cancels the long-press message, `i()` clears all
  pointers on ACTION_DOWN, `j()` records an `InitPoint` per pointer id, `k()`
  marks a pointer moved when squared displacement > `scaledTouchSlop²`,
  `h()` cancels the tap on move/cancel and reports the tap on up via `c(count,
  x, y)` when the point was not marked moved; a delayed message fires `b()`.
  `ViewConfiguration.getLongPressTimeout()` supplies the delay.
- `TrackpadHandler.java`: tracks one pointer id; on move emits
  `listener.a(lastX - x, lastY - y)` and updates `last`; up/cancel/pointer-up
  for the tracked id stops tracking.
- `TouchEventDispatcher.java`: `p()` action codes 18 (DOWN/POINTER_DOWN), 19
  (MOVE) and 20 (other), emitting 19 before 20 on release; `i()` posts a
  50 ms delayed release then an immediate press; `l()`/`n()`/`m()`/`h()` map
  to `InputCallback.a(...)`; `o()` maps to `c(dx,dy)`. Constructor defaults:
  sensitivity 100.0, scroll scale 0.65, `[0,1,2,3]` button ids, 10 slots.
- `InputCallback.java`: the three sink methods (see table above).
- `InputInterface.java`: `NullInput` no-ops; `SimulatedTouchInput.d()` double
  tap = scaled-coordinate squared distance <= `(getScaledDoubleTapSlop() *
  0.25)²` within `getDoubleTapTimeout()`; `TrackpadInput` uses the same
  dispatcher calls with `trackpad = true`.
- `TouchInputManager.java` was inspected for context only; its
  `View.OnTouchListener` / gesture wiring is Android-specific and out of scope.

## Thresholds / constants found

| Constant | Value | Source |
|---|---|---|
| Touch slop (tap + swipe) | `ViewConfiguration.getScaledTouchSlop()`; default **8.0 px** in this model, squared before comparison | `SwipeHandler` ctor line 35, `TapHandler` ctor line 89. Android supplies the literal, so it is device/density dependent; exposed as `GestureRecognizer::touch_slop`. |
| Swipe pointer count | exactly **2** | `SwipeHandler.b` `getPointerCount() != 2` -> reset |
| Swipe direction test | both pointers moved > slop² **and** `dx0*dx1 + dy0*dy1 > 0` | `SwipeHandler.b` lines 70–85 |
| Long-press timeout | `ViewConfiguration.getLongPressTimeout()`; default **500 ms** here | `TapHandler.h` line 117 |
| Double-tap timeout | `ViewConfiguration.getDoubleTapTimeout()`; default **300 ms** | `InputInterface.SimulatedTouchInput` ctor line 75 |
| Double-tap slop | `getScaledDoubleTapSlop() * 0.25`, squared; default slop 100 px -> **25 px** | `SimulatedTouchInput` lines 76–77, `d()` line 128 |
| Tap release delay | **50 ms** | `TouchEventDispatcher.i` `postDelayed(..., 50L)` |
| Dispatcher defaults | sensitivity **100.0**, scroll scale **0.65** | `TouchEventDispatcher` fields `f20546e` / `f20547f` |
| Virtual button ids | `[0, 1, 2, 3]` | `TouchEventDispatcher.f20541k = Arrays.asList(0,1,2,3)` |
| Pointer slot count | **10** | `TouchEventDispatcher.f20550i = new boolean[10]` |
| Input action codes | DOWN=18, MOVE=19, UP=20 | `TouchEventDispatcher.p` lines 155–159 |

Swipe `Direction` (8-way, 2:1 dominance rule) is de novo: the reference never
classifies a direction, only "parallel or not". All other numeric values above
come from the cited sources or documented Android `ViewConfiguration` defaults
and are configurable in Rust.

## Out of scope (explicitly not implemented)

- **Real touch injection / JNI**: no `InputCallback` native implementation,
  no `InputManager.injectInputEvent`, no `MotionEvent` construction or
  `SystemClock.uptimeMillis()` dependency. `InputCallbackEvent` values are
  returned to the caller instead.
- **Android UI/runtime**: `View`, `ViewConfiguration` lookups, `Handler`/
  `Looper` scheduling (the 50 ms delay is only documented, not slept),
  `WeakReference` message handlers, `Context`, `PointF`, `SparseArray` and
  gesture detectors from the Android framework.
- **Native gamepad/server side**: `GamepadState`, slot assignment and rumble;
  see the sibling `app/sources/com/winemu/core/gamepad_server/` report. The BLE
  controller model in `app/sources/com/xj/device/ble/` is a separate area and
  was not modified.
- **Higher-level input flows**: `TouchInputManager` activity/service wiring,
  key mapping remapping and cloud/streaming input paths.
