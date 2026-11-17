//! Rust model of the launcher event package (GameHub Lite v5.1.8).
//!
//! Mirrors `sources/com/xj/landscape/launcher/event/` and its `com.drake.channel`
//! bus usage. Only portable logic is modeled: the event class names, a family
//! grouping, and a small synchronous broadcast bus with sticky delivery and a
//! dispatch log. Coroutine `Channel`, `LiveData`, `View` and other Android types
//! are intentionally out of scope (see `REPORT.md`).

pub mod event_bus;
pub mod launcher_event;

pub use self::event_bus::{DispatchRecord, EventBus, PublishResult, Subscription};
pub use self::launcher_event::{EventFamily, LauncherEvent};
