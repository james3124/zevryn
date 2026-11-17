//! Portable data structs derived from the OTA bean classes.
//!
//! Sources (all read-only reference, GameHub Lite v5.1.8):
//! * `com.xj.ota.OTAInfor` — update metadata bean (obfuscated `f3357x`
//!   fields; names below are inferred from the getters and from call sites in
//!   `BaseOTA`, `GamepadInfor`, `FirmwareUpdateCheckUtil`).
//! * `com.xj.ota.GamepadInfor` — per-device update wrapper (`f3356x` fields).
//! * `com.xj.ota.data.entity.GameSirersionsState` — server firmware manifest
//!   (`DataBean`, `FirmwareDataBean`).
//! * `com.xj.ota.data.model.DeviceStatusInfo` — device state bean, including
//!   `FIRMWARE_TYPE_*` constants and its own `FirmwareDataBean`.
//! * `com.xj.ota.data.model.RadiatorInfo` — temperature / voltage pair.
//!
//! Android-only members (`Bundle mExtras`, `Context`, lists typed as raw
//! `List`) are dropped or replaced by std equivalents.

use super::check::needs_upgrade;
use std::collections::BTreeMap;

/// Firmware types used by `DeviceStatusInfo` and the server manifest.
pub mod firmware_type {
    pub const MAIN: &str = "main";
    pub const PD: &str = "pd";
    pub const SLAVE: &str = "slave";
}

/// `GameSirersionsState.FirmwareDataBean`: one manifest entry
/// (type, version, bin file name).
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct FirmwareEntry {
    pub firmware_type: String,
    pub version: String,
    pub bin_file: String,
}

impl FirmwareEntry {
    pub fn new(firmware_type: impl Into<String>, version: impl Into<String>, bin_file: impl Into<String>) -> Self {
        Self {
            firmware_type: firmware_type.into(),
            version: version.into(),
            bin_file: bin_file.into(),
        }
    }

    /// `OTAInfor.j(String)` matches the type case-insensitively.
    pub fn matches_type(&self, other: &str) -> bool {
        self.firmware_type.eq_ignore_ascii_case(other)
    }
}

/// `GameSirersionsState.DataBean`: a single available firmware release.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct FirmwareInfo {
    pub version: String,
    pub path: String,
    pub md5_file: String,
    pub manifest: Vec<FirmwareEntry>,
    pub update_status: i32,
    pub upgrade_msg: String,
    pub upgrade_time: String,
    /// `l_decode` / `r_decode` from the Java bean (vendor decoding flags).
    pub l_decode: i32,
    pub r_decode: i32,
}

impl FirmwareInfo {
    /// Find a manifest entry by firmware type, case-insensitive.
    pub fn entry_for_type(&self, firmware_type: &str) -> Option<&FirmwareEntry> {
        self.manifest
            .iter()
            .find(|e| e.matches_type(firmware_type))
    }
}

/// `OTAInfor` update metadata bean.
///
/// Field names are inferred from the Java getters and from usage in
/// `BaseOTA.g()`: `d()` is the download URL, `e()` the save path, `f()` the
/// expected MD5, `n()` the mode (0 => verify MD5, non-zero => only existence),
/// `l()` the version string and `k()` the manifest used by update checks.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct OtaInfo {
    pub download_url: String,
    pub save_path: String,
    pub expected_md5: String,
    pub version: String,
    /// 0 => verify MD5; non-zero => only check file existence.
    pub mode: i32,
    /// Update manifest (server firmware list).
    pub manifest: Vec<FirmwareEntry>,
    /// Device firmware list used for per-component update checks.
    pub device_firmware: Vec<FirmwareDataEntry>,
    pub raw_a: String,
    pub raw_b: String,
    pub raw_h: String,
    pub raw_i: String,
    pub raw_j: i32,
    pub raw_k: i32,
}

impl OtaInfo {
    pub fn new(download_url: impl Into<String>, save_path: impl Into<String>, version: impl Into<String>) -> Self {
        Self {
            download_url: download_url.into(),
            save_path: save_path.into(),
            version: version.into(),
            ..Self::default()
        }
    }

    /// `OTAInfor.j(String)` — case-insensitive manifest lookup.
    pub fn entry_for_type(&self, firmware_type: &str) -> Option<&FirmwareEntry> {
        self.manifest
            .iter()
            .find(|e| e.matches_type(firmware_type))
    }

    /// `BaseOTA.c(String)`: with a non-zero mode only file existence matters;
    /// with mode 0 the MD5 must also match. Paths are compared as strings
    /// since no filesystem access is performed here.
    pub fn cached_file_valid(&self, _path: &str, path_exists: bool, actual_md5: Option<&str>) -> bool {
        if self.mode != 0 {
            return path_exists;
        }
        path_exists
            && self
                .expected_md5
                .eq_ignore_ascii_case(actual_md5.unwrap_or(""))
    }
}

