//! Portable model of the component "select-or-default" logic in
//! `com.xj.winemu.settings.PcGameSettingOperations` /
//! `PcGameSettingDataHelper` (GameHub Lite v5.1.8).
//!
//! The decompiled originals are `suspend` functions that call
//! `WinEmuFileExplorer.q(..)` / `.v(..)` to check whether a component is
//! present on disk, then pick either the user's current selection or a
//! downloaded default. None of that Android/I/O machinery is portable, so
//! this module keeps only the decision logic and models the "is downloaded"
//! answer as a plain `bool` on [`ComponentSelection`].
//!
//! Source methods mirrored:
//! * `getDefaultContainerEntityByDownload` (`PcGameSettingDataHelper.k`)
//! * `getDefaultGpuByDownload` (`PcGameSettingOperations.T`)
//! * `getDefaultDxvkByDownload` (`PcGameSettingOperations.R`)
//! * `getDefaultVkd3dByDownload` (`PcGameSettingOperations.W`)
//! * `getDefaultTranslatorByDownload` (`PcGameSettingDataHelper.o`)
//! * `getSelectContainerOrDefault` (`PcGameSettingOperations.y0`)
//! * `getSelectGpuOrDefault` (`PcGameSettingOperations.I0`)
//! * `getSelectDxvkOrDefault` (`PcGameSettingOperations.F0`)
//! * `getSelectVkd3dOrDefault` (`PcGameSettingOperations.U0`)

use std::collections::HashMap;

/// The five component families handled by the `getDefault*ByDownload`
/// methods. Mirrors the relevant subset of `com.xj.winemu.api.bean.ComponentType`
/// (`GENERAL`/`DEPENDENCY`/`STEAMCLIENT` are out of scope here).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum ComponentKind {
    /// Container / environment layer (`getDefaultContainerEntityByDownload`,
    /// checked with `WinEmuFileExplorer.v` because it carries a file md5).
    Container,
    /// GPU driver (`getDefaultGpuByDownload`).
    Gpu,
    /// DXVK (`getDefaultDxvkByDownload`).
    Dxvk,
    /// VKD3D (`getDefaultVkd3dByDownload`).
    Vkd3d,
    /// Box64/FEX translator (`getDefaultTranslatorByDownload`).
    Translator,
}

impl ComponentKind {
    /// The `ComponentType` name this kind corresponds to in the decompiled
    /// sources (`GENERAL` for the container).
    pub fn component_type_name(self) -> &'static str {
        match self {
            Self::Container => "GENERAL",
            Self::Gpu => "GPU",
            Self::Dxvk => "DXVK",
            Self::Vkd3d => "VKD3D",
            Self::Translator => "TRANSLATOR",
        }
    }
}

/// A single component candidate. Mirrors the portable fields of
/// `com.xj.winemu.bean.PcSettingDataEntity` that the default/select methods
/// actually read (`id`, `name`, `version`, `file_name`, `file_md5`), plus the
/// two derived states the resolver needs.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ComponentSelection {
    pub kind: ComponentKind,
    pub id: i32,
    pub name: String,
    pub version: String,
    pub file_name: String,
    pub file_md5: String,
    /// Result of the `WinEmuFileExplorer.q` / `.v` download check.
    pub downloaded: bool,
    /// Whether the component is installed (used by the selection dialogs).
    pub installed: bool,
}

impl ComponentSelection {
    pub fn new(kind: ComponentKind, name: impl Into<String>, version: impl Into<String>) -> Self {
        Self {
            kind,
            id: 0,
            name: name.into(),
            version: version.into(),
            file_name: String::new(),
            file_md5: String::new(),
            downloaded: false,
            installed: false,
        }
    }

    pub fn with_id(mut self, id: i32) -> Self {
        self.id = id;
        self
    }

    pub fn with_file_name(mut self, file_name: impl Into<String>) -> Self {
        self.file_name = file_name.into();
        self
    }

    pub fn with_file_md5(mut self, file_md5: impl Into<String>) -> Self {
        self.file_md5 = file_md5.into();
        self
    }

    pub fn with_downloaded(mut self, downloaded: bool) -> Self {
        self.downloaded = downloaded;
        self
    }

    pub fn with_installed(mut self, installed: bool) -> Self {
        self.installed = installed;
        self
    }
}

/// The GPU "official driver" sentinel returned by `getSelectGpuOrDefault` /
/// `getDefaultGpuByDownload` when no user selection and no downloaded driver
/// exist (`new PcSettingDataEntity(-1, ..., null, ...)` with id `-1`).
pub fn official_gpu_driver_selection() -> ComponentSelection {
    ComponentSelection::new(ComponentKind::Gpu, "Official driver", "")
        .with_id(-1)
        .with_downloaded(false)
        .with_installed(false)
}

