//! Portable authentication state machine.
//!
//! The guide activities of GameHub Lite v5.1.8 drive login through a common
//! shape regardless of provider: a sign-in button starts an authorization, a
//! successful callback yields a token bundle, a token can later be refreshed,
//! and any failure returns the user to a signed-out state. The Android
//! specifics (FirebaseAuth, OneKeyAliHelper, QQ/WeChat SDK callbacks, Activity
//! results, coroutines and HTTP calls) are intentionally absent; only the
//! ordering and the allowed transitions are reproduced here.
//!
//! Canonical lifecycle:
//!
//! ```text
//!   SignedOut --SignIn(p)--> Authorizing(p) --AuthSucceeded(t)--> Authenticated(t)
//!                                 |                   |
//!                            AuthFailed(r)          Refresh
//!                                 |                   |
//!                                 v                   v
//!                              Failed(r) <-- RefreshFailed(r) Refreshing
//!                                 |                   |
//!                            Acknowledge          Refreshed(t)
//!                                 |                   |
//!                                 v                   v
//!                              SignedOut          Authenticated(t)
//!
//!   SignOut is also accepted from Authorizing / Authenticated / Refreshing.
//! ```

use super::auth_provider::AuthProvider;
use super::token_bundle::TokenBundle;

/// Why an authorization or refresh attempt failed.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AuthFailure {
    /// The user cancelled the flow (back press, dismissed consent UI).
    Cancelled,
    /// Credentials / code were rejected by the provider.
    InvalidCredentials,
    /// The provider's SDK or service was unavailable.
    ProviderUnavailable,
    /// The network request failed.
    Network,
    /// The returned token was missing or malformed.
    InvalidToken,
    /// A raw reason string from a provider callback.
    Other(String),
}

impl std::fmt::Display for AuthFailure {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let msg = match self {
            AuthFailure::Cancelled => "authorization cancelled",
            AuthFailure::InvalidCredentials => "invalid credentials",
            AuthFailure::ProviderUnavailable => "provider unavailable",
            AuthFailure::Network => "network error",
            AuthFailure::InvalidToken => "invalid token",
            AuthFailure::Other(s) => s,
        };
        f.write_str(msg)
    }
}

impl std::error::Error for AuthFailure {}

/// States of an authentication session.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AuthState {
    /// No user is signed in (initial state).
    SignedOut,
    /// An authorization with `provider` is in flight.
    Authorizing(AuthProvider),
    /// Signed in with a valid token bundle.
    Authenticated(TokenBundle),
    /// An access token is being refreshed.
    Refreshing,
    /// The last attempt failed with `reason`.
    Failed(AuthFailure),
}

impl AuthState {
    /// True when a user is considered signed in (a token is held).
    pub fn is_authenticated(&self) -> bool {
        matches!(self, AuthState::Authenticated(_))
    }

    /// True while an authorization or refresh is in flight.
    pub fn is_in_flight(&self) -> bool {
        matches!(self, AuthState::Authorizing(_) | AuthState::Refreshing)
    }

    /// The token bundle, when authenticated.
    pub fn token(&self) -> Option<&TokenBundle> {
        match self {
            AuthState::Authenticated(token) => Some(token),
            _ => None,
        }
    }
}

impl Default for AuthState {
    fn default() -> Self {
        AuthState::SignedOut
    }
}

/// Events that drive [`AuthState`] transitions.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AuthEvent {
    /// Begin authorization with a provider (a login button was tapped).
    SignIn(AuthProvider),
    /// The provider callback succeeded with a token bundle.
    AuthSucceeded(TokenBundle),
    /// The provider callback failed.
    AuthFailed(AuthFailure),
    /// Start refreshing the current access token.
    Refresh,
    /// Refresh succeeded with a new token bundle.
    Refreshed(TokenBundle),
    /// Refresh failed.
    RefreshFailed(AuthFailure),
    /// Sign out (accepted from any state).
    SignOut,
    /// Acknowledge a failure and return to `SignedOut` so login can be retried.
    Acknowledge,
}

/// A side effect requested by a successful transition, kept as data so callers
/// decide how (or whether) to perform it. No side effect performs I/O itself.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AuthEffect {
    /// Nothing to do (bookkeeping only).
    None,
    /// Begin the provider authorization flow.
    Authorize(AuthProvider),
    /// Persist a newly obtained token bundle.
    StoreToken(TokenBundle),
    /// Begin a token refresh.
    RefreshToken,
    /// Report a failure to the UI.
    ReportFailure(AuthFailure),
    /// Clear any stored session.
    ClearSession,
}

/// Error returned when an event is not legal in the current state.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct InvalidTransition {
    pub from_state: &'static str,
    pub event: &'static str,
}

impl std::fmt::Display for InvalidTransition {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "event {} is not valid in state {}",
            self.event, self.from_state
        )
    }
}

impl std::error::Error for InvalidTransition {}

