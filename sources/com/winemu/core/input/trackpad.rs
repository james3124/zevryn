//! Model of `com.winemu.core.input.TrackpadHandler`.
//!
//! The reference tracks one pointer id, records its last `(x, y)` on down and
//! emits `listener.a(last_x - x, last_y - y)` (note the reversed subtraction:
//! the delta points opposite the finger motion) on each move, updating `last`
//! afterwards. Up/cancel/pointer-up for the tracked id stops tracking.

use super::touch_data::TouchData;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TrackpadDelta {
    pub dx: f32,
    pub dy: f32,
}

#[derive(Debug, Default, Clone, Copy)]
pub struct TrackpadHandler {
    pub active_pointer_id: Option<i32>,
    last: (f32, f32),
}

impl TrackpadHandler {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn is_tracking(&self) -> bool {
        self.active_pointer_id.is_some()
    }

    pub fn last_position(&self) -> (f32, f32) {
        self.last
    }

    pub fn stop(&mut self) {
        self.active_pointer_id = None;
    }

    /// Feed a sample; returns a delta only while tracking the same pointer and
    /// moving it.
    pub fn push(&mut self, touch: &TouchData) -> Option<TrackpadDelta> {
        match touch.action {
            action if action.is_start() => {
                if self.active_pointer_id.is_none() {
                    self.active_pointer_id = Some(touch.pointer_id);
                    self.last = (touch.x, touch.y);
                }
                None
            }
            action if action.is_end() => {
                if self.active_pointer_id == Some(touch.pointer_id) {
                    self.stop();
                }
                None
            }
            action if action.is_move() => {
                if self.active_pointer_id != Some(touch.pointer_id) {
                    return None;
                }
                let delta = TrackpadDelta {
                    dx: self.last.0 - touch.x,
                    dy: self.last.1 - touch.y,
                };
                self.last = (touch.x, touch.y);
                Some(delta)
            }
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::super::touch_data::TouchAction;

    #[test]
    fn tracks_one_pointer_and_emits_reversed_delta() {
        let mut handler = TrackpadHandler::new();
        assert!(!handler.is_tracking());
        handler.push(&TouchData::new(4, TouchAction::Down, 100.0, 100.0));
        assert!(handler.is_tracking());
        assert_eq!(handler.last_position(), (100.0, 100.0));
        assert_eq!(
            handler.push(&TouchData::new(4, TouchAction::Move, 90.0, 110.0)),
            Some(TrackpadDelta { dx: 10.0, dy: -10.0 })
        );
        assert_eq!(handler.last_position(), (90.0, 110.0));
    }

    #[test]
    fn ignores_other_pointers_and_stops_on_up() {
        let mut handler = TrackpadHandler::new();
        handler.push(&TouchData::new(1, TouchAction::Down, 0.0, 0.0));
        assert_eq!(
            handler.push(&TouchData::new(2, TouchAction::Move, 5.0, 5.0)),
            None
        );
        handler.push(&TouchData::new(2, TouchAction::Up, 5.0, 5.0));
        assert!(handler.is_tracking());
        handler.push(&TouchData::new(1, TouchAction::Cancel, 0.0, 0.0));
        assert!(!handler.is_tracking());
        assert_eq!(handler.push(&TouchData::new(1, TouchAction::Move, 1.0, 1.0)), None);
    }
}
