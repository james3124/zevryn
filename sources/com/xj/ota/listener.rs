//! Upgrade listener event model.
//!
//! `com.xj.ota.set.IUpListener` defines five callbacks:
//! `a(float)` and `e(float)` — progress, `b(int)` — error code,
//! `c()` — finished/cancelled, `d(int)` — device-specific status.
//! The per-device OTA classes report numeric codes through `b(int)`.

/// Codes observed in the reference OTA implementations.
pub mod codes {
    /// `BaseOTA`: null update info / checksum / download / verify / install failure.
    pub const BASE_FAILURE: i32 = 7;
    /// `BaseOTA.i()`: unzip source missing.
    pub const UP_FILE_NOT_FOUND: i32 = 3;
    /// `T4nLiteOTA`, `X3ProOTA`, `JieLiSingleFirmwareDeviceOTA`.
    pub const DEVICE_FAILURE: i32 = 110;
    /// `G8TypeCOTA` device-specific codes.
    pub const G8_FAILURE_A: i32 = 711;
    pub const G8_FAILURE_B: i32 = 79;
    pub const G8_FAILURE_C: i32 = 71;
    /// `TsExtractor.TS_STREAM_TYPE_AIT` passed to `d(int)` by several classes.
    pub const STATUS_AIT: i32 = 0x5;
}

/// A single upgrade-listener callback, reduced to data.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum UpgradeEvent {
    /// `a(float)` / `e(float)` progress percentage.
    Progress(f32),
    /// `b(int)` error code.
    Error(i32),
    /// `c()` completion.
    Finished,
    /// `d(int)` device-specific status code.
    Status(i32),
}

impl UpgradeEvent {
    pub fn is_error(&self) -> bool {
        matches!(self, UpgradeEvent::Error(_))
    }

    /// Error event for the generic OTA failure code.
    pub fn base_failure() -> Self {
        UpgradeEvent::Error(codes::BASE_FAILURE)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn upgrade_event_classification() {
        assert!(UpgradeEvent::base_failure().is_error());
        assert_eq!(UpgradeEvent::base_failure(), UpgradeEvent::Error(7));
        assert!(!UpgradeEvent::Progress(50.0).is_error());
        assert!(!UpgradeEvent::Finished.is_error());
        assert_eq!(UpgradeEvent::Status(codes::STATUS_AIT), UpgradeEvent::Status(5));
    }
}
