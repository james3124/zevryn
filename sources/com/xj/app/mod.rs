//! Rust model of the `com.xj.app` package (portable logic only).
//!
//! Mirrors `sources/com/xj/app/` from GameHub Lite v5.1.8: the `Application`
//! boot chain and the splash-to-main hand-off. Android `Activity`/UI code
//! (SplashActivity's GIF/dialogs/USB-permission flow,
//! DeepLinkRouterActivity's coroutine bridge) is intentionally not modeled.

pub mod boot;

pub use self::boot::{AppBootPhase, BootSequence, BootStep, BOOT_STEPS};
