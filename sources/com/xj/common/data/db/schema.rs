//! Schema description and SQLite `CREATE TABLE` text generation for the
//! databases of GameHub Lite v5.1.8.
//!
//! Source of truth (read-only reference):
//!   * `GameSirUxDB_Impl.createAllTables()` — version 5, 11 entity tables.
//!   * `SteamDB_Impl.createAllTables()` — version 2, 3 entity tables.
//!   * `com.xj.mapping.db.DBHelper` — legacy `gameinfo.db`, version 7.
//!
//! No SQLite/driver dependency is used: this module only *emits SQL text*.
//! Executing that text requires a future `rusqlite`/`sqlx` dependency
//! (out of scope here — see REPORT.md).

use std::fmt::Write as _;

/// SQLite storage class used by Room for the entity columns.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SqlType {
    Integer,
    Text,
    Real,
}

impl SqlType {
    /// Room-style SQL spelling (uppercase).
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Integer => "INTEGER",
            Self::Text => "TEXT",
            Self::Real => "REAL",
        }
    }

    /// Spelling used by the legacy `DBHelper` `create table` statements.
    fn as_legacy_str(self) -> &'static str {
        match self {
            Self::Integer => "integer",
            Self::Text => "TEXT",
            Self::Real => "REAL",
        }
    }
}

/// SQL formatting flavour. Room emits quoted identifiers and `IF NOT EXISTS`;
/// the legacy `SQLiteOpenHelper` used unquoted identifiers and lower-case DDL.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SqlStyle {
    Room,
    Legacy,
}

/// One column of a table. `default` holds the *literal* SQL default as written
/// by Room (e.g. `''`, `0`, `-1`), or `None`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Column {
    pub name: &'static str,
    pub sql_type: SqlType,
    pub not_null: bool,
    /// Emit an inline `PRIMARY KEY` constraint on this column.
    pub primary_key: bool,
    /// Emit `AUTOINCREMENT` (Room only emits this together with a PK).
    pub auto_increment: bool,
    pub default: Option<&'static str>,
}

impl Column {
    pub fn new(name: &'static str, sql_type: SqlType) -> Self {
        Self {
            name,
            sql_type,
            not_null: false,
            primary_key: false,
            auto_increment: false,
            default: None,
        }
    }

    pub fn not_null(mut self) -> Self {
        self.not_null = true;
        self
    }

    pub fn primary_key(mut self) -> Self {
        self.primary_key = true;
        self
    }

    /// Marks the column as an auto-incrementing primary key.
    pub fn auto_increment(mut self) -> Self {
        self.primary_key = true;
        self.auto_increment = true;
        self
    }

    pub fn default(mut self, literal: &'static str) -> Self {
        self.default = Some(literal);
        self
    }
}

fn int(name: &'static str) -> Column {
    Column::new(name, SqlType::Integer)
}
fn text(name: &'static str) -> Column {
    Column::new(name, SqlType::Text)
}
fn real(name: &'static str) -> Column {
    Column::new(name, SqlType::Real)
}
/// The common Room rowid-alias primary key: `... PRIMARY KEY AUTOINCREMENT`.
fn auto_pk(name: &'static str) -> Column {
    int(name).auto_increment()
}

/// A table definition: its SQL name, the Java `@Entity` class it maps to, its
/// columns in declaration order, and an optional table-level `PRIMARY KEY(...)`
/// constraint (used by Room for non-auto-increment single-column keys).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Table {
    pub name: &'static str,
    pub entity: &'static str,
    pub columns: Vec<Column>,
    pub table_primary_key: Vec<&'static str>,
}

impl Table {
    pub fn new(name: &'static str, entity: &'static str, columns: Vec<Column>) -> Self {
        Self {
            name,
            entity,
            columns,
            table_primary_key: Vec::new(),
        }
    }

    pub fn with_primary_key(
        name: &'static str,
        entity: &'static str,
        columns: Vec<Column>,
        table_primary_key: Vec<&'static str>,
    ) -> Self {
        Self {
            name,
            entity,
            columns,
            table_primary_key,
        }
    }
}

/// A whole database schema and the SQL dialect flavour of its DDL.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Schema {
    pub name: &'static str,
    pub version: u32,
    pub style: SqlStyle,
    pub tables: Vec<Table>,
}

