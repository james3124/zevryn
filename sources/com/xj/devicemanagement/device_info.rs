//! Portable `DeviceInfo` bean from `com.xj.bussiness.devicemanagement.entity`.
//!
//! The Kotlin class has fourteen positional fields (`f23228a` .. `f23241n`).
//! The field names below are restored from its constructor parameter names and
//! getters (`c()` = deviceName, `h()` = mainMode, `k()` = subMode,
//! `e()` = firmwareVersion, `f()` = gcmVersion, `a()` = battery,
//! `l()` = temp, `n()` = voltage, `b()` = btOrBleVersion, `d()` = firmware
//! list, `g()` = macAddress, `i()` = pid, `m()` = vid).
//!
//! Device discovery, BLE/USB transports and permission flows are not modeled.

/// `DeviceInfo.FirmwareDataBean` (Kotlin variant): equality is over all three
/// fields, unlike the OTA `DeviceStatusInfo.FirmwareDataBean` whose equality is
/// over `type` only.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct FirmwareDataBean {
    pub firmware_type: String,
    pub version: String,
    pub support_ota: bool,
}

impl FirmwareDataBean {
    pub fn new(firmware_type: impl Into<String>, version: impl Into<String>, support_ota: bool) -> Self {
        Self {
            firmware_type: firmware_type.into(),
            version: version.into(),
            support_ota,
        }
    }

    /// `a()` — OTA support flag.
    pub const fn support_ota(&self) -> bool {
        self.support_ota
    }

    /// `b()` / `c()` — type and version accessors.
    pub fn kind(&self) -> &str {
        &self.firmware_type
    }

    pub fn ver(&self) -> &str {
        &self.version
    }
}

/// `DeviceInfo` portable bean.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DeviceInfo {
    pub device_name: String,
    pub main_mode: i32,
    pub sub_mode: i32,
    pub firmware_version: String,
    pub gcm_version: String,
    pub device_type: i32,
    pub battery: i32,
    pub temp: i32,
    pub voltage: i32,
    pub bt_or_ble_version: String,
    pub firmware_data: Vec<FirmwareDataBean>,
    pub mac_address: String,
    pub pid: i32,
    pub vid: i32,
}

impl Default for DeviceInfo {
    fn default() -> Self {
        // Mirrors the Kotlin no-arg constructor defaults.
        Self {
            device_name: String::new(),
            main_mode: -1,
            sub_mode: -1,
            firmware_version: "0".into(),
            gcm_version: String::new(),
            device_type: -1,
            battery: 0,
            temp: 0,
            voltage: 0,
            bt_or_ble_version: String::new(),
            firmware_data: Vec::new(),
            mac_address: String::new(),
            pid: -1,
            vid: -1,
        }
    }
}

impl DeviceInfo {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        device_name: impl Into<String>,
        main_mode: i32,
        sub_mode: i32,
        firmware_version: impl Into<String>,
        gcm_version: impl Into<String>,
        device_type: i32,
        battery: i32,
        temp: i32,
        voltage: i32,
        bt_or_ble_version: impl Into<String>,
        firmware_data: Vec<FirmwareDataBean>,
        mac_address: impl Into<String>,
        pid: i32,
        vid: i32,
    ) -> Self {
        Self {
            device_name: device_name.into(),
            main_mode,
            sub_mode,
            firmware_version: firmware_version.into(),
            gcm_version: gcm_version.into(),
            device_type,
            battery,
            temp,
            voltage,
            bt_or_ble_version: bt_or_ble_version.into(),
            firmware_data,
            mac_address: mac_address.into(),
            pid,
            vid,
        }
    }

    /// A device is addressable when it has a name and a MAC address.
    pub fn is_addressable(&self) -> bool {
        !self.device_name.is_empty() && !self.mac_address.is_empty()
    }

    pub fn firmware_for(&self, firmware_type: &str) -> Option<&FirmwareDataBean> {
        self.firmware_data
            .iter()
            .find(|f| f.firmware_type == firmware_type)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn defaults_match_kotlin_no_arg_constructor() {
        let d = DeviceInfo::default();
        assert_eq!(d.device_name, "");
        assert_eq!(d.main_mode, -1);
        assert_eq!(d.sub_mode, -1);
        assert_eq!(d.firmware_version, "0");
        assert_eq!(d.gcm_version, "");
        assert_eq!(d.device_type, -1);
        assert_eq!(d.battery, 0);
        assert_eq!(d.temp, 0);
        assert_eq!(d.voltage, 0);
        assert_eq!(d.bt_or_ble_version, "");
        assert!(d.firmware_data.is_empty());
        assert_eq!(d.mac_address, "");
        assert_eq!(d.pid, -1);
        assert_eq!(d.vid, -1);
        assert!(!d.is_addressable());
    }

    #[test]
    fn full_constructor_round_trip() {
        let fw = vec![FirmwareDataBean::new("main", "1.2.3", true)];
        let d = DeviceInfo::new(
            "GameSir-G8", 1, 2, "1.2.3", "4.5", 7, 80, 35, 5000, "BLE-1.0",
            fw.clone(), "AA:BB:CC:DD:EE:FF", 0x1234, 0x5678,
        );
        assert_eq!(d.device_name, "GameSir-G8");
        assert_eq!(d.battery, 80);
        assert_eq!(d.pid, 0x1234);
        assert!(d.is_addressable());
        assert_eq!(d.firmware_for("main"), Some(&fw[0]));
    }

    #[test]
    fn kotlin_firmware_equality_includes_all_fields() {
        let a = FirmwareDataBean::new("main", "1.0", true);
        let b = FirmwareDataBean::new("main", "1.0", true);
        assert_eq!(a, b);
        assert_ne!(a, FirmwareDataBean::new("main", "1.1", true));
        assert_ne!(a, FirmwareDataBean::new("main", "1.0", false));
    }
}
