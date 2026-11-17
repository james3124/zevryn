//! Model of `com.winemu.core.input.TouchEventDispatcher`.
//!
//! Defaults from the reference constructor:
//! - `sensitivity = 100.0f` (`f20546e`, adjustable via `q(float)`).
//! - `scroll_scale = 0.65f` (`f20547f`).
//! - supported virtual button ids `[0, 1, 2, 3]` (`f20541k`).
//! - 10 pointer slots (`f20550i`).
//! - tap release is posted with a 50 ms delay (`i()` / `postDelayed`), while
//!   the press is posted immediately.
//!
//! Native event injection is out of scope; every method returns the
//! [`InputCallbackEvent`] list it would have dispatched.

use super::input_callback::{InputCallbackEvent, TouchActionCode};
use super::touch_data::{TouchAction, TouchData};

pub const SUPPORTED_BUTTONS: [i32; 4] = [0, 1, 2, 3];
pub const POINTER_SLOTS: usize = 10;
pub const TAP_RELEASE_DELAY_MS: u64 = 50;

#[derive(Debug, Clone)]
pub struct TouchEventDispatcher {
    pub sensitivity: f32,
    pub scroll_scale: f32,
    pub pointer_slots: [bool; POINTER_SLOTS],
}

impl Default for TouchEventDispatcher {
    fn default() -> Self {
        Self::new()
    }
}

impl TouchEventDispatcher {
    pub fn new() -> Self {
        Self {
            sensitivity: 100.0,
            scroll_scale: 0.65,
            pointer_slots: [false; POINTER_SLOTS],
        }
    }

    pub fn set_sensitivity(&mut self, value: f32) {
        self.sensitivity = value;
    }

    pub fn set_scroll_scale(&mut self, value: f32) {
        self.scroll_scale = value;
    }

    fn supported(button: i32) -> bool {
        SUPPORTED_BUTTONS.contains(&button)
    }

    /// `p(MotionEvent, TouchData)`: translate one touch sample into action
    /// events. Down/pointer-down -> 18; move -> 19; up/cancel -> 19 then 20
    /// (the reference emits a move before releasing).
    pub fn dispatch_touch(&mut self, touch: &TouchData) -> Vec<InputCallbackEvent> {
        let (x, y) = touch.scaled_clamped();
        match touch.action {
            TouchAction::Move
            | TouchAction::HoverMove
            | TouchAction::Scroll
            | TouchAction::Outside => {
                if (touch.pointer_id as usize) < POINTER_SLOTS {
                    self.pointer_slots[touch.pointer_id as usize] = true;
                }
                vec![InputCallbackEvent::touch(TouchActionCode::Move, touch.pointer_id, x, y)]
            }
            action => {
                let code = if action.is_start() {
                    TouchActionCode::Down
                } else {
                    TouchActionCode::Up
                };
                if code == TouchActionCode::Up {
                    vec![
                        InputCallbackEvent::touch(TouchActionCode::Move, touch.pointer_id, x, y),
                        InputCallbackEvent::touch(TouchActionCode::Up, touch.pointer_id, 0, 0),
                    ]
                } else {
                    vec![InputCallbackEvent::touch(code, touch.pointer_id, x, y)]
                }
            }
        }
    }

    /// `i(int, boolean)`: tap a virtual button. Returns the delayed release
    /// first (50 ms) then the immediate press, matching the two `post` calls.
    pub fn tap(&self, button: i32, trackpad: bool) -> Vec<InputCallbackEvent> {
        if !Self::supported(button) {
            return Vec::new();
        }
        vec![
            InputCallbackEvent::pointer(0.0, 0.0, button, false, trackpad),
            InputCallbackEvent::pointer(0.0, 0.0, button, true, trackpad),
        ]
    }

    /// `l(int, boolean)`: press a virtual button.
    pub fn button_down(&self, button: i32, trackpad: bool) -> Option<InputCallbackEvent> {
        if Self::supported(button) {
            Some(InputCallbackEvent::pointer(0.0, 0.0, button, true, trackpad))
        } else {
            None
        }
    }

