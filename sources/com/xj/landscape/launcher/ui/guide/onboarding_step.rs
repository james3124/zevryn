//! Ordered onboarding checklist for the GameHub Lite v5.1.8 guide flow.
//!
//! The reference package `com.xj.landscape.launcher.ui.guide` contains ~20+
//! `Guide*Activity` / `Guide*Fragment` screens shown on first launch: login,
//! account creation, validation-code entry, permission requests, contact
//! discovery and a gamepad tutorial. Each screen here is a single
//! [`OnboardingStep`]; the ordering in [`OnboardingStep::ALL`] mirrors the
//! flow from sign-in through the gamepad guides.
//!
//! The Android activities themselves are intentionally absent: no `Activity`,
//! `Fragment`, layout, view binding, Intents or permission dialogs are modeled.
//! Only the checklist identity and its completion state are portable.

use std::collections::BTreeSet;

/// A single step of the first-launch guide flow.
///
/// Each variant names one guide screen / fragment from the reference package.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum OnboardingStep {
    /// `GuideLoginActivity` — provider sign-in.
    Login,
    /// `GuidePhoneValidateActivity` — phone number entry.
    PhoneValidate,
    /// `GuideEmailValidateActivity` — e-mail entry.
    EmailValidate,
    /// `GuideInputValidateCodeActivity` — SMS / e-mail code entry.
    InputValidateCode,
    /// `GuideCreateNameActivity` — choose a nickname.
    CreateName,
    /// `GuideCreateAvatarActivity` — choose an avatar.
    CreateAvatar,
    /// `GuideCreateInfoActivity` — remaining profile info.
    CreateInfo,
    /// `GuideRequestNotificationPermissionActivity` — notification permission.
    RequestNotificationPermission,
    /// `GuideRequestStoragePermissionActivity` — storage permission.
    RequestStoragePermission,
    /// `GuideRequestUsbtPermissionActivity` — USB permission.
    RequestUsbPermission,
    /// `GuideFindContactActivity` — find friends via contacts.
    FindContact,
    /// `GuiRequestReadContactsPermissionActivity` — contacts permission.
    RequestReadContactsPermission,
    /// `GuideFindContactIngFragment` — contact discovery in progress.
    FindContactIng,
    /// `GuideFindContactSuccessFragment` — contact discovery succeeded.
    FindContactSuccess,
    /// `GuideFindContactErrFragment` — contact discovery failed.
    FindContactErr,
    /// `GuideNotConnectGamePadActivity` — "no gamepad connected" prompt.
    NotConnectGamePad,
    /// `GuideBuyGamePadActivity` — gamepad purchase prompt.
    BuyGamePad,
    /// `GuideGamePadCover1Activity` — gamepad cover tutorial 1.
    GamePadCover1,
    /// `GuideGamePadCover2Activity` — gamepad cover tutorial 2.
    GamePadCover2,
    /// `GuideGamePadPromotional1Activity` — gamepad promo 1.
    GamePadPromotional1,
    /// `GuideGamePadPromotional2Activity` — gamepad promo 2.
    GamePadPromotional2,
    /// `GuideGamePadPromotional3Activity` — gamepad promo 3.
    GamePadPromotional3,
    /// `GuideKeypadOperationActivity` — keypad operation tutorial.
    KeypadOperation,
    /// `GuideButtonAActivity` — button A tutorial.
    ButtonA,
    /// `GuideHighlightActivity` — closing highlight screen.
    Highlight,
}

impl OnboardingStep {
    /// All guide steps, in flow order (sign-in -> gamepad tutorials -> end).
    pub const ALL: [Self; 25] = [
        Self::Login,
        Self::PhoneValidate,
        Self::EmailValidate,
        Self::InputValidateCode,
        Self::CreateName,
        Self::CreateAvatar,
        Self::CreateInfo,
        Self::RequestNotificationPermission,
        Self::RequestStoragePermission,
        Self::RequestUsbPermission,
        Self::FindContact,
        Self::RequestReadContactsPermission,
        Self::FindContactIng,
        Self::FindContactSuccess,
        Self::FindContactErr,
        Self::NotConnectGamePad,
        Self::BuyGamePad,
        Self::GamePadCover1,
        Self::GamePadCover2,
        Self::GamePadPromotional1,
        Self::GamePadPromotional2,
        Self::GamePadPromotional3,
        Self::KeypadOperation,
        Self::ButtonA,
        Self::Highlight,
    ];

    /// 1-based position of this step in the flow.
    pub fn position(self) -> usize {
        Self::ALL.iter().position(|s| *s == self).unwrap() + 1
    }

    /// The reference class name of the screen this step represents.
    pub fn class_name(self) -> &'static str {
        match self {
            Self::Login => "GuideLoginActivity",
            Self::PhoneValidate => "GuidePhoneValidateActivity",
            Self::EmailValidate => "GuideEmailValidateActivity",
            Self::InputValidateCode => "GuideInputValidateCodeActivity",
            Self::CreateName => "GuideCreateNameActivity",
            Self::CreateAvatar => "GuideCreateAvatarActivity",
            Self::CreateInfo => "GuideCreateInfoActivity",
            Self::RequestNotificationPermission => "GuideRequestNotificationPermissionActivity",
            Self::RequestStoragePermission => "GuideRequestStoragePermissionActivity",
            Self::RequestUsbPermission => "GuideRequestUsbtPermissionActivity",
            Self::FindContact => "GuideFindContactActivity",
            Self::RequestReadContactsPermission => "GuiRequestReadContactsPermissionActivity",
            Self::FindContactIng => "GuideFindContactIngFragment",
            Self::FindContactSuccess => "GuideFindContactSuccessFragment",
            Self::FindContactErr => "GuideFindContactErrFragment",
            Self::NotConnectGamePad => "GuideNotConnectGamePadActivity",
            Self::BuyGamePad => "GuideBuyGamePadActivity",
            Self::GamePadCover1 => "GuideGamePadCover1Activity",
            Self::GamePadCover2 => "GuideGamePadCover2Activity",
            Self::GamePadPromotional1 => "GuideGamePadPromotional1Activity",
            Self::GamePadPromotional2 => "GuideGamePadPromotional2Activity",
            Self::GamePadPromotional3 => "GuideGamePadPromotional3Activity",
            Self::KeypadOperation => "GuideKeypadOperationActivity",
            Self::ButtonA => "GuideButtonAActivity",
            Self::Highlight => "GuideHighlightActivity",
        }
    }
}

