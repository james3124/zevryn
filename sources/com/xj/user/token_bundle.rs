//! Portable token bundle model.
//!
//! The reference app persists an access token, a refresh token and an expiry
//! timestamp for third-party / API sessions (e.g. `WxAccessToken`,
//! `QQLoginToken` and the "token" payload returned by login APIs). Only the
//! data shape and the expiry predicate are reproduced here. No token is
//! fetched, stored to disk or refreshed over the network.

/// A pair of access/refresh tokens together with an absolute expiry instant.
///
/// `expires_at` is a Unix timestamp in **seconds since the epoch**, matching
/// the integer expiry values used by the reference APIs. An expiry of `0` is
/// treated as "no expiry" so a bundle with an unknown lifetime is not
/// considered expired.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TokenBundle {
    /// Bearer token used to authenticate API calls.
    pub access_token: String,
    /// Token used to obtain a fresh access token.
    pub refresh_token: String,
    /// Absolute expiry as Unix seconds; `0` means "never expires".
    pub expires_at: u64,
}

impl TokenBundle {
    /// Construct a token bundle.
    pub fn new(access_token: impl Into<String>, refresh_token: impl Into<String>, expires_at: u64) -> Self {
        Self {
            access_token: access_token.into(),
            refresh_token: refresh_token.into(),
            expires_at,
        }
    }

    /// Construct a non-expiring token bundle (`expires_at == 0`).
    pub fn never_expires(access_token: impl Into<String>, refresh_token: impl Into<String>) -> Self {
        Self::new(access_token, refresh_token, 0)
    }

    /// True when `now` (Unix seconds) is at or past the expiry instant.
    ///
    /// A zero `expires_at` never expires.
    pub fn is_expired(&self, now: u64) -> bool {
        self.expires_at != 0 && now >= self.expires_at
    }

    /// Expiry check with a safety margin: the token is considered expired
    /// `leeway_secs` before its actual expiry, so refresh can be started early.
    pub fn is_expired_with_leeway(&self, now: u64, leeway_secs: u64) -> bool {
        if self.expires_at == 0 {
            return false;
        }
        now.saturating_add(leeway_secs) >= self.expires_at
    }

    /// Seconds remaining until expiry, or `None` when the token never expires.
    /// Saturates at zero once expired.
    pub fn remaining_secs(&self, now: u64) -> Option<u64> {
        if self.expires_at == 0 {
            None
        } else {
            Some(self.expires_at.saturating_sub(now))
        }
    }

    /// Convenience check against the current system time.
    pub fn is_expired_now(&self) -> bool {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);
        self.is_expired(now)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn bundle() -> TokenBundle {
        TokenBundle::new("access", "refresh", 1_000)
    }

    #[test]
    fn not_expired_before_expiry() {
        assert!(!bundle().is_expired(999));
        assert!(!bundle().is_expired(0));
    }

    #[test]
    fn expired_at_and_after_expiry() {
        assert!(bundle().is_expired(1_000));
        assert!(bundle().is_expired(1_001));
    }

    #[test]
    fn zero_expiry_never_expires() {
        let t = TokenBundle::never_expires("a", "r");
        assert!(!t.is_expired(u64::MAX));
        assert_eq!(t.remaining_secs(5), None);
    }

    #[test]
    fn leeway_marks_token_expired_early() {
        let t = bundle();
        assert!(!t.is_expired_with_leeway(980, 10));
        assert!(t.is_expired_with_leeway(990, 10));
        assert!(t.is_expired_with_leeway(995, 10));
    }

    #[test]
    fn remaining_seconds_saturate_at_zero() {
        let t = bundle();
        assert_eq!(t.remaining_secs(100), Some(900));
        assert_eq!(t.remaining_secs(1_000), Some(0));
        assert_eq!(t.remaining_secs(2_000), Some(0));
    }
}
