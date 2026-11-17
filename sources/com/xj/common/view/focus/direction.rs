//! `FocusDirection` — the D-pad directions of GameHub Lite's custom focus
//! system (`com.xj.common.view.focus.focus.FocusDirection`).
//!
//! The Java enum also declares `L1`, `R1`, `BUTTON_A` and `X`. Those are
//! gamepad *button* events (handled separately by the launcher activity), not
//! navigation axes, so they are deliberately not represented here. Only the
//! four portable navigation directions are modelled.

use std::fmt;

/// The four navigation directions understood by the focus controllers.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum FocusDirection {
    Left,
    Right,
    Up,
    Down,
}

impl FocusDirection {
    /// All directions in declaration order (mirrors `values()`).
    pub const ALL: [Self; 4] = [Self::Left, Self::Right, Self::Up, Self::Down];

    /// Opposite direction: `Left <-> Right`, `Up <-> Down`.
    ///
    /// The decompiled controllers never spell this out, but every `e()` /
    /// `d()` handler pairs a direction with its inverse (e.g. the launcher's
    /// LEFT/RIGHT edges), so this is the natural inverse used for backtracking.
    pub fn opposite(self) -> Self {
        match self {
            Self::Left => Self::Right,
            Self::Right => Self::Left,
            Self::Up => Self::Down,
            Self::Down => Self::Up,
        }
    }

    /// True for `Left`/`Right`.
    pub fn is_horizontal(self) -> bool {
        matches!(self, Self::Left | Self::Right)
    }

    /// True for `Up`/`Down`.
    pub fn is_vertical(self) -> bool {
        matches!(self, Self::Up | Self::Down)
    }

    /// Signed index step along an ordered axis: `+1` for `Right`/`Down`,
    /// `-1` for `Left`/`Up`. Used by [`super::FocusController`] to apply the
    /// wrap/clamp boundary policy.
    pub fn step(self) -> isize {
        match self {
            Self::Right | Self::Down => 1,
            Self::Left | Self::Up => -1,
        }
    }

    /// Stable lowercase label, useful for diagnostics.
    pub fn label(self) -> &'static str {
        match self {
            Self::Left => "LEFT",
            Self::Right => "RIGHT",
            Self::Up => "UP",
            Self::Down => "DOWN",
        }
    }
}

impl fmt::Display for FocusDirection {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.label())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn opposite_is_an_involution() {
        for dir in FocusDirection::ALL {
            assert_eq!(dir.opposite().opposite(), dir);
            assert_ne!(dir.opposite(), dir);
        }
        assert_eq!(FocusDirection::Left.opposite(), FocusDirection::Right);
        assert_eq!(FocusDirection::Right.opposite(), FocusDirection::Left);
        assert_eq!(FocusDirection::Up.opposite(), FocusDirection::Down);
        assert_eq!(FocusDirection::Down.opposite(), FocusDirection::Up);
    }

    #[test]
    fn axes_and_steps() {
        assert!(FocusDirection::Left.is_horizontal());
        assert!(FocusDirection::Right.is_horizontal());
        assert!(FocusDirection::Up.is_vertical());
        assert!(FocusDirection::Down.is_vertical());
        assert_eq!(FocusDirection::Left.step(), -1);
        assert_eq!(FocusDirection::Up.step(), -1);
        assert_eq!(FocusDirection::Right.step(), 1);
        assert_eq!(FocusDirection::Down.step(), 1);
    }

    #[test]
    fn display_and_labels() {
        assert_eq!(FocusDirection::Left.to_string(), "LEFT");
        assert_eq!(FocusDirection::Down.label(), "DOWN");
        assert_eq!(FocusDirection::ALL.len(), 4);
    }
}
