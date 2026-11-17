# Area report — `com.xiaoji.sdk` constants (Rust port)

Reference (read-only): `sources/com/xiaoji/sdk/` (360 files) of GameHub Lite
v5.1.8. The SDK is almost entirely device/USB/HID/GCM plumbing (Android
`UsbManager`, JNI, native injection) and is out of scope; only the portable
constant holders in `constants/` are modeled.

## Files created

| File | Models | Contents |
|---|---|---|
| `mod.rs` | package root | `pub mod constants;` + re-exports of `GcmKeyCode`, `HidKeyCode`, `XjStateCode` |
| `constants.rs` | `com.xiaoji.sdk.constants.*` | zero-sized marker types for the two stripped holders; exact 3-byte markers for `XjStateCode`; name lookup and stream search helpers; tests |

Wiring (`app/src/lib.rs`):
`#[path = "../sources/com/xiaoji/sdk/mod.rs"] pub mod sdk;` — `constants.rs`
resolves automatically as its sibling. No external dependencies.

## Decompilation status of the three holders

All three classes in `sources/com/xiaoji/sdk/constants/` failed JADX's
`KotlinMetadataDecompile` pass:

- **`GcmKeyCode.java`** — only an empty `Companion` survives; no key-code
  constants are present. Modeled as a zero-sized marker struct; **no constants
  were invented**.
- **`HidKeyCode.java`** — same: empty `Companion`, no constants. Modeled as a
  zero-sized marker struct.
- **`XjStateCode.java`** — the class body survives, including three `byte[]`
  fields (names mangled to `f21782b`, `f21783c`, `f21784d`) and a
  `Companion.a()` accessor returning the third one. These are modeled exactly:

| Port constant | Decompiled field | Decompiled initializer | Bytes |
|---|---|---|---|
| `XjStateCode::MARKER_A` | `f21782b` | `{-1, Ascii.SI, 0}` | `[0xFF, 0x0E, 0x00]` |
| `XjStateCode::MARKER_C` | `f21783c` | `{0, MessagePack.Code.EXT32, -1}` | `[0x00, 0xC9, 0xFF]` |
| `XjStateCode::MARKER_D` | `f21784d` | `{0, -1, 0}` | `[0x00, 0xFF, 0x00]` |

`Ascii.SI` is Guava's Shift-In control char (`0x0E`); `MessagePack.Code.EXT32`
is `0xC9`; both are widened from signed Java `byte` to unsigned `u8` in the
port. `XjStateCode::state_end()` reproduces `Companion.a()` -> `MARKER_D`.

`XjStateCode::name()` and `XjStateCode::find()` are small derived helpers for
identifying the markers in a byte stream; they are new portable logic, not
claims about decompiled methods.

## Intentionally not implemented

- GCM/HID key-code maps (stripped — nothing to restore).
- Device configuration (`device/**`), lighting (`gcm/GcmLightingUtil`), USB
  access, native `libxj*` libraries and all injection paths.

## Verification

Compiled and tested standalone with `rustc --edition 2021 --test` (together
with the `com.xj.mapping` area): all tests pass, 0 warnings.
