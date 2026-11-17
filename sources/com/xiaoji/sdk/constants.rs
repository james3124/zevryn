//! Constant holders from `com.xiaoji.sdk.constants`.
//!
//! In the reference all three are Kotlin `object`/`final class` constant
//! containers. JADX failed the `KotlinMetadataDecompile` pass on them and only
//! the class shells plus three byte arrays (in `XjStateCode`) survived. We
//! model exactly what is portable and reconstructible; nothing is invented.

/// `com.xiaoji.sdk.constants.GcmKeyCode`.
///
/// The decompiled class contains only an empty `Companion`; every key-code
/// constant was stripped. Kept as a zero-sized marker type so the name exists
/// in the port without fabricating key-code values.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct GcmKeyCode;

/// `com.xiaoji.sdk.constants.HidKeyCode`.
///
/// Same situation as [`GcmKeyCode`]: constants stripped, only the class shell
/// survives. Kept as a zero-sized marker type.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct HidKeyCode;

/// `com.xiaoji.sdk.constants.XjStateCode`.
///
/// Three 3-byte markers survive decompilation. The original fields are named
/// `f21782b`, `f21783c` and `f21784d` in the decompiled source; the names below
/// are stable aliases used by the port.
///
/// | Field | Decompiled value | Bytes |
/// |---|---|---|
/// | `f21782b` | `{-1, Ascii.SI, 0}` | `[0xFF, 0x0E, 0x00]` |
/// | `f21783c` | `{0, MessagePack.Code.EXT32, -1}` | `[0x00, 0xC9, 0xFF]` |
/// | `f21784d` | `{0, -1, 0}` | `[0x00, 0xFF, 0x00]` |
///
/// `Ascii.SI` is Guava's Shift-In control character (`0x0E`) and
/// `MessagePack.Code.EXT32` is `0xC9`, both widened to unsigned bytes here.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct XjStateCode;

impl XjStateCode {
    /// `f21782b = {-1, Ascii.SI, 0}`.
    pub const MARKER_A: [u8; 3] = [0xFF, 0x0E, 0x00];
    /// `f21783c = {0, MessagePack.Code.EXT32, -1}`.
    pub const MARKER_C: [u8; 3] = [0x00, 0xC9, 0xFF];
    /// `f21784d = {0, -1, 0}` — also returned by the `Companion.a()` accessor.
    pub const MARKER_D: [u8; 3] = [0x00, 0xFF, 0x00];

    /// All surviving markers, in declaration order.
    pub const ALL: [[u8; 3]; 3] = [Self::MARKER_A, Self::MARKER_C, Self::MARKER_D];

    /// The marker returned by the decompiled `Companion.a()` method (`f21784d`).
    pub fn state_end() -> [u8; 3] {
        Self::MARKER_D
    }

    /// Name of a marker, or `None` if the bytes are not one of the three.
    pub fn name(marker: &[u8; 3]) -> Option<&'static str> {
        match *marker {
            Self::MARKER_A => Some("MARKER_A"),
            Self::MARKER_C => Some("MARKER_C"),
            Self::MARKER_D => Some("MARKER_D"),
            _ => None,
        }
    }

    /// Scan a byte stream and return the first `(offset, marker_name)` hit.
    pub fn find(data: &[u8]) -> Option<(usize, &'static str)> {
        for i in 0..data.len().saturating_sub(2) {
            let window = [data[i], data[i + 1], data[i + 2]];
            if let Some(name) = Self::name(&window) {
                return Some((i, name));
            }
        }
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn marker_values_match_decompiled_arrays() {
        assert_eq!(XjStateCode::MARKER_A, [0xFF, 0x0E, 0x00]);
        assert_eq!(XjStateCode::MARKER_C, [0x00, 0xC9, 0xFF]);
        assert_eq!(XjStateCode::MARKER_D, [0x00, 0xFF, 0x00]);
        assert_eq!(XjStateCode::state_end(), XjStateCode::MARKER_D);
        assert_eq!(XjStateCode::ALL.len(), 3);
    }

    #[test]
    fn marker_names() {
        assert_eq!(XjStateCode::name(&XjStateCode::MARKER_A), Some("MARKER_A"));
        assert_eq!(XjStateCode::name(&XjStateCode::MARKER_C), Some("MARKER_C"));
        assert_eq!(XjStateCode::name(&XjStateCode::MARKER_D), Some("MARKER_D"));
        assert_eq!(XjStateCode::name(&[0, 0, 0]), None);
    }

    #[test]
    fn finds_marker_in_stream() {
        let data = [0x01, 0x02, 0x00, 0xC9, 0xFF, 0x03];
        assert_eq!(XjStateCode::find(&data), Some((2, "MARKER_C")));
        assert_eq!(XjStateCode::find(&[1, 2]), None);
        assert_eq!(XjStateCode::find(&[]), None);
    }

    #[test]
    fn stripped_holders_are_zero_sized() {
        assert_eq!(std::mem::size_of::<GcmKeyCode>(), 0);
        assert_eq!(std::mem::size_of::<HidKeyCode>(), 0);
    }
}
