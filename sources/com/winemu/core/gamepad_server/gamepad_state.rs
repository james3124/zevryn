//! Pure state model for the Wine-side virtual gamepad buffers.
//!
//! Mirrors `com.winemu.core.gamepad.GamepadState` and the field-level writes
//! performed by `GamepadInputRouter` / `VirtualGamepadController`. The Java
//! class writes into a native `ByteBuffer`; here the same fields are plain Rust
//! values so the model is dependency-free and testable.
//!
//! Buffer layout from `GamepadState.java` (native byte order):
//! | offset | width | field | writer |
//! |---|---|---|---|
//! | 0  | i16 | left stick X  | `e(short)` |
//! | 2  | i16 | left stick Y  | `f(short)` |
//! | 4  | i16 | right stick X | `i(short)` |
//! | 6  | i16 | right stick Y | `j(short)` |
//! | 8  | i16 | left trigger  | `g(short)` |
//! | 10 | i16 | right trigger | `k(short)` |
//! | 12..26 | u8 × 15 | buttons | `b(int, boolean)` |
//! | 27 | u8 | hat | `c(byte)` |
//!
//! `a()` (reset) writes sticks 0, triggers -32767, hat 0 and clears all 15
//! buttons. [`GamepadState::neutral`] reproduces exactly that.

/// The 15 button indices accepted by `GamepadState.b(int, boolean)`.
///
/// Names follow the key-code mapping in `GamepadInputRouter.z(int)`:
/// `96=A(0)`, `97=B(1)`, `99=X(2)`, `100=Y(3)`,
/// `4`/`109=Select(4)`, `110=Mode(5)`, `82`/`108=Start(6)`,
/// `106=ThumbL(7)`, `107=ThumbR(8)`, `102=L1(9)`, `103=R1(10)`,
/// `19..22=D-pad up/down/left/right (11..14)`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum GamepadButton {
    A = 0,
    B = 1,
    X = 2,
    Y = 3,
    Select = 4,
    Mode = 5,
    Start = 6,
    ThumbL = 7,
    ThumbR = 8,
    L1 = 9,
    R1 = 10,
    DpadUp = 11,
    DpadDown = 12,
    DpadLeft = 13,
    DpadRight = 14,
}

impl GamepadButton {
    pub const ALL: [Self; 15] = [
        Self::A,
        Self::B,
        Self::X,
        Self::Y,
        Self::Select,
        Self::Mode,
        Self::Start,
        Self::ThumbL,
        Self::ThumbR,
        Self::L1,
        Self::R1,
        Self::DpadUp,
        Self::DpadDown,
        Self::DpadLeft,
        Self::DpadRight,
    ];

    pub const fn index(self) -> usize {
        self as usize
    }

    pub fn from_index(index: usize) -> Option<Self> {
        Self::ALL.iter().copied().find(|b| b.index() == index)
    }

    /// Mirrors `GamepadInputRouter.z(int)`: Android `KeyEvent` code -> slot.
    /// Returns `None` for unmapped keys (the router returns -1 and ignores them).
    pub fn from_android_keycode(code: i32) -> Option<Self> {
        match code {
            96 => Some(Self::A),
            97 => Some(Self::B),
            99 => Some(Self::X),
            100 => Some(Self::Y),
            4 | 109 => Some(Self::Select),
            110 => Some(Self::Mode),
            82 | 108 => Some(Self::Start),
            106 => Some(Self::ThumbL),
            107 => Some(Self::ThumbR),
            102 => Some(Self::L1),
            103 => Some(Self::R1),
            19 => Some(Self::DpadUp),
            20 => Some(Self::DpadDown),
            21 => Some(Self::DpadLeft),
            22 => Some(Self::DpadRight),
            _ => None,
        }
    }

    pub const fn bit(self) -> u16 {
        1u16 << (self as u16)
    }
}

/// One-bit-per-button set, covering exactly the 15 bytes 12..=26.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Hash)]
pub struct GamepadButtons(pub u16);

impl GamepadButtons {
    pub const fn empty() -> Self {
        Self(0)
    }

