//! Plain Rust data models mirroring the Room `@Entity` classes of GameHub Lite
//! v5.1.8 (std only, no Room / Android / serde dependency).
//!
//! Two families of entities are reproduced here:
//!   * classes physically located in `sources/com/xj/common/data/table/`, and
//!   * the `@Entity` model classes in `sources/com/xj/common/data/model/` and
//!     `sources/com/xj/standalone/steam/...` that are registered with
//!     `GameSirUxDB` / `SteamDB` (see `GameSirUxDB_Impl` / `SteamDB_Impl`).
//!
//! Java `camelCase` field names become Rust `snake_case`. Java boxed/`@Nullable`
//! types become `Option<_>`; Java `boolean` maps to `bool`, `int` to `i32`,
//! `long` to `i64`, `double` to `f64`, `String` to `String`. Fields that Room
//! persists via a `TypeConverter` are kept as their serialized `TEXT` form
//! (`String`) so this module stays dependency-free. Fields explicitly marked
//! "not persisted" exist in the Java class but not in the generated table.

/// Epoch timestamp in milliseconds (Java `java.sql.Date` / `System.currentTimeMillis()`).
pub type TimestampMillis = i64;

// ---------------------------------------------------------------------------
// Entities in com.xj.common.data.table
// ---------------------------------------------------------------------------

/// `com.xj.common.data.table.GameLibraryTable` -> table `t_game_library`.
#[derive(Debug, Clone, PartialEq)]
pub struct GameLibraryTable {
    /// Java `_id`, PK, column `id`.
    pub id: i64,
    /// Java `userId`, column `user_id`.
    pub user_id: String,
    /// Java `launchType`, column `launch_type`.
    pub launch_type: i32,
    /// Java `packageName`, column `package_name`.
    pub package_name: String,
    /// Java `gameId` (`@Nullable Integer`), column `game_id`.
    pub game_id: Option<i32>,
    /// Java `localGameId` (`@Nullable`), column `local_game_id`.
    pub local_game_id: Option<String>,
    /// Java `localMobileAppId` (`@Nullable`), column `local_mobile_app_id`.
    pub local_mobile_app_id: Option<String>,
    /// Java `data`, column `data` (serialized game-library payload, TEXT).
    pub data: String,
    /// Java `time` (`@Nullable Date`), column `time`.
    pub time: Option<TimestampMillis>,
    /// Java `isLastLaunched`, column `is_last_launched`.
    pub is_last_launched: bool,
    /// Java `gameType`, column `game_type`.
    pub game_type: i32,
}

/// `com.xj.common.data.table.WinEmuFileOpRecord` -> table `winemu_file_op_record`.
#[derive(Debug, Clone, PartialEq)]
pub struct WinEmuFileOpRecord {
    /// PK, column `id`.
    pub id: i64,
    pub uid: i32,
    /// Java `fileId`, column `file_id`.
    pub file_id: i64,
    pub name: String,
    /// Java `fileType`, column `file_type`.
    pub file_type: i32,
    pub dir: String,
    /// Java `filePath`, column `file_path`.
    pub file_path: String,
    pub version: String,
    /// Java `versionCode`, column `version_code`.
    pub version_code: i32,
    pub status: i32,
    /// Java `inContainerName`, column `in_c_name`.
    pub in_container_name: String,
    /// Java `inContainerPath`, column `in_c_path`.
    pub in_container_path: String,
    /// Java `inWinePath`, column `in_wine_path`.
    pub in_wine_path: String,
    /// Java `componentType`, column `component_type`.
    pub component_type: i32,
    /// Java `winePathInSdk`, column `w_path_in_sdk`.
    pub wine_path_in_sdk: String,
    /// Java `wineVersionInSdk`, column `w_version_in_sdk`.
    pub wine_version_in_sdk: String,
    pub extend: String,
    pub time: i64,
}

/// `com.xj.common.data.table.WinEmuTranslationConfigEntity` ->
/// table `win_emu_trans_config_table`.
#[derive(Debug, Clone, PartialEq)]
pub struct WinEmuTranslationConfigEntity {
    pub uid: i32,
    /// PK.
    pub id: i32,
    pub name: String,
    /// Java `configId`, column `config_id`.
    pub config_id: String,
    /// Java `createTime`, column `create_time`.
    pub create_time: i64,
    /// Java `lastUpdateTime`, column `last_update_time`.
    pub last_update_time: i64,
    /// Java `gameId`, column `game_id`.
    pub game_id: String,
    pub config: String,
    /// Java `copyFromId`, column `copy_from_id`.
    pub copy_from_id: String,
    /// Java `copyFromName`, column `copy_from_name`.
    pub copy_from_name: String,
}

