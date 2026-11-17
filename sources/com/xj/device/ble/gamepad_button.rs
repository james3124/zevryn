//! Gamepad button codes, modeled from the first-party XiaoJi/GameSir SDK.
//!
//! The `GcmKeyCode` / `HidKeyCode` constant classes in the decompiled sources
//! lost their `const val` initializers during JADX decompilation (malformed
//! Kotlin metadata), so the actual byte values are taken from the code that
//! converts between Android key codes and the on-wire GCM/HID byte codes:
//!
//! - `sources/com/xiaoji/sdk/utils/KeyCodeUtil.java` — `Companion.a(int)`
//!   (Android key code -> GCM byte) and `Companion.b(byte)` (GCM byte ->
//!   Android key code), plus `Companion.d(int)` for the human label.
//! - Verified against the literal immediates in
//!   `apktool/smali_classes4/com/xiaoji/sdk/utils/KeyCodeUtil$Companion.smali`
//!   (`method b(B)I`), which are not affected by JADX constant substitution.
//! - `sources/com/xj/devicesetting/gamepad/GamePadTestActivity.java#E1`
//!   independently confirms the Android key codes used by the gamepad test
//!   screen (A=96, B=97, X=99, Y=100, L1=102, R1=103, L2=104, R2=105,
//!   L3=106, R3=107, Start=108, Select=109, Home=3).
//!
//! Every numeric code below comes from those sources; none are invented. The
//! bit positions used by [`ButtonMask`] are our own (de novo) one-bit-per-button
//! assignment — the reference app has no such mask — but each bit maps to a
//! real, source-verified code.

/// A physical controller button with its verified protocol codes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum GamepadButton {
    DpadUp,
    DpadDown,
    DpadLeft,
    DpadRight,
    A,
    B,
    C,
    X,
    Y,
    Z,
    L1,
    R1,
    Select,
    Start,
    L2,
    R2,
    L3,
    R3,
    Home,
    B1,
    B2,
    B3,
    B4,
    B5,
    B6,
    B7,
    B8,
    L4,
    R4,
    L5,
    R5,
    Power,
    Mode,
    Turbo,
    Mouse,
    Screenshot,
    Macro,
}

impl GamepadButton {
    /// All buttons, in the order their codes appear in `KeyCodeUtil.b(byte)`.
    pub const ALL: [Self; 37] = [
        Self::DpadUp,
        Self::DpadDown,
        Self::DpadLeft,
        Self::DpadRight,
        Self::A,
        Self::B,
        Self::C,
        Self::X,
        Self::Y,
        Self::Z,
        Self::L1,
        Self::R1,
        Self::Select,
        Self::Start,
        Self::L2,
        Self::R2,
        Self::L3,
        Self::R3,
        Self::Home,
        Self::B1,
        Self::B2,
        Self::B3,
        Self::B4,
        Self::B5,
        Self::B6,
        Self::B7,
        Self::B8,
        Self::L4,
        Self::R4,
        Self::L5,
        Self::R5,
        Self::Power,
        Self::Mode,
        Self::Turbo,
        Self::Mouse,
        Self::Screenshot,
        Self::Macro,
    ];

