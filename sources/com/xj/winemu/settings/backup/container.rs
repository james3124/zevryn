//! Result model for `PcGameDataHandler.checkGameVirtualContainerExist`.
//!
//! The original returns `Pair<File?, String>`: a `File` pointing at the
//! container directory when the emulator service resolves an `IEmuContainer`,
//! otherwise `null` plus the "install dependency / container" prompt string.
//! Native file handles and resource IDs are replaced with plain data here.

/// Outcome of checking whether a game's virtual container exists.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VirtualContainerCheck {
    pub exists: bool,
    /// Absolute container path when one was found.
    pub path: Option<String>,
    /// User-facing message (empty when a container was found).
    pub message: String,
}

impl VirtualContainerCheck {
    /// A resolved container (`Pair(File, "")` in the source).
    pub fn found(path: impl Into<String>) -> Self {
        Self {
            exists: true,
            path: Some(path.into()),
            message: String::new(),
        }
    }

    /// A missing container (`Pair(null, message)` in the source).
    pub fn missing(message: impl Into<String>) -> Self {
        Self {
            exists: false,
            path: None,
            message: message.into(),
        }
    }

    pub fn path_str(&self) -> Option<&str> {
        self.path.as_deref()
    }

    pub fn is_missing(&self) -> bool {
        !self.exists
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn found_carries_path_and_empty_message() {
        let check = VirtualContainerCheck::found("/data/containers/game1");
        assert!(check.exists);
        assert_eq!(check.path_str(), Some("/data/containers/game1"));
        assert!(check.message.is_empty());
    }

    #[test]
    fn missing_carries_message_and_no_path() {
        let check = VirtualContainerCheck::missing("winemu_install_dep_tips_container");
        assert!(check.is_missing());
        assert!(check.path.is_none());
        assert_eq!(check.message, "winemu_install_dep_tips_container");
    }
}
