//! Rust model of `com.winemu.core.pe.IconImage` and a minimal ICO directory
//! reader (GameHub Lite v5.1.8).
//!
//! `IconImage.c(byte[], int)` from the reference:
//! * stores the raw bytes and the "available" length,
//! * parses a `BitmapInfoHeader` from the start of the image blob,
//! * sets `width = header.g()` and `height = Math.abs(header.d() / 2)`.
//!
//! `ExeFileInfo.Companion.a(PE)` feeds it the single RT_ICON resource bytes
//! for the largest group-icon entry with width <= 32.
//!
//! Header parsing only: pixel decoding (`IconImage.b()/d()`,
//! `BitmapFactory.decodeByteArray`, color quads) is out of scope.

use super::bitmap_info_header::{BitmapInfoHeader, ColorFormat, HeaderError};

/// `IconImage.c(byte[], int)` — the header-derived fields plus raw data size.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PeIcon {
    /// Parsed `BITMAPINFOHEADER` (`f20617a`).
    pub header: BitmapInfoHeader,
    /// `f20618b = header.g()` (DIB width in pixels).
    pub width: i32,
    /// `f20619c = Math.abs(header.d() / 2)` (ICO height is doubled).
    pub height: i32,
    /// `f20621e = i3` — available byte count passed by the caller.
    pub image_data_len: usize,
}

impl PeIcon {
    /// Port of `IconImage.c(bArr, i3)` minus pixel decoding: parse the DIB
    /// header at the start of the blob and derive width / half-height.
    pub fn from_bmp_bytes(bytes: &[u8], available_len: usize) -> Result<Self, HeaderError> {
        let header = BitmapInfoHeader::parse(bytes)?;
        let width = header.width;
        let height = header.half_abs_height();
        Ok(Self { header, width, height, image_data_len: available_len })
    }

    pub fn color_format(&self) -> ColorFormat {
        self.header.color_format
    }

    /// `IconImage.b()` branch: true => hand-written DIB path; the PNG path is
    /// deliberately not implemented here.
    pub fn is_win32_dib(&self) -> bool {
        self.header.is_win32_dib()
    }
}

/// One `ICONDIRENTRY` (16 bytes) from an ICO file.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct IcoDirectoryEntry {
    pub width: u8,
    pub height: u8,
    pub color_count: u8,
    pub reserved: u8,
    pub planes: u16,
    pub bit_count: u16,
    pub bytes_in_res: u32,
    pub image_offset: u32,
}

impl IcoDirectoryEntry {
    /// ICO stores 0 to mean 256 pixels.
    pub fn entry_width(&self) -> u16 {
        if self.width == 0 { 256 } else { self.width as u16 }
    }

    pub fn entry_height(&self) -> u16 {
        if self.height == 0 { 256 } else { self.height as u16 }
    }
}

/// Errors while reading the tiny ICO container around the DIB header.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum IcoError {
    /// Missing/short ICONDIR, or reserved/type fields are not 0/1.
    NotIco,
    /// ICONDIR declares zero images.
    NoEntries,
    /// First directory entry or the DIB image blob is truncated.
    Truncated,
    /// The DIB header itself could not be parsed.
    Header(HeaderError),
}

impl From<HeaderError> for IcoError {
    fn from(err: HeaderError) -> Self {
        IcoError::Header(err)
    }
}

