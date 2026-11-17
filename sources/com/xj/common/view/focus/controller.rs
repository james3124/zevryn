//! Focus controllers: resolve the next focused node for a direction.
//!
//! Mirrors the portable parts of `RecyclerViewFocusController`,
//! `LinearRecyclerViewFocusController`, `GridRecyclerViewFocusController` and
//! `FocusableRoot.DefaultImpls.d(event, last)`:
//!
//! * An explicit per-direction edge is followed first (the root's `d()` /
//!   the launcher activity's hand-written edge table).
//! * Otherwise the controller steps along an ordered axis by ±1. At a
//!   boundary the [`BoundaryPolicy`] decides between:
//!   * [`BoundaryPolicy::Clamp`] — return `None` (hand the event to the parent,
//!     matching Java's `onInterceptEndEvent` which defaults to `false`, and
//!     `FocusableRoot`'s `getOrNull(index ± 1)`), or
//!   * [`BoundaryPolicy::Wrap`] — wrap around with modular arithmetic.

use super::{FocusDirection, FocusEvent, FocusGraph, NodeId};

#[cfg(test)]
use super::KeyAction;

/// Behavior at the end of an ordered axis.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BoundaryPolicy {
    /// Stop at the boundary and return `None` (the default in GameHub Lite).
    Clamp,
    /// Wrap around to the other end of the axis.
    Wrap,
}

/// Which axis a controller operates on. Perpendicular directions are ignored,
/// just as `LinearRecyclerViewFocusController.b()/c()` only handle the
/// layout manager's orientation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Axis {
    Horizontal,
    Vertical,
    Both,
}

impl Axis {
    pub fn allows(self, direction: FocusDirection) -> bool {
        match self {
            Self::Horizontal => direction.is_horizontal(),
            Self::Vertical => direction.is_vertical(),
            Self::Both => true,
        }
    }
}

/// Resolves the next focused node given a direction and the current node.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FocusController {
    graph: FocusGraph,
    policy: BoundaryPolicy,
    axis: Axis,
}

impl FocusController {
    /// Empty controller with no nodes.
    pub fn new(policy: BoundaryPolicy) -> Self {
        Self {
            graph: FocusGraph::new(),
            policy,
            axis: Axis::Both,
        }
    }

    /// Controller backed by an existing graph (axis unrestricted).
    pub fn with_graph(graph: FocusGraph, policy: BoundaryPolicy) -> Self {
        Self {
            graph,
            policy,
            axis: Axis::Both,
        }
    }

    /// Build a horizontal linear controller from an ordered list of nodes.
    pub fn horizontal<I, S>(ids: I, policy: BoundaryPolicy) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<NodeId>,
    {
        Self::ordered(ids, Axis::Horizontal, policy)
    }

    /// Build a vertical linear controller from an ordered list of nodes.
    pub fn vertical<I, S>(ids: I, policy: BoundaryPolicy) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<NodeId>,
    {
        Self::ordered(ids, Axis::Vertical, policy)
    }

    fn ordered<I, S>(ids: I, axis: Axis, policy: BoundaryPolicy) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<NodeId>,
    {
        let mut graph = FocusGraph::new();
        for id in ids {
            graph.add_node(id);
        }
        Self {
            graph,
            policy,
            axis,
        }
    }

    pub fn graph(&self) -> &FocusGraph {
        &self.graph
    }

    pub fn policy(&self) -> BoundaryPolicy {
        self.policy
    }

    pub fn axis(&self) -> Axis {
        self.axis
    }

    pub fn set_policy(&mut self, policy: BoundaryPolicy) {
        self.policy = policy;
    }

    pub fn add_node(&mut self, id: impl Into<NodeId>) {
        self.graph.add_node(id);
    }

    pub fn link(&mut self, from: &str, direction: FocusDirection, to: &str) {
        self.graph.link(from, direction, to);
    }

