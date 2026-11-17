//! Tap vs. swipe gesture recognition over a sequence of [`TouchData`].
//!
//! Threshold sources (all from the reference `com.winemu.core.input` package):
//!
//! | Value | Default | Source |
//! |---|---|---|
//! | touch slop | 8.0 px, squared before comparison | `SwipeHandler` ctor: `scaledTouchSlop * scaledTouchSlop`; `TapHandler` ctor: same. The literal comes from Android `ViewConfiguration`, so it is device/density dependent — here it is configurable, default 8.0. |
//! | two-pointer swipe | exactly 2 pointers; both must exceed slop; `dx0*dx1 + dy0*dy1 > 0` (parallel motion) | `SwipeHandler.b(MotionEvent)` lines 79–85. |
//! | long-press timeout | 500 ms (configurable) | `TapHandler.h`: `sendEmptyMessageDelayed(0, ViewConfiguration.getLongPressTimeout())`. |
//! | double-tap timeout | 300 ms (configurable) | `InputInterface.SimulatedTouchInput` ctor: `ViewConfiguration.getDoubleTapTimeout()`. |
//! | double-tap slop | 25.0 px, squared | `SimulatedTouchInput`: `(int)(getScaledDoubleTapSlop() * 0.25f)` then squared. Default slop 100 px * 0.25 = 25 px. |
//!
//! Direction classification is our own addition (the reference only asks
//! whether the two pointers moved in parallel): 8-way by dominant axis, with a
//! 2:1 dominance rule documented on [`Direction::from_delta`].

use std::collections::{BTreeMap, HashSet};

use super::touch_data::{TouchAction, TouchData};

pub const DEFAULT_TOUCH_SLOP: f32 = 8.0;
pub const DEFAULT_LONG_PRESS_TIMEOUT_MS: u64 = 500;
pub const DEFAULT_DOUBLE_TAP_TIMEOUT_MS: u64 = 300;
pub const DEFAULT_DOUBLE_TAP_SLOP: f32 = 25.0;

/// 8-way swipe direction, derived from the movement delta.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Direction {
    Up,
    UpRight,
    Right,
    DownRight,
    Down,
    DownLeft,
    Left,
    UpLeft,
}

impl Direction {
    /// Dominant-axis classification: a component at least twice the other
    /// yields a cardinal direction, otherwise the pair forms a diagonal.
    pub fn from_delta(dx: f32, dy: f32) -> Option<Self> {
        if dx == 0.0 && dy == 0.0 {
            return None;
        }
        let horizontal = dx.abs() >= dy.abs() * 2.0;
        let vertical = dy.abs() >= dx.abs() * 2.0;
        match (dx > 0.0, dy > 0.0) {
            (true, true) => {
                if horizontal {
                    Some(Self::Right)
                } else if vertical {
                    Some(Self::Down)
                } else {
                    Some(Self::DownRight)
                }
            }
            (false, true) => {
                if horizontal {
                    Some(Self::Left)
                } else if vertical {
                    Some(Self::Down)
                } else {
                    Some(Self::DownLeft)
                }
            }
            (true, false) => {
                if horizontal {
                    Some(Self::Right)
                } else if vertical {
                    Some(Self::Up)
                } else {
                    Some(Self::UpRight)
                }
            }
            (false, false) => {
                if horizontal {
                    Some(Self::Left)
                } else if vertical {
                    Some(Self::Up)
                } else {
                    Some(Self::UpLeft)
                }
            }
        }
    }
}

/// Result of a recognized gesture.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Gesture {
    Tap {
        x: f32,
        y: f32,
        pointer_count: usize,
        /// True when the press lasted at least the long-press timeout.
        long_press: bool,
    },
    Swipe {
        direction: Direction,
        dx: f32,
        dy: f32,
    },
}

/// `TapHandler.k(MotionEvent)`: has this pointer moved further than the
/// (squared) touch slop since it was first seen?
pub fn moved_beyond_slop(start: (f32, f32), current: (f32, f32), slop: f32) -> bool {
    let dx = current.0 - start.0;
    let dy = current.1 - start.1;
    dx * dx + dy * dy > slop * slop
}

