//! Generic asynchronous state container.
//!
//! The decompiled ViewModels expose their state through `MutableLiveData` /
//! `UnPeekLiveData` plus a separate `Boolean` "loading" LiveData (e.g.
//! `GameDetailVM.f30419a`, `NewLandscapeLauncherMainVM.f30447e`). This enum
//! folds those two channels into one portable, testable value.

/// The four states an asynchronous operation can be in.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AsyncState<T> {
    /// Nothing has been requested yet.
    Idle,
    /// A request is in flight; the previous value (if any) is not retained.
    Loading,
    /// The request completed with a value.
    Success(T),
    /// The request failed with an error message.
    Error(String),
}

impl<T> AsyncState<T> {
    /// Creates an idle state.
    pub fn new() -> Self {
        AsyncState::Idle
    }

    // --- queries ---------------------------------------------------------

    pub fn is_idle(&self) -> bool {
        matches!(self, AsyncState::Idle)
    }

    pub fn is_loading(&self) -> bool {
        matches!(self, AsyncState::Loading)
    }

    pub fn is_success(&self) -> bool {
        matches!(self, AsyncState::Success(_))
    }

    pub fn is_error(&self) -> bool {
        matches!(self, AsyncState::Error(_))
    }

    /// Borrows the success value, if present.
    pub fn value(&self) -> Option<&T> {
        match self {
            AsyncState::Success(v) => Some(v),
            _ => None,
        }
    }

    /// Mutably borrows the success value, if present.
    pub fn value_mut(&mut self) -> Option<&mut T> {
        match self {
            AsyncState::Success(v) => Some(v),
            _ => None,
        }
    }

    /// Borrows the error message, if present.
    pub fn error(&self) -> Option<&str> {
        match self {
            AsyncState::Error(e) => Some(e.as_str()),
            _ => None,
        }
    }

    /// Borrows the contained value or falls back to `default`.
    pub fn unwrap_or<'a>(&'a self, default: &'a T) -> &'a T {
        self.value().unwrap_or(default)
    }

    /// Maps the contained value through `f`, if present.
    pub fn map<U, F>(&self, f: F) -> Option<U>
    where
        F: FnOnce(&T) -> U,
    {
        self.value().map(f)
    }

    // --- transitions -----------------------------------------------------

    /// `Idle | Success | Error -> Loading`.
    pub fn begin(&mut self) {
        *self = AsyncState::Loading;
    }

    /// `* -> Success(value)`.
    pub fn succeed(&mut self, value: T) {
        *self = AsyncState::Success(value);
    }

    /// `* -> Error(message)`.
    pub fn fail<S: Into<String>>(&mut self, message: S) {
        *self = AsyncState::Error(message.into());
    }

    /// `* -> Idle`.
    pub fn reset(&mut self) {
        *self = AsyncState::Idle;
    }

    /// Moves the success value out, leaving the state `Idle`.
    pub fn take_value(&mut self) -> Option<T> {
        match std::mem::replace(self, AsyncState::Idle) {
            AsyncState::Success(v) => Some(v),
            other => {
                *self = other;
                None
            }
        }
    }

    /// Moves the success value out and drops the state entirely.
    pub fn into_value(self) -> Option<T> {
        match self {
            AsyncState::Success(v) => Some(v),
            _ => None,
        }
    }
}

impl<T> Default for AsyncState<T> {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_idle() {
        let s: AsyncState<i32> = AsyncState::new();
        assert!(s.is_idle());
        assert!(!s.is_loading());
        assert_eq!(s.value(), None);
        assert_eq!(s.error(), None);
        assert_eq!(AsyncState::<i32>::default(), s);
    }

    #[test]
    fn idle_to_loading_to_success() {
        let mut s = AsyncState::new();
        s.begin();
        assert!(s.is_loading());
        assert_eq!(s.value(), None);

        s.succeed(7);
        assert!(s.is_success());
        assert_eq!(s.value(), Some(&7));
        assert_eq!(s.map(|v| v * 2), Some(14));
        assert_eq!(s.unwrap_or(&0), &7);
        assert_eq!(s.error(), None);
    }

    #[test]
    fn success_to_error_and_failure_accessors() {
        let mut s = AsyncState::new();
        s.succeed(3);
        s.fail("boom");
        assert!(s.is_error());
        assert_eq!(s.error(), Some("boom"));
        assert_eq!(s.value(), None);
        assert_eq!(s.map(|v| v + 1), None);

        s.begin();
        assert!(!s.is_error());
        assert_eq!(s.error(), None);
    }

    #[test]
    fn take_value_restores_idle_and_preserves_other_states() {
        let mut s = AsyncState::new();
        s.succeed(String::from("ok"));
        assert_eq!(s.take_value().as_deref(), Some("ok"));
        assert!(s.is_idle());

        s.fail("still an error");
        assert_eq!(s.take_value(), None);
        assert_eq!(s.error(), Some("still an error"));

        assert_eq!(AsyncState::<i32>::new().into_value(), None);
        assert_eq!(AsyncState::Success(9).into_value(), Some(9));
    }

    #[test]
    fn value_mut_allows_in_place_update() {
        let mut s = AsyncState::new();
        s.succeed(vec![1, 2]);
        s.value_mut().unwrap().push(3);
        assert_eq!(s.value(), Some(&vec![1, 2, 3]));
        s.reset();
        assert!(s.is_idle());
    }
}
