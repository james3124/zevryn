//! Pure slot-assignment logic for the Wine-side gamepad server.
//!
//! Mirrors `com.winemu.core.gamepad.GamepadSlotManager`: four slots, slot 0
//! reserved for the single virtual gamepad, physical devices grouped by their
//! stable descriptor and merged into a slot.
//!
//! Note on the decompilation: `GamepadSlotManager.g()`/`f()` contain empty loop
//! bodies whose control flow was mangled by JADX's Kotlin inlining. The
//! behaviour implemented here follows the observable design and log messages
//! ("Compacted physical device ... to slot", "Virtual gamepad assigned to
//! slot 0", "Swapped physical devices between slot ... (virtual gamepad stays
//! in slot 0)"): the first free slot is one without a physical device, so an
//! unclaimed physical gamepad can merge into the virtual one in slot 0.

use super::player_slot::{GamepadDevice, PlayerSlot, SlotState};

pub const SLOT_COUNT: usize = 4;
pub const VIRTUAL_SLOT: i32 = 0;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SlotError {
    /// `m()` returned -1 because `device.e()` was true.
    DeviceDisabled,
    /// `m()` threw "Virtual gamepad already exists in slot 0".
    DuplicateVirtual,
    /// `m()` threw "No available slots for device ...".
    NoAvailableSlots,
    /// `t()` ignored an out-of-range or identical slot pair.
    InvalidSwap,
}

#[derive(Debug, Clone)]
pub struct SlotManager {
    pub slots: Vec<SlotState>,
    pub virtual_enabled: bool,
}

impl Default for SlotManager {
    fn default() -> Self {
        Self::new()
    }
}

impl SlotManager {
    pub fn new() -> Self {
        Self {
            slots: (0..SLOT_COUNT as i32).map(SlotState::new).collect(),
            virtual_enabled: false,
        }
    }

    fn slot_mut(&mut self, index: i32) -> Option<&mut SlotState> {
        if index >= 0 && (index as usize) < self.slots.len() {
            Some(&mut self.slots[index as usize])
        } else {
            None
        }
    }

    fn slot(&self, index: i32) -> Option<&SlotState> {
        if index >= 0 && (index as usize) < self.slots.len() {
            Some(&self.slots[index as usize])
        } else {
            None
        }
    }

    /// `GamepadSlotManager.h(String)`: find a slot containing a device with
    /// this descriptor. Returns -1 when absent.
    pub fn find_by_descriptor(&self, descriptor: &str) -> i32 {
        for slot in &self.slots {
            for device in &slot.devices {
                if device.owned_descriptor() == descriptor {
                    return slot.index;
                }
            }
        }
        -1
    }

    /// `GamepadSlotManager.g()`: first slot with no physical device. A virtual
    /// gamepad alone does not block the slot (it is merged with the first
    /// unclaimed physical device).
    pub fn first_free_slot(&self) -> i32 {
        for slot in &self.slots {
            if slot.physical().is_none() {
                return slot.index;
            }
        }
        -1
    }

    /// `GamepadSlotManager.k(int)`: slot index of a runtime device id.
    pub fn slot_of_device_id(&self, device_id: i32) -> Option<i32> {
        self.slots.iter().find(|slot| slot.devices.iter().any(|d| d.device_id() == device_id)).map(|s| s.index)
    }

