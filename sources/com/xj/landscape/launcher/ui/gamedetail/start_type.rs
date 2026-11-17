//! Portable model of the `start_type` codes used by GameHub Lite v5.1.8.
//!
//! Every numeric value in this module is copied verbatim from the decompiled
//! `com.xj.common.data.launch.LauncherTypeEnum` (see `REPORT.md`). No code is
//! invented: unknown values map to [`None`] / [`GameStartCategory::Other`].
//!
//! Android/transport concerns (Activities, `AppLauncher`, Steam/cloud
//! protocols) are deliberately out of scope.

/// A single `start_type` value from `LauncherTypeEnum`.
///
/// The friendly name is the semantic category from the task description; the
/// doc comment on each variant gives the exact decompiled constant it maps to.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum GameStartType {
    /// `START_TYPE_LAUNCH_OTHER_APP = 1001`
    LaunchOtherApp,
    /// `START_TYPE_OPEN_URL = 1201`
    OpenUrl,
    /// `START_TYPE_XBOX_CLOUD_GAMING = 1202` (CLOUD)
    XboxCloud,
    /// `START_TYPE_HID_GAME = 1301` (MOBILE)
    HidGame,
    /// `START_TYPE_MOBILE_PLAY = 1302` (MOBILE)
    Mobile,
    /// `START_TYPE_PSLINK = 1401`
    Pslink,
    /// `START_TYPE_PCLINK = 1402`
    Pclink,
    /// `START_TYPE_PC_EMULATOR = 1403` (LOCAL_PC_EMU)
    LocalPcEmu,
    /// `START_TYPE_MOVING_CLOUD_GAMING = 1406` (CLOUD)
    Cloud,
    /// `START_TYPE_STEAM_GAME_BY_PC_EMULATOR = 1407` (STEAM_NATIVE, "Steam version")
    SteamNative,
    /// `START_TYPE_STEAM_GAME_DETAIL_BY_WEB = 1501`
    SteamWebDetail,
    /// `START_TYPE_MOBILE_INSTALL_APP = 1502` (MOBILE install flow)
    MobileInstall,
}

/// Exact `START_TYPE_*` constants from `LauncherTypeEnum.java`.
pub mod code {
    pub const START_TYPE_LAUNCH_OTHER_APP: i32 = 1001;
    pub const START_TYPE_OPEN_URL: i32 = 1201;
    pub const START_TYPE_XBOX_CLOUD_GAMING: i32 = 1202;
    pub const START_TYPE_HID_GAME: i32 = 1301;
    pub const START_TYPE_MOBILE_PLAY: i32 = 1302;
    pub const START_TYPE_PSLINK: i32 = 1401;
    pub const START_TYPE_PCLINK: i32 = 1402;
    pub const START_TYPE_PC_EMULATOR: i32 = 1403;
    pub const START_TYPE_MOVING_CLOUD_GAMING: i32 = 1406;
    pub const START_TYPE_STEAM_GAME_BY_PC_EMULATOR: i32 = 1407;
    pub const START_TYPE_STEAM_GAME_DETAIL_BY_WEB: i32 = 1501;
    pub const START_TYPE_MOBILE_INSTALL_APP: i32 = 1502;

    /// Values accepted by `LauncherTypeEnum.isMobileGameStartType` (it also
    /// accepts the raw game TYPE codes 10 and 7).
    pub const MOBILE_GAME_START_CODES: [i32; 5] = [1301, 1302, 1502, 10, 7];
    /// Values accepted by `LauncherTypeEnum.isPcEmuGameLaunchType` and
    /// `GameStartupParams.isPcEmuGame`.
    pub const PC_EMU_GAME_CODES: [i32; 2] = [1403, 1407];
}

/// Coarse semantic grouping requested by the task (`LOCAL_PC_EMU`, `MOBILE`,
/// `CLOUD`, `STEAM_NATIVE`, ...).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum GameStartCategory {
    LocalPcEmu,
    Mobile,
    Cloud,
    SteamNative,
    Other,
}

impl GameStartType {
    /// Map a raw `start_type` code to its type. Unknown codes yield `None`.
    pub fn from_code(code: i32) -> Option<Self> {
        match code {
            code::START_TYPE_LAUNCH_OTHER_APP => Some(Self::LaunchOtherApp),
            code::START_TYPE_OPEN_URL => Some(Self::OpenUrl),
            code::START_TYPE_XBOX_CLOUD_GAMING => Some(Self::XboxCloud),
            code::START_TYPE_HID_GAME => Some(Self::HidGame),
            code::START_TYPE_MOBILE_PLAY => Some(Self::Mobile),
            code::START_TYPE_PSLINK => Some(Self::Pslink),
            code::START_TYPE_PCLINK => Some(Self::Pclink),
            code::START_TYPE_PC_EMULATOR => Some(Self::LocalPcEmu),
            code::START_TYPE_MOVING_CLOUD_GAMING => Some(Self::Cloud),
            code::START_TYPE_STEAM_GAME_BY_PC_EMULATOR => Some(Self::SteamNative),
            code::START_TYPE_STEAM_GAME_DETAIL_BY_WEB => Some(Self::SteamWebDetail),
            code::START_TYPE_MOBILE_INSTALL_APP => Some(Self::MobileInstall),
            _ => None,
        }
    }

