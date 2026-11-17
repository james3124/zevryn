//! GCM wire-frame model, derived from `com.xiaoji.sdk.gcm.GcmProtocol` and its
//! `com.xiaoji.sdk.gcm.check.*` validators.
//!
//! # Verified byte layout
//!
//! Every field below is confirmed by executable code in the reference; nothing
//! is guessed. `send` is the command we wrote, `receive` the device reply.
//!
//! | Offset | Field    | Verified by                                                                 |
//! |--------|----------|------------------------------------------------------------------------------|
//! | 0      | COMMAND  | `BaseCheck.b`: `receive[0] == send[0]`; `GcmProtocol.B(3)` = "COMMAND值不正确" |
//! | 1      | TYPE     | `NormalCommandWithActionCheck.h` / `NormalCommandWithoutActionCheck.g`: `b[1] == send[1]`; `B(4)` = "TYPE值不正确" |
//! | 2      | LENGTH   | `NormalCommandWithActionCheck.g` / `NormalCommandWithoutActionCheck.f`: `b.len() == (b[2] & 0xFF)`; `B(5)` = "LENGTH值不正确" |
//! | 3      | ACTION   | `NormalCommandWithActionCheck.f`: `b[3] == send[3]` (only in the 4-byte-header form); `B(7)` = "Action值不正确" |
//! | last   | CHECKSUM | `BaseCheck.d/e`, `GcmProtocol.Q/r0`: `b[last] == sum(b[0..last])`; `B(8)` = "校验和不正确" |
//!
//! `GcmProtocol.E(cmd, type, action, params)` builds a 4-byte-header frame
//! `{cmd, type, LENGTH, action, <payload>, 0x00}` and sets
//! `LENGTH = payload.len() + 5` (3 header + 1 action + 1 checksum).
//! `GcmProtocol.F(cmd, type, params)` builds a 3-byte-header frame
//! `{cmd, type, LENGTH, <payload>, 0x00}` with `LENGTH = payload.len() + 4`
//! (3 header + 1 checksum). In both cases `r0()` overwrites the final `0x00`
//! with the checksum, which `Q()` computes as the wrapping 8-bit sum of all
//! bytes except the last. `BaseCheck.c` additionally requires a minimum data
//! length (`receive.len() >= 2 + min`, error code 6).
//!
//! Our [`GcmPacket::encode`] and [`GcmPacket::decode`] reproduce exactly those
//! two forms. The ACTION-vs-no-ACTION choice is a property of the command
//! (`GcmProtocol.D` picks a `NormalCommandWithActionCheck` or
//! `NormalCommandWithoutActionCheck`), so the caller selects it explicitly via
//! [`HeaderForm`] when decoding.

/// Which header form a frame uses (see module docs).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HeaderForm {
    /// `GcmProtocol.F` style: 3-byte header, no ACTION at offset 3.
    WithoutAction,
    /// `GcmProtocol.E` style: 4-byte header with ACTION at offset 3.
    WithAction,
}

impl HeaderForm {
    /// Number of header bytes before the payload (3 or 4).
    pub const fn header_len(self) -> usize {
        match self {
            Self::WithoutAction => 3,
            Self::WithAction => 4,
        }
    }
}

/// A decoded/encodable GCM packet.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GcmPacket {
    /// COMMAND — frame byte 0.
    pub command: u8,
    /// TYPE — frame byte 1.
    pub kind: u8,
    /// ACTION — frame byte 3, present only in [`HeaderForm::WithAction`] frames.
    pub action: Option<u8>,
    /// Data bytes between the header and the trailing checksum.
    pub payload: Vec<u8>,
}

/// Why a byte buffer is not a valid GCM frame.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DecodeError {
    /// Fewer bytes than the header plus checksum.
    TooShort { len: usize },
    /// `frame[2] != frame.len()` (LENGTH field mismatch, error code 5).
    LengthMismatch { length_field: u8, actual: usize },
    /// Trailing checksum byte does not equal the sum of preceding bytes
    /// (error code 8).
    ChecksumMismatch { expected: u8, actual: u8 },
}

/// Verified field offsets within a frame.
pub mod offsets {
    pub const COMMAND: usize = 0;
    pub const TYPE: usize = 1;
    pub const LENGTH: usize = 2;
    pub const ACTION: usize = 3;
}

/// Wrapping 8-bit checksum: sum of every byte in `bytes` (the caller excludes
/// the checksum byte itself). Mirrors `GcmProtocol.Q` / `BaseCheck.e`.
pub fn checksum(bytes: &[u8]) -> u8 {
    bytes.iter().fold(0u8, |sum, &b| sum.wrapping_add(b))
}

impl GcmPacket {
    /// Build a packet. Supplying `Some(action)` selects the 4-byte-header form.
    pub fn new(command: u8, kind: u8, action: Option<u8>, payload: Vec<u8>) -> Self {
        Self { command, kind, action, payload }
    }

    /// The header form implied by this packet's `action` field.
    pub fn header_form(&self) -> HeaderForm {
        if self.action.is_some() { HeaderForm::WithAction } else { HeaderForm::WithoutAction }
    }

    /// Total number of bytes in the encoded frame (header + payload + checksum).
    pub fn frame_len(&self) -> usize {
        self.header_form().header_len() + self.payload.len() + 1
    }

