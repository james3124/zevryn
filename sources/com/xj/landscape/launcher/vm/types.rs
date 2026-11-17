//! Self-contained value types referenced by the portable viewmodels.
//!
//! These are deliberately *not* the real `../data` / game-model DTOs; only the
//! fields the ViewModel state machines touch are represented, so this module
//! stays dependency-free (`std` only).

/// Play-button state (`com.xj.common.data.gameinfo.PlayButtonState`).
/// String resources are omitted; only the enum identity matters here.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PlayButtonState {
    InvalidGame,
    GetTheGame,
    BuyNow,
    SteamLogin,
    OneClickAdd2Lib,
    Play,
    PlayInSecond,
    InDownloadQueue,
    InUnzipState,
}

/// Start-type codes from `com.xj.common.data.launch.LauncherTypeEnum`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StartType {
    MobilePlay,         // 1302 (TYPE_MOBILE_PLAY)
    HidGame,            // 1301 (START_TYPE_HID_GAME)
    MobileInstallApp,   // 1502 (START_TYPE_MOBILE_INSTALL_APP)
    PcLink,             // 1402 (START_TYPE_PCLINK)
    PcEmulator,         // 1403 (START_TYPE_PC_EMULATOR)
    SteamByPcEmulator,  // 1407 (START_TYPE_STEAM_GAME_BY_PC_EMULATOR)
    MovingCloudGaming,  // 1406
    Other(i32),
}

impl Default for StartType {
    fn default() -> Self {
        StartType::Other(0)
    }
}

impl StartType {
    pub fn from_code(code: i32) -> Self {
        match code {
            1302 => StartType::MobilePlay,
            1301 => StartType::HidGame,
            1502 => StartType::MobileInstallApp,
            1402 => StartType::PcLink,
            1403 => StartType::PcEmulator,
            1407 => StartType::SteamByPcEmulator,
            1406 => StartType::MovingCloudGaming,
            other => StartType::Other(other),
        }
    }

    pub fn code(self) -> i32 {
        match self {
            StartType::MobilePlay => 1302,
            StartType::HidGame => 1301,
            StartType::MobileInstallApp => 1502,
            StartType::PcLink => 1402,
            StartType::PcEmulator => 1403,
            StartType::SteamByPcEmulator => 1407,
            StartType::MovingCloudGaming => 1406,
            StartType::Other(code) => code,
        }
    }

    /// `LauncherTypeEnum.isMobileGameStartType`: 1301, 1302, 1502, 10, 7.
    pub fn is_mobile_game(self) -> bool {
        matches!(self.code(), 1301 | 1302 | 1502 | 10 | 7)
    }

    /// `LauncherTypeEnum.isPcEmuGameLaunchType`: 1403 or 1407.
    pub fn is_pc_emu(self) -> bool {
        matches!(self.code(), 1403 | 1407)
    }

    /// The two start types that go through a decorator in
    /// `GameDetailVM.f30425g` (1403 / 1407).
    pub fn needs_decorator(self) -> bool {
        self.is_pc_emu()
    }

    /// The start type used when the Steam version is active.
    pub fn is_steam_version(self) -> bool {
        self == StartType::SteamByPcEmulator
    }
}

/// A game-circle entry (`getGameCircleList` items on the launcher home).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GameCircle {
    pub id: i32,
    pub name: String,
}

impl GameCircle {
    pub fn new(id: i32, name: impl Into<String>) -> Self {
        Self { id, name: name.into() }
    }
}

/// A launcher tab.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TabItem {
    pub id: i32,
    pub title: String,
}

impl TabItem {
    pub fn new(id: i32, title: impl Into<String>) -> Self {
        Self { id, title: title.into() }
    }
}

/// A connected device row (`DeviceSubMenuItemEntity` / device-list item).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConnectedDevice {
    pub id: String,
    pub name: String,
    pub connected: bool,
}

impl ConnectedDevice {
    pub fn new(id: impl Into<String>, name: impl Into<String>, connected: bool) -> Self {
        Self { id: id.into(), name: name.into(), connected }
    }
}

