//! Outbox of pending local changes ("changes made offline, synced when online").
//!
//! Every mutation the launcher performs while offline is recorded as an
//! [`OutboxItem`]. When connectivity returns, a caller drains the queue and
//! replays the items against a remote (out of scope here — no network, no I/O).
//! This models the app's real "sync when online" behaviour without performing
//! any networking itself.

use super::offline_source::{GameLibraryTable, Settings};

/// One pending local change awaiting synchronization.
#[derive(Debug, Clone, PartialEq)]
pub enum OutboxItem {
    /// A game was added/imported into the local library (`t_game_library`).
    AddGame(GameLibraryTable),
    /// A game row was removed locally, identified by its row id.
    RemoveGame(i64),
    /// A settings value was changed (key + new string value).
    UpdateSetting { key: String, value: String },
}

impl OutboxItem {
    /// Stable ordering/label used when replaying items deterministically.
    pub fn kind(&self) -> &'static str {
        match self {
            OutboxItem::AddGame(_) => "AddGame",
            OutboxItem::RemoveGame(_) => "RemoveGame",
            OutboxItem::UpdateSetting { .. } => "UpdateSetting",
        }
    }
}

/// A FIFO queue of pending changes.
#[derive(Debug, Clone, Default)]
pub struct Outbox {
    items: Vec<OutboxItem>,
}

impl Outbox {
    pub fn new() -> Self {
        Self::default()
    }

    /// Append a change to the tail of the queue.
    pub fn enqueue(&mut self, item: OutboxItem) {
        self.items.push(item);
    }

    /// Convenience: enqueue an added game.
    pub fn enqueue_add_game(&mut self, row: GameLibraryTable) {
        self.enqueue(OutboxItem::AddGame(row));
    }

    /// Convenience: enqueue a removed game id.
    pub fn enqueue_remove_game(&mut self, id: i64) {
        self.enqueue(OutboxItem::RemoveGame(id));
    }

    /// Convenience: enqueue a settings change.
    pub fn enqueue_update_setting(&mut self, key: &str, value: &str) {
        self.enqueue(OutboxItem::UpdateSetting {
            key: key.to_owned(),
            value: value.to_owned(),
        });
    }

    /// Record the current full settings snapshot as a single update note
    /// (models "settings changed offline" without real KV persistence).
    pub fn enqueue_settings_snapshot(&mut self, settings: &Settings) {
        self.enqueue_update_setting("steam_offline_mode", settings.steam_offline_mode_label());
    }

    /// Remove and return all pending items, in enqueue (FIFO) order. After a
    /// drain the outbox is empty and ready for the next offline session.
    pub fn drain(&mut self) -> Vec<OutboxItem> {
        std::mem::take(&mut self.items)
    }

    /// Borrow the pending items without removing them.
    pub fn pending(&self) -> &[OutboxItem] {
        &self.items
    }

    pub fn len(&self) -> usize {
        self.items.len()
    }

    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }
}

// Small helper kept local to the outbox so it does not widen the public
// Settings surface in offline_source.rs.
trait SettingsLabel {
    fn steam_offline_mode_label(&self) -> &'static str;
}

impl SettingsLabel for Settings {
    fn steam_offline_mode_label(&self) -> &'static str {
        if self.steam_offline_mode_enabled() {
            "enabled"
        } else {
            "disabled"
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn row(id: i64) -> GameLibraryTable {
        GameLibraryTable {
            id,
            user_id: String::new(),
            launch_type: 1403,
            package_name: "p".into(),
            game_id: Some(id as i32),
            local_game_id: None,
            local_mobile_app_id: None,
            data: String::new(),
            time: None,
            is_last_launched: false,
            game_type: 2,
        }
    }

    #[test]
    fn enqueue_and_drain_are_fifo() {
        let mut ob = Outbox::new();
        ob.enqueue_add_game(row(1));
        ob.enqueue_remove_game(2);
        ob.enqueue_update_setting("language", "zh$CN");
        assert_eq!(ob.len(), 3);
        assert!(!ob.is_empty());
        assert_eq!(ob.pending()[0].kind(), "AddGame");
        assert_eq!(ob.pending()[2].kind(), "UpdateSetting");

        let drained = ob.drain();
        assert_eq!(drained.len(), 3);
        assert_eq!(drained[0], OutboxItem::AddGame(row(1)));
        assert_eq!(drained[1], OutboxItem::RemoveGame(2));
        assert_eq!(
            drained[2],
            OutboxItem::UpdateSetting {
                key: "language".into(),
                value: "zh$CN".into()
            }
        );
        assert!(ob.is_empty());
        assert!(ob.drain().is_empty());
    }

    #[test]
    fn settings_snapshot_note() {
        let mut ob = Outbox::new();
        let mut s = Settings::default();
        s.steam_offline_mode = super::super::offline_source::OfflineMode::Enabled;
        ob.enqueue_settings_snapshot(&s);
        match ob.drain().pop().unwrap() {
            OutboxItem::UpdateSetting { key, value } => {
                assert_eq!(key, "steam_offline_mode");
                assert_eq!(value, "enabled");
            }
            other => panic!("unexpected item {:?}", other),
        }
    }
}
