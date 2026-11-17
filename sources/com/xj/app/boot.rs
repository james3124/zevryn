//! Portable boot sequence for `com.xj.app.App` -> `SplashActivity` -> main.
//!
//! Models only ordering and state, not the Android lifecycle or UI. Each
//! [`BootStep`] names the Java method/class it corresponds to in the reference.

/// Coarse boot phases, in the order the reference app visits them.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AppBootPhase {
    /// `App.attachBaseContext(Context)` — reflection unlock + router pre-init.
    AttachBaseContext,
    /// `App.onCreate()` — locale, storage, router and feature-module init.
    OnCreate,
    /// `SplashActivity` — locale GIF, first-run/UMeng/intro-video checks,
    /// USB permission check, then hand-off.
    Splash,
    /// Main launcher activity.
    Main,
}

impl AppBootPhase {
    /// Every phase in boot order.
    pub const ALL: [Self; 4] = [
        Self::AttachBaseContext,
        Self::OnCreate,
        Self::Splash,
        Self::Main,
    ];

    /// The next phase, or `None` at the terminal `Main` phase.
    pub fn next(self) -> Option<Self> {
        match self {
            Self::AttachBaseContext => Some(Self::OnCreate),
            Self::OnCreate => Some(Self::Splash),
            Self::Splash => Some(Self::Main),
            Self::Main => None,
        }
    }

    pub fn index(self) -> usize {
        match self {
            Self::AttachBaseContext => 0,
            Self::OnCreate => 1,
            Self::Splash => 2,
            Self::Main => 3,
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Self::AttachBaseContext => "attach_base_context",
            Self::OnCreate => "on_create",
            Self::Splash => "splash",
            Self::Main => "main",
        }
    }
}

/// One concrete boot step. `reference` names the Java method/class modeled.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BootStep {
    pub phase: AppBootPhase,
    pub name: &'static str,
    pub reference: &'static str,
}

/// Ordered portable boot steps (Android view/IO details omitted).
pub const BOOT_STEPS: &[BootStep] = &[
    BootStep {
        phase: AppBootPhase::AttachBaseContext,
        name: "reflection_unlock",
        reference: "Reflection.b(context)",
    },
    BootStep {
        phase: AppBootPhase::AttachBaseContext,
        name: "router_preinit",
        reference: "TheRouter.q(false)",
    },
    BootStep {
        phase: AppBootPhase::OnCreate,
        name: "locale_init",
        reference: "GHLocaleManager.a.a(this)",
    },
    BootStep {
        phase: AppBootPhase::OnCreate,
        name: "storage_init",
        reference: "MMKV.x(this)",
    },
    BootStep {
        phase: AppBootPhase::OnCreate,
        name: "router_init",
        reference: "TheRouter.k(this, false)",
    },
    BootStep {
        phase: AppBootPhase::OnCreate,
        name: "im_service_init",
        reference: "IMService.f23993a.a().b(this)",
    },
    BootStep {
        phase: AppBootPhase::OnCreate,
        name: "feature_modules_init",
        reference: "WinEmuApp/LandscapeLauncherApp/SteamModuleApp init",
    },
    BootStep {
        phase: AppBootPhase::Splash,
        name: "first_run_check",
        reference: "SplashActivity.k1()",
    },
    BootStep {
        phase: AppBootPhase::Splash,
        name: "analytics_init",
        reference: "SplashActivity.l1(1)",
    },
    BootStep {
        phase: AppBootPhase::Splash,
        name: "gif_then_handoff",
        reference: "SplashActivity.j1() -> ILandscapeLauncherNavService.p()",
    },
    BootStep {
        phase: AppBootPhase::Main,
        name: "main_activity",
        reference: "LandscapeLauncherMainActivity",
    },
];

/// A small state machine walking [`BOOT_STEPS`] in order.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BootSequence {
    index: usize,
}

impl BootSequence {
    pub fn new() -> Self {
        Self { index: 0 }
    }

    /// Advance to and return the next step, or `None` once boot is complete.
    pub fn advance(&mut self) -> Option<&'static BootStep> {
        let step = BOOT_STEPS.get(self.index)?;
        self.index += 1;
        Some(step)
    }

    pub fn remaining(&self) -> usize {
        BOOT_STEPS.len().saturating_sub(self.index)
    }

    pub fn is_done(&self) -> bool {
        self.index >= BOOT_STEPS.len()
    }
}

impl Default for BootSequence {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn phases_form_a_linear_chain() {
        let mut phase = AppBootPhase::AttachBaseContext;
        let mut seen = 0;
        loop {
            seen += 1;
            match phase.next() {
                Some(next) => {
                    assert_eq!(next.index(), phase.index() + 1);
                    phase = next;
                }
                None => break,
            }
        }
        assert_eq!(seen, 4);
        assert_eq!(phase, AppBootPhase::Main);
        assert_eq!(AppBootPhase::Main.next(), None);
    }

    #[test]
    fn steps_cover_all_phases_in_order_and_end_at_main() {
        assert_eq!(BOOT_STEPS.first().unwrap().phase, AppBootPhase::AttachBaseContext);
        assert_eq!(BOOT_STEPS.last().unwrap().phase, AppBootPhase::Main);
        let mut last = 0;
        let mut phases = [false; 4];
        for step in BOOT_STEPS {
            let idx = step.phase.index();
            assert!(idx >= last, "phases must not go backwards");
            last = idx;
            phases[idx] = true;
            assert!(!step.name.is_empty());
            assert!(!step.reference.is_empty());
        }
        assert!(phases.iter().all(|seen| *seen));
    }

    #[test]
    fn sequence_walks_every_step_then_stops() {
        let mut seq = BootSequence::new();
        let mut count = 0;
        while let Some(step) = seq.advance() {
            assert_eq!(step, &BOOT_STEPS[count]);
            count += 1;
        }
        assert_eq!(count, BOOT_STEPS.len());
        assert!(seq.is_done());
        assert_eq!(seq.remaining(), 0);
        assert_eq!(seq.advance(), None);
    }

    #[test]
    fn phase_strings_are_stable() {
        assert_eq!(AppBootPhase::AttachBaseContext.as_str(), "attach_base_context");
        assert_eq!(AppBootPhase::OnCreate.as_str(), "on_create");
        assert_eq!(AppBootPhase::Splash.as_str(), "splash");
        assert_eq!(AppBootPhase::Main.as_str(), "main");
    }
}
