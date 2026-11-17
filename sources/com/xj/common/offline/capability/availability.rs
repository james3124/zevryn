//! Dynamic availability decision: given a feature's offline level and the
//! current device state, decide whether the feature can be used.
//!
//! # Rules (in priority order)
//!
//! Let `L` be the capability's [`OfflineLevel`].
//!
//! 1. **`Full`** — the feature is backed entirely by Room / MMKV / on-disk
//!    files. Available **iff `has_local_data`** is true. Network state and
//!    cache freshness are irrelevant. If the library/setting/keymap has never
//!    been created there is nothing to operate on, so it is unavailable even
//!    while online (the caller must import/create it first).
//! 2. **`Partial`** — the feature has a local fallback *or* a live fetch path.
//!    Available iff `has_local_data || network_online`:
//!    - local data present (even if `cache_fresh == false`): available in a
//!      *degraded* form. A stale cached DTO is shown via stale-while-revalidate;
//!      a refresh is attempted when the network returns. This is why
//!      `cache_fresh` does **not** gate the boolean.
//!    - no local data but online: available by fetching fresh data.
//!    - neither: unavailable.
//! 3. **`OnlineOnly`** — available **iff `network_online`**. Local data cannot
//!    substitute for a cloud host or an API feed; `cache_fresh` is ignored.
//!
//! `cache_fresh` is consumed by [`availability_status`] to distinguish a fresh
//! cached hit from a stale one (the hint callers use to trigger a background
//! refresh). It is intentionally not part of the boolean for `Full` (no cache
//! involved) or `OnlineOnly` (nothing cached is usable).
//!
//! The decision is pure: no I/O, no clock, no global state.

use super::capability::OfflineLevel;

/// Quality of an available feature; also reports the unavailable case.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum AvailabilityStatus {
    /// Local data exists and is within its TTL.
    AvailableFresh,
    /// Local data exists but is past its TTL — serve it now, refresh later.
    AvailableStale,
    /// No usable local data, but the network can supply it.
    AvailableOnline,
    /// No local data and no network.
    Unavailable,
}

impl AvailabilityStatus {
    pub fn is_available(self) -> bool {
        self != Self::Unavailable
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Self::AvailableFresh => "available_fresh",
            Self::AvailableStale => "available_stale",
            Self::AvailableOnline => "available_online",
            Self::Unavailable => "unavailable",
        }
    }
}

/// The core availability predicate. See the module docs for the full truth
/// table.
pub fn is_available(
    level: OfflineLevel,
    has_local_data: bool,
    network_online: bool,
    _cache_fresh: bool,
) -> bool {
    match level {
        OfflineLevel::Full => has_local_data,
        OfflineLevel::Partial => has_local_data || network_online,
        OfflineLevel::OnlineOnly => network_online,
    }
}

/// Richer form of [`is_available`] that also reports *how* the feature is
/// available, using `cache_fresh` to mark the stale-while-revalidate case.
pub fn availability_status(
    level: OfflineLevel,
    has_local_data: bool,
    network_online: bool,
    cache_fresh: bool,
) -> AvailabilityStatus {
    match level {
        OfflineLevel::Full => {
            if has_local_data {
                AvailabilityStatus::AvailableFresh
            } else {
                AvailabilityStatus::Unavailable
            }
        }
        OfflineLevel::Partial => {
            if has_local_data {
                if cache_fresh {
                    AvailabilityStatus::AvailableFresh
                } else {
                    AvailabilityStatus::AvailableStale
                }
            } else if network_online {
                AvailabilityStatus::AvailableOnline
            } else {
                AvailabilityStatus::Unavailable
            }
        }
        OfflineLevel::OnlineOnly => {
            if network_online {
                AvailabilityStatus::AvailableOnline
            } else {
                AvailabilityStatus::Unavailable
            }
        }
    }
}

/// Convenience overload taking a capability and consulting the default policy.
pub fn is_capability_available(
    cap: super::capability::OfflineCapability,
    has_local_data: bool,
    network_online: bool,
    cache_fresh: bool,
) -> bool {
    is_available(cap.level(), has_local_data, network_online, cache_fresh)
}
