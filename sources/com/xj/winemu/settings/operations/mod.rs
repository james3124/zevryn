//! Rust model of the portable parts of `com.xj.winemu.settings` operations
//! (GameHub Lite v5.1.8): component select-or-default resolution, translation
//! config state, and the YML list descriptor.
//!
//! Mirrors `sources/com/xj/winemu/settings/PcGameSettingOperations.java`,
//! `PcGameSettingDataHelper.java`, `translations/data/ConfigModifyState.java`
//! and `common/data/table/WinEmuTranslationConfigEntity.java`.

pub mod component_selection;
pub mod translation_config;
pub mod yml_config;

pub use component_selection::{
    default_by_download, official_gpu_driver_selection, select_or_default, ComponentKind,
    ComponentSelection, SelectionResolver,
};
pub use translation_config::{ConfigChange, ConfigModifyState, TranslationConfig};
pub use yml_config::{YmlConfig, YmlListType};