/// `SwipeHandler.b(MotionEvent)` two-pointer test: both pointers must have
/// moved and their motion vectors must point in the same general direction.
pub fn parallel_swipe(
    start0: (f32, f32),
    current0: (f32, f32),
    start1: (f32, f32),
    current1: (f32, f32),
    slop: f32,
) -> Option<(f32, f32)> {
    let dx0 = current0.0 - start0.0;
    let dy0 = current0.1 - start0.1;
    let dx1 = current1.0 - start1.0;
    let dy1 = current1.1 - start1.1;
    let moved0 = dx0 * dx0 + dy0 * dy0 > slop * slop;
    let moved1 = dx1 * dx1 + dy1 * dy1 > slop * slop;
    if moved0 && moved1 && dx0 * dx1 + dy0 * dy1 > 0.0 {
        Some((dx0, dy0))
    } else {
        None
    }
}

/// Timeout/slop config for the double-tap check in `SimulatedTouchInput.d`.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct DoubleTapConfig {
    pub timeout_ms: u64,
    pub slop: f32,
}

impl Default for DoubleTapConfig {
    fn default() -> Self {
        Self {
            timeout_ms: DEFAULT_DOUBLE_TAP_TIMEOUT_MS,
            slop: DEFAULT_DOUBLE_TAP_SLOP,
        }
    }
}

/// `InputInterface.SimulatedTouchInput.d(float, float, long)`: compare the
/// scaled coordinates of two taps, requiring them to be close in both time and
/// space.
pub fn is_double_tap(first: &TouchData, second: &TouchData, config: &DoubleTapConfig) -> bool {
    let elapsed = second.time_ms.saturating_sub(first.time_ms);
    if elapsed > config.timeout_ms {
        return false;
    }
    let x0 = (first.x * first.scale.0) as i32;
    let y0 = (first.y * first.scale.1) as i32;
    let x1 = (second.x * first.scale.0) as i32;
    let y1 = (second.y * first.scale.1) as i32;
    let dx = x0 - x1;
    let dy = y0 - y1;
    let slop = config.slop as i64;
    dx as i64 * dx as i64 + dy as i64 * dy as i64 <= slop * slop
}

/// Stateful recognizer. Feed it touch samples in chronological order via
/// [`GestureRecognizer::push`]; each call returns at most one completed gesture.
#[derive(Debug, Clone)]
pub struct GestureRecognizer {
    pub touch_slop: f32,
    pub long_press_timeout_ms: u64,
    starts: BTreeMap<i32, (f32, f32, u64)>,
    positions: BTreeMap<i32, (f32, f32)>,
    moved: HashSet<i32>,
    swipe_baseline: Option<(i32, i32, (f32, f32), (f32, f32))>,
    emitted: bool,
}

impl Default for GestureRecognizer {
    fn default() -> Self {
        Self::with_touch_slop(DEFAULT_TOUCH_SLOP)
    }
}

impl GestureRecognizer {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_touch_slop(touch_slop: f32) -> Self {
        Self {
            touch_slop,
            long_press_timeout_ms: DEFAULT_LONG_PRESS_TIMEOUT_MS,
            starts: BTreeMap::new(),
            positions: BTreeMap::new(),
            moved: HashSet::new(),
            swipe_baseline: None,
            emitted: false,
        }
    }

    pub fn with_long_press_timeout_ms(mut self, timeout: u64) -> Self {
        self.long_press_timeout_ms = timeout;
        self
    }

    pub fn active_pointer_count(&self) -> usize {
        self.starts.len()
    }

    pub fn reset(&mut self) {
        self.starts.clear();
        self.positions.clear();
        self.moved.clear();
        self.swipe_baseline = None;
        self.emitted = false;
    }

    fn seed_swipe_baseline(&mut self) {
        let ids: Vec<i32> = self.starts.keys().copied().collect();
        if ids.len() == 2 {
            let p0 = self.positions[&ids[0]];
            let p1 = self.positions[&ids[1]];
            self.swipe_baseline = Some((ids[0], ids[1], p0, p1));
        }
    }

