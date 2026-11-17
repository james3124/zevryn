//! Rust model of the `SettingTypeValue` family (GameHub Lite v5.1.8).
//!
//! Sources: `sources/com/xj/winemu/settings/models/{SettingTypeValue,
//! SwitchSettingTypeValue, ButtonSettingTypeValue, InnerInputSettingTypeValue}.java`.
//!
//! In Java, `SettingTypeValue` is an empty `Parcelable` marker interface with
//! three `@Parcelize` implementations. The only portable parts are their data
//! fields; Android-only callbacks (`Function1 onSwitchChanged`,
//! `Function1 onTextChanged`), the `Radii` view property, and all `Parcelable`
//! plumbing are dropped. The callback slots are represented by plain optional
//! string ids where a logical "action" is needed (`ButtonSettingTypeValue.action`).

/// The three concrete setting-value payloads, mirroring the Java implementations
/// of the `SettingTypeValue` marker interface.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SettingTypeValue {
    /// `SwitchSettingTypeValue` — a boolean switch value.
    Switch(SwitchSettingTypeValue),
    /// `ButtonSettingTypeValue` — a tappable button's styling plus an optional
    /// logical action id standing in for the Android `Function1 action` callback.
    Button(ButtonSettingTypeValue),
    /// `InnerInputSettingTypeValue` — inline text-input environment params.
    InnerInput(InnerInputSettingTypeValue),
}

/// Port of `SwitchSettingTypeValue`.
///
/// Java fields: `isChecked: Boolean`, `onSwitchChanged: Function1`. Only the
/// boolean value is portable; the change callback is Android UI wiring.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct SwitchSettingTypeValue {
    /// `isChecked` — whether the switch is on.
    pub is_checked: bool,
}

/// Port of `ButtonSettingTypeValue`.
///
/// Java fields: `textColor`, `backgroundColor`, `backgroundUnselectColor`
/// (all `Int`) and `radius: Radii`. `Radii` is an Android view property and is
/// intentionally omitted; `action` is a portable stand-in for the click
/// callback stored on the owning `ChildSetting` in the Java model.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ButtonSettingTypeValue {
    /// `textColor` (ARGB int).
    pub text_color: i32,
    /// `backgroundColor` (ARGB int).
    pub background_color: i32,
    /// `backgroundUnselectColor` (ARGB int).
    pub background_unselect_color: i32,
    /// Portable logical action id (the Java callback is out of scope).
    pub action: Option<String>,
}

/// Port of `InnerInputSettingTypeValue`.
///
/// Java fields: `inputHint: String?`, `inputPlaceholder: String?`,
/// `onTextChanged: Function1`. Only the two environment strings are portable.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct InnerInputSettingTypeValue {
    /// `inputHint` — hint shown in the input field.
    pub input_hint: Option<String>,
    /// `inputPlaceholder` — placeholder text for the input field.
    pub input_placeholder: Option<String>,
}

impl SettingTypeValue {
    /// Convenience constructor for a switch value.
    pub fn switch(is_checked: bool) -> Self {
        Self::Switch(SwitchSettingTypeValue { is_checked })
    }

    /// Convenience constructor for a button value.
    pub fn button(text_color: i32, background_color: i32, background_unselect_color: i32) -> Self {
        Self::Button(ButtonSettingTypeValue {
            text_color,
            background_color,
            background_unselect_color,
            action: None,
        })
    }

    /// Convenience constructor for an inline-input value.
    pub fn input(hint: Option<String>, placeholder: Option<String>) -> Self {
        Self::InnerInput(InnerInputSettingTypeValue {
            input_hint: hint,
            input_placeholder: placeholder,
        })
    }

    /// The discriminant name, useful for validation and debugging.
    pub fn kind(&self) -> &'static str {
        match self {
            Self::Switch(_) => "Switch",
            Self::Button(_) => "Button",
            Self::InnerInput(_) => "InnerInput",
        }
    }

    /// Read the switch boolean when this is a switch value.
    pub fn as_switch(&self) -> Option<bool> {
        match self {
            Self::Switch(v) => Some(v.is_checked),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn constructors_pick_the_right_variant() {
        let switch = SettingTypeValue::switch(true);
        assert_eq!(switch.kind(), "Switch");
        assert_eq!(switch.as_switch(), Some(true));

        let button = SettingTypeValue::button(1, 2, 3);
        assert_eq!(button.kind(), "Button");
        assert_eq!(button.as_switch(), None);

        let input = SettingTypeValue::input(Some("h".into()), None);
        assert_eq!(input.kind(), "InnerInput");
    }

    #[test]
    fn payloads_are_value_types() {
        let a = SwitchSettingTypeValue { is_checked: true };
        assert_eq!(a, SwitchSettingTypeValue { is_checked: true });
        assert_ne!(a, SwitchSettingTypeValue::default());

        let b = ButtonSettingTypeValue {
            text_color: 10,
            background_color: 20,
            background_unselect_color: 30,
            action: Some("reset".into()),
        };
        assert_eq!(b.clone(), b);
        assert_eq!(b.action.as_deref(), Some("reset"));

        let c = InnerInputSettingTypeValue {
            input_hint: Some("hint".into()),
            input_placeholder: Some("ph".into()),
        };
        assert_eq!(c.input_hint.as_deref(), Some("hint"));
    }
}
