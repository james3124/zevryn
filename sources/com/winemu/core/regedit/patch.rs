//! Rust model of `com.winemu.core.regedit.RegistryPatcher` and the relevant
//! parts of `RegistryComparator` (GameHub Lite v5.1.8).
//!
//! The decompiled outer bodies of `RegistryPatcher` and `RegistryComparator`
//! were stripped from the reference (only the data classes and the comparator
//! helper classes survive). The surviving helpers show exactly how changes are
//! ordered before being applied:
//!
//! * `KeyAdded` — `compareBy` the number of `\` in the path (shallow keys
//!   first), then `thenBy` the change's index;
//! * `KeyDeleted` — `compareByDescending` the number of `\` (deepest keys
//!   first), then `thenBy` the index;
//! * `KeyModified`, `ValueAdded`, `ValueModified`, `ValueDeleted` —
//!   `sortedBy` the change's index (stable, insertion order).
//!
//! [`RegPatch::order_changes`] reproduces those comparators and concatenates
//! the groups in a deterministic application order: delete values, delete keys
//! (deepest first), add keys (shallowest first), add values, modify values,
//! modify key properties. A stable sequence number mirrors the map index used
//! by the Kotlin `thenBy`/`sortedBy` helpers, so applying the same patch twice
//! always yields the same result.

use super::key::{split_reg_path, RegHive};
use super::value::RegValue;

/// Key-property changes from `RegistryComparator.KeyPropertyChange`.
///
/// These are carried for completeness; the portable `RegKey` does not store
/// the Win32 metadata flags, so applying a `KeyModified` change records it as
/// applied without mutating the tree.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum KeyPropertyChange {
    /// `ClassNameChange(oldValue, newValue)`.
    ClassNameChange(Option<String>, Option<String>),
    /// `SymlinkChange(oldValue, newValue)`.
    SymlinkChange(bool, bool),
    /// `VolatileChange(oldValue, newValue)`.
    VolatileChange(bool, bool),
}

/// A single registry change, mirroring `RegistryComparator.RegistryChange`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RegChange {
    KeyAdded {
        path: String,
    },
    KeyDeleted {
        path: String,
    },
    KeyModified {
        path: String,
        properties: Vec<KeyPropertyChange>,
    },
    ValueAdded {
        key_path: String,
        value_name: String,
        value: RegValue,
    },
    ValueDeleted {
        key_path: String,
        value_name: String,
        value: RegValue,
    },
    ValueModified {
        key_path: String,
        value_name: String,
        old: RegValue,
        new: RegValue,
    },
}

impl RegChange {
    /// Path of the key this change targets (equal to `key_path` for value
    /// changes), used by the depth comparators.
    pub fn target_path(&self) -> &str {
        match self {
            RegChange::KeyAdded { path }
            | RegChange::KeyDeleted { path }
            | RegChange::KeyModified { path, .. } => path,
            RegChange::ValueAdded { key_path, .. }
            | RegChange::ValueDeleted { key_path, .. }
            | RegChange::ValueModified { key_path, .. } => key_path,
        }
    }

    /// Number of `\` separators in the target path — the key used by the
    /// decompiled `compareBy` / `compareByDescending` comparators.
    pub fn depth(&self) -> usize {
        self.target_path().matches('\\').count()
    }

    fn group(&self) -> u8 {
        match self {
            RegChange::ValueDeleted { .. } => 0,
            RegChange::KeyDeleted { .. } => 1,
            RegChange::KeyAdded { .. } => 2,
            RegChange::ValueAdded { .. } => 3,
            RegChange::ValueModified { .. } => 4,
            RegChange::KeyModified { .. } => 5,
        }
    }
}

/// Options mirroring `RegistryPatcher.PatchOptions`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PatchOptions {
    /// `ignoreFailures` — continue applying after a failed change.
    pub ignore_failures: bool,
    /// `createMissingKeys` — create a key that a value change refers to.
    pub create_missing_keys: bool,
    /// `overwriteExistingValues` — allow `ValueAdded` to replace a value.
    pub overwrite_existing_values: bool,
    /// `deleteEmptyKeys` — allow `KeyDeleted` to remove a non-empty subtree.
    pub delete_empty_keys: bool,
    /// `validateBeforeApply` — documented for parity; the portable model has
    /// no external validator, so this only affects report bookkeeping.
    pub validate_before_apply: bool,
}

