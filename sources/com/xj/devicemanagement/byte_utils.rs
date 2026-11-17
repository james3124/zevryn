//! Portable byte/hex helpers from the device-management and OTA utility
//! classes (pure functions, no Android / JNI dependencies):
//!
//! * `com.xj.devicemanagement.utils.StringUtil.a(byte[])` — prefixed hex.
//! * `com.krly.platform.util.Utils` — `a()` big-endian int, `b()` CRC-16,
//!   `c()` uppercase hex (also mirrored here for device-management callers).
//! * `com.xj.ota.set.HexTools` — `a()` bracketed hex, `b()` u16 -> LE bytes,
//!   `c()` byte sum.

/// `StringUtil.a(byte[])`: `0xNN, 0xNN, ...` (uppercase, two digits) or `None`
/// for an empty/null array.
pub fn format_bytes_prefixed(bytes: &[u8]) -> Option<String> {
    if bytes.is_empty() {
        return None;
    }
    let parts: Vec<String> = bytes.iter().map(|b| format!("0x{:02X}", b)).collect();
    Some(parts.join(", "))
}

/// `Utils.c(byte[])` / `StringUtil` uppercase hex without separators.
pub fn hex_upper(bytes: &[u8]) -> String {
    let mut out = String::with_capacity(bytes.len() * 2);
    for b in bytes {
        out.push(char::from_digit((b >> 4) as u32, 16).unwrap().to_ascii_uppercase());
        out.push(char::from_digit((b & 0x0f) as u32, 16).unwrap().to_ascii_uppercase());
    }
    out
}

/// `HexTools.a(byte[])`: lowercase bracketed, comma-separated hex.
pub fn hex_bracketed(bytes: &[u8]) -> String {
    let mut out = String::from("[");
    for (i, b) in bytes.iter().enumerate() {
        if i > 0 {
            out.push(',');
        }
        out.push_str(&format!("{:x}", b));
    }
    out.push(']');
    out
}

/// `Utils.a(byte[])`: interpret the last (up to) four bytes as a big-endian
/// unsigned integer, padding with zeroes when the slice is shorter.
pub fn bytes_to_u32_be(bytes: &[u8]) -> u32 {
    let mut buf = [0u8; 4];
    for i in 0..4 {
        let src = bytes.len() as isize - 1 - i as isize;
        if src >= 0 {
            buf[3 - i] = bytes[src as usize];
        }
    }
    u32::from_be_bytes(buf)
}

/// `HexTools.b(long)`: little-endian low 16 bits.
pub const fn u16_to_le_bytes(value: u16) -> [u8; 2] {
    [value as u8, (value >> 8) as u8]
}

/// `HexTools.c(byte[])`: sum of unsigned byte values (modulo 2^32).
pub fn byte_sum(bytes: &[u8]) -> u32 {
    bytes.iter().fold(0u32, |acc, b| acc.wrapping_add(*b as u32))
}

/// `Utils.b(byte[])`: CRC-16/CCITT (poly 0x1021, initial value 0), the CRC
/// used by `KrCmdController` before sending a firmware image.
pub fn crc16_ccitt(bytes: &[u8]) -> u16 {
    let mut crc: u32 = 0;
    for &byte in bytes {
        for bit in 0..8 {
            let data_bit = (byte >> (7 - bit)) & 1 == 1;
            let crc_msb = (crc >> 15) & 1 == 1;
            crc <<= 1;
            if data_bit ^ crc_msb {
                crc ^= 0x1021;
            }
        }
    }
    (crc & 0xffff) as u16
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn prefixed_hex_matches_string_util() {
        assert_eq!(format_bytes_prefixed(&[]), None);
        assert_eq!(format_bytes_prefixed(&[0x01, 0xab]), Some("0x01, 0xAB".into()));
    }

    #[test]
    fn hex_formats_match_java_helpers() {
        assert_eq!(hex_upper(&[0x00, 0x0f, 0xff]), "000FFF");
        assert_eq!(hex_bracketed(&[0x0a, 0xff]), "[a,ff]");
        assert_eq!(hex_bracketed(&[]), "[]");
    }

    #[test]
    fn big_endian_int_uses_last_four_bytes() {
        assert_eq!(bytes_to_u32_be(&[0x01, 0x02, 0x03, 0x04]), 0x0102_0304);
        assert_eq!(bytes_to_u32_be(&[0xff, 0x01, 0x02, 0x03, 0x04]), 0x0102_0304);
        assert_eq!(bytes_to_u32_be(&[0x12, 0x34]), 0x1234);
        assert_eq!(bytes_to_u32_be(&[]), 0);
    }

    #[test]
    fn u16_conversion_and_byte_sum() {
        assert_eq!(u16_to_le_bytes(0x1234), [0x34, 0x12]);
        assert_eq!(byte_sum(&[0xff, 0x01, 0x80]), 0xff + 1 + 0x80);
        assert_eq!(byte_sum(&[]), 0);
    }

    #[test]
    fn crc16_matches_known_ccitt_vector() {
        // Utils.b() uses poly 0x1021 with initial value 0x0000 (not the
        // 0xFFFF CCITT-FALSE variant), giving 0x31C3 for "123456789".
        assert_eq!(crc16_ccitt(b"123456789"), 0x31c3);
        assert_eq!(crc16_ccitt(&[]), 0);
        assert_eq!(crc16_ccitt(&[0x01]), crc16_ccitt(&[0x01]));
    }
}
