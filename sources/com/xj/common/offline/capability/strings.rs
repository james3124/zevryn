//! Exact English resource strings from `res/values/strings.xml` that the app
//! uses to communicate offline state. Values are byte-for-byte from the
//! reference; resource names are preserved for traceability.

/// `pc_cc_steam_offline_mode` — "Offline Mode".
pub const STEAM_OFFLINE_MODE: &str = "Offline Mode";
/// `pc_cc_steam_offline_mode_desc` — the rationale behind the
/// `SteamOfflineMode` capability.
pub const STEAM_OFFLINE_MODE_DESC: &str =
    "Allow running verified Steam games without network connection";
/// `pcview_menu_header_offline` — section header for offline content.
pub const PCVIEW_HEADER_OFFLINE: &str = "Offline";
/// `comm_network_err` — generic network failure.
pub const COMM_NETWORK_ERR: &str = "Network error";
/// `no_network` — shown in game titles when connectivity is absent.
pub const NO_NETWORK: &str = "Network Error.";
/// `error_pc_offline` — cloud/remote play host unreachable.
pub const ERROR_PC_OFFLINE: &str = "Computer is offline";

/// (resource name, English value) pairs for the strings above.
pub const OFFLINE_STRINGS: [(&str, &str); 6] = [
    ("pc_cc_steam_offline_mode", STEAM_OFFLINE_MODE),
    ("pc_cc_steam_offline_mode_desc", STEAM_OFFLINE_MODE_DESC),
    ("pcview_menu_header_offline", PCVIEW_HEADER_OFFLINE),
    ("comm_network_err", COMM_NETWORK_ERR),
    ("no_network", NO_NETWORK),
    ("error_pc_offline", ERROR_PC_OFFLINE),
];
