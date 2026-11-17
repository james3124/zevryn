//! Pure launch-decision logic ported from the branching in the decompiled
//! `com.xj.landscape.launcher.ui.gamedetail.GameDetailActivity` (see
//! `REPORT.md`): `1406` -> cloud, `1403` -> local-PC-emu with an installed-game
//! check, `1407` -> Steam version with a download check, and
//! `isMobileGameStartType(start_type)` -> mobile launch.
//!
//! This module performs no I/O and contains no Android code: availability is
//! supplied by the caller as [`LocalAvailability`] flags.

use super::start_type::{GameStartCategory, GameStartType, code};

/// Local availability flags, each corresponding to a decompiled check.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct LocalAvailability {
    /// PC emulator runtime installed (`GHPcEmuLaunchStrategy`).
    pub emu_installed: bool,
    /// Local PC game present in library / game state
    /// (`GameLibraryRepository.a(localGameId)` / `GameStateMgr.a(id)` for
    /// start type `1403`).
    pub game_installed: bool,
    /// Mobile app installed / launchable (`i3(curStartupParams)` for mobile
    /// start types).
    pub mobile_app_installed: bool,
    /// Steam PC game downloaded (`ISteamGameDownloadService.b(appId)` for
    /// start type `1407`).
    pub steam_native_installed: bool,
    /// Cloud gaming session available (start types `1202` / `1406`).
    pub cloud_available: bool,
}

impl LocalAvailability {
    pub fn new() -> Self {
        Self::default()
    }
}

/// The portable outcome of a launch decision.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LaunchAction {
    /// Launch through the local PC emulator (decompiled `1403` / `1407`).
    LaunchEmu,
    /// Launch a mobile game (decompiled `1301` / `1302` / `1502`).
    LaunchMobile,
    /// Launch a cloud gaming session (decompiled `1202` / `1406`).
    LaunchCloud,
    /// Prompt the user to install the missing game/runtime before launching.
    PromptInstall,
    /// No portable launch path for this start type (e.g. `1001`, `1201`,
    /// `1401`, `1402`, `1501`, or an unknown code).
    Unsupported,
}

/// Decide what to do for a parsed [`GameStartType`] given local availability.
pub fn decide_launch(start_type: GameStartType, available: LocalAvailability) -> LaunchAction {
    match start_type.category() {
        GameStartCategory::LocalPcEmu => {
            if available.game_installed && available.emu_installed {
                LaunchAction::LaunchEmu
            } else {
                LaunchAction::PromptInstall
            }
        }
        GameStartCategory::SteamNative => {
            // `1407` is START_TYPE_STEAM_GAME_BY_PC_EMULATOR: it runs through
            // the PC-emulator strategy in the decompiled app.
            if available.steam_native_installed && available.emu_installed {
                LaunchAction::LaunchEmu
            } else {
                LaunchAction::PromptInstall
            }
        }
        GameStartCategory::Mobile => {
            if start_type == GameStartType::MobileInstall {
                // `1502` is the "mobile install app" flow.
                LaunchAction::PromptInstall
            } else if available.mobile_app_installed {
                LaunchAction::LaunchMobile
            } else {
                LaunchAction::PromptInstall
            }
        }
        GameStartCategory::Cloud => {
            if available.cloud_available {
                LaunchAction::LaunchCloud
            } else {
                LaunchAction::Unsupported
            }
        }
        GameStartCategory::Other => LaunchAction::Unsupported,
    }
}

/// Convenience wrapper: parse a raw `start_type` code, then decide.
/// Unknown codes are [`LaunchAction::Unsupported`] — no codes are invented.
pub fn decide_launch_for_code(code: i32, available: LocalAvailability) -> LaunchAction {
    match GameStartType::from_code(code) {
        Some(ty) => decide_launch(ty, available),
        None => LaunchAction::Unsupported,
    }
}

/// True when the decompiled activity would treat this start type as a
/// cloud launch (`start_type == 1406`). Kept public for parity checks.
pub fn is_cloud_start_code(code: i32) -> bool {
    code == code::START_TYPE_MOVING_CLOUD_GAMING
}

#[cfg(test)]
mod tests {
    use super::*;

    fn all_available() -> LocalAvailability {
        LocalAvailability {
            emu_installed: true,
            game_installed: true,
            mobile_app_installed: true,
            steam_native_installed: true,
            cloud_available: true,
        }
    }

    #[test]
    fn local_pc_emu_requires_game_and_emu() {
        assert_eq!(
            decide_launch(GameStartType::LocalPcEmu, all_available()),
            LaunchAction::LaunchEmu
        );
        let mut avail = all_available();
        avail.game_installed = false;
        assert_eq!(
            decide_launch(GameStartType::LocalPcEmu, avail),
            LaunchAction::PromptInstall
        );
        let mut avail = all_available();
        avail.emu_installed = false;
        assert_eq!(
            decide_launch(GameStartType::LocalPcEmu, avail),
            LaunchAction::PromptInstall
        );
    }

    #[test]
    fn steam_native_uses_pc_emu_strategy() {
        assert_eq!(
            decide_launch(GameStartType::SteamNative, all_available()),
            LaunchAction::LaunchEmu
        );
        let mut avail = all_available();
        avail.steam_native_installed = false;
        assert_eq!(
            decide_launch(GameStartType::SteamNative, avail),
            LaunchAction::PromptInstall
        );
        let mut avail = all_available();
        avail.emu_installed = false;
        assert_eq!(
            decide_launch(GameStartType::SteamNative, avail),
            LaunchAction::PromptInstall
        );
    }

    #[test]
    fn mobile_and_install_flows() {
        assert_eq!(
            decide_launch(GameStartType::Mobile, all_available()),
            LaunchAction::LaunchMobile
        );
        assert_eq!(
            decide_launch(GameStartType::HidGame, all_available()),
            LaunchAction::LaunchMobile
        );
        let mut avail = all_available();
        avail.mobile_app_installed = false;
        assert_eq!(
            decide_launch(GameStartType::Mobile, avail),
            LaunchAction::PromptInstall
        );
        // 1502 is the install-app flow regardless of availability.
        assert_eq!(
            decide_launch(GameStartType::MobileInstall, all_available()),
            LaunchAction::PromptInstall
        );
    }

    #[test]
    fn cloud_codes() {
        assert_eq!(
            decide_launch(GameStartType::Cloud, all_available()),
            LaunchAction::LaunchCloud
        );
        assert_eq!(
            decide_launch(GameStartType::XboxCloud, all_available()),
            LaunchAction::LaunchCloud
        );
        let mut avail = all_available();
        avail.cloud_available = false;
        assert_eq!(
            decide_launch(GameStartType::Cloud, avail),
            LaunchAction::Unsupported
        );
        assert!(is_cloud_start_code(1406));
        assert!(!is_cloud_start_code(1202));
    }

    #[test]
    fn unsupported_start_types_and_unknown_codes() {
        for ty in [
            GameStartType::LaunchOtherApp,
            GameStartType::OpenUrl,
            GameStartType::Pslink,
            GameStartType::Pclink,
            GameStartType::SteamWebDetail,
        ] {
            assert_eq!(decide_launch(ty, all_available()), LaunchAction::Unsupported);
        }
        assert_eq!(decide_launch_for_code(9999, all_available()), LaunchAction::Unsupported);
        assert_eq!(
            decide_launch_for_code(code::START_TYPE_PC_EMULATOR, all_available()),
            LaunchAction::LaunchEmu
        );
        assert_eq!(LocalAvailability::new(), LocalAvailability::default());
    }
}
