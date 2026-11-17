//! Dependency-free Rust model of GameHub Lite v5.1.8's portable focus logic
//! (`com.xj.common.view.focus.focus` and its `controller`, `view`, `app`
//! sub-packages).
//!
//! Only the View-independent state machines are modelled: directions, focus
//! events, a pure focus graph, controller wrap/clamp resolution, tab focus
//! selection and focus state with history. Android `View`/`ViewGroup`,
//! `KeyEvent`, RecyclerView layout managers and animations are out of scope
//! (see `REPORT.md`).
//!
//! Wire from `lib.rs` with:
//!
//! ```text
//! #[path = "../sources/com/xj/common/view/focus/mod.rs"]
//! pub mod focus;
//! ```
//!
//! This is a fuller, standalone system and is intentionally independent of
//! the simpler launcher `focus.rs` FocusGraph.

mod controller;
mod direction;
mod event;
mod graph;
mod state;
mod tab;

pub use controller::{Axis, BoundaryPolicy, FocusController, GridFocusController};
pub use direction::FocusDirection;
pub use event::{FocusEvent, KeyAction};
pub use graph::{FocusGraph, FocusNode, FocusableNode, NodeId};
pub use state::FocusState;
pub use tab::{AutoScrollMode, FocusTabModel, Visibility};