/// Completion state for the ordered onboarding checklist.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct OnboardingChecklist {
    completed: BTreeSet<OnboardingStep>,
}

impl OnboardingChecklist {
    /// An empty checklist with no steps completed.
    pub fn new() -> Self {
        Self::default()
    }

    /// Mark a step complete.
    pub fn complete(&mut self, step: OnboardingStep) {
        self.completed.insert(step);
    }

    /// Mark a step incomplete again.
    pub fn uncomplete(&mut self, step: OnboardingStep) {
        self.completed.remove(&step);
    }

    /// True when `step` has been completed.
    pub fn is_complete(&self, step: OnboardingStep) -> bool {
        self.completed.contains(&step)
    }

    /// Number of completed steps.
    pub fn completed_count(&self) -> usize {
        self.completed.len()
    }

    /// Total number of steps in the flow.
    pub fn total(&self) -> usize {
        OnboardingStep::ALL.len()
    }

    /// True when every step has been completed.
    pub fn is_all_complete(&self) -> bool {
        self.completed_count() == self.total()
    }

    /// The first step in flow order that is not yet complete.
    pub fn next_pending(&self) -> Option<OnboardingStep> {
        OnboardingStep::ALL
            .iter()
            .copied()
            .find(|s| !self.is_complete(*s))
    }

    /// Progress as a value in `0.0..=1.0`.
    pub fn progress(&self) -> f64 {
        self.completed_count() as f64 / self.total() as f64
    }

    /// Clear all completion state.
    pub fn reset(&mut self) {
        self.completed.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn checklist_has_all_guide_steps() {
        assert_eq!(OnboardingStep::ALL.len(), 25);
        let mut positions: Vec<usize> = OnboardingStep::ALL.iter().map(|s| s.position()).collect();
        positions.sort_unstable();
        assert_eq!(positions, (1..=25).collect::<Vec<_>>());
    }

    #[test]
    fn flow_starts_with_login_and_ends_with_highlight() {
        assert_eq!(OnboardingStep::ALL[0], OnboardingStep::Login);
        assert_eq!(OnboardingStep::ALL[24], OnboardingStep::Highlight);
        assert_eq!(OnboardingStep::Login.position(), 1);
        assert_eq!(OnboardingStep::Highlight.position(), 25);
    }

    #[test]
    fn new_checklist_is_empty() {
        let c = OnboardingChecklist::new();
        assert_eq!(c.completed_count(), 0);
        assert_eq!(c.total(), 25);
        assert!(!c.is_all_complete());
        assert_eq!(c.next_pending(), Some(OnboardingStep::Login));
        assert_eq!(c.progress(), 0.0);
    }

    #[test]
    fn progression_follows_flow_order() {
        let mut c = OnboardingChecklist::new();
        for step in [
            OnboardingStep::Login,
            OnboardingStep::PhoneValidate,
            OnboardingStep::EmailValidate,
        ] {
            assert_eq!(c.next_pending(), Some(step));
            c.complete(step);
            assert!(c.is_complete(step));
        }
        assert_eq!(c.completed_count(), 3);
        assert_eq!(c.next_pending(), Some(OnboardingStep::InputValidateCode));
        assert!((c.progress() - 3.0 / 25.0).abs() < f64::EPSILON);
    }

    #[test]
    fn completing_out_of_order_still_advances_next_pending() {
        let mut c = OnboardingChecklist::new();
        c.complete(OnboardingStep::Highlight);
        // Login is still pending even though a later step is complete.
        assert_eq!(c.next_pending(), Some(OnboardingStep::Login));
    }

    #[test]
    fn completing_every_step_finishes_the_flow() {
        let mut c = OnboardingChecklist::new();
        for step in OnboardingStep::ALL {
            c.complete(step);
        }
        assert!(c.is_all_complete());
        assert_eq!(c.completed_count(), 25);
        assert_eq!(c.next_pending(), None);
        assert!((c.progress() - 1.0).abs() < f64::EPSILON);
    }

    #[test]
    fn uncomplete_and_reset_work() {
        let mut c = OnboardingChecklist::new();
        c.complete(OnboardingStep::Login);
        c.uncomplete(OnboardingStep::Login);
        assert!(!c.is_complete(OnboardingStep::Login));
        for step in OnboardingStep::ALL {
            c.complete(step);
        }
        c.reset();
        assert!(c.completed_count() == 0);
        assert_eq!(c.next_pending(), Some(OnboardingStep::Login));
    }

    #[test]
    fn class_names_are_unique() {
        let mut names: Vec<_> = OnboardingStep::ALL.iter().map(|s| s.class_name()).collect();
        names.sort_unstable();
        let deduped: std::collections::BTreeSet<_> = names.iter().collect();
        assert_eq!(deduped.len(), OnboardingStep::ALL.len());
    }
}