impl Default for PatchOptions {
    fn default() -> Self {
        PatchOptions {
            ignore_failures: false,
            create_missing_keys: true,
            overwrite_existing_values: true,
            delete_empty_keys: true,
            validate_before_apply: false,
        }
    }
}

/// A failed change and the reason it could not be applied, mirroring
/// `RegistryPatcher.PatchFailure` (the Java `Exception` is represented as a
/// plain message string).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PatchFailure {
    pub change: RegChange,
    pub reason: String,
}

/// Report returned by [`RegPatch::apply`], mirroring
/// `RegistryPatcher.PatchResult` / `RegistryResult` at the patch layer.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RegResult {
    /// `true` when every change was applied (no failures).
    pub success: bool,
    /// Changes that were applied, in the order they were applied.
    pub applied: Vec<RegChange>,
    /// Changes that failed, in application order.
    pub failures: Vec<PatchFailure>,
}

impl RegResult {
    pub fn new(applied: Vec<RegChange>, failures: Vec<PatchFailure>) -> Self {
        RegResult {
            success: failures.is_empty(),
            applied,
            failures,
        }
    }
}

/// A collection of changes to apply to a [`RegHive`].
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct RegPatch {
    pub changes: Vec<RegChange>,
}

impl RegPatch {
    pub fn new() -> Self {
        RegPatch {
            changes: Vec::new(),
        }
    }

    pub fn from_changes(changes: Vec<RegChange>) -> Self {
        RegPatch { changes }
    }

    /// Add a value at `key_path` (`""` name means the default value).
    pub fn add_value(
        &mut self,
        key_path: impl Into<String>,
        name: impl Into<String>,
        value: RegValue,
    ) {
        self.changes.push(RegChange::ValueAdded {
            key_path: key_path.into(),
            value_name: name.into(),
            value,
        });
    }

    /// Delete a value at `key_path`, recording its previous payload.
    pub fn delete_value(
        &mut self,
        key_path: impl Into<String>,
        name: impl Into<String>,
        old: RegValue,
    ) {
        self.changes.push(RegChange::ValueDeleted {
            key_path: key_path.into(),
            value_name: name.into(),
            value: old,
        });
    }

    /// Modify a value at `key_path`, recording both old and new payloads.
    pub fn modify_value(
        &mut self,
        key_path: impl Into<String>,
        name: impl Into<String>,
        old: RegValue,
        new: RegValue,
    ) {
        self.changes.push(RegChange::ValueModified {
            key_path: key_path.into(),
            value_name: name.into(),
            old,
            new,
        });
    }

    pub fn add_key(&mut self, path: impl Into<String>) {
        self.changes.push(RegChange::KeyAdded { path: path.into() });
    }

    pub fn delete_key(&mut self, path: impl Into<String>) {
        self.changes
            .push(RegChange::KeyDeleted { path: path.into() });
    }

    /// Return the changes in deterministic application order. See the module
    /// docs for the comparator reconstruction.
    pub fn order_changes(&self) -> Vec<RegChange> {
        let indexed: Vec<(usize, &RegChange)> = self.changes.iter().enumerate().collect();
        let mut indexed = indexed;
        indexed.sort_by(|(ia, a), (ib, b)| {
            let ga = a.group();
            let gb = b.group();
            if ga != gb {
                return ga.cmp(&gb);
            }
            // KeyAdded: shallow first. KeyDeleted: deepest first. Both then
            // fall back to the original index, matching `thenBy`.
            let da = if matches!(a, RegChange::KeyDeleted { .. }) {
                usize::MAX - a.depth()
            } else {
                a.depth()
            };
            let db = if matches!(b, RegChange::KeyDeleted { .. }) {
                usize::MAX - b.depth()
            } else {
                b.depth()
            };
            da.cmp(&db).then(ia.cmp(ib))
        });
        indexed.into_iter().map(|(_, c)| c.clone()).collect()
    }

