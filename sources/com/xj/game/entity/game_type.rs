//! Game type codes used by `ImportedGameEntity.gameType` and stored in the
//! `game_type` column of `t_game_library`.
//!
//! Exact source (read-only reference):
//! `sources/com/xj/common/data/model/GameType.java`
//!
//! ```java
//! public enum GameType {
//!     DEFAULT(0),
//!     MOBILE_INSTALL_APP(1),
//!     PC_STEAM_IMPORT(2);
//! }
//! ```
//!
//! Only these three codes are invented here — no additional values are made up.

/// Portable mirror of `com.xj.common.data.model.GameType`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(i32)]
pub enum GameType {
    /// `DEFAULT(0)`
    Default = 0,
    /// `MOBILE_INSTALL_APP(1)`
    MobileInstallApp = 1,
    /// `PC_STEAM_IMPORT(2)`
    PcSteamImport = 2,
}

impl GameType {
    /// The default code used by `ImportedGameEntity`'s synthetic constructor
    /// (`GameType.DEFAULT.getValue()`).
    pub const DEFAULT_CODE: i32 = 0;

    /// The integer code stored in `ImportedGameEntity.gameType` /
    /// `GameLibraryTable.game_type`.
    pub fn code(self) -> i32 {
        self as i32
    }

    /// Parse a raw `game_type` code, rejecting anything not declared by the
    /// decompiled enum.
    pub fn from_code(code: i32) -> Option<Self> {
        match code {
            0 => Some(Self::Default),
            1 => Some(Self::MobileInstallApp),
            2 => Some(Self::PcSteamImport),
            _ => None,
        }
    }

    pub fn is_known(code: i32) -> bool {
        Self::from_code(code).is_some()
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Default => "DEFAULT",
            Self::MobileInstallApp => "MOBILE_INSTALL_APP",
            Self::PcSteamImport => "PC_STEAM_IMPORT",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn exact_codes_match_decompiled_enum() {
        assert_eq!(GameType::Default.code(), 0);
        assert_eq!(GameType::MobileInstallApp.code(), 1);
        assert_eq!(GameType::PcSteamImport.code(), 2);
        assert_eq!(GameType::DEFAULT_CODE, 0);
    }

    #[test]
    fn round_trips_and_rejects_unknown() {
        for t in [GameType::Default, GameType::MobileInstallApp, GameType::PcSteamImport] {
            assert_eq!(GameType::from_code(t.code()), Some(t));
            assert!(GameType::is_known(t.code()));
        }
        assert_eq!(GameType::from_code(-1), None);
        assert_eq!(GameType::from_code(3), None);
        assert!(!GameType::is_known(1537)); // the Kotlin default-arg bitmask, not a game type
    }

    #[test]
    fn names_match_java() {
        assert_eq!(GameType::Default.as_str(), "DEFAULT");
        assert_eq!(GameType::MobileInstallApp.as_str(), "MOBILE_INSTALL_APP");
        assert_eq!(GameType::PcSteamImport.as_str(), "PC_STEAM_IMPORT");
    }
}
