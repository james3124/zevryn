//! `FocusState`: current focused node plus navigation history.
//!
//! The Java system tracks a single `lastFocusedView`; this model keeps the
//! whole path so a caller can backtrack. Movement delegates to
//! [`FocusController::resolve`].

use super::{FocusController, FocusDirection, NodeId};

/// Mutable focus state: the current node and the stack of previously focused
/// nodes. [`FocusState::move_focus`] is the Rust equivalent of Java's
/// `move(direction)` (`move` is a reserved keyword in Rust).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FocusState {
    controller: FocusController,
    current: Option<NodeId>,
    history: Vec<NodeId>,
}

impl FocusState {
    pub fn new(controller: FocusController, current: impl Into<NodeId>) -> Self {
        Self {
            controller,
            current: Some(current.into()),
            history: Vec::new(),
        }
    }

    /// State with no focused node yet.
    pub fn empty(controller: FocusController) -> Self {
        Self {
            controller,
            current: None,
            history: Vec::new(),
        }
    }

    pub fn controller(&self) -> &FocusController {
        &self.controller
    }

    pub fn controller_mut(&mut self) -> &mut FocusController {
        &mut self.controller
    }

    pub fn current(&self) -> Option<&str> {
        self.current.as_deref()
    }

    pub fn history(&self) -> &[NodeId] {
        &self.history
    }

    /// Focus a node directly, recording the previous one in history.
    pub fn set_current(&mut self, id: impl Into<NodeId>) {
        if let Some(previous) = self.current.take() {
            self.history.push(previous);
        }
        self.current = Some(id.into());
    }

    /// Move one step in `direction`. Returns the new node id on success;
    /// failed moves leave both current node and history unchanged.
    pub fn move_focus(&mut self, direction: FocusDirection) -> Option<&str> {
        let current = self.current.clone()?;
        let next = self.controller.resolve(&current, direction)?;
        self.history.push(current);
        self.current = Some(next);
        self.current.as_deref()
    }

    /// Backtrack to the previous node, if any.
    pub fn back(&mut self) -> Option<&str> {
        let previous = self.history.pop()?;
        self.current = Some(previous);
        self.current.as_deref()
    }

    pub fn history_len(&self) -> usize {
        self.history.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::super::BoundaryPolicy;

    #[test]
    fn move_records_history_and_backtracks() {
        let controller = FocusController::horizontal(["a", "b", "c"], BoundaryPolicy::Clamp);
        let mut state = FocusState::new(controller, "a");
        assert_eq!(state.move_focus(FocusDirection::Right), Some("b"));
        assert_eq!(state.current(), Some("b"));
        assert_eq!(state.history(), ["a"]);
        assert_eq!(state.move_focus(FocusDirection::Right), Some("c"));
        assert_eq!(state.history_len(), 2);
        assert_eq!(state.back(), Some("b"));
        assert_eq!(state.back(), Some("a"));
        assert_eq!(state.back(), None);
    }

    #[test]
    fn failed_move_is_noop() {
        let controller = FocusController::horizontal(["a", "b"], BoundaryPolicy::Clamp);
        let mut state = FocusState::new(controller, "a");
        assert_eq!(state.move_focus(FocusDirection::Left), None);
        assert_eq!(state.current(), Some("a"));
        assert!(state.history().is_empty());
    }

    #[test]
    fn wrap_move_and_direct_set() {
        let controller = FocusController::horizontal(["a", "b"], BoundaryPolicy::Wrap);
        let mut state = FocusState::new(controller, "b");
        assert_eq!(state.move_focus(FocusDirection::Right), Some("a"));
        state.set_current("c");
        assert_eq!(state.current(), Some("c"));
        assert_eq!(state.history(), ["b", "a"]);
    }

    #[test]
    fn empty_state_has_no_focus() {
        let controller = FocusController::horizontal(["a"], BoundaryPolicy::Clamp);
        let mut state = FocusState::empty(controller);
        assert_eq!(state.current(), None);
        assert_eq!(state.move_focus(FocusDirection::Right), None);
    }
}
