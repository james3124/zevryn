# Area report — `com.xj.apk` (Rust port)

See `../ota/REPORT.md` (master report) for the full file inventory, field
sources, not-implemented list and verification.

Files: `mod.rs` (includes the shared `../ota/download.rs` as `download`),
`update.rs` (`ApkUpdateConfig`, validation from `DownloadManager.a()/b()`,
`UpdateDecision` from `download()`, `UpdatePaths`, constants).

Not implemented: Android `DownloadService`, `HttpDownloadManager` networking,
notifications / `FileProvider`, `UpdateDialogActivity`, lifecycle callbacks and
package installation. BLE and USB are out of scope for this area entirely.
