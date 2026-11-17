//! Disk-cache capacity model: the LRU eviction policy and byte limits used by
//! the real application.
//!
//! # Evidence (read-only reference)
//!
//! * `sources/defpackage/Media3CacheUtils.java:43-49` — a Media3 `SimpleCache`
//!   rooted at `<external-files-or-files>/<package>/egg_media3_cache`, built
//!   with `new LeastRecentlyUsedCacheEvictor(1048576000L)` (1000 MiB) and
//!   `new EggExoplayerDatabaseProvider(app, null, 0, 6, null)` (SQLite index of
//!   cached spans). The `CacheDataSource.Factory` uses a
//!   `DefaultHttpDataSource.Factory` upstream and `setFlags(2)` (Media3's
//!   `FLAG_IGNORE_CACHE_ON_ERROR`: a cache miss/error falls back to the network,
//!   so cached video remains usable offline and refreshes when online).
//! * `sources/androidx/media3/datasource/cache/LeastRecentlyUsedCacheEvictor.java`
//!   and `SimpleCache.java` — eviction by last-access order until total bytes
//!   fit the limit.
//! * `sources/com/xj/common/http/EggGameHttpConfig.java:68-70` — OkHttp
//!   `new Cache(context.getCacheDir(), 134217728L)` = 128 MiB HTTP disk cache.
//! * `sources/com/drake/net/NetConfig.java:152-160` and
//!   `sources/com/drake/net/cache/ForceCache.java` — `ForceCache` wraps OkHttp
//!   `DiskLruCache` for forced offline HTTP caching (tag a request with
//!   `ForceCache` to serve the cached response even when stale).
//! * `reference/mapping/04-androidx-framework.md:108-109,146,152` — media3
//!   `datasource/cache` (offline caching) + `database` (download cache) and
//!   Room described as "Local structured DB / offline cache".
//! * `reference/mapping/06-networking.md:12,37,68` — OkHttp 4.12.0 cache and
//!   `internal/cache/DiskLruCache`, `com.drake.net` HTTP stack.
//!
//! This module models only the *policy* (capacity accounting + eviction
//! order); it performs no file or network I/O.

/// Media3 cache directory name (`egg_media3_cache`).
pub const MEDIA3_CACHE_DIR: &str = "egg_media3_cache";

/// `LeastRecentlyUsedCacheEvictor(1048576000L)` — 1,000 MiB (~1 GB).
pub const MEDIA3_MAX_BYTES: u64 = 1_048_576_000;

/// OkHttp `new Cache(cacheDir, 134217728L)` — 128 MiB.
pub const OKHTTP_MAX_BYTES: u64 = 134_217_728;

/// Media3 `CacheDataSource.Factory.setFlags(2)`.
pub const CACHE_DATASOURCE_FLAGS: i32 = 2;

/// One byte-counting, access-ordered cache entry.
#[derive(Debug, Clone, PartialEq, Eq)]
struct Tracked {
    key: String,
    size_bytes: u64,
    /// Monotonic access sequence; larger means more recently used.
    seq: u64,
}

/// Least-recently-used evictor over a byte capacity.
///
/// Mirrors `LeastRecentlyUsedCacheEvictor`: `put` records a payload's size,
/// `touch` marks an access, and `evict_to_capacity` drops the oldest entries
/// until `used_bytes <= capacity_bytes`, returning the evicted keys.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LruEvictor {
    capacity_bytes: u64,
    entries: Vec<Tracked>,
    used_bytes: u64,
    next_seq: u64,
}

impl LruEvictor {
    /// Evictor with an explicit byte capacity.
    pub fn new(capacity_bytes: u64) -> Self {
        Self {
            capacity_bytes,
            entries: Vec::new(),
            used_bytes: 0,
            next_seq: 0,
        }
    }

    /// Evictor configured like the app's Media3 `SimpleCache` (1,000 MiB).
    pub fn media3() -> Self {
        Self::new(MEDIA3_MAX_BYTES)
    }

    /// Evictor configured like the app's OkHttp `Cache` (128 MiB).
    pub fn okhttp() -> Self {
        Self::new(OKHTTP_MAX_BYTES)
    }

    pub fn capacity_bytes(&self) -> u64 {
        self.capacity_bytes
    }

    pub fn used_bytes(&self) -> u64 {
        self.used_bytes
    }

    pub fn len(&self) -> usize {
        self.entries.len()
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    pub fn contains(&self, key: &str) -> bool {
        self.entries.iter().any(|e| e.key == key)
    }

    /// Size recorded for `key`, if present.
    pub fn size_of(&self, key: &str) -> Option<u64> {
        self.entries.iter().find(|e| e.key == key).map(|e| e.size_bytes)
    }

    /// Record or replace an entry, marking it most-recently-used.
    /// Returns the keys evicted (oldest first) to stay within capacity.
    pub fn put(&mut self, key: impl Into<String>, size_bytes: u64) -> Vec<String> {
        let key = key.into();
        if let Some(pos) = self.entries.iter().position(|e| e.key == key) {
            let old = self.entries.remove(pos);
            self.used_bytes = self.used_bytes.saturating_sub(old.size_bytes);
        }
        let seq = self.next_seq;
        self.next_seq += 1;
        self.entries.push(Tracked {
            key,
            size_bytes,
            seq,
        });
        self.used_bytes = self.used_bytes.saturating_add(size_bytes);
        self.evict_to_capacity()
    }

    /// Mark an existing entry as most-recently-used. Returns `false` if absent.
    pub fn touch(&mut self, key: &str) -> bool {
        let seq = self.next_seq;
        match self.entries.iter_mut().find(|e| e.key == key) {
            Some(entry) => {
                entry.seq = seq;
                self.next_seq += 1;
                true
            }
            None => false,
        }
    }

    /// Remove an entry explicitly. Returns its size if it existed.
    pub fn remove(&mut self, key: &str) -> Option<u64> {
        let pos = self.entries.iter().position(|e| e.key == key)?;
        let removed = self.entries.remove(pos);
        self.used_bytes = self.used_bytes.saturating_sub(removed.size_bytes);
        Some(removed.size_bytes)
    }

    /// Evict least-recently-used entries until usage fits the capacity.
    /// Returns evicted keys in eviction order (oldest first).
    pub fn evict_to_capacity(&mut self) -> Vec<String> {
        let mut evicted = Vec::new();
        if self.used_bytes <= self.capacity_bytes {
            return evicted;
        }
        self.entries.sort_by_key(|e| e.seq);
        while self.used_bytes > self.capacity_bytes && !self.entries.is_empty() {
            let victim = self.entries.remove(0);
            self.used_bytes = self.used_bytes.saturating_sub(victim.size_bytes);
            evicted.push(victim.key);
        }
        evicted
    }

    pub fn clear(&mut self) {
        self.entries.clear();
        self.used_bytes = 0;
    }
}
