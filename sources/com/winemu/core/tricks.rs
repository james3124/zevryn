//! Rust model of `com.winemu.core.Tricks` (GameHub Lite v5.1.8).
//!
//! A per-game-id lookup table: certain Steam game ids get config overrides
//! (`a(Config)`) or extra environment variables (`b(Config, EnvVars)`). All of
//! this is plain data + integer membership, so it ports dependency-free.
//! Applying the values to a real `Config`/`EnvVars` (Android objects) is out of
//! scope; the portable results are returned as plain values.

/// Config overrides produced by `Tricks.a(Config)`.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct ConfigOverrides {
    /// `config.O(false)` for ids in `f20260b`.
    pub disable_o: bool,
    /// `config.R(false)` for ids in `f20261c`.
    pub disable_r: bool,
}

pub struct Tricks;

impl Tricks {
    /// `f20260b` — ids that disable config option O.
    pub const DISABLE_O: [i32; 5] = [2630, 1060210, 414740, 201510, 1233880];
    /// `f20261c` — ids that disable config option R.
    pub const DISABLE_R: [i32; 2] = [1182900, 752590];
    /// `f20262d` — ids needing the OpenSSL capability workaround.
    pub const OPENSSL_CAP: [i32; 55] = [
        661920, 558260, 386360, 487720, 649950, 1150080, 463150, 420290, 1092660, 1052070,
        399810, 967390, 968870, 544920, 1055610, 682990, 551770, 392110, 447020, 834280,
        890880, 433530, 521890, 783170, 1195460, 1029890, 535850, 711750, 882020, 824280,
        775900, 415200, 748360, 1058450, 556180, 561600, 798290, 366870, 485030, 469610,
        759740, 1342260, 425670, 1096570, 492230, 996580, 437630, 442780, 433100, 406970,
        451520, 1237970, 1051200, 285190, 1133320,
    ];
    /// `f20263e` — ids that must not create a DXGI device manager.
    pub const NO_DXGI_MANAGER: [i32; 4] = [692890, 1017900, 1331440, 2620730];
    /// `f20264f` — ids that disable hardware scheduling.
    pub const NO_HW_SCHEDULING: [i32; 2] = [275850, 2012840];
    /// `f20265g` — ids that delay-free the Wine heap.
    pub const HEAP_DELAY_FREE: [i32; 7] = [202990, 212910, 499100, 1404090, 2052410, 789910, 1183470];
    /// `f20266h` — ids that zero the Wine heap.
    pub const HEAP_ZERO_MEMORY: [i32; 3] = [21980, 553850, 2055290];
    /// `f20267i` — ids that allocate the Wine heap top-down.
    pub const HEAP_TOP_DOWN: [i32; 2] = [71230, 3328910];

    /// `f20268j` — CEF file list override for a single id.
    pub const CEF_FILES_ID: i32 = 3350200;
    pub const CEF_FILES_VALUE: &'static str = "RevengeOnGoldDiggers.exe,情感反诈模拟器.exe";

    /// Mirrors `a(Config)`: compute config overrides for a game id.
    pub fn config_overrides(game_id: i32) -> ConfigOverrides {
        ConfigOverrides {
            disable_o: Self::DISABLE_O.contains(&game_id),
            disable_r: Self::DISABLE_R.contains(&game_id),
        }
    }

    /// Mirrors `b(Config, EnvVars)`: compute the ordered env assignments for a
    /// game id. Each entry maps to one `envVars.c(key, value)` call.
    pub fn env_tricks(game_id: i32) -> Vec<(&'static str, &'static str)> {
        let mut out = Vec::new();
        if Self::OPENSSL_CAP.contains(&game_id) {
            out.push(("OPENSSL_ia32cap", "~0x20000000"));
        }
        if Self::NO_DXGI_MANAGER.contains(&game_id) {
            out.push(("WINE_DO_NOT_CREATE_DXGI_DEVICE_MANAGER", "1"));
        }
        if Self::NO_HW_SCHEDULING.contains(&game_id) {
            out.push(("WINE_DISABLE_HARDWARE_SCHEDULING", "1"));
        }
        if Self::HEAP_DELAY_FREE.contains(&game_id) {
            out.push(("WINE_HEAP_DELAY_FREE", "1"));
        }
        if Self::HEAP_ZERO_MEMORY.contains(&game_id) {
            out.push(("WINE_HEAP_ZERO_MEMORY", "1"));
        }
        if Self::HEAP_TOP_DOWN.contains(&game_id) {
            out.push(("WINE_HEAP_TOP_DOWN", "1"));
        }
        if game_id == Self::CEF_FILES_ID {
            out.push(("WINEMU_CEF_FILES", Self::CEF_FILES_VALUE));
        }
        out
    }

    /// Convenience wrapper for the CEF override alone.
    pub fn cef_files(game_id: i32) -> Option<&'static str> {
        if game_id == Self::CEF_FILES_ID {
            Some(Self::CEF_FILES_VALUE)
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn config_overrides_match_lists() {
        assert_eq!(
            Tricks::config_overrides(2630),
            ConfigOverrides {
                disable_o: true,
                disable_r: false
            }
        );
        assert!(Tricks::config_overrides(752590).disable_r);
        assert_eq!(Tricks::config_overrides(12345), ConfigOverrides::default());
    }

    #[test]
    fn env_tricks_match_lists() {
        assert_eq!(
            Tricks::env_tricks(661920),
            vec![("OPENSSL_ia32cap", "~0x20000000")]
        );
        assert_eq!(
            Tricks::env_tricks(692890),
            vec![("WINE_DO_NOT_CREATE_DXGI_DEVICE_MANAGER", "1")]
        );
        assert_eq!(
            Tricks::env_tricks(21980),
            vec![("WINE_HEAP_ZERO_MEMORY", "1")]
        );
        assert!(Tricks::env_tricks(12345).is_empty());
    }

    #[test]
    fn cef_files_override() {
        assert_eq!(Tricks::cef_files(3350200), Some(Tricks::CEF_FILES_VALUE));
        assert_eq!(Tricks::cef_files(3350201), None);
        assert_eq!(
            Tricks::env_tricks(3350200),
            vec![("WINEMU_CEF_FILES", Tricks::CEF_FILES_VALUE)]
        );
    }
}
