//! Rust model of the `com.xj.common.deeplink` package (portable logic only).
//!
//! Mirrors `sources/com/xj/common/deeplink/` from GameHub Lite v5.1.8. The
//! Android-side router (`GameHubDeepLinkManager`'s coroutine/Intent plumbing)
//! is intentionally left out; only the page enum and URI parser are modeled,
//! matching the core crate's `deeplink` module.

pub mod game_hub_deep_link_manager;
pub mod gh_deep_link_page_type;

pub use self::game_hub_deep_link_manager::{parse, DeepLink, ParseError, SCHEME_HOST};
pub use self::gh_deep_link_page_type::DeepLinkPage;
