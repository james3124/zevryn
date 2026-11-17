//! A generic "code + payload bytes" server message model.
//!
//! FRAMING NOTE (important):
//! ------------------------
//! The decompiled request handlers do **not** define a length-prefixed frame
//! around each message. The on-wire request shape observed in
//! `*RequestHandler.java` is:
//!
//! ```text
//! [ 1-byte request code ][ subsystem-specific payload ... ]
//! ```
//!
//! For example `ALSARequestHandler.a(Client)` first checks that at least
//! `MIN_BUFFERED_BYTES` (5) bytes are available, reads the code byte with
//! `SocketInputStream.e()`, then reads a 4-byte int length with `h()` in the
//! cases that need one. `PerfRequestHandler`, `SysVSHMRequestHandler`,
//! `RenderDocRequestHandler` and `WMRequestHandler` follow the same
//! "code byte first" convention. No encompassing length prefix is present.
//!
//! Consequently this module models **the code byte and the raw payload only**;
//! it deliberately does not implement a synthetic length-prefixed framing that
//! the source does not use. Real socket I/O is out of scope.

/// Minimum number of buffered bytes the handlers require before parsing a
/// request (`socketInputStream.a() < 5` guards in the handlers).
pub const MIN_BUFFERED_BYTES: usize = 5;

/// A decoded server request: a subsystem command code plus its raw payload.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ServerMessage {
    /// The command code. Handlers compare this against a single byte; values
    /// are stored as `i32` to match the `i32` request-code model.
    pub code: i32,
    /// Raw payload bytes following the code byte (may be empty).
    pub payload: Vec<u8>,
}

impl ServerMessage {
    /// Build a message from a code and an owned payload.
    pub fn new(code: i32, payload: Vec<u8>) -> Self {
        Self { code, payload }
    }

    /// Build a code-only message with an empty payload.
    pub fn from_code(code: i32) -> Self {
        Self::new(code, Vec::new())
    }

    /// Serialize to the observed wire shape: one code byte followed by the
    /// raw payload. The code is truncated to its low 8 bits, matching the
    /// single-byte code read (`SocketInputStream.e()`) performed by handlers.
    pub fn encode(&self) -> Vec<u8> {
        let mut out = Vec::with_capacity(1 + self.payload.len());
        out.push(self.code as u8);
        out.extend_from_slice(&self.payload);
        out
    }

    /// Decode from the observed wire shape. Returns `None` for an empty input
    /// (there is no code byte to read). All remaining bytes are the payload.
    pub fn decode(bytes: &[u8]) -> Option<Self> {
        let (&code, payload) = bytes.split_first()?;
        Some(Self::new(i32::from(code), payload.to_vec()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::super::request_codes::{AlsaRequestCode, SysvRequestCode};

    #[test]
    fn code_only_message_encode_decode() {
        let msg = ServerMessage::from_code(AlsaRequestCode::Play.code());
        let bytes = msg.encode();
        assert_eq!(bytes, vec![1]);
        assert_eq!(ServerMessage::decode(&bytes), Some(msg));
    }

    #[test]
    fn message_with_payload_round_trips() {
        let msg = ServerMessage::new(SysvRequestCode::GetSize.code(), vec![0x2A, 0x00, 0x01]);
        let encoded = msg.encode();
        assert_eq!(encoded[0], 3); // code byte (GetSize == 3)
        assert_eq!(&encoded[1..], &[0x2A, 0x00, 0x01]);
        assert_eq!(ServerMessage::decode(&encoded), Some(msg));
    }

    #[test]
    fn decode_empty_input_is_none() {
        assert_eq!(ServerMessage::decode(&[]), None);
    }

    #[test]
    fn min_buffered_bytes_is_five() {
        assert_eq!(MIN_BUFFERED_BYTES, 5);
    }
}
