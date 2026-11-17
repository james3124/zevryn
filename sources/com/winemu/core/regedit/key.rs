//! Rust model of `com.winemu.core.regedit.RegistryKey` and an in-memory hive
//! (GameHub Lite v5.1.8).
//!
//! The decompiled `RegistryKey` is a tree node with a name, a parent, a sorted
//! map of child keys (`TreeMap`, field `f20664h`), a sorted map of values
//! (`TreeMap`, field `f20665i`) and a few metadata flags. Its portable methods
//! are:
//!
//! * `b(path)` / `a(path)` — get-or-create a key by a `\`-separated path;
//! * `g(path)` — look up a key by path, returning `null` if absent;
//! * `e(name, recursive)` / `f(name)` — delete a subkey / a value;
//! * `y(name, value)` / `z(name, value)` — insert a value;
//! * `p(name)` — read a value.
//!
//! Here the parent link is implicit in the nesting and each node stores its
//! full `path`. `BTreeMap` gives the same deterministic, case-sensitive
//! ordering as the Java `TreeMap` (which used `String.CASE_INSENSITIVE_ORDER`
//! for lookup only; the portable model uses exact names). The "dirty" and
//! volatile/symlink flags plus the real Windows registry backend are out of
//! scope — only the tree shape and its values are modeled.

use std::collections::BTreeMap;

use super::value::RegValue;

/// Split a `\`-separated registry path into non-empty components. Leading,
/// trailing and repeated separators are ignored, matching the filter in
/// `RegistryKey.a(path)` / `g(path)`.
pub fn split_reg_path(path: &str) -> Vec<String> {
    path.split('\\')
        .filter(|s| !s.is_empty())
        .map(str::to_owned)
        .collect()
}

/// Join non-empty path components into a canonical `\`-separated path with a
/// single leading backslash (e.g. `Software\Foo` -> `\Software\Foo`).
pub fn canonical_path(path: &str) -> String {
    let parts = split_reg_path(path);
    if parts.is_empty() {
        String::new()
    } else {
        format!("\\{}", parts.join("\\"))
    }
}

/// One node of the registry tree.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RegKey {
    /// Full canonical path of this key from the hive root, e.g.
    /// `\REGISTRY\Software\Wine`. The root itself is `\REGISTRY` (mirroring
    /// `RegistryKey.Companion.a()` in the reference).
    pub path: String,
    /// Values attached to this key, keyed by value name (`""` is the default
    /// value, written as `@=` in a `.reg` file).
    pub values: BTreeMap<String, RegValue>,
    /// Direct child keys, keyed by their last path component.
    pub subkeys: BTreeMap<String, RegKey>,
}

impl RegKey {
    /// Create an empty key. `path` is stored verbatim; use [`canonical_path`]
    /// first if a normalized path is desired.
    pub fn new(path: impl Into<String>) -> Self {
        RegKey {
            path: path.into(),
            values: BTreeMap::new(),
            subkeys: BTreeMap::new(),
        }
    }

    /// Last path component (the key's name), or the whole path if it has no
    /// separator.
    pub fn name(&self) -> &str {
        self.path.rsplit('\\').find(|s| !s.is_empty()).unwrap_or(&self.path)
    }

    /// Insert or replace a value, mirroring `RegistryKey.y(name, value)`.
    pub fn insert_value(&mut self, name: impl Into<String>, value: RegValue) {
        self.values.insert(name.into(), value);
    }

    /// Read a value by name, mirroring `RegistryKey.p(name)`.
    pub fn get_value(&self, name: &str) -> Option<&RegValue> {
        self.values.get(name)
    }

    /// Remove and return a value by name, mirroring `RegistryKey.f(name)`.
    pub fn remove_value(&mut self, name: &str) -> Option<RegValue> {
        self.values.remove(name)
    }

    /// Direct subkey lookup by name, mirroring `RegistryKey.n(name)`.
    pub fn get_subkey(&self, name: &str) -> Option<&RegKey> {
        self.subkeys.get(name)
    }

    /// Mutable direct subkey lookup by name.
    pub fn get_subkey_mut(&mut self, name: &str) -> Option<&mut RegKey> {
        self.subkeys.get_mut(name)
    }

    /// Walk a path relative to this key without creating anything, mirroring
    /// `RegistryKey.g(path)`. Returns `None` at the first missing component.
    pub fn lookup(&self, path: &str) -> Option<&RegKey> {
        let mut cur = self;
        for part in split_reg_path(path) {
            cur = cur.subkeys.get(&part)?;
        }
        Some(cur)
    }

    /// Walk a path, creating missing keys, mirroring `RegistryKey.a(path)` /
    /// `b(path)`. Returns a mutable reference to the final node.
    pub fn ensure_subkey(&mut self, path: &str) -> &mut RegKey {
        let mut cur = self;
        for part in split_reg_path(path) {
            let child_path = {
                let base = canonical_path(&cur.path);
                if base.is_empty() {
                    format!("\\{part}")
                } else {
                    format!("{base}\\{part}")
                }
            };
            cur = cur
                .subkeys
                .entry(part)
                .or_insert_with(|| RegKey::new(child_path));
        }
        cur
    }
}

/// In-memory registry hive: a single root key. This is the portable
/// counterpart of the `RegistryKey("\\REGISTRY", ...)` root built by
/// `RegistryParser` (`Companion.a()`), populated through its get-or-create
/// path helpers.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RegHive {
    /// Root of the tree; always has path `\REGISTRY`.
    pub root: RegKey,
}

