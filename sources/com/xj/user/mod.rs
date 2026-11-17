//! Rust model of the `com.xj.user` package (portable logic only).
//!
//! Mirrors the identity / authentication concepts used by GameHub Lite v5.1.8
//! around `com.xj.user`, `LoginPlatformEnum`, `UserInfo` and the guide login
//! activities. Android UI, SDK callbacks and network calls are intentionally
//! left out; only identity enums, data models and the auth state machine are
//! reproduced.

pub mod auth_provider;
pub mod auth_state;
pub mod token_bundle;
pub mod user_profile;

pub use self::auth_provider::AuthProvider;
pub use self::auth_state::{
    can_transition, transition, AuthEffect, AuthEvent, AuthFailure, AuthState, AuthStateMachine,
    InvalidTransition,
};
pub use self::token_bundle::TokenBundle;
pub use self::user_profile::UserProfile;
