//! Reset scopes and a deterministic reset-plan builder.
//!
//! `PcGameDataHandler.resetGameData(gameId)` delegates to the emulator service
//! and clears the per-game data (translation config, env params, exe path).
//! `PcGameSettingDataHelper.resetAllData` clears every `SPUtils` store plus the
//! translation-config repository, while the virtual container is a separate
//! directory tree. These three behaviours map onto [`ResetScope`].

/// A concrete thing a reset removes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ResetTarget {
    /// Per-game translation / configuration repository data.
    TranslationConfig,
    /// `KEY=VALUE` environment-variable string.
    EnvParams,
    /// Locally chosen local PC executable path.
    ExePath,
    /// The game's virtual container directory (the ZIP payload).
    VirtualContainer,
    /// Global `SPUtils` preference stores (`resetAllData`).
    GlobalPreferences,
    /// Cached component / file data.
    Caches,
}

impl ResetTarget {
    pub fn as_str(self) -> &'static str {
        match self {
            ResetTarget::TranslationConfig => "translation_config",
            ResetTarget::EnvParams => "env_params",
            ResetTarget::ExePath => "exe_path",
            ResetTarget::VirtualContainer => "virtual_container",
            ResetTarget::GlobalPreferences => "global_preferences",
            ResetTarget::Caches => "caches",
        }
    }
}

/// How much of the game's data a reset should erase.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ResetScope {
    /// Only per-game settings (mirrors `resetGameData`).
    SettingsOnly,
    /// Per-game settings plus the virtual container directory.
    SettingsAndContainer,
    /// Everything: settings, container, global preferences and caches
    /// (mirrors `resetAllData` plus container removal).
    Full,
}

/// An ordered, scope-derived list of reset targets.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResetPlan {
    pub scope: ResetScope,
    pub targets: Vec<ResetTarget>,
}

impl ResetPlan {
    /// Build the canonical target list for a scope.
    pub fn new(scope: ResetScope) -> Self {
        let targets = match scope {
            ResetScope::SettingsOnly => vec![
                ResetTarget::TranslationConfig,
                ResetTarget::EnvParams,
                ResetTarget::ExePath,
            ],
            ResetScope::SettingsAndContainer => vec![
                ResetTarget::TranslationConfig,
                ResetTarget::EnvParams,
                ResetTarget::ExePath,
                ResetTarget::VirtualContainer,
            ],
            ResetScope::Full => vec![
                ResetTarget::TranslationConfig,
                ResetTarget::EnvParams,
                ResetTarget::ExePath,
                ResetTarget::VirtualContainer,
                ResetTarget::GlobalPreferences,
                ResetTarget::Caches,
            ],
        };
        Self { scope, targets }
    }

    /// Add a target if not already present, preserving order.
    pub fn with_target(mut self, target: ResetTarget) -> Self {
        if !self.targets.contains(&target) {
            self.targets.push(target);
        }
        self
    }

    /// Remove a target from the plan.
    pub fn without_target(mut self, target: ResetTarget) -> Self {
        self.targets.retain(|t| *t != target);
        self
    }

    pub fn contains(&self, target: ResetTarget) -> bool {
        self.targets.contains(&target)
    }

    pub fn len(&self) -> usize {
        self.targets.len()
    }

    pub fn is_empty(&self) -> bool {
        self.targets.is_empty()
    }

    pub fn affects_container(&self) -> bool {
        self.contains(ResetTarget::VirtualContainer)
    }

    pub fn is_full(&self) -> bool {
        self.scope == ResetScope::Full
    }

    /// Ordered labels for reporting / logging.
    pub fn labels(&self) -> Vec<&'static str> {
        self.targets.iter().map(|t| t.as_str()).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn settings_only_targets_match_reset_game_data() {
        let plan = ResetPlan::new(ResetScope::SettingsOnly);
        assert_eq!(
            plan.targets,
            vec![
                ResetTarget::TranslationConfig,
                ResetTarget::EnvParams,
                ResetTarget::ExePath,
            ]
        );
        assert!(!plan.affects_container());
        assert!(!plan.is_full());
    }

    #[test]
    fn settings_and_container_adds_container_last() {
        let plan = ResetPlan::new(ResetScope::SettingsAndContainer);
        assert_eq!(plan.len(), 4);
        assert_eq!(plan.targets.last(), Some(&ResetTarget::VirtualContainer));
        assert!(plan.affects_container());
    }

    #[test]
    fn full_includes_global_preferences_and_caches() {
        let plan = ResetPlan::new(ResetScope::Full);
        assert!(plan.is_full());
        assert!(plan.contains(ResetTarget::GlobalPreferences));
        assert!(plan.contains(ResetTarget::Caches));
        assert!(plan.contains(ResetTarget::VirtualContainer));
        assert_eq!(
            plan.labels(),
            vec![
                "translation_config",
                "env_params",
                "exe_path",
                "virtual_container",
                "global_preferences",
                "caches",
            ]
        );
    }

    #[test]
    fn builder_add_and_remove_are_ordered_and_idempotent() {
        let plan = ResetPlan::new(ResetScope::SettingsOnly)
            .with_target(ResetTarget::VirtualContainer)
            .with_target(ResetTarget::VirtualContainer) // no duplicate
            .without_target(ResetTarget::EnvParams);
        assert_eq!(plan.len(), 3);
        assert_eq!(
            plan.targets,
            vec![
                ResetTarget::TranslationConfig,
                ResetTarget::ExePath,
                ResetTarget::VirtualContainer,
            ]
        );
    }
}
