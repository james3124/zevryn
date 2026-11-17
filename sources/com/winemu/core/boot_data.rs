//! Rust model of `com.winemu.core.BootData` (GameHub Lite v5.1.8).
//!
//! Two mutable booleans with getters/setters (`a/b/c/d`). No Android or native
//! dependencies; modeled as a plain, default-constructible data struct.

/// Two boot flags carried through the emulation setup chain.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct BootData {
    /// Mirrors `f20194a` (`a()` / `c()`).
    pub first: bool,
    /// Mirrors `f20195b` (`b()` / `d()`).
    pub second: bool,
}

impl BootData {
    pub fn new(first: bool, second: bool) -> Self {
        Self { first, second }
    }

    pub fn a(&self) -> bool {
        self.first
    }

    pub fn b(&self) -> bool {
        self.second
    }

    pub fn c(&mut self, value: bool) {
        self.first = value;
    }

    pub fn d(&mut self, value: bool) {
        self.second = value;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn boot_data_getters_and_setters() {
        let mut bd = BootData::default();
        assert_eq!((bd.a(), bd.b()), (false, false));
        bd.c(true);
        bd.d(true);
        assert_eq!(bd, BootData::new(true, true));
        assert_eq!((bd.a(), bd.b()), (true, true));
    }
}