impl WinEmuTranslationConfigEntity {
    /// Companion `ID_PREFIX` used for locally created configs.
    pub const ID_PREFIX: &'static str = "local_";
}

/// `com.xj.common.data.table.ComponentInstallInContainer` (`@Keep`, not a Room
/// entity — a serialized value carried inside `extend`).
#[derive(Debug, Clone, PartialEq)]
pub struct ComponentInstallInContainer {
    pub wine_id: i64,
    pub wine_name: String,
    pub wine_version: String,
    pub wine_version_code: i32,
    pub status: i32,
}

/// `com.xj.common.data.table.ComponentInstallInContainerList` (`@Keep`).
#[derive(Debug, Clone, PartialEq)]
pub struct ComponentInstallInContainerList {
    pub list: Vec<ComponentInstallInContainer>,
}

/// `com.xj.common.data.table.DepComponentFileOpExt` (`@Keep`).
#[derive(Debug, Clone, PartialEq)]
pub struct DepComponentFileOpExt {
    pub game_id: String,
    pub container_name: String,
    pub status: i32,
}

/// `com.xj.common.data.table.DepComponentFileOpList` (`@Keep`).
#[derive(Debug, Clone, PartialEq)]
pub struct DepComponentFileOpList {
    pub list: Vec<DepComponentFileOpExt>,
}

// ---------------------------------------------------------------------------
// Room entities in com.xj.common.data.model (registered in GameSirUxDB)
// ---------------------------------------------------------------------------

/// `com.xj.common.data.model.StarterGame` -> table `StarterGame`.
#[derive(Debug, Clone, PartialEq)]
pub struct StarterGame {
    pub id: i32,
    pub game_id: i32,
    pub game_icon: String,
    pub game_bg_icon: String,
    pub game_name: String,
    pub game_desc: String,
    /// Java `List<GameTagEntity>`, stored as serialized TEXT via converter.
    pub game_tag: String,
    pub download_url: String,
    pub download_type: i32,
    pub file_size: f64,
    pub switch_id: String,
    pub emulator_id: i32,
    pub emulator_name: String,
    pub version: String,
    pub rom_path: String,
    pub file_path: String,
    pub has_downloaded: bool,
    pub emulator_kernel_id: i32,
    pub last_operating_time: i64,
    pub add_time: i64,
    pub is_add_from_local: bool,
}

/// `com.xj.common.data.model.EmulatorInfo` -> table `Emulator`.
#[derive(Debug, Clone, PartialEq)]
pub struct EmulatorInfo {
    pub id: i32,
    pub name: String,
    pub icon: String,
    /// Java `sIcon`, column `sicon`.
    pub sicon: String,
    pub start_mode: i32,
    pub agreement: String,
    /// Java `List<String> platform`, stored as serialized TEXT via converter.
    pub platform: String,
    pub download_url: String,
    pub package_name: String,
}

/// `com.xj.common.data.model.KernelInfo` -> table `Kernel`.
#[derive(Debug, Clone, PartialEq)]
pub struct KernelInfo {
    pub kernel_id: i32,
    pub title: String,
    pub emulator_name: String,
    pub emulator_id: i32,
    pub package_name: String,
    pub activity_name: String,
    pub download_url: String,
    /// Java `ver`.
    pub ver: String,
    pub file_md5: String,
    pub add_time: i64,
}

/// `com.xj.common.data.model.GameMainTagEntity` -> table `GameMainTag`.
#[derive(Debug, Clone, PartialEq)]
pub struct GameMainTagEntity {
    /// TEXT primary key.
    pub id: String,
    pub name: String,
    pub icon: String,
    /// Java `sIcon`, column `sIcon`.
    pub s_icon: String,
}

/// `com.xj.common.data.model.ScreenCaptureEntity` -> table `ScreenCaptures`.
#[derive(Debug, Clone, PartialEq)]
pub struct ScreenCaptureEntity {
    pub id: i64,
    /// Java `uriString`.
    pub uri_string: String,
    pub abs_path: String,
    pub is_show: bool,
    pub uid: i32,
}

/// `com.xj.common.data.model.ScreenRecordEntity` -> table `ScreenRecords`.
#[derive(Debug, Clone, PartialEq)]
pub struct ScreenRecordEntity {
    pub id: i64,
    /// Java `coverPath`.
    pub cover_path: String,
    pub abs_path: String,
    pub is_show: bool,
    pub width: i32,
    pub height: i32,
    pub duration: i64,
    pub ext: String,
    pub uid: i32,
}

