//! Data model for the gamepad calibration screens.
//!
//! Motivation in the reference (all first-party, no native code needed here):
//!
//! - `sources/com/xj/devicesetting/gamepad/GamePadCalibrationActivity.java` —
//!   the calibration flow (a sequence of illustrated steps).
//! - `sources/com/xj/devicesetting/bean/JoystickSettingEntity.java` — per-stick
//!   inner/outer dead-zone values (`dead_zone_ljs_inner/outer`,
//!   `dead_zone_rjs_inner/outer`).
//! - `sources/com/xiaoji/sdk/bluetooth/le/Gamesir.java` — native helpers such
//!   as `setFirstPoint`, `touchpadTouchPoint` and `translatePoint`, which operate
//!   on 2-D touch/stick points. Only the portable point/range values are modeled
//!   here; the native `.so` itself is out of scope.
//!
//! Coordinates are plain `f32` pairs. The reference does not fix a single
//! numeric range, so callers supply the [`AxisRange`] that matches their screen
//! (e.g. a normalized `[-1.0, 1.0]` stick range or a pixel touch-pad range).

/// A 2-D calibration sample (touch point or joystick position).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CalibrationPoint {
    pub x: f32,
    pub y: f32,
}

impl CalibrationPoint {
    pub const ORIGIN: Self = Self { x: 0.0, y: 0.0 };

    pub const fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }

    /// Euclidean distance from the origin (stick displacement magnitude).
    pub fn magnitude(self) -> f32 {
        (self.x * self.x + self.y * self.y).sqrt()
    }

    pub fn distance_to(self, other: Self) -> f32 {
        Self::new(self.x - other.x, self.y - other.y).magnitude()
    }

    /// True when this point lies inside the given X and Y ranges (inclusive).
    pub fn is_inside(self, x_range: AxisRange, y_range: AxisRange) -> bool {
        x_range.contains(self.x) && y_range.contains(self.y)
    }

    /// Clamp each coordinate into its range, yielding a point inside both.
    pub fn clamp_to(self, x_range: AxisRange, y_range: AxisRange) -> Self {
        Self::new(x_range.clamp(self.x), y_range.clamp(self.y))
    }
}

/// An inclusive `[min, max]` range along one axis.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct AxisRange {
    pub min: f32,
    pub max: f32,
}

impl AxisRange {
    /// Build a range, swapping the endpoints if they were supplied reversed.
    pub fn new(min: f32, max: f32) -> Self {
        Self { min: min.min(max), max: min.max(max) }
    }

    /// A symmetric range `[-half, +half]` centered on zero.
    pub fn centered(half: f32) -> Self {
        Self::new(-half.abs(), half.abs())
    }

    pub fn contains(self, value: f32) -> bool {
        value >= self.min && value <= self.max
    }

    /// Restrict `value` to this range.
    pub fn clamp(self, value: f32) -> f32 {
        value.max(self.min).min(self.max)
    }

    pub const fn center(self) -> f32 {
        (self.min + self.max) / 2.0
    }

    pub fn span(self) -> f32 {
        self.max - self.min
    }

    /// A range is degenerate when its endpoints are equal.
    pub fn is_degenerate(self) -> bool {
        self.min == self.max
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn approx(a: f32, b: f32) -> bool {
        (a - b).abs() < 1e-6
    }

    #[test]
    fn axis_range_normalizes_and_checks() {
        let r = AxisRange::new(1.0, -1.0);
        assert_eq!(r.min, -1.0);
        assert_eq!(r.max, 1.0);
        assert!(r.contains(0.0));
        assert!(r.contains(-1.0));
        assert!(!r.contains(1.5));
        assert_eq!(r.clamp(5.0), 1.0);
        assert_eq!(r.clamp(-5.0), -1.0);
        assert_eq!(r.center(), 0.0);
        assert!(approx(r.span(), 2.0));
        assert!(!r.is_degenerate());
    }

    #[test]
    fn centered_and_degenerate_ranges() {
        let c = AxisRange::centered(0.5);
        assert_eq!(c.min, -0.5);
        assert_eq!(c.max, 0.5);
        assert!(AxisRange::new(2.0, 2.0).is_degenerate());
    }

    #[test]
    fn point_geometry() {
        let p = CalibrationPoint::new(3.0, 4.0);
        assert!(approx(p.magnitude(), 5.0));
        assert!(approx(p.distance_to(CalibrationPoint::ORIGIN), 5.0));
        assert_eq!(CalibrationPoint::ORIGIN.magnitude(), 0.0);
    }

    #[test]
    fn point_inside_and_clamp() {
        let x = AxisRange::new(-1.0, 1.0);
        let y = AxisRange::new(-1.0, 1.0);
        assert!(CalibrationPoint::new(0.5, -0.5).is_inside(x, y));
        assert!(!CalibrationPoint::new(1.5, 0.0).is_inside(x, y));
        let clamped = CalibrationPoint::new(2.0, -3.0).clamp_to(x, y);
        assert_eq!(clamped, CalibrationPoint::new(1.0, -1.0));
        assert!(clamped.is_inside(x, y));
    }
}
