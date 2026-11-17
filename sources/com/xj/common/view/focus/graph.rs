//! Pure focus graph: nodes with an id and explicit per-direction neighbors.
//!
//! This is the dependency-free counterpart of the `FocusableView` /
//! `FocusableRoot` interfaces. Android concepts (View visibility, enabled
//! state, fragments, RecyclerView adapters) are gone; what remains is exactly
//! the portable part: an identity and a `neighbor(direction)` lookup, which is
//! what `FocusableRoot.DefaultImpls.d(event, last)` and the launcher activity's
//! hand-written `d(event, last)` really do.

use std::collections::HashMap;

use super::FocusDirection;

/// Node identifiers are strings, matching Android view ids used as focus keys.
pub type NodeId = String;

/// Trait-like abstraction of a focusable element: an id and a per-direction
/// neighbor lookup. Kept as a trait so callers can model their own node types
/// while the controllers stay generic.
pub trait FocusableNode {
    /// Stable identity of this node.
    fn id(&self) -> &str;

    /// Neighbor in `direction`, or `None` when that edge is unhandled (the
    /// Java methods return `null` in this case).
    fn neighbor(&self, direction: FocusDirection) -> Option<NodeId>;
}

/// Concrete [`FocusableNode`]: an id with an explicit neighbor table.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FocusNode {
    pub id: NodeId,
    neighbors: HashMap<FocusDirection, NodeId>,
}

impl FocusNode {
    pub fn new(id: impl Into<NodeId>) -> Self {
        Self {
            id: id.into(),
            neighbors: HashMap::new(),
        }
    }

    /// Builder-style neighbor insertion.
    pub fn with_neighbor(mut self, direction: FocusDirection, to: impl Into<NodeId>) -> Self {
        self.set_neighbor(direction, to);
        self
    }

    pub fn set_neighbor(&mut self, direction: FocusDirection, to: impl Into<NodeId>) {
        self.neighbors.insert(direction, to.into());
    }

    /// Direct lookup on this node (also available through the trait).
    pub fn neighbor(&self, direction: FocusDirection) -> Option<&NodeId> {
        self.neighbors.get(&direction)
    }
}

impl FocusableNode for FocusNode {
    fn id(&self) -> &str {
        &self.id
    }

    fn neighbor(&self, direction: FocusDirection) -> Option<NodeId> {
        self.neighbors.get(&direction).cloned()
    }
}

/// A collection of focus nodes with insertion order preserved.
///
/// Insertion order doubles as the navigation axis used by
/// [`super::FocusController`] when applying clamp/wrap at a boundary.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct FocusGraph {
    nodes: Vec<FocusNode>,
}

impl FocusGraph {
    pub fn new() -> Self {
        Self::default()
    }

    /// Add a node if absent. Returns `true` when a new node was inserted.
    pub fn add_node(&mut self, id: impl Into<NodeId>) -> bool {
        let id = id.into();
        if self.contains(&id) {
            return false;
        }
        self.nodes.push(FocusNode::new(id));
        true
    }

    /// Link `from --direction--> to`, creating either node if missing.
    pub fn link(&mut self, from: &str, direction: FocusDirection, to: &str) {
        self.add_node(from);
        self.add_node(to);
        // `add_node` above guarantees `from` is present.
        let i = self.index_of(from).unwrap();
        self.nodes[i].set_neighbor(direction, to);
    }

    pub fn contains(&self, id: &str) -> bool {
        self.index_of(id).is_some()
    }

    pub fn len(&self) -> usize {
        self.nodes.len()
    }

    pub fn is_empty(&self) -> bool {
        self.nodes.is_empty()
    }

    pub fn get(&self, id: &str) -> Option<&FocusNode> {
        self.index_of(id).map(|i| &self.nodes[i])
    }

    /// Node ids in insertion order.
    pub fn node_ids(&self) -> impl Iterator<Item = &str> + '_ {
        self.nodes.iter().map(|node| node.id.as_str())
    }

    /// Neighbor of `id` in `direction`, if an explicit edge exists.
    pub fn neighbor(&self, id: &str, direction: FocusDirection) -> Option<NodeId> {
        self.get(id).and_then(|node| node.neighbor(direction)).cloned()
    }

    fn index_of(&self, id: &str) -> Option<usize> {
        self.nodes.iter().position(|node| node.id == id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn node_neighbor_trait_and_builder() {
        let node = FocusNode::new("a")
            .with_neighbor(FocusDirection::Right, "b")
            .with_neighbor(FocusDirection::Down, "c");
        assert_eq!(node.id(), "a");
        assert_eq!(FocusableNode::neighbor(&node, FocusDirection::Right), Some("b".into()));
        assert_eq!(FocusableNode::neighbor(&node, FocusDirection::Up), None);
        assert_eq!(node.neighbor(FocusDirection::Down).map(String::as_str), Some("c"));
    }

    #[test]
    fn graph_links_and_lookup() {
        let mut graph = FocusGraph::new();
        assert!(graph.add_node("a"));
        assert!(!graph.add_node("a"));
        graph.link("a", FocusDirection::Left, "b");
        graph.link("a", FocusDirection::Right, "c");
        assert_eq!(graph.len(), 3);
        assert!(graph.contains("b"));
        assert_eq!(graph.neighbor("a", FocusDirection::Left), Some("b".into()));
        assert_eq!(graph.neighbor("a", FocusDirection::Right), Some("c".into()));
        assert_eq!(graph.neighbor("a", FocusDirection::Up), None);
        assert_eq!(graph.neighbor("b", FocusDirection::Right), None);
        let ids: Vec<_> = graph.node_ids().collect();
        assert_eq!(ids, ["a", "b", "c"]);
    }
}
