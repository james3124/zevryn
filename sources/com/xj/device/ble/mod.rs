//! Portable device/controller model for `com.xj.device.ble`.
//!
//! This module mirrors the dependency-free parts of the GameHub Lite v5.1.8
//! controller stack (XiaoJi / GameSir SDK and the device-settings screens).
//! Only pure data models and state logic live here — BLE/USB transport code,
//! Android UI and the native `Gamesir`/Wine `.so` libraries are out of scope.
//!
//! Reference sources (read-only):
//! - `sources/com/xiaoji/sdk/utils/KeyCodeUtil.java` (+ its smali) — button codes
//! - `sources/com/xiaoji/sdk/gcm/GcmProtocol.java` and `gcm/check/*` — frame layout
//! - `sources/com/xj/devicesetting/gamepad/GamePadCalibrationActivity.java`,
//!   `bean/JoystickSettingEntity.java` — calibration model
//! - `sources/com/xj/bussiness/devicemanagement/utils/DeviceConnectionMonitor.java`
//!   and `resources/res/xml/device_filter.xml` — connection lifecycle / VID-PIDs
//!
//! Wire into the crate with, e.g.:
//! ```ignore
//! #[path = "../sources/com/xj/device/ble/mod.rs"]
//! pub mod device_ble;
//! ```

pub mod calibration;
pub mod device_connection_state;
pub mod gamepad_button;
pub mod gcm_packet;

pub use self::calibration::{AxisRange, CalibrationPoint};
pub use self::device_connection_state::{
    ConnectionEvent, DeviceConnectionState, InvalidTransition,
};
pub use self::gamepad_button::{ButtonMask, GamepadButton};
pub use self::gcm_packet::{checksum, DecodeError, GcmPacket, HeaderForm, offsets};