impl Default for RegHive {
    fn default() -> Self {
        Self::new()
    }
}

impl RegHive {
    /// Build an empty hive with a `\REGISTRY` root.
    pub fn new() -> Self {
        RegHive {
            root: RegKey::new("\\REGISTRY"),
        }
    }

    /// Look up a key by an absolute-or-relative `\`-separated path.
    pub fn lookup(&self, path: &str) -> Option<&RegKey> {
        self.root.lookup(path)
    }

    /// Mutable lookup of an existing key.
    pub fn lookup_mut(&mut self, path: &str) -> Option<&mut RegKey> {
        let mut cur = &mut self.root;
        for part in split_reg_path(path) {
            cur = cur.subkeys.get_mut(&part)?;
        }
        Some(cur)
    }

    /// Get-or-create a key at `path`, mirroring `RegistryKey.b(path)`.
    pub fn ensure_key(&mut self, path: &str) -> &mut RegKey {
        self.root.ensure_subkey(path)
    }

    /// Insert a value at `path`, creating the key if needed. Returns `false`
    /// only when the path resolves to the root, which has no values in this
    /// model (mirrors the way the parser requires `[key]` before values).
    pub fn insert_value(
        &mut self,
        path: &str,
        name: impl Into<String>,
        value: RegValue,
    ) -> bool {
        if split_reg_path(path).is_empty() {
            return false;
        }
        self.ensure_key(path).insert_value(name, value);
        true
    }

    /// Read a value at `path`.
    pub fn get_value(&self, path: &str, name: &str) -> Option<&RegValue> {
        self.lookup(path)?.get_value(name)
    }

    /// Remove a value at `path`, returning the old value on success. Mirrors
    /// `RegistryKey.f(name)`.
    pub fn delete_value(&mut self, path: &str, name: &str) -> Option<RegValue> {
        self.lookup_mut(path)?.remove_value(name)
    }

    /// Delete a key at `path`. When `recursive` is `false` a key that still has
    /// subkeys is left untouched, mirroring `RegistryKey.e(name, false)`.
    /// Returns `false` for the root or a missing/non-empty key.
    pub fn delete_key(&mut self, path: &str, recursive: bool) -> bool {
        let parts = split_reg_path(path);
        if parts.is_empty() {
            return false;
        }
        let (parent_parts, leaf) = parts.split_at(parts.len() - 1);
        let leaf = &leaf[0];
        let mut cur = &mut self.root;
        for part in parent_parts {
            match cur.subkeys.get_mut(part) {
                Some(k) => cur = k,
                None => return false,
            }
        }
        if !recursive && cur.subkeys.get(leaf).map_or(false, |k| !k.subkeys.is_empty()) {
            return false;
        }
        cur.subkeys.remove(leaf).is_some()
    }

    /// Every key in the hive, in deterministic depth-first path order
    /// (`BTreeMap` iteration), including the root. Useful for serializers and
    /// patch application.
    pub fn keys(&self) -> Vec<&RegKey> {
        fn walk<'a>(key: &'a RegKey, out: &mut Vec<&'a RegKey>) {
            out.push(key);
            for child in key.subkeys.values() {
                walk(child, out);
            }
        }
        let mut out = Vec::new();
        walk(&self.root, &mut out);
        out
    }
}

#[cfg(test)]
mod tests {
    use super::{canonical_path, split_reg_path, RegHive};
    use super::super::value::RegValue;

    #[test]
    fn path_helpers() {
        assert_eq!(split_reg_path("\\Software\\Wine\\"), vec!["Software", "Wine"]);
        assert_eq!(split_reg_path(""), Vec::<String>::new());
        assert_eq!(canonical_path("Software\\\\Wine"), "\\Software\\Wine");
    }

    #[test]
    fn tree_insert_lookup_delete() {
        let mut hive = RegHive::new();
        assert_eq!(hive.root.path, "\\REGISTRY");
        assert!(hive.insert_value("Software\\Wine", "Version", RegValue::String("9.0".into())));
        assert!(hive.insert_value("Software\\Wine\\Config", "D", RegValue::Dword(7)));

        let key = hive.lookup("Software\\Wine").unwrap();
        assert_eq!(key.get_value("Version"), Some(&RegValue::String("9.0".into())));
        assert_eq!(key.name(), "Wine");
        assert_eq!(
            hive.get_value("Software\\Wine\\Config", "D"),
            Some(&RegValue::Dword(7))
        );

        // Get-or-create must not duplicate existing keys.
        hive.ensure_key("Software\\Wine\\Config");
        assert_eq!(hive.keys().len(), 4); // root, Software, Wine, Config

        assert_eq!(hive.delete_value("Software\\Wine", "Version"), Some(RegValue::String("9.0".into())));
        assert_eq!(hive.get_value("Software\\Wine", "Version"), None);
    }

    #[test]
    fn delete_key_respects_children() {
        let mut hive = RegHive::new();
        hive.ensure_key("A\\B\\C");
        assert!(!hive.delete_key("A", false)); // A still has B
        assert!(!hive.delete_key("Missing", true));
        assert!(hive.delete_key("A", true));
        assert!(hive.lookup("A").is_none());
        assert!(!hive.delete_key("", true)); // cannot delete root
    }
}
