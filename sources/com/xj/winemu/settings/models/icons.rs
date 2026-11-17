//! Rust model of `com.xj.winemu.settings.models.Icons` (GameHub Lite v5.1.8).
//!
//! Source: `sources/com/xj/winemu/settings/models/Icons.java`. The Java enum
//! has a single constant, `launch_container`, whose only method is
//! `setImage(ImageView, boolean)` — that is Android view binding and is out of
//! scope here. Only the portable identity of the icon is modeled.

/// Portable identity of a setting icon.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum Icons {
    /// `Icons.launch_container` — "open container / desktop" action icon.
    #[default]
    LaunchContainer,
}

impl Icons {
    /// Every variant in declaration order.
    pub const ALL: [Self; 1] = [Self::LaunchContainer];

    /// The Java enum constant name (`name()`), used by Parcelable serialization.
    pub fn as_str(self) -> &'static str {
        match self {
            Self::LaunchContainer => "launch_container",
        }
    }

    /// Inverse of [`Icons::as_str`] (`Icons.valueOf(name)`).
    pub fn from_name(name: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|v| v.as_str() == name)
    }
}

impl core::fmt::Display for Icons {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_str(self.as_str())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn icons_name_round_trip() {
        assert_eq!(Icons::LaunchContainer.as_str(), "launch_container");
        assert_eq!(Icons::from_name("launch_container"), Some(Icons::LaunchContainer));
        assert_eq!(Icons::from_name("missing"), None);
        assert_eq!(Icons::ALL.len(), 1);
    }
}
