//! Event → UI-action mapping for the import flow (GameHub Lite v5.1.8).
//!
//! `Navigate2ImportPcGameFileEvent` (decompiled:
//! `sources/com/xj/landscape/launcher/event/Navigate2ImportPcGameFileEvent.java`)
//! is an empty marker event. It is posted by `SelectImportGameTypeDialog.u0`
//! when the PC-game row is clicked, and consumed by
//! `LandscapeLauncherMainActivity.AnonymousClass8`, which checks storage
//! permission and launches the file picker for `exe` (plus `txt` on the
//! logitech build). This module models that request → action translation
//! without any Android `Channel`, `ActivityResultLauncher`, or permission
//! plumbing.

use super::import_dialog::{ImportChoice, ImportDialogModel};

/// A request to enter an import flow. Variants mirror the decompiled triggers:
///
/// - [`ImportRequest::Navigate2ImportPcGameFile`] — the marker event emitted
///   by `SelectImportGameTypeDialog.u0` (PC row click).
/// - [`ImportRequest::ImportMobileGame`] — the mobile row click
///   (`SelectImportGameTypeDialog.v0` → `PageRouterUtils.j`).
/// - [`ImportRequest::ScanSteamLibrary`] — Steam-library scan. There is no
///   corresponding entry in the decompiled dialog; this variant exists because
///   the portable UI action set includes `OpenSteamScan` (Steam import is a
///   separate flow in the app, e.g. `winemu_import_steam_*` resources).
/// - [`ImportRequest::Dismiss`] — `ivClose` / dialog dismissal.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ImportRequest {
    Navigate2ImportPcGameFile,
    ImportMobileGame,
    ScanSteamLibrary,
    Dismiss,
}

/// The portable UI action produced by handling an [`ImportRequest`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum UiAction {
    /// Open a local file picker. For `Navigate2ImportPcGameFile` this selects
    /// `exe` files (`txt` additionally when the logitech flag is set); for
    /// `ImportMobileGame` it selects an APK. The file-kind distinction lives
    /// on the Android side, so the portable action is a single variant.
    OpenFilePicker,
    /// Open the Steam-library import/scan flow.
    OpenSteamScan,
    /// Close/cancel the import flow without importing.
    Cancel,
}

impl Default for UiAction {
    fn default() -> Self {
        Self::Cancel
    }
}

/// Pure translation of a request into a UI action.
pub fn process_import_request(request: ImportRequest) -> UiAction {
    match request {
        ImportRequest::Navigate2ImportPcGameFile | ImportRequest::ImportMobileGame => {
            UiAction::OpenFilePicker
        }
        ImportRequest::ScanSteamLibrary => UiAction::OpenSteamScan,
        ImportRequest::Dismiss => UiAction::Cancel,
    }
}

/// Translate the current selection of the import dialog into a request.
/// `None` (dismissed) becomes [`ImportRequest::Dismiss`].
pub fn request_from_dialog(dialog: &ImportDialogModel) -> ImportRequest {
    match dialog.selected() {
        Some(ImportChoice::PcGame) => ImportRequest::Navigate2ImportPcGameFile,
        Some(ImportChoice::MobileGame) => ImportRequest::ImportMobileGame,
        None => ImportRequest::Dismiss,
    }
}

/// Convenience: handle the dialog's current selection end-to-end.
pub fn action_from_dialog(dialog: &ImportDialogModel) -> UiAction {
    process_import_request(request_from_dialog(dialog))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn marker_event_maps_to_file_picker() {
        assert_eq!(
            process_import_request(ImportRequest::Navigate2ImportPcGameFile),
            UiAction::OpenFilePicker
        );
    }

    #[test]
    fn all_requests_map_to_actions() {
        assert_eq!(
            process_import_request(ImportRequest::ImportMobileGame),
            UiAction::OpenFilePicker
        );
        assert_eq!(
            process_import_request(ImportRequest::ScanSteamLibrary),
            UiAction::OpenSteamScan
        );
        assert_eq!(process_import_request(ImportRequest::Dismiss), UiAction::Cancel);
        assert_eq!(UiAction::default(), UiAction::Cancel);
    }

    #[test]
    fn dialog_selection_drives_action() {
        let mut d = ImportDialogModel::new();
        assert_eq!(action_from_dialog(&d), UiAction::Cancel);

        d.select(ImportChoice::PcGame);
        assert_eq!(request_from_dialog(&d), ImportRequest::Navigate2ImportPcGameFile);
        assert_eq!(action_from_dialog(&d), UiAction::OpenFilePicker);

        d.select(ImportChoice::MobileGame);
        assert_eq!(request_from_dialog(&d), ImportRequest::ImportMobileGame);
        assert_eq!(action_from_dialog(&d), UiAction::OpenFilePicker);

        d.dismiss();
        assert_eq!(action_from_dialog(&d), UiAction::Cancel);
    }
}
