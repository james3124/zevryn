//! Portable command model of `com.krly.platform.controller.KrCmdController`
//! and the `com.krly.platform.kr.request` beans (GameHub Lite v5.1.8).
//!
//! Command codes are taken from the `BaseReq(int)` constructors:
//! * `DeviceOTAReadyReq` -> `super(80)` with a 5-byte payload (`l(int,int)`).
//! * `SendBytesNumberReq` -> `super(81)`; response carries the packet limit.
//! * `SetDataSendReq` -> `super(82)`.
//! * `DeviceOTACompleteReq` -> `super(83)` with an empty payload.
//!
//! `DeviceOTAVerifyReq` and `DonglePairingReq` have **no constructor and no
//! code** in the decompiled sources (they are never created by `ReqFactory`),
//! so no numeric code is invented for them here.
//!
//! BLE writes, notifications, callbacks and RxJava scheduling are excluded.

/// Protocol commands emitted by `KrCmdController`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u16)]
pub enum KrCommand {
    /// `DeviceOTAReadyReq` — firmware size + CRC handshake.
    DeviceOtaReady = 80,
    /// `SendBytesNumberReq` — query maximum bytes per packet.
    QuerySendBytesNumber = 81,
    /// `SetDataSendReq` — configure data transmission.
    SetDataSend = 82,
    /// `DeviceOTACompleteReq` — finish the OTA session.
    DeviceOtaComplete = 83,
}

impl KrCommand {
    pub const ALL: [KrCommand; 4] = [
        KrCommand::DeviceOtaReady,
        KrCommand::QuerySendBytesNumber,
        KrCommand::SetDataSend,
        KrCommand::DeviceOtaComplete,
    ];

    pub const fn code(self) -> u16 {
        self as u16
    }

    pub const fn name(self) -> &'static str {
        match self {
            KrCommand::DeviceOtaReady => "DeviceOTAReady",
            KrCommand::QuerySendBytesNumber => "SendBytesNumber",
            KrCommand::SetDataSend => "SetDataSend",
            KrCommand::DeviceOtaComplete => "DeviceOTAComplete",
        }
    }

    /// Reverse lookup used when parsing a frame's command byte.
    pub const fn from_code(code: u16) -> Option<Self> {
        match code {
            80 => Some(KrCommand::DeviceOtaReady),
            81 => Some(KrCommand::QuerySendBytesNumber),
            82 => Some(KrCommand::SetDataSend),
            83 => Some(KrCommand::DeviceOtaComplete),
            _ => None,
        }
    }

    /// `KrCmdController` calls these three commands with a response callback.
    pub const fn expects_response(self) -> bool {
        matches!(
            self,
            KrCommand::DeviceOtaReady
                | KrCommand::QuerySendBytesNumber
                | KrCommand::DeviceOtaComplete
        )
    }
}

/// A request frame: command plus payload.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KrRequest {
    pub command: KrCommand,
    pub payload: Vec<u8>,
}

impl KrRequest {
    pub fn new(command: KrCommand, payload: impl Into<Vec<u8>>) -> Self {
        Self {
            command,
            payload: payload.into(),
        }
    }

    /// `BaseReq.c()`: `[len, cmd_lo, cmd_hi, payload...]` where `len` is the
    /// total frame length (`payload.len() + 3`) truncated to one byte.
    pub fn encode(&self) -> Vec<u8> {
        let len = (self.payload.len() + 3) & 0xff;
        let code = self.command.code();
        let mut out = Vec::with_capacity(self.payload.len() + 3);
        out.push(len as u8);
        out.push((code & 0xff) as u8);
        out.push(((code >> 8) & 0xff) as u8);
        out.extend_from_slice(&self.payload);
        out
    }

    /// Parse a frame produced by [`KrRequest::encode`].
    pub fn decode(frame: &[u8]) -> Option<Self> {
        if frame.len() < 3 || frame[0] as usize != frame.len() {
            return None;
        }
        let code = u16::from_le_bytes([frame[1], frame[2]]);
        Some(Self {
            command: KrCommand::from_code(code)?,
            payload: frame[3..].to_vec(),
        })
    }
}

/// `DeviceOTAReadyReq.l(int crc, int length)` payload:
/// `[crc_hi, crc_lo, len_b2, len_b1, len_b0]` (big-endian CRC16, 24-bit length).
pub fn ready_payload(crc: u16, length: usize) -> [u8; 5] {
    let len = length as u32 & 0x00ff_ffff;
    [
        (crc >> 8) as u8,
        crc as u8,
        ((len >> 16) & 0xff) as u8,
        ((len >> 8) & 0xff) as u8,
        (len & 0xff) as u8,
    ]
}

/// Convenience builders matching `ReqFactory` output.
pub struct ReqFactory;

