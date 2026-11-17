//! `FocusEvent` — the event payload that carries a [`FocusDirection`] through
//! the focus system (`com.xj.common.view.focus.focus.FocusEvent`).
//!
//! The Java class holds a `FocusDirection`, an Android `KeyEvent` and the
//! `KeyEvent` action code. `KeyEvent` is an Android framework type, so it is
//! replaced here by the portable [`KeyAction`] enum; the numeric codes and
//! string labels are kept exactly as in the decompiled `b(int)` helper.

use std::fmt;

use super::FocusDirection;

/// Portable equivalent of the `KeyEvent.ACTION_*` constants used by
/// `FocusEvent.f24734e/f24735f/f24736g`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum KeyAction {
    /// `KeyEvent.ACTION_DOWN` (code 0).
    Down = 0,
    /// `KeyEvent.ACTION_UP` (code 1).
    Up = 1,
    /// `KeyEvent.ACTION_MULTIPLE` (code 2).
    Multiple = 2,
}

impl KeyAction {
    /// Map a raw action code to a [`KeyAction`], or `None` for unknown codes.
    pub fn from_code(code: i32) -> Option<Self> {
        match code {
            0 => Some(Self::Down),
            1 => Some(Self::Up),
            2 => Some(Self::Multiple),
            _ => None,
        }
    }

    /// Raw action code (`ACTION_DOWN` = 0, etc.).
    pub fn code(self) -> i32 {
        self as i32
    }

    /// The label returned by Java's `FocusEvent.b(int)`.
    pub fn label(self) -> &'static str {
        match self {
            Self::Down => "ACTION_DOWN",
            Self::Up => "ACTION_UP",
            Self::Multiple => "ACTION_MULTIPLE",
        }
    }
}

impl fmt::Display for KeyAction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.label())
    }
}

/// A focus event: which direction was pressed and which key action it was.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FocusEvent {
    pub direction: FocusDirection,
    pub action: KeyAction,
}

impl FocusEvent {
    pub fn new(direction: FocusDirection, action: KeyAction) -> Self {
        Self { direction, action }
    }

    /// Convenience constructor for the common `ACTION_DOWN` case.
    pub fn down(direction: FocusDirection) -> Self {
        Self::new(direction, KeyAction::Down)
    }

    /// Whether this event represents a key press (focus moves on DOWN).
    pub fn is_down(self) -> bool {
        self.action == KeyAction::Down
    }
}

impl fmt::Display for FocusEvent {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "FocusEvent {{ action={}, direction={} }}",
            self.action, self.direction
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn action_codes_round_trip() {
        for action in [KeyAction::Down, KeyAction::Up, KeyAction::Multiple] {
            assert_eq!(KeyAction::from_code(action.code()), Some(action));
        }
        assert_eq!(KeyAction::Down.code(), 0);
        assert_eq!(KeyAction::Up.code(), 1);
        assert_eq!(KeyAction::Multiple.code(), 2);
        assert_eq!(KeyAction::from_code(99), None);
    }

    #[test]
    fn action_labels_match_java() {
        assert_eq!(KeyAction::Down.label(), "ACTION_DOWN");
        assert_eq!(KeyAction::Up.label(), "ACTION_UP");
        assert_eq!(KeyAction::Multiple.label(), "ACTION_MULTIPLE");
    }

    #[test]
    fn event_display_and_down_check() {
        let event = FocusEvent::down(FocusDirection::Left);
        assert!(event.is_down());
        assert_eq!(event.direction, FocusDirection::Left);
        assert_eq!(
            event.to_string(),
            "FocusEvent { action=ACTION_DOWN, direction=LEFT }"
        );
        assert!(!FocusEvent::new(FocusDirection::Right, KeyAction::Up).is_down());
    }
}
