//! Portable state for `GameDetailVM`.
//!
//! Modeled pieces:
//! * game-detail fetch state (`fetchGameDetailInfo` → `f30420b`);
//! * download-status refresh (`refreshDownloadStatusUI` → play-button state
//!   and the Steam-price side effect of the BuyNow branch);
//! * the Steam-vs-native start-type switch
//!   (`changeSteamPcGameStartUpParamsIfUseSteamVersion` → `useSteamVersion`
//!   and `currentOpenType` = 1407 when the Steam version is active).
//!
//! No Steam protocol, no repository I/O and no coroutine scope are modeled;
//! results are fed in through the `on_*` methods and pure resolvers.

use super::async_state::AsyncState;
use super::types::{GameDetail, PlayButtonState, StartType, SteamPrice, UserSteamGameState};

/// State machine for a single game-detail screen.
#[derive(Debug, Clone, Default)]
pub struct GameDetailViewModel {
    /// The fetched detail (`f30420b`) plus its fetch lifecycle.
    pub detail: AsyncState<GameDetail>,
    /// Resolved play-button/download state (`E()` / `f30422d`).
    pub download_status: AsyncState<PlayButtonState>,
    /// Steam price result of the BuyNow branch (`G()` / `f30423e`).
    pub steam_price: AsyncState<SteamPrice>,
    /// Set when `H()` detected BuyNow and a price query was queued.
    pub buy_now_query_pending: bool,
    /// Current open/start type, kept beside `detail` so it is usable even
    /// before a detail is loaded.
    pub current_start_type: StartType,
    /// Native start type restored when Steam-version mode is turned off.
    pub native_start_type: StartType,
}

impl GameDetailViewModel {
    pub fn new() -> Self {
        Self::default()
    }

    // --- detail fetch ----------------------------------------------------

    pub fn begin_fetch(&mut self) {
        self.detail.begin();
    }

    pub fn on_detail_success(&mut self, detail: GameDetail) {
        self.current_start_type = detail.start_type;
        self.native_start_type = detail.native_start_type;
        self.detail.succeed(detail);
    }

    pub fn on_detail_failure<S: Into<String>>(&mut self, message: S) {
        self.detail.fail(message);
    }

    pub fn detail(&self) -> Option<&GameDetail> {
        self.detail.value()
    }

    /// Mirrors `GameDetailVM.H`: true when the detail asks to buy the game.
    pub fn is_buy_now(&self) -> bool {
        self.detail.value().map(GameDetail::is_buy_now).unwrap_or(false)
    }

    /// Mirrors `GameDetailVM.J`: a Steam app id must parse to a positive int.
    pub fn is_valid_steam_app_id(appid: &str) -> bool {
        appid.parse::<i32>().map(|v| v > 0).unwrap_or(false)
    }

    // --- download-status refresh -----------------------------------------

    /// Portable resolver for `refreshDownloadStatusUI`'s play-button state.
    ///
    /// `installed` stands in for `AppLauncher.c(...)` / the decorators' Steam
    /// library checks; `config_allows_play` stands in for
    /// `LauncherConfig.b(currentOpenType)`.
    pub fn resolve_play_button_state(
        start_type: StartType,
        installed: bool,
        config_allows_play: bool,
    ) -> PlayButtonState {
        if start_type.needs_decorator() {
            // 1403 / 1407 go through PcDemoGameDecorator / SteamGameDecorator.
            if installed {
                PlayButtonState::Play
            } else {
                PlayButtonState::GetTheGame
            }
        } else if start_type.is_mobile_game() {
            if installed {
                PlayButtonState::Play
            } else {
                PlayButtonState::GetTheGame
            }
        } else if config_allows_play || start_type == StartType::PcLink {
            PlayButtonState::Play
        } else {
            PlayButtonState::GetTheGame
        }
    }

    /// Starts a download-status refresh.
    pub fn begin_download_refresh(&mut self) {
        self.download_status.begin();
    }

