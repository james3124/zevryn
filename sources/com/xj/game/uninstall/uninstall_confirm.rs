//! Result model for the confirmation / removal callback.
//!
//! Exact port of `com.xj.common.utils.RemoveOrUninstallGameListener`
//! (`RemoveOrUninstallGameListener.java`, line 17):
//!
//! ```java
//! void a(boolean z3, boolean z4, String str);
//! ```
//!
//! * `success` (`z3`) — whether the overall operation succeeded. In
//!   `toUninstallPcGame$1` this is the result of `removeFromGameLibrary` and is
//!   also set on exceptions.
//! * `removed` (`z4`) — whether something was actually removed from the
//!   library / whether the confirm checkbox (delete offline files) was set.
//! * `message` (`str`) — `""` on the normal path, `"remove failure"` on the
//!   error path and app-store / activity error strings on the dialog path.

/// The `(boolean, boolean, String)` triple delivered to
/// `RemoveOrUninstallGameListener.a`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UninstallConfirm {
    pub success: bool,
    pub removed: bool,
    pub message: String,
}

impl UninstallConfirm {
    pub fn new(success: bool, removed: bool, message: impl Into<String>) -> Self {
        UninstallConfirm {
            success,
            removed,
            message: message.into(),
        }
    }

    /// Normal successful completion (`a(true, removed, "")`).
    pub fn ok(removed: bool) -> Self {
        UninstallConfirm::new(true, removed, "")
    }

    /// Failure with a message (`success = false`, `removed = false`).
    pub fn failure(message: impl Into<String>) -> Self {
        UninstallConfirm::new(false, false, message)
    }

    /// The Java helper's error-path message.
    pub fn remove_failure() -> Self {
        UninstallConfirm::failure("remove failure")
    }

    /// The dialog-path cancellation / invalid-state messages.
    pub fn cancelled() -> Self {
        UninstallConfirm::failure("cancelled")
    }

    pub fn is_success(&self) -> bool {
        self.success
    }

    pub fn is_removed(&self) -> bool {
        self.removed
    }
}

/// Rust mirror of the `RemoveOrUninstallGameListener` interface.
pub trait UninstallConfirmListener {
    fn on_result(&self, result: &UninstallConfirm);
}

/// A no-op listener, useful when a caller only needs the returned model.
pub struct NoopConfirmListener;

impl UninstallConfirmListener for NoopConfirmListener {
    fn on_result(&self, _result: &UninstallConfirm) {}
}

impl<F: Fn(&UninstallConfirm)> UninstallConfirmListener for F {
    fn on_result(&self, result: &UninstallConfirm) {
        self(result);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::RefCell;

    #[test]
    fn confirm_models_callback_triple() {
        let ok = UninstallConfirm::ok(true);
        assert_eq!(
            ok,
            UninstallConfirm {
                success: true,
                removed: true,
                message: String::new(),
            }
        );
        assert_eq!(UninstallConfirm::remove_failure().message, "remove failure");
        assert!(!UninstallConfirm::cancelled().is_success());
        let custom = UninstallConfirm::new(true, false, "activity is destroyed");
        assert!(custom.is_success());
        assert!(!custom.is_removed());
        assert_eq!(custom.message, "activity is destroyed");
    }

    #[test]
    fn listener_receives_result() {
        let seen = RefCell::new(Vec::new());
        {
            let listener = |r: &UninstallConfirm| seen.borrow_mut().push(r.clone());
            listener.on_result(&UninstallConfirm::ok(false));
        }
        assert_eq!(seen.into_inner(), vec![UninstallConfirm::ok(false)]);
    }
}
