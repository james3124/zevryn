//! Offline capability classification and availability decision
//! (`com.xj.common.offline.capability`).
//!
//! Models which launcher features can run without a connection, grounded in
//! the local data sources of GameHub Lite v5.1.8 (Room, MMKV, imported files,
//! Media3/OkHttp caches). No real I/O is performed. See `REPORT.md`.

mod availability;
mod capability;
mod policy;
mod strings;

pub use availability::{
    availability_status, is_available, is_capability_available, AvailabilityStatus,
};
pub use capability::{DataSource, OfflineCapability, OfflineLevel};
pub use policy::{default_data_source, default_level, OfflinePolicy};
pub use strings::{
    COMM_NETWORK_ERR, ERROR_PC_OFFLINE, NO_NETWORK, OFFLINE_STRINGS, PCVIEW_HEADER_OFFLINE,
    STEAM_OFFLINE_MODE, STEAM_OFFLINE_MODE_DESC,
};

#[cfg(test)]
mod tests {
    use super::*;

    // -----------------------------------------------------------------------
    // Static policy mapping
    // -----------------------------------------------------------------------

    #[test]
    fn policy_levels_cover_all_three_tiers() {
        let policy = OfflinePolicy;
        let full = policy.capabilities_for_level(OfflineLevel::Full);
        let partial = policy.capabilities_for_level(OfflineLevel::Partial);
        let online = policy.capabilities_for_level(OfflineLevel::OnlineOnly);

        assert_eq!(
            full,
            vec![
                OfflineCapability::LocalGameLibrary,
                OfflineCapability::ImportGame,
                OfflineCapability::LaunchPcEmu,
                OfflineCapability::GamepadMapping,
                OfflineCapability::Settings,
            ]
        );
        assert_eq!(
            partial,
            vec![
                OfflineCapability::GameDetailCached,
                OfflineCapability::SteamOfflineMode,
            ]
        );
        assert_eq!(
            online,
            vec![OfflineCapability::CloudPlay, OfflineCapability::SocialStore]
        );
        assert_eq!(full.len() + partial.len() + online.len(), 9);
    }

    #[test]
    fn each_capability_has_a_grounded_data_source() {
        let policy = OfflinePolicy;
        let expected = [
            (OfflineCapability::LocalGameLibrary, DataSource::Room),
            (OfflineCapability::ImportGame, DataSource::LocalFiles),
            (OfflineCapability::LaunchPcEmu, DataSource::Room),
            (OfflineCapability::GamepadMapping, DataSource::LocalFiles),
            (OfflineCapability::Settings, DataSource::Mmkv),
            (OfflineCapability::GameDetailCached, DataSource::OkHttpCache),
            (OfflineCapability::CloudPlay, DataSource::Network),
            (OfflineCapability::SocialStore, DataSource::Network),
            (OfflineCapability::SteamOfflineMode, DataSource::Room),
        ];
        for (cap, source) in expected {
            assert_eq!(policy.data_source_for(cap), source, "{cap:?}");
            assert_eq!(cap.data_source(), source, "{cap:?}");
            assert_eq!(default_data_source(cap), source, "{cap:?}");
            assert_eq!(default_level(cap), policy.level_for(cap), "{cap:?}");
        }
    }

    #[test]
    fn every_level_has_a_local_fallback_flag_consistent_with_mapping() {
        for cap in OfflineCapability::ALL {
            let level = cap.level();
            let source = cap.data_source();
            // No OnlineOnly capability may claim a local primary source, and
            // no Full/Partial capability may be backed only by the network.
            if level == OfflineLevel::OnlineOnly {
                assert!(!source.is_local(), "{cap:?}");
                assert!(!level.has_offline_fallback(), "{cap:?}");
            } else {
                assert!(source.is_local(), "{cap:?}");
                assert!(level.has_offline_fallback(), "{cap:?}");
            }
        }
    }

    #[test]
    fn as_str_tokens_are_stable_and_unique() {
        let mut tokens = std::collections::HashSet::new();
        for cap in OfflineCapability::ALL {
            assert!(!cap.as_str().is_empty());
            assert!(tokens.insert(cap.as_str()), "duplicate token {}", cap.as_str());
        }
        assert_eq!(OfflineLevel::Full.as_str(), "full");
        assert_eq!(OfflineLevel::Partial.as_str(), "partial");
        assert_eq!(OfflineLevel::OnlineOnly.as_str(), "online_only");
        assert_eq!(OfflineLevel::ALL.len(), 3);
        assert_eq!(DataSource::ALL.len(), 6);
    }

    // -----------------------------------------------------------------------
    // Availability matrix — all 2^3 states for each level
    // -----------------------------------------------------------------------