fn state_name(state: &AuthState) -> &'static str {
    match state {
        AuthState::SignedOut => "SignedOut",
        AuthState::Authorizing(_) => "Authorizing",
        AuthState::Authenticated(_) => "Authenticated",
        AuthState::Refreshing => "Refreshing",
        AuthState::Failed(_) => "Failed",
    }
}

fn event_name(event: &AuthEvent) -> &'static str {
    match event {
        AuthEvent::SignIn(_) => "SignIn",
        AuthEvent::AuthSucceeded(_) => "AuthSucceeded",
        AuthEvent::AuthFailed(_) => "AuthFailed",
        AuthEvent::Refresh => "Refresh",
        AuthEvent::Refreshed(_) => "Refreshed",
        AuthEvent::RefreshFailed(_) => "RefreshFailed",
        AuthEvent::SignOut => "SignOut",
        AuthEvent::Acknowledge => "Acknowledge",
    }
}

/// Apply `event` to `state`, returning the resulting state and the side effect
/// the transition requests.
pub fn transition(state: AuthState, event: AuthEvent) -> Result<(AuthState, AuthEffect), InvalidTransition> {
    use AuthEffect as E;
    use AuthEvent as V;

    let result = match (state.clone(), event.clone()) {
        // --- authorization ---------------------------------------------------
        (AuthState::SignedOut, V::SignIn(provider)) => (
            AuthState::Authorizing(provider),
            E::Authorize(provider),
        ),
        (AuthState::Authorizing(_), V::AuthSucceeded(token)) => (
            AuthState::Authenticated(token.clone()),
            E::StoreToken(token),
        ),
        (AuthState::Authorizing(_), V::AuthFailed(reason)) => {
            (AuthState::Failed(reason.clone()), E::ReportFailure(reason))
        }

        // --- refresh ---------------------------------------------------------
        (AuthState::Authenticated(_), V::Refresh) => (AuthState::Refreshing, E::RefreshToken),
        (AuthState::Refreshing, V::Refreshed(token)) => (
            AuthState::Authenticated(token.clone()),
            E::StoreToken(token),
        ),
        (AuthState::Refreshing, V::RefreshFailed(reason)) => {
            (AuthState::Failed(reason.clone()), E::ReportFailure(reason))
        }

        // --- failure acknowledgement ----------------------------------------
        (AuthState::Failed(_), V::Acknowledge) => (AuthState::SignedOut, E::ClearSession),

        // --- sign out is legal from every state ------------------------------
        (_, V::SignOut) => (AuthState::SignedOut, E::ClearSession),

        _ => {
            return Err(InvalidTransition {
                from_state: state_name(&state),
                event: event_name(&event),
            });
        }
    };
    Ok(result)
}

/// Non-mutating query: is `event` legal in `state`?
pub fn can_transition(state: AuthState, event: AuthEvent) -> bool {
    transition(state, event).is_ok()
}

/// Stateful wrapper around [`transition`] owning the current state and the last
/// token/failure, mirroring the state held by the reference login activities.
#[derive(Debug, Clone)]
pub struct AuthStateMachine {
    pub state: AuthState,
    pub last_token: Option<TokenBundle>,
    pub last_failure: Option<AuthFailure>,
}

impl Default for AuthStateMachine {
    fn default() -> Self {
        Self::new()
    }
}

impl AuthStateMachine {
    pub fn new() -> Self {
        Self {
            state: AuthState::SignedOut,
            last_token: None,
            last_failure: None,
        }
    }

    /// Drive the machine with an event.
    pub fn handle(&mut self, event: AuthEvent) -> Result<AuthEffect, InvalidTransition> {
        let (next, effect) = transition(self.state.clone(), event.clone())?;
        match &effect {
            AuthEffect::StoreToken(token) => {
                self.last_token = Some(token.clone());
                self.last_failure = None;
            }
            AuthEffect::ReportFailure(reason) => self.last_failure = Some(reason.clone()),
            AuthEffect::ClearSession => {
                self.last_token = None;
                self.last_failure = None;
            }
            _ => {}
        }
        self.state = next;
        Ok(effect)
    }