    /// Resolve the next node. Explicit edges win; otherwise step ±1 along the
    /// insertion order and apply the boundary policy.
    pub fn resolve(&self, current: &str, direction: FocusDirection) -> Option<NodeId> {
        if let Some(next) = self.graph.neighbor(current, direction) {
            return Some(next);
        }
        if !self.axis.allows(direction) {
            return None;
        }

        let ids: Vec<&str> = self.graph.node_ids().collect();
        let len = ids.len();
        if len == 0 {
            return None;
        }
        let index = ids.iter().position(|id| *id == current)?;
        let target = index as isize + direction.step();
        let target = match self.policy {
            BoundaryPolicy::Clamp => {
                if target < 0 || target >= len as isize {
                    return None;
                }
                target
            }
            BoundaryPolicy::Wrap => target.rem_euclid(len as isize),
        };
        Some(ids[target as usize].to_owned())
    }

    /// Resolve from a [`FocusEvent`]; focus moves on `ACTION_DOWN` only.
    pub fn resolve_event(&self, current: &str, event: &FocusEvent) -> Option<NodeId> {
        if !event.is_down() {
            return None;
        }
        self.resolve(current, event.direction)
    }
}

/// Grid controller, modelling `GridRecyclerViewFocusController`'s row/column
/// movement: LEFT/RIGHT move within a row, UP/DOWN move to the same column of
/// the adjacent row (or the nearest column when that row is shorter).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GridFocusController {
    rows: Vec<Vec<NodeId>>,
    policy: BoundaryPolicy,
}

impl GridFocusController {
    pub fn new<I, R, S>(rows: I, policy: BoundaryPolicy) -> Self
    where
        I: IntoIterator<Item = R>,
        R: IntoIterator<Item = S>,
        S: Into<NodeId>,
    {
        Self {
            rows: rows
                .into_iter()
                .map(|row| row.into_iter().map(Into::into).collect())
                .collect(),
            policy,
        }
    }

    pub fn rows(&self) -> &[Vec<NodeId>] {
        &self.rows
    }

    pub fn policy(&self) -> BoundaryPolicy {
        self.policy
    }

    /// Resolve the next node in a grid.
    pub fn resolve(&self, current: &str, direction: FocusDirection) -> Option<NodeId> {
        let (row, col) = self.find(current)?;
        match direction {
            FocusDirection::Left | FocusDirection::Right => {
                let cells = &self.rows[row];
                let target = self.apply_policy(
                    col as isize + direction.step(),
                    cells.len(),
                )?;
                Some(cells[target].clone())
            }
            FocusDirection::Up | FocusDirection::Down => {
                let target_row = self.apply_policy(
                    row as isize + direction.step(),
                    self.rows.len(),
                )?;
                // Prefer the same column; otherwise use the closest cell in
                // the target row. This is the portable analogue of the span
                // index / span group matching done with GridLayoutManager.
                self.rows[target_row]
                    .iter()
                    .enumerate()
                    .min_by_key(|(c, _)| (*c as isize - col as isize).abs())
                    .map(|(_, id)| id.clone())
            }
        }
    }

    fn apply_policy(&self, target: isize, len: usize) -> Option<usize> {
        if len == 0 {
            return None;
        }
        match self.policy {
            BoundaryPolicy::Clamp => {
                if target < 0 || target >= len as isize {
                    None
                } else {
                    Some(target as usize)
                }
            }
            BoundaryPolicy::Wrap => Some(target.rem_euclid(len as isize) as usize),
        }
    }