    /// Applies a resolved status to the detail and publishes it as `Success`.
    pub fn apply_download_status(&mut self, state: PlayButtonState) {
        if let Some(detail) = self.detail.value_mut() {
            detail.play_button_state = state;
        }
        self.download_status.succeed(state);
        // GameDetailVM.H -> GameDetailVM$refreshDownloadStatusUI$2
        self.buy_now_query_pending = state == PlayButtonState::BuyNow;
    }

    /// Refreshes download status from the current start type and the portable
    /// install/config flags.
    pub fn refresh_download_status(&mut self, installed: bool, config_allows_play: bool) {
        self.begin_download_refresh();
        let start_type = self.current_start_type;
        let state = Self::resolve_play_button_state(start_type, installed, config_allows_play);
        self.apply_download_status(state);
    }

    /// Result of the BuyNow Steam price query.
    pub fn on_steam_price_loaded(&mut self, price: SteamPrice) {
        self.steam_price.succeed(price);
        self.buy_now_query_pending = false;
    }

    pub fn on_steam_price_failed<S: Into<String>>(&mut self, message: S) {
        self.steam_price.fail(message);
        self.buy_now_query_pending = false;
    }

    // --- Steam-vs-native start-type switching ----------------------------

    /// Mirrors `GameDetailVM.r`:
    /// * `useSteamVersion` is true for Downloaded / ImportedByUser /
    ///   ImportedByTool / InDownloadQueue;
    /// * when true, the Steam start type (1407) is selected; otherwise the
    ///   recorded native start type is restored.
    pub fn apply_steam_game_state(&mut self, state: UserSteamGameState) {
        self.set_steam_version(state.means_use_steam_version());
    }

    /// Direct start-type choice (no Steam protocol involved): toggles between
    /// the native start type and the Steam-version start type.
    pub fn switch_start_type(&mut self, use_steam: bool) {
        self.set_steam_version(use_steam);
    }

    fn set_steam_version(&mut self, use_steam: bool) {
        let native = self.native_start_type;
        let target = if use_steam {
            StartType::SteamByPcEmulator
        } else {
            native
        };
        self.current_start_type = target;
        if let Some(detail) = self.detail.value_mut() {
            detail.use_steam_version = use_steam;
            detail.start_type = target;
        }
    }

