//! Slot occupancy model for the Wine-side gamepad server.
//!
//! Mirrors `com.winemu.core.gamepad.PlayerSlot`, `SlotState` and the identity
//! fields of `GamepadDevice` / `GamepadDevice.Physical` /
//! `GamepadDevice.Virtual`. Android-only fields (Vibrator handles, Context) are
//! omitted; only the data the slot logic actually inspects is kept:
//!
//! - `Physical.a()`  -> [`GamepadDevice::descriptor`] (stable identity)
//! - `Physical.b()`  -> [`GamepadDevice::device_id`] (runtime id)
//! - `Physical.c()`  / `Virtual.c()` -> [`GamepadDevice::name`]
//! - `Physical.l()`  -> [`PhysicalDevice::android_device_id`]
//! - `e()`           -> [`GamepadDevice::is_disabled`]
//!
//! Slot 0 is reserved for the (single) virtual gamepad; physical devices may
//! share a slot (merging into one Wine gamepad) or occupy their own.

/// Identity of a physical gamepad, from `GamepadDevice.Physical`.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct PhysicalDevice {
    pub device_id: i32,
    pub name: String,
    pub is_connected: bool,
    pub android_device_id: i32,
    pub vendor_id: i32,
    pub product_id: i32,
    pub descriptor: String,
    pub is_disabled: bool,
}

impl PhysicalDevice {
    pub fn new(device_id: i32, name: impl Into<String>, descriptor: impl Into<String>) -> Self {
        Self {
            device_id,
            name: name.into(),
            is_connected: true,
            android_device_id: -1,
            vendor_id: -1,
            product_id: -1,
            descriptor: descriptor.into(),
            is_disabled: false,
        }
    }
}

/// Identity of the one virtual gamepad, from `GamepadDevice.Virtual`.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct VirtualDevice {
    pub device_id: i32,
    pub name: String,
    pub is_connected: bool,
    pub is_disabled: bool,
}

impl VirtualDevice {
    /// The default name comes from `Virtual` constructor's default parameter:
    /// `name = "Virtual Gamepad"`.
    pub fn new(device_id: i32) -> Self {
        Self {
            device_id,
            name: "Virtual Gamepad".to_owned(),
            is_connected: true,
            is_disabled: false,
        }
    }

    /// `GamepadDevice.Virtual.a()`: `descriptor = "virtual_gamepad_" + deviceId`.
    pub fn descriptor(&self) -> String {
        format!("virtual_gamepad_{}", self.device_id)
    }
}

/// A physical or virtual gamepad, identified as used by the slot manager.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum GamepadDevice {
    Physical(PhysicalDevice),
    Virtual(VirtualDevice),
}

impl GamepadDevice {
    pub fn physical(device_id: i32, name: impl Into<String>, descriptor: impl Into<String>) -> Self {
        Self::Physical(PhysicalDevice::new(device_id, name, descriptor))
    }

    pub fn virtual_device(device_id: i32) -> Self {
        Self::Virtual(VirtualDevice::new(device_id))
    }

    /// `GamepadDevice.a()`: stable descriptor key.
    pub fn descriptor(&self) -> &str {
        match self {
            Self::Physical(p) => &p.descriptor,
            Self::Virtual(v) => {
                // Virtual descriptors are derived; callers that need an owned
                // key use `virtual_device.descriptor()` directly.
                let _ = v;
                "virtual"
            }
        }
    }

    pub fn owned_descriptor(&self) -> String {
        match self {
            Self::Physical(p) => p.descriptor.clone(),
            Self::Virtual(v) => v.descriptor(),
        }
    }

    /// `GamepadDevice.b()`: runtime device id.
    pub fn device_id(&self) -> i32 {
        match self {
            Self::Physical(p) => p.device_id,
            Self::Virtual(v) => v.device_id,
        }
    }

    /// `GamepadDevice.c()`: display name.
    pub fn name(&self) -> &str {
        match self {
            Self::Physical(p) => &p.name,
            Self::Virtual(v) => &v.name,
        }
    }

    /// `GamepadDevice.e()`: disabled devices are excluded from slot assignment.
    pub fn is_disabled(&self) -> bool {
        match self {
            Self::Physical(p) => p.is_disabled,
            Self::Virtual(v) => v.is_disabled,
        }
    }

    pub fn is_physical(&self) -> bool {
        matches!(self, Self::Physical(_))
    }

    pub fn is_virtual(&self) -> bool {
        matches!(self, Self::Virtual(_))
    }

