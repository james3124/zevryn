//! Event vocabulary and handler shape for `com.winemu.core.input.InputCallback`.
//!
//! The Java interface has three methods:
//! - `a(float x, float y, int pointerId, boolean down, boolean trackpad)`
//! - `b(int action, int pointerId, int x, int y)`
//! - `c(float dx, float dy)`
//!
//! They are modeled as a single enum ([`InputCallbackEvent`]) that a handler
//! can consume. Action codes in `b()` are from `TouchEventDispatcher.p()`:
//! 18 = pointer down (source action 0/ACTION_DOWN or 5/ACTION_POINTER_DOWN),
//! 19 = pointer move (source action 2/ACTION_MOVE etc.),
//! 20 = pointer up (other actions).

/// Action codes used in `InputCallback.b(int, int, int, int)`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TouchActionCode {
    Down = 18,
    Move = 19,
    Up = 20,
}

impl TouchActionCode {
    pub fn from_i32(code: i32) -> Option<Self> {
        match code {
            18 => Some(Self::Down),
            19 => Some(Self::Move),
            20 => Some(Self::Up),
            _ => None,
        }
    }

    pub const fn code(self) -> i32 {
        self as i32
    }
}

/// One event emitted toward the native input sink.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum InputCallbackEvent {
    /// `InputCallback.a(x, y, pointer_id, down, trackpad)`.
    Pointer {
        x: f32,
        y: f32,
        pointer_id: i32,
        down: bool,
        trackpad: bool,
    },
    /// `InputCallback.b(action, pointer_id, x, y)`.
    TouchAction {
        action: TouchActionCode,
        pointer_id: i32,
        x: i32,
        y: i32,
    },
    /// `InputCallback.c(dx, dy)` — relative scroll/trackpad delta.
    Scroll { dx: f32, dy: f32 },
}

impl InputCallbackEvent {
    pub fn pointer(x: f32, y: f32, pointer_id: i32, down: bool, trackpad: bool) -> Self {
        Self::Pointer {
            x,
            y,
            pointer_id,
            down,
            trackpad,
        }
    }

    pub fn touch(action: TouchActionCode, pointer_id: i32, x: i32, y: i32) -> Self {
        Self::TouchAction {
            action,
            pointer_id,
            x,
            y,
        }
    }

    pub fn scroll(dx: f32, dy: f32) -> Self {
        Self::Scroll { dx, dy }
    }
}

/// Trait-like sink, the Rust equivalent of implementing `InputCallback`.
pub trait InputCallback {
    fn on_event(&mut self, event: InputCallbackEvent);
}

/// A sink that collects every event; useful for tests and dispatch modeling.
#[derive(Debug, Default, Clone)]
pub struct RecordingCallback {
    pub events: Vec<InputCallbackEvent>,
}

impl RecordingCallback {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn take(&mut self) -> Vec<InputCallbackEvent> {
        std::mem::take(&mut self.events)
    }
}

impl InputCallback for RecordingCallback {
    fn on_event(&mut self, event: InputCallbackEvent) {
        self.events.push(event);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn action_codes_round_trip() {
        assert_eq!(TouchActionCode::from_i32(18), Some(TouchActionCode::Down));
        assert_eq!(TouchActionCode::from_i32(19), Some(TouchActionCode::Move));
        assert_eq!(TouchActionCode::from_i32(20), Some(TouchActionCode::Up));
        assert_eq!(TouchActionCode::from_i32(21), None);
        assert_eq!(TouchActionCode::Move.code(), 19);
    }

    #[test]
    fn recording_callback_collects_events() {
        let mut sink = RecordingCallback::new();
        sink.on_event(InputCallbackEvent::pointer(1.0, 2.0, 3, true, false));
        sink.on_event(InputCallbackEvent::touch(TouchActionCode::Move, 3, 4, 5));
        sink.on_event(InputCallbackEvent::scroll(0.5, -0.5));
        assert_eq!(sink.take().len(), 3);
        assert!(sink.events.is_empty());
    }
}
