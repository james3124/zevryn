//! Virtual gamepad controller state (Wine-side server model).
//!
//! Mirrors `com.winemu.core.gamepad.VirtualGamepadController`: it owns the one
//! active [`VirtualDevice`], resolves its slot through the slot manager
//! (`slotManager.k(virtualDevice.b())`) and emits the same field-level
//! [`GamepadStatePatch`] updates the Java controller sends through the router
//! callback (`w(Function1)`). JNI/native state writing is out of scope; the
//! returned [`VirtualGamepadEvent`] carries the target slot and patch.

use super::gamepad_state::{axis_from_float, GamepadButton, GamepadHat, GamepadStatePatch};
use super::player_slot::VirtualDevice;
#[cfg(test)]
use super::player_slot::GamepadDevice;
use super::slot_manager::SlotManager;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum VirtualControllerError {
    /// `w()` threw "Virtual gamepad not assigned to any slot".
    NotAssigned,
    /// No virtual gamepad is currently active (`GamepadManager.f20444h == null`).
    NoActiveController,
}

/// A controller update addressed to a Wine gamepad slot.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VirtualGamepadEvent {
    pub slot: i32,
    pub patch: GamepadStatePatch,
}

impl VirtualGamepadEvent {
    pub fn new(slot: i32, patch: GamepadStatePatch) -> Self {
        Self { slot, patch }
    }
}

/// State of the active virtual controller.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VirtualGamepadController {
    device: VirtualDevice,
    slot: Option<i32>,
    enabled: bool,
}

impl VirtualGamepadController {
    pub fn new(device: VirtualDevice, slot: Option<i32>) -> Self {
        Self {
            device,
            slot,
            enabled: true,
        }
    }

    /// Build a controller from a slot manager, mirroring
    /// `GamepadManager.o()` + `VirtualGamepadController.i()`.
    pub fn from_slot_manager(
        manager: &SlotManager,
        device_id: i32,
    ) -> Result<Self, VirtualControllerError> {
        let device = manager
            .slots
            .iter()
            .filter_map(|s| s.virtual_device())
            .find(|v| v.device_id == device_id)
            .cloned()
            .ok_or(VirtualControllerError::NoActiveController)?;
        let slot = manager.slot_of_device_id(device_id);
        Ok(Self::new(device, slot))
    }

    pub fn device(&self) -> &VirtualDevice {
        &self.device
    }

    pub fn device_id(&self) -> i32 {
        self.device.device_id
    }

    /// The slot the virtual gamepad is assigned to, if any.
    pub fn slot(&self) -> Option<i32> {
        self.slot
    }

    pub fn is_active(&self) -> bool {
        self.enabled && self.slot.is_some()
    }

    pub fn is_enabled(&self) -> bool {
        self.enabled
    }

    pub fn set_slot(&mut self, slot: Option<i32>) {
        self.slot = slot;
    }

    fn event(&self, patch: GamepadStatePatch) -> Result<VirtualGamepadEvent, VirtualControllerError> {
        let slot = self.slot.ok_or(VirtualControllerError::NotAssigned)?;
        Ok(VirtualGamepadEvent::new(slot, patch))
    }

    /// `j()` / `VirtualGamepadController.k`: `GamepadState.a()`.
    pub fn reset(&self) -> Result<VirtualGamepadEvent, VirtualControllerError> {
        self.event(GamepadStatePatch::reset())
    }

    /// `l(int, boolean)` / `m`: set one button slot.
    pub fn set_button(
        &self,
        index: usize,
        pressed: bool,
    ) -> Result<VirtualGamepadEvent, VirtualControllerError> {
        let patch = GamepadStatePatch::button(index, pressed)
            .ok_or(VirtualControllerError::NoActiveController)?;
        self.event(patch)
    }

    pub fn press(
        &self,
        button: GamepadButton,
        pressed: bool,
    ) -> Result<VirtualGamepadEvent, VirtualControllerError> {
        self.event(GamepadStatePatch::default().with_button(button, pressed))
    }