    pub const fn from_button(button: GamepadButton) -> Self {
        Self(button.bit())
    }

    pub fn set(&mut self, button: GamepadButton, pressed: bool) {
        if pressed {
            self.0 |= button.bit();
        } else {
            self.0 &= !button.bit();
        }
    }

    pub fn set_index(&mut self, index: usize, pressed: bool) -> bool {
        match GamepadButton::from_index(index) {
            Some(button) => {
                self.set(button, pressed);
                true
            }
            None => false,
        }
    }

    pub fn contains(self, button: GamepadButton) -> bool {
        self.0 & button.bit() != 0
    }

    pub const fn is_empty(self) -> bool {
        self.0 == 0
    }

    pub const fn bits(self) -> u16 {
        self.0
    }

    pub fn iter(self) -> impl Iterator<Item = GamepadButton> {
        GamepadButton::ALL.iter().copied().filter(move |b| self.contains(*b))
    }

    pub const fn union(self, other: Self) -> Self {
        Self(self.0 | other.0)
    }

    pub const fn intersection(self, other: Self) -> Self {
        Self(self.0 & other.0)
    }

    pub const fn difference(self, other: Self) -> Self {
        Self(self.0 & !other.0)
    }
}

impl From<GamepadButton> for GamepadButtons {
    fn from(button: GamepadButton) -> Self {
        Self::from_button(button)
    }
}

/// Hat value written to byte 27 by `GamepadState.c(byte)`.
///
/// The reference only ever writes 0 (centered) at reset and drives the D-pad
/// through buttons 11..=14 using the ±0.5 axis thresholds in
/// `GamepadInputRouter.A()`. The bit encoding below is our own de-novo
/// representation so the hat can be carried as a value; [`Self::byte`] keeps
/// neutral == 0 as the reference does.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum GamepadHat {
    Neutral,
    Up,
    Down,
    Left,
    Right,
    UpLeft,
    UpRight,
    DownLeft,
    DownRight,
}

impl Default for GamepadHat {
    fn default() -> Self {
        Self::Neutral
    }
}

/// From `GamepadInputRouter.A()`: axis 15 (x): `< -0.5` = left, `> 0.5` =
/// right; axis 16 (y): `< -0.5` = up, `> 0.5` = down.
pub const DPAD_AXIS_THRESHOLD: f32 = 0.5;

impl GamepadHat {
    pub fn from_axes(x: f32, y: f32) -> Self {
        let left = x < -DPAD_AXIS_THRESHOLD;
        let right = x > DPAD_AXIS_THRESHOLD;
        let up = y < -DPAD_AXIS_THRESHOLD;
        let down = y > DPAD_AXIS_THRESHOLD;
        match (up, down, left, right) {
            (true, _, true, _) => Self::UpLeft,
            (true, _, _, true) => Self::UpRight,
            (_, true, true, _) => Self::DownLeft,
            (_, true, _, true) => Self::DownRight,
            (true, _, _, _) => Self::Up,
            (_, true, _, _) => Self::Down,
            (_, _, true, _) => Self::Left,
            (_, _, _, true) => Self::Right,
            _ => Self::Neutral,
        }
    }

    /// N=1, E=2, S=4, W=8; diagonals are the corresponding sum.
    /// Neutral is 0, matching the value written by `GamepadState.a()`.
    pub const fn byte(self) -> u8 {
        match self {
            Self::Neutral => 0,
            Self::Up => 1,
            Self::Right => 2,
            Self::Down => 4,
            Self::Left => 8,
            Self::UpLeft => 1 | 8,
            Self::UpRight => 1 | 2,
            Self::DownLeft => 4 | 8,
            Self::DownRight => 4 | 2,
        }
    }

    pub const fn is_neutral(self) -> bool {
        matches!(self, Self::Neutral)
    }

