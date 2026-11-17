//! Rust model of the portable part of `defpackage.ComponentRepo`
//! (GameHub Lite v5.1.8).
//!
//! The Java data class tracks a Wine/emulator component: name, version,
//! [`ComponentState`], plus two flags (`isDep`, `isBase`). Two of its fields —
//! `entry: EnvLayerEntity` and `depInfo: DependencyManager.Companion.Info` —
//! are Android/xj-specific and are deliberately omitted; only the portable
//! fields that describe component identity and install state are kept.

use super::state::ComponentState;

/// Portable identity/state of one installable component.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ComponentRepo {
    pub name: String,
    pub version: String,
    pub state: ComponentState,
    /// Mirrors `isDep`: this component is pulled in as a dependency.
    pub is_dep: bool,
    /// Mirrors `isBase`: this component belongs to the base environment.
    pub is_base: bool,
}

impl ComponentRepo {
    pub fn new(name: impl Into<String>, version: impl Into<String>, state: ComponentState) -> Self {
        Self {
            name: name.into(),
            version: version.into(),
            state,
            is_dep: false,
            is_base: false,
        }
    }

    /// `getComponentFile()`/`getComponentPath()` use `name` as a child of the
    /// WinEmu component directory; the portable part is just that child name.
    pub fn component_file_name(&self) -> &str {
        &self.name
    }

    pub fn is_installed(&self) -> bool {
        self.state.is_installed()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn component_repo_identity() {
        let repo = ComponentRepo::new("wine", "9.0", ComponentState::Downloaded);
        assert_eq!(repo.component_file_name(), "wine");
        assert!(!repo.is_installed());
        assert!(!repo.is_dep && !repo.is_base);
    }

    #[test]
    fn installed_component() {
        let repo = ComponentRepo {
            name: "box64".into(),
            version: "0.3".into(),
            state: ComponentState::Installed,
            is_dep: true,
            is_base: false,
        };
        assert!(repo.is_installed());
        assert!(repo.is_dep);
    }
}
