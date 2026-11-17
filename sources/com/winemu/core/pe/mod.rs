//! Rust port of the portable `com.winemu.core.pe` header readers and EXE info
//! model (GameHub Lite v5.1.8).
//!
//! * [`bitmap_info_header`] — `BitmapInfoHeader` field reader plus its
//!   `BitmapCompression`, `BitmapOriginDirection` and `ColorFormat` enums.
//! * [`icon_image`] — `PeIcon` (from `IconImage`) and a minimal ICO directory
//!   reader; no pixel decoding.
//! * [`exe_info`] — `ExeFileInfo` / `ExeInfo` data model and version-key mapping.
//!
//! Real PE parsing (pecoff4j), image decoding and Android `Bitmap` allocation
//! are intentionally out of scope.

pub mod bitmap_info_header;
pub mod exe_info;
pub mod icon_image;

pub use self::bitmap_info_header::{
    BitmapCompression, BitmapInfoHeader, BitmapOriginDirection, ColorFormat, HeaderError,
};
pub use self::exe_info::{ExeFileInfo, ExeInfo};
pub use self::icon_image::{parse_ico_header, IcoDirectoryEntry, IcoError, PeIcon};
