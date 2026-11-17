//! Rust model of the portable `com.winemu.core.input` layer
//! (GameHub Lite v5.1.8).
//!
//! Covers the touch data record, tap/swipe gesture recognition and the event
//! vocabulary used by the native input sink. Android `MotionEvent`, the
//! `Handler`/`Looper` threading, `ViewConfiguration` lookups and real touch
//! injection (JNI / `InputManager`) are out of scope.
//!
//! Reference sources (read-only):
//! - `sources/com/winemu/core/input/TouchData.java` — record + scale/clamp.
//! - `sources/com/winemu/core/input/SwipeHandler.java` — two-pointer swipe.
//! - `sources/com/winemu/core/input/TapHandler.java` — tap / long press.
//! - `sources/com/winemu/core/input/TrackpadHandler.java` — relative deltas.
//! - `sources/com/winemu/core/input/TouchEventDispatcher.java` — action codes
//!   (18/19/20), button ids `[0,1,2,3]`, 50 ms tap delay, defaults 100/0.65.
//! - `sources/com/winemu/core/input/InputCallback.java` — three sink methods.
//! - `sources/com/winemu/core/input/InputInterface.java` — Null/Simulated/
//!   Trackpad implementations and their double-tap timing.
//!
//! Only `std` is used. Wire into the crate with, e.g.:
//! ```ignore
//! #[path = "../sources/com/winemu/core/input/mod.rs"]
//! pub mod gamepad_input;
//! ```

pub mod gesture;
pub mod input_callback;
pub mod input_interface;
pub mod touch_data;
pub mod touch_dispatcher;
pub mod trackpad;

pub use self::gesture::{
    is_double_tap, moved_beyond_slop, parallel_swipe, DoubleTapConfig, Direction, Gesture,
    GestureRecognizer, DEFAULT_DOUBLE_TAP_SLOP, DEFAULT_DOUBLE_TAP_TIMEOUT_MS,
    DEFAULT_LONG_PRESS_TIMEOUT_MS, DEFAULT_TOUCH_SLOP,
};
pub use self::input_callback::{
    InputCallback, InputCallbackEvent, RecordingCallback, TouchActionCode,
};
pub use self::input_interface::{
    InputInterfaceEvent, NullInput, SimulatedTouchInput, TrackpadInput,
};
pub use self::touch_data::{TouchAction, TouchData};
pub use self::touch_dispatcher::{
    TouchEventDispatcher, POINTER_SLOTS, SUPPORTED_BUTTONS, TAP_RELEASE_DELAY_MS,
};
pub use self::trackpad::{TrackpadDelta, TrackpadHandler};
