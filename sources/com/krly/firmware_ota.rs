//! Portable firmware-OTA session logic of
//! `com.krly.platform.controller.FirmwareOtaController`.
//!
//! The Java controller drives an external BLE player through
//! `KrCmdController` + `BLEOtaManager`. What is portable is the *sequencing*
//! and the *chunking*:
//!
//! 1. `z(...)` registers callbacks and connects.
//! 2. `OtaMessageCallback.c()` -> `w(firmware)` -> `A(firmware)` sends the
//!    `DeviceOTAReady` command with CRC16 and total length.
//! 3. On ACK, `SendBytesNumberReq`/`SendBytesNumberRsp.a()` yields
//!    `bytesPerPacket`; `x(firmware, bytesPerPacket)` splits the image into
//!    chunks; `v(0)` sends the first chunk.
//! 4. Each progress notification reports bytes written; the controller maps
//!    it to a packet index with `ceil(written / bytesPerPacket)` and calls
//!    `v(index)`. A repeated index retries the same chunk.
//! 5. When all chunks are sent, the "complete" command is issued; its ACK
//!    reports success, any other path reports a failure string via `o(...)`.
//!
//! All BLE I/O, threads and `Thread.sleep(100)` retries are excluded; the
//! session only records state and emits [`UpgradeEvent`]s.

use super::command::{CommandAck, KrCommand, KrRequest, ReqFactory, SendBytesNumberRsp};

/// Failure reasons from `FirmwareOtaController.o(String)` and its callbacks.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum FirmwareOtaError {
    /// `OtaMessageCallback.a(int)` -> `"error code N"`.
    ErrorCode(i32),
    /// `p(...)` -> "完成命令失败" (complete command failed).
    CompleteFailed,
    /// `q(...)` -> "固件长时间无反馈" (no firmware feedback).
    NoFeedback,
    /// `r(-1)` -> "蓝牙断连" (Bluetooth disconnected).
    BleDisconnected,
    /// `t(...)` -> "限制长度错误" (invalid length limit).
    LengthLimit,
    /// `u(...)` -> "crc错误" (CRC error).
    CrcError,
    /// `-2` -> "notify 异常".
    NotifyError,
    /// `-3` -> "写入数据 异常".
    WriteError,
    /// other OTA callback code -> "交互异常".
    InteractionError,
    /// "获取字节数异常" (packet limit was never received).
    NoPacketLimit,
}

impl FirmwareOtaError {
    pub fn message(&self) -> String {
        match self {
            FirmwareOtaError::ErrorCode(code) => format!("error code {}", code),
            FirmwareOtaError::CompleteFailed => "complete command failed".into(),
            FirmwareOtaError::NoFeedback => "firmware did not respond".into(),
            FirmwareOtaError::BleDisconnected => "bluetooth disconnected".into(),
            FirmwareOtaError::LengthLimit => "invalid length limit".into(),
            FirmwareOtaError::CrcError => "crc error".into(),
            FirmwareOtaError::NotifyError => "notify error".into(),
            FirmwareOtaError::WriteError => "write error".into(),
            FirmwareOtaError::InteractionError => "interaction error".into(),
            FirmwareOtaError::NoPacketLimit => "packet byte count unavailable".into(),
        }
    }

    /// Map a raw OTA callback code using the `z(...)` callback rules.
    pub fn from_callback_code(code: i32) -> Self {
        match code {
            -1 => FirmwareOtaError::BleDisconnected,
            -2 => FirmwareOtaError::NotifyError,
            -3 => FirmwareOtaError::WriteError,
            other => FirmwareOtaError::ErrorCode(other),
        }
    }
}

impl std::fmt::Display for FirmwareOtaError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.message())
    }
}

/// `UpgradeCallBack` callbacks reduced to data.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum UpgradeEvent {
    /// `a()` — upgrade started.
    Started,
    /// `b(int current, int total)` — progress (current/total bytes or packets).
    Progress { current: i32, total: i32 },
    /// `c()` is used for failures in the controller, success is signalled by
    /// `a()` after the complete ACK; this variant keeps both explicit.
    Completed,
    /// Failure reported through `c()` / `o(...)`.
    Failed(FirmwareOtaError),
}

/// `FirmwareOtaController.x(byte[], int)`: split the firmware image into
/// chunks of at most `chunk_size` bytes.
pub fn chunk_firmware(firmware: &[u8], chunk_size: usize) -> Vec<Vec<u8>> {
    if firmware.is_empty() || chunk_size == 0 {
        return Vec::new(); // the Java code would loop forever for size 0
    }
    firmware.chunks(chunk_size).map(|c| c.to_vec()).collect()
}