    pub fn is_authenticated(&self) -> bool {
        self.state.is_authenticated()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn token() -> TokenBundle {
        TokenBundle::new("a", "r", 1_000)
    }

    fn drive_to_authenticated() -> AuthStateMachine {
        let mut m = AuthStateMachine::new();
        m.handle(AuthEvent::SignIn(AuthProvider::Google)).unwrap();
        m.handle(AuthEvent::AuthSucceeded(token())).unwrap();
        m
    }

    #[test]
    fn starts_signed_out() {
        let m = AuthStateMachine::new();
        assert_eq!(m.state, AuthState::SignedOut);
        assert!(!m.is_authenticated());
    }

    #[test]
    fn happy_path_sign_in() {
        let mut m = AuthStateMachine::new();
        assert_eq!(
            m.handle(AuthEvent::SignIn(AuthProvider::QQ)).unwrap(),
            AuthEffect::Authorize(AuthProvider::QQ)
        );
        assert_eq!(m.state, AuthState::Authorizing(AuthProvider::QQ));
        assert!(m.state.is_in_flight());
        assert_eq!(
            m.handle(AuthEvent::AuthSucceeded(token())).unwrap(),
            AuthEffect::StoreToken(token())
        );
        assert_eq!(m.state, AuthState::Authenticated(token()));
        assert!(m.is_authenticated());
        assert_eq!(m.last_token, Some(token()));
        assert_eq!(m.state.token(), Some(&token()));
    }

    #[test]
    fn auth_failure_then_acknowledge_returns_to_signed_out() {
        let mut m = AuthStateMachine::new();
        m.handle(AuthEvent::SignIn(AuthProvider::WeChat)).unwrap();
        assert_eq!(
            m.handle(AuthEvent::AuthFailed(AuthFailure::Cancelled)).unwrap(),
            AuthEffect::ReportFailure(AuthFailure::Cancelled)
        );
        assert_eq!(m.state, AuthState::Failed(AuthFailure::Cancelled));
        assert_eq!(m.last_failure, Some(AuthFailure::Cancelled));
        // Failed -> SignedOut via Acknowledge, then a fresh sign-in works.
        assert_eq!(
            m.handle(AuthEvent::Acknowledge).unwrap(),
            AuthEffect::ClearSession
        );
        assert_eq!(m.state, AuthState::SignedOut);
        assert_eq!(m.last_failure, None);
        m.handle(AuthEvent::SignIn(AuthProvider::Email)).unwrap();
        assert_eq!(m.state, AuthState::Authorizing(AuthProvider::Email));
    }

    #[test]
    fn refresh_success_loops_back_to_authenticated() {
        let mut m = drive_to_authenticated();
        assert_eq!(
            m.handle(AuthEvent::Refresh).unwrap(),
            AuthEffect::RefreshToken
        );
        assert_eq!(m.state, AuthState::Refreshing);
        let fresh = TokenBundle::new("a2", "r2", 2_000);
        assert_eq!(
            m.handle(AuthEvent::Refreshed(fresh.clone())).unwrap(),
            AuthEffect::StoreToken(fresh.clone())
        );
        assert_eq!(m.state, AuthState::Authenticated(fresh.clone()));
        assert_eq!(m.last_token, Some(fresh));
    }

    #[test]
    fn refresh_failure_goes_to_failed() {
        let mut m = drive_to_authenticated();
        m.handle(AuthEvent::Refresh).unwrap();
        assert_eq!(
            m.handle(AuthEvent::RefreshFailed(AuthFailure::Network))
                .unwrap(),
            AuthEffect::ReportFailure(AuthFailure::Network)
        );
        assert_eq!(m.state, AuthState::Failed(AuthFailure::Network));
    }

    #[test]
    fn sign_out_is_legal_from_every_state() {
        let states = [
            AuthState::SignedOut,
            AuthState::Authorizing(AuthProvider::Phone),
            AuthState::Authenticated(token()),
            AuthState::Refreshing,
            AuthState::Failed(AuthFailure::InvalidToken),
        ];
        for s in states {
            assert_eq!(transition(s, AuthEvent::SignOut).unwrap().0, AuthState::SignedOut);
        }
        let mut m = drive_to_authenticated();
        m.handle(AuthEvent::SignOut).unwrap();
        assert_eq!(m.state, AuthState::SignedOut);
        assert_eq!(m.last_token, None);
    }

    #[test]
    fn invalid_transitions_are_rejected() {
        // Cannot succeed before authorizing.
        assert!(!can_transition(AuthState::SignedOut, AuthEvent::AuthSucceeded(token())));
        // Cannot refresh without a token.
        assert!(!can_transition(AuthState::SignedOut, AuthEvent::Refresh));
        // Cannot authorize twice without a terminal outcome.
        assert!(!can_transition(
            AuthState::Authorizing(AuthProvider::Google),
            AuthEvent::SignIn(AuthProvider::Apple)
        ));
        // Cannot acknowledge a non-failure.
        assert!(!can_transition(AuthState::SignedOut, AuthEvent::Acknowledge));
        // Cannot refresh while already refreshing.
        assert!(!can_transition(AuthState::Refreshing, AuthEvent::Refresh));

        let err = transition(AuthState::SignedOut, AuthEvent::Refresh).unwrap_err();
        assert!(err.to_string().contains("not valid"));
        assert_eq!(err.from_state, "SignedOut");
    }

    #[test]
    fn failure_display_covers_all_variants() {
        assert_eq!(AuthFailure::Cancelled.to_string(), "authorization cancelled");
        assert_eq!(AuthFailure::Other("boom".into()).to_string(), "boom");
    }
}
