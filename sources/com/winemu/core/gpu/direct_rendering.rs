//! Rust model of the direct-rendering state machine in GameHub Lite v5.1.8.
//!
//! Sources mirrored here:
//! * `sources/com/winemu/openapi/DirectRenderingMode.java` — enum `Auto(0)`,
//!   `Never(1)`, `Always(2)`.
//! * `sources/com/xj/winemu/bean/NativeRendering.java` /
//!   `NativeRenderingMode.java` — the settings-facing mode and its
//!   `trans2NativeRenderingMode` mapping.
//! * `sources/com/winemu/core/DirectRendering.java` —
//!   `onDirectRenderingStateChanged(boolean)` / companion `c(boolean)`: the
//!   native boolean is inverted for `XServer.setRenderingEnabled(z3 ^ true)`
//!   before listeners are notified.
//! * `sources/com/winemu/core/DirectRenderingStateListener.java` — the
//!   `a(boolean z3)` listener shape.
//! * `sources/com/winemu/openapi/WinUIBridge.java` — listener uses
//!   `setRenderingEnabled(!z3)`.
//! * `sources/com/winemu/core/controller/GameScopeController.java` — the 4-byte
//!   control file: version at bytes 0..2, toggle at byte 2, mode at byte 3.
//!
//! Android `SurfaceControl` / `Surface` objects, `libwinemu.so` native calls and
//! real listener registration are out of scope. The state machine keeps only
//! the decision logic and the listener event data.

/// State of direct rendering, mirroring `DirectRenderingMode` (Auto/Never/Always).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DirectRenderingState {
    /// Follow whatever the host reports as supported.
    Auto,
    /// Force software/X-server rendering.
    Never,
    /// Force direct rendering regardless of host support.
    Always,
}

impl DirectRenderingState {
    /// Java enum ordinal / `getValue()` mapping.
    pub fn from_value(value: i32) -> Self {
        match value {
            1 => DirectRenderingState::Never,
            2 => DirectRenderingState::Always,
            _ => DirectRenderingState::Auto,
        }
    }

    pub fn value(self) -> i32 {
        match self {
            DirectRenderingState::Auto => 0,
            DirectRenderingState::Never => 1,
            DirectRenderingState::Always => 2,
        }
    }

    /// Settings-facing `NativeRenderingMode` equivalent.
    pub fn from_native_mode(mode: NativeRenderingMode) -> Self {
        match mode {
            NativeRenderingMode::Auto => DirectRenderingState::Auto,
            NativeRenderingMode::Never => DirectRenderingState::Never,
            NativeRenderingMode::Always => DirectRenderingState::Always,
        }
    }

    /// Port of the `WhenMappings` switch in `NativeRendering.Companion`.
    pub fn to_native_mode(self) -> NativeRenderingMode {
        match self {
            DirectRenderingState::Auto => NativeRenderingMode::Auto,
            DirectRenderingState::Never => NativeRenderingMode::Never,
            DirectRenderingState::Always => NativeRenderingMode::Always,
        }
    }

    /// Decision rule: `Always` and `Never` override the host capability,
    /// `Auto` follows it.
    pub fn should_enable(self, host_supports_direct_rendering: bool) -> bool {
        match self {
            DirectRenderingState::Always => true,
            DirectRenderingState::Never => false,
            DirectRenderingState::Auto => host_supports_direct_rendering,
        }
    }
}

/// The three choices in `com.xj.winemu.bean.NativeRenderingMode`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum NativeRenderingMode {
    Auto,
    Never,
    Always,
}

/// Event delivered to a `DirectRenderingStateListener.a(boolean)`:
/// `direct_enabled` is the native boolean, while `xserver_rendering_enabled`
/// is the inverted value passed to `XServer.setRenderingEnabled`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DirectRenderingEvent {
    pub direct_enabled: bool,
    pub xserver_rendering_enabled: bool,
}

/// Pure state holder modeling the parts of `DirectRendering` that do not need
/// Android surfaces or native code.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DirectRenderingController {
    mode: DirectRenderingState,
    direct_enabled: bool,
}

