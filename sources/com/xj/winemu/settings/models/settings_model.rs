//! Portable `SettingsModel`: an in-memory settings tree with lookup, update,
//! reset, flatten and validation helpers.
//!
//! This is the dependency-free counterpart of the tree assembled by
//! `PcGameSettingsKt` (`sources/com/xj/winemu/settings/models/PcGameSettingsKt.java`)
//! via `SettingDSLKt.e/h/l` (`SettingDSLKt.java`): a `Settings` list of
//! `GroupSetting`s, each holding `ChildSetting` rows with an optional
//! `SettingTypeValue`. The Android activity, dialogs, `Function1`/`Function2`
//! callbacks, resource ids (`R.string.*`) and view holders are out of scope —
//! only the portable data shape and its tree operations are modeled.

use super::child_setting::ChildSetting;
use super::group_setting::GroupSetting;
use super::setting_type_value::SettingTypeValue;

/// An owned settings tree: ordered groups of child settings.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct SettingsModel {
    /// Groups in display order.
    pub groups: Vec<GroupSetting>,
}

/// A problem found by [`SettingsModel::validate`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ValidationError {
    /// A child has an empty key (it could not be looked up).
    EmptyKey { group: String, index: usize },
    /// A child has an empty label.
    EmptyLabel { key: String },
    /// The same key appears more than once in the tree.
    DuplicateKey { key: String },
    /// `value` and `default` hold different payload variants.
    ValueDefaultKindMismatch {
        key: String,
        value_kind: &'static str,
        default_kind: &'static str,
    },
}

impl core::fmt::Display for ValidationError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::EmptyKey { group, index } => {
                write!(f, "child #{index} in group '{group}' has an empty key")
            }
            Self::EmptyLabel { key } => write!(f, "child '{key}' has an empty label"),
            Self::DuplicateKey { key } => write!(f, "duplicate child key '{key}'"),
            Self::ValueDefaultKindMismatch {
                key,
                value_kind,
                default_kind,
            } => write!(
                f,
                "child '{key}' has value kind {value_kind} but default kind {default_kind}"
            ),
        }
    }
}

impl SettingsModel {
    /// Build a model from an ordered list of groups.
    pub fn new(groups: Vec<GroupSetting>) -> Self {
        Self { groups }
    }

    /// An empty model (equivalent to `SettingsModel::default()`).
    pub fn empty() -> Self {
        Self::default()
    }

    /// Add a group to the end of the tree.
    pub fn push_group(&mut self, group: GroupSetting) {
        self.groups.push(group);
    }

    /// Borrow the groups slice (`Settings.a()`).
    pub fn groups(&self) -> &[GroupSetting] {
        &self.groups
    }

    /// Mutably borrow the groups slice.
    pub fn groups_mut(&mut self) -> &mut [GroupSetting] {
        &mut self.groups
    }

    /// Number of groups.
    pub fn group_count(&self) -> usize {
        self.groups.len()
    }

    /// Find a group by its id.
    pub fn group(&self, id: &str) -> Option<&GroupSetting> {
        self.groups.iter().find(|g| g.id == id)
    }

    /// Find the first child with `key` across all groups.
    pub fn find(&self, key: &str) -> Option<&ChildSetting> {
        self.groups.iter().find_map(|g| g.find(key))
    }

    /// Find the first child with `key` across all groups, mutably.
    pub fn find_mut(&mut self, key: &str) -> Option<&mut ChildSetting> {
        self.groups.iter_mut().find_map(|g| g.find_mut(key))
    }

    /// Update the value of the child with `key` in place.
    ///
    /// Returns `true` when a child was found and updated.
    pub fn set_value(&mut self, key: &str, value: SettingTypeValue) -> bool {
        match self.find_mut(key) {
            Some(child) => {
                child.value = Some(value);
                true
            }
            None => false,
        }
    }

    /// Return an updated copy of the tree with `key` set to `value`.
    ///
    /// This is the immutable/builder counterpart of [`SettingsModel::set_value`]
    /// for callers that want the updated tree returned directly.
    pub fn with_value(&self, key: &str, value: SettingTypeValue) -> Self {
        let mut next = self.clone();
        next.set_value(key, value);
        next
    }

    /// Restore every child's current value from its default.
    ///
    /// Returns the number of children whose value actually changed.
    pub fn reset_to_defaults(&mut self) -> usize {
        let mut changed = 0;
        for group in &mut self.groups {
            for child in &mut group.children {
                if child.reset() {
                    changed += 1;
                }
            }
        }
        changed
    }

    /// Return an updated copy of the tree with all values reset to defaults.
    pub fn with_defaults(&self) -> Self {
        let mut next = self.clone();
        next.reset_to_defaults();
        next
    }

    /// Total number of child rows across all groups.
    pub fn child_count(&self) -> usize {
        self.groups.iter().map(GroupSetting::len).sum()
    }

    /// Borrow every child row in group-then-position order.
    pub fn flatten(&self) -> Vec<&ChildSetting> {
        flatten(self)
    }

    /// Validate keys, labels and value/default consistency.
    pub fn validate(&self) -> Result<(), Vec<ValidationError>> {
        validate(self)
    }
}

/// Flatten a model into a flat list of child references in display order.
pub fn flatten(model: &SettingsModel) -> Vec<&ChildSetting> {
    model
        .groups
        .iter()
        .flat_map(|g| g.children.iter())
        .collect()
}

