//! Portable model of the game-compatibility DTOs in
//! `com.xj.common.data.gameinfo` (GameHub Lite v5.1.8).
//!
//! Field names mirror the `@SerializedName` annotations of
//! `GameCompatibilityParams`, `GameCompatibilityItemParams` and
//! `SimpleGameCompatibility` exactly (including the snake_case keys
//! `is_check`, `game_id`, `steam_appid` and the `level` column).
//!
//! Out of scope: Gson parsing, the `CompatibleDialog` UI, and the Android
//! resource strings used for display.

/// One entry of a compatibility preset's `data` list.
/// Source: `GameCompatibilityItemParams.java` (`@SerializedName` fields).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CompatibilityItem {
    /// `@SerializedName("title")`
    pub title: String,
    /// `@SerializedName("icon")`
    pub icon: String,
    /// `@SerializedName("desc")`
    pub desc: String,
    /// `@SerializedName("level")`
    pub level: i32,
    /// `@SerializedName("is_check")`
    pub is_check: bool,
}

impl CompatibilityItem {
    pub fn new(title: impl Into<String>, level: i32, is_check: bool) -> Self {
        Self {
            title: title.into(),
            icon: String::new(),
            desc: String::new(),
            level,
            is_check,
        }
    }
}

/// A compatibility preset returned as `cst_data` / `notFountCts`.
/// Source: `GameCompatibilityParams.java` (`@SerializedName` fields).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CompatibilityPreset {
    /// `@SerializedName("title")`
    pub title: String,
    /// `@SerializedName("icon")`
    pub icon: String,
    /// `@SerializedName("desc")`
    pub desc: String,
    /// `@SerializedName("level")`
    pub level: i32,
    /// `@SerializedName("data")` (list of [`CompatibilityItem`])
    pub data: Vec<CompatibilityItem>,
}

impl CompatibilityPreset {
    pub fn new(title: impl Into<String>, level: i32) -> Self {
        Self {
            title: title.into(),
            icon: String::new(),
            desc: String::new(),
            level,
            data: Vec::new(),
        }
    }

    pub fn with_desc(mut self, desc: impl Into<String>) -> Self {
        self.desc = desc.into();
        self
    }

    pub fn with_icon(mut self, icon: impl Into<String>) -> Self {
        self.icon = icon.into();
        self
    }

    pub fn with_data(mut self, data: Vec<CompatibilityItem>) -> Self {
        self.data = data;
        self
    }

    pub fn item_count(&self) -> usize {
        self.data.len()
    }
}

/// Compact per-game compatibility record.
/// Source: `SimpleGameCompatibility.java` (`@SerializedName` fields).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SimpleGameCompatibility {
    /// `@SerializedName("game_id")`
    pub game_id: i32,
    /// `@SerializedName("steam_appid")`
    pub steam_app_id: String,
    /// `@SerializedName("title")`
    pub title: String,
    /// `@SerializedName("icon")`
    pub icon: String,
    /// `@SerializedName("desc")`
    pub desc: String,
    /// `@SerializedName("level")`
    pub level: i32,
}

impl SimpleGameCompatibility {
    pub fn new(game_id: i32, steam_app_id: impl Into<String>, level: i32) -> Self {
        Self {
            game_id,
            steam_app_id: steam_app_id.into(),
            title: String::new(),
            icon: String::new(),
            desc: String::new(),
            level,
        }
    }

    /// `SimpleGameCompatibility.isUnknownLevel()` -> `level == 0`.
    pub fn is_unknown_level(&self) -> bool {
        self.level == 0
    }
}

/// Find a preset by its `level` among a list of presets.
pub fn find_preset_by_level<'a>(
    presets: &'a [CompatibilityPreset],
    level: i32,
) -> Option<&'a CompatibilityPreset> {
    presets.iter().find(|preset| preset.level == level)
}

/// The two presets passed to `CompatibleDialog(compatibleData,
/// compatibleNotFoundData)` from `GameDetailSettingMenu.F`: the "found"
/// preset (`cst_data`) and the "not found" preset (`notFountCts`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CompatibilityCatalog {
    pub found: CompatibilityPreset,
    pub not_found: CompatibilityPreset,
}

impl CompatibilityCatalog {
    pub fn new(found: CompatibilityPreset, not_found: CompatibilityPreset) -> Self {
        Self { found, not_found }
    }

    /// Pick the preset to display: when the game's compatibility level is
    /// unknown (`isUnknownLevel()`), fall back to the "not found" preset;
    /// otherwise use the "found" preset.
    pub fn lookup<'a>(&'a self, compat: &SimpleGameCompatibility) -> &'a CompatibilityPreset {
        if compat.is_unknown_level() {
            &self.not_found
        } else {
            &self.found
        }
    }

    /// Level-based lookup across an explicit preset list, returning `None`
    /// for an unknown compatibility level.
    pub fn lookup_by_level<'a>(
        &self,
        presets: &'a [CompatibilityPreset],
        compat: &SimpleGameCompatibility,
    ) -> Option<&'a CompatibilityPreset> {
        if compat.is_unknown_level() {
            None
        } else {
            find_preset_by_level(presets, compat.level)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn catalog() -> CompatibilityCatalog {
        let found = CompatibilityPreset::new("Compatible", 2)
            .with_data(vec![CompatibilityItem::new("dxvk", 2, true)]);
        let not_found = CompatibilityPreset::new("Not found", 0);
        CompatibilityCatalog::new(found, not_found)
    }

    #[test]
    fn unknown_level_selects_not_found_preset() {
        let catalog = catalog();
        let compat = SimpleGameCompatibility::new(1, "10", 0);
        assert!(compat.is_unknown_level());
        assert_eq!(catalog.lookup(&compat).title, "Not found");
    }

    #[test]
    fn known_level_selects_found_preset_and_keeps_items() {
        let catalog = catalog();
        let compat = SimpleGameCompatibility::new(2, "20", 2);
        assert!(!compat.is_unknown_level());
        let preset = catalog.lookup(&compat);
        assert_eq!(preset.title, "Compatible");
        assert_eq!(preset.item_count(), 1);
        assert_eq!(preset.data[0].title, "dxvk");
        assert!(preset.data[0].is_check);
    }

    #[test]
    fn find_preset_by_level_and_lookup_by_level() {
        let found = CompatibilityPreset::new("Tier 3", 3);
        let other = CompatibilityPreset::new("Tier 2", 2);
        let presets = vec![other, found];
        let catalog = catalog();

        assert_eq!(find_preset_by_level(&presets, 3).map(|p| p.title.as_str()), Some("Tier 3"));
        assert_eq!(find_preset_by_level(&presets, 99), None);

        let known = SimpleGameCompatibility::new(3, "30", 3);
        assert_eq!(catalog.lookup_by_level(&presets, &known).map(|p| p.title.as_str()), Some("Tier 3"));
        let unknown = SimpleGameCompatibility::new(4, "40", 0);
        assert_eq!(catalog.lookup_by_level(&presets, &unknown), None);
    }
}