impl DirectRenderingController {
    /// `GameScopeController` starts in `Auto` with the control byte cleared.
    pub fn new(mode: DirectRenderingState) -> Self {
        Self { mode, direct_enabled: false }
    }

    pub fn mode(&self) -> DirectRenderingState {
        self.mode
    }

    pub fn set_mode(&mut self, mode: DirectRenderingState) {
        self.mode = mode;
    }

    pub fn is_direct_enabled(&self) -> bool {
        self.direct_enabled
    }

    /// Decide whether direct rendering should be active for the given host.
    pub fn decide(&self, host_supports_direct_rendering: bool) -> bool {
        self.mode.should_enable(host_supports_direct_rendering)
    }

    /// Port of `DirectRendering.Companion.c(boolean)` /
    /// `onDirectRenderingStateChanged(boolean)`: update the state, invert it
    /// for the X server and return the listener payload.
    pub fn on_state_changed(&mut self, enabled: bool) -> DirectRenderingEvent {
        self.direct_enabled = enabled;
        DirectRenderingEvent {
            direct_enabled: enabled,
            xserver_rendering_enabled: !enabled, // `z3 ^ true`
        }
    }

    /// The 4-byte `GameScopeController` control block: version (u16 LE) at
    /// bytes 0..2, direct-enabled toggle at byte 2, mode at byte 3.
    pub fn control_bytes(&self, version: u16) -> [u8; 4] {
        [
            (version & 0xFF) as u8,
            ((version >> 8) & 0xFF) as u8,
            self.direct_enabled as u8,
            self.mode.value() as u8,
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mode_values_match_the_java_enum() {
        assert_eq!(DirectRenderingState::from_value(0), DirectRenderingState::Auto);
        assert_eq!(DirectRenderingState::from_value(1), DirectRenderingState::Never);
        assert_eq!(DirectRenderingState::from_value(2), DirectRenderingState::Always);
        assert_eq!(DirectRenderingState::from_value(99), DirectRenderingState::Auto);
        assert_eq!(DirectRenderingState::Always.value(), 2);
    }

    #[test]
    fn native_mode_translation_is_bijective() {
        let all = [
            NativeRenderingMode::Auto,
            NativeRenderingMode::Never,
            NativeRenderingMode::Always,
        ];
        for mode in all {
            assert_eq!(
                DirectRenderingState::from_native_mode(mode).to_native_mode(),
                mode
            );
        }
    }

    #[test]
    fn decision_logic_covers_all_three_modes() {
        assert!(!DirectRenderingState::Never.should_enable(true));
        assert!(DirectRenderingState::Always.should_enable(false));
        assert!(DirectRenderingState::Auto.should_enable(true));
        assert!(!DirectRenderingState::Auto.should_enable(false));
    }

    #[test]
    fn state_change_inverts_xserver_rendering() {
        let mut controller = DirectRenderingController::new(DirectRenderingState::Auto);
        assert_eq!(controller.mode(), DirectRenderingState::Auto);
        assert!(!controller.is_direct_enabled());

        let on = controller.on_state_changed(true);
        assert!(on.direct_enabled);
        assert!(!on.xserver_rendering_enabled); // `z3 ^ true`
        assert!(controller.is_direct_enabled());

        let off = controller.on_state_changed(false);
        assert!(!off.direct_enabled);
        assert!(off.xserver_rendering_enabled);
    }

    #[test]
    fn mode_change_then_decide() {
        let mut controller = DirectRenderingController::new(DirectRenderingState::Auto);
        assert!(!controller.decide(false));
        controller.set_mode(DirectRenderingState::Always);
        assert!(controller.decide(false));
        controller.set_mode(DirectRenderingState::Never);
        assert!(!controller.decide(true));
    }

    #[test]
    fn gamescope_control_bytes_layout() {
        let mut controller = DirectRenderingController::new(DirectRenderingState::Always);
        controller.on_state_changed(true);
        assert_eq!(controller.control_bytes(0), [0, 0, 1, 2]);
        assert_eq!(controller.control_bytes(0x0102), [0x02, 0x01, 1, 2]);
    }
}