    /// `GamepadSlotManager.m(GamepadDevice)`: assign a device to a slot.
    /// Returns the slot index, or a [`SlotError`]. A disabled device yields
    /// -1 in the reference; here it is reported explicitly as
    /// [`SlotError::DeviceDisabled`].
    pub fn assign(&mut self, device: GamepadDevice) -> Result<i32, SlotError> {
        if device.is_disabled() {
            return Err(SlotError::DeviceDisabled);
        }

        if device.is_virtual() {
            let slot = self
                .slot_mut(VIRTUAL_SLOT)
                .ok_or(SlotError::InvalidSwap)?;
            if slot.devices.iter().any(GamepadDevice::is_virtual) {
                return Err(SlotError::DuplicateVirtual);
            }
            slot.devices.push(device);
            self.virtual_enabled = true;
            return Ok(VIRTUAL_SLOT);
        }

        let descriptor = device.owned_descriptor();
        let existing = self.find_by_descriptor(&descriptor);
        let slot_index = if existing >= 0 {
            existing
        } else {
            let free = self.first_free_slot();
            if free < 0 {
                return Err(SlotError::NoAvailableSlots);
            }
            free
        };
        self.slot_mut(slot_index)
            .ok_or(SlotError::InvalidSwap)?
            .devices
            .push(device);
        Ok(slot_index)
    }

    /// `GamepadSlotManager.f()`: compact physical devices, keeping the virtual
    /// gamepad in slot 0 and reassigning physicals to the first free slots in
    /// their original order.
    pub fn compact(&mut self) {
        let virtual_device = self
            .slot(VIRTUAL_SLOT)
            .and_then(|s| s.virtual_device().cloned())
            .map(GamepadDevice::Virtual);
        let physicals: Vec<GamepadDevice> = self
            .slots
            .iter()
            .flat_map(|s| s.devices.iter().filter(|d| d.is_physical()).cloned())
            .collect();

        for slot in &mut self.slots {
            slot.devices.clear();
        }
        if let Some(virtual_device) = virtual_device {
            self.slots[VIRTUAL_SLOT as usize].devices.push(virtual_device);
        }
        for physical in physicals {
            let descriptor = physical.owned_descriptor();
            let index = {
                let existing = self.find_by_descriptor(&descriptor);
                if existing >= 0 {
                    existing
                } else {
                    self.first_free_slot()
                }
            };
            if index >= 0 {
                self.slot_mut(index).unwrap().devices.push(physical);
            }
        }
    }

    /// `GamepadSlotManager.q(GamepadDevice)`: remove a device by runtime id.
    /// Returns the slot it was removed from, or -1. Physical removal triggers
    /// compaction; virtual removal clears `virtual_enabled`.
    pub fn remove(&mut self, device: &GamepadDevice) -> i32 {
        let mut found = (-1, None); // (slot index, position in slot)
        for slot in &self.slots {
            if let Some(pos) = slot.devices.iter().position(|d| d.device_id() == device.device_id()) {
                found = (slot.index, Some(pos));
                break;
            }
        }
        let (index, pos) = found;
        if let Some(pos) = pos {
            self.slots[index as usize].devices.remove(pos);
            if device.is_virtual() {
                self.virtual_enabled = false;
            } else {
                self.compact();
            }
        }
        index
    }

    /// `GamepadSlotManager.n(GamepadDevice)`: mark a device disconnected by
    /// identity (descriptor for physical, any virtual for virtual).
    pub fn disconnect(&mut self, device: &GamepadDevice) -> i32 {
        let mut found = -1;
        for slot in &self.slots {
            let matched = if device.is_virtual() {
                slot.devices.iter().any(GamepadDevice::is_virtual)
            } else {
                slot.devices.iter().any(|d| {
                    d.is_physical() && d.owned_descriptor() == device.owned_descriptor()
                })
            };
            if matched {
                found = slot.index;
                break;
            }
        }
        if found >= 0 {
            if device.is_virtual() {
                self.virtual_enabled = false;
            } else {
                self.compact();
            }
        }
        found
    }