    /// The D-pad buttons implied by this hat position.
    pub fn dpad_buttons(self) -> GamepadButtons {
        let mut buttons = GamepadButtons::empty();
        // Vertical component.
        match self {
            Self::Up | Self::UpLeft | Self::UpRight => buttons.set(GamepadButton::DpadUp, true),
            Self::Down | Self::DownLeft | Self::DownRight => {
                buttons.set(GamepadButton::DpadDown, true)
            }
            _ => {}
        }
        // Horizontal component (separate match avoids overlapping patterns).
        match self {
            Self::Left | Self::UpLeft | Self::DownLeft => buttons.set(GamepadButton::DpadLeft, true),
            Self::Right | Self::UpRight | Self::DownRight => {
                buttons.set(GamepadButton::DpadRight, true)
            }
            _ => {}
        }
        buttons
    }
}

/// Axis scale used by `GamepadInputRouter.e` and
/// `VirtualGamepadController.g`: `(short) clamp(f * DNSRecordClass.CLASS_MASK,
/// -32768f, 32767f)` where `CLASS_MASK == 0xffff`.
pub const AXIS_SCALE: f32 = 65535.0;

/// Trigger resting value written by `GamepadState.a()` (`g((short) -32767)`).
pub const TRIGGER_NEUTRAL: i16 = -32767;

/// `(short) clamp(value * 0xffff, -32768, 32767)`.
pub fn axis_from_float(value: f32) -> i16 {
    (value * AXIS_SCALE).clamp(-32768.0, 32767.0) as i16
}

/// `GamepadInputRouter.f`: 0 -> -32767, 1 -> 32767, otherwise
/// `sqrt(value) * 65534 - 32767` (square-law trigger curve).
pub fn trigger_from_float(value: f32) -> i16 {
    if value <= 0.0 {
        TRIGGER_NEUTRAL
    } else if value >= 1.0 {
        i16::MAX
    } else {
        (value.sqrt() * 65534.0 - 32767.0) as i16
    }
}

/// `GamepadInputRouter.p`: discard tiny trigger noise and clamp to 0..=1.
pub fn normalized_trigger(value: f32) -> f32 {
    if value > 0.01 {
        value.clamp(0.0, 1.0)
    } else {
        0.0
    }
}

/// A complete gamepad snapshot, equivalent to the 28-byte native buffer.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct GamepadState {
    pub left_stick: (i16, i16),
    pub right_stick: (i16, i16),
    pub left_trigger: i16,
    pub right_trigger: i16,
    pub buttons: GamepadButtons,
    pub hat: GamepadHat,
}

impl Default for GamepadState {
    fn default() -> Self {
        Self::neutral()
    }
}

impl GamepadState {
    /// `GamepadState.a()`: sticks 0, triggers -32767, hat 0, no buttons.
    pub fn neutral() -> Self {
        Self {
            left_stick: (0, 0),
            right_stick: (0, 0),
            left_trigger: TRIGGER_NEUTRAL,
            right_trigger: TRIGGER_NEUTRAL,
            buttons: GamepadButtons::empty(),
            hat: GamepadHat::Neutral,
        }
    }

    pub fn reset(&mut self) {
        *self = Self::neutral();
    }

    pub fn set_button(&mut self, button: GamepadButton, pressed: bool) {
        self.buttons.set(button, pressed);
    }

    /// Returns `false` for an index outside 0..=14 (matches `b(int, boolean)`).
    pub fn set_button_index(&mut self, index: usize, pressed: bool) -> bool {
        self.buttons.set_index(index, pressed)
    }

    pub fn set_hat(&mut self, hat: GamepadHat) {
        const DPAD_MASK: u16 = GamepadButton::DpadUp.bit()
            | GamepadButton::DpadDown.bit()
            | GamepadButton::DpadLeft.bit()
            | GamepadButton::DpadRight.bit();
        self.hat = hat;
        self.buttons.0 = (self.buttons.0 & !DPAD_MASK) | hat.dpad_buttons().0;
    }