/// `v(int)` / `b(int)`: ceil division mapping bytes written to packet index.
pub const fn packet_index(bytes_written: u32, bytes_per_packet: u32) -> u32 {
    if bytes_per_packet == 0 {
        return 0;
    }
    let remainder = bytes_written % bytes_per_packet;
    bytes_written / bytes_per_packet + if remainder == 0 { 0 } else { 1 }
}

/// Phase of a portable OTA session.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FirmwareOtaPhase {
    /// Before `z(...)`.
    Idle,
    /// Firmware ready command sent, awaiting its ACK.
    AwaitingReadyAck,
    /// Awaiting the packet-size response.
    AwaitingPacketLimit,
    /// Sending chunk `index` (0-based).
    Sending { index: usize },
    /// All chunks sent, awaiting completion ACK.
    Completing,
    /// Session finished successfully.
    Completed,
    /// Session aborted.
    Failed(FirmwareOtaError),
}

/// Stateful, I/O-free firmware upgrade session.
#[derive(Debug, Clone)]
pub struct FirmwareOtaSession {
    pub phase: FirmwareOtaPhase,
    firmware: Vec<u8>,
    chunks: Vec<Vec<u8>>,
    bytes_per_packet: usize,
    pub total_bytes: i32,
}

impl Default for FirmwareOtaSession {
    fn default() -> Self {
        Self::new()
    }
}

impl FirmwareOtaSession {
    pub fn new() -> Self {
        Self {
            phase: FirmwareOtaPhase::Idle,
            firmware: Vec::new(),
            chunks: Vec::new(),
            bytes_per_packet: 0,
            total_bytes: 0,
        }
    }

    /// `z(...)` + `OtaMessageCallback.c()` -> `w(bArr)`: store the image and
    /// create the `DeviceOTAReady` request (CRC + length).
    pub fn begin(&mut self, firmware: &[u8]) -> KrRequest {
        self.firmware = firmware.to_vec();
        self.total_bytes = firmware.len() as i32;
        self.chunks.clear();
        self.bytes_per_packet = 0;
        self.phase = FirmwareOtaPhase::AwaitingReadyAck;
        let crc = crc16_local(&self.firmware);
        ReqFactory::device_ota_ready(crc, firmware.len())
    }

    /// ACK of the ready command: on success issue the packet-limit query.
    pub fn on_ready_ack(&mut self, ack: CommandAck) -> Result<KrRequest, FirmwareOtaError> {
        if !ack.is_ok() {
            let err = FirmwareOtaError::CrcError;
            self.phase = FirmwareOtaPhase::Failed(err);
            return Err(err);
        }
        self.phase = FirmwareOtaPhase::AwaitingPacketLimit;
        Ok(ReqFactory::query_send_bytes_number())
    }

    /// `t(...)` -> `x(bArr, limit)`: validate the limit, split the image and
    /// start with the first chunk (`v(0)`).
    pub fn on_packet_limit(
        &mut self,
        rsp: SendBytesNumberRsp,
    ) -> Result<UpgradeEvent, FirmwareOtaError> {
        if !rsp.is_valid() {
            let err = FirmwareOtaError::LengthLimit;
            self.phase = FirmwareOtaPhase::Failed(err);
            return Err(err);
        }
        self.bytes_per_packet = rsp.max_bytes as usize;
        self.chunks = chunk_firmware(&self.firmware, self.bytes_per_packet);
        if self.chunks.is_empty() {
            let err = FirmwareOtaError::NoPacketLimit;
            self.phase = FirmwareOtaPhase::Failed(err);
            return Err(err);
        }
        self.phase = FirmwareOtaPhase::Sending { index: 0 };
        Ok(UpgradeEvent::Progress {
            current: 0,
            total: self.chunks.len() as i32,
        })
    }

    /// Bytes-per-packet reported by the session (0 before the limit arrives).
    pub fn bytes_per_packet(&self) -> usize {
        self.bytes_per_packet
    }

    /// Number of chunks produced by `x(...)`.
    pub fn chunk_count(&self) -> usize {
        self.chunks.len()
    }

