//! Portable state for the device-management ViewModels.
//!
//! Sources: `DeviceManagerVM` (a bare `BaseViewModel` subclass) and
//! `MineDeviceVM` (holds a `DeviceSubMenuItemEntity` LiveData and posts the
//! result of `MineDeviceRepository.a(devicesName, ...)` to it via `k(...)`).
//!
//! Repository/callback plumbing is not modeled; results are submitted through
//! `on_devices_loaded` / `on_devices_failed`.

use super::async_state::AsyncState;
use super::types::ConnectedDevice;

/// Connected-device list state with a single selected device.
#[derive(Debug, Clone, Default)]
pub struct DeviceManagerViewModel {
    /// The connected-device list and its load lifecycle.
    pub devices: AsyncState<Vec<ConnectedDevice>>,
    /// Id of the currently selected device, if any.
    pub selected_device_id: Option<String>,
}

impl DeviceManagerViewModel {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn begin_load(&mut self) {
        self.devices.begin();
    }

    /// Mirrors `MineDeviceVM.k`: publish the device data. When nothing is
    /// selected yet, the first device becomes the selection (matching the
    /// single-item `DeviceSubMenuItemEntity` the original posts).
    pub fn on_devices_loaded(&mut self, devices: Vec<ConnectedDevice>) {
        if self.selected_device_id.is_none() {
            self.selected_device_id = devices.first().map(|d| d.id.clone());
        }
        self.devices.succeed(devices);
    }

    pub fn on_devices_failed<S: Into<String>>(&mut self, message: S) {
        self.devices.fail(message);
    }

    /// Replaces the device list without auto-selecting a new device.
    pub fn set_devices(&mut self, devices: Vec<ConnectedDevice>) {
        self.devices.succeed(devices);
    }

    /// Selects a device by id. Returns `false` when the id is unknown.
    pub fn select_device(&mut self, id: impl Into<String>) -> bool {
        let id = id.into();
        let exists = self
            .devices
            .value()
            .map(|list| list.iter().any(|d| d.id == id))
            .unwrap_or(false);
        if exists {
            self.selected_device_id = Some(id);
        }
        exists
    }

    pub fn selected_device(&self) -> Option<&ConnectedDevice> {
        let id = self.selected_device_id.as_ref()?;
        self.devices
            .value()?
            .iter()
            .find(|d| &d.id == id)
    }

    /// Returns only the devices currently connected.
    pub fn connected_devices(&self) -> Vec<&ConnectedDevice> {
        self.devices
            .value()
            .map(|list| list.iter().filter(|d| d.connected).collect())
            .unwrap_or_default()
    }

    /// Updates a device's connection flag; disconnecting the selected device
    /// clears the selection.
    pub fn set_connected(&mut self, id: &str, connected: bool) -> bool {
        let mut found = false;
        if let Some(list) = self.devices.value_mut() {
            if let Some(device) = list.iter_mut().find(|d| d.id == id) {
                device.connected = connected;
                found = true;
            }
        }
        if found && !connected && self.selected_device_id.as_deref() == Some(id) {
            self.selected_device_id = None;
        }
        found
    }

    pub fn clear(&mut self) {
        self.devices.reset();
        self.selected_device_id = None;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn devices() -> Vec<ConnectedDevice> {
        vec![
            ConnectedDevice::new("a", "Controller A", true),
            ConnectedDevice::new("b", "Controller B", true),
            ConnectedDevice::new("c", "Controller C", false),
        ]
    }

    #[test]
    fn load_state_transitions() {
        let mut vm = DeviceManagerViewModel::new();
        assert!(vm.devices.is_idle());
        vm.begin_load();
        assert!(vm.devices.is_loading());

        vm.on_devices_loaded(devices());
        assert!(vm.devices.is_success());
        assert_eq!(vm.devices.value().unwrap().len(), 3);
        // First device auto-selected.
        assert_eq!(vm.selected_device_id.as_deref(), Some("a"));

        vm.on_devices_failed("timeout");
        assert_eq!(vm.devices.error(), Some("timeout"));
    }

    #[test]
    fn selection_updates_and_lookup() {
        let mut vm = DeviceManagerViewModel::new();
        vm.on_devices_loaded(devices());

        assert!(vm.select_device("b"));
        assert_eq!(vm.selected_device().map(|d| d.name.as_str()), Some("Controller B"));
        assert!(!vm.select_device("missing"));
        // Failed selection keeps the previous device.
        assert_eq!(vm.selected_device_id.as_deref(), Some("b"));
    }

    #[test]
    fn connected_filter_and_disconnect_clears_selection() {
        let mut vm = DeviceManagerViewModel::new();
        vm.on_devices_loaded(devices());
        assert_eq!(vm.connected_devices().len(), 2);
        assert!(vm.select_device("b"));

        assert!(vm.set_connected("b", false));
        assert_eq!(vm.connected_devices().len(), 1);
        assert_eq!(vm.selected_device_id, None);

        assert!(!vm.set_connected("missing", true));
        vm.set_devices(devices());
        assert_eq!(vm.selected_device_id, None); // set_devices does not auto-select
    }

    #[test]
    fn clear_resets_list_and_selection() {
        let mut vm = DeviceManagerViewModel::new();
        vm.on_devices_loaded(devices());
        vm.clear();
        assert!(vm.devices.is_idle());
        assert!(vm.selected_device().is_none());
    }
}