/// `DeviceStatusInfo.FirmwareDataBean` (also structurally identical to the
/// Kotlin `DeviceInfo.FirmwareDataBean`): type, version, OTA support flag.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct FirmwareDataEntry {
    pub firmware_type: String,
    pub version: String,
    pub support_ota: bool,
}

impl FirmwareDataEntry {
    pub fn new(firmware_type: impl Into<String>, version: impl Into<String>, support_ota: bool) -> Self {
        Self {
            firmware_type: firmware_type.into(),
            version: version.into(),
            support_ota,
        }
    }
}

/// `RadiatorInfo` — temperature and voltage readings.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash)]
pub struct RadiatorInfo {
    pub temp: i32,
    pub voltage: i32,
}

impl RadiatorInfo {
    pub const fn new(temp: i32, voltage: i32) -> Self {
        Self { temp, voltage }
    }
}

/// `DeviceStatusInfo` portable bean.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DeviceStatusInfo {
    pub hardware: String,
    pub ble_ver: String,
    pub firmware: String,
    pub gcm_version: String,
    pub battery: i32,
    pub additional_battery_levels: Vec<i32>,
    pub slave_battery: i32,
    pub bilateral_device_state: bool,
    pub bilateral_device_state_changed: bool,
    pub mode: i32,
    pub firmware_data: Vec<FirmwareDataEntry>,
    pub radiator_info: Option<RadiatorInfo>,
    /// Replacement for the Android `Bundle mExtras`.
    pub extras: BTreeMap<String, String>,
}

impl Default for DeviceStatusInfo {
    fn default() -> Self {
        Self {
            hardware: String::new(),
            ble_ver: "0.0".into(),
            firmware: "0.0".into(),
            gcm_version: "0.0".into(),
            battery: 0,
            additional_battery_levels: Vec::new(),
            slave_battery: 0,
            bilateral_device_state: false,
            bilateral_device_state_changed: false,
            mode: 0,
            firmware_data: Vec::new(),
            radiator_info: None,
            extras: BTreeMap::new(),
        }
    }
}

impl DeviceStatusInfo {
    /// `setMode(int)` ignores non-positive values.
    pub fn set_mode(&mut self, mode: i32) {
        if mode > 0 {
            self.mode = mode;
        }
    }

    /// `getBattery2()`: empty string when <= 0, else `"N%"`.
    pub fn battery_str(&self) -> String {
        percent_str(self.battery)
    }

    /// `getSlaveBatteryStr()`.
    pub fn slave_battery_str(&self) -> String {
        percent_str(self.slave_battery)
    }

    /// `setFirmwareDataBean`: replace by type when present, else append.
    pub fn set_firmware(&mut self, entry: FirmwareDataEntry) {
        if let Some(existing) = self
            .firmware_data
            .iter_mut()
            .find(|e| e.firmware_type == entry.firmware_type)
        {
            *existing = entry;
        } else {
            self.firmware_data.push(entry);
        }
    }

    pub fn remove_firmware(&mut self, firmware_type: &str) {
        self.firmware_data
            .retain(|e| e.firmware_type != firmware_type);
    }

    pub fn firmware_for_type(&self, firmware_type: &str) -> Option<&FirmwareDataEntry> {
        self.firmware_data
            .iter()
            .find(|e| e.firmware_type == firmware_type)
    }
}

fn percent_str(value: i32) -> String {
    if value <= 0 {
        String::new()
    } else {
        format!("{}%", value)
    }
}

/// `GamepadInfor` per-device update wrapper.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct GamepadInfo {
    pub device_name: String,
    pub ble_name: String,
    pub device_status: Option<DeviceStatusInfo>,
    pub ota_info: Option<OtaInfo>,
    pub upgrading: bool,
}

impl GamepadInfo {
    /// `GamepadInfor.f()`: for "GameSir-X4A for Xbox" join per-component
    /// versions with '-', otherwise return the single OTA version.
    pub fn effective_version(&self) -> Option<String> {
        let ota = self.ota_info.as_ref()?;
        let is_x4a = self.device_name.eq_ignore_ascii_case("GameSir-X4A for Xbox");
        if !is_x4a {
            return Some(ota.version.clone());
        }
        let status = self.device_status.as_ref()?;
        let parts: Vec<String> = status
            .firmware_data
            .iter()
            .map(|fw| {
                ota.entry_for_type(&fw.firmware_type)
                    .map(|e| e.version.clone())
                    .unwrap_or_else(|| fw.version.clone())
            })
            .collect();
        if parts.is_empty() {
            Some(ota.version.clone())
        } else {
            Some(parts.join("-"))
        }
    }

