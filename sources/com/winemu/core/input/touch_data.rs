//! Portable touch record, mirroring `com.winemu.core.input.TouchData`.
//!
//! The Java class carries a `PointF` position, a "last" point, pointer id,
//! action and screen/scale metadata used by `TouchEventDispatcher.p()`:
//! `x = clamp(event.x * scale.x, 0, width)`, `y = ...`. JNI injection is out of
//! scope; these are plain values.

/// Android `MotionEvent` action values used by the reference handlers.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TouchAction {
    Down = 0,
    Up = 1,
    Move = 2,
    Cancel = 3,
    PointerDown = 5,
    PointerUp = 6,
    HoverMove = 7,
    Scroll = 8,
    Outside = 10,
}

impl TouchAction {
    pub fn from_i32(code: i32) -> Option<Self> {
        match code {
            0 => Some(Self::Down),
            1 => Some(Self::Up),
            2 => Some(Self::Move),
            3 => Some(Self::Cancel),
            5 => Some(Self::PointerDown),
            6 => Some(Self::PointerUp),
            7 => Some(Self::HoverMove),
            8 => Some(Self::Scroll),
            10 => Some(Self::Outside),
            _ => None,
        }
    }

    pub const fn code(self) -> i32 {
        self as i32
    }

    pub const fn is_start(self) -> bool {
        matches!(self, Self::Down | Self::PointerDown)
    }

    pub const fn is_end(self) -> bool {
        matches!(self, Self::Up | Self::PointerUp | Self::Cancel)
    }

    pub const fn is_move(self) -> bool {
        matches!(self, Self::Move | Self::HoverMove)
    }
}

/// A single pointer sample.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TouchData {
    pub x: f32,
    pub y: f32,
    pub pointer_id: i32,
    pub action: TouchAction,
    /// Monotonic time in milliseconds, used by tap/double-tap timeouts.
    pub time_ms: u64,
    /// Screen width (`TouchData.f()`), used when clamping injected coords.
    pub screen_width: i32,
    /// Screen height (`TouchData.c()`).
    pub screen_height: i32,
    /// Horizontal / vertical scale (`TouchData.d()`, a `PointF`).
    pub scale: (f32, f32),
    /// Last coordinates (`TouchData.a()` / `g(x, y)`).
    pub last: (f32, f32),
}

impl TouchData {
    /// A simple sample with default metadata (`time_ms = 0`, neutral scale).
    pub fn new(pointer_id: i32, action: TouchAction, x: f32, y: f32) -> Self {
        Self {
            x,
            y,
            pointer_id,
            action,
            time_ms: 0,
            screen_width: 0,
            screen_height: 0,
            scale: (1.0, 1.0),
            last: (x, y),
        }
    }

    pub fn with_time(mut self, time_ms: u64) -> Self {
        self.time_ms = time_ms;
        self
    }

    pub fn with_screen(mut self, width: i32, height: i32) -> Self {
        self.screen_width = width;
        self.screen_height = height;
        self
    }

    pub fn with_scale(mut self, sx: f32, sy: f32) -> Self {
        self.scale = (sx, sy);
        self
    }

    /// `TouchData.g(float, float)`: update `last`, returning true if changed.
    pub fn set_last(&mut self, x: f32, y: f32) -> bool {
        let changed = x != self.last.0 || y != self.last.1;
        if changed {
            self.last = (x, y);
        }
        changed
    }

    /// Apply this record's scale, then clamp to screen bounds, matching the
    /// coordinate transform in `TouchEventDispatcher.p()`.
    pub fn scaled_clamped(&self) -> (i32, i32) {
        let x = (self.x * self.scale.0) as i32;
        let y = (self.y * self.scale.1) as i32;
        let clamp = |v: i32, max: i32| -> i32 {
            if max > 0 {
                v.clamp(0, max)
            } else {
                v.max(0)
            }
        };
        (clamp(x, self.screen_width), clamp(y, self.screen_height))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn action_round_trip_and_predicates() {
        for code in [0, 1, 2, 3, 5, 6, 7, 8, 10] {
            assert_eq!(TouchAction::from_i32(code).unwrap().code(), code);
        }
        assert_eq!(TouchAction::from_i32(99), None);
        assert!(TouchAction::Down.is_start());
        assert!(TouchAction::PointerDown.is_start());
        assert!(TouchAction::Up.is_end());
        assert!(TouchAction::Cancel.is_end());
        assert!(TouchAction::Move.is_move());
    }

    #[test]
    fn last_point_and_scaled_clamp() {
        let mut touch = TouchData::new(0, TouchAction::Move, 5.0, 6.0);
        assert!(!touch.set_last(5.0, 6.0));
        assert!(touch.set_last(7.0, 8.0));
        assert_eq!(touch.last, (7.0, 8.0));

        let touch = TouchData::new(0, TouchAction::Move, 150.0, -3.0)
            .with_screen(100, 100)
            .with_scale(2.0, 0.5);
        assert_eq!(touch.scaled_clamped(), (100, 0));
    }
}
