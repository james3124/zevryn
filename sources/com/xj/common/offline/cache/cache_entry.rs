//! A single cached value with provenance, timestamp and TTL.
//!
//! This is the portable model of a persisted DTO / cached response. In the
//! reference app the same role is played by rows in Room (e.g. `GameLibraryTable.data`),
//! `@SerializedName` DTOs stored by OkHttp/ForceCache, and Media3 cache spans.

use std::time::{SystemTime, UNIX_EPOCH};

/// What a cached payload represents. The four kinds mirror the distinct
/// persisted payload families in the reference app.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CacheKind {
    /// Game detail DTO (`@SerializedName` fields), e.g. a `GameDetailEntity`.
    GameDetail,
    /// Downloaded / decoded game icon bitmap bytes (raw path / URI string).
    GameIcon,
    /// Per-game configuration (keymap/config JSON persisted via MMKV or Room).
    GameConfig,
    /// Any signed-API JSON response cached by OkHttp / ForceCache.
    ApiResponse,
}

impl CacheKind {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::GameDetail => "game_detail",
            Self::GameIcon => "game_icon",
            Self::GameConfig => "game_config",
            Self::ApiResponse => "api_response",
        }
    }

    pub const ALL: [Self; 4] = [
        Self::GameDetail,
        Self::GameIcon,
        Self::GameConfig,
        Self::ApiResponse,
    ];
}

/// A key/value cache record.
///
/// `K` defaults to `String` (the cache key) and `V` to `String` (the raw
/// payload), matching the reference where keys are URLs/game IDs and payloads
/// are serialized JSON or file paths. Both are kept generic so callers can
/// store byte blobs or typed DTOs without copying.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CacheEntry<K = String, V = String> {
    pub key: K,
    pub kind: CacheKind,
    /// Unix timestamp (seconds) at which the payload was stored.
    pub stored_at: u64,
    /// Time-to-live in seconds. `0` means "expire on the next read".
    pub ttl_secs: u64,
    /// Raw payload; the model does no serialization itself.
    pub payload: V,
}

impl<K, V> CacheEntry<K, V> {
    pub fn new(key: K, kind: CacheKind, stored_at: u64, ttl_secs: u64, payload: V) -> Self {
        Self {
            key,
            kind,
            stored_at,
            ttl_secs,
            payload,
        }
    }

    /// Exclusive expiry timestamp. Saturating addition avoids wrapping a
    /// caller-supplied `u64::MAX` TTL into an already-expired entry.
    pub fn expires_at(&self) -> u64 {
        self.stored_at.saturating_add(self.ttl_secs)
    }

    /// True when `now >= expires_at()`.
    pub fn is_expired_at(&self, now: u64) -> bool {
        now >= self.expires_at()
    }

    /// True when the entry is still within its TTL at `now`.
    pub fn is_fresh_at(&self, now: u64) -> bool {
        !self.is_expired_at(now)
    }

    /// Seconds remaining until expiry at `now` (0 when already expired).
    pub fn remaining_secs(&self, now: u64) -> u64 {
        self.expires_at().saturating_sub(now)
    }

    /// Replace the payload and reset the timestamp to `stored_at`.
    pub fn refresh(&mut self, payload: V, stored_at: u64) {
        self.payload = payload;
        self.stored_at = stored_at;
    }
}

/// Current Unix time in seconds; the default clock for [`super::store::CacheStore`].
///
/// This is a pure clock read, not disk or network I/O. Tests inject their own
/// clock instead.
pub fn system_now_secs() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}