/// Read the ICONDIR (6 bytes), the first ICONDIRENTRY (16 bytes) and the
/// `BITMAPINFOHEADER` at its offset. Returns the directory entry together with
/// the header-derived [`PeIcon`]. No pixel data is decoded.
pub fn parse_ico_header(bytes: &[u8]) -> Result<(IcoDirectoryEntry, PeIcon), IcoError> {
    if bytes.len() < 6 {
        return Err(IcoError::NotIco);
    }
    let reserved = u16::from_le_bytes([bytes[0], bytes[1]]);
    let kind = u16::from_le_bytes([bytes[2], bytes[3]]);
    let count = u16::from_le_bytes([bytes[4], bytes[5]]);
    if reserved != 0 || kind != 1 {
        return Err(IcoError::NotIco);
    }
    if count == 0 {
        return Err(IcoError::NoEntries);
    }
    if bytes.len() < 22 {
        return Err(IcoError::Truncated);
    }
    let e = &bytes[6..22];
    let entry = IcoDirectoryEntry {
        width: e[0],
        height: e[1],
        color_count: e[2],
        reserved: e[3],
        planes: u16::from_le_bytes([e[4], e[5]]),
        bit_count: u16::from_le_bytes([e[6], e[7]]),
        bytes_in_res: u32::from_le_bytes([e[8], e[9], e[10], e[11]]),
        image_offset: u32::from_le_bytes([e[12], e[13], e[14], e[15]]),
    };
    let start = entry.image_offset as usize;
    let blob = bytes.get(start..).ok_or(IcoError::Truncated)?;
    let icon = PeIcon::from_bmp_bytes(blob, entry.bytes_in_res as usize)?;
    Ok((entry, icon))
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Build a minimal ICO in memory: ICONDIR, one ICONDIRENTRY and a 40-byte
    /// top-down BITMAPINFOHEADER whose real height is doubled (ICO mask).
    fn build_synthetic_ico(
        width: u8,
        height: u8,
        bit_count: u16,
        compression: i32,
    ) -> (Vec<u8>, usize, u32) {
        let mut dib = Vec::new();
        dib.extend_from_slice(&40i32.to_le_bytes()); // biSize
        dib.extend_from_slice(&(width as i32).to_le_bytes()); // biWidth
        dib.extend_from_slice(&(-((height as i32) * 2)).to_le_bytes()); // biHeight
        dib.extend_from_slice(&1u16.to_le_bytes()); // biPlanes
        dib.extend_from_slice(&bit_count.to_le_bytes()); // biBitCount
        dib.extend_from_slice(&compression.to_le_bytes()); // biCompression
        for _ in 0..5 {
            dib.extend_from_slice(&0i32.to_le_bytes()); // size/image + ppm + clr fields
        }

        let mut bytes = vec![0u8, 0, 1, 0, 1, 0]; // reserved=0, type=1, count=1
        bytes.push(width);
        bytes.push(height);
        bytes.push(0); // color count
        bytes.push(0); // reserved
        bytes.extend_from_slice(&1u16.to_le_bytes()); // planes
        bytes.extend_from_slice(&bit_count.to_le_bytes());
        bytes.extend_from_slice(&(dib.len() as u32).to_le_bytes()); // bytes_in_res
        bytes.extend_from_slice(&22u32.to_le_bytes()); // image_offset
        bytes.extend_from_slice(&dib);
        (bytes, 22, dib.len() as u32)
    }

    #[test]
    fn parses_synthetic_ico_first_image() {
        let (bytes, expected_offset, expected_len) = build_synthetic_ico(32, 32, 32, 0);
        let (entry, icon) = parse_ico_header(&bytes).unwrap();

        assert_eq!(entry.width, 32);
        assert_eq!(entry.height, 32);
        assert_eq!(entry.planes, 1);
        assert_eq!(entry.bit_count, 32);
        assert_eq!(entry.bytes_in_res, expected_len);
        assert_eq!(entry.image_offset as usize, expected_offset);

        assert_eq!(icon.width, 32);
        assert_eq!(icon.height, 32);
        assert_eq!(icon.image_data_len, expected_len as usize);
        assert_eq!(icon.header.bit_count, 32);
        assert_eq!(icon.color_format(), ColorFormat::Sampled32);
        assert!(icon.is_win32_dib());
    }

    #[test]
    fn zero_dimension_means_256() {
        let entry = IcoDirectoryEntry {
            width: 0,
            height: 0,
            color_count: 0,
            reserved: 0,
            planes: 1,
            bit_count: 32,
            bytes_in_res: 0,
            image_offset: 22,
        };
        assert_eq!(entry.entry_width(), 256);
        assert_eq!(entry.entry_height(), 256);
    }

    #[test]
    fn rejects_non_ico_and_truncated_inputs() {
        assert_eq!(parse_ico_header(&[]), Err(IcoError::NotIco));
        assert_eq!(parse_ico_header(&[0, 0, 2, 0, 1, 0]), Err(IcoError::NotIco));
        assert_eq!(parse_ico_header(&[0, 0, 1, 0, 0, 0]), Err(IcoError::NoEntries));
        assert_eq!(parse_ico_header(&[0, 0, 1, 0, 1, 0, 0]), Err(IcoError::Truncated));

        let mut bytes = vec![0, 0, 1, 0, 1, 0];
        bytes.extend_from_slice(&[0; 16]); // entry with image_offset = 0
        assert!(matches!(parse_ico_header(&bytes), Err(IcoError::Header(_))));
    }
}
