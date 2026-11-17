//! Rust model of `com.winemu.core.utils.EnvVars` (GameHub Lite v5.1.8).
//!
//! The Java class is a Kotlin `@Parcelize` data holder wrapping a
//! `LinkedHashMap`, i.e. an **insertion-ordered** string-to-string map. All
//! behavior that does not require Android `Parcel`/`Log` is portable and kept
//! here:
//!
//! | Java method | Rust equivalent |
//! |---|---|
//! | `EnvVars(LinkedHashMap)` / synthetic ctor | [`EnvVars::new`] / [`EnvVars::default`] |
//! | `a(name)` | [`EnvVars::get`] |
//! | `c(name, value.toString())` | [`EnvVars::set`] |
//! | `g(other)` (`putAll`) | [`EnvVars::extend`] |
//! | `h(name, List)` / `i(name, vararg)` | [`EnvVars::set_path`] (empty parts dropped, joined with `:`) |
//! | `toStringArray()` | [`EnvVars::export`] / [`EnvVars::to_string_array`] |
//! | `toString()` | [`EnvVars::to_env_string`] (space-joined `KEY=VALUE`) |
//! | `b()` (debug log) | [`EnvVars::to_export_script`] (shell `export KEY="VALUE"` lines) |
//! | `iterator()` over keys | [`EnvVars::keys`] / `IntoIterator for &EnvVars` |
//!
//! Parcelable `CREATOR`/`writeToParcel`/`describeContents` and the Android
//! `Log.d` side effect are intentionally out of scope.

use std::collections::HashMap;

/// An insertion-ordered environment-variable map.
///
/// Mirrors Java `LinkedHashMap<String, String>` semantics: iterating, exporting
/// and stringifying all preserve the order in which keys were first inserted.
/// Re-setting an existing key updates its value **without** changing its
/// position (exactly like `LinkedHashMap.put`).
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct EnvVars {
    /// Insertion order of keys, kept alongside the lookup map.
    order: Vec<String>,
    values: HashMap<String, String>,
}

impl EnvVars {
    /// Construct an empty map (Java synthetic no-arg constructor).
    pub fn new() -> Self {
        Self::default()
    }

    /// Build from an ordered sequence of `(key, value)` pairs.
    pub fn from_pairs<I, K, V>(pairs: I) -> Self
    where
        I: IntoIterator<Item = (K, V)>,
        K: Into<String>,
        V: ToString,
    {
        let mut env = Self::new();
        for (k, v) in pairs {
            env.set(k, v);
        }
        env
    }

    /// `a(name)` — look up a value by key.
    pub fn get(&self, name: &str) -> Option<&str> {
        self.values.get(name).map(String::as_str)
    }

    /// Java `LinkedHashMap.containsKey`.
    pub fn contains_key(&self, name: &str) -> bool {
        self.values.contains_key(name)
    }

    /// `c(name, value)` — insert or replace a value (`value.toString()` in Java).
    /// Replacing an existing key preserves its original position.
    pub fn set(&mut self, name: impl Into<String>, value: impl ToString) {
        let name = name.into();
        if !self.values.contains_key(&name) {
            self.order.push(name.clone());
        }
        self.values.insert(name, value.to_string());
    }

    /// Alias for [`EnvVars::set`] matching Java's `put` naming used by Box64
    /// config export code.
    pub fn put(&mut self, name: impl Into<String>, value: impl ToString) {
        self.set(name, value);
    }

    /// Remove a key, preserving the order of the remaining keys.
    pub fn remove(&mut self, name: &str) -> Option<String> {
        let removed = self.values.remove(name);
        if removed.is_some() {
            self.order.retain(|k| k != name);
        }
        removed
    }

    /// `g(envVars)` — `putAll`: copy every entry from `other`, preserving this
    /// map's order for existing keys and appending new keys in `other`'s order.
    pub fn extend(&mut self, other: &EnvVars) {
        for key in &other.order {
            if let Some(value) = other.values.get(key) {
                self.set(key.clone(), value.clone());
            }
        }
    }

    /// `h(name, List)` / `i(name, String...)` — drop empty components and join
    /// the rest with `:` (Kotlin `joinToString(":")`). This is how Java builds
    /// `PATH`, `LD_LIBRARY_PATH`, etc.
    pub fn set_path<I, S>(&mut self, name: impl Into<String>, components: I)
    where
        I: IntoIterator<Item = S>,
        S: AsRef<str>,
    {
        let joined = components
            .into_iter()
            .map(|c| c.as_ref().to_owned())
            .filter(|c| !c.is_empty())
            .collect::<Vec<_>>()
            .join(":");
        self.set(name, joined);
    }

    /// `toStringArray()` → ordered `Vec<"KEY=VALUE">`.
    ///
    /// This is the primary `export` shape requested by the Rust model: an
    /// ordered vector of environment entries ready to hand to a launcher.
    pub fn export(&self) -> Vec<String> {
        self.order
            .iter()
            .filter_map(|k| self.values.get(k).map(|v| format!("{}={}", k, v)))
            .collect()
    }

