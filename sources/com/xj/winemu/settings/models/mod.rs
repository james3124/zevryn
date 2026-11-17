//! Rust model of the `com.xj.winemu.settings.models` package and the portable
//! logic of `PcGameSettingsKt` (GameHub Lite v5.1.8).
//!
//! Mirrors `sources/com/xj/winemu/settings/models/` from the reference. Only
//! dependency-free (std-only) data types and tree operations are modeled:
//! Android `Parcelable`, `Function1`/`Function2` callbacks, `ImageView`/`Radii`
//! view state, resource ids (`R.string.*`) and view holders are intentionally
//! excluded. Layout follows the house style: a `mod.rs` package root with
//! sibling `.rs` files and re-exports. The `EditType` enum lives here too even
//! though its source package is `com.xj.winemu.settings`, because the settings
//! tree API requires it.

pub mod child_setting;
pub mod edit_type;
pub mod group_setting;
pub mod icons;
pub mod setting_type_value;
pub mod settings_model;

pub use child_setting::ChildSetting;
pub use edit_type::EditType;
pub use group_setting::GroupSetting;
pub use icons::Icons;
pub use setting_type_value::{
    ButtonSettingTypeValue, InnerInputSettingTypeValue, SettingTypeValue,
    SwitchSettingTypeValue,
};
pub use settings_model::{flatten, validate, SettingsModel, ValidationError};
