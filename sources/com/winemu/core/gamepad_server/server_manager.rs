//! Pure state of the Wine-side gamepad server.
//!
//! Mirrors `com.winemu.core.gamepad.GamepadServerManager`: up to four
//! `GamepadState` buffers, the active gamepad count and the rumble callback
//! (`onRumble`). The native socket, JNI methods (`nativeCreate`,
//! `nativeGetGamepadBuffer`, `nativeUpdateGamepadCount`, `nativeDestroy`) and
//! actual Vibrator access are out of scope; only the portable state they
//! operate on is modeled.

use super::gamepad_state::{GamepadState, GamepadStatePatch};
use super::player_slot::GamepadDevice;
use super::slot_manager::SlotManager;

pub const GAMEPAD_BUFFER_COUNT: usize = 4;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ServerError {
    /// `o(int)` threw "GamepadServerManager not initialized".
    NotInitialized,
    /// `o(int)` threw "Invalid gamepad count: ...".
    InvalidCount(i32),
    /// `c(int)` returned null for an index outside 0..4.
    InvalidSlot(i32),
}

/// A rumble request received from the native side (`onRumble`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RumbleCommand {
    pub slot: i32,
    pub strong: i32,
    pub weak: i32,
}

#[derive(Debug, Clone)]
pub struct GamepadServerManager {
    pub socket_path: String,
    pub initialized: bool,
    pub gamepad_count: usize,
    pub states: [GamepadState; GAMEPAD_BUFFER_COUNT],
    pub pending_rumble: Vec<RumbleCommand>,
}

impl Default for GamepadServerManager {
    fn default() -> Self {
        Self::new("")
    }
}

impl GamepadServerManager {
    pub fn new(socket_path: impl Into<String>) -> Self {
        Self {
            socket_path: socket_path.into(),
            initialized: false,
            gamepad_count: 0,
            states: [GamepadState::neutral(); GAMEPAD_BUFFER_COUNT],
            pending_rumble: Vec::new(),
        }
    }

    /// `Companion.a(socketPath)` / `d()`: allocate the four neutral buffers.
    /// No native handle is created here; initialization always succeeds.
    pub fn initialize(&mut self) -> bool {
        for state in &mut self.states {
            state.reset();
        }
        self.initialized = true;
        true
    }

    /// `c(int)`: gamepad state for a slot, `None` for an invalid index.
    pub fn state(&self, index: i32) -> Option<&GamepadState> {
        if index >= 0 && (index as usize) < GAMEPAD_BUFFER_COUNT {
            Some(&self.states[index as usize])
        } else {
            None
        }
    }

    pub fn state_mut(&mut self, index: i32) -> Option<&mut GamepadState> {
        if index >= 0 && (index as usize) < GAMEPAD_BUFFER_COUNT {
            Some(&mut self.states[index as usize])
        } else {
            None
        }
    }

    /// Apply a router/virtual-controller patch to a slot's state.
    pub fn apply(
        &mut self,
        index: i32,
        patch: &GamepadStatePatch,
    ) -> Result<(), ServerError> {
        let state = self.state_mut(index).ok_or(ServerError::InvalidSlot(index))?;
        state.apply(patch);
        Ok(())
    }

    /// `o(int)`: reset the first `count` states and update the native count.
    /// The reference validates `0 <= count < 5` but only four buffers exist,
    /// so this model accepts `0..=4` and rejects larger values.
    pub fn set_gamepad_count(&mut self, count: i32) -> Result<(), ServerError> {
        if !self.initialized {
            return Err(ServerError::NotInitialized);
        }
        if !(0..=GAMEPAD_BUFFER_COUNT as i32).contains(&count) {
            return Err(ServerError::InvalidCount(count));
        }
        for state in self.states.iter_mut().take(count as usize) {
            state.reset();
        }
        self.gamepad_count = count as usize;
        Ok(())
    }

    /// `onRumble(int, int, int)`: ignore slots outside 0..4 and queue the
    /// command. The native callback is otherwise dispatched by the manager.
    pub fn on_rumble(&mut self, slot: i32, strong: i32, weak: i32) -> Option<RumbleCommand> {
        if slot < 0 || slot as usize >= GAMEPAD_BUFFER_COUNT {
            return None;
        }
        let command = RumbleCommand {
            slot,
            strong,
            weak,
        };
        self.pending_rumble.push(command);
        Some(command)
    }

    pub fn take_rumble(&mut self) -> Vec<RumbleCommand> {
        std::mem::take(&mut self.pending_rumble)
    }