impl Schema {
    /// All table names, in declaration order.
    pub fn table_names(&self) -> Vec<&'static str> {
        self.tables.iter().map(|t| t.name).collect()
    }

    /// Render a single `CREATE TABLE` statement (no trailing semicolon).
    pub fn create_table_sql(&self, table: &Table) -> String {
        let mut out = String::new();
        match self.style {
            SqlStyle::Room => {
                let _ = write!(out, "CREATE TABLE IF NOT EXISTS `{}` (", table.name);
                for (i, col) in table.columns.iter().enumerate() {
                    if i > 0 {
                        out.push_str(", ");
                    }
                    self.write_column(&mut out, col);
                }
                if !table.table_primary_key.is_empty() {
                    out.push_str(", PRIMARY KEY(");
                    for (i, key) in table.table_primary_key.iter().enumerate() {
                        if i > 0 {
                            out.push_str(", ");
                        }
                        let _ = write!(out, "`{}`", key);
                    }
                    out.push(')');
                }
            }
            SqlStyle::Legacy => {
                let _ = write!(out, "create table {}(", table.name);
                for (i, col) in table.columns.iter().enumerate() {
                    if i > 0 {
                        out.push(',');
                    }
                    self.write_column(&mut out, col);
                }
                // The legacy DBHelper never used a table-level PK constraint.
            }
        }
        out.push(')');
        out
    }

    fn write_column(&self, out: &mut String, col: &Column) {
        let quote = self.style == SqlStyle::Room;
        let type_str = match self.style {
            SqlStyle::Room => col.sql_type.as_str(),
            SqlStyle::Legacy => col.sql_type.as_legacy_str(),
        };
        if quote {
            let _ = write!(out, "`{}` {}", col.name, type_str);
        } else {
            let _ = write!(out, "{} {}", col.name, type_str);
        }
        if col.primary_key {
            out.push_str(" PRIMARY KEY");
        }
        if col.auto_increment {
            out.push_str(" AUTOINCREMENT");
        }
        if col.not_null {
            out.push_str(" NOT NULL");
        }
        if let Some(default) = col.default {
            let _ = write!(out, " DEFAULT {}", default);
        }
    }

    /// Render every table's `CREATE TABLE`, joined by `;\n` with a trailing
    /// semicolon (a runnable script for a future SQLite driver).
    ///
    /// For Room schemas this also appends the internal `room_master_table`
    /// statement emitted by `createAllTables()` (it is not an `@Entity`, so it
    /// is not part of `tables`).
    pub fn create_all_sql(&self) -> String {
        let mut script = String::new();
        for table in &self.tables {
            script.push_str(&self.create_table_sql(table));
            script.push_str(";\n");
        }
        if self.style == SqlStyle::Room {
            script.push_str(ROOM_MASTER_TABLE_SQL);
            script.push_str(";\n");
        }
        script
    }

    /// Render the `DROP TABLE IF EXISTS` statements Room would issue.
    pub fn drop_all_sql(&self) -> String {
        let mut script = String::new();
        for table in &self.tables {
            if self.style == SqlStyle::Room {
                let _ = writeln!(script, "DROP TABLE IF EXISTS `{}`;", table.name);
            } else {
                let _ = writeln!(script, "DROP TABLE IF EXISTS {};", table.name);
            }
        }
        script
    }
}

// ---------------------------------------------------------------------------
// GameSirUxDB (version 5)
// ---------------------------------------------------------------------------

/// Room's internal bookkeeping table, emitted by every `createAllTables()`.
pub const ROOM_MASTER_TABLE_SQL: &str =
    "CREATE TABLE IF NOT EXISTS room_master_table (id INTEGER PRIMARY KEY,identity_hash TEXT)";

/// Room identity hash recorded by `GameSirUxDB_Impl` (for documentation).
pub const GAME_SIR_UX_DB_IDENTITY_HASH: &str = "d3c62afad9e9e48351ec41ab493a126e";

