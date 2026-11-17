//! Portable model of the translation-configuration state used by
//! `com.xj.winemu.settings.translations` (GameHub Lite v5.1.8).
//!
//! [`TranslationConfig`] carries the exact fields of the Room entity
//! `com.xj.common.data.table.WinEmuTranslationConfigEntity` (the table behind
//! `WinEmuTranslationConfigDao`). [`ConfigModifyState`] mirrors the enum in
//! `sources/com/xj/winemu/settings/translations/data/ConfigModifyState.java`;
//! [`ConfigChange`] mirrors the `Pair<ConfigModifyState, config>` events fired
//! by `TranslationConfigViewModel` / `UpdateTranslationConfigEvent`.
//!
//! Out of scope: Room persistence, DAO queries, coroutine `Flow` plumbing.

/// Result state of a create/delete/apply/update operation on a translation
/// config. Source: `ConfigModifyState.java` (declaration order preserved).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ConfigModifyState {
    CreateSuccess,
    CreateFailure,
    DeleteSuccess,
    DeleteFailure,
    ApplySuccess,
    ApplyFailure,
    UpdateSuccess,
    UpdateFailure,
}

impl ConfigModifyState {
    /// The exact Kotlin/Java enum constant name.
    pub fn as_str(self) -> &'static str {
        match self {
            Self::CreateSuccess => "CreateSuccess",
            Self::CreateFailure => "CreateFailure",
            Self::DeleteSuccess => "DeleteSuccess",
            Self::DeleteFailure => "DeleteFailure",
            Self::ApplySuccess => "ApplySuccess",
            Self::ApplyFailure => "ApplyFailure",
            Self::UpdateSuccess => "UpdateSuccess",
            Self::UpdateFailure => "UpdateFailure",
        }
    }

    pub fn parse(name: &str) -> Option<Self> {
        match name {
            "CreateSuccess" => Some(Self::CreateSuccess),
            "CreateFailure" => Some(Self::CreateFailure),
            "DeleteSuccess" => Some(Self::DeleteSuccess),
            "DeleteFailure" => Some(Self::DeleteFailure),
            "ApplySuccess" => Some(Self::ApplySuccess),
            "ApplyFailure" => Some(Self::ApplyFailure),
            "UpdateSuccess" => Some(Self::UpdateSuccess),
            "UpdateFailure" => Some(Self::UpdateFailure),
            _ => None,
        }
    }

    pub fn is_success(self) -> bool {
        matches!(
            self,
            Self::CreateSuccess | Self::DeleteSuccess | Self::ApplySuccess | Self::UpdateSuccess
        )
    }
}

/// The translation-config table row. Field names mirror the `@ColumnInfo`
/// fields / constructor components of `WinEmuTranslationConfigEntity` exactly.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TranslationConfig {
    /// `uid`
    pub uid: i32,
    /// `id` (`@PrimaryKey`)
    pub id: i32,
    /// `name`
    pub name: String,
    /// `configId`
    pub config_id: String,
    /// `createTime`
    pub create_time: i64,
    /// `lastUpdateTime`
    pub last_update_time: i64,
    /// `gameId`
    pub game_id: String,
    /// `config`
    pub config: String,
    /// `copyFromId`
    pub copy_from_id: String,
    /// `copyFromName`
    pub copy_from_name: String,
}

impl TranslationConfig {
    /// `WinEmuTranslationConfigEntity.ID_PREFIX`.
    pub const ID_PREFIX: &'static str = "local_";

    /// Mirrors the entity defaults (empty strings, id 0, times 0).
    pub fn new(name: impl Into<String>, config_id: impl Into<String>) -> Self {
        Self {
            uid: 0,
            id: 0,
            name: name.into(),
            config_id: config_id.into(),
            create_time: 0,
            last_update_time: 0,
            game_id: String::new(),
            config: String::new(),
            copy_from_id: String::new(),
            copy_from_name: String::new(),
        }
    }

    /// `Companion.createLocalId(sourceId)` -> `"local_" + sourceId`.
    pub fn create_local_id(source_id: &str) -> String {
        format!("{}{}", Self::ID_PREFIX, source_id)
    }

    /// A locally created config has a `configId` with the `local_` prefix.
    pub fn is_local(&self) -> bool {
        self.config_id.starts_with(Self::ID_PREFIX)
    }

    /// `setLastUpdateTime(System.currentTimeMillis())` equivalent; the caller
    /// supplies the timestamp so the model stays free of wall-clock I/O.
    pub fn touch(&mut self, now_millis: i64) {
        self.last_update_time = now_millis;
    }
}

/// A `Pair<ConfigModifyState, config>` emitted after a config operation
/// (`UpdateTranslationConfigEvent`, `TranslationConfigViewModel.*$1`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConfigChange {
    pub state: ConfigModifyState,
    pub config: Option<TranslationConfig>,
}

impl ConfigChange {
    pub fn new(state: ConfigModifyState, config: Option<TranslationConfig>) -> Self {
        Self { state, config }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn config_modify_state_round_trip_and_success() {
        for state in [
            ConfigModifyState::CreateSuccess,
            ConfigModifyState::CreateFailure,
            ConfigModifyState::DeleteSuccess,
            ConfigModifyState::DeleteFailure,
            ConfigModifyState::ApplySuccess,
            ConfigModifyState::ApplyFailure,
            ConfigModifyState::UpdateSuccess,
            ConfigModifyState::UpdateFailure,
        ] {
            assert_eq!(ConfigModifyState::parse(state.as_str()), Some(state));
            assert_eq!(state.is_success(), state.as_str().ends_with("Success"));
        }
        assert_eq!(ConfigModifyState::parse("Unknown"), None);
    }

    #[test]
    fn create_local_id_and_is_local() {
        assert_eq!(TranslationConfig::create_local_id("abc"), "local_abc");
        let local = TranslationConfig::new("My config", TranslationConfig::create_local_id("7"));
        assert!(local.is_local());
        let official = TranslationConfig::new("Official", "official_7");
        assert!(!official.is_local());
    }

    #[test]
    fn translation_config_fields_and_touch() {
        let mut config = TranslationConfig::new("Cfg", "local_1");
        config.uid = 42;
        config.id = 9;
        config.game_id = "100".into();
        config.config = "{}".into();
        config.copy_from_id = "official_1".into();
        config.copy_from_name = "Base".into();
        assert_eq!(config.uid, 42);
        assert_eq!(config.game_id, "100");
        assert_eq!(config.copy_from_name, "Base");
        config.touch(1234);
        assert_eq!(config.last_update_time, 1234);
    }

    #[test]
    fn config_change_carries_state_and_optional_config() {
        let ok = ConfigChange::new(ConfigModifyState::UpdateSuccess, Some(TranslationConfig::new("n", "local_1")));
        assert!(ok.state.is_success());
        assert_eq!(ok.config.unwrap().name, "n");
        let fail = ConfigChange::new(ConfigModifyState::ApplyFailure, None);
        assert!(fail.config.is_none());
    }
}
