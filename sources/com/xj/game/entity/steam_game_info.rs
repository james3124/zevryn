//! Minimal, dependency-free stand-in for
//! `com.xj.common.data.gameinfo.SteamGameInfo`, which `ImportedGameEntity`
//! references through its optional `steamGameInfo` field.
//!
//! Field names/types mirror the decompiled `SteamGameInfo.java` constructor
//! (`steamAppId`, `steamAppFolder`, `steamUserId`, `steamUserName`,
//! `gameLaunchArgs`). Gson/Parcelable behaviour is out of scope; the type only
//! needs to exist so `ImportedGame.steam_game_info` has a portable shape.

#[derive(Debug, Clone, PartialEq)]
pub struct SteamGameInfo {
    pub steam_app_id: i64,
    pub steam_app_folder: String,
    pub steam_user_id: String,
    pub steam_user_name: String,
    pub game_launch_args: Option<String>,
}

impl SteamGameInfo {
    pub fn new(steam_app_id: i64) -> Self {
        Self {
            steam_app_id,
            steam_app_folder: String::new(),
            steam_user_id: String::new(),
            steam_user_name: String::new(),
            game_launch_args: None,
        }
    }
}

impl Default for SteamGameInfo {
    fn default() -> Self {
        Self::new(0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_info() {
        let info = SteamGameInfo::default();
        assert_eq!(info.steam_app_id, 0);
        assert_eq!(info.game_launch_args, None);
    }
}
