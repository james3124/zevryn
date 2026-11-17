//! The default offline policy for every [`OfflineCapability`].
//!
//! The mapping is static data: it records *where the feature's bytes live* in
//! the real app, not whether the network happens to be up right now. The
//! dynamic decision is made by [`super::availability::is_available`].

use super::capability::{DataSource, OfflineCapability, OfflineLevel};

/// Mapping from capability to offline level, primary data source and rationale.
///
/// It is a zero-sized value; use [`OfflinePolicy::default`] or just
/// `OfflinePolicy`. All queries are `const`-style pure functions.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct OfflinePolicy;

impl OfflinePolicy {
    /// The reference-grounded level for `cap`.
    pub fn level_for(self, cap: OfflineCapability) -> OfflineLevel {
        match cap {
            OfflineCapability::LocalGameLibrary
            | OfflineCapability::ImportGame
            | OfflineCapability::LaunchPcEmu
            | OfflineCapability::GamepadMapping
            | OfflineCapability::Settings => OfflineLevel::Full,
            OfflineCapability::GameDetailCached | OfflineCapability::SteamOfflineMode => {
                OfflineLevel::Partial
            }
            OfflineCapability::CloudPlay | OfflineCapability::SocialStore => {
                OfflineLevel::OnlineOnly
            }
        }
    }

    /// The primary backing store for `cap`.
    pub fn data_source_for(self, cap: OfflineCapability) -> DataSource {
        match cap {
            OfflineCapability::LocalGameLibrary => DataSource::Room,
            OfflineCapability::ImportGame => DataSource::LocalFiles,
            OfflineCapability::LaunchPcEmu => DataSource::Room,
            OfflineCapability::GamepadMapping => DataSource::LocalFiles,
            OfflineCapability::Settings => DataSource::Mmkv,
            OfflineCapability::GameDetailCached => DataSource::OkHttpCache,
            OfflineCapability::CloudPlay => DataSource::Network,
            OfflineCapability::SocialStore => DataSource::Network,
            OfflineCapability::SteamOfflineMode => DataSource::Room,
        }
    }

    /// Every capability whose level equals `level`, in declaration order.
    pub fn capabilities_for_level(self, level: OfflineLevel) -> Vec<OfflineCapability> {
        OfflineCapability::ALL
            .iter()
            .copied()
            .filter(|&cap| self.level_for(cap) == level)
            .collect()
    }

    /// Convenience wrapper combining the dynamic availability decision with
    /// this policy's static classification.
    pub fn is_available(
        self,
        cap: OfflineCapability,
        has_local_data: bool,
        network_online: bool,
        cache_fresh: bool,
    ) -> bool {
        super::availability::is_available(
            self.level_for(cap),
            has_local_data,
            network_online,
            cache_fresh,
        )
    }
}

/// Level lookup with the default policy (free-function form).
pub fn default_level(cap: OfflineCapability) -> OfflineLevel {
    OfflinePolicy.level_for(cap)
}

/// Data-source lookup with the default policy (free-function form).
pub fn default_data_source(cap: OfflineCapability) -> DataSource {
    OfflinePolicy.data_source_for(cap)
}
