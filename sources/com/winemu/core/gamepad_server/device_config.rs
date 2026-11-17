//! Disabled-device (blacklist) configuration for the gamepad server.
//!
//! Mirrors `com.winemu.core.gamepad.DeviceConfig(disabledDevices)` and the
//! blacklist operations in `GamepadManager.e()` / `GamepadManager.X()`, which
//! enable/disable devices by their stable descriptor (`GamepadDevice.a()`).

use std::collections::HashSet;

use super::player_slot::GamepadDevice;

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct DeviceConfig {
    pub disabled_devices: HashSet<String>,
}

impl DeviceConfig {
    pub fn new(disabled_devices: impl IntoIterator<Item = impl Into<String>>) -> Self {
        Self {
            disabled_devices: disabled_devices.into_iter().map(Into::into).collect(),
        }
    }

    pub fn empty() -> Self {
        Self {
            disabled_devices: HashSet::new(),
        }
    }

    /// `GamepadDeviceManager.k(descriptor, true)`.
    pub fn disable(&mut self, descriptor: impl Into<String>) {
        self.disabled_devices.insert(descriptor.into());
    }

    /// `GamepadDeviceManager.k(descriptor, false)`.
    pub fn enable(&mut self, descriptor: &str) {
        self.disabled_devices.remove(descriptor);
    }

    pub fn is_disabled(&self, descriptor: &str) -> bool {
        self.disabled_devices.contains(descriptor)
    }

    pub fn disabled(&self) -> impl Iterator<Item = &str> {
        self.disabled_devices.iter().map(String::as_str)
    }

    /// Apply this config's blacklist to a device before slot assignment.
    pub fn apply_to(&self, device: &mut GamepadDevice) {
        let disabled = self.is_disabled(&device.owned_descriptor());
        match device {
            GamepadDevice::Physical(p) => p.is_disabled = disabled,
            GamepadDevice::Virtual(v) => v.is_disabled = disabled,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn disable_enable_and_query() {
        let mut config = DeviceConfig::empty();
        assert!(!config.is_disabled("desc-1"));
        config.disable("desc-1");
        assert!(config.is_disabled("desc-1"));
        config.enable("desc-1");
        assert!(!config.is_disabled("desc-1"));
    }

    #[test]
    fn new_from_iterable() {
        let config = DeviceConfig::new(["a", "b"]);
        assert_eq!(config.disabled().count(), 2);
        assert!(config.is_disabled("a"));
        assert!(config.is_disabled("b"));
    }

    #[test]
    fn apply_to_device() {
        let config = DeviceConfig::new(["desc-x"]);
        let mut device = GamepadDevice::physical(1, "Pad", "desc-x");
        config.apply_to(&mut device);
        assert!(device.is_disabled());
        let mut other = GamepadDevice::physical(2, "Pad", "desc-y");
        config.apply_to(&mut other);
        assert!(!other.is_disabled());
    }
}
