//! Sync-decision policy for the offline-first facade.
//!
//! The app ships several *local* caches that make remote fetches optional:
//!   * OkHttp's 128 MB disk cache and drake `ForceCache` (`DiskLruCache`),
//!     with `noCache` / `noStore` cache-control overrides — the real-world
//!     basis for [`CachePolicy`] and [`SyncDecision::CacheFirst`].
//!   * Room `GameSirUxDB` / `SteamDB` rows and the Media3 offline cache — the
//!     persisted payloads a `CacheFirst` hit actually reads.
//!
//! This module does not perform any fetch or disk I/O. It only *decides* which
//! strategy a caller should use, given whether the network is available and
//! the state of the local cache. The network-error resource keys
//! (`comm_network_err`, `no_network`, `error_pc_offline`) are re-exported from
//! [`super::offline_source`] for callers that need to render a failure.

use super::offline_source::{ERR_COMMON_NETWORK, ERR_NO_NETWORK};
#[cfg(test)]
use super::offline_source::ERR_PC_OFFLINE;

/// Whether a usable cached copy exists locally.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CacheState {
    /// A fresh, usable cached payload exists (Room row / disk/Media3 cache).
    Fresh,
    /// A cached payload exists but is stale; usable as a fallback only.
    Stale,
    /// No usable local copy.
    Empty,
}

impl CacheState {
    pub fn is_available(self) -> bool {
        self != CacheState::Empty
    }
}

/// Cache-control behaviour, mirroring the app's OkHttp / drake `ForceCache`
/// knobs (`noCache`, `noStore`, and the forced-cache 128 MB `DiskLruCache`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CachePolicy {
    /// Network only; ignore the cache (drake `noCache`).
    NoCache,
    /// Do not write the response to the cache (drake `noStore`).
    NoStore,
    /// Prefer the on-disk/Media3 cache; only fetch on a miss.
    ForceCache,
}

impl Default for CachePolicy {
    fn default() -> Self {
        CachePolicy::ForceCache
    }
}

/// The strategy a caller should follow when a remote fetch is *attempted*.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SyncDecision {
    /// Try the network first, fall back to cache on failure. Maps to OkHttp's
    /// normal cache behaviour and is meaningful only when online.
    NetworkFirst,
    /// Serve the local cache first; refresh from network only on a miss/stale.
    /// This is the offline default (mirrors drake `ForceCache` / 128 MB cache).
    CacheFirst,
    /// No network: use local data only. Reported with `no_network` /
    /// `error_pc_offline` when the user attempts a remote action.
    LocalOnly,
    /// A usable cache exists but a refresh is warranted when the network
    /// returns (stale cache + online). Queue the work in the outbox.
    NeedSync,
}

/// Decide the [`SyncDecision`] from network availability and cache state.
///
/// Rules (deterministic and total over the input domain):
/// - `network_available == false` → [`SyncDecision::LocalOnly`] regardless of
///   cache (a stale cache is still better than nothing, so it is used locally).
/// - online + [`CacheState::Empty`] → [`SyncDecision::NetworkFirst`] (nothing
///   local to serve, so the network is the only source).
/// - online + [`CacheState::Stale`] → [`SyncDecision::NeedSync`] (serve stale
///   locally but flag that a refresh is due).
/// - online + [`CacheState::Fresh`] → [`SyncDecision::CacheFirst`] (serve the
///   fresh cache, skip the round trip).
pub fn decide(network_available: bool, cache_state: CacheState) -> SyncDecision {
    if !network_available {
        return SyncDecision::LocalOnly;
    }
    match cache_state {
        CacheState::Empty => SyncDecision::NetworkFirst,
        CacheState::Stale => SyncDecision::NeedSync,
        CacheState::Fresh => SyncDecision::CacheFirst,
    }
}

/// Map a decision to the network-error resource key that applies when the
/// caller cannot satisfy it offline. `None` means no error is expected.
pub fn error_key_for(decision: SyncDecision) -> Option<&'static str> {
    match decision {
        SyncDecision::LocalOnly => Some(ERR_NO_NETWORK),
        SyncDecision::NeedSync => Some(ERR_COMMON_NETWORK),
        SyncDecision::NetworkFirst | SyncDecision::CacheFirst => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn offline_always_local_only() {
        for cache in [CacheState::Fresh, CacheState::Stale, CacheState::Empty] {
            assert_eq!(decide(false, cache), SyncDecision::LocalOnly);
        }
    }

    #[test]
    fn online_decisions_by_cache_state() {
        assert_eq!(decide(true, CacheState::Empty), SyncDecision::NetworkFirst);
        assert_eq!(decide(true, CacheState::Stale), SyncDecision::NeedSync);
        assert_eq!(decide(true, CacheState::Fresh), SyncDecision::CacheFirst);
    }

    #[test]
    fn cache_state_and_policy_helpers() {
        assert!(CacheState::Fresh.is_available());
        assert!(CacheState::Stale.is_available());
        assert!(!CacheState::Empty.is_available());
        assert_eq!(CachePolicy::default(), CachePolicy::ForceCache);
    }

    #[test]
    fn error_keys_match_resource_names() {
        assert_eq!(error_key_for(SyncDecision::LocalOnly), Some("no_network"));
        assert_eq!(
            error_key_for(SyncDecision::NeedSync),
            Some("comm_network_err")
        );
        assert_eq!(error_key_for(SyncDecision::CacheFirst), None);
        // error_pc_offline is surfaced by callers for PC games specifically.
        assert_eq!(ERR_PC_OFFLINE, "error_pc_offline");
    }
}