    /// Merge a partial update into this snapshot. Mirrors the way the router
    /// and virtual controller call individual setters on a shared state.
    pub fn apply(&mut self, patch: &GamepadStatePatch) {
        if patch.reset {
            self.reset();
        }
        if let Some(left) = patch.left_stick {
            self.left_stick = left;
        }
        if let Some(right) = patch.right_stick {
            self.right_stick = right;
        }
        if let Some(trigger) = patch.left_trigger {
            self.left_trigger = trigger;
        }
        if let Some(trigger) = patch.right_trigger {
            self.right_trigger = trigger;
        }
        if let Some(hat) = patch.hat {
            self.set_hat(hat);
        }
        // Only the bits flagged in `changed_buttons` are overwritten; all
        // other buttons keep their previous value.
        self.buttons.0 = (self.buttons.0 & !patch.changed_buttons.0)
            | (patch.pressed_buttons.0 & patch.changed_buttons.0);
    }

    pub fn merge(&mut self, patch: &GamepadStatePatch) {
        self.apply(patch);
    }
}

/// A field-level update, mirroring the writes a router callback performs on a
/// `GamepadState`. `None` fields mean "leave unchanged".
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct GamepadStatePatch {
    pub reset: bool,
    pub left_stick: Option<(i16, i16)>,
    pub right_stick: Option<(i16, i16)>,
    pub left_trigger: Option<i16>,
    pub right_trigger: Option<i16>,
    /// Buttons whose value this patch supplies.
    pub changed_buttons: GamepadButtons,
    /// New pressed state for the changed buttons.
    pub pressed_buttons: GamepadButtons,
    pub hat: Option<GamepadHat>,
}

impl GamepadStatePatch {
    /// Mirrors `GamepadInputRouter.u`: one button set from a key event.
    pub fn button(index: usize, pressed: bool) -> Option<Self> {
        GamepadButton::from_index(index).map(|b| Self {
            changed_buttons: GamepadButtons::from_button(b),
            pressed_buttons: if pressed {
                GamepadButtons::from_button(b)
            } else {
                GamepadButtons::empty()
            },
            ..Self::default()
        })
    }

    /// Mirrors `GamepadInputRouter.A`: full motion frame (sticks, triggers and
    /// D-pad buttons derived from the hat axes).
    pub fn motion(
        left: (i16, i16),
        right: (i16, i16),
        left_trigger: i16,
        right_trigger: i16,
        hat: GamepadHat,
    ) -> Self {
        let dpad = hat.dpad_buttons();
        Self {
            left_stick: Some(left),
            right_stick: Some(right),
            left_trigger: Some(left_trigger),
            right_trigger: Some(right_trigger),
            changed_buttons: dpad,
            pressed_buttons: dpad,
            hat: Some(hat),
            ..Self::default()
        }
    }

    /// `GamepadState.a()` as a patch (`VirtualGamepadController.j`).
    pub fn reset() -> Self {
        Self {
            reset: true,
            ..Self::default()
        }
    }

    pub fn with_button(mut self, button: GamepadButton, pressed: bool) -> Self {
        self.changed_buttons.set(button, true);
        self.pressed_buttons.set(button, pressed);
        self
    }

    pub fn with_left_trigger(mut self, value: i16) -> Self {
        self.left_trigger = Some(value);
        self
    }