    /// `GamepadInfor.g()`: decide whether an update is available using the
    /// `DefaultUpdateCheck` rules (`allow_different = true`).
    pub fn can_update(&self, current_version: &str) -> bool {
        let Some(ota) = &self.ota_info else {
            return false;
        };
        if current_version.is_empty() {
            return false;
        }
        // `DefaultUpdateCheck.a()` falls back to a single version comparison
        // when either firmware list is null (no DeviceStatusInfo attached).
        let Some(status) = &self.device_status else {
            return needs_upgrade(&ota.version, current_version, true);
        };
        needs_upgrade(&ota.version, current_version, true)
            || super::check::is_update_available(&status.firmware_data, &ota.manifest, true)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn device_status_defaults_match_java_initializers() {
        let d = DeviceStatusInfo::default();
        assert_eq!(d.ble_ver, "0.0");
        assert_eq!(d.firmware, "0.0");
        assert_eq!(d.gcm_version, "0.0");
        assert!(d.hardware.is_empty());
        assert_eq!(d.battery_str(), "");
        assert_eq!(d.slave_battery_str(), "");
    }

    #[test]
    fn battery_strings_follow_bounds() {
        let mut d = DeviceStatusInfo::default();
        d.battery = 0;
        assert_eq!(d.battery_str(), "");
        d.battery = 85;
        assert_eq!(d.battery_str(), "85%");
        d.slave_battery = 42;
        assert_eq!(d.slave_battery_str(), "42%");
    }

    #[test]
    fn set_mode_ignores_non_positive() {
        let mut d = DeviceStatusInfo::default();
        d.set_mode(0);
        d.set_mode(-3);
        assert_eq!(d.mode, 0);
        d.set_mode(7);
        assert_eq!(d.mode, 7);
    }

    #[test]
    fn firmware_entries_are_replaced_by_type() {
        let mut d = DeviceStatusInfo::default();
        d.set_firmware(FirmwareDataEntry::new(firmware_type::MAIN, "1.0", true));
        d.set_firmware(FirmwareDataEntry::new(firmware_type::PD, "2.0", true));
        assert_eq!(d.firmware_data.len(), 2);
        d.set_firmware(FirmwareDataEntry::new(firmware_type::MAIN, "1.1", true));
        assert_eq!(d.firmware_data.len(), 2);
        assert_eq!(d.firmware_for_type(firmware_type::MAIN).unwrap().version, "1.1");
        d.remove_firmware(firmware_type::PD);
        assert_eq!(d.firmware_data.len(), 1);
    }

    #[test]
    fn cached_file_valid_respects_mode_and_md5() {
        let mut ota = OtaInfo::new("u", "p", "1.0");
        ota.expected_md5 = "abc".into();
        assert!(!ota.cached_file_valid("p", false, Some("abc")));
        assert!(ota.cached_file_valid("p", true, Some("abc")));
        assert!(ota.cached_file_valid("p", true, Some("ABC")));
        assert!(!ota.cached_file_valid("p", true, Some("def")));
        ota.mode = 1;
        assert!(ota.cached_file_valid("p", true, None), "mode!=0 skips MD5");
        assert!(!ota.cached_file_valid("p", false, Some("def")));
    }

    #[test]
    fn ota_entry_lookup_is_case_insensitive() {
        let mut ota = OtaInfo::default();
        ota.manifest.push(FirmwareEntry::new("main", "3.0", "ota.bin"));
        assert!(ota.entry_for_type("MAIN").is_some());
        assert!(ota.entry_for_type("pd").is_none());
    }

    #[test]
    fn x4a_effective_version_joins_components() {
        let status = DeviceStatusInfo {
            firmware_data: vec![
                FirmwareDataEntry::new(firmware_type::MAIN, "1.0", true),
                FirmwareDataEntry::new(firmware_type::PD, "2.0", true),
            ],
            ..DeviceStatusInfo::default()
        };
        let ota = OtaInfo {
            manifest: vec![
                FirmwareEntry::new("main", "1.1", "main.bin"),
                FirmwareEntry::new("pd", "2.0", "pd.bin"),
            ],
            ..OtaInfo::default()
        };
        let g = GamepadInfo {
            device_name: "GameSir-X4A for Xbox".into(),
            device_status: Some(status),
            ota_info: Some(ota),
            ..GamepadInfo::default()
        };
        assert_eq!(g.effective_version().as_deref(), Some("1.1-2.0"));

        let plain = GamepadInfo {
            device_name: "Gamesir-G8".into(),
            ota_info: Some(OtaInfo::new("u", "p", "9.9")),
            ..GamepadInfo::default()
        };
        assert_eq!(plain.effective_version().as_deref(), Some("9.9"));
        assert!(!plain.can_update("9.9"));
        assert!(plain.can_update("9.8"));
    }

    #[test]
    fn radiator_info_round_trip() {
        let r = RadiatorInfo::new(38, 5_000);
        assert_eq!(r.temp, 38);
        assert_eq!(r.voltage, 5_000);
    }
}
