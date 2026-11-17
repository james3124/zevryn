//! Rust model of `com.xj.winemu.settings.models.ChildSetting` (GameHub Lite
//! v5.1.8). Source: `sources/com/xj/winemu/settings/models/ChildSetting.java`.
//!
//! Java fields and their portable mapping:
//! - `id`       -> [`ChildSetting::key`]
//! - `name`     -> [`ChildSetting::label`]
//! - `desc`     -> [`ChildSetting::desc`]
//! - `defaultRightText` -> conceptual default value -> [`ChildSetting::default`]
//! - `icon`     -> [`ChildSetting::icon`]
//! - `enable`   -> [`ChildSetting::enabled`]
//! - `value: SettingTypeValue` -> [`ChildSetting::value`]
//!
//! `edit_type` is requested by the port's tree API. Note that the Java
//! `ChildSetting` has no `EditType` field — `EditType` is a separate enum used
//! by the "edit game vars / translation config name" flows — so it is modeled
//! here as a stored attribute defaulting to the first declared constant.
//! Android-only callback slots (`Function1 action`, `Function1 onChanged`,
//! `Function0 click`) and `Parcelable` plumbing are out of scope.

use super::edit_type::EditType;
use super::icons::Icons;
use super::setting_type_value::SettingTypeValue;

/// A single leaf in the settings tree.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ChildSetting {
    /// Stable identifier (`id`), used as the lookup key in `SettingsModel`.
    pub key: String,
    /// Human-facing title (`name`).
    pub label: String,
    /// Description (`desc`).
    pub desc: String,
    /// Which editor opens this setting (portable attribute, see module docs).
    pub edit_type: EditType,
    /// Current value; `None` for pure "jump"/action rows.
    pub value: Option<SettingTypeValue>,
    /// Default value restored by `SettingsModel::reset_to_defaults`.
    pub default: Option<SettingTypeValue>,
    /// Whether the row is interactive (`enable`).
    pub enabled: bool,
    /// Optional icon (`icon`).
    pub icon: Option<Icons>,
}

impl ChildSetting {
    /// Construct a fully specified child setting.
    pub fn new(
        key: impl Into<String>,
        label: impl Into<String>,
        edit_type: EditType,
        value: Option<SettingTypeValue>,
        default: Option<SettingTypeValue>,
        enabled: bool,
    ) -> Self {
        Self {
            key: key.into(),
            label: label.into(),
            desc: String::new(),
            edit_type,
            value,
            default,
            enabled,
            icon: None,
        }
    }

    /// Minimal constructor matching the most common "jump"/action row shape.
    pub fn action(key: impl Into<String>, label: impl Into<String>) -> Self {
        Self::new(key, label, EditType::EditGameVars, None, None, true)
    }

    /// Builder-style setter for the description.
    pub fn with_desc(mut self, desc: impl Into<String>) -> Self {
        self.desc = desc.into();
        self
    }

    /// Builder-style setter for the icon.
    pub fn with_icon(mut self, icon: Icons) -> Self {
        self.icon = Some(icon);
        self
    }

    /// Builder-style setter for the edit type.
    pub fn with_edit_type(mut self, edit_type: EditType) -> Self {
        self.edit_type = edit_type;
        self
    }

    /// Builder-style setter for the current and default value at once.
    pub fn with_value(mut self, value: SettingTypeValue) -> Self {
        self.default = Some(value.clone());
        self.value = Some(value);
        self
    }

    /// Set the current value, returning the previous value.
    pub fn set_value(&mut self, value: Option<SettingTypeValue>) -> Option<SettingTypeValue> {
        core::mem::replace(&mut self.value, value)
    }

    /// Restore the current value from `default`, returning whether anything
    /// changed.
    pub fn reset(&mut self) -> bool {
        if self.value == self.default {
            false
        } else {
            self.value = self.default.clone();
            true
        }
    }

    /// True when the current value differs from the default.
    pub fn is_modified(&self) -> bool {
        self.value != self.default
    }

    /// Set the enabled flag (`ChildSetting.o(boolean)`).
    pub fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
    }
}

impl Default for ChildSetting {
    fn default() -> Self {
        Self {
            key: String::new(),
            label: String::new(),
            desc: String::new(),
            edit_type: EditType::default(),
            value: None,
            default: None,
            enabled: true,
            icon: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn action_row_defaults() {
        let row = ChildSetting::action("launch", "Launch container");
        assert_eq!(row.key, "launch");
        assert_eq!(row.label, "Launch container");
        assert_eq!(row.edit_type, EditType::EditGameVars);
        assert!(row.value.is_none());
        assert!(row.enabled);
        assert!(!row.is_modified());
    }

    #[test]
    fn value_set_reset_and_modified() {
        let mut row = ChildSetting::action("switch", "Enable").with_value(SettingTypeValue::switch(false));
        assert!(!row.is_modified());
        row.set_value(Some(SettingTypeValue::switch(true)));
        assert!(row.is_modified());
        assert_eq!(row.value.as_ref().unwrap().as_switch(), Some(true));
        assert!(row.reset());
        assert_eq!(row.value.as_ref().unwrap().as_switch(), Some(false));
        assert!(!row.is_modified());
        assert!(!row.reset());
    }

    #[test]
    fn builder_chain_applies_all_fields() {
        let row = ChildSetting::action("icon", "Open")
            .with_desc("desc")
            .with_icon(Icons::LaunchContainer)
            .with_edit_type(EditType::EditTranslationConfigName);
        assert_eq!(row.desc, "desc");
        assert_eq!(row.icon, Some(Icons::LaunchContainer));
        assert_eq!(row.edit_type, EditType::EditTranslationConfigName);
    }
}
