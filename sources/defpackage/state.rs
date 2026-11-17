//! Rust model of `defpackage.State` (GameHub Lite v5.1.8).
//!
//! The Java enum is a Kotlin `enum class` with a single `type: Int` payload.
//! Only the portable value mapping is modeled here; `@Keep`, `@Metadata`,
//! `EnumEntries` and the Kotlin metadata are JVM-only and left out.
//!
//! Kept name-for-name compatible with the core crate's `src/emu.rs`.

/// Component install state (`defpackage.State`).
///
/// The numeric codes are the exact `type` values from the reference enum:
/// `None=0, Downloaded=1, Extracted=2, NeedUpdate=4, INSTALLED=8`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ComponentState {
    None,       // 0
    Downloaded, // 1
    Extracted,  // 2
    NeedUpdate, // 4
    Installed,  // 8
}

impl ComponentState {
    /// Every state in declaration order (mirrors `State.values()`).
    pub const ALL: [Self; 5] = [
        Self::None,
        Self::Downloaded,
        Self::Extracted,
        Self::NeedUpdate,
        Self::Installed,
    ];

    /// Build a state from its wire/store code (`getType()` inverse).
    pub fn from_code(code: i32) -> Option<Self> {
        Some(match code {
            0 => Self::None,
            1 => Self::Downloaded,
            2 => Self::Extracted,
            4 => Self::NeedUpdate,
            8 => Self::Installed,
            _ => return None,
        })
    }

    /// The numeric code stored in the DB (`getType()`).
    pub fn code(self) -> i32 {
        match self {
            Self::None => 0,
            Self::Downloaded => 1,
            Self::Extracted => 2,
            Self::NeedUpdate => 4,
            Self::Installed => 8,
        }
    }

    /// True when the component is fully ready to use.
    pub fn is_installed(self) -> bool {
        self == Self::Installed
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn component_state_codes() {
        for code in [0, 1, 2, 4, 8] {
            assert_eq!(ComponentState::from_code(code).unwrap().code(), code);
        }
        assert_eq!(ComponentState::from_code(3), None);
        assert_eq!(ComponentState::from_code(-1), None);
    }

    #[test]
    fn all_states_round_trip() {
        assert_eq!(ComponentState::ALL.len(), 5);
        for state in ComponentState::ALL {
            assert_eq!(ComponentState::from_code(state.code()), Some(state));
        }
    }

    #[test]
    fn only_installed_is_ready() {
        assert!(ComponentState::Installed.is_installed());
        for state in [0, 1, 2, 4] {
            assert!(!ComponentState::from_code(state).unwrap().is_installed());
        }
    }
}