    /// Encode the frame: `[command, kind, length, (action), payload, checksum]`.
    ///
    /// `length` is set to the total frame size (`GcmProtocol` sets it to
    /// `payload.len() + 5` or `+ 4` for the two forms, which is the same value).
    pub fn encode(&self) -> Vec<u8> {
        let mut out = Vec::with_capacity(self.frame_len());
        out.push(self.command);
        out.push(self.kind);
        out.push(0); // LENGTH placeholder at byte 2.
        if let Some(action) = self.action {
            out.push(action);
        }
        out.extend_from_slice(&self.payload);
        out[offsets::LENGTH] = out.len() as u8 + 1; // + checksum byte
        let sum = checksum(&out);
        out.push(sum);
        out
    }

    /// Decode a frame, checking LENGTH and CHECKSUM exactly as the reference
    /// validators do. `form` tells us whether byte 3 is an ACTION field.
    pub fn decode(bytes: &[u8], form: HeaderForm) -> Result<Self, DecodeError> {
        let min = form.header_len() + 1; // header + checksum
        if bytes.len() < min {
            return Err(DecodeError::TooShort { len: bytes.len() });
        }
        if bytes.len() > u8::MAX as usize + 1 {
            return Err(DecodeError::LengthMismatch {
                length_field: bytes[offsets::LENGTH],
                actual: bytes.len(),
            });
        }
        let length_field = bytes[offsets::LENGTH];
        if length_field as usize != bytes.len() {
            return Err(DecodeError::LengthMismatch { length_field, actual: bytes.len() });
        }
        let body = &bytes[..bytes.len() - 1];
        let expected = checksum(body);
        let actual = bytes[bytes.len() - 1];
        if expected != actual {
            return Err(DecodeError::ChecksumMismatch { expected, actual });
        }
        let action = match form {
            HeaderForm::WithoutAction => None,
            HeaderForm::WithAction => Some(bytes[offsets::ACTION]),
        };
        let payload = body[form.header_len()..].to_vec();
        Ok(Self {
            command: bytes[offsets::COMMAND],
            kind: bytes[offsets::TYPE],
            action,
            payload,
        })
    }

    /// Convenience decode for the 3-byte-header (`GcmProtocol.F`) form.
    pub fn decode_plain(bytes: &[u8]) -> Result<Self, DecodeError> {
        Self::decode(bytes, HeaderForm::WithoutAction)
    }

    /// Convenience decode for the 4-byte-header (`GcmProtocol.E`) form.
    pub fn decode_with_action(bytes: &[u8]) -> Result<Self, DecodeError> {
        Self::decode(bytes, HeaderForm::WithAction)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn checksum_is_wrapping_sum_of_body() {
        // GcmProtocol.Q / BaseCheck.e: sum of all bytes except the last.
        assert_eq!(checksum(&[1, 2, 3]), 6);
        assert_eq!(checksum(&[0xFF, 0x02, 0x03]), 4); // wraps mod 256
    }

    #[test]
    fn round_trip_plain_frame() {
        let packet = GcmPacket::new(0x01, 0x00, None, vec![0x10, 0x20, 0x30]);
        let bytes = packet.encode();
        // Header = 3, payload = 3, checksum = 1 -> length byte 7.
        assert_eq!(bytes.len(), 7);
        assert_eq!(bytes[offsets::COMMAND], 0x01);
        assert_eq!(bytes[offsets::TYPE], 0x00);
        assert_eq!(bytes[offsets::LENGTH] as usize, bytes.len());
        assert_eq!(bytes[bytes.len() - 1], checksum(&bytes[..bytes.len() - 1]));
        assert_eq!(GcmPacket::decode_plain(&bytes), Ok(packet));
    }

    #[test]
    fn round_trip_frame_with_action() {
        let packet = GcmPacket::new(0x08, 0x01, Some(0x06), vec![0xAA, 0xBB]);
        let bytes = packet.encode();
        // Header = 4, payload = 2, checksum = 1 -> length byte 7.
        assert_eq!(bytes.len(), 7);
        assert_eq!(bytes[offsets::ACTION], 0x06);
        assert_eq!(GcmPacket::decode_with_action(&bytes), Ok(packet));
    }

    #[test]
    fn empty_payload_frames() {
        let plain = GcmPacket::new(0x05, 0x03, None, vec![]);
        let b = plain.encode();
        assert_eq!(b.len(), 4);
        assert_eq!(GcmPacket::decode_plain(&b), Ok(plain));

        let with_action = GcmPacket::new(0x05, 0x03, Some(0x01), vec![]);
        let b = with_action.encode();
        assert_eq!(b.len(), 5);
        assert_eq!(GcmPacket::decode_with_action(&b), Ok(with_action));
    }

    #[test]
    fn rejects_bad_length() {
        let bytes = GcmPacket::new(1, 0, None, vec![1, 2]).encode();
        let mut bad = bytes.clone();
        bad[offsets::LENGTH] = 0x7F;
        assert_eq!(
            GcmPacket::decode_plain(&bad),
            Err(DecodeError::LengthMismatch { length_field: 0x7F, actual: bytes.len() })
        );
    }

    #[test]
    fn rejects_bad_checksum() {
        let mut bad = GcmPacket::new(1, 0, None, vec![1, 2]).encode();
        let last = bad.len() - 1;
        bad[last] ^= 0xFF;
        assert!(matches!(
            GcmPacket::decode_plain(&bad),
            Err(DecodeError::ChecksumMismatch { .. })
        ));
    }

    #[test]
    fn rejects_too_short() {
        assert_eq!(GcmPacket::decode_plain(&[1, 2]), Err(DecodeError::TooShort { len: 2 }));
        // A 4-byte-header frame needs at least 5 bytes (header + checksum).
        assert_eq!(
            GcmPacket::decode_with_action(&[1, 2, 4, 0]),
            Err(DecodeError::TooShort { len: 4 })
        );
    }
}
