//! Portable state of `com.winemu.core.input.InputInterface` implementations.
//!
//! The reference has three implementations: `NullInput` (no-ops),
//! `SimulatedTouchInput` (touchscreen injection with double-tap tracking) and
//! `TrackpadInput` (relative trackpad injection). Only their state is modeled;
//! `TouchEventDispatcher` calls are replaced by emitted event lists.

use super::gesture::{
    is_double_tap, DoubleTapConfig, DEFAULT_DOUBLE_TAP_SLOP, DEFAULT_DOUBLE_TAP_TIMEOUT_MS,
};
use super::input_callback::InputCallbackEvent;
use super::touch_data::TouchData;

/// Commands accepted by an `InputInterface`, mirroring its four methods.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum InputInterfaceEvent {
    /// `a(MotionEvent)` — a new touch sample or pointer-up.
    Touch(TouchData),
    /// `b(int)` — a button/key was pressed.
    Button(i32),
    /// `c(int)` — begin tracking a button; returns true for simulated inputs.
    BeginButton(i32),
    /// `onScroll(float, float)`.
    Scroll { dx: f32, dy: f32 },
}

/// The no-op implementation: emits nothing, always returns false.
#[derive(Debug, Default, Clone, Copy)]
pub struct NullInput;

impl NullInput {
    pub fn new() -> Self {
        Self
    }

    pub fn handle(&self, _event: InputInterfaceEvent) -> Option<Vec<InputCallbackEvent>> {
        None
    }

    pub fn begin_button(&self, _button: i32) -> bool {
        false
    }
}

/// State of `SimulatedTouchInput`.
#[derive(Debug, Clone)]
pub struct SimulatedTouchInput {
    pub double_tap: DoubleTapConfig,
    last_tap: Option<TouchData>,
    active_button: i32,
}

impl Default for SimulatedTouchInput {
    fn default() -> Self {
        Self::new(DoubleTapConfig {
            timeout_ms: DEFAULT_DOUBLE_TAP_TIMEOUT_MS,
            slop: DEFAULT_DOUBLE_TAP_SLOP,
        })
    }
}

impl SimulatedTouchInput {
    pub fn new(double_tap: DoubleTapConfig) -> Self {
        Self {
            double_tap,
            last_tap: None,
            active_button: 0,
        }
    }

    /// `b(int)`: track tap timing for button 1; other buttons clear history.
    /// Returns true when the press completed a double tap.
    pub fn on_button(&mut self, button: i32, tap: TouchData) -> bool {
        let is_double = if button == 1 {
            match &self.last_tap {
                Some(previous) if is_double_tap(previous, &tap, &self.double_tap) => true,
                _ => {
                    self.last_tap = Some(tap);
                    false
                }
            }
        } else {
            false
        };
        if is_double {
            self.last_tap = None;
        }
        is_double
    }

    pub fn clear_tap_history(&mut self) {
        self.last_tap = None;
    }

    /// `a(MotionEvent)`: on UP, release the currently tracked button.
    pub fn on_touch(&mut self, touch: &TouchData) -> Option<i32> {
        if touch.action.is_end() && self.active_button != 0 {
            let released = self.active_button;
            self.active_button = 0;
            Some(released)
        } else {
            None
        }
    }

    pub fn begin_button(&mut self, button: i32) -> bool {
        self.active_button = button;
        true
    }

    pub fn active_button(&self) -> i32 {
        self.active_button
    }
}

/// State of `TrackpadInput`: the pointer currently producing relative motion.
#[derive(Debug, Default, Clone, Copy)]
pub struct TrackpadInput {
    pub active_pointer_id: Option<i32>,
}

impl TrackpadInput {
    pub fn new() -> Self {
        Self::default()
    }

    /// `a(MotionEvent)`: clear the tracked pointer on UP.
    pub fn on_touch(&mut self, touch: &TouchData) {
        if touch.action.is_end() && self.active_pointer_id == Some(touch.pointer_id) {
            self.active_pointer_id = None;
        }
    }

    pub fn begin_button(&mut self, button: i32) -> bool {
        self.active_pointer_id = Some(button);
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::super::touch_data::TouchAction;

    #[test]
    fn null_input_is_inert() {
        let input = NullInput::new();
        assert!(!input.begin_button(1));
        assert_eq!(input.handle(InputInterfaceEvent::Button(1)), None);
    }

    #[test]
    fn simulated_double_tap_tracking() {
        let mut input = SimulatedTouchInput::default();
        let first = TouchData::new(0, TouchAction::Up, 10.0, 10.0).with_time(100);
        let second = TouchData::new(0, TouchAction::Up, 11.0, 10.0).with_time(200);
        assert!(!input.on_button(1, first));
        assert!(input.on_button(1, second));
        assert!(!input.on_button(2, TouchData::new(0, TouchAction::Up, 1.0, 1.0)));
    }

    #[test]
    fn simulated_touch_releases_button() {
        let mut input = SimulatedTouchInput::default();
        assert!(input.begin_button(3));
        assert_eq!(input.active_button(), 3);
        assert_eq!(
            input.on_touch(&TouchData::new(0, TouchAction::Up, 0.0, 0.0)),
            Some(3)
        );
        assert_eq!(input.active_button(), 0);
    }

    #[test]
    fn trackpad_pointer_lifecycle() {
        let mut input = TrackpadInput::new();
        assert!(input.begin_button(5));
        assert_eq!(input.active_pointer_id, Some(5));
        input.on_touch(&TouchData::new(5, TouchAction::Move, 1.0, 1.0));
        assert_eq!(input.active_pointer_id, Some(5));
        input.on_touch(&TouchData::new(5, TouchAction::Up, 1.0, 1.0));
        assert_eq!(input.active_pointer_id, None);
    }
}