/// Minimal game-detail state touched by `GameDetailViewModel`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GameDetail {
    pub id: i32,
    pub name: String,
    pub steam_appid: Option<String>,
    /// Current open/start type (`currentOpenType`).
    pub start_type: StartType,
    /// Start type restored when Steam-version mode is turned off.
    pub native_start_type: StartType,
    pub play_button_state: PlayButtonState,
    pub use_steam_version: bool,
    /// Portable proxy for the "is installed / in library" checks performed by
    /// `AppLauncher`, `SteamGameManager` and `LauncherConfig` in the original.
    pub installed: bool,
}

impl GameDetail {
    pub fn new(id: i32, name: impl Into<String>, start_type: StartType) -> Self {
        Self {
            id,
            name: name.into(),
            steam_appid: None,
            start_type,
            native_start_type: start_type,
            play_button_state: PlayButtonState::GetTheGame,
            use_steam_version: false,
            installed: false,
        }
    }

    pub fn with_steam_appid(mut self, appid: impl Into<String>) -> Self {
        self.steam_appid = Some(appid.into());
        self
    }

    /// Mirrors `GameDetailVM.H`: the BuyNow branch triggers a Steam price query.
    pub fn is_buy_now(&self) -> bool {
        self.play_button_state == PlayButtonState::BuyNow
    }
}

/// User's Steam ownership state (`com.xj.game.entity.UserSteamGameState`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UserSteamGameState {
    Unknown,
    NotDownloaded,
    Downloaded,
    ImportedByUser,
    ImportedByTool,
    InDownloadQueue,
}

impl UserSteamGameState {
    /// Mirrors `GameDetailVM.r`: the states that make `useSteamVersion` true.
    pub fn means_use_steam_version(self) -> bool {
        matches!(
            self,
            UserSteamGameState::Downloaded
                | UserSteamGameState::ImportedByUser
                | UserSteamGameState::ImportedByTool
                | UserSteamGameState::InDownloadQueue
        )
    }
}

/// Minimal Steam price result (`SteamGamePriceEntity`).
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct SteamPrice {
    pub app_id: i32,
    /// Price in the smallest currency unit; 0 on a failed/empty lookup.
    pub price: i64,
    pub available: bool,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn start_type_codes_round_trip() {
        for code in [1301, 1302, 1402, 1403, 1406, 1407, 1502, 9999] {
            assert_eq!(StartType::from_code(code).code(), code);
        }
    }

    #[test]
    fn start_type_predicates_match_launcher_enum() {
        assert!(StartType::from_code(1301).is_mobile_game());
        assert!(StartType::from_code(1302).is_mobile_game());
        assert!(StartType::from_code(1502).is_mobile_game());
        assert!(StartType::from_code(10).is_mobile_game());
        assert!(StartType::from_code(7).is_mobile_game());
        assert!(!StartType::PcLink.is_mobile_game());

        assert!(StartType::PcEmulator.is_pc_emu());
        assert!(StartType::SteamByPcEmulator.is_pc_emu());
        assert!(StartType::SteamByPcEmulator.needs_decorator());
        assert!(!StartType::PcLink.needs_decorator());
        assert!(StartType::SteamByPcEmulator.is_steam_version());
    }

    #[test]
    fn steam_game_state_means_use_steam_version() {
        assert!(!UserSteamGameState::NotDownloaded.means_use_steam_version());
        assert!(!UserSteamGameState::Unknown.means_use_steam_version());
        assert!(UserSteamGameState::Downloaded.means_use_steam_version());
        assert!(UserSteamGameState::ImportedByUser.means_use_steam_version());
        assert!(UserSteamGameState::ImportedByTool.means_use_steam_version());
        assert!(UserSteamGameState::InDownloadQueue.means_use_steam_version());
    }

    #[test]
    fn game_detail_buy_now_predicate() {
        let mut d = GameDetail::new(1, "X", StartType::PcEmulator);
        assert!(!d.is_buy_now());
        d.play_button_state = PlayButtonState::BuyNow;
        assert!(d.is_buy_now());
        assert_eq!(d.native_start_type, StartType::PcEmulator);
    }
}