impl ReqFactory {
    pub fn device_ota_ready(crc: u16, length: usize) -> KrRequest {
        KrRequest::new(KrCommand::DeviceOtaReady, ready_payload(crc, length).to_vec())
    }

    pub fn query_send_bytes_number() -> KrRequest {
        KrRequest::new(KrCommand::QuerySendBytesNumber, Vec::new())
    }

    pub fn set_data_send() -> KrRequest {
        KrRequest::new(KrCommand::SetDataSend, Vec::new())
    }

    pub fn device_ota_complete() -> KrRequest {
        KrRequest::new(KrCommand::DeviceOtaComplete, Vec::new())
    }
}

/// `SendBytesNumberRsp`: the maximum number of bytes per packet. `-1` is the
/// Java field initializer, meaning "not received yet".
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SendBytesNumberRsp {
    pub max_bytes: i32,
}

impl Default for SendBytesNumberRsp {
    fn default() -> Self {
        Self { max_bytes: -1 }
    }
}

impl SendBytesNumberRsp {
    /// `SendBytesNumberReq.f(byte[])`: `bArr[4] | (bArr[3] << 8)`.
    pub fn parse(frame: &[u8]) -> Option<Self> {
        if frame.len() < 5 {
            return None;
        }
        Some(Self {
            max_bytes: (frame[4] as i32) | ((frame[3] as i32) << 8),
        })
    }

    pub fn is_valid(self) -> bool {
        self.max_bytes > 0
    }
}

/// `OnMessageCallback` result: `KrCmdController` compares the leading int to
/// zero (`i3 != 0` => failure).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CommandAck {
    pub code: i32,
}

impl CommandAck {
    pub const OK: Self = Self { code: 0 };

    pub const fn is_ok(self) -> bool {
        self.code == 0
    }

    pub fn from_code(code: i32) -> Self {
        Self { code }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn command_codes_match_base_req_constructors() {
        assert_eq!(KrCommand::DeviceOtaReady.code(), 80);
        assert_eq!(KrCommand::QuerySendBytesNumber.code(), 81);
        assert_eq!(KrCommand::SetDataSend.code(), 82);
        assert_eq!(KrCommand::DeviceOtaComplete.code(), 83);
        assert_eq!(KrCommand::from_code(81), Some(KrCommand::QuerySendBytesNumber));
        assert_eq!(KrCommand::from_code(99), None);
        for c in KrCommand::ALL {
            assert!(!c.name().is_empty());
        }
    }

    #[test]
    fn frame_encoding_follows_base_req_c() {
        let req = KrRequest::new(KrCommand::DeviceOtaReady, vec![0x01, 0x02]);
        assert_eq!(req.encode(), vec![5, 80, 0, 0x01, 0x02]);
        let decoded = KrRequest::decode(&req.encode()).unwrap();
        assert_eq!(decoded, req);
    }

    #[test]
    fn empty_payload_frames() {
        let frame = ReqFactory::query_send_bytes_number();
        assert_eq!(frame.command, KrCommand::QuerySendBytesNumber);
        for cmd in KrCommand::ALL {
            let req = KrRequest::new(cmd, Vec::new());
            assert_eq!(req.encode(), vec![3, cmd.code() as u8, 0]);
            assert_eq!(KrRequest::decode(&req.encode()).unwrap(), req);
        }
    }

    #[test]
    fn decode_rejects_bad_frames() {
        assert_eq!(KrRequest::decode(&[]), None);
        assert_eq!(KrRequest::decode(&[2, 80, 0]), None); // length mismatch
        assert_eq!(KrRequest::decode(&[3, 99, 0]), None); // unknown command
    }

    #[test]
    fn ready_payload_matches_device_ota_ready_req_l() {
        // crc = 0x31C3, length = 0x010203 (3 bytes).
        assert_eq!(
            ready_payload(0x31c3, 0x010203),
            [0x31, 0xc3, 0x01, 0x02, 0x03]
        );
        let req = ReqFactory::device_ota_ready(0x1234, 0x0102);
        assert_eq!(req.payload, [0x12, 0x34, 0x00, 0x01, 0x02]);
        assert_eq!(req.encode()[0], 8, "3 header bytes + 5 payload bytes");
    }

    #[test]
    fn send_bytes_number_response_parsing() {
        assert_eq!(SendBytesNumberRsp::default().max_bytes, -1);
        let rsp = SendBytesNumberRsp::parse(&[0, 0, 0, 0x01, 0x00]).unwrap();
        assert_eq!(rsp.max_bytes, 256);
        assert!(rsp.is_valid());
        assert_eq!(SendBytesNumberRsp::parse(&[0, 0]), None);
        assert!(!SendBytesNumberRsp::default().is_valid());
    }

    #[test]
    fn command_ack_zero_is_success() {
        assert!(CommandAck::OK.is_ok());
        assert!(CommandAck::from_code(0).is_ok());
        assert!(!CommandAck::from_code(-1).is_ok());
    }
}