    pub fn with_right_trigger(mut self, value: i16) -> Self {
        self.right_trigger = Some(value);
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn neutral_matches_java_reset() {
        let state = GamepadState::neutral();
        assert_eq!(state.left_stick, (0, 0));
        assert_eq!(state.right_stick, (0, 0));
        assert_eq!(state.left_trigger, -32767);
        assert_eq!(state.right_trigger, -32767);
        assert!(state.buttons.is_empty());
        assert_eq!(state.hat, GamepadHat::Neutral);
        assert_eq!(state.hat.byte(), 0);
    }

    #[test]
    fn button_bitmask_and_keycode_mapping() {
        let mut buttons = GamepadButtons::empty();
        buttons.set(GamepadButton::A, true);
        buttons.set(GamepadButton::Start, true);
        assert!(buttons.contains(GamepadButton::A));
        assert!(buttons.contains(GamepadButton::Start));
        assert!(!buttons.contains(GamepadButton::B));
        assert_eq!(buttons.iter().collect::<Vec<_>>(), vec![GamepadButton::A, GamepadButton::Start]);
        buttons.set(GamepadButton::A, false);
        assert!(!buttons.contains(GamepadButton::A));
        assert_eq!(GamepadButton::from_android_keycode(96), Some(GamepadButton::A));
        assert_eq!(GamepadButton::from_android_keycode(4), Some(GamepadButton::Select));
        assert_eq!(GamepadButton::from_android_keycode(108), Some(GamepadButton::Start));
        assert_eq!(GamepadButton::from_android_keycode(999), None);
    }

    #[test]
    fn partial_button_merge_preserves_untouched_buttons() {
        let mut state = GamepadState::neutral();
        state.apply(&GamepadStatePatch::button(0, true).unwrap()); // A down
        state.apply(&GamepadStatePatch::button(6, true).unwrap()); // Start down
        state.apply(&GamepadStatePatch::button(0, false).unwrap()); // A up
        assert!(!state.buttons.contains(GamepadButton::A));
        assert!(state.buttons.contains(GamepadButton::Start));
        assert!(GamepadStatePatch::button(15, true).is_none());
    }

    #[test]
    fn motion_patch_merges_axes_and_hat() {
        let mut state = GamepadState::neutral();
        state.apply(&GamepadStatePatch::button(0, true).unwrap());
        let patch = GamepadStatePatch::motion(
            (axis_from_float(0.5), axis_from_float(-0.5)),
            (0, 0),
            trigger_from_float(1.0),
            trigger_from_float(0.0),
            GamepadHat::from_axes(0.0, -1.0),
        );
        state.apply(&patch);
        assert_eq!(state.left_stick, (axis_from_float(0.5), axis_from_float(-0.5)));
        assert_eq!(state.right_stick, (0, 0));
        assert_eq!(state.left_trigger, 32767);
        assert_eq!(state.right_trigger, TRIGGER_NEUTRAL);
        assert_eq!(state.hat, GamepadHat::Up);
        assert!(state.buttons.contains(GamepadButton::DpadUp));
        assert!(!state.buttons.contains(GamepadButton::DpadDown));
        assert!(state.buttons.contains(GamepadButton::A)); // untouched by motion
    }

    #[test]
    fn reset_patch_clears_everything() {
        let mut state = GamepadState::neutral();
        state.apply(
            &GamepadStatePatch::motion((1, 2), (3, 4), 5, 6, GamepadHat::Right)
                .with_button(GamepadButton::A, true),
        );
        state.apply(&GamepadStatePatch::reset());
        assert_eq!(state, GamepadState::neutral());
    }

    #[test]
    fn conversion_helpers_match_reference_formulas() {
        assert_eq!(axis_from_float(0.0), 0);
        assert_eq!(axis_from_float(1.0), 32767);
        assert_eq!(axis_from_float(-2.0), -32768);
        assert_eq!(trigger_from_float(0.0), TRIGGER_NEUTRAL);
        assert_eq!(trigger_from_float(1.0), 32767);
        assert_eq!(trigger_from_float(-1.0), TRIGGER_NEUTRAL);
        assert_eq!(normalized_trigger(0.001), 0.0);
        assert_eq!(normalized_trigger(0.5), 0.5);
        assert_eq!(normalized_trigger(2.0), 1.0);
    }

    #[test]
    fn hat_axis_thresholds_and_dpad_bits() {
        assert_eq!(GamepadHat::from_axes(0.0, -0.6), GamepadHat::Up);
        assert_eq!(GamepadHat::from_axes(0.6, 0.0), GamepadHat::Right);
        assert_eq!(GamepadHat::from_axes(-0.5, 0.5), GamepadHat::Neutral);
        assert_eq!(GamepadHat::from_axes(-0.9, -0.9), GamepadHat::UpLeft);
        assert_eq!(GamepadHat::UpRight.byte(), 3);
        assert_eq!(
            GamepadHat::DownLeft.dpad_buttons().bits(),
            GamepadButton::DpadDown.bit() | GamepadButton::DpadLeft.bit()
        );
    }
}
