//! Portable user profile model.
//!
//! Mirrors the fields of `com.xj.common.user.UserInfo` (`uid`, `nickname`,
//! `avatar`) and `UserProfileEntity` in GameHub Lite v5.1.8, plus the provider
//! that authenticated the account. The reference `uid` is an `int`; here it is
//! an `i64` so string-style platform uids can be represented without loss.

use super::auth_provider::AuthProvider;

/// The signed-in user's public profile.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UserProfile {
    /// Platform / account uid.
    pub uid: i64,
    /// Display nickname.
    pub nickname: String,
    /// Avatar URL (empty when the user has not set one).
    pub avatar_url: String,
    /// Provider that authenticated this account.
    pub provider: AuthProvider,
}

impl UserProfile {
    /// Construct a profile.
    pub fn new(
        uid: i64,
        nickname: impl Into<String>,
        avatar_url: impl Into<String>,
        provider: AuthProvider,
    ) -> Self {
        Self {
            uid,
            nickname: nickname.into(),
            avatar_url: avatar_url.into(),
            provider,
        }
    }

    /// True when the user has a non-empty avatar URL.
    pub fn has_avatar(&self) -> bool {
        !self.avatar_url.is_empty()
    }
}

impl Default for UserProfile {
    fn default() -> Self {
        Self {
            uid: 0,
            nickname: String::new(),
            avatar_url: String::new(),
            provider: AuthProvider::Google,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn constructs_profile_with_provider() {
        let p = UserProfile::new(7, "player7", "https://img.test/a.png", AuthProvider::QQ);
        assert_eq!(p.uid, 7);
        assert_eq!(p.nickname, "player7");
        assert_eq!(p.provider, AuthProvider::QQ);
        assert!(p.has_avatar());
    }

    #[test]
    fn empty_avatar_reports_no_avatar() {
        let p = UserProfile::new(1, "n", "", AuthProvider::Steam);
        assert!(!p.has_avatar());
    }

    #[test]
    fn default_is_unpopulated() {
        let p = UserProfile::default();
        assert_eq!(p.uid, 0);
        assert!(p.nickname.is_empty());
        assert_eq!(p.provider, AuthProvider::Google);
    }
}
