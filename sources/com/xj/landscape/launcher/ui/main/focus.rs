//! D-pad focus-navigation state machine, mirroring the launcher's custom
//! focus system (`FocusableActivity` / `FocusableRoot`).
//!
//! The portable behavior is `LandscapeLauncherMainActivity.d(event, last)`
//! plus the L1/R1 handling in `b(event)`. Screen elements are identified by
//! their view ids from `LlauncherActivityNewLauncherMainBinding`:
//! `mainTabView`, `flOpenNav`, `llRightTopStatus`, and the current child
//! fragment's content view (modelled here as `cards`).

use std::collections::HashMap;

/// D-pad directions used by the launcher's custom focus system.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Direction {
    Left,
    Right,
    Up,
    Down,
}

/// The four canonical focus nodes of the launcher home screen.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum FocusNode {
    MainTabView,
    FlOpenNav,
    LlRightTopStatus,
    Cards,
}

impl FocusNode {
    pub fn id(self) -> &'static str {
        match self {
            Self::MainTabView => "mainTabView",
            Self::FlOpenNav => "flOpenNav",
            Self::LlRightTopStatus => "llRightTopStatus",
            Self::Cards => "cards",
        }
    }
}

/// Focus-navigation graph. Nodes are screen-element ids; edges are explicit
/// per-direction links. Mirrors `FocusableActivity` / `FocusTabLayout`
/// behavior without depending on Android.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FocusGraph {
    nodes: Vec<String>,
    edges: HashMap<(String, Direction), String>,
    current: String,
}

impl FocusGraph {
    pub fn new(current: impl Into<String>) -> Self {
        let current = current.into();
        Self {
            nodes: vec![current.clone()],
            edges: HashMap::new(),
            current,
        }
    }

    pub fn add_node(&mut self, id: impl Into<String>) {
        let id = id.into();
        if !self.nodes.contains(&id) {
            self.nodes.push(id);
        }
    }

    pub fn link(&mut self, from: &str, dir: Direction, to: &str) {
        self.add_node(from);
        self.add_node(to);
        self.edges.insert((from.to_owned(), dir), to.to_owned());
    }

    pub fn current(&self) -> &str {
        &self.current
    }

    /// Move focus; returns the new node when a link exists.
    pub fn move_focus(&mut self, dir: Direction) -> Option<&str> {
        let next = self.edges.get(&(self.current.clone(), dir))?.clone();
        self.current = next;
        Some(&self.current)
    }

    pub fn has_node(&self, id: &str) -> bool {
        self.nodes.iter().any(|n| n == id)
    }
}

/// Build the launcher's canonical focus graph with the exact edges taken from
/// `LandscapeLauncherMainActivity.d(event, lastFocusedView)`.
///
/// * UP from `cards` -> `mainTabView`
/// * DOWN from any header node -> `cards` (the current child fragment)
/// * LEFT: `mainTabView`/`flOpenNav` -> `flOpenNav`,
///         `llRightTopStatus` -> `mainTabView`
/// * RIGHT: `flOpenNav` -> `mainTabView`,
///          `mainTabView` -> `llRightTopStatus`
///
/// Directions with no edge (e.g. RIGHT from `llRightTopStatus`, UP from the
/// header nodes, LEFT/RIGHT from `cards`) are left unhandled, so `move_focus`
/// returns `None` — matching the method's `null` return.
pub fn launcher_focus_graph() -> FocusGraph {
    use Direction::*;
    use FocusNode::*;

    let mut g = FocusGraph::new(MainTabView.id());
    g.link(Cards.id(), Up, MainTabView.id());

    g.link(MainTabView.id(), Down, Cards.id());
    g.link(FlOpenNav.id(), Down, Cards.id());
    g.link(LlRightTopStatus.id(), Down, Cards.id());

    // LEFT from flOpenNav stays on flOpenNav (self-loop in the Java code).
    g.link(FlOpenNav.id(), Left, FlOpenNav.id());
    g.link(MainTabView.id(), Left, FlOpenNav.id());
    g.link(LlRightTopStatus.id(), Left, MainTabView.id());

    g.link(FlOpenNav.id(), Right, MainTabView.id());
    g.link(MainTabView.id(), Right, LlRightTopStatus.id());
    g
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn generic_graph_navigation() {
        let mut g = FocusGraph::new("mainTabView");
        g.link("mainTabView", Direction::Left, "flOpenNav");
        g.link("mainTabView", Direction::Right, "llRightTopStatus");
        assert_eq!(g.move_focus(Direction::Left), Some("flOpenNav"));
        assert_eq!(g.current(), "flOpenNav");
        assert_eq!(g.move_focus(Direction::Left), None);
    }

    #[test]
    fn launcher_graph_has_four_nodes() {
        let g = launcher_focus_graph();
        assert_eq!(g.current(), FocusNode::MainTabView.id());
        for n in [FocusNode::MainTabView, FocusNode::FlOpenNav, FocusNode::LlRightTopStatus, FocusNode::Cards] {
            assert!(g.has_node(n.id()), "missing node {}", n.id());
        }
    }

    #[test]
    fn launcher_horizontal_navigation() {
        let mut g = launcher_focus_graph();
        assert_eq!(g.move_focus(Direction::Right), Some("llRightTopStatus"));
        assert_eq!(g.move_focus(Direction::Right), None);
        assert_eq!(g.move_focus(Direction::Left), Some("mainTabView"));
        assert_eq!(g.move_focus(Direction::Left), Some("flOpenNav"));
        assert_eq!(g.move_focus(Direction::Right), Some("mainTabView"));
    }

    #[test]
    fn launcher_vertical_navigation() {
        let mut g = launcher_focus_graph();
        // mainTabView UP is unhandled.
        assert_eq!(g.move_focus(Direction::Up), None);
        assert_eq!(g.move_focus(Direction::Down), Some("cards"));
        assert_eq!(g.move_focus(Direction::Up), Some("mainTabView"));
        assert_eq!(g.move_focus(Direction::Down), Some("cards"));
        // LEFT/RIGHT from cards are unhandled.
        assert_eq!(g.move_focus(Direction::Left), None);
        assert_eq!(g.move_focus(Direction::Right), None);
    }
}
