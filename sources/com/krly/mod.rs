//! Rust port of the portable logic in `com.krly.platform` (GameHub Lite v5.1.8).
//!
//! * [`command`] — `KrCmdController` command codes, request frames and the
//!   `SendBytesNumberRsp` / ACK models from the `kr.request` / `kr.response`
//!   beans.
//! * [`firmware_ota`] — `FirmwareOtaController` sequencing, chunking and
//!   failure model.
//!
//! BLE transport (`BLEOtaManager`, listeners), Android `Context`, and RxJava
//! scheduling are intentionally not reimplemented.

pub mod command;
pub mod firmware_ota;

pub use command::{
    ready_payload, CommandAck, KrCommand, KrRequest, ReqFactory, SendBytesNumberRsp,
};
pub use firmware_ota::{
    chunk_firmware, completion_command, packet_index, FirmwareOtaError, FirmwareOtaPhase,
    FirmwareOtaSession, UpgradeEvent,
};
