//! Core vocabulary for the offline-support model of GameHub Lite v5.1.8.
//!
//! Every capability is classified by where its data lives: on-device (Room
//! `GameSirUxDB` / `SteamDB`, the MMKV KV store, imported game files, the
//! Media3 `egg_media3_cache`, or the OkHttp/ForceCache disk caches) versus
//! requiring a live connection (the signed GameHub API, Steam WebAPI, cloud
//! streaming). See `REPORT.md` for exact reference paths and line numbers.

use super::policy::OfflinePolicy;

/// The three ways a launcher feature can depend on connectivity.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum OfflineLevel {
    /// Everything the feature needs is persisted on-device. No network and no
    /// cache freshness check are required; only the presence of local data is.
    Full,
    /// The feature has a usable on-device fallback (Room/MMKV/files/cache) but
    /// fresh metadata, sync, or verification needs a connection. It degrades
    /// gracefully and can serve stale data while revalidating.
    Partial,
    /// The feature is inherently remote. No amount of local data makes it work
    /// without a connection; there is no offline fallback.
    OnlineOnly,
}

impl OfflineLevel {
    /// Stable lowercase token used in logs/tests.
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Full => "full",
            Self::Partial => "partial",
            Self::OnlineOnly => "online_only",
        }
    }

    /// Whether a capability at this level has an on-device fallback.
    pub fn has_offline_fallback(self) -> bool {
        matches!(self, Self::Full | Self::Partial)
    }

    /// Every level, in declaration order.
    pub const ALL: [Self; 3] = [Self::Full, Self::Partial, Self::OnlineOnly];
}

/// Where a capability's backing data lives in the real application.
///
/// This is the *primary* source; a capability may consult several (for example
/// `GameDetailCached` reads the Room cache first and the OkHttp cache second).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DataSource {
    /// Room structured storage: `GameSirUxDB` (v5) and `SteamDB` (v2), plus the
    /// legacy `gameinfo.db`. Local-only.
    Room,
    /// MMKV / SharedPreferences KV store (`AppPreferences`,
    /// `_mmkv_app_global_preferences`, `gh-language`, `spUtils`). Local-only.
    Mmkv,
    /// Imported game files and emulator components on disk (`filePath`,
    /// `romPath`, `isAddFromLocal`, `.icp`/`.xj` profiles). Local-only.
    LocalFiles,
    /// Media3 `SimpleCache` at `egg_media3_cache`, 1000 MiB LRU, backed by the
    /// `EggExoplayerDatabaseProvider` SQLite index. Local-only.
    Media3Cache,
    /// OkHttp `Cache` (128 MB) and `com.drake.net` `ForceCache` backed by
    /// OkHttp `DiskLruCache`. Local copy of remote responses.
    OkHttpCache,
    /// Live request to `https://gamehub-lite-api.emuready.workers.dev/`, the
    /// Steam WebAPI, or a cloud/remote-play host. Always requires network.
    Network,
}

impl DataSource {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Room => "room",
            Self::Mmkv => "mmkv",
            Self::LocalFiles => "local_files",
            Self::Media3Cache => "media3_cache",
            Self::OkHttpCache => "okhttp_cache",
            Self::Network => "network",
        }
    }

    /// Whether this source can be read without connectivity.
    pub fn is_local(self) -> bool {
        !matches!(self, Self::Network)
    }

    pub const ALL: [Self; 6] = [
        Self::Room,
        Self::Mmkv,
        Self::LocalFiles,
        Self::Media3Cache,
        Self::OkHttpCache,
        Self::Network,
    ];
}

/// A launcher feature whose offline behaviour we model.
///
/// The first eight variants are the capabilities named by the rebuild scope;
/// `SteamOfflineMode` is added from the real offline-mode strings
/// (`pc_cc_steam_offline_mode`, `pc_cc_steam_offline_mode_desc`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum OfflineCapability {
    /// Showing/launching the user's imported game list. Backed by Room
    /// `t_game_library` / `StarterGame` and the imported files on disk.
    LocalGameLibrary,
    /// Adding a local game by importing a file / installed app. Disk only
    /// (the record is written to Room `t_game_library`, `isAddFromLocal=1`).
    ImportGame,
    /// Launching an already-installed Windows-EMU game. Local components
    /// (`winemu_file_op_record`, `win_emu_trans_config_table`) + `.icp` keymaps.
    LaunchPcEmu,
    /// Loading/editing controller mappings. `.icp`/`.xj` files, keymap rows in
    /// Room, and MMKV keys for input controls.
    GamepadMapping,
    /// Reading/writing settings. Entirely MMKV/SharedPreferences.
    Settings,
    /// Rendering a game's detail page from a previously persisted DTO.
    /// `@SerializedName` DTOs live in the OkHttp/ForceCache disk caches; fresh
    /// values need the signed API, hence stale-while-revalidate.
    GameDetailCached,
    /// Cloud / remote play (Limelight/Moonlight-derived). Needs a live host;
    /// `error_pc_offline` = "Computer is offline" when the host is unreachable.
    CloudPlay,
    /// Explore / free-games / news / social store feeds served by the signed
    /// GameHub API. No offline fallback.
    SocialStore,
    /// Running verified Steam games without a connection, per
    /// `pc_cc_steam_offline_mode_desc`. Requires prior online verification and
    /// cached SteamDB account/game rows, so it is only partially offline.
    SteamOfflineMode,
}

impl OfflineCapability {
    /// Stable lowercase token used in logs/tests.
    pub fn as_str(self) -> &'static str {
        match self {
            Self::LocalGameLibrary => "local_game_library",
            Self::ImportGame => "import_game",
            Self::LaunchPcEmu => "launch_pc_emu",
            Self::GamepadMapping => "gamepad_mapping",
            Self::Settings => "settings",
            Self::GameDetailCached => "game_detail_cached",
            Self::CloudPlay => "cloud_play",
            Self::SocialStore => "social_store",
            Self::SteamOfflineMode => "steam_offline_mode",
        }
    }

    /// Offline level from the default, reference-grounded policy.
    pub fn level(self) -> OfflineLevel {
        OfflinePolicy.level_for(self)
    }

    /// Primary backing data source from the default policy.
    pub fn data_source(self) -> DataSource {
        OfflinePolicy.data_source_for(self)
    }

    /// Short, human-readable justification for the level.
    pub fn rationale(self) -> &'static str {
        match self {
            Self::LocalGameLibrary => "Room t_game_library/StarterGame + imported files; no network",
            Self::ImportGame => "Local file import writes a Room row; no network required",
            Self::LaunchPcEmu => "Installed emulator components/keymaps are on-device; first download needs network",
            Self::GamepadMapping => ".icp/.xj profiles + Room keymaps + MMKV; no network",
            Self::Settings => "MMKV/SharedPreferences only; no network",
            Self::GameDetailCached => "Persisted @SerializedName DTO in OkHttp/ForceCache; fresh copy needs API",
            Self::CloudPlay => "Streaming host is remote; error_pc_offline when unreachable",
            Self::SocialStore => "Explore/store/news feeds come from the signed API",
            Self::SteamOfflineMode => "Verified Steam games need prior online verification + SteamDB cache",
        }
    }

    /// All nine capabilities, in declaration order.
    pub const ALL: [Self; 9] = [
        Self::LocalGameLibrary,
        Self::ImportGame,
        Self::LaunchPcEmu,
        Self::GamepadMapping,
        Self::Settings,
        Self::GameDetailCached,
        Self::CloudPlay,
        Self::SocialStore,
        Self::SteamOfflineMode,
    ];
}