    /// Reverse mapping: the exact decompiled `start_type` code.
    pub fn code(self) -> i32 {
        match self {
            Self::LaunchOtherApp => code::START_TYPE_LAUNCH_OTHER_APP,
            Self::OpenUrl => code::START_TYPE_OPEN_URL,
            Self::XboxCloud => code::START_TYPE_XBOX_CLOUD_GAMING,
            Self::HidGame => code::START_TYPE_HID_GAME,
            Self::Mobile => code::START_TYPE_MOBILE_PLAY,
            Self::Pslink => code::START_TYPE_PSLINK,
            Self::Pclink => code::START_TYPE_PCLINK,
            Self::LocalPcEmu => code::START_TYPE_PC_EMULATOR,
            Self::Cloud => code::START_TYPE_MOVING_CLOUD_GAMING,
            Self::SteamNative => code::START_TYPE_STEAM_GAME_BY_PC_EMULATOR,
            Self::SteamWebDetail => code::START_TYPE_STEAM_GAME_DETAIL_BY_WEB,
            Self::MobileInstall => code::START_TYPE_MOBILE_INSTALL_APP,
        }
    }

    /// Semantic grouping for the launch decision layer.
    pub fn category(self) -> GameStartCategory {
        match self {
            Self::LocalPcEmu => GameStartCategory::LocalPcEmu,
            Self::HidGame | Self::Mobile | Self::MobileInstall => GameStartCategory::Mobile,
            Self::XboxCloud | Self::Cloud => GameStartCategory::Cloud,
            Self::SteamNative => GameStartCategory::SteamNative,
            Self::LaunchOtherApp | Self::OpenUrl | Self::Pslink | Self::Pclink | Self::SteamWebDetail => {
                GameStartCategory::Other
            }
        }
    }

    /// Mirrors `LauncherTypeEnum.isMobileGameStartType(code)`.
    pub fn is_mobile_game_start_type(self) -> bool {
        is_mobile_game_start_type(self.code())
    }

    /// Mirrors `LauncherTypeEnum.isPcEmuGameLaunchType(code)` /
    /// `GameStartupParams.isPcEmuGame()`.
    pub fn is_pc_emu_game(self) -> bool {
        is_pc_emu_game(self.code())
    }
}

/// Mirrors `LauncherTypeEnum.isMobileGameStartType(int)`:
/// `1301 || 1302 || 1502 || 10 || 7`.
pub fn is_mobile_game_start_type(code: i32) -> bool {
    code::MOBILE_GAME_START_CODES.contains(&code)
}

/// Mirrors `LauncherTypeEnum.isPcEmuGameLaunchType(int)` and
/// `GameStartupParams.isPcEmuGame()`: `1403 || 1407`.
pub fn is_pc_emu_game(code: i32) -> bool {
    code::PC_EMU_GAME_CODES.contains(&code)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn round_trips_every_decompiled_start_type_code() {
        let all = [
            (1001, GameStartType::LaunchOtherApp),
            (1201, GameStartType::OpenUrl),
            (1202, GameStartType::XboxCloud),
            (1301, GameStartType::HidGame),
            (1302, GameStartType::Mobile),
            (1401, GameStartType::Pslink),
            (1402, GameStartType::Pclink),
            (1403, GameStartType::LocalPcEmu),
            (1406, GameStartType::Cloud),
            (1407, GameStartType::SteamNative),
            (1501, GameStartType::SteamWebDetail),
            (1502, GameStartType::MobileInstall),
        ];
        for (raw, ty) in all {
            assert_eq!(GameStartType::from_code(raw), Some(ty));
            assert_eq!(ty.code(), raw);
        }
    }

    #[test]
    fn unknown_codes_are_not_invented() {
        assert_eq!(GameStartType::from_code(0), None);
        assert_eq!(GameStartType::from_code(1404), None);
        assert_eq!(GameStartType::from_code(-1), None);
    }

    #[test]
    fn categories_match_task_names() {
        assert_eq!(GameStartType::LocalPcEmu.category(), GameStartCategory::LocalPcEmu);
        assert_eq!(GameStartType::Mobile.category(), GameStartCategory::Mobile);
        assert_eq!(GameStartType::HidGame.category(), GameStartCategory::Mobile);
        assert_eq!(GameStartType::MobileInstall.category(), GameStartCategory::Mobile);
        assert_eq!(GameStartType::Cloud.category(), GameStartCategory::Cloud);
        assert_eq!(GameStartType::XboxCloud.category(), GameStartCategory::Cloud);
        assert_eq!(GameStartType::SteamNative.category(), GameStartCategory::SteamNative);
        assert_eq!(GameStartType::Pclink.category(), GameStartCategory::Other);
    }

    #[test]
    fn classification_mirrors_decompiled_helpers() {
        // isMobileGameStartType: 1301, 1302, 1502, 10, 7
        for c in [1301, 1302, 1502, 10, 7] {
            assert!(is_mobile_game_start_type(c));
        }
        assert!(!is_mobile_game_start_type(1403));
        assert!(!is_mobile_game_start_type(1202));

        // isPcEmuGameLaunchType / isPcEmuGame: 1403, 1407
        assert!(is_pc_emu_game(1403));
        assert!(is_pc_emu_game(1407));
        assert!(!is_pc_emu_game(1406));
        assert!(GameStartType::LocalPcEmu.is_pc_emu_game());
        assert!(GameStartType::SteamNative.is_pc_emu_game());
        assert!(GameStartType::Mobile.is_mobile_game_start_type());
    }
}
