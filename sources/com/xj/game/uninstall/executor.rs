//! Pure executor for an [`UninstallPlan`].
//!
//! No file deletion, Android dialogs, coroutines, Steam/emulator services or
//! reflection are performed here. Every step except `RemoveFromLibrary` is a
//! state transition recorded in `completed_steps`; the library step is the
//! only one that touches the outside world, through the minimal
//! [`RemoveLibrary`] trait so it can be backed by the built-in storage.

use super::uninstall_confirm::UninstallConfirm;
use super::uninstall_plan::UninstallPlan;
use super::uninstall_step::UninstallStep;

/// Minimal storage hook used by the `RemoveFromLibrary` step.
///
/// `remove(id)` should return `true` when a row was actually removed, matching
/// `removeFromGameLibrary`'s `!list.isEmpty()` result.
pub trait RemoveLibrary {
    fn remove(&mut self, id: &str) -> bool;
}

/// Final outcome of executing a plan; carries the callback triple.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UninstallResult {
    /// Overall success (`z3` of `RemoveOrUninstallGameListener.a`).
    pub success: bool,
    /// Whether a library row was removed (`z4`).
    pub removed: bool,
    /// `""`, `"cancelled"` or `"remove failure"`.
    pub message: String,
}

impl UninstallResult {
    pub fn new(success: bool, removed: bool, message: impl Into<String>) -> Self {
        UninstallResult {
            success,
            removed,
            message: message.into(),
        }
    }

    /// Convert into the listener's callback model.
    pub fn to_confirm(&self) -> UninstallConfirm {
        UninstallConfirm::new(self.success, self.removed, self.message.clone())
    }
}

/// The plan's trace plus its final result.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Execution {
    /// Steps that were actually performed, in order.
    pub completed_steps: Vec<UninstallStep>,
    pub result: UninstallResult,
}

/// Run `plan` against `store`.
///
/// `confirmed` models the `CommUninstallConfirmDialog` checkbox/confirm
/// result: when `false`, execution stops before any destructive step and the
/// result is `UninstallConfirm::cancelled()`.
pub fn execute<S: RemoveLibrary>(
    plan: &UninstallPlan,
    store: &mut S,
    confirmed: bool,
) -> Execution {
    let mut completed_steps = Vec::new();
    let mut removed = false;

    for step in &plan.steps {
        match *step {
            UninstallStep::ConfirmDialog => {
                if !confirmed {
                    return Execution {
                        completed_steps,
                        result: UninstallResult {
                            success: false,
                            removed: false,
                            message: "cancelled".to_owned(),
                        },
                    };
                }
                completed_steps.push(*step);
            }
            UninstallStep::RemoveFromLibrary => {
                if store.remove(plan.game_id()) {
                    removed = true;
                    completed_steps.push(*step);
                } else {
                    // removeFromGameLibrary returned an empty affected list.
                    return Execution {
                        completed_steps,
                        result: UninstallResult {
                            success: false,
                            removed: false,
                            message: "remove failure".to_owned(),
                        },
                    };
                }
            }
            // Pure state transitions: record and continue (no real I/O).
            UninstallStep::DeleteOnlineDownload
            | UninstallStep::DeleteSteamGame
            | UninstallStep::DeleteLocalCover
            | UninstallStep::ResetGameData
            | UninstallStep::UninstallPcDemo => completed_steps.push(*step),
        }
    }

    Execution {
        completed_steps,
        result: UninstallResult {
            success: true,
            removed,
            message: String::new(),
        },
    }
}

/// Convenience: execute with the confirmation dialog accepted.
pub fn execute_confirmed<S: RemoveLibrary>(plan: &UninstallPlan, store: &mut S) -> Execution {
    execute(plan, store, true)
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::super::game_from::GameFrom;
    use super::super::uninstall_plan::{Game, UninstallFlags};
    use std::collections::HashSet;

    #[derive(Default)]
    struct MemoryStore {
        ids: HashSet<String>,
        calls: Vec<String>,
    }

    impl MemoryStore {
        fn with(id: &str) -> Self {
            let mut s = MemoryStore::default();
            s.ids.insert(id.to_owned());
            s
        }
    }

    impl RemoveLibrary for MemoryStore {
        fn remove(&mut self, id: &str) -> bool {
            self.calls.push(id.to_owned());
            self.ids.remove(id)
        }
    }

    fn plan(flags: UninstallFlags) -> UninstallPlan {
        let game = Game::new("42", "620");
        UninstallPlan::new(&game, GameFrom::DownloadFromSvr, flags)
    }

    #[test]
    fn executes_every_step_and_maps_to_callback() {
        let flags = UninstallFlags::new(false, true, true, true);
        let p = plan(flags);
        let mut store = MemoryStore::with("42");
        let ex = execute_confirmed(&p, &mut store);
        assert_eq!(ex.completed_steps, p.steps);
        assert_eq!(store.calls, vec!["42".to_owned()]);
        assert_eq!(
            ex.result,
            UninstallResult::new(true, true, "")
        );
        assert_eq!(ex.result.to_confirm(), UninstallConfirm::ok(true));
        assert!(!store.ids.contains("42"));
    }

    #[test]
    fn declining_confirm_does_nothing() {
        let p = plan(UninstallFlags::default());
        let mut store = MemoryStore::with("42");
        let ex = execute(&p, &mut store, false);
        assert!(ex.completed_steps.is_empty());
        assert!(store.calls.is_empty());
        assert_eq!(ex.result, UninstallResult::new(false, false, "cancelled"));
        assert_eq!(ex.result.to_confirm(), UninstallConfirm::cancelled());
    }

    #[test]
    fn failed_library_removal_reports_remove_failure() {
        let p = plan(UninstallFlags::default());
        // Store without the id: remove() returns false.
        let mut store = MemoryStore::default();
        let ex = execute(&p, &mut store, true);
        assert_eq!(
            ex.result,
            UninstallResult::new(false, false, "remove failure")
        );
        assert!(!ex.completed_steps.contains(&UninstallStep::RemoveFromLibrary));
        assert!(ex.completed_steps.contains(&UninstallStep::DeleteOnlineDownload));
    }

    #[test]
    fn no_library_step_still_succeeds_with_removed_false() {
        let p = plan(UninstallFlags::new(false, false, false, false));
        let mut store = MemoryStore::default();
        let ex = execute_confirmed(&p, &mut store);
        assert!(store.calls.is_empty());
        assert_eq!(ex.result, UninstallResult::new(true, false, ""));
        assert_eq!(ex.result.to_confirm(), UninstallConfirm::ok(false));
    }
}