    /// `GamepadSlotManager.t(int, int)`: swap the physical devices of two
    /// slots. The virtual gamepad stays in slot 0. Out-of-range or equal
    /// indices are ignored.
    pub fn swap(&mut self, a: i32, b: i32) -> Result<(), SlotError> {
        if a < 0 || b < 0 || a == b || self.slot(a).is_none() || self.slot(b).is_none() {
            return Err(SlotError::InvalidSwap);
        }
        let physicals_a: Vec<GamepadDevice> = self
            .slot(a)
            .unwrap()
            .devices
            .iter()
            .filter(|d| d.is_physical())
            .cloned()
            .collect();
        let physicals_b: Vec<GamepadDevice> = self
            .slot(b)
            .unwrap()
            .devices
            .iter()
            .filter(|d| d.is_physical())
            .cloned()
            .collect();

        for slot in &mut self.slots {
            if slot.index == a || slot.index == b {
                slot.devices.retain(GamepadDevice::is_virtual);
            }
        }
        self.slot_mut(a).unwrap().devices.extend(physicals_b);
        self.slot_mut(b).unwrap().devices.extend(physicals_a);

        // Slot 0 must keep the virtual device at the front.
        let slot0 = &mut self.slots[VIRTUAL_SLOT as usize];
        if let Some(pos) = slot0.devices.iter().position(GamepadDevice::is_virtual) {
            let virtual_device = slot0.devices.remove(pos);
            slot0.devices.insert(0, virtual_device);
        }
        Ok(())
    }

    /// `GamepadSlotManager.j(int)`: devices in a slot (empty for bad index).
    pub fn devices_in(&self, index: i32) -> Vec<GamepadDevice> {
        self.slot(index).map(|s| s.devices.clone()).unwrap_or_default()
    }

    /// `GamepadSlotManager.l()`: player-slot list for the UI/count sync.
    pub fn player_slots(&self) -> Vec<PlayerSlot> {
        self.slots.iter().map(SlotState::to_player_slot).collect()
    }

    /// Number of occupied slots (drives the native gamepad count in
    /// `GamepadManager.A0()`).
    pub fn active_count(&self) -> usize {
        self.slots.iter().filter(|s| !s.is_empty()).count()
    }

