//! Rust model of `com.xj.landscape.launcher.ui.guide` (portable logic only).
//!
//! The reference package holds the first-launch `Guide*Activity` / fragment
//! screens. Only the ordered onboarding checklist is modeled here; Android UI
//! and permission/SDK plumbing are intentionally left out.

pub mod onboarding_step;

pub use self::onboarding_step::{OnboardingChecklist, OnboardingStep};