fn game_sir_ux_tables() -> Vec<Table> {
    vec![
        Table::new(
            "StarterGame",
            "com.xj.common.data.model.StarterGame",
            vec![
                auto_pk("id").not_null(),
                int("gameId").not_null(),
                text("gameIcon"),
                text("gameBgIcon"),
                text("gameName"),
                text("gameDesc"),
                text("gameTag"),
                text("downloadUrl"),
                int("downloadType").not_null(),
                real("fileSize").not_null(),
                text("switchId"),
                int("emulatorId").not_null(),
                text("emulatorName"),
                text("version"),
                text("romPath"),
                text("filePath"),
                int("hasDownloaded").not_null(),
                int("emulatorKernelId").not_null(),
                int("lastOperatingTime").not_null(),
                int("addTime").not_null(),
                int("isAddFromLocal").not_null(),
            ],
        ),
        Table::with_primary_key(
            "Emulator",
            "com.xj.common.data.model.EmulatorInfo",
            vec![
                int("id").not_null(),
                text("name"),
                text("icon"),
                text("sicon"),
                int("startMode").not_null(),
                text("agreement"),
                text("platform"),
                text("downloadUrl"),
                text("packageName"),
            ],
            vec!["id"],
        ),
        Table::with_primary_key(
            "Kernel",
            "com.xj.common.data.model.KernelInfo",
            vec![
                int("kernelId").not_null(),
                text("title"),
                text("emulatorName"),
                int("emulatorId").not_null(),
                text("packageName"),
                text("activityName"),
                text("downloadUrl"),
                text("ver"),
                text("fileMd5"),
                int("addTime").not_null(),
            ],
            vec!["kernelId"],
        ),
        Table::with_primary_key(
            "GameMainTag",
            "com.xj.common.data.model.GameMainTagEntity",
            vec![
                text("id").not_null(),
                text("name"),
                text("icon"),
                text("sIcon"),
            ],
            vec!["id"],
        ),
        Table::new(
            "ScreenCaptures",
            "com.xj.common.data.model.ScreenCaptureEntity",
            vec![
                auto_pk("id").not_null(),
                text("uriString").not_null(),
                text("absPath"),
                int("isShow").not_null(),
                int("uid").not_null(),
            ],
        ),
        Table::new(
            "ScreenRecords",
            "com.xj.common.data.model.ScreenRecordEntity",
            vec![
                auto_pk("id").not_null(),
                text("coverPath"),
                text("absPath").not_null(),
                int("isShow").not_null(),
                int("width").not_null(),
                int("height").not_null(),
                int("duration").not_null(),
                text("ext"),
                int("uid").not_null(),
            ],
        ),
        Table::new(
            "t_game_library",
            "com.xj.common.data.table.GameLibraryTable",
            vec![
                auto_pk("id").not_null(),
                text("user_id").not_null().default("''"),
                int("launch_type").not_null(),
                text("package_name").not_null().default("''"),
                int("game_id").default("0"),
                text("local_game_id").default("''"),
                text("local_mobile_app_id").default("''"),
                text("data").not_null().default("''"),
                int("time"),
                int("is_last_launched").not_null(),
                int("game_type").not_null().default("0"),
            ],
        ),
        Table::new(
            "winemu_file_op_record",
            "com.xj.common.data.table.WinEmuFileOpRecord",
            vec![
                auto_pk("id").not_null(),
                int("uid").not_null(),
                int("file_id").not_null(),
                text("name").not_null().default("''"),
                int("file_type").not_null(),
                text("dir").not_null(),
                text("file_path").not_null(),
                text("version").not_null(),
                int("version_code").not_null(),
                int("status").not_null(),
                text("in_c_name").not_null(),
                text("in_c_path").not_null(),
                text("in_wine_path").not_null(),
                int("component_type").not_null(),
                text("w_path_in_sdk").not_null(),
                text("w_version_in_sdk").not_null(),
                text("extend").not_null(),
                int("time").not_null(),
            ],
        ),
        Table::new(
            "steam_account",
            "com.xj.common.data.model.SteamAccountTable",
            vec![
                auto_pk("id"),
                text("account_name").not_null().default("''"),
                text("refresh_token").default("''"),
                text("access_token").default("''"),
                text("new_guard_data").default("''"),
                text("personal_name").default("''"),
                text("avatar_url").default("''"),
                int("is_current_user").not_null().default("0"),
                int("modify_time").not_null(),
            ],
        ),
        Table::new(
            "steam_download_entity",
            "com.xj.common.data.model.SteamDownloadEntity",
            vec![
                auto_pk("id").not_null(),
                int("uid").not_null(),
                text("steam_account").not_null(),
                int("steam_appid").not_null(),
                text("name").not_null(),
                text("icon_hash").not_null(),
                int("status").not_null(),
                int("size").not_null(),
                int("download_size").not_null(),
                int("progress_size").not_null(),
                text("path").not_null(),
                int("last_modify_time").not_null(),
                text("extend").not_null(),
            ],
        ),
        Table::new(
            "win_emu_trans_config_table",
            "com.xj.common.data.table.WinEmuTranslationConfigEntity",
            vec![
                int("uid").not_null(),
                auto_pk("id").not_null(),
                text("name").not_null().default("''"),
                text("config_id").not_null().default("''"),
                int("create_time").not_null(),
                int("last_update_time").not_null(),
                text("game_id").not_null().default("''"),
                text("config").not_null(),
                text("copy_from_id").not_null().default("''"),
                text("copy_from_name").not_null().default("''"),
            ],
        ),
    ]
}