    /// Apply with default options.
    pub fn apply(&self, hive: &mut RegHive) -> RegResult {
        self.apply_with_options(hive, PatchOptions::default())
    }

    /// Apply all changes in deterministic order, collecting successes and
    /// failures into a [`RegResult`].
    pub fn apply_with_options(&self, hive: &mut RegHive, options: PatchOptions) -> RegResult {
        let mut applied = Vec::new();
        let mut failures = Vec::new();

        for change in self.order_changes() {
            let outcome: Result<(), String> = match &change {
                RegChange::ValueAdded {
                    key_path,
                    value_name,
                    value,
                } => {
                    if hive.lookup(key_path).is_none() && !options.create_missing_keys {
                        Err(format!("missing key: {key_path}"))
                    } else if !options.overwrite_existing_values
                        && hive
                            .lookup(key_path)
                            .map_or(false, |k| k.get_value(value_name).is_some())
                    {
                        Err(format!("value already exists: {key_path}\\{value_name}"))
                    } else {
                        hive.ensure_key(key_path)
                            .insert_value(value_name.clone(), value.clone());
                        Ok(())
                    }
                }
                RegChange::ValueDeleted {
                    key_path,
                    value_name,
                    ..
                } => match hive.delete_value(key_path, value_name) {
                    Some(_) => Ok(()),
                    None => Err(format!("value not found: {key_path}\\{value_name}")),
                },
                RegChange::ValueModified {
                    key_path,
                    value_name,
                    new,
                    ..
                } => match hive.lookup_mut(key_path) {
                    Some(key) if key.get_value(value_name).is_some() => {
                        key.insert_value(value_name.clone(), new.clone());
                        Ok(())
                    }
                    _ => Err(format!("value not found: {key_path}\\{value_name}")),
                },
                RegChange::KeyAdded { path } => {
                    if split_reg_path(path).is_empty() {
                        Err("cannot add the hive root".into())
                    } else {
                        hive.ensure_key(path);
                        Ok(())
                    }
                }
                RegChange::KeyDeleted { path } => {
                    if hive.delete_key(path, options.delete_empty_keys) {
                        Ok(())
                    } else {
                        Err(format!("key not found or non-empty: {path}"))
                    }
                }
                RegChange::KeyModified { path, .. } => {
                    if hive.lookup(path).is_some() {
                        // Metadata flags are not part of the portable model.
                        Ok(())
                    } else {
                        Err(format!("key not found: {path}"))
                    }
                }
            };

            match outcome {
                Ok(()) => applied.push(change),
                Err(reason) => {
                    if !options.ignore_failures {
                        failures.push(PatchFailure { change, reason });
                    }
                }
            }
        }

        RegResult::new(applied, failures)
    }
}

#[cfg(test)]
mod tests {
    use super::{KeyPropertyChange, PatchOptions, RegChange, RegPatch};
    use super::super::key::RegHive;
    use super::super::value::RegValue;

    #[test]
    fn depth_counts_backslashes() {
        let a = RegChange::KeyAdded { path: "\\A\\B".into() };
        let b = RegChange::ValueAdded {
            key_path: "\\A".into(),
            value_name: "v".into(),
            value: RegValue::None,
        };
        assert_eq!(a.depth(), 2);
        assert_eq!(b.depth(), 1);
        assert_eq!(a.target_path(), "\\A\\B");
    }