    /// The GCM/HID on-wire byte code (`KeyCodeUtil` `a`/`b`, smali-verified).
    pub const fn gcm_code(self) -> i8 {
        match self {
            Self::DpadUp => -0x50,
            Self::DpadDown => -0x4f,
            Self::DpadLeft => -0x4e,
            Self::DpadRight => -0x4d,
            Self::A => -0x4c,
            Self::B => -0x4b,
            Self::C => -0x4a,
            Self::X => -0x49,
            Self::Y => -0x48,
            Self::Z => -0x47,
            Self::L1 => -0x46,
            Self::R1 => -0x45,
            Self::Select => -0x44,
            Self::Start => -0x43,
            Self::L2 => -0x42,
            Self::R2 => -0x41,
            Self::L3 => -0x40,
            Self::R3 => -0x3f,
            Self::Home => -0x3e,
            Self::B1 => -0x3d,
            Self::B2 => -0x3c,
            Self::B3 => -0x3b,
            Self::B4 => -0x3a,
            Self::B5 => -0x39,
            Self::B6 => -0x38,
            Self::B7 => -0x37,
            Self::B8 => -0x36,
            Self::L4 => -0x35,
            // Note: -0x34 (-52) is absent from KeyCodeUtil.b; -0x33 maps to 197.
            Self::R4 => -0x33,
            Self::L5 => -0x32,
            Self::R5 => -0x31,
            Self::Power => -0x30,
            Self::Mode => -0x2f,
            Self::Turbo => -0x2e,
            Self::Mouse => -0x2d,
            Self::Screenshot => -0x2c,
            Self::Macro => -0x01,
        }
    }

    /// The Android `KeyEvent` key code this button maps to (`KeyCodeUtil.d`).
    pub const fn android_keycode(self) -> i32 {
        match self {
            Self::DpadUp => 19,
            Self::DpadDown => 20,
            Self::DpadLeft => 21,
            Self::DpadRight => 22,
            Self::A => 96,
            Self::B => 97,
            Self::C => 98,
            Self::X => 99,
            Self::Y => 100,
            Self::Z => 101,
            Self::L1 => 102,
            Self::R1 => 103,
            Self::Select => 109,
            Self::Start => 108,
            Self::L2 => 104,
            Self::R2 => 105,
            Self::L3 => 106,
            Self::R3 => 107,
            Self::Home => 3,
            Self::B1 => 188,
            Self::B2 => 189,
            Self::B3 => 190,
            Self::B4 => 191,
            Self::B5 => 192,
            Self::B6 => 193,
            Self::B7 => 194,
            Self::B8 => 195,
            Self::L4 => 196,
            Self::R4 => 197,
            Self::L5 => 198,
            Self::R5 => 199,
            Self::Power => 200,
            Self::Mode => 201,
            Self::Turbo => 202,
            Self::Mouse => 203,
            Self::Screenshot => 120,
            Self::Macro => 255,
        }
    }

    /// The label from `KeyCodeUtil.d(int)` (Chinese labels kept verbatim).
    pub const fn label(self) -> &'static str {
        match self {
            Self::DpadUp => "↑",
            Self::DpadDown => "↓",
            Self::DpadLeft => "←",
            Self::DpadRight => "→",
            Self::A => "A",
            Self::B => "B",
            Self::C => "C",
            Self::X => "X",
            Self::Y => "Y",
            Self::Z => "Z",
            Self::L1 => "L1",
            Self::R1 => "R1",
            Self::Select => "Select",
            Self::Start => "Start",
            Self::L2 => "L2",
            Self::R2 => "R2",
            Self::L3 => "L3",
            Self::R3 => "R3",
            Self::Home => "Home",
            Self::B1 => "B1",
            Self::B2 => "B2",
            Self::B3 => "B3",
            Self::B4 => "B4",
            Self::B5 => "B5",
            Self::B6 => "B6",
            Self::B7 => "B7",
            Self::B8 => "B8",
            Self::L4 => "L4",
            Self::R4 => "R4",
            Self::L5 => "L5",
            Self::R5 => "R5",
            Self::Power => "Power",
            Self::Mode => "Mode",
            Self::Turbo => "Turbo",
            Self::Mouse => "Mouse",
            Self::Screenshot => "截图",
            Self::Macro => "宏",
        }
    }

    /// Look up a button by its GCM/HID byte code (mirrors `KeyCodeUtil.b`).
    pub fn from_gcm_code(code: u8) -> Option<Self> {
        Self::ALL.iter().copied().find(|b| b.gcm_code() as u8 == code)
    }

    /// Look up a button by Android key code.
    pub fn from_android_keycode(code: i32) -> Option<Self> {
        Self::ALL.iter().copied().find(|b| b.android_keycode() == code)
    }

    /// The single bit representing this button in a [`ButtonMask`].
    pub const fn bit(self) -> u64 {
        1u64 << (self as u64)
    }
}