/// The `GameSirUxDB` schema (11 entity tables, version 5).
pub fn game_sir_ux_db() -> Schema {
    Schema {
        name: "GameSirUxDB",
        version: 5,
        style: SqlStyle::Room,
        tables: game_sir_ux_tables(),
    }
}

// ---------------------------------------------------------------------------
// SteamDB (version 2)
// ---------------------------------------------------------------------------

/// Room identity hash recorded by `SteamDB_Impl` (for documentation).
pub const STEAM_DB_IDENTITY_HASH: &str = "44e473a979c13eb6fed9ffdba9a2dc64";

fn steam_db_tables() -> Vec<Table> {
    vec![
        Table::new(
            "steam_account",
            "com.xj.standalone.steam.data.db.tables.SteamUserTable",
            vec![
                auto_pk("id"),
                int("steam_id").not_null().default("0"),
                text("account_name").not_null().default("''"),
                text("refresh_token").default("''"),
                text("access_token").default("''"),
                text("new_guard_data").default("''"),
                text("personal_name").default("''"),
                text("avatar_url").default("''"),
                int("is_current_user").not_null().default("0"),
                int("is_remember").not_null().default("0"),
                int("modify_time").not_null(),
                text("extra_maps").not_null().default("''"),
            ],
        ),
        Table::new(
            "t_steam_user_games",
            "com.xj.standalone.steam.data.db.tables.SteamUserGameTable",
            vec![
                auto_pk("id"),
                int("user_id").not_null().default("0"),
                int("appid").not_null().default("0"),
                text("name").not_null().default("''"),
                int("last_update_time").not_null(),
                text("extra_maps").not_null().default("''"),
            ],
        ),
        Table::new(
            "steam_download_entity",
            "com.xj.standalone.steam.data.bean.SteamModuleDownloadEntity",
            vec![
                auto_pk("id").not_null(),
                int("uid").not_null(),
                text("steam_account").not_null(),
                int("steam_appid").not_null(),
                text("name").not_null(),
                text("icon_hash").not_null(),
                int("status").not_null(),
                int("size").not_null(),
                int("download_size").not_null(),
                int("progress_size").not_null(),
                text("path").not_null(),
                int("last_modify_time").not_null(),
                text("extend").not_null(),
            ],
        ),
    ]
}

/// The standalone `SteamDB` schema (3 entity tables, version 2).
pub fn steam_db() -> Schema {
    Schema {
        name: "SteamDB",
        version: 2,
        style: SqlStyle::Room,
        tables: steam_db_tables(),
    }
}

// ---------------------------------------------------------------------------
// Legacy DBHelper (gameinfo.db, version 7) — com.xj.mapping.db.DBHelper
// ---------------------------------------------------------------------------

/// `ALTER TABLE` migrations applied by `DBHelper.b()` to the legacy `mygame`
/// table across versions 2/4/5/6. Kept verbatim from the decompiled source.
pub const LEGACY_MYGAME_ALTERS: &[&str] = &[
    "ALTER TABLE mygame ADD COLUMN is_pk INTEGER NOT NULL DEFAULT -1",
    "ALTER TABLE mygame ADD COLUMN is_ol INTEGER NOT NULL DEFAULT -1",
    "ALTER TABLE mygame ADD COLUMN is_handle INTEGER NOT NULL DEFAULT -1",
    "ALTER TABLE mygame ADD COLUMN is_vr INTEGER NOT NULL DEFAULT -1",
    "ALTER TABLE mygame ADD COLUMN max INTEGER NOT NULL DEFAULT -1",
];