    /// `n(int, boolean)` / `j(...)`: release a virtual button.
    pub fn button_release(&self, button: i32, trackpad: bool) -> Option<InputCallbackEvent> {
        if Self::supported(button) {
            Some(InputCallbackEvent::pointer(0.0, 0.0, button, false, trackpad))
        } else {
            None
        }
    }

    /// `h(float, float, boolean)`: move the default pointer (id 0).
    pub fn move_pointer(&self, x: f32, y: f32, trackpad: bool) -> InputCallbackEvent {
        InputCallbackEvent::pointer(x, y, 0, false, trackpad)
    }

    /// `o(float, float)`: relative scroll.
    pub fn scroll(&self, dx: f32, dy: f32) -> InputCallbackEvent {
        InputCallbackEvent::scroll(dx * self.scroll_scale, dy * self.scroll_scale)
    }

    /// `m(PointF, int, boolean, boolean)`: pointer at explicit coordinates.
    pub fn pointer_at(
        &self,
        x: f32,
        y: f32,
        button: i32,
        down: bool,
        trackpad: bool,
    ) -> Option<InputCallbackEvent> {
        if Self::supported(button) {
            Some(InputCallbackEvent::pointer(x, y, button, down, trackpad))
        } else {
            None
        }
    }

    pub fn clear_slots(&mut self) {
        self.pointer_slots = [false; POINTER_SLOTS];
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn defaults_match_reference() {
        let dispatcher = TouchEventDispatcher::new();
        assert_eq!(dispatcher.sensitivity, 100.0);
        assert_eq!(dispatcher.scroll_scale, 0.65);
        assert_eq!(SUPPORTED_BUTTONS, [0, 1, 2, 3]);
        assert_eq!(dispatcher.pointer_slots.len(), 10);
    }

    #[test]
    fn touch_translation() {
        let mut dispatcher = TouchEventDispatcher::new();
        let touch = TouchData::new(2, TouchAction::Down, 5.0, 6.0)
            .with_screen(100, 100)
            .with_scale(2.0, 3.0);
        assert_eq!(
            dispatcher.dispatch_touch(&touch),
            vec![InputCallbackEvent::touch(TouchActionCode::Down, 2, 10, 18)]
        );
        let move_touch = TouchData::new(2, TouchAction::Move, 1.0, 1.0);
        assert_eq!(
            dispatcher.dispatch_touch(&move_touch),
            vec![InputCallbackEvent::touch(TouchActionCode::Move, 2, 1, 1)]
        );
        assert!(dispatcher.pointer_slots[2]);
        let up = TouchData::new(2, TouchAction::Up, 0.0, 0.0);
        assert_eq!(
            dispatcher.dispatch_touch(&up),
            vec![
                InputCallbackEvent::touch(TouchActionCode::Move, 2, 0, 0),
                InputCallbackEvent::touch(TouchActionCode::Up, 2, 0, 0),
            ]
        );
    }

    #[test]
    fn virtual_button_events() {
        let dispatcher = TouchEventDispatcher::new();
        assert_eq!(
            dispatcher.tap(1, false),
            vec![
                InputCallbackEvent::pointer(0.0, 0.0, 1, false, false),
                InputCallbackEvent::pointer(0.0, 0.0, 1, true, false),
            ]
        );
        assert!(dispatcher.tap(7, false).is_empty());
        assert_eq!(
            dispatcher.button_down(2, true),
            Some(InputCallbackEvent::pointer(0.0, 0.0, 2, true, true))
        );
        assert_eq!(
            dispatcher.button_release(2, true),
            Some(InputCallbackEvent::pointer(0.0, 0.0, 2, false, true))
        );
        assert_eq!(dispatcher.button_down(9, true), None);
        assert_eq!(
            dispatcher.move_pointer(3.0, 4.0, false),
            InputCallbackEvent::pointer(3.0, 4.0, 0, false, false)
        );
    }

    #[test]
    fn scroll_uses_scroll_scale() {
        let mut dispatcher = TouchEventDispatcher::new();
        assert_eq!(
            dispatcher.scroll(10.0, -20.0),
            InputCallbackEvent::scroll(6.5, -13.0)
        );
        dispatcher.set_scroll_scale(1.0);
        assert_eq!(
            dispatcher.scroll(10.0, 0.0),
            InputCallbackEvent::scroll(10.0, 0.0)
        );
    }
}
