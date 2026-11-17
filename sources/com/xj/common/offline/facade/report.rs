//! Summary of the offline facade's state, for diagnostics/UI.

use super::offline_source::{LocalDataSource, OfflineDataSource};
use super::outbox::Outbox;

use std::fmt::Write as _;

/// A snapshot of "how offline-ready is the launcher right now?".
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OfflineReport {
    /// Number of locally known game rows (`t_game_library`).
    pub game_count: usize,
    /// Number of cached `GameDetailEntity` values.
    pub cached_details: usize,
    /// Number of pending outbox changes awaiting sync.
    pub pending_outbox_items: usize,
    /// Whether the settings source has been loaded into memory.
    pub settings_loaded: bool,
}

impl OfflineReport {
    /// Build a report from the current state of the local source and outbox.
    pub fn capture(data: &LocalDataSource, outbox: &Outbox) -> Self {
        OfflineReport {
            game_count: data.list_games().len(),
            cached_details: data.cached_detail_count(),
            pending_outbox_items: outbox.len(),
            settings_loaded: data.settings_loaded(),
        }
    }

    /// Human-readable one-line summary.
    pub fn summary(&self) -> String {
        let mut out = String::new();
        let _ = write!(
            out,
            "OfflineReport: games={}, cached_details={}, pending_sync={}, settings_loaded={}",
            self.game_count,
            self.cached_details,
            self.pending_outbox_items,
            if self.settings_loaded { "yes" } else { "no" }
        );
        out
    }
}

impl std::fmt::Display for OfflineReport {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.summary())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::super::offline_source::GameLibraryTable;

    fn row(id: i64) -> GameLibraryTable {
        GameLibraryTable {
            id,
            user_id: String::new(),
            launch_type: 1403,
            package_name: "p".into(),
            game_id: None,
            local_game_id: None,
            local_mobile_app_id: None,
            data: String::new(),
            time: None,
            is_last_launched: false,
            game_type: 2,
        }
    }

    #[test]
    fn report_reflects_state_and_formats() {
        let mut ds = LocalDataSource::new();
        ds.upsert_game(row(1));
        ds.upsert_game(row(2));
        let mut ob = Outbox::new();
        ob.enqueue_remove_game(7);
        ob.enqueue_update_setting("language", "en$US");

        let report = OfflineReport::capture(&ds, &ob);
        assert_eq!(report.game_count, 2);
        assert_eq!(report.cached_details, 0);
        assert_eq!(report.pending_outbox_items, 2);
        assert!(!report.settings_loaded);
        assert_eq!(
            report.summary(),
            "OfflineReport: games=2, cached_details=0, pending_sync=2, settings_loaded=no"
        );
        assert_eq!(report.to_string(), report.summary());
    }

    #[test]
    fn report_updates_after_mutation() {
        let mut ds = LocalDataSource::new();
        let mut ob = Outbox::new();
        let empty = OfflineReport::capture(&ds, &ob);
        assert_eq!(empty.game_count, 0);

        ds.load_settings();
        let pending = ob.drain();
        assert!(pending.is_empty());
        let after = OfflineReport::capture(&ds, &ob);
        assert!(after.settings_loaded);
        assert_eq!(after.pending_outbox_items, 0);
    }
}
