//! Pure header reader for the `BITMAPINFOHEADER` used by
//! `com.winemu.core.pe.BitmapInfoHeader` (GameHub Lite v5.1.8).
//!
//! Port of `BitmapInfoHeader.h(IDataReader)`, which reads 11 fields in sequence:
//! `biSize` (DWORD), `biWidth` (LONG), `biHeight` (LONG), `biPlanes` (WORD),
//! `biBitCount` (WORD), `biCompression` (DWORD), `biSizeImage` (DWORD),
//! `biXPelsPerMeter` (LONG), `biYPelsPerMeter` (LONG), `biClrUsed` (DWORD),
//! `biClrImportant` (DWORD) — 40 bytes total.
//!
//! No image decoding, color-table reading or Android `Bitmap` creation is done
//! here; those live in `IconImage.b()/d()` and are out of scope.

use std::convert::TryInto;
use std::fmt;

/// Truncated or malformed header input.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HeaderError {
    /// The buffer ended before the 40-byte DIB header was complete.
    NeedMore { have: usize, need: usize },
}

impl fmt::Display for HeaderError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            HeaderError::NeedMore { have, need } => {
                write!(f, "need {need} bytes for BITMAPINFOHEADER, got {have}")
            }
        }
    }
}

impl std::error::Error for HeaderError {}

fn read_i32(bytes: &[u8], offset: usize) -> Result<i32, HeaderError> {
    let end = offset
        .checked_add(4)
        .ok_or(HeaderError::NeedMore { have: bytes.len(), need: offset + 4 })?;
    let chunk: [u8; 4] = bytes
        .get(offset..end)
        .ok_or(HeaderError::NeedMore { have: bytes.len(), need: end })?
        .try_into()
        .unwrap();
    Ok(i32::from_le_bytes(chunk))
}

fn read_u16(bytes: &[u8], offset: usize) -> Result<u16, HeaderError> {
    let end = offset
        .checked_add(2)
        .ok_or(HeaderError::NeedMore { have: bytes.len(), need: offset + 2 })?;
    let chunk: [u8; 2] = bytes
        .get(offset..end)
        .ok_or(HeaderError::NeedMore { have: bytes.len(), need: end })?
        .try_into()
        .unwrap();
    Ok(u16::from_le_bytes(chunk))
}

/// `BitmapInfoHeader.BitmapCompression` — `fromValue` falls back to `RGB`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BitmapCompression {
    Rgb,
    Rle8,
    Rle4,
    Bitfields,
    Jpeg,
    Png,
    AlphaBitfields,
    Cmyk,
    CmykRle8,
    CmykRle4,
}

impl BitmapCompression {
    pub fn from_value(value: i32) -> Self {
        match value {
            1 => BitmapCompression::Rle8,
            2 => BitmapCompression::Rle4,
            3 => BitmapCompression::Bitfields,
            4 => BitmapCompression::Jpeg,
            5 => BitmapCompression::Png,
            6 => BitmapCompression::AlphaBitfields,
            11 => BitmapCompression::Cmyk,
            12 => BitmapCompression::CmykRle8,
            13 => BitmapCompression::CmykRle4,
            _ => BitmapCompression::Rgb,
        }
    }

    pub fn value(self) -> i32 {
        match self {
            BitmapCompression::Rgb => 0,
            BitmapCompression::Rle8 => 1,
            BitmapCompression::Rle4 => 2,
            BitmapCompression::Bitfields => 3,
            BitmapCompression::Jpeg => 4,
            BitmapCompression::Png => 5,
            BitmapCompression::AlphaBitfields => 6,
            BitmapCompression::Cmyk => 11,
            BitmapCompression::CmykRle8 => 12,
            BitmapCompression::CmykRle4 => 13,
        }
    }
}

/// `BitmapInfoHeader.BitmapOriginDirection` — chosen from the sign of
/// `biHeight` (`< 0` => `UPPER_LEFT`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BitmapOriginDirection {
    BottomLeft,
    UpperLeft,
}

