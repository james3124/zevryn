//! Identity-only model of the sign-in providers offered by the GameHub Lite
//! v5.1.8 guide login screen (`GuideLoginActivity`).
//!
//! The reference app exposes Google / Apple (via FirebaseAuth), QQ / WeChat
//! (via `LoginPlatformEnum`: `Google, Apple, QQ, WeiXin`), phone & e-mail code
//! login (`GuidePhoneValidateActivity`, `GuideEmailValidateActivity`,
//! `GuideInputValidateCodeActivity`), Ali one-key login (`OneKeyAliHelper`) and
//! Steam linking. Each provider here is a pure identity tag: no SDK, protocol,
//! credential or network behavior is modeled.

/// A sign-in provider identity (no protocol or SDK behavior).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum AuthProvider {
    /// Firebase `google.com` provider (`googleLoginBtn`).
    Google,
    /// Firebase `apple.com` provider (`appleLoginBtn`).
    Apple,
    /// `LoginPlatformEnum.QQ` / `QQLoginToken`.
    QQ,
    /// `LoginPlatformEnum.WeiXin` / `wxLoginBtn` (`WxAccessToken`).
    WeChat,
    /// Phone number + SMS code (`GuidePhoneValidateActivity`).
    Phone,
    /// E-mail address + code (`GuideEmailValidateActivity`).
    Email,
    /// Ali one-key number authentication (`OneKeyAliHelper`).
    AliOneKey,
    /// Steam account linking (`SteamUserInfo`).
    Steam,
}

impl AuthProvider {
    /// Every provider, in the order presented by the guide login screen.
    pub const ALL: [Self; 8] = [
        Self::Google,
        Self::Apple,
        Self::QQ,
        Self::WeChat,
        Self::Phone,
        Self::Email,
        Self::AliOneKey,
        Self::Steam,
    ];

    /// Stable wire identifier for the provider, mirroring the provider ids and
    /// platform tags used by the reference sources.
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Google => "google.com",
            Self::Apple => "apple.com",
            Self::QQ => "qq",
            Self::WeChat => "weixin",
            Self::Phone => "phone",
            Self::Email => "email",
            Self::AliOneKey => "ali_one_key",
            Self::Steam => "steam",
        }
    }

    /// Parse a wire identifier into a provider.
    pub fn from_str(s: &str) -> Option<Self> {
        Self::ALL.iter().copied().find(|p| p.as_str() == s)
    }

    /// True when the provider is a third-party social account
    /// (`LoginPlatformEnum` values plus Steam).
    pub const fn is_social(self) -> bool {
        matches!(
            self,
            Self::Google | Self::Apple | Self::QQ | Self::WeChat | Self::Steam
        )
    }
}

impl core::fmt::Display for AuthProvider {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_str(self.as_str())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn has_all_eight_providers() {
        assert_eq!(AuthProvider::ALL.len(), 8);
    }

    #[test]
    fn round_trips_wire_identifiers() {
        for provider in AuthProvider::ALL {
            assert_eq!(AuthProvider::from_str(provider.as_str()), Some(provider));
            assert_eq!(provider.to_string(), provider.as_str());
        }
    }

    #[test]
    fn firebase_provider_ids_are_preserved() {
        assert_eq!(AuthProvider::Google.as_str(), "google.com");
        assert_eq!(AuthProvider::Apple.as_str(), "apple.com");
    }

    #[test]
    fn social_classification() {
        assert!(AuthProvider::Google.is_social());
        assert!(AuthProvider::Steam.is_social());
        assert!(!AuthProvider::Phone.is_social());
        assert!(!AuthProvider::Email.is_social());
        assert!(!AuthProvider::AliOneKey.is_social());
    }

    #[test]
    fn rejects_unknown_wire_identifier() {
        assert_eq!(AuthProvider::from_str(""), None);
        assert_eq!(AuthProvider::from_str("twitter"), None);
    }
}
