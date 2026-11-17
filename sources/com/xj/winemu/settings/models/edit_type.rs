//! Rust model of `com.xj.winemu.settings.EditType` (GameHub Lite v5.1.8).
//!
//! Source: `sources/com/xj/winemu/settings/EditType.java` and the matching
//! `apktool/smali_classes4/com/xj/winemu/settings/EditType.smali`. Both contain
//! **exactly two** enum constants, in this declaration order:
//!
//! - `EditGameVars` (ordinal 0)
//! - `EditTranslationConfigName` (ordinal 1)
//!
//! The task brief mentioned "SWITCH/BUTTON/INPUT/JUMP/etc." as examples of
//! editor types. Those names do **not** occur in `EditType.java`; they belong
//! to other enums (`com.xj.winemu.bean.SettingType` -> Option/Switch/Input and
//! `PcSettingItemEntity.TYPE_SWITCH` / `TYPE_JUMP_ITEM`). Per the instruction to
//! use only values actually found, this enum models the two real `EditType`
//! constants. JVM-only metadata (`@Metadata`, `EnumEntries`, `values()` bridge)
//! is intentionally omitted.

/// The exact set of setting editor types from `EditType.java`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum EditType {
    /// `EditType.EditGameVars` (ordinal 0) — edit the game variables file.
    #[default]
    EditGameVars,
    /// `EditType.EditTranslationConfigName` (ordinal 1) — edit the translation
    /// config name.
    EditTranslationConfigName,
}

impl EditType {
    /// Every variant in declaration order (mirrors `EditType.values()`).
    pub const ALL: [Self; 2] = [Self::EditGameVars, Self::EditTranslationConfigName];

    /// The Java enum constant name (`name()`).
    pub fn as_str(self) -> &'static str {
        match self {
            Self::EditGameVars => "EditGameVars",
            Self::EditTranslationConfigName => "EditTranslationConfigName",
        }
    }

    /// The Kotlin/Java `ordinal()` of this constant.
    pub fn ordinal(self) -> usize {
        match self {
            Self::EditGameVars => 0,
            Self::EditTranslationConfigName => 1,
        }
    }

    /// Inverse of [`EditType::as_str`] (`EditType.valueOf(name)`).
    pub fn from_name(name: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|v| v.as_str() == name)
    }

    /// Inverse of [`EditType::ordinal`].
    pub fn from_ordinal(ordinal: usize) -> Option<Self> {
        Self::ALL.get(ordinal).copied()
    }
}

impl core::fmt::Display for EditType {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_str(self.as_str())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn edit_type_has_exactly_two_values() {
        assert_eq!(EditType::ALL.len(), 2);
        assert_eq!(EditType::EditGameVars.ordinal(), 0);
        assert_eq!(EditType::EditTranslationConfigName.ordinal(), 1);
    }

    #[test]
    fn edit_type_name_round_trip() {
        for variant in EditType::ALL {
            assert_eq!(EditType::from_name(variant.as_str()), Some(variant));
            assert_eq!(EditType::from_ordinal(variant.ordinal()), Some(variant));
        }
        assert_eq!(EditType::from_name("SWITCH"), None);
        assert_eq!(EditType::from_name("Jump"), None);
        assert_eq!(EditType::from_ordinal(2), None);
    }

    #[test]
    fn edit_type_default_is_first_declared() {
        assert_eq!(EditType::default(), EditType::EditGameVars);
        assert_eq!(EditType::EditGameVars.to_string(), "EditGameVars");
    }
}
