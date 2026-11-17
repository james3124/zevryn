//! Rust port of the portable logic in `com.xj.devicemanagement`
//! (GameHub Lite v5.1.8).
//!
//! * [`device_info`] — the `DeviceInfo` / `FirmwareDataBean` data structs.
//! * [`byte_utils`] — hex / checksum / CRC helpers used by OTA and BLE code.
//!
//! Not implemented: BLE (`BLEScanner`, `BLEService`, receivers), USB
//! (`USBMonitor`, `UsbOTGService`, permissions), broadcast receivers and
//! coroutine/callback plumbing. Only portable data and pure functions live here.

pub mod byte_utils;
pub mod device_info;

pub use byte_utils::{
    byte_sum, bytes_to_u32_be, crc16_ccitt, format_bytes_prefixed, hex_bracketed, hex_upper,
    u16_to_le_bytes,
};
pub use device_info::{DeviceInfo, FirmwareDataBean};