    /// Dispatch a rumble command to the devices in a slot, mirroring
    /// `GamepadServerManager.e`: each distinct descriptor is driven once.
    /// Returns the descriptors that would receive the rumble.
    pub fn rumble_targets(&self, manager: &SlotManager, slot: i32) -> Vec<String> {
        let mut seen = std::collections::HashSet::new();
        let mut targets = Vec::new();
        for device in manager.devices_in(slot) {
            let key = device.owned_descriptor();
            if seen.insert(key.clone()) {
                targets.push(key);
            }
        }
        targets
    }

    /// Device-level rumble intensity (0..=255) used by `GamepadDevice.f`:
    /// `(0,0)` cancels; otherwise strong/weak are blended 60/40.
    pub fn rumble_intensity(strong: i32, weak: i32) -> i32 {
        let strong = (strong & 0xffff) as f32 / 65535.0;
        let weak = (weak & 0xffff) as f32 / 65535.0;
        if strong <= 0.0 && weak <= 0.0 {
            return 0;
        }
        ((strong * 0.6 + weak * 0.4) * 255.0) as i32
    }

    /// `close()`: clear all state references and reset the count.
    pub fn close(&mut self) {
        for state in &mut self.states {
            state.reset();
        }
        self.initialized = false;
        self.gamepad_count = 0;
        self.pending_rumble.clear();
    }

    /// Convenience for tests: is the given slot occupied in a slot manager?
    pub fn slot_has_device(manager: &SlotManager, slot: i32) -> bool {
        manager.devices_in(slot).iter().any(|d: &GamepadDevice| d.device_id() >= 0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::super::gamepad_state::GamepadButton;

    #[test]
    fn initialize_creates_four_neutral_buffers() {
        let mut server = GamepadServerManager::new("/tmp/gamepad");
        assert!(!server.initialized);
        assert!(server.initialize());
        assert_eq!(server.states.len(), 4);
        assert!(server.states.iter().all(|s| *s == GamepadState::neutral()));
        assert_eq!(server.socket_path, "/tmp/gamepad");
    }

    #[test]
    fn count_validation_and_state_reset() {
        let mut server = GamepadServerManager::new("x");
        assert_eq!(server.set_gamepad_count(1), Err(ServerError::NotInitialized));
        server.initialize();
        server
            .apply(0, &GamepadStatePatch::default().with_button(GamepadButton::A, true))
            .unwrap();
        assert!(server.state(0).unwrap().buttons.contains(GamepadButton::A));
        assert_eq!(server.set_gamepad_count(1), Ok(()));
        assert_eq!(server.gamepad_count, 1);
        assert!(!server.state(0).unwrap().buttons.contains(GamepadButton::A));
        assert_eq!(server.set_gamepad_count(4), Ok(()));
        assert_eq!(server.set_gamepad_count(5), Err(ServerError::InvalidCount(5)));
        assert_eq!(server.set_gamepad_count(-1), Err(ServerError::InvalidCount(-1)));
    }

    #[test]
    fn apply_patch_to_invalid_slot() {
        let mut server = GamepadServerManager::new("x");
        assert_eq!(server.state(4), None);
        assert_eq!(
            server.apply(-1, &GamepadStatePatch::reset()),
            Err(ServerError::InvalidSlot(-1))
        );
        assert_eq!(
            server.apply(9, &GamepadStatePatch::reset()),
            Err(ServerError::InvalidSlot(9))
        );
    }

    #[test]
    fn rumble_queue_and_target_dedup() {
        let mut server = GamepadServerManager::new("x");
        assert_eq!(server.on_rumble(-1, 1, 2), None);
        assert_eq!(server.on_rumble(5, 1, 2), None);
        assert_eq!(server.on_rumble(1, 10, 20).unwrap().slot, 1);
        assert_eq!(server.take_rumble().len(), 1);
        assert!(server.take_rumble().is_empty());

        let mut manager = SlotManager::new();
        manager.assign(GamepadDevice::physical(1, "A", "descriptor")).unwrap();
        manager.assign(GamepadDevice::physical(2, "B", "descriptor")).unwrap();
        assert_eq!(server.rumble_targets(&manager, 0), vec!["descriptor".to_owned()]);
        assert!(server.rumble_targets(&manager, 1).is_empty());
    }

    #[test]
    fn rumble_intensity_blend() {
        assert_eq!(GamepadServerManager::rumble_intensity(0, 0), 0);
        assert_eq!(GamepadServerManager::rumble_intensity(65535, 0), 153);
        assert_eq!(GamepadServerManager::rumble_intensity(0, 65535), 102);
    }

    #[test]
    fn close_resets_server() {
        let mut server = GamepadServerManager::new("x");
        server.initialize();
        server.set_gamepad_count(2).unwrap();
        server.close();
        assert!(!server.initialized);
        assert_eq!(server.gamepad_count, 0);
        assert!(server.pending_rumble.is_empty());
    }
}