    /// Consume one sample. `None` means no gesture has completed yet.
    pub fn push(&mut self, touch: &TouchData) -> Option<Gesture> {
        let action = touch.action;
        let pos = (touch.x, touch.y);

        if action == TouchAction::Down {
            // TapHandler.i(): ACTION_DOWN clears all tracked pointers.
            self.reset();
        }

        match action {
            TouchAction::Down | TouchAction::PointerDown => {
                self.starts
                    .insert(touch.pointer_id, (touch.x, touch.y, touch.time_ms));
                self.positions.insert(touch.pointer_id, pos);
                self.moved.remove(&touch.pointer_id);
                self.seed_swipe_baseline();
                return None;
            }
            TouchAction::Move | TouchAction::HoverMove => {
                self.positions.insert(touch.pointer_id, pos);
                if let Some(&(sx, sy, _)) = self.starts.get(&touch.pointer_id) {
                    if moved_beyond_slop((sx, sy), pos, self.touch_slop) {
                        self.moved.insert(touch.pointer_id);
                    }
                }
                if self.emitted {
                    return None;
                }
                if let Some((id0, id1, p0, p1)) = self.swipe_baseline {
                    if let (Some(&c0), Some(&c1)) =
                        (self.positions.get(&id0), self.positions.get(&id1))
                    {
                        if let Some((dx, dy)) =
                            parallel_swipe(p0, c0, p1, c1, self.touch_slop)
                        {
                            if let Some(direction) = Direction::from_delta(dx, dy) {
                                self.emitted = true;
                                return Some(Gesture::Swipe { direction, dx, dy });
                            }
                        }
                    }
                }
                return None;
            }
            TouchAction::Up | TouchAction::PointerUp => {
                let start = self.starts.get(&touch.pointer_id).copied();
                let was_moved = self.moved.contains(&touch.pointer_id);
                let pointer_count = self.starts.len();
                self.starts.remove(&touch.pointer_id);
                self.positions.remove(&touch.pointer_id);
                self.moved.remove(&touch.pointer_id);
                self.swipe_baseline = None;

                let gesture = if !self.emitted && !was_moved && start.is_some() {
                    let long_press = start
                        .map(|(_, _, t0)| touch.time_ms.saturating_sub(t0) >= self.long_press_timeout_ms)
                        .unwrap_or(false);
                    Some(Gesture::Tap {
                        x: touch.x,
                        y: touch.y,
                        pointer_count,
                        long_press,
                    })
                } else {
                    None
                };
                if gesture.is_some() || self.starts.is_empty() {
                    self.reset();
                }
                return gesture;
            }
            TouchAction::Cancel => {
                self.reset();
                return None;
            }
            TouchAction::Outside | TouchAction::Scroll => return None,
        }
    }