    /// `GamepadSlotManager.s()`: is the virtual gamepad enabled?
    pub fn has_virtual(&self) -> bool {
        self.virtual_enabled
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn eight_slots_max_and_compaction() {
        let mut mgr = SlotManager::new();
        assert_eq!(mgr.slots.len(), 4);
        assert_eq!(mgr.assign(GamepadDevice::virtual_device(1)), Ok(0));
        assert!(mgr.has_virtual());
        // First physical merges into the virtual slot.
        assert_eq!(
            mgr.assign(GamepadDevice::physical(2, "Pad A", "desc-a")),
            Ok(0)
        );
        assert_eq!(
            mgr.assign(GamepadDevice::physical(3, "Pad B", "desc-b")),
            Ok(1)
        );
        assert_eq!(
            mgr.assign(GamepadDevice::physical(4, "Pad C", "desc-c")),
            Ok(2)
        );
        assert_eq!(
            mgr.assign(GamepadDevice::physical(5, "Pad D", "desc-d")),
            Ok(3)
        );
        assert_eq!(
            mgr.assign(GamepadDevice::physical(6, "Pad E", "desc-e")),
            Err(SlotError::NoAvailableSlots)
        );
        assert_eq!(mgr.active_count(), 4);
    }

    #[test]
    fn descriptor_merge_and_duplicate_virtual() {
        let mut mgr = SlotManager::new();
        assert_eq!(
            mgr.assign(GamepadDevice::physical(10, "Pad X", "same")),
            Ok(0)
        );
        // Same descriptor -> merged into the same slot.
        assert_eq!(
            mgr.assign(GamepadDevice::physical(11, "Pad X again", "same")),
            Ok(0)
        );
        assert_eq!(mgr.devices_in(0).len(), 2);
        assert_eq!(mgr.find_by_descriptor("same"), 0);
        assert_eq!(mgr.find_by_descriptor("missing"), -1);
        assert_eq!(mgr.slot_of_device_id(11), Some(0));
        assert_eq!(mgr.slot_of_device_id(99), None);

        mgr.assign(GamepadDevice::virtual_device(20)).unwrap();
        assert_eq!(
            mgr.assign(GamepadDevice::virtual_device(21)),
            Err(SlotError::DuplicateVirtual)
        );
    }

    #[test]
    fn remove_compacts_and_disables_virtual() {
        let mut mgr = SlotManager::new();
        let virtual_pad = GamepadDevice::virtual_device(1);
        mgr.assign(virtual_pad.clone()).unwrap();
        let pad_a = GamepadDevice::physical(2, "A", "a");
        let pad_b = GamepadDevice::physical(3, "B", "b");
        mgr.assign(pad_a.clone()).unwrap(); // slot 0
        mgr.assign(pad_b.clone()).unwrap(); // slot 1

        assert_eq!(mgr.remove(&pad_a), 0);
        // Compaction moves the remaining physical into the free virtual slot.
        assert_eq!(mgr.devices_in(0).len(), 2);
        assert!(mgr.devices_in(0)[0].is_virtual());
        assert_eq!(mgr.slot_of_device_id(3), Some(0));

        assert_eq!(mgr.remove(&virtual_pad), 0);
        assert!(!mgr.has_virtual());
        assert_eq!(mgr.devices_in(0).len(), 1); // compacted physical
        assert!(mgr.devices_in(0)[0].is_physical());
    }

    #[test]
    fn disconnect_by_descriptor_triggers_compaction() {
        let mut mgr = SlotManager::new();
        let pad = GamepadDevice::physical(5, "Pad", "desc");
        mgr.assign(pad.clone()).unwrap();
        let mut other = pad.clone();
        if let GamepadDevice::Physical(p) = &mut other {
            p.device_id = 99;
        }
        // The reference `n()` only finds the slot and recompacts; it does not
        // remove the device from the slot list.
        assert_eq!(mgr.disconnect(&other), 0);
        assert_eq!(mgr.devices_in(0).len(), 1);
        assert_eq!(mgr.disconnect(&GamepadDevice::physical(6, "Unknown", "none")), -1);
    }

    #[test]
    fn swap_keeps_virtual_in_slot_zero() {
        let mut mgr = SlotManager::new();
        mgr.assign(GamepadDevice::virtual_device(1)).unwrap();
        let pad_a = GamepadDevice::physical(2, "A", "a");
        let pad_b = GamepadDevice::physical(3, "B", "b");
        mgr.assign(pad_a.clone()).unwrap(); // slot 0 with virtual
        mgr.assign(pad_b.clone()).unwrap(); // slot 1

        assert_eq!(mgr.swap(0, 1), Ok(()));
        assert_eq!(mgr.devices_in(0)[0].device_id(), 1); // virtual first
        assert_eq!(mgr.devices_in(0)[1].device_id(), 3);
        assert_eq!(mgr.devices_in(1)[0].device_id(), 2);
        assert_eq!(mgr.swap(0, 0), Err(SlotError::InvalidSwap));
        assert_eq!(mgr.swap(-1, 1), Err(SlotError::InvalidSwap));
    }

    #[test]
    fn player_slots_report_activity() {
        let mut mgr = SlotManager::new();
        let slots = mgr.player_slots();
        assert_eq!(slots.len(), 4);
        assert!(slots.iter().all(|s| !s.is_active && s.name == "Empty"));

        mgr.assign(GamepadDevice::virtual_device(1)).unwrap();
        mgr.assign(GamepadDevice::physical(2, "Pad", "p")).unwrap();
        let slots = mgr.player_slots();
        assert!(slots[0].is_active);
        assert_eq!(slots[0].name, "Pad + Virtual Gamepad");
        assert!(!slots[1].is_active);
    }

    #[test]
    fn disabled_device_is_not_assigned() {
        let mut mgr = SlotManager::new();
        let mut pad = GamepadDevice::physical(1, "Pad", "p");
        if let GamepadDevice::Physical(p) = &mut pad {
            p.is_disabled = true;
        }
        assert_eq!(mgr.assign(pad), Err(SlotError::DeviceDisabled));
        assert_eq!(mgr.active_count(), 0);
    }
}
