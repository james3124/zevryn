//! In-memory cache store with an injectable clock, TTL expiry and
//! stale-while-revalidate reads.
//!
//! Keys are `String` (URLs / game IDs / Room row keys); payloads are generic
//! `V` and default to raw `String`. The backing map is a `HashMap`; no disk or
//! network I/O occurs. The injectable `now` makes expiry and staleness fully
//! deterministic in tests.

use std::collections::HashMap;

use super::cache_entry::{system_now_secs, CacheEntry, CacheKind};

/// A `HashMap`-backed cache keyed by `String`.
///
/// `V` defaults to `String` so `CacheStore::new()` can be used as a raw
/// response cache; pass e.g. `CacheStore<Vec<u8>>` for icon bytes.
pub struct CacheStore<V = String> {
    entries: HashMap<String, CacheEntry<String, V>>,
    now: Box<dyn Fn() -> u64>,
}

impl<V: std::fmt::Debug> std::fmt::Debug for CacheStore<V> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CacheStore")
            .field("entries", &self.entries)
            .field("now", &"<closure>")
            .finish()
    }
}

impl<V> Default for CacheStore<V> {
    fn default() -> Self {
        Self::new()
    }
}

impl<V> CacheStore<V> {
    /// Store using the wall clock as its time source.
    pub fn new() -> Self {
        Self::with_clock(system_now_secs)
    }

    /// Store with a caller-supplied clock returning Unix seconds.
    ///
    /// The closure is boxed so callers may capture mutable-tuned fixtures
    /// (e.g. an `Rc<Cell<u64>>`) and advance time deterministically.
    pub fn with_clock<F>(clock: F) -> Self
    where
        F: Fn() -> u64 + 'static,
    {
        Self {
            entries: HashMap::new(),
            now: Box::new(clock),
        }
    }

    /// Current time reported by the injected clock.
    pub fn now(&self) -> u64 {
        (self.now)()
    }

    /// Insert a fully-formed entry, returning the previous value for that key.
    pub fn put(&mut self, entry: CacheEntry<String, V>) -> Option<CacheEntry<String, V>> {
        let key = entry.key.clone();
        self.entries.insert(key, entry)
    }

    /// Convenience insert: builds an entry stamped with the current clock.
    pub fn insert(
        &mut self,
        key: impl Into<String>,
        kind: CacheKind,
        ttl_secs: u64,
        payload: V,
    ) -> Option<CacheEntry<String, V>> {
        let now = self.now();
        self.put(CacheEntry::new(key.into(), kind, now, ttl_secs, payload))
    }

    /// Raw lookup that ignores expiry (used to inspect stale entries).
    pub fn peek(&self, key: &str) -> Option<&CacheEntry<String, V>> {
        self.entries.get(key)
    }

    /// Fresh-only read: returns the payload iff the key exists and has not
    /// expired at the current clock value.
    pub fn get(&self, key: &str) -> Option<&V> {
        let now = self.now();
        let entry = self.entries.get(key)?;
        if entry.is_expired_at(now) {
            None
        } else {
            Some(&entry.payload)
        }
    }

    /// Fresh-only read that also returns the full entry (kind, timestamp, TTL).
    pub fn get_entry(&self, key: &str) -> Option<&CacheEntry<String, V>> {
        let now = self.now();
        let entry = self.entries.get(key)?;
        if entry.is_expired_at(now) {
            None
        } else {
            Some(entry)
        }
    }

    /// Stale-while-revalidate read.
    ///
    /// Returns `(value, is_stale)` for any stored key: `is_stale == false`
    /// means the entry is within its TTL; `is_stale == true` means it is past
    /// its TTL but can still be shown to the user while a refresh is requested
    /// (e.g. a cached `GameDetailEntity` DTO). Returns `None` only when the key
    /// is absent.
    pub fn get_with_policy(&self, key: &str) -> Option<(&V, bool)> {
        let now = self.now();
        let entry = self.entries.get(key)?;
        Some((&entry.payload, entry.is_expired_at(now)))
    }

    /// Stale-while-revalidate read with full entry metadata.
    pub fn entry_with_policy(&self, key: &str) -> Option<(&CacheEntry<String, V>, bool)> {
        let now = self.now();
        let entry = self.entries.get(key)?;
        Some((entry, entry.is_expired_at(now)))
    }

    /// Whether a key is present but past its TTL at the current clock value.
    /// Absent keys return `false`.
    pub fn is_expired(&self, key: &str) -> bool {
        let now = self.now();
        self.entries
            .get(key)
            .map(|e| e.is_expired_at(now))
            .unwrap_or(false)
    }

    /// Remove a key, returning the removed entry (fresh or stale).
    pub fn invalidate(&mut self, key: &str) -> Option<CacheEntry<String, V>> {
        self.entries.remove(key)
    }

    /// Remove a key **only if it is currently expired**.
    /// Returns `true` when something was removed.
    pub fn expire(&mut self, key: &str) -> bool {
        let now = self.now();
        let expired = self
            .entries
            .get(key)
            .map(|e| e.is_expired_at(now))
            .unwrap_or(false);
        if expired {
            self.entries.remove(key);
        }
        expired
    }

    /// Remove every expired entry; returns the number removed.
    pub fn purge_expired(&mut self) -> usize {
        let now = self.now();
        let before = self.entries.len();
        self.entries.retain(|_, e| !e.is_expired_at(now));
        before - self.entries.len()
    }

    pub fn contains_key(&self, key: &str) -> bool {
        self.entries.contains_key(key)
    }

    pub fn len(&self) -> usize {
        self.entries.len()
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    pub fn clear(&mut self) {
        self.entries.clear();
    }

    /// Iterate over all raw entries (including stale ones), unspecified order.
    pub fn iter(&self) -> impl Iterator<Item = &CacheEntry<String, V>> {
        self.entries.values()
    }
}

/// Alias for the common raw-text cache used by DTO/response payloads.
pub type StringCacheStore = CacheStore<String>;
