//! Cache entry model, in-memory store, stale-while-revalidate policy and disk
//! capacity model (`com.xj.common.offline.cache`).
//!
//! std-only, no disk or network I/O. See `REPORT.md` for the reference data
//! sources (Media3 `egg_media3_cache`, Room, OkHttp/ForceCache caches).

mod cache_entry;
mod lru;
mod store;

pub use cache_entry::{system_now_secs, CacheEntry, CacheKind};
pub use lru::{
    LruEvictor, CACHE_DATASOURCE_FLAGS, MEDIA3_CACHE_DIR, MEDIA3_MAX_BYTES, OKHTTP_MAX_BYTES,
};
pub use store::{CacheStore, StringCacheStore};

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::Cell;
    use std::rc::Rc;

    fn clock_at(start: u64) -> (Rc<Cell<u64>>, impl Fn() -> u64) {
        let t = Rc::new(Cell::new(start));
        let handle = Rc::clone(&t);
        (t, move || handle.get())
    }

    // -----------------------------------------------------------------------
    // CacheEntry / expiry
    // -----------------------------------------------------------------------

    #[test]
    fn cache_entry_expiry_is_exclusive_and_overflow_safe() {
        let entry = CacheEntry::new("game:1".to_string(), CacheKind::GameDetail, 100, 10, "x");
        assert_eq!(entry.expires_at(), 110);
        assert!(entry.is_fresh_at(109));
        assert!(entry.is_expired_at(110));
        assert!(entry.is_expired_at(111));
        assert_eq!(entry.remaining_secs(105), 5);
        assert_eq!(entry.remaining_secs(110), 0);

        let forever = CacheEntry::new("k", CacheKind::GameConfig, u64::MAX - 1, u64::MAX, "v");
        assert_eq!(forever.expires_at(), u64::MAX);
        assert!(forever.is_fresh_at(u64::MAX - 1));
        assert!(forever.is_expired_at(u64::MAX));

        let zero = CacheEntry::new("k", CacheKind::ApiResponse, 50, 0, "v");
        assert!(zero.is_expired_at(50));
    }

    #[test]
    fn refresh_replaces_payload_and_resets_timestamp() {
        let mut entry = CacheEntry::new("k", CacheKind::GameIcon, 10, 5, "old");
        assert!(entry.is_expired_at(15));
        entry.refresh("new", 100);
        assert_eq!(entry.payload, "new");
        assert_eq!(entry.stored_at, 100);
        assert!(entry.is_fresh_at(100));
        assert!(entry.is_expired_at(105));
    }

    #[test]
    fn cache_kind_tokens() {
        assert_eq!(
            CacheKind::ALL.map(|k| k.as_str()),
            ["game_detail", "game_icon", "game_config", "api_response"]
        );
    }

    // -----------------------------------------------------------------------
    // CacheStore: put/get/expire/invalidate with injected clock
    // -----------------------------------------------------------------------

    #[test]
    fn put_get_expire_and_invalidate() {
        let (time, clock) = clock_at(1_000);
        let mut store: CacheStore = CacheStore::with_clock(clock);

        store.insert("detail:7", CacheKind::GameDetail, 10, "{\"name\":\"A\"}".to_string());
        assert_eq!(store.len(), 1);
        assert_eq!(store.now(), 1_000);
        assert_eq!(store.get("detail:7").map(String::as_str), Some("{\"name\":\"A\"}"));
        assert_eq!(
            store.get_entry("detail:7").map(|e| e.kind),
            Some(CacheKind::GameDetail)
        );
        assert!(!store.is_expired("detail:7"));
        assert!(store.contains_key("detail:7"));

        // Not expired before TTL boundary.
        time.set(1_009);
        assert!(store.get("detail:7").is_some());

        // Expired at the boundary: fresh read hides it but peek still sees it.
        time.set(1_010);
        assert!(store.get("detail:7").is_none());
        assert!(store.get_entry("detail:7").is_none());
        assert!(store.is_expired("detail:7"));
        assert!(store.peek("detail:7").is_some());

        // expire() removes only expired keys.
        assert!(store.expire("detail:7"));
        assert_eq!(store.len(), 0);
        assert!(!store.expire("detail:7"));
        assert!(store.invalidate("detail:7").is_none());
    }

    #[test]
    fn put_returns_previous_entry_and_invalidate_removes_fresh_data() {
        let (_t, clock) = clock_at(0);
        let mut store: CacheStore = CacheStore::with_clock(clock);
        let first = CacheEntry::new(
            "icon".to_string(),
            CacheKind::GameIcon,
            0,
            100,
            "a.webp".to_string(),
        );
        assert!(store.put(first).is_none());
        let second = CacheEntry::new(
            "icon".to_string(),
            CacheKind::GameIcon,
            5,
            100,
            "b.webp".to_string(),
        );
        let old = store.put(second).expect("previous entry");
        assert_eq!(old.payload, "a.webp");
        let removed = store.invalidate("icon").expect("removed entry");
        assert_eq!(removed.payload, "b.webp");
        assert!(store.is_empty());
    }

    #[test]
    fn purge_expired_removes_only_stale_entries() {
        let (time, clock) = clock_at(100);
        let mut store: CacheStore = CacheStore::with_clock(clock);
        store.insert("fresh", CacheKind::ApiResponse, 50, "f".to_string());
        store.insert("stale", CacheKind::ApiResponse, 5, "s".to_string());
        time.set(106);
        assert_eq!(store.purge_expired(), 1);
        assert_eq!(store.len(), 1);
        assert_eq!(store.get("fresh").map(String::as_str), Some("f"));
        assert!(store.peek("stale").is_none());
        assert_eq!(store.purge_expired(), 0);
    }

    // -----------------------------------------------------------------------
    // Stale-while-revalidate
    // -----------------------------------------------------------------------

    #[test]
    fn get_with_policy_reports_fresh_then_stale_then_none() {
        let (time, clock) = clock_at(0);
        let mut store: CacheStore = CacheStore::with_clock(clock);
        store.insert(
            "detail:1",
            CacheKind::GameDetail,
            30,
            "cached-dto".to_string(),
        );

        let (value, stale) = store.get_with_policy("detail:1").expect("fresh hit");
        assert_eq!(value, "cached-dto");
        assert!(!stale);

        time.set(30); // exact TTL boundary -> stale
        let (entry, stale) = store.entry_with_policy("detail:1").expect("stale hit");
        assert_eq!(entry.payload, "cached-dto");
        assert!(stale);
        assert_eq!(entry.remaining_secs(store.now()), 0);

        assert!(store.invalidate("detail:1").is_some());
        assert!(store.get_with_policy("detail:1").is_none());
    }

    #[test]
    fn stale_while_revalidate_survives_clock_advance_and_absent_keys() {
        let (time, clock) = clock_at(10);
        let mut store: CacheStore = CacheStore::with_clock(clock);
        store.insert("k", CacheKind::GameConfig, 1, "cfg".to_string());
        time.set(1_000_000);
        let (value, stale) = store.get_with_policy("k").expect("stale still served");
        assert_eq!(value, "cfg");
        assert!(stale);
        assert!(store.get("k").is_none(), "strict read hides stale value");
        assert!(store.get_with_policy("missing").is_none());
    }

    #[test]
    fn store_iter_and_clear() {
        let (_t, clock) = clock_at(0);
        let mut store: CacheStore = CacheStore::with_clock(clock);
        store.insert("a", CacheKind::GameDetail, 10, "1".to_string());
        store.insert("b", CacheKind::GameIcon, 10, "2".to_string());
        assert_eq!(store.iter().count(), 2);
        store.clear();
        assert!(store.is_empty());
        assert_eq!(store.iter().count(), 0);
    }

    // -----------------------------------------------------------------------
    // LRU / capacity (Media3 + OkHttp grounding)
    // -----------------------------------------------------------------------

    #[test]
    fn capacity_constants_match_reference_byte_literals() {
        assert_eq!(MEDIA3_CACHE_DIR, "egg_media3_cache");
        assert_eq!(MEDIA3_MAX_BYTES, 1_048_576_000); // 1048576000L
        assert_eq!(OKHTTP_MAX_BYTES, 134_217_728); // 128 MiB
        assert_eq!(CACHE_DATASOURCE_FLAGS, 2);
        assert_eq!(LruEvictor::media3().capacity_bytes(), MEDIA3_MAX_BYTES);
        assert_eq!(LruEvictor::okhttp().capacity_bytes(), OKHTTP_MAX_BYTES);
    }

    #[test]
    fn lru_evicts_oldest_until_under_capacity() {
        let mut evictor = LruEvictor::new(100);
        assert!(evictor.put("a", 60).is_empty());
        assert_eq!(evictor.used_bytes(), 60);
        assert_eq!(evictor.put("b", 60), vec!["a".to_string()]);
        assert_eq!(evictor.used_bytes(), 60);
        assert!(!evictor.contains("a"));
        assert!(evictor.contains("b"));
        assert_eq!(evictor.put("c", 30), Vec::<String>::new());
        assert_eq!(evictor.used_bytes(), 90);
        assert_eq!(evictor.len(), 2);
    }

    #[test]
    fn lru_touch_changes_the_victim() {
        let mut evictor = LruEvictor::new(100);
        evictor.put("a", 50);
        evictor.put("b", 50);
        assert!(evictor.touch("a"));
        assert!(!evictor.touch("missing"));
        // "b" is now least-recently-used and must be the victim.
        assert_eq!(evictor.put("c", 50), vec!["b".to_string()]);
        assert!(evictor.contains("a"));
        assert!(evictor.contains("c"));
        assert_eq!(evictor.used_bytes(), 100);
    }

    #[test]
    fn lru_replace_updates_size_and_remove_frees_bytes() {
        let mut evictor = LruEvictor::new(100);
        evictor.put("a", 40);
        assert_eq!(evictor.size_of("a"), Some(40));
        assert!(evictor.put("a", 70).is_empty());
        assert_eq!(evictor.used_bytes(), 70);
        assert_eq!(evictor.size_of("a"), Some(70));
        assert_eq!(evictor.remove("a"), Some(70));
        assert_eq!(evictor.used_bytes(), 0);
        assert_eq!(evictor.remove("a"), None);
        evictor.put("b", 10);
        evictor.clear();
        assert!(evictor.is_empty());
        assert_eq!(evictor.used_bytes(), 0);
    }

    #[test]
    fn lru_single_oversized_entry_is_evicted_immediately() {
        let mut evictor = LruEvictor::new(10);
        assert_eq!(evictor.put("huge", 50), vec!["huge".to_string()]);
        assert!(evictor.is_empty());
        assert_eq!(evictor.used_bytes(), 0);
    }
}