/// Validate a model's structure, returning every problem found.
pub fn validate(model: &SettingsModel) -> Result<(), Vec<ValidationError>> {
    let mut errors = Vec::new();
    let mut seen: Vec<&str> = Vec::new();

    for group in &model.groups {
        for (index, child) in group.children.iter().enumerate() {
            if child.key.is_empty() {
                errors.push(ValidationError::EmptyKey {
                    group: group.id.clone(),
                    index,
                });
            } else if seen.contains(&child.key.as_str()) {
                errors.push(ValidationError::DuplicateKey {
                    key: child.key.clone(),
                });
            } else {
                seen.push(&child.key);
            }

            if child.label.is_empty() {
                errors.push(ValidationError::EmptyLabel {
                    key: child.key.clone(),
                });
            }

            if let (Some(v), Some(d)) = (&child.value, &child.default) {
                if v.kind() != d.kind() {
                    errors.push(ValidationError::ValueDefaultKindMismatch {
                        key: child.key.clone(),
                        value_kind: v.kind(),
                        default_kind: d.kind(),
                    });
                }
            }
        }
    }

    if errors.is_empty() {
        Ok(())
    } else {
        Err(errors)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::super::EditType;

    fn sample_model() -> SettingsModel {
        let mut general = GroupSetting::new("general", "General");
        general.push(
            ChildSetting::new(
                "enable_hud",
                "Enable HUD",
                EditType::EditGameVars,
                Some(SettingTypeValue::switch(false)),
                Some(SettingTypeValue::switch(false)),
                true,
            )
            .with_desc("Toggle the HUD"),
        );
        general.push(ChildSetting::action("exe_path", "Local exe path"));

        let mut compat = GroupSetting::new("compatibility", "Compatibility");
        compat.push(
            ChildSetting::action("dxvk", "DXVK")
                .with_edit_type(EditType::EditTranslationConfigName)
                .with_value(SettingTypeValue::button(1, 2, 3)),
        );

        SettingsModel::new(vec![general, compat])
    }

    #[test]
    fn tree_shape_and_find() {
        let model = sample_model();
        assert_eq!(model.group_count(), 2);
        assert_eq!(model.child_count(), 3);
        assert_eq!(model.groups().len(), 2);
        assert_eq!(model.group("compatibility").unwrap().title, "Compatibility");
        assert_eq!(model.find("dxvk").unwrap().label, "DXVK");
        assert!(model.find("nope").is_none());
        assert_eq!(model.flatten().len(), 3);
    }

    #[test]
    fn set_value_in_place_and_return_updated_tree() {
        let mut model = sample_model();
        assert!(model.set_value("enable_hud", SettingTypeValue::switch(true)));
        assert_eq!(model.find("enable_hud").unwrap().value.as_ref().unwrap().as_switch(), Some(true));
        assert!(!model.set_value("missing", SettingTypeValue::switch(true)));

        let updated = model.with_value("dxvk", SettingTypeValue::button(9, 8, 7));
        assert!(updated.find("dxvk").unwrap().is_modified());
        assert_eq!(
            model.find("dxvk").unwrap().value,
            Some(SettingTypeValue::button(1, 2, 3)),
            "the original tree must be unchanged by with_value"
        );
    }

    #[test]
    fn reset_to_defaults() {
        let mut model = sample_model();
        model.set_value("enable_hud", SettingTypeValue::switch(true));
        assert_eq!(model.reset_to_defaults(), 1);
        assert_eq!(model.find("enable_hud").unwrap().value.as_ref().unwrap().as_switch(), Some(false));
        assert_eq!(model.reset_to_defaults(), 0);

        let updated = model.with_value("dxvk", SettingTypeValue::button(5, 5, 5));
        let reset = updated.with_defaults();
        assert!(!reset.find("dxvk").unwrap().is_modified());
    }

    #[test]
    fn find_mut_updates_a_child() {
        let mut model = sample_model();
        model.find_mut("exe_path").unwrap().set_enabled(false);
        assert!(!model.find("exe_path").unwrap().enabled);
        model.groups_mut()[0].push(ChildSetting::action("extra", "Extra"));
        assert_eq!(model.child_count(), 4);
    }

    #[test]
    fn validate_accepts_a_well_formed_tree() {
        assert!(sample_model().validate().is_ok());
        assert!(SettingsModel::empty().validate().is_ok());
    }

    #[test]
    fn validate_detects_problems() {
        let mut model = SettingsModel::empty();
        let mut group = GroupSetting::new("g", "G");
        group.push(ChildSetting::action("", "No key"));
        group.push(ChildSetting::action("dup", "First"));
        group.push(ChildSetting::action("dup", "Second"));
        group.push(ChildSetting::action("blank", ""));
        group.push(
            ChildSetting::action("mismatch", "Mismatch")
                .with_value(SettingTypeValue::switch(true)),
        );
        // set a differing default variant directly
        model.push_group(group);
        model.find_mut("mismatch").unwrap().default = Some(SettingTypeValue::button(0, 0, 0));

        let errors = model.validate().unwrap_err();
        assert!(errors
            .iter()
            .any(|e| matches!(e, ValidationError::EmptyKey { .. })));
        assert!(errors
            .iter()
            .any(|e| matches!(e, ValidationError::DuplicateKey { key } if key == "dup")));
        assert!(errors
            .iter()
            .any(|e| matches!(e, ValidationError::EmptyLabel { key } if key == "blank")));
        assert!(errors.iter().any(|e| matches!(
            e,
            ValidationError::ValueDefaultKindMismatch { key, .. } if key == "mismatch"
        )));
    }
}