/// `BitmapInfoHeader.ColorFormat` — `fromValue` falls back to `Unknown`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ColorFormat {
    Unknown,
    Indexed1,
    Indexed2,
    Indexed4,
    Indexed8,
    Sampled16,
    Sampled24,
    Sampled32,
}

impl ColorFormat {
    pub fn from_value(value: u16) -> Self {
        match value {
            1 => ColorFormat::Indexed1,
            2 => ColorFormat::Indexed2,
            4 => ColorFormat::Indexed4,
            8 => ColorFormat::Indexed8,
            16 => ColorFormat::Sampled16,
            24 => ColorFormat::Sampled24,
            32 => ColorFormat::Sampled32,
            _ => ColorFormat::Unknown,
        }
    }

    pub fn value(self) -> u16 {
        match self {
            ColorFormat::Unknown => 0,
            ColorFormat::Indexed1 => 1,
            ColorFormat::Indexed2 => 2,
            ColorFormat::Indexed4 => 4,
            ColorFormat::Indexed8 => 8,
            ColorFormat::Sampled16 => 16,
            ColorFormat::Sampled24 => 24,
            ColorFormat::Sampled32 => 32,
        }
    }
}

/// Header-only projection of `com.winemu.core.pe.BitmapInfoHeader`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BitmapInfoHeader {
    /// `biSize` (`f20601a`, exposed by `f()`); 40 for Win32 BITMAPINFOHEADER.
    pub size: i32,
    /// `biWidth` (`f20602b`, exposed by `g()`).
    pub width: i32,
    /// `biHeight` (`f20603c`, exposed by `d()`). Negative means top-down.
    pub height: i32,
    /// `biPlanes` (`f20604d`).
    pub planes: u16,
    /// `biBitCount` (`f20605e`, exposed by `a()`).
    pub bit_count: u16,
    /// `biCompression` (`f20606f`).
    pub compression: BitmapCompression,
    /// `biSizeImage` (`f20607g`).
    pub size_image: i32,
    /// `biXPelsPerMeter` (`f20608h`).
    pub x_pixels_per_meter: i32,
    /// `biYPelsPerMeter` (`f20609i`).
    pub y_pixels_per_meter: i32,
    /// `biClrUsed` (`f20611k`).
    pub clr_used: i32,
    /// `biClrImportant` (`f20610j`, exposed by `b()`).
    pub clr_important: i32,
    /// Derived in `h()` from the sign of `biHeight` (`f20612l` / `e()`).
    pub origin: BitmapOriginDirection,
    /// Derived from `biBitCount` (`f20613m` / `c()`).
    pub color_format: ColorFormat,
}

impl BitmapInfoHeader {
    /// Port of `BitmapInfoHeader.h(IDataReader)` over a little-endian byte slice.
    /// Requires the full 40-byte header; trailing pixel/color-table data is
    /// ignored.
    pub fn parse(bytes: &[u8]) -> Result<Self, HeaderError> {
        if bytes.len() < 40 {
            return Err(HeaderError::NeedMore { have: bytes.len(), need: 40 });
        }

        let size = read_i32(bytes, 0)?;
        let width = read_i32(bytes, 4)?;
        let height = read_i32(bytes, 8)?;
        let planes = read_u16(bytes, 12)?;
        let bit_count = read_u16(bytes, 14)?;
        let compression = BitmapCompression::from_value(read_i32(bytes, 16)?);
        let size_image = read_i32(bytes, 20)?;
        let x_pixels_per_meter = read_i32(bytes, 24)?;
        let y_pixels_per_meter = read_i32(bytes, 28)?;
        let clr_used = read_i32(bytes, 32)?;
        let clr_important = read_i32(bytes, 36)?;

        let origin = if height < 0 {
            BitmapOriginDirection::UpperLeft
        } else {
            BitmapOriginDirection::BottomLeft
        };
        let color_format = ColorFormat::from_value(bit_count);

        Ok(Self {
            size,
            width,
            height,
            planes,
            bit_count,
            compression,
            size_image,
            x_pixels_per_meter,
            y_pixels_per_meter,
            clr_used,
            clr_important,
            origin,
            color_format,
        })
    }

