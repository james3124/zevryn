# Area report — `com.winemu.core.pe` (Rust port)

Full report: `../gpu/REPORT.md` (covers both assigned write areas, all reference
sources, exact env var names and out-of-scope items).

Files here:
- `mod.rs` — re-exports `BitmapInfoHeader`, `BitmapCompression`,
  `BitmapOriginDirection`, `ColorFormat`, `HeaderError`, `PeIcon`,
  `IcoDirectoryEntry`, `IcoError`, `parse_ico_header`, `ExeFileInfo`, `ExeInfo`.
- `bitmap_info_header.rs` — `BitmapInfoHeader::parse(&[u8])`, a pure 40-byte
  little-endian reader with the same field order as
  `BitmapInfoHeader.h(IDataReader)`, plus the three derived enums.
- `icon_image.rs` — `PeIcon::from_bmp_bytes` (port of `IconImage.c`) and
  `parse_ico_header` for a minimal ICONDIR/first-entry reader.
- `exe_info.rs` — `ExeInfo` (the `ExeFileInfo.Info` bean) and `ExeFileInfo`.

Not implemented: image/pixel decoding (`IconImage.b()/d()`, color quads,
`BitmapFactory`), real PE parsing (`PEParser`, `ResourceHelper`, `VersionInfo`,
`StringTable`), Android `Bitmap` objects. Tests build synthetic ICO/BMP byte
buffers locally and assert header fields, driver env mapping and state
transitions in the sibling `gpu/` tests.