    /// Records the start type of a freshly loaded detail as its native type.
    pub fn remember_native_start_type(&mut self, start_type: StartType) {
        self.native_start_type = start_type;
        if let Some(detail) = self.detail.value_mut() {
            detail.native_start_type = start_type;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn detail() -> GameDetail {
        GameDetail::new(57, "Doom", StartType::PcEmulator).with_steam_appid("570")
    }

    #[test]
    fn fetch_state_transitions() {
        let mut vm = GameDetailViewModel::new();
        assert!(vm.detail.is_idle());
        vm.begin_fetch();
        assert!(vm.detail.is_loading());
        assert!(vm.detail().is_none());
        assert!(!vm.is_buy_now());

        vm.on_detail_success(detail());
        let d = vm.detail().unwrap();
        assert_eq!(d.id, 57);
        assert_eq!(vm.current_start_type, StartType::PcEmulator);
        assert!(vm.detail.is_success());

        vm.on_detail_failure("404");
        assert_eq!(vm.detail.error(), Some("404"));
        assert!(!vm.is_buy_now());
    }

    #[test]
    fn steam_app_id_validation_matches_j() {
        assert!(GameDetailViewModel::is_valid_steam_app_id("570"));
        assert!(GameDetailViewModel::is_valid_steam_app_id("1"));
        assert!(!GameDetailViewModel::is_valid_steam_app_id("0"));
        assert!(!GameDetailViewModel::is_valid_steam_app_id("-3"));
        assert!(!GameDetailViewModel::is_valid_steam_app_id("abc"));
        assert!(!GameDetailViewModel::is_valid_steam_app_id(""));
    }

    #[test]
    fn play_button_resolver_covers_all_branches() {
        use PlayButtonState::*;
        assert_eq!(
            GameDetailViewModel::resolve_play_button_state(StartType::PcEmulator, true, false),
            Play
        );
        assert_eq!(
            GameDetailViewModel::resolve_play_button_state(StartType::SteamByPcEmulator, false, false),
            GetTheGame
        );
        assert_eq!(
            GameDetailViewModel::resolve_play_button_state(StartType::MobilePlay, true, false),
            Play
        );
        assert_eq!(
            GameDetailViewModel::resolve_play_button_state(StartType::MobilePlay, false, false),
            GetTheGame
        );
        assert_eq!(
            GameDetailViewModel::resolve_play_button_state(StartType::PcLink, false, false),
            Play
        );
        assert_eq!(
            GameDetailViewModel::resolve_play_button_state(StartType::Other(1234), false, true),
            Play
        );
        assert_eq!(
            GameDetailViewModel::resolve_play_button_state(StartType::Other(1234), false, false),
            GetTheGame
        );
    }

    #[test]
    fn download_refresh_updates_detail_and_buy_now_flag() {
        let mut vm = GameDetailViewModel::new();
        vm.on_detail_success(detail());
        vm.refresh_download_status(false, false);
        assert!(vm.download_status.is_success());
        assert_eq!(vm.download_status.value(), Some(&PlayButtonState::GetTheGame));
        assert_eq!(vm.detail().unwrap().play_button_state, PlayButtonState::GetTheGame);
        assert!(!vm.buy_now_query_pending);

        // BuyNow queues the Steam price query (H -> refreshDownloadStatusUI$2).
        vm.apply_download_status(PlayButtonState::BuyNow);
        assert!(vm.is_buy_now());
        assert!(vm.buy_now_query_pending);

        vm.on_steam_price_loaded(SteamPrice { app_id: 570, price: 1999, available: true });
        assert!(!vm.buy_now_query_pending);
        assert!(vm.steam_price.is_success());
        assert_eq!(vm.steam_price.value().unwrap().price, 1999);

        vm.apply_download_status(PlayButtonState::BuyNow);
        vm.on_steam_price_failed("offline");
        assert_eq!(vm.steam_price.error(), Some("offline"));
        assert!(!vm.buy_now_query_pending);
    }

    #[test]
    fn steam_state_switches_current_start_type() {
        let mut vm = GameDetailViewModel::new();
        vm.on_detail_success(detail());
        assert_eq!(vm.current_start_type, StartType::PcEmulator);

        vm.apply_steam_game_state(UserSteamGameState::Downloaded);
        assert!(vm.detail().unwrap().use_steam_version);
        assert_eq!(vm.current_start_type, StartType::SteamByPcEmulator);
        assert_eq!(vm.detail().unwrap().start_type, StartType::SteamByPcEmulator);

        // Imported games count as Steam versions too.
        vm.apply_steam_game_state(UserSteamGameState::ImportedByTool);
        assert!(vm.detail().unwrap().use_steam_version);

        // Not-downloaded restores the native start type.
        vm.apply_steam_game_state(UserSteamGameState::NotDownloaded);
        assert!(!vm.detail().unwrap().use_steam_version);
        assert_eq!(vm.current_start_type, StartType::PcEmulator);
        assert_eq!(vm.detail().unwrap().start_type, StartType::PcEmulator);
    }

    #[test]
    fn direct_start_type_switch_works_without_loaded_detail() {
        let mut vm = GameDetailViewModel::new();
        vm.remember_native_start_type(StartType::MobilePlay);
        vm.switch_start_type(true);
        assert_eq!(vm.current_start_type, StartType::SteamByPcEmulator);
        vm.switch_start_type(false);
        assert_eq!(vm.current_start_type, StartType::MobilePlay);
    }
}