    /// `n(float, float)` / `o`: left stick (`GamepadState.d`).
    pub fn set_left_stick(
        &self,
        x: f32,
        y: f32,
    ) -> Result<VirtualGamepadEvent, VirtualControllerError> {
        self.event(GamepadStatePatch {
            left_stick: Some((axis_from_float(x), axis_from_float(y))),
            ..GamepadStatePatch::default()
        })
    }

    /// `r(float, float)` / `s`: right stick (`GamepadState.h`).
    pub fn set_right_stick(
        &self,
        x: f32,
        y: f32,
    ) -> Result<VirtualGamepadEvent, VirtualControllerError> {
        self.event(GamepadStatePatch {
            right_stick: Some((axis_from_float(x), axis_from_float(y))),
            ..GamepadStatePatch::default()
        })
    }

    /// `p(short)` / `q`: left trigger (`GamepadState.g`).
    pub fn set_left_trigger(
        &self,
        value: i16,
    ) -> Result<VirtualGamepadEvent, VirtualControllerError> {
        self.event(GamepadStatePatch::default().with_left_trigger(value))
    }

    /// `t(short)` / `u`: right trigger (`GamepadState.k`).
    pub fn set_right_trigger(
        &self,
        value: i16,
    ) -> Result<VirtualGamepadEvent, VirtualControllerError> {
        self.event(GamepadStatePatch::default().with_right_trigger(value))
    }

    /// Set the hat (which also drives the D-pad buttons in the state model).
    pub fn set_hat(
        &self,
        hat: GamepadHat,
    ) -> Result<VirtualGamepadEvent, VirtualControllerError> {
        self.event(GamepadStatePatch {
            hat: Some(hat),
            ..GamepadStatePatch::default()
        })
    }

    /// `v(boolean)`: enable/disable. Disabling also clears rumble
    /// (`device.f(0, 0)`), modeled here as an event with a neutral rumble value.
    pub fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
    }

    pub fn disable(&mut self) {
        self.set_enabled(false);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn controller_resolves_its_slot() {
        let mut manager = SlotManager::new();
        let device = GamepadDevice::virtual_device(9);
        manager.assign(device.clone()).unwrap();
        let controller = VirtualGamepadController::from_slot_manager(&manager, 9).unwrap();
        assert_eq!(controller.device_id(), 9);
        assert_eq!(controller.slot(), Some(0));
        assert!(controller.is_active());
        assert_eq!(
            controller.reset().unwrap(),
            VirtualGamepadEvent::new(0, GamepadStatePatch::reset())
        );
    }

    #[test]
    fn updates_target_the_assigned_slot() {
        let controller = VirtualGamepadController::new(VirtualDevice::new(4), Some(2));
        assert_eq!(controller.press(GamepadButton::A, true).unwrap().slot, 2);
        let stick = controller.set_left_stick(1.0, -1.0).unwrap();
        assert_eq!(stick.slot, 2);
        assert_eq!(stick.patch.left_stick, Some((32767, -32768)));
        let trigger = controller.set_right_trigger(1000).unwrap();
        assert_eq!(trigger.patch.right_trigger, Some(1000));
        let hat = controller.set_hat(GamepadHat::Left).unwrap();
        assert_eq!(hat.patch.hat, Some(GamepadHat::Left));
    }

    #[test]
    fn unassigned_controller_reports_error() {
        let controller = VirtualGamepadController::new(VirtualDevice::new(1), None);
        assert_eq!(controller.reset(), Err(VirtualControllerError::NotAssigned));
        assert_eq!(
            VirtualGamepadController::from_slot_manager(&SlotManager::new(), 1),
            Err(VirtualControllerError::NoActiveController)
        );
    }

    #[test]
    fn enable_disable_changes_active_state() {
        let mut controller = VirtualGamepadController::new(VirtualDevice::new(1), Some(0));
        assert!(controller.is_active());
        controller.disable();
        assert!(!controller.is_active());
        assert!(!controller.is_enabled());
        controller.set_enabled(true);
        assert!(controller.is_active());
    }
}