/// `com.xj.common.data.model.SteamAccountTable` (GameSirUxDB variant) ->
/// table `steam_account`.
#[derive(Debug, Clone, PartialEq)]
pub struct SteamAccountTable {
    /// Java boxed `Long id` (nullable rowid alias).
    pub id: Option<i64>,
    pub account_name: String,
    pub refresh_token: String,
    pub access_token: String,
    pub new_guard_data: String,
    pub personal_name: String,
    pub avatar_url: String,
    pub is_current_user: bool,
    pub modify_time: i64,
}

/// `com.xj.common.data.model.SteamDownloadEntity` (GameSirUxDB variant) ->
/// table `steam_download_entity`.
#[derive(Debug, Clone, PartialEq)]
pub struct SteamDownloadEntity {
    pub id: i64,
    pub uid: i32,
    pub steam_account: String,
    pub steam_app_id: i64,
    pub name: String,
    pub icon_hash: String,
    pub status: i32,
    pub size: i64,
    pub download_size: i64,
    pub progress_size: i64,
    pub path: String,
    pub last_modify_time: i64,
    pub extend: String,
    /// Present in the Java class but **not** a persisted column.
    pub speed: i64,
}

// ---------------------------------------------------------------------------
// Room entities of the standalone SteamDB
// ---------------------------------------------------------------------------

/// `com.xj.standalone.steam.data.db.tables.SteamUserTable` ->
/// table `steam_account` (the SteamDB version of the account table).
#[derive(Debug, Clone, PartialEq)]
pub struct SteamUserTable {
    pub id: Option<i64>,
    pub steam_id: i64,
    pub account_name: String,
    pub refresh_token: String,
    pub access_token: String,
    pub new_guard_data: String,
    pub personal_name: String,
    pub avatar_url: String,
    pub is_current_user: bool,
    pub is_remember: bool,
    pub modify_time: i64,
    /// Java `Map`, stored as serialized TEXT via converter.
    pub extra_maps: String,
}

/// `com.xj.standalone.steam.data.db.tables.SteamUserGameTable` ->
/// table `t_steam_user_games`.
#[derive(Debug, Clone, PartialEq)]
pub struct SteamUserGameTable {
    pub id: Option<i64>,
    pub user_id: i64,
    /// Java `appId`, column `appid`.
    pub app_id: i32,
    pub name: String,
    pub last_update_time: i64,
    /// Java `Map`, stored as serialized TEXT via converter.
    pub extra_maps: String,
}

/// `com.xj.standalone.steam.data.bean.SteamModuleDownloadEntity` ->
/// table `steam_download_entity` (the SteamDB version).
#[derive(Debug, Clone, PartialEq)]
pub struct SteamModuleDownloadEntity {
    pub id: i64,
    pub uid: i32,
    pub steam_account: String,
    pub steam_app_id: i64,
    pub name: String,
    pub icon_hash: String,
    pub status: i32,
    pub size: i64,
    pub download_size: i64,
    pub progress_size: i64,
    pub path: String,
    pub last_modify_time: i64,
    pub extend: String,
    /// Present in the Java class but **not** a persisted column.
    pub speed: i64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn local_config_id_prefix_matches_java() {
        assert_eq!(WinEmuTranslationConfigEntity::ID_PREFIX, "local_");
    }

    #[test]
    fn nullable_fields_use_option() {
        let row = GameLibraryTable {
            id: 1,
            user_id: "u".into(),
            launch_type: 0,
            package_name: "p".into(),
            game_id: None,
            local_game_id: None,
            local_mobile_app_id: None,
            data: String::new(),
            time: None,
            is_last_launched: false,
            game_type: 0,
        };
        assert_eq!(row.game_id, None);
        assert_eq!(row.time, None);
        assert!(row.local_game_id.is_none());

        let account = SteamAccountTable {
            id: None,
            account_name: "a".into(),
            refresh_token: String::new(),
            access_token: String::new(),
            new_guard_data: String::new(),
            personal_name: String::new(),
            avatar_url: String::new(),
            is_current_user: false,
            modify_time: 0,
        };
        assert_eq!(account.id, None);
    }

    #[test]
    fn list_entities_hold_nested_values() {
        let wrapped = ComponentInstallInContainerList {
            list: vec![ComponentInstallInContainer {
                wine_id: 7,
                wine_name: "wine".into(),
                wine_version: "9.0".into(),
                wine_version_code: 1,
                status: 0,
            }],
        };
        assert_eq!(wrapped.list[0].wine_id, 7);

        let deps = DepComponentFileOpList {
            list: vec![DepComponentFileOpExt {
                game_id: "g".into(),
                container_name: "c".into(),
                status: 2,
            }],
        };
        assert_eq!(deps.list[0].status, 2);
    }
}