fn legacy_game_info_tables() -> Vec<Table> {
    let mut gameinfo = vec![auto_pk("_id")];
    gameinfo.extend(
        [
            "bad",
            "categoryid",
            "categoryshortname",
            "description",
            "downloads",
            "emulatorid",
            "emulatorshortname",
            "fee",
            "gameid",
            "gamename",
            "good",
            "icon",
            "language",
            "orgname",
            "package_name",
            "playurl",
            "rating",
            "size",
            "start_sell_time",
            "storeurl",
            "uid",
            "updatedtime",
            "username",
            "version",
            "screens",
        ]
        .iter()
        .map(|name| text(name)),
    );

    // Columns of `mygame` as created at version 0. The five INTEGER columns
    // added by migrations are listed in `LEGACY_MYGAME_ALTERS` instead.
    let mygame = vec![
        auto_pk("_id"),
        text("gameid"),
        text("icon"),
        text("size"),
        text("package_name"),
        text("gamename"),
        text("emulatorType"),
        text("description"),
        text("fileName"),
        text("filePath"),
        text("downloadId"),
        text("playtime"),
        text("isplay"),
    ];

    let mydownloadconfig = vec![
        auto_pk("_id"),
        text("vssid"),
        text("md5"),
        text("content"),
        text("pngPath"),
        text("pkg"),
        text("brand"),
        text("mobile_model"),
        text("resolution"),
        text("description"),
        text("create_time"),
    ];

    let video_thumb = vec![auto_pk("_id"), text("gameid"), text("videopath")];

    let mycheat = vec![
        auto_pk("_id"),
        text("md5"),
        text("cheat_name"),
        text("cheat_content"),
        text("gameid"),
        text("download"),
        text("share_time"),
    ];

    vec![
        Table::new("gameinfo", "gameinfo.db (legacy)", gameinfo),
        Table::new("mygame", "gameinfo.db (legacy)", mygame),
        Table::new("mydownloadconfig", "gameinfo.db (legacy)", mydownloadconfig),
        Table::new("videoThumb", "gameinfo.db (legacy)", video_thumb),
        Table::new("mycheat", "gameinfo.db (legacy)", mycheat),
    ]
}

/// The legacy `gameinfo.db` schema created by `SQLiteOpenHelper` (version 7).
pub fn legacy_game_info_db() -> Schema {
    Schema {
        name: "gameinfo.db (legacy DBHelper)",
        version: 7,
        style: SqlStyle::Legacy,
        tables: legacy_game_info_tables(),
    }
}

