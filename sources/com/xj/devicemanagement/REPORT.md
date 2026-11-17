# Area report — `com.xj.devicemanagement` (Rust port)

See `../ota/REPORT.md` (master report) for the full file inventory, field
sources, not-implemented list and verification.

Files: `mod.rs`, `device_info.rs` (14-field Kotlin `DeviceInfo` bean and its
`FirmwareDataBean`), `byte_utils.rs` (hex / big-endian int / checksum / CRC-16
helpers from `StringUtil`, `HexTools`, `Utils`).

Not implemented: all BLE (`BLEScanner`, `BLEService`, receivers) and USB
(`USBMonitor`, `USBFinder`, `UsbOTGService`, permissions) transports, broadcast
utils and coroutine plumbing — only portable data structures and pure byte
helpers are modeled here.