    /// `BLEOtaManager.Q(chunk[index])` payload for the current packet.
    pub fn current_chunk(&self) -> Option<(usize, &[u8])> {
        match self.phase {
            FirmwareOtaPhase::Sending { index } => self
                .chunks
                .get(index)
                .map(|chunk| (index, chunk.as_slice())),
            _ => None,
        }
    }

    /// `v(int index)`: move to a specific packet index. Out-of-range indices
    /// trigger the completion command; a repeated index re-sends the chunk.
    pub fn send_packet(&mut self, index: usize) -> Option<KrRequest> {
        if index >= self.chunks.len() {
            self.phase = FirmwareOtaPhase::Completing;
            return Some(ReqFactory::device_ota_complete());
        }
        self.phase = FirmwareOtaPhase::Sending { index };
        None
    }

    /// `OtaMessageCallback.b(int)` — map bytes written to a packet index and
    /// advance the session, exactly as `v(ceil(written / bytesPerPacket))`.
    pub fn on_progress(&mut self, bytes_written: u32) -> Option<KrRequest> {
        if self.bytes_per_packet == 0 {
            return None;
        }
        let index = packet_index(bytes_written, self.bytes_per_packet as u32) as usize;
        self.send_packet(index)
    }

    /// `p(i3, rsp)` — completion ACK. `i3 == 0` -> success, else failure.
    pub fn on_complete_ack(&mut self, ack: CommandAck) -> UpgradeEvent {
        if ack.is_ok() {
            self.phase = FirmwareOtaPhase::Completed;
            UpgradeEvent::Completed
        } else {
            self.phase = FirmwareOtaPhase::Failed(FirmwareOtaError::CompleteFailed);
            UpgradeEvent::Failed(FirmwareOtaError::CompleteFailed)
        }
    }

    /// `o(String)` — abort with a reason.
    pub fn fail(&mut self, err: FirmwareOtaError) -> UpgradeEvent {
        self.phase = FirmwareOtaPhase::Failed(err);
        UpgradeEvent::Failed(err)
    }

    pub fn is_finished(&self) -> bool {
        matches!(
            self.phase,
            FirmwareOtaPhase::Completed | FirmwareOtaPhase::Failed(_)
        )
    }
}

/// Local CRC helper (same algorithm as
/// `com.xj.devicemanagement.byte_utils::crc16_ccitt`) kept private so the
/// `krly` area stays self-contained.
fn crc16_local(bytes: &[u8]) -> u16 {
    let mut crc: u32 = 0;
    for &b in bytes {
        for bit in 0..8 {
            let data_bit = (b >> (7 - bit)) & 1 == 1;
            let crc_msb = (crc >> 15) & 1 == 1;
            crc <<= 1;
            if data_bit ^ crc_msb {
                crc ^= 0x1021;
            }
        }
    }
    (crc & 0xffff) as u16
}