    pub fn as_physical(&self) -> Option<&PhysicalDevice> {
        match self {
            Self::Physical(p) => Some(p),
            Self::Virtual(_) => None,
        }
    }

    pub fn as_virtual(&self) -> Option<&VirtualDevice> {
        match self {
            Self::Virtual(v) => Some(v),
            Self::Physical(_) => None,
        }
    }
}

/// A compact view of a slot, from `GamepadSlotManager.l()` /
/// `PlayerSlot(slotId, name, isActive)`.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct PlayerSlot {
    pub slot_id: i32,
    pub name: String,
    pub is_active: bool,
}

impl PlayerSlot {
    pub fn new(slot_id: i32, name: impl Into<String>, is_active: bool) -> Self {
        Self {
            slot_id,
            name: name.into(),
            is_active,
        }
    }
}

/// Classifies what occupies a [`SlotState`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SlotOccupancy {
    Empty,
    PhysicalOnly,
    VirtualOnly,
    /// A physical device is merged with the virtual gamepad (slot 0).
    Mixed,
}

/// One of the four Wine gamepad slots, from `SlotState(index, devices)`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SlotState {
    pub index: i32,
    pub devices: Vec<GamepadDevice>,
}

impl SlotState {
    pub fn new(index: i32) -> Self {
        Self {
            index,
            devices: Vec::new(),
        }
    }

    pub fn with_devices(index: i32, devices: Vec<GamepadDevice>) -> Self {
        Self { index, devices }
    }

    pub fn is_empty(&self) -> bool {
        self.devices.is_empty()
    }

    /// `SlotState.d()`: first physical device in the slot.
    pub fn physical(&self) -> Option<&PhysicalDevice> {
        self.devices.iter().find_map(GamepadDevice::as_physical)
    }

    /// `SlotState.e()`: first virtual device in the slot.
    pub fn virtual_device(&self) -> Option<&VirtualDevice> {
        self.devices.iter().find_map(GamepadDevice::as_virtual)
    }

    pub fn occupancy(&self) -> SlotOccupancy {
        match (self.physical().is_some(), self.virtual_device().is_some()) {
            (false, false) => SlotOccupancy::Empty,
            (true, false) => SlotOccupancy::PhysicalOnly,
            (false, true) => SlotOccupancy::VirtualOnly,
            (true, true) => SlotOccupancy::Mixed,
        }
    }

    /// `SlotState.b()`: "Empty" / physical name / virtual name /
    /// `"physical + virtual"` when both are present.
    pub fn display_name(&self) -> String {
        match (self.physical(), self.virtual_device()) {
            (None, None) => "Empty".to_owned(),
            (None, Some(v)) => v.name.clone(),
            (Some(p), None) => p.name.clone(),
            (Some(p), Some(v)) => format!("{} + {}", p.name, v.name),
        }
    }

    pub fn to_player_slot(&self) -> PlayerSlot {
        PlayerSlot::new(self.index, self.display_name(), !self.is_empty())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn virtual_descriptor_matches_reference() {
        let v = VirtualDevice::new(7);
        assert_eq!(v.name, "Virtual Gamepad");
        assert_eq!(v.descriptor(), "virtual_gamepad_7");
        let device = GamepadDevice::virtual_device(7);
        assert_eq!(device.owned_descriptor(), "virtual_gamepad_7");
        assert_eq!(device.device_id(), 7);
        assert!(device.is_virtual());
    }

    #[test]
    fn slot_state_display_and_occupancy() {
        let mut slot = SlotState::new(0);
        assert_eq!(slot.display_name(), "Empty");
        assert_eq!(slot.occupancy(), SlotOccupancy::Empty);
        assert_eq!(slot.to_player_slot(), PlayerSlot::new(0, "Empty", false));

        slot.devices
            .push(GamepadDevice::virtual_device(41));
        assert_eq!(slot.display_name(), "Virtual Gamepad");
        assert_eq!(slot.occupancy(), SlotOccupancy::VirtualOnly);

        slot.devices
            .push(GamepadDevice::physical(3, "Xbox", "desc-1"));
        assert_eq!(slot.display_name(), "Xbox + Virtual Gamepad");
        assert_eq!(slot.occupancy(), SlotOccupancy::Mixed);
        assert_eq!(slot.physical().unwrap().descriptor, "desc-1");
        assert_eq!(slot.virtual_device().unwrap().device_id, 41);
        assert_eq!(
            slot.to_player_slot(),
            PlayerSlot::new(0, "Xbox + Virtual Gamepad", true)
        );
    }
}
