# REPORT — third-party / native trees (GameHub Lite v5.1.8)

## Scope and honest disclosure

The directories assigned to this area are **third-party libraries and native
binaries**, not first-party GameHub application logic:

- `sources/org/apache/commons/compress/**` — Apache Commons Compress (archives
  and stream codecs).
- `sources/org/tukaani/xz/**` — XZ / LZMA Java implementation.
- `sources/net/lingala/zip4j/**` — ZIP / encrypted-ZIP library.
- `sources/org/bouncycastle/**` — Bouncy Castle cryptography provider.
- `sources/org/conscrypt/**` — Conscrypt TLS/crypto provider (incl. HPKE).
- `apktool/lib/arm64-v8a/**` — 19 prebuilt native shared objects (`.so`).

**These trees are NOT reimplemented in Rust.** The decompiled Java sources and
the prebuilt `.so` files are large, vendor-owned, and outside the scope of a
portable reimplementation. This deliverable only **inventories** the portable,
useful metadata (format/algorithm sets, ABI, library names/sizes/roles) and
records which Rust crates would be used as replacements. No Java logic was
translated and no native code was ported.

The following focused modules were produced (std-only, no external deps):

| File | Contents |
| --- | --- |
| `sources/org/compression.rs` | `CompressionFormat` enum (7 formats) + crate mapping + tests |
| `sources/org/crypto.rs` | `CryptoAlgorithm` enum (7 algorithms) + crate mapping + tests |
| `apktool/lib/native.rs` | `NativeAbi`, `NativeDisposition`, 19-entry `NATIVE_LIBS` table + tests |

Each module is self-contained and includes `#[cfg(test)]` unit tests
(13 tests total; all pass with `rustc --edition 2021 --test`).

## Compression — replacement crates

| Format | Java origin | Suggested Rust crate |
| --- | --- | --- |
| ZIP | `net.lingala.zip4j`, `commons.compress.archivers.zip` | `zip` |
| 7z | `commons.compress.archivers.sevenz` | `sevenz-rust` |
| tar | `commons.compress.archivers.tar` | `tar` |
| gzip | `commons.compress.compressors.gzip` | `flate2` |
| xz / lzma | `org.tukaani.xz`, `commons.compress.compressors.xz` | `xz2` |
| zstd | `commons.compress.compressors.zstandard` (+ `libzstd-jni`) | `zstd` |
| bzip2 | `commons.compress.compressors.bzip2` | `bzip2` |

## Crypto — replacement crates

| Algorithm | Java origin | Suggested Rust crate |
| --- | --- | --- |
| AES | Bouncy Castle `AESEngine` | `aes-gcm` |
| ChaCha20 | `ChaCha7539Engine`, Conscrypt | `chacha20poly1305` |
| RSA | `RSABlindedEngine` | `rsa` |
| SHA-1 | `SHA1Digest` | `sha1` |
| HMAC | `HMac`, `javax.crypto.Mac` | `hmac` |
| HPKE | Conscrypt `HpkeImpl` / `HpkeSpi` | `hpke` |
| PQC | Bouncy Castle `pqc.crypto.crystals` | `ml-kem` |

## Native ABI

Only one ABI is shipped: **`arm64-v8a`** (`NativeAbi::Arm64V8a`). There are no
32-bit ARM, x86, or x86_64 libraries in the reference build.

## Native library inventory (19 `.so` files, 15,315,720 bytes ≈ 14.61 MiB)

Disposition legend: **FFI** = keep and call through FFI; **crate** = replaceable
by a Rust crate; **drop** = optional/vendor-specific blob that can be dropped.

| Library | Bytes | Role | Disposition / replacement |
| --- | ---: | --- | --- |
| `libGamesir.so` | 27,232 | GameSir gamepad SDK JNI bridge | drop (optional controller support) |
| `libadb.so` | 550,928 | ADB client for remote device install/debug | FFI (no drop-in Rust equivalent) |
| `libavutil-55.so` | 366,152 | FFmpeg AVUtil core | FFI (media stack) |
| `libblur-lib.so` | 9,944 | Native image blur filter | crate — `image` / blur filter |
| `libgpuinfo.so` | 340,264 | GPU / GLES capability probe | crate — GLES queries in Rust |
| `libijkplayer.so` | 545,704 | IjkPlayer video playback engine | FFI (media stack) |
| `libijksdl.so` | 362,832 | IjkPlayer SDL output layer | FFI (media stack) |
| `libimage_processing_util_jni.so` | 28,944 | Image-processing utilities | crate — `image` |
| `libimagescale.so` | 55,280 | Fast image scaling helper | crate — `image` |
| `libjl_ota_auth.so` | 15,704 | Jieli vendor OTA firmware auth | drop (vendor-specific) |
| `liblog.so` | 66,376 | Native Android log shim | crate — `log` |
| `libmmkv.so` | 586,640 | Tencent MMKV key-value store | crate — `redb` / `rkv` |
| `libswscale-4.so` | 349,736 | FFmpeg pixel-format scaling | FFI (media stack) |
| `libvfs.so` | 984,680 | Virtual file-system for game archives | FFI (custom VFS semantics) |
| `libwebdav_jni.so` | 2,736,848 | WebDAV remote-storage client | crate — WebDAV client crate |
| `libwinemu.so` | 592,856 | Windows-game emulation glue | FFI (emulator core) |
| `libxjps-jni.so` | 3,185,896 | XJPS image/codec JNI bridge | FFI (proprietary codec) |
| `libxserver.so` | 3,913,824 | Embedded X11 server for compatibility layer | FFI (emulator core) |
| `libzstd-jni-1.5.5-11.so` | 595,880 | zstd-jni native binding | crate — `zstd` |

## Notes for the integrating agent

- `app/src/lib.rs` and `Cargo.toml` are owned by other agents. Wire these files
  in with `#[path = "..."] mod ...;` (or equivalent) when integrating; each
  module compiles independently with only `std`.
- The crate names above are *suggestions* recorded as string data in the enums;
  no dependencies were added and no crates were vendored.
- The sizes are byte-exact copies of the reference APK's `lib/arm64-v8a/`
  entries, so the inventory can be verified against the source tree.
