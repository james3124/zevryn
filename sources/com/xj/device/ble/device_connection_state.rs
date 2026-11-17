//! Connection lifecycle state machine for a GameSir controller.
//!
//! The reference SDK does not expose a named state enum: connection is reported
//! through callbacks such as `IConnectStateListener.onConnectState(boolean)` and
//! `DeviceConnectionMonitor` (which listens for
//! `android.bluetooth.device.action.ACL_DISCONNECTED`), while calibration lives in
//! `com.xj.devicesetting.gamepad.GamePadCalibrationActivity`. This module models
//! that lifecycle as an explicit state machine, using the transition sequence
//! specified for this rebuild:
//!
//! ```text
//! Disconnected -> Scanning -> Connecting -> Connected -> Calibrating -> Disconnected
//! ```
//!
//! `Disconnect` is additionally accepted from every state (covers a cancelled
//! scan/connect and an unexpected `ACL_DISCONNECTED` broadcast). No transport
//! (BLE/USB) code is included.

/// The connection lifecycle states.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DeviceConnectionState {
    Disconnected,
    Scanning,
    Connecting,
    Connected,
    Calibrating,
}

/// Events that drive [`DeviceConnectionState`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ConnectionEvent {
    /// Begin scanning for a controller.
    StartScan,
    /// A matching device was found; attempt to connect.
    DeviceFound,
    /// The link was established.
    LinkEstablished,
    /// Enter the calibration screen/flow.
    BeginCalibration,
    /// Disconnect, cancel, or an ACL_DISCONNECTED notification.
    Disconnect,
}

/// Error returned when an event is not valid for the current state.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct InvalidTransition {
    pub from: DeviceConnectionState,
    pub event: ConnectionEvent,
}

impl DeviceConnectionState {
    pub const fn initial() -> Self {
        Self::Disconnected
    }

    /// Apply `event`, returning the next state or an error if the event is not
    /// valid in the current state.
    pub fn transition(self, event: ConnectionEvent) -> Result<Self, InvalidTransition> {
        use ConnectionEvent as E;
        use DeviceConnectionState as S;
        let next = match (self, event) {
            (S::Disconnected, E::StartScan) => S::Scanning,
            (S::Scanning, E::DeviceFound) => S::Connecting,
            (S::Connecting, E::LinkEstablished) => S::Connected,
            (S::Connected, E::BeginCalibration) => S::Calibrating,
            (_, E::Disconnect) => S::Disconnected,
            (from, event) => return Err(InvalidTransition { from, event }),
        };
        Ok(next)
    }

    pub fn can_transition(self, event: ConnectionEvent) -> bool {
        self.transition(event).is_ok()
    }

    pub const fn is_connected(self) -> bool {
        matches!(self, Self::Connected | Self::Calibrating)
    }

    /// True while a scan or connection attempt is in progress.
    pub const fn is_busy(self) -> bool {
        matches!(self, Self::Scanning | Self::Connecting)
    }

    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Disconnected => "disconnected",
            Self::Scanning => "scanning",
            Self::Connecting => "connecting",
            Self::Connected => "connected",
            Self::Calibrating => "calibrating",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn run(events: &[ConnectionEvent]) -> Result<DeviceConnectionState, InvalidTransition> {
        let mut state = DeviceConnectionState::initial();
        for &event in events {
            state = state.transition(event)?;
        }
        Ok(state)
    }

    #[test]
    fn happy_path_cycle() {
        use ConnectionEvent as E;
        let states = [
            DeviceConnectionState::Disconnected,
            DeviceConnectionState::Scanning,
            DeviceConnectionState::Connecting,
            DeviceConnectionState::Connected,
            DeviceConnectionState::Calibrating,
            DeviceConnectionState::Disconnected,
        ];
        let events = [E::StartScan, E::DeviceFound, E::LinkEstablished, E::BeginCalibration, E::Disconnect];
        let mut state = DeviceConnectionState::initial();
        for (i, &event) in events.iter().enumerate() {
            assert_eq!(state, states[i]);
            state = state.transition(event).unwrap();
        }
        assert_eq!(state, states[states.len() - 1]);
    }

    #[test]
    fn disconnect_is_valid_from_every_state() {
        for state in [
            DeviceConnectionState::Disconnected,
            DeviceConnectionState::Scanning,
            DeviceConnectionState::Connecting,
            DeviceConnectionState::Connected,
            DeviceConnectionState::Calibrating,
        ] {
            assert_eq!(state.transition(ConnectionEvent::Disconnect), Ok(DeviceConnectionState::Disconnected));
        }
    }

    #[test]
    fn rejects_out_of_order_events() {
        use ConnectionEvent as E;
        // Cannot connect without first finding a device.
        assert_eq!(
            DeviceConnectionState::initial().transition(E::DeviceFound),
            Err(InvalidTransition { from: DeviceConnectionState::Disconnected, event: E::DeviceFound })
        );
        // Cannot calibrate while scanning.
        assert!(!DeviceConnectionState::Scanning.can_transition(E::BeginCalibration));
        // Full sequence must fail if a step is skipped.
        assert!(run(&[E::StartScan, E::LinkEstablished]).is_err());
    }

    #[test]
    fn status_helpers() {
        assert!(DeviceConnectionState::Connected.is_connected());
        assert!(DeviceConnectionState::Calibrating.is_connected());
        assert!(!DeviceConnectionState::Scanning.is_connected());
        assert!(DeviceConnectionState::Scanning.is_busy());
        assert!(DeviceConnectionState::Connecting.is_busy());
        assert_eq!(DeviceConnectionState::Calibrating.as_str(), "calibrating");
    }
}