    #[test]
    fn ordering_mirrors_comparators() {
        let mut patch = RegPatch::new();
        patch.add_key("\\A\\B\\C"); // depth 3
        patch.add_key("\\A"); // depth 1
        patch.add_key("\\A\\B"); // depth 2
        patch.delete_key("\\A\\B\\C\\D"); // deepest deleted first
        patch.delete_key("\\A\\B");
        patch.add_value("\\A", "x", RegValue::Dword(1));
        patch.delete_value("\\A", "y", RegValue::None);
        patch.modify_value("\\A", "z", RegValue::None, RegValue::Dword(2));

        let ordered = patch.order_changes();
        let kinds: Vec<&str> = ordered
            .iter()
            .map(|c| match c {
                RegChange::ValueDeleted { .. } => "vdel",
                RegChange::KeyDeleted { .. } => "kdel",
                RegChange::KeyAdded { .. } => "kadd",
                RegChange::ValueAdded { .. } => "vadd",
                RegChange::ValueModified { .. } => "vmod",
                RegChange::KeyModified { .. } => "kmod",
            })
            .collect();
        assert_eq!(
            kinds,
            vec!["vdel", "kdel", "kdel", "kadd", "kadd", "kadd", "vadd", "vmod"]
        );

        // KeyAdded shallow -> deep, KeyDeleted deep -> shallow.
        let added: Vec<&RegChange> = ordered
            .iter()
            .filter(|c| matches!(c, RegChange::KeyAdded { .. }))
            .collect();
        assert_eq!(added[0].target_path(), "\\A");
        assert_eq!(added[1].target_path(), "\\A\\B");
        assert_eq!(added[2].target_path(), "\\A\\B\\C");
        let deleted: Vec<&RegChange> = ordered
            .iter()
            .filter(|c| matches!(c, RegChange::KeyDeleted { .. }))
            .collect();
        assert_eq!(deleted[0].target_path(), "\\A\\B\\C\\D");
        assert_eq!(deleted[1].target_path(), "\\A\\B");
    }

    #[test]
    fn apply_adds_modifies_and_deletes() {
        let mut hive = RegHive::new();
        hive.insert_value("Software\\Foo", "Old", RegValue::String("old".into()));
        hive.insert_value("Software\\Foo", "Gone", RegValue::Dword(9));

        let mut patch = RegPatch::new();
        patch.add_value("Software\\Foo", "New", RegValue::Dword(1));
        patch.modify_value(
            "Software\\Foo",
            "Old",
            RegValue::String("old".into()),
            RegValue::String("new".into()),
        );
        patch.delete_value("Software\\Foo", "Gone", RegValue::Dword(9));

        let result = patch.apply(&mut hive);
        assert!(result.success);
        assert_eq!(result.applied.len(), 3);
        assert!(result.failures.is_empty());

        assert_eq!(hive.get_value("Software\\Foo", "New"), Some(&RegValue::Dword(1)));
        assert_eq!(
            hive.get_value("Software\\Foo", "Old"),
            Some(&RegValue::String("new".into()))
        );
        assert_eq!(hive.get_value("Software\\Foo", "Gone"), None);
        // apply() is idempotent for the final state after a second run would
        // fail on the now-missing "Gone" value; verify a fresh patch is clean.
        assert!(!patch.apply(&mut hive).success);
    }

    #[test]
    fn apply_respects_options_and_reports_failures() {
        let mut hive = RegHive::new();
        let mut patch = RegPatch::new();
        patch.add_value("Missing", "v", RegValue::Dword(1));

        let strict = PatchOptions {
            create_missing_keys: false,
            ..PatchOptions::default()
        };
        let result = patch.apply_with_options(&mut hive, strict);
        assert!(!result.success);
        assert_eq!(result.applied.len(), 0);
        assert_eq!(result.failures.len(), 1);
        assert!(hive.lookup("Missing").is_none());

        // With ignore_failures the change is skipped but no failure is stored.
        let lenient = PatchOptions {
            create_missing_keys: false,
            ignore_failures: true,
            ..PatchOptions::default()
        };
        let result = patch.apply_with_options(&mut hive, lenient);
        assert!(result.success);
        assert!(result.applied.is_empty());
    }

    #[test]
    fn key_property_change_is_no_op_but_reported() {
        let mut hive = RegHive::new();
        hive.ensure_key("A");
        let patch = RegPatch::from_changes(vec![RegChange::KeyModified {
            path: "\\A".into(),
            properties: vec![KeyPropertyChange::ClassNameChange(None, Some("C".into()))],
        }]);
        let result = patch.apply(&mut hive);
        assert!(result.success);
        assert_eq!(result.applied.len(), 1);
    }
}