    /// Alias for [`EnvVars::export`] matching the Java method name.
    pub fn to_string_array(&self) -> Vec<String> {
        self.export()
    }

    /// `toString()` → the exported entries joined by a single space.
    pub fn to_env_string(&self) -> String {
        self.export().join(" ")
    }

    /// `b()` → shell-script lines `export KEY="VALUE"` (with `"` escaped as
    /// `\"` in values, as Java's `replace("\"", "\\\"")` does). The Java
    /// original logs these; here they are returned as data.
    pub fn to_export_script(&self) -> Vec<String> {
        self.order
            .iter()
            .filter_map(|k| {
                self.values.get(k).map(|v| {
                    format!("export {}=\"{}\"", k, v.replace('"', "\\\""))
                })
            })
            .collect()
    }

    /// Iterator over keys in insertion order (Java `iterator()` over
    /// `keySet()`).
    pub fn keys(&self) -> impl Iterator<Item = &str> {
        self.order.iter().map(String::as_str)
    }

    /// Iterator over `(key, value)` pairs in insertion order.
    pub fn iter(&self) -> impl Iterator<Item = (&str, &str)> {
        self.order
            .iter()
            .filter_map(move |k| self.values.get(k).map(|v| (k.as_str(), v.as_str())))
    }

    /// Number of variables.
    pub fn len(&self) -> usize {
        self.order.len()
    }

    /// True when no variables are set.
    pub fn is_empty(&self) -> bool {
        self.order.is_empty()
    }

    /// Drop all variables.
    pub fn clear(&mut self) {
        self.order.clear();
        self.values.clear();
    }
}

impl<'a> IntoIterator for &'a EnvVars {
    type Item = (&'a str, &'a str);
    type IntoIter = Box<dyn Iterator<Item = Self::Item> + 'a>;

    fn into_iter(self) -> Self::IntoIter {
        Box::new(self.iter())
    }
}

impl std::fmt::Display for EnvVars {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.to_env_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn insertion_order_is_preserved_by_export() {
        let mut env = EnvVars::new();
        env.set("Z", "last");
        env.set("A", "first");
        env.set("M", "middle");
        assert_eq!(env.export(), vec!["Z=last", "A=first", "M=middle"]);
        assert_eq!(env.keys().collect::<Vec<_>>(), vec!["Z", "A", "M"]);
    }

    #[test]
    fn replacing_a_key_keeps_its_position() {
        let mut env = EnvVars::new();
        env.set("A", "1");
        env.set("B", "2");
        env.set("A", "9");
        assert_eq!(env.export(), vec!["A=9", "B=2"]);
        assert_eq!(env.get("A"), Some("9"));
        assert_eq!(env.len(), 2);
    }

    #[test]
    fn get_contains_remove_and_clear() {
        let mut env = EnvVars::from_pairs([("X", "1"), ("Y", "2")]);
        assert_eq!(env.get("X"), Some("1"));
        assert_eq!(env.get("Y"), Some("2"));
        assert_eq!(env.get("N"), None);
        assert!(env.contains_key("X"));
        assert_eq!(env.remove("X"), Some("1".to_owned()));
        assert_eq!(env.export(), vec!["Y=2"]);
        env.clear();
        assert!(env.is_empty());
    }

    #[test]
    fn extend_preserves_order_and_updates_values() {
        let mut base = EnvVars::from_pairs([("A", "1"), ("B", "2")]);
        let extra = EnvVars::from_pairs([("B", "22"), ("C", "3")]);
        base.extend(&extra);
        assert_eq!(base.export(), vec!["A=1", "B=22", "C=3"]);
    }

    #[test]
    fn set_path_filters_empty_and_joins_with_colon() {
        let mut env = EnvVars::new();
        env.set_path("PATH", ["/bin", "", "/usr/bin", "", "/local/bin"]);
        assert_eq!(env.get("PATH"), Some("/bin:/usr/bin:/local/bin"));
        env.set_path("EMPTY", ["", ""]);
        assert_eq!(env.get("EMPTY"), Some(""));
    }

    #[test]
    fn to_env_string_and_export_script() {
        let mut env = EnvVars::new();
        env.set("BOX64_LOG", "info");
        env.set("MSG", "say \"hi\"");
        assert_eq!(
            env.to_env_string(),
            "BOX64_LOG=info MSG=say \"hi\""
        );
        assert_eq!(
            env.to_export_script(),
            vec!["export BOX64_LOG=\"info\"", "export MSG=\"say \\\"hi\\\"\""]
        );
    }

    #[test]
    fn iter_yields_insertion_order() {
        let env = EnvVars::from_pairs([("B", "2"), ("A", "1")]);
        let pairs: Vec<_> = (&env).into_iter().collect();
        assert_eq!(pairs, vec![("B", "2"), ("A", "1")]);
    }
}