/// Mirror of `getDefault*ByDownload`: a default candidate is only usable when
/// it exists and the download check succeeded.
pub fn default_by_download(candidate: Option<ComponentSelection>) -> Option<ComponentSelection> {
    candidate.filter(|selection| selection.downloaded)
}

/// Mirror of `getSelect*OrDefault`: the user's current selection wins; only
/// when it is absent do we fall back to a downloaded default.
///
/// `getSelect*OrDefault` checks `selected != null` first and returns it
/// immediately; the default entity is only consulted when `selected == null`,
/// and is itself returned only when the download check passes.
pub fn select_or_default(
    selected: Option<ComponentSelection>,
    default: Option<ComponentSelection>,
) -> Option<ComponentSelection> {
    selected.or_else(|| default_by_download(default))
}

/// Holds the per-kind default candidates read from the "default" preferences
/// (`pc_d_dxvk`, `pc_d_gpu`, `pc_d_vkd3d`, `pc_d_translator`, container key),
/// and exposes the resolver operations used by `PcGameSettingOperations`.
#[derive(Debug, Clone, Default)]
pub struct SelectionResolver {
    defaults: HashMap<ComponentKind, ComponentSelection>,
}

impl SelectionResolver {
    pub fn new() -> Self {
        Self::default()
    }

    /// Register/replace the default candidate for a kind.
    pub fn set_default(&mut self, default: ComponentSelection) {
        self.defaults.insert(default.kind, default);
    }

    /// The raw default candidate, before any download check.
    pub fn default_candidate(&self, kind: ComponentKind) -> Option<&ComponentSelection> {
        self.defaults.get(&kind)
    }

    /// `getDefault*ByDownload` for a kind.
    pub fn default_by_download(&self, kind: ComponentKind) -> Option<ComponentSelection> {
        default_by_download(self.defaults.get(&kind).cloned())
    }

    /// `getSelect*OrDefault` for a kind.
    pub fn select_or_default(
        &self,
        kind: ComponentKind,
        selected: Option<ComponentSelection>,
    ) -> Option<ComponentSelection> {
        select_or_default(selected, self.default_by_download(kind))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn default(kind: ComponentKind, downloaded: bool) -> ComponentSelection {
        ComponentSelection::new(kind, "Default", "1.0").with_downloaded(downloaded)
    }

    #[test]
    fn default_by_download_requires_downloaded() {
        assert_eq!(default_by_download(Some(default(ComponentKind::Dxvk, true))), Some(default(ComponentKind::Dxvk, true)));
        assert_eq!(default_by_download(Some(default(ComponentKind::Dxvk, false))), None);
        assert_eq!(default_by_download(None), None);
        assert_eq!(default_by_download(Some(default(ComponentKind::Dxvk, true))).unwrap().kind, ComponentKind::Dxvk);
    }

    #[test]
    fn select_or_default_prefers_existing_selection() {
        let selected = ComponentSelection::new(ComponentKind::Dxvk, "Selected", "9.9");
        let fallback = default(ComponentKind::Dxvk, true);
        // An existing selection wins even though a downloaded default exists.
        assert_eq!(select_or_default(Some(selected.clone()), Some(fallback)), Some(selected));
    }

    #[test]
    fn select_or_default_uses_downloaded_default_when_unselected() {
        let fallback = default(ComponentKind::Vkd3d, true);
        assert_eq!(select_or_default(None, Some(fallback.clone())), Some(fallback));
    }

    #[test]
    fn select_or_default_returns_none_when_default_not_downloaded() {
        assert_eq!(select_or_default(None, Some(default(ComponentKind::Container, false))), None);
        assert_eq!(select_or_default(None, None), None);
    }

    #[test]
    fn resolver_mirrors_select_or_default_per_kind() {
        let mut resolver = SelectionResolver::new();
        resolver.set_default(default(ComponentKind::Dxvk, true));
        resolver.set_default(default(ComponentKind::Gpu, false));

        assert_eq!(resolver.default_by_download(ComponentKind::Dxvk).map(|s| s.kind), Some(ComponentKind::Dxvk));
        assert_eq!(resolver.default_by_download(ComponentKind::Gpu), None);
        assert_eq!(resolver.default_by_download(ComponentKind::Translator), None);

        let selected_gpu = ComponentSelection::new(ComponentKind::Gpu, "Chosen", "2.0");
        assert_eq!(
            resolver.select_or_default(ComponentKind::Gpu, Some(selected_gpu.clone())),
            Some(selected_gpu),
        );
        assert_eq!(resolver.select_or_default(ComponentKind::Dxvk, None).map(|s| s.name), Some("Default".to_string()));
    }

    #[test]
    fn gpu_sentinel_uses_id_minus_one() {
        let sentinel = official_gpu_driver_selection();
        assert_eq!(sentinel.kind, ComponentKind::Gpu);
        assert_eq!(sentinel.id, -1);
        assert!(!sentinel.downloaded);
    }
}
