//! Generic `old_key -> new_key` migration for a [`KvStore`].
//!
//! The decompiled sources contain no explicit runtime migration routine. The
//! only old/new name relationship found is between the Kotlin *property*
//! names in `AppPreferences` and the *stored* key strings they map to
//! (`deviceUniqueId` -> `uniqueId`, `lastScrollPositionByMyGames` ->
//! `lastMyGamesScrollPosition` — see `AppPreferences.java` lines 94 and 301).
//! [`LEGACY_KEY_MAP`] records those two pairs; the helpers themselves are
//! generic so callers can supply any mapping.

use super::kv::KvStore;

/// Documented property-name -> stored-key pairs from `AppPreferences`.
pub const LEGACY_KEY_MAP: &[(&str, &str)] = &[
    ("deviceUniqueId", "uniqueId"),
    ("lastScrollPositionByMyGames", "lastMyGamesScrollPosition"),
];

/// Copy the value at `old` to `new`, then delete `old`.
///
/// Returns `true` when `old` was present. `new` is written only when it is
/// absent (an existing new value always wins); `old` is removed in either case
/// so the store is left with a single canonical key.
pub fn migrate<S: KvStore>(store: &mut S, old: &str, new: &str) -> bool {
    if old == new {
        return false;
    }
    let Some(value) = store.get(old) else {
        return false;
    };
    if !store.contains(new) {
        store.set(new, value);
    }
    store.remove(old);
    true
}

/// Apply [`LEGACY_KEY_MAP`] to `store`; return how many old keys were migrated.
pub fn migrate_all<S: KvStore>(store: &mut S) -> usize {
    LEGACY_KEY_MAP
        .iter()
        .filter(|(old, new)| migrate(store, old, new))
        .count()
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::super::InMemoryKv;

    #[test]
    fn migrates_value_and_removes_old_key() {
        let mut kv = InMemoryKv::new();
        kv.set_string("deviceUniqueId", "legacy-id");
        assert!(migrate(&mut kv, "deviceUniqueId", "uniqueId"));
        assert!(!kv.contains("deviceUniqueId"));
        assert_eq!(kv.get_string("uniqueId", ""), "legacy-id");
    }

    #[test]
    fn no_op_when_old_key_absent() {
        let mut kv = InMemoryKv::new();
        assert!(!migrate(&mut kv, "missing", "uniqueId"));
        assert!(kv.is_empty());
    }

    #[test]
    fn existing_new_value_wins_but_old_is_removed() {
        let mut kv = InMemoryKv::new();
        kv.set_string("old", "stale");
        kv.set_string("new", "canonical");
        assert!(migrate(&mut kv, "old", "new"));
        assert_eq!(kv.get_string("new", ""), "canonical");
        assert!(!kv.contains("old"));
    }

    #[test]
    fn identical_keys_do_nothing() {
        let mut kv = InMemoryKv::new();
        kv.set_i64("k", 5);
        assert!(!migrate(&mut kv, "k", "k"));
        assert_eq!(kv.get_i64("k", 0), 5);
    }

    #[test]
    fn migrate_all_applies_documented_pairs() {
        let mut kv = InMemoryKv::new();
        kv.set_string("deviceUniqueId", "id");
        kv.set_i64("lastScrollPositionByMyGames", 9);
        assert_eq!(migrate_all(&mut kv), 2);
        assert_eq!(kv.get_string("uniqueId", ""), "id");
        assert_eq!(kv.get_i64("lastMyGamesScrollPosition", -1), 9);
        assert_eq!(migrate_all(&mut kv), 0); // idempotent
    }
}
