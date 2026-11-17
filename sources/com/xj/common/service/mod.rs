//! Rust model of the `com.xj.common.service` package (portable logic only).
//!
//! Mirrors `sources/com/xj/common/service/` from GameHub Lite v5.1.8. The
//! concrete interfaces are TheRouter service-locator types full of Android
//! objects; here we keep only the portable navigation contract.

// Reuse the single deep-link definition from the sibling area so the seven
// page types are the same enum, not a second competing definition.
#[path = "../deeplink/mod.rs"]
pub mod deeplink;

pub mod nav_service;

pub use self::deeplink::DeepLinkPage;
pub use self::nav_service::{dispatch, NavError, NavOutcome, NavRequest, NavService};