    fn find(&self, id: &str) -> Option<(usize, usize)> {
        self.rows.iter().enumerate().find_map(|(r, row)| {
            row.iter()
                .position(|cell| cell == id)
                .map(|c| (r, c))
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn explicit_edges_take_precedence() {
        let mut controller = FocusController::new(BoundaryPolicy::Clamp);
        controller.link("a", FocusDirection::Right, "x");
        controller.add_node("b");
        assert_eq!(
            controller.resolve("a", FocusDirection::Right),
            Some("x".into())
        );
        assert_eq!(
            controller.resolve("a", FocusDirection::Left),
            None
        );
    }

    #[test]
    fn linear_clamp_stops_at_boundaries() {
        let controller = FocusController::horizontal(["a", "b", "c"], BoundaryPolicy::Clamp);
        assert_eq!(controller.resolve("a", FocusDirection::Right), Some("b".into()));
        assert_eq!(controller.resolve("b", FocusDirection::Right), Some("c".into()));
        assert_eq!(controller.resolve("c", FocusDirection::Right), None);
        assert_eq!(controller.resolve("a", FocusDirection::Left), None);
        // Perpendicular directions are ignored by a horizontal controller.
        assert_eq!(controller.resolve("b", FocusDirection::Down), None);
    }

    #[test]
    fn linear_wrap_uses_modular_index() {
        let controller = FocusController::horizontal(["a", "b", "c"], BoundaryPolicy::Wrap);
        assert_eq!(controller.resolve("c", FocusDirection::Right), Some("a".into()));
        assert_eq!(controller.resolve("a", FocusDirection::Left), Some("c".into()));
        assert_eq!(controller.resolve("b", FocusDirection::Right), Some("c".into()));
    }

    #[test]
    fn vertical_axis_and_policy_change() {
        let mut controller = FocusController::vertical(["top", "middle", "bottom"], BoundaryPolicy::Clamp);
        assert_eq!(controller.resolve("top", FocusDirection::Down), Some("middle".into()));
        assert_eq!(controller.resolve("bottom", FocusDirection::Down), None);
        assert_eq!(controller.resolve("top", FocusDirection::Right), None);
        controller.set_policy(BoundaryPolicy::Wrap);
        assert_eq!(controller.resolve("bottom", FocusDirection::Down), Some("top".into()));
        assert_eq!(controller.resolve("top", FocusDirection::Up), Some("bottom".into()));
    }

    #[test]
    fn resolve_event_requires_key_down() {
        let controller = FocusController::horizontal(["a", "b"], BoundaryPolicy::Clamp);
        assert_eq!(
            controller.resolve_event("a", &FocusEvent::down(FocusDirection::Right)),
            Some("b".into())
        );
        assert_eq!(
            controller.resolve_event("a", &FocusEvent::new(FocusDirection::Right, KeyAction::Up)),
            None
        );
    }

    #[test]
    fn grid_moves_within_rows_and_columns() {
        let clamp = GridFocusController::new([["a", "b", "c"], ["d", "e", "f"]], BoundaryPolicy::Clamp);
        assert_eq!(clamp.resolve("a", FocusDirection::Right), Some("b".into()));
        assert_eq!(clamp.resolve("c", FocusDirection::Right), None);
        assert_eq!(clamp.resolve("b", FocusDirection::Down), Some("e".into()));
        assert_eq!(clamp.resolve("e", FocusDirection::Up), Some("b".into()));

        let wrap = GridFocusController::new([["a", "b", "c"], ["d", "e", "f"]], BoundaryPolicy::Wrap);
        assert_eq!(wrap.resolve("c", FocusDirection::Right), Some("a".into()));
        assert_eq!(wrap.resolve("e", FocusDirection::Down), Some("b".into()));
    }

    #[test]
    fn grid_handles_ragged_rows() {
        let grid = GridFocusController::new(vec![vec!["a", "b", "c"], vec!["d"]], BoundaryPolicy::Clamp);
        // No cell in column 2 of the second row: fall back to its nearest cell.
        assert_eq!(grid.resolve("c", FocusDirection::Down), Some("d".into()));
        assert_eq!(grid.resolve("d", FocusDirection::Up), Some("a".into()));
        assert!(grid.resolve("missing", FocusDirection::Right).is_none());
    }
}