    /// Java `f() == 40` gate used by `IconImage.b()` to choose the hand-written
    /// DIB path over `BitmapFactory.decodeByteArray`.
    pub fn is_win32_dib(&self) -> bool {
        self.size == 40
    }

    /// `Math.abs(header.d() / 2)` from `IconImage.c()` — ICO images store the
    /// real height doubled (including the XOR+AND bitmap mask).
    pub fn half_abs_height(&self) -> i32 {
        (self.height / 2).abs()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Build a synthetic 40-byte BITMAPINFOHEADER (plus optional trailing
    /// bytes) with the given fields, little-endian.
    fn synthetic_dib(
        size: i32,
        width: i32,
        height: i32,
        planes: u16,
        bit_count: u16,
        compression: i32,
        trailing: &[u8],
    ) -> Vec<u8> {
        let mut bytes = Vec::with_capacity(40 + trailing.len());
        bytes.extend_from_slice(&size.to_le_bytes());
        bytes.extend_from_slice(&width.to_le_bytes());
        bytes.extend_from_slice(&height.to_le_bytes());
        bytes.extend_from_slice(&planes.to_le_bytes());
        bytes.extend_from_slice(&bit_count.to_le_bytes());
        bytes.extend_from_slice(&compression.to_le_bytes());
        bytes.extend_from_slice(&8192i32.to_le_bytes()); // biSizeImage
        bytes.extend_from_slice(&2835i32.to_le_bytes()); // biXPelsPerMeter
        bytes.extend_from_slice(&2835i32.to_le_bytes()); // biYPelsPerMeter
        bytes.extend_from_slice(&0i32.to_le_bytes()); // biClrUsed
        bytes.extend_from_slice(&0i32.to_le_bytes()); // biClrImportant
        bytes.extend_from_slice(trailing);
        bytes
    }

    #[test]
    fn parses_synthetic_top_down_32bpp_header() {
        let bytes = synthetic_dib(40, 32, -64, 1, 32, 0, b"PIXELS");
        let header = BitmapInfoHeader::parse(&bytes).unwrap();
        assert_eq!(header.size, 40);
        assert_eq!(header.width, 32);
        assert_eq!(header.height, -64);
        assert_eq!(header.planes, 1);
        assert_eq!(header.bit_count, 32);
        assert_eq!(header.compression, BitmapCompression::Rgb);
        assert_eq!(header.size_image, 8192);
        assert_eq!(header.x_pixels_per_meter, 2835);
        assert_eq!(header.y_pixels_per_meter, 2835);
        assert_eq!(header.clr_used, 0);
        assert_eq!(header.clr_important, 0);
        assert_eq!(header.origin, BitmapOriginDirection::UpperLeft);
        assert_eq!(header.color_format, ColorFormat::Sampled32);
        assert!(header.is_win32_dib());
        assert_eq!(header.half_abs_height(), 32);
    }

    #[test]
    fn parses_bottom_up_8bpp_header() {
        let bytes = synthetic_dib(40, 16, 32, 1, 8, 1, &[]);
        let header = BitmapInfoHeader::parse(&bytes).unwrap();
        assert_eq!(header.origin, BitmapOriginDirection::BottomLeft);
        assert_eq!(header.compression, BitmapCompression::Rle8);
        assert_eq!(header.color_format, ColorFormat::Indexed8);
        assert_eq!(header.half_abs_height(), 16);
    }

    #[test]
    fn rejects_truncated_header() {
        assert_eq!(
            BitmapInfoHeader::parse(&[0u8; 20]),
            Err(HeaderError::NeedMore { have: 20, need: 40 })
        );
    }

    #[test]
    fn unknown_values_fall_back_like_java_fromvalue() {
        assert_eq!(BitmapCompression::from_value(99), BitmapCompression::Rgb);
        assert_eq!(ColorFormat::from_value(99), ColorFormat::Unknown);
        assert_eq!(BitmapCompression::Png.value(), 5);
        assert_eq!(ColorFormat::Sampled24.value(), 24);
    }
}
