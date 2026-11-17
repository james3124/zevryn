# Area report — `com.krly` (Rust port)

See `../xj/ota/REPORT.md` (master report) for the full file inventory, field
sources, not-implemented list and verification.

Files: `mod.rs`, `command.rs` (`KrCommand` codes 80–83 from `KrCmdController`
request beans, `KrRequest` frame encode/decode from `BaseReq.c()`, `ready_payload`,
`ReqFactory`, `SendBytesNumberRsp`, `CommandAck`), `firmware_ota.rs`
(`FirmwareOtaError`, `UpgradeEvent`, `chunk_firmware`, `packet_index`,
`FirmwareOtaPhase`, `FirmwareOtaSession` modeled on `FirmwareOtaController`).

Not implemented: `BLEOtaManager` GATT I/O, Bluetooth state listeners, RxJava
scheduling, `Thread.sleep` retries, Android `Context`. `DeviceOTAVerifyReq` and
`DonglePairingReq` have no identifiable command code in the decompiled sources,
so none was invented.
