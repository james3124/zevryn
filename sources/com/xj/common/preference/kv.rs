//! Minimal, dependency-free key/value store abstraction.
//!
//! Models the three scalar accessors the decompiled code actually uses on
//! `SharedPreferences`/`MMKV`: strings, booleans and integers. Floats
//! (`wineScreenBrightness`) are carried as strings in this model because the
//! store's primitives are string/bool/i64 only; see `get_f64`/`set_f64`.

use std::collections::HashMap;

/// A single persisted scalar value.
#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    Str(String),
    Bool(bool),
    Int(i64),
}

impl Value {
    /// Interpret this value as a string, using `default` for a type mismatch.
    pub fn as_str_or<'a>(&'a self, default: &'a str) -> &'a str {
        match self {
            Value::Str(s) => s.as_str(),
            _ => default,
        }
    }

    /// Interpret this value as a bool, using `default` for a type mismatch.
    pub fn as_bool_or(&self, default: bool) -> bool {
        match self {
            Value::Bool(b) => *b,
            _ => default,
        }
    }

    /// Interpret this value as an integer, using `default` for a type mismatch.
    pub fn as_i64_or(&self, default: i64) -> i64 {
        match self {
            Value::Int(i) => *i,
            _ => default,
        }
    }
}

/// Read/write typed values by string key.
///
/// Mirrors the surface used by the Kotlin `field` delegates in
/// `AppPreferences`: `contains`, `getString/getBoolean/getLong` and
/// `putString/putBoolean/putLong` (with `apply()` being a no-op in memory).
pub trait KvStore {
    /// Raw value for `key`, if present.
    fn get(&self, key: &str) -> Option<Value>;

    /// Store a raw value under `key`.
    fn set(&mut self, key: &str, value: Value);

    /// Delete `key`.
    fn remove(&mut self, key: &str);

    /// Whether `key` is present (Java `SharedPreferences.contains` / `MMKV.contains`).
    fn contains(&self, key: &str) -> bool {
        self.get(key).is_some()
    }

    /// Java `getString(key, default)`.
    fn get_string(&self, key: &str, default: &str) -> String {
        self.get(key)
            .map(|v| v.as_str_or(default).to_owned())
            .unwrap_or_else(|| default.to_owned())
    }

    /// Java `getBoolean(key, default)`.
    fn get_bool(&self, key: &str, default: bool) -> bool {
        self.get(key).map(|v| v.as_bool_or(default)).unwrap_or(default)
    }

    /// Java `getLong(key, default)`.
    fn get_i64(&self, key: &str, default: i64) -> i64 {
        self.get(key).map(|v| v.as_i64_or(default)).unwrap_or(default)
    }

    /// Java `putString(key, value)` followed by `apply()`.
    fn set_string(&mut self, key: &str, value: &str) {
        self.set(key, Value::Str(value.to_owned()));
    }

    /// Java `putBoolean(key, value)` followed by `apply()`.
    fn set_bool(&mut self, key: &str, value: bool) {
        self.set(key, Value::Bool(value));
    }

    /// Java `putLong(key, value)` followed by `apply()`.
    fn set_i64(&mut self, key: &str, value: i64) {
        self.set(key, Value::Int(value));
    }

    /// Float accessor for `wineScreenBrightness` (Java `getFloat`, default
    /// `-1.0f`). Encoded as a string because the store has no float primitive.
    fn get_f64(&self, key: &str, default: f64) -> f64 {
        self.get(key)
            .and_then(|v| match v {
                Value::Str(s) => s.parse::<f64>().ok(),
                _ => None,
            })
            .unwrap_or(default)
    }

    /// Float setter paired with [`KvStore::get_f64`].
    fn set_f64(&mut self, key: &str, value: f64) {
        self.set_string(key, &value.to_string());
    }
}

/// In-memory `KvStore` backed by a `HashMap` — the portable stand-in for a
/// `SharedPreferences` file or an MMKV instance.
#[derive(Debug, Default, Clone)]
pub struct InMemoryKv {
    values: HashMap<String, Value>,
}

impl InMemoryKv {
    /// Empty store.
    pub fn new() -> Self {
        Self::default()
    }

    /// Number of stored keys (handy for migration tests).
    pub fn len(&self) -> usize {
        self.values.len()
    }

    /// Whether the store has no keys.
    pub fn is_empty(&self) -> bool {
        self.values.is_empty()
    }
}

impl KvStore for InMemoryKv {
    fn get(&self, key: &str) -> Option<Value> {
        self.values.get(key).cloned()
    }

    fn set(&mut self, key: &str, value: Value) {
        self.values.insert(key.to_owned(), value);
    }

    fn remove(&mut self, key: &str) {
        self.values.remove(key);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn string_round_trip_and_defaults() {
        let mut kv = InMemoryKv::new();
        assert_eq!(kv.get_string("k", "fallback"), "fallback");
        kv.set_string("k", "value");
        assert_eq!(kv.get_string("k", "fallback"), "value");
        assert!(kv.contains("k"));
        assert!(!kv.contains("missing"));
    }

    #[test]
    fn bool_and_int_round_trip() {
        let mut kv = InMemoryKv::new();
        kv.set_bool("flag", true);
        kv.set_i64("count", -1);
        assert!(kv.get_bool("flag", false));
        assert!(!kv.get_bool("absent", false));
        assert_eq!(kv.get_i64("count", 0), -1);
        assert_eq!(kv.get_i64("absent", 7), 7);
    }

    #[test]
    fn float_via_string_and_remove() {
        let mut kv = InMemoryKv::new();
        assert_eq!(kv.get_f64("brightness", -1.0), -1.0);
        kv.set_f64("brightness", 0.75);
        assert_eq!(kv.get_f64("brightness", -1.0), 0.75);
        kv.remove("brightness");
        assert_eq!(kv.get_f64("brightness", -1.0), -1.0);
    }

    #[test]
    fn type_mismatch_uses_default() {
        let mut kv = InMemoryKv::new();
        kv.set_string("x", "not a bool");
        assert!(!kv.get_bool("x", false));
        assert_eq!(kv.get_i64("x", 42), 42);
        assert_eq!(kv.get_string("x", "d"), "not a bool");
    }
}
