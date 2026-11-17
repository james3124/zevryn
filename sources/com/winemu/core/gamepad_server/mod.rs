//! Rust model of the Wine-side gamepad server (`com.winemu.core.gamepad`).
//!
//! This is the **server** side of the gamepad stack — the state that is
//! mirrored into the native Wine gamepad buffers. It is distinct from the
//! XiaoJi/GameSir BLE device model in `com.xj.device.ble`.
//!
//! Reference sources (read-only):
//! - `sources/com/winemu/core/gamepad/GamepadState.java` — buffer layout,
//!   neutral values, button/hat setters.
//! - `sources/com/winemu/core/gamepad/GamepadInputRouter.java` — button mapping
//!   (`z`), D-pad axis thresholds (`A`), axis/trigger conversions (`e`, `f`,
//!   `p`), key events (`u`) and motion events (`w`/`A`).
//! - `sources/com/winemu/core/gamepad/GamepadServerManager.java` — four state
//!   buffers, gamepad count, rumble callback.
//! - `sources/com/winemu/core/gamepad/GamepadSlotManager.java` — four slots,
//!   virtual gamepad in slot 0, descriptor grouping, compaction, swap.
//! - `sources/com/winemu/core/gamepad/PlayerSlot.java`, `SlotState.java`,
//!   `GamepadDevice.java`, `VirtualGamepadController.java`, `DeviceConfig.java`.
//!
//! Only `std` is used. Wire into the crate with, e.g.:
//! ```ignore
//! #[path = "../sources/com/winemu/core/gamepad_server/mod.rs"]
//! pub mod gamepad_server;
//! ```

pub mod device_config;
pub mod gamepad_state;
pub mod player_slot;
pub mod server_manager;
pub mod slot_manager;
pub mod virtual_controller;

pub use self::device_config::DeviceConfig;
pub use self::gamepad_state::{
    axis_from_float, normalized_trigger, trigger_from_float, GamepadButton, GamepadButtons,
    GamepadHat, GamepadState, GamepadStatePatch, AXIS_SCALE, DPAD_AXIS_THRESHOLD,
    TRIGGER_NEUTRAL,
};
pub use self::player_slot::{
    GamepadDevice, PhysicalDevice, PlayerSlot, SlotOccupancy, SlotState, VirtualDevice,
};
pub use self::server_manager::{
    GamepadServerManager, RumbleCommand, ServerError, GAMEPAD_BUFFER_COUNT,
};
pub use self::slot_manager::{SlotError, SlotManager, SLOT_COUNT, VIRTUAL_SLOT};
pub use self::virtual_controller::{
    VirtualControllerError, VirtualGamepadController, VirtualGamepadEvent,
};
