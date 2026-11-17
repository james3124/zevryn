//! Keymap helpers mirrored from
//! `/home/Project/Zevryn/src/keymap.rs` and originally derived from
//! `com.xj.landscape.launcher.ui.main.me.MyGamesItemData.TopPlatFormType`
//! (= `com.xj.common.data.list.TopPlatFormType`) plus the `.icp` binding
//! vocabulary.
//!
//! Uses `super::{Element, ElementType}` so the module root (`mapping/mod.rs`)
//! owns the profile types and both modules share the exact same types.

use super::{Element, ElementType};

/// Platform tag from the decompiled `TopPlatFormType` enum (8 values).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TopPlatform {
    Steam,
    GameLib,
    PcLink,
    PsLink,
    MobileGame,
    PcEmulator,
    ImportGame,
    Other,
}

impl TopPlatform {
    pub fn from_str(s: &str) -> Self {
        match s {
            "Steam" => Self::Steam,
            "GameLib" => Self::GameLib,
            "PCLink" => Self::PcLink,
            "PSLink" => Self::PsLink,
            "MobileGame" => Self::MobileGame,
            "PcEmulator" => Self::PcEmulator,
            "ImportGame" => Self::ImportGame,
            _ => Self::Other,
        }
    }

    /// All 8 platform tags, in declaration order of the Kotlin enum.
    pub const ALL: [Self; 8] = [
        Self::Steam,
        Self::GameLib,
        Self::PcLink,
        Self::PsLink,
        Self::MobileGame,
        Self::PcEmulator,
        Self::ImportGame,
        Self::Other,
    ];

    /// Round-trip back to the exact token used by the decompiled enum.
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Steam => "Steam",
            Self::GameLib => "GameLib",
            Self::PcLink => "PCLink",
            Self::PsLink => "PSLink",
            Self::MobileGame => "MobileGame",
            Self::PcEmulator => "PcEmulator",
            Self::ImportGame => "ImportGame",
            Self::Other => "Other",
        }
    }
}

/// Classify a binding token from an `.icp` profile.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BindingKind {
    Unbound,
    DPad,
    Button,
    Thumb,
    Trigger,
    Unknown,
}

pub fn classify_binding(token: &str) -> BindingKind {
    if token == "NONE" || token.is_empty() {
        BindingKind::Unbound
    } else if token.contains("DPAD") {
        BindingKind::DPad
    } else if token.contains("THUMB") {
        BindingKind::Thumb
    } else if token.contains("TRIGGER") || token.contains("_LT") || token.contains("_RT") {
        BindingKind::Trigger
    } else if token.contains("BUTTON") {
        BindingKind::Button
    } else {
        BindingKind::Unknown
    }
}

/// Human-readable summary of an element's bindings, independent of Android.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResolvedBinding {
    pub kind: ElementType,
    pub bound: usize,
    pub unbound: usize,
    pub first_action: Option<String>,
}

pub fn resolve(element: &Element) -> ResolvedBinding {
    let bound = element
        .bindings
        .iter()
        .filter(|b| classify_binding(b) != BindingKind::Unbound)
        .count();
    let first_action = element
        .bindings
        .iter()
        .find(|b| classify_binding(b) != BindingKind::Unbound)
        .cloned();
    ResolvedBinding {
        kind: element.kind,
        bound,
        unbound: element.bindings.len() - bound,
        first_action,
    }
}

/// Resolve every element of a profile into a list of summaries.
pub fn resolve_profile(elements: &[Element]) -> Vec<ResolvedBinding> {
    elements.iter().map(resolve).collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::super::Shape;

    #[test]
    fn platform_mapping() {
        assert_eq!(TopPlatform::from_str("PCLink"), TopPlatform::PcLink);
        assert_eq!(TopPlatform::from_str("PSLink"), TopPlatform::PsLink);
        assert_eq!(TopPlatform::from_str("nonsense"), TopPlatform::Other);
        assert_eq!(TopPlatform::ALL.len(), 8);
        for p in TopPlatform::ALL {
            assert_eq!(TopPlatform::from_str(p.as_str()), p);
        }
    }

    #[test]
    fn binding_classification() {
        assert_eq!(classify_binding("NONE"), BindingKind::Unbound);
        assert_eq!(classify_binding(""), BindingKind::Unbound);
        assert_eq!(classify_binding("GAMEPAD_DPAD_UP"), BindingKind::DPad);
        assert_eq!(classify_binding("GAMEPAD_LEFT_THUMB_UP"), BindingKind::Thumb);
        assert_eq!(classify_binding("GAMEPAD_BUTTON_X"), BindingKind::Button);
        assert_eq!(classify_binding("GAMEPAD_LT"), BindingKind::Trigger);
        assert_eq!(classify_binding("KEY_A"), BindingKind::Unknown);
    }

    #[test]
    fn resolves_element() {
        let e = Element {
            kind: ElementType::DPad,
            shape: Shape::Circle,
            bindings: vec!["GAMEPAD_DPAD_UP".into(), "NONE".into(), "NONE".into(), "NONE".into()],
            scale: 1.0,
            x: 0.3,
            y: 0.7,
            toggle_switch: false,
            text: String::new(),
            icon_id: 0,
            range: None,
        };
        let r = resolve(&e);
        assert_eq!(r.kind, ElementType::DPad);
        assert_eq!(r.bound, 1);
        assert_eq!(r.unbound, 3);
        assert_eq!(r.first_action.as_deref(), Some("GAMEPAD_DPAD_UP"));
    }

    #[test]
    fn resolves_profile_from_real_apk_when_present() {
        let dir = "/home/Project/Zevryn/reference/gamehub-lite-v518-decompiled/gamehub-lite-v518-decompiled/apktool/assets/inputcontrols/profiles";
        let Ok(entries) = std::fs::read_dir(dir) else { return; };
        let mut profiles = 0;
        for entry in entries.flatten() {
            let text = std::fs::read_to_string(entry.path()).unwrap();
            let p = super::super::Profile::parse(&text).unwrap();
            let summaries = resolve_profile(&p.elements);
            assert_eq!(summaries.len(), p.elements.len());
            for s in summaries {
                assert_eq!(s.bound + s.unbound, s.kind.binding_count().unwrap_or(s.bound + s.unbound));
            }
            profiles += 1;
        }
        assert_eq!(profiles, 4, "expected 4 bundled .icp profiles");
    }
}