    /// Convenience: feed a whole sequence, returning all completed gestures.
    pub fn push_all(&mut self, samples: &[TouchData]) -> Vec<Gesture> {
        samples.iter().filter_map(|s| self.push(s)).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::super::touch_data::TouchAction::*;

    fn sample(id: i32, action: TouchAction, x: f32, y: f32, t: u64) -> TouchData {
        TouchData::new(id, action, x, y).with_time(t)
    }

    #[test]
    fn tap_detected_within_slop() {
        let mut recognizer = GestureRecognizer::new();
        assert_eq!(
            recognizer.push_all(&[
                sample(0, Down, 100.0, 100.0, 0),
                sample(0, Move, 103.0, 100.0, 20),
                sample(0, Up, 103.0, 100.0, 40),
            ]),
            vec![Gesture::Tap {
                x: 103.0,
                y: 100.0,
                pointer_count: 1,
                long_press: false,
            }]
        );
    }

    #[test]
    fn movement_beyond_slop_suppresses_tap() {
        let mut recognizer = GestureRecognizer::new();
        assert_eq!(
            recognizer.push_all(&[
                sample(0, Down, 100.0, 100.0, 0),
                sample(0, Move, 120.0, 100.0, 20),
                sample(0, Up, 120.0, 100.0, 40),
            ]),
            Vec::new()
        );
        assert_eq!(
            moved_beyond_slop((100.0, 100.0), (107.0, 100.0), DEFAULT_TOUCH_SLOP),
            false
        );
        assert!(moved_beyond_slop(
            (100.0, 100.0),
            (109.0, 100.0),
            DEFAULT_TOUCH_SLOP
        ));
    }

    #[test]
    fn long_press_flag() {
        let mut recognizer = GestureRecognizer::new().with_long_press_timeout_ms(500);
        let gestures = recognizer.push_all(&[
            sample(0, Down, 10.0, 10.0, 0),
            sample(0, Up, 10.0, 10.0, 600),
        ]);
        assert_eq!(
            gestures,
            vec![Gesture::Tap {
                x: 10.0,
                y: 10.0,
                pointer_count: 1,
                long_press: true,
            }]
        );
    }

    #[test]
    fn two_pointer_parallel_swipe_directions() {
        let cases = [
            ((10.0, 0.0), Direction::Right),
            ((-10.0, 0.0), Direction::Left),
            ((0.0, -10.0), Direction::Up),
            ((0.0, 10.0), Direction::Down),
            ((10.0, 8.0), Direction::DownRight),
            ((-10.0, -8.0), Direction::UpLeft),
        ];
        for ((dx, dy), expected) in cases {
            let mut recognizer = GestureRecognizer::new();
            let gestures = recognizer.push_all(&[
                sample(0, Down, 100.0, 100.0, 0),
                sample(1, PointerDown, 200.0, 100.0, 1),
                sample(0, Move, 100.0 + dx, 100.0 + dy, 10),
                sample(1, Move, 200.0 + dx, 100.0 + dy, 11),
            ]);
            assert_eq!(
                gestures,
                vec![Gesture::Swipe {
                    direction: expected,
                    dx,
                    dy,
                }],
                "delta ({dx}, {dy})"
            );
        }
    }

    #[test]
    fn opposing_or_single_pointer_motion_is_not_swipe() {
        assert_eq!(
            parallel_swipe((0.0, 0.0), (10.0, 0.0), (0.0, 0.0), (-10.0, 0.0), 8.0),
            None
        );
        assert_eq!(
            parallel_swipe((0.0, 0.0), (10.0, 0.0), (0.0, 0.0), (0.0, 0.0), 8.0),
            None
        );
        let mut recognizer = GestureRecognizer::new();
        assert_eq!(
            recognizer.push_all(&[
                sample(0, Down, 100.0, 100.0, 0),
                sample(1, PointerDown, 200.0, 100.0, 1),
                sample(0, Move, 120.0, 100.0, 10),
                sample(1, Move, 180.0, 100.0, 11),
            ]),
            Vec::new()
        );
    }

    #[test]
    fn swipe_suppresses_later_tap_and_cancel_resets() {
        let mut recognizer = GestureRecognizer::new();
        let gestures = recognizer.push_all(&[
            sample(0, Down, 100.0, 100.0, 0),
            sample(1, PointerDown, 200.0, 100.0, 1),
            sample(0, Move, 120.0, 100.0, 10),
            sample(1, Move, 220.0, 100.0, 11),
            sample(0, Up, 120.0, 100.0, 20),
            sample(1, Up, 220.0, 100.0, 21),
        ]);
        assert_eq!(gestures.len(), 1);
        assert!(matches!(gestures[0], Gesture::Swipe { .. }));

        recognizer.reset();
        assert_eq!(
            recognizer.push_all(&[
                sample(2, Down, 1.0, 1.0, 0),
                sample(2, Cancel, 1.0, 1.0, 1),
                sample(2, Up, 1.0, 1.0, 2),
            ]),
            Vec::new()
        );
    }

    #[test]
    fn double_tap_uses_timeout_and_scaled_slop() {
        let config = DoubleTapConfig::default();
        let first = sample(0, Up, 100.0, 100.0, 1000).with_scale(2.0, 2.0);
        let close = sample(0, Up, 103.0, 100.0, 1200).with_scale(2.0, 2.0);
        let far = sample(0, Up, 200.0, 100.0, 1200).with_scale(2.0, 2.0);
        let late = sample(0, Up, 101.0, 100.0, 1400).with_scale(2.0, 2.0);
        assert!(is_double_tap(&first, &close, &config));
        assert!(!is_double_tap(&first, &far, &config));
        assert!(!is_double_tap(&first, &late, &config));
    }

    #[test]
    fn direction_classification() {
        assert_eq!(Direction::from_delta(0.0, 0.0), None);
        assert_eq!(Direction::from_delta(1.0, 0.0), Some(Direction::Right));
        assert_eq!(Direction::from_delta(0.0, -1.0), Some(Direction::Up));
        assert_eq!(Direction::from_delta(-3.0, 2.0), Some(Direction::DownLeft));
    }
}
