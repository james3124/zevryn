//! Portable launch-parameter model, derived from the decompiled
//! `com.xj.common.data.gameinfo.GameStartupParams` and the `id` / `valid_id`
//! fields of `com.xj.common.data.gameinfo.GameDetailEntity`.
//!
//! Holds only the fields the launch decision needs: `game_id`, `valid_id`,
//! `start_type` and extra arguments. All Android/serialization concerns
//! (Gson `@SerializedName`, drawables, icons, file sizes) are out of scope.

use super::start_type::GameStartType;

/// Extra launch arguments, kept as opaque key/value pairs so the portable
/// layer does not depend on Steam/cloud protocol types.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct ExtraArgs {
    entries: Vec<(String, String)>,
}

impl ExtraArgs {
    pub fn new() -> Self {
        Self { entries: Vec::new() }
    }

    pub fn insert(&mut self, key: impl Into<String>, value: impl Into<String>) {
        self.entries.push((key.into(), value.into()));
    }

    pub fn get(&self, key: &str) -> Option<&str> {
        self.entries
            .iter()
            .find(|(k, _)| k == key)
            .map(|(_, v)| v.as_str())
    }

    pub fn len(&self) -> usize {
        self.entries.len()
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    pub fn as_slice(&self) -> &[(String, String)] {
        &self.entries
    }
}

/// Parameters describing a game launch request.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GameLaunchParams {
    /// `GameDetailEntity.id` (numeric internal game id).
    pub game_id: i64,
    /// `GameDetailEntity.valid_id` (external/valid id string).
    pub valid_id: String,
    /// The launch type, mapped from a decompiled `start_type` code.
    pub start_type: GameStartType,
    /// Extra, opaque arguments (`start_res`, `exe_path`, `pkg_name`, ...).
    pub extra: ExtraArgs,
}

impl GameLaunchParams {
    pub fn builder() -> GameLaunchParamsBuilder {
        GameLaunchParamsBuilder::new()
    }
}

/// Builder for [`GameLaunchParams`]. `game_id` and `valid_id` are required;
/// `start_type` defaults to the decompiled default `start_type` value `1`
/// (the `GameStartupParams` synthetic constructor default, which is *not* a
/// named `START_TYPE_*` code). Set it explicitly for real launches.
#[derive(Clone, Debug)]
pub struct GameLaunchParamsBuilder {
    game_id: i64,
    valid_id: String,
    start_type: GameStartType,
    extra: ExtraArgs,
}

impl Default for GameLaunchParamsBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl GameLaunchParamsBuilder {
    pub fn new() -> Self {
        Self {
            game_id: 0,
            valid_id: String::new(),
            start_type: GameStartType::Mobile,
            extra: ExtraArgs::new(),
        }
    }

    pub fn game_id(mut self, id: i64) -> Self {
        self.game_id = id;
        self
    }

    pub fn valid_id(mut self, id: impl Into<String>) -> Self {
        self.valid_id = id.into();
        self
    }

    pub fn start_type(mut self, ty: GameStartType) -> Self {
        self.start_type = ty;
        self
    }

    /// Convenience: set start type from a raw decompiled `start_type` code.
    /// Returns `false` (leaving the builder unchanged) for unknown codes.
    pub fn start_type_code(mut self, code: i32) -> Option<Self> {
        self.start_type = GameStartType::from_code(code)?;
        Some(self)
    }

    pub fn extra_arg(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.extra.insert(key, value);
        self
    }

    pub fn extra(mut self, extra: ExtraArgs) -> Self {
        self.extra = extra;
        self
    }

    pub fn build(self) -> GameLaunchParams {
        GameLaunchParams {
            game_id: self.game_id,
            valid_id: self.valid_id,
            start_type: self.start_type,
            extra: self.extra,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn builder_sets_all_fields() {
        let params = GameLaunchParams::builder()
            .game_id(42)
            .valid_id("valid-42")
            .start_type(GameStartType::LocalPcEmu)
            .extra_arg("start_res", "https://example/game.exe")
            .extra_arg("pkg_name", "com.example.game")
            .build();

        assert_eq!(params.game_id, 42);
        assert_eq!(params.valid_id, "valid-42");
        assert_eq!(params.start_type, GameStartType::LocalPcEmu);
        assert_eq!(params.extra.get("start_res"), Some("https://example/game.exe"));
        assert_eq!(params.extra.get("pkg_name"), Some("com.example.game"));
        assert_eq!(params.extra.len(), 2);
    }

    #[test]
    fn builder_accepts_raw_decompiled_start_type_codes() {
        let p = GameLaunchParams::builder()
            .start_type_code(1403)
            .expect("1403 is a known code")
            .build();
        assert_eq!(p.start_type, GameStartType::LocalPcEmu);

        let p2 = GameLaunchParams::builder().start_type_code(9999);
        assert!(p2.is_none(), "unknown codes must not be invented");
    }

    #[test]
    fn defaults_are_stable() {
        let p = GameLaunchParams::builder().build();
        assert_eq!(p.game_id, 0);
        assert_eq!(p.valid_id, "");
        assert_eq!(p.start_type, GameStartType::Mobile);
        assert!(p.extra.is_empty());
    }

    #[test]
    fn extra_args_lookup_and_clone() {
        let mut extra = ExtraArgs::new();
        extra.insert("exe_path", "C:\\game\\game.exe");
        let params = GameLaunchParams::builder()
            .valid_id("v")
            .extra(extra.clone())
            .build();
        assert_eq!(params.extra.as_slice(), extra.as_slice());
        assert_eq!(params.extra.get("exe_path"), Some("C:\\game\\game.exe"));
        assert_eq!(params.extra.get("missing"), None);
    }
}