/// A one-bit-per-button set, used to represent simultaneously pressed buttons.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Hash)]
pub struct ButtonMask(pub u64);

impl ButtonMask {
    pub const fn empty() -> Self {
        Self(0)
    }

    pub const fn from_button(button: GamepadButton) -> Self {
        Self(button.bit())
    }

    pub fn insert(&mut self, button: GamepadButton) {
        self.0 |= button.bit();
    }

    pub fn remove(&mut self, button: GamepadButton) {
        self.0 &= !button.bit();
    }

    pub fn contains(self, button: GamepadButton) -> bool {
        self.0 & button.bit() != 0
    }

    pub const fn is_empty(self) -> bool {
        self.0 == 0
    }

    pub const fn bits(self) -> u64 {
        self.0
    }

    /// Iterate over the buttons present in this mask, in declaration order.
    pub fn iter(self) -> impl Iterator<Item = GamepadButton> {
        GamepadButton::ALL.iter().copied().filter(move |b| self.contains(*b))
    }

    pub fn union(self, other: Self) -> Self {
        Self(self.0 | other.0)
    }

    pub fn intersection(self, other: Self) -> Self {
        Self(self.0 & other.0)
    }
}

impl From<GamepadButton> for ButtonMask {
    fn from(button: GamepadButton) -> Self {
        Self::from_button(button)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn all_codes_are_unique() {
        let mut codes = std::collections::HashSet::new();
        for button in GamepadButton::ALL {
            assert!(codes.insert(button.gcm_code()), "duplicate GCM code for {:?}", button);
        }
        assert_eq!(codes.len(), GamepadButton::ALL.len());
    }

    #[test]
    fn gcm_code_round_trips() {
        for button in GamepadButton::ALL {
            assert_eq!(GamepadButton::from_gcm_code(button.gcm_code() as u8), Some(button));
        }
    }

    #[test]
    fn android_keycode_round_trips() {
        for button in GamepadButton::ALL {
            assert_eq!(GamepadButton::from_android_keycode(button.android_keycode()), Some(button));
        }
    }

    #[test]
    fn source_verified_spot_checks() {
        // Values read directly from KeyCodeUtil$Companion.smali method b(B)I.
        assert_eq!(GamepadButton::A.gcm_code(), -0x4c);
        assert_eq!(GamepadButton::A.android_keycode(), 96);
        assert_eq!(GamepadButton::Home.gcm_code(), -0x3e);
        assert_eq!(GamepadButton::Select.gcm_code(), -0x44);
        assert_eq!(GamepadButton::Screenshot.gcm_code(), -0x2c);
        assert_eq!(GamepadButton::Macro.android_keycode(), 255);
        // -0x34 (-52) is not a known button.
        assert_eq!(GamepadButton::from_gcm_code((-0x34i8) as u8), None);
    }

    #[test]
    fn mask_insert_remove_contains() {
        let mut mask = ButtonMask::empty();
        mask.insert(GamepadButton::A);
        mask.insert(GamepadButton::Start);
        assert!(mask.contains(GamepadButton::A));
        assert!(mask.contains(GamepadButton::Start));
        assert!(!mask.contains(GamepadButton::B));
        assert_eq!(mask.iter().collect::<Vec<_>>(), vec![GamepadButton::A, GamepadButton::Start]);
        mask.remove(GamepadButton::A);
        assert!(!mask.contains(GamepadButton::A));
        assert!(mask.contains(GamepadButton::Start));
    }

    #[test]
    fn mask_set_operations() {
        let a = ButtonMask::from_button(GamepadButton::A);
        let b = ButtonMask::from_button(GamepadButton::B);
        assert_eq!((a.union(b)).bits(), a.bits() | b.bits());
        assert!(a.intersection(b).is_empty());
    }
}
