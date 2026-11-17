//! Rust model of the portable parts of `com.xiaoji.sdk`.
//!
//! Mirrors `sources/com/xiaoji/sdk/` from GameHub Lite v5.1.8. The SDK is
//! overwhelmingly device/USB/HID/GCM plumbing (Android `UsbManager`, JNI,
//! native injection) and is out of scope. The only portable, reconstructible
//! data here are the three constant holders in `constants`:
//!
//! - `GcmKeyCode` and `HidKeyCode` — their constant tables were stripped by
//!   the decompiler (empty Kotlin companion classes), so only marker types
//!   are modeled; see `REPORT.md`.
//! - `XjStateCode` — three 3-byte state markers survive decompilation and are
//!   modeled exactly as `[u8; 3]` constants.

pub mod constants;

pub use self::constants::{GcmKeyCode, HidKeyCode, XjStateCode};