/// Assert the command used for "all chunks sent" (documentation helper for
/// callers that drive [`FirmwareOtaSession`] manually).
pub fn completion_command() -> KrRequest {
    KrRequest::new(KrCommand::DeviceOtaComplete, Vec::new())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn chunking_splits_image_and_tracks_remainder() {
        let data: Vec<u8> = (0..10).collect();
        let chunks = chunk_firmware(&data, 4);
        assert_eq!(chunks.len(), 3);
        assert_eq!(chunks[0], vec![0, 1, 2, 3]);
        assert_eq!(chunks[1], vec![4, 5, 6, 7]);
        assert_eq!(chunks[2], vec![8, 9]);
        assert!(chunk_firmware(&[], 4).is_empty());
        assert!(chunk_firmware(&data, 0).is_empty());
    }

    #[test]
    fn packet_index_is_ceiling_division() {
        assert_eq!(packet_index(0, 4), 0);
        assert_eq!(packet_index(1, 4), 1);
        assert_eq!(packet_index(4, 4), 1);
        assert_eq!(packet_index(5, 4), 2);
        assert_eq!(packet_index(8, 4), 2);
        assert_eq!(packet_index(5, 0), 0);
    }

    #[test]
    fn errors_map_from_callback_codes() {
        assert_eq!(
            FirmwareOtaError::from_callback_code(-1),
            FirmwareOtaError::BleDisconnected
        );
        assert_eq!(
            FirmwareOtaError::from_callback_code(-2),
            FirmwareOtaError::NotifyError
        );
        assert_eq!(
            FirmwareOtaError::from_callback_code(-3),
            FirmwareOtaError::WriteError
        );
        assert_eq!(
            FirmwareOtaError::from_callback_code(100),
            FirmwareOtaError::ErrorCode(100)
        );
        assert!(FirmwareOtaError::CrcError.to_string().contains("crc"));
    }

    #[test]
    fn happy_path_session_ready_limit_chunks_complete() {
        let mut session = FirmwareOtaSession::new();
        let firmware: Vec<u8> = (0..6).collect();

        let ready = session.begin(&firmware);
        assert_eq!(ready.command, KrCommand::DeviceOtaReady);
        assert_eq!(ready.payload.len(), 5);
        assert_eq!(session.phase, FirmwareOtaPhase::AwaitingReadyAck);
        assert_eq!(session.total_bytes, 6);

        let limit_req = session.on_ready_ack(CommandAck::OK).unwrap();
        assert_eq!(limit_req.command, KrCommand::QuerySendBytesNumber);

        let progress = session
            .on_packet_limit(SendBytesNumberRsp { max_bytes: 4 })
            .unwrap();
        assert_eq!(
            progress,
            UpgradeEvent::Progress {
                current: 0,
                total: 2
            }
        );
        assert_eq!(session.bytes_per_packet(), 4);
        assert_eq!(session.chunk_count(), 2);
        assert_eq!(session.current_chunk().unwrap().1, &[0, 1, 2, 3]);

        // Progress of 1 byte maps to packet 1 (second chunk).
        assert_eq!(session.on_progress(1), None);
        assert_eq!(session.current_chunk().unwrap().1, &[4, 5]);

        // Progress beyond the image triggers the complete command.
        let complete = session.on_progress(6).unwrap();
        assert_eq!(complete.command, KrCommand::DeviceOtaComplete);
        assert_eq!(session.phase, FirmwareOtaPhase::Completing);

        assert_eq!(session.on_complete_ack(CommandAck::OK), UpgradeEvent::Completed);
        assert!(session.is_finished());
        assert_eq!(session.phase, FirmwareOtaPhase::Completed);
    }

    #[test]
    fn ready_ack_failure_records_crc_error() {
        let mut session = FirmwareOtaSession::new();
        session.begin(&[1, 2, 3]);
        let err = session.on_ready_ack(CommandAck::from_code(1)).unwrap_err();
        assert_eq!(err, FirmwareOtaError::CrcError);
        assert_eq!(session.phase, FirmwareOtaPhase::Failed(FirmwareOtaError::CrcError));
        assert!(session.is_finished());
    }

    #[test]
    fn invalid_packet_limit_fails() {
        let mut session = FirmwareOtaSession::new();
        session.begin(&[1, 2, 3]);
        session.on_ready_ack(CommandAck::OK).unwrap();
        assert_eq!(
            session.on_packet_limit(SendBytesNumberRsp::default()).unwrap_err(),
            FirmwareOtaError::LengthLimit
        );
        assert_eq!(session.phase, FirmwareOtaPhase::Failed(FirmwareOtaError::LengthLimit));
    }

    #[test]
    fn empty_firmware_has_no_packet_limit() {
        let mut session = FirmwareOtaSession::new();
        session.begin(&[]);
        session.on_ready_ack(CommandAck::OK).unwrap();
        let err = session
            .on_packet_limit(SendBytesNumberRsp { max_bytes: 4 })
            .unwrap_err();
        assert_eq!(err, FirmwareOtaError::NoPacketLimit);
    }

    #[test]
    fn completion_failure_and_manual_fail() {
        let mut session = FirmwareOtaSession::new();
        session.begin(&[1, 2]);
        session.on_ready_ack(CommandAck::OK).unwrap();
        session.on_packet_limit(SendBytesNumberRsp { max_bytes: 2 }).unwrap();
        assert_eq!(
            session.on_complete_ack(CommandAck::from_code(-1)),
            UpgradeEvent::Failed(FirmwareOtaError::CompleteFailed)
        );
        assert!(session.is_finished());

        let mut s2 = FirmwareOtaSession::new();
        assert_eq!(
            s2.fail(FirmwareOtaError::NoFeedback),
            UpgradeEvent::Failed(FirmwareOtaError::NoFeedback)
        );
        assert_eq!(s2.phase, FirmwareOtaPhase::Failed(FirmwareOtaError::NoFeedback));
    }

    #[test]
    fn explicit_completion_command_matches_enum() {
        let req = completion_command();
        assert_eq!(req.command, KrCommand::DeviceOtaComplete);
        assert!(req.payload.is_empty());
    }
}