/// Every schema known to this module.
pub fn all_schemas() -> Vec<Schema> {
    vec![game_sir_ux_db(), steam_db(), legacy_game_info_db()]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn game_sir_ux_db_has_all_eleven_tables() {
        let schema = game_sir_ux_db();
        assert_eq!(schema.version, 5);
        assert_eq!(schema.tables.len(), 11);
        assert_eq!(
            schema.table_names(),
            vec![
                "StarterGame",
                "Emulator",
                "Kernel",
                "GameMainTag",
                "ScreenCaptures",
                "ScreenRecords",
                "t_game_library",
                "winemu_file_op_record",
                "steam_account",
                "steam_download_entity",
                "win_emu_trans_config_table",
            ]
        );
    }

    #[test]
    fn generated_starter_game_sql_matches_room_columns() {
        let schema = game_sir_ux_db();
        let sql = schema.create_table_sql(&schema.tables[0]);
        assert!(sql.starts_with("CREATE TABLE IF NOT EXISTS `StarterGame` ("));
        for col in [
            "`id` INTEGER PRIMARY KEY AUTOINCREMENT NOT NULL",
            "`gameId` INTEGER NOT NULL",
            "`fileSize` REAL NOT NULL",
            "`hasDownloaded` INTEGER NOT NULL",
            "`isAddFromLocal` INTEGER NOT NULL",
        ] {
            assert!(sql.contains(col), "missing `{}` in {}", col, sql);
        }
        assert!(sql.ends_with(')'));
    }

    #[test]
    fn generated_game_library_sql_has_snake_case_defaults() {
        let schema = game_sir_ux_db();
        let table = schema
            .tables
            .iter()
            .find(|t| t.name == "t_game_library")
            .unwrap();
        let sql = schema.create_table_sql(table);
        assert!(sql.contains("`user_id` TEXT NOT NULL DEFAULT ''"));
        assert!(sql.contains("`game_id` INTEGER DEFAULT 0"));
        assert!(sql.contains("`game_type` INTEGER NOT NULL DEFAULT 0"));
        assert!(sql.contains("`is_last_launched` INTEGER NOT NULL"));
    }

    #[test]
    fn table_level_primary_keys_are_emitted() {
        let schema = game_sir_ux_db();
        let emulator = schema.tables.iter().find(|t| t.name == "Emulator").unwrap();
        let sql = schema.create_table_sql(emulator);
        assert!(sql.contains("PRIMARY KEY(`id`)"));
        assert!(sql.contains("`sicon` TEXT"));

        let kernel = schema.tables.iter().find(|t| t.name == "Kernel").unwrap();
        assert!(schema
            .create_table_sql(kernel)
            .contains("PRIMARY KEY(`kernelId`)"));
    }

    #[test]
    fn game_sir_account_pk_is_nullable() {
        let schema = game_sir_ux_db();
        let table = schema
            .tables
            .iter()
            .find(|t| t.name == "steam_account")
            .unwrap();
        let sql = schema.create_table_sql(table);
        assert!(sql.contains("`id` INTEGER PRIMARY KEY AUTOINCREMENT,"));
        assert!(!sql.contains("`id` INTEGER PRIMARY KEY AUTOINCREMENT NOT NULL"));
    }

    #[test]
    fn steam_db_has_three_tables_and_its_own_columns() {
        let schema = steam_db();
        assert_eq!(schema.version, 2);
        assert_eq!(
            schema.table_names(),
            vec!["steam_account", "t_steam_user_games", "steam_download_entity"]
        );
        let sql = schema.create_all_sql();
        assert!(sql.contains("`steam_id` INTEGER NOT NULL DEFAULT 0"));
        assert!(sql.contains("`is_remember` INTEGER NOT NULL DEFAULT 0"));
        assert!(sql.contains("`extra_maps` TEXT NOT NULL DEFAULT ''"));
        assert!(sql.contains("`t_steam_user_games`"));
        assert!(sql.contains("`appid` INTEGER NOT NULL DEFAULT 0"));
    }

    #[test]
    fn legacy_schema_uses_legacy_ddl_style_and_all_tables() {
        let schema = legacy_game_info_db();
        assert_eq!(schema.version, 7);
        assert_eq!(
            schema.table_names(),
            vec![
                "gameinfo",
                "mygame",
                "mydownloadconfig",
                "videoThumb",
                "mycheat"
            ]
        );
        let gameinfo = schema.create_table_sql(&schema.tables[0]);
        assert!(gameinfo.starts_with("create table gameinfo("));
        assert!(gameinfo.contains("_id integer PRIMARY KEY AUTOINCREMENT"));
        assert!(gameinfo.contains("categoryshortname TEXT"));
        assert!(!gameinfo.contains('`'));
        assert_eq!(LEGACY_MYGAME_ALTERS.len(), 5);
        assert!(LEGACY_MYGAME_ALTERS[0].contains("is_pk"));
    }

    #[test]
    fn create_all_sql_contains_every_table_name() {
        for schema in all_schemas() {
            let script = schema.create_all_sql();
            for name in schema.table_names() {
                assert!(script.contains(name), "{} missing {}", schema.name, name);
            }
            assert!(script.ends_with(";\n"));
        }
    }

    #[test]
    fn create_all_sql_includes_room_master_table_for_room_schemas() {
        assert!(game_sir_ux_db().create_all_sql().contains(ROOM_MASTER_TABLE_SQL));
        assert!(steam_db().create_all_sql().contains(ROOM_MASTER_TABLE_SQL));
        assert!(!legacy_game_info_db()
            .create_all_sql()
            .contains("room_master_table"));
    }

    #[test]
    fn drop_all_sql_mentions_every_table() {
        let schema = game_sir_ux_db();
        let script = schema.drop_all_sql();
        assert!(script.contains("DROP TABLE IF EXISTS `StarterGame`;"));
        assert!(script.contains("DROP TABLE IF EXISTS `win_emu_trans_config_table`;"));
    }
}