    /// Expected boolean for (has_local_data, network_online, cache_fresh).
    fn expected(level: OfflineLevel, local: bool, online: bool, _fresh: bool) -> bool {
        match level {
            OfflineLevel::Full => local,
            OfflineLevel::Partial => local || online,
            OfflineLevel::OnlineOnly => online,
        }
    }

    #[test]
    fn availability_matrix_for_every_level_and_state() {
        for level in OfflineLevel::ALL {
            for local in [false, true] {
                for online in [false, true] {
                    for fresh in [false, true] {
                        let want = expected(level, local, online, fresh);
                        let got = is_available(level, local, online, fresh);
                        assert_eq!(got, want, "{level:?} local={local} online={online} fresh={fresh}");
                        // The capability overload must agree via the policy.
                        let probe = OfflinePolicy
                            .capabilities_for_level(level)
                            .first()
                            .copied()
                            .unwrap_or(OfflineCapability::LocalGameLibrary);
                        assert_eq!(
                            is_capability_available(probe, local, online, fresh),
                            want,
                            "capability overload {probe:?}"
                        );
                        assert_eq!(
                            availability_status(level, local, online, fresh).is_available(),
                            want,
                            "status {level:?}"
                        );
                    }
                }
            }
        }
    }

    #[test]
    fn full_capabilities_ignore_network_and_cache() {
        for cap in OfflinePolicy.capabilities_for_level(OfflineLevel::Full) {
            assert!(is_capability_available(cap, true, false, false), "{cap:?}");
            assert!(is_capability_available(cap, true, true, false), "{cap:?}");
            assert!(is_capability_available(cap, true, false, true), "{cap:?}");
            assert!(!is_capability_available(cap, false, true, true), "{cap:?}");
            assert!(!is_capability_available(cap, false, false, false), "{cap:?}");
        }
    }

    #[test]
    fn online_only_ignores_local_data_and_cache() {
        for cap in OfflinePolicy.capabilities_for_level(OfflineLevel::OnlineOnly) {
            assert!(is_capability_available(cap, false, true, false), "{cap:?}");
            assert!(is_capability_available(cap, true, true, false), "{cap:?}");
            assert!(!is_capability_available(cap, true, false, true), "{cap:?}");
            assert!(!is_capability_available(cap, false, false, true), "{cap:?}");
        }
    }

    #[test]
    fn partial_capabilities_serve_stale_data_offline() {
        for cap in OfflinePolicy.capabilities_for_level(OfflineLevel::Partial) {
            // Stale local copy is still usable offline (stale-while-revalidate).
            assert!(is_capability_available(cap, true, false, false), "{cap:?}");
            assert_eq!(
                availability_status(cap.level(), true, false, false),
                AvailabilityStatus::AvailableStale,
                "{cap:?}"
            );
            // Fresh local copy is reported as fresh.
            assert_eq!(
                availability_status(cap.level(), true, false, true),
                AvailabilityStatus::AvailableFresh,
                "{cap:?}"
            );
            // No local data + online -> fetch path.
            assert_eq!(
                availability_status(cap.level(), false, true, false),
                AvailabilityStatus::AvailableOnline,
                "{cap:?}"
            );
            // No local data + offline -> unavailable.
            assert!(!is_capability_available(cap, false, false, false), "{cap:?}");
        }
    }

    #[test]
    fn steam_offline_mode_needs_prior_verified_local_data() {
        // Modelled by Partial + has_local_data (SteamDB rows from an earlier
        // online verification). Offline without them must be unavailable.
        assert!(!is_capability_available(
            OfflineCapability::SteamOfflineMode,
            false,
            false,
            false
        ));
        assert!(is_capability_available(
            OfflineCapability::SteamOfflineMode,
            true,
            false,
            false
        ));
        assert_eq!(STEAM_OFFLINE_MODE, "Offline Mode");
        assert_eq!(
            STEAM_OFFLINE_MODE_DESC,
            "Allow running verified Steam games without network connection"
        );
    }

    #[test]
    fn offline_ui_strings_are_byte_exact() {
        assert_eq!(OFFLINE_STRINGS.len(), 6);
        let expected = [
            ("pc_cc_steam_offline_mode", "Offline Mode"),
            (
                "pc_cc_steam_offline_mode_desc",
                "Allow running verified Steam games without network connection",
            ),
            ("pcview_menu_header_offline", "Offline"),
            ("comm_network_err", "Network error"),
            ("no_network", "Network Error."),
            ("error_pc_offline", "Computer is offline"),
        ];
        for (i, (name, value)) in expected.iter().enumerate() {
            assert_eq!(OFFLINE_STRINGS[i], (*name, *value));
        }
        assert_eq!(PCVIEW_HEADER_OFFLINE, "Offline");
        assert_eq!(COMM_NETWORK_ERR, "Network error");
        assert_eq!(NO_NETWORK, "Network Error.");
        assert_eq!(ERROR_PC_OFFLINE, "Computer is offline");
    }
}
