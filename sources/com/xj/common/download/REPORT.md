# REPORT — general Aria / WinEmu download pipeline (Rust reimplementation)

## What was implemented

Dependency-free (std-only) Rust models of the general download → unzip → merge →
verify pipeline used for WinEmu environment/container and game downloads at the
path `sources/com/xj/common/download/`.

| File | Contents |
|---|---|
| `mod.rs` | Module declaration and re-exports of all public items. |
| `download_job.rs` | `DownloadJob` (`id`, `url`, `dest`, `total`, `downloaded`, `state`, `speed`, `progress` 0–100) and `JobState` (11 states, integer codes 0–10 from `CommonDownloadTaskState`). Byte/progress bookkeeping, speed semantics, state transitions (`start`/`pause`/`resume`/`download_config`/`verify`/`merge`/`complete`/`fail`/`cancel`), `left_time()` and `upgrade()`. |
| `extraction_plan.rs` | `ExtractionStep` (`Download` → `Unzip` → `Merge` → `Verify`, with Android string keys), `ArchiveFormat` (extension **and** magic-byte detection) and `ExtractionPlan`. |
| `file_merge.rs` | `ManifestEntry {name, size, md5}`, `FileMerge` manifest, `ObservedFile`, `Mismatch`, `VerifyReport`, `FileMerge::verify()` over precomputed hashes (no crypto), plus `content_equal()` mirroring `FileMerger.e`. |
| `aria_task_config.rs` | `AriaTaskConfig` (threads, retry, block mode and the other `aria_config.xml` download settings), `QueueMod`, `Default`/`DEFAULT` from the XML semantics, `sanitized()`, `effective_threads()`, and a minimal `from_xml()` parser. |

All four modules include `#[cfg(test)]` tests covering state transitions,
progress clamping, format detection (extension + magic, incl. TAR `ustar`
offset and magic-over-extension priority), manifest verify (missing/size/hash/
unexpected cases), and Aria defaults/overrides/sanitization.

## Reference sources studied (read-only)

From `reference/gamehub-lite-v518-decompiled/gamehub-lite-v518-decompiled/`:

- `sources/com/xj/common/download/bean/CommonDownloadTask.java` — field names,
  `percent`, `speed` setter semantics, `isDownloading()`, `getLeftTime()`,
  `upgrade()`.
- `sources/com/xj/common/download/bean/CommonDownloadTaskState.java` — the 11
  state codes (Normal=0 … Pending=10).
- `sources/com/xj/common/download/bean/AriaDownloadArgs.java`, `SubData.java`,
  `DownloaderType.java` — `fileMd5`/`fileSize`, `{sub_file_name,
  sub_download_url, sub_file_md5}` manifest shape, downloader kinds.
- `sources/com/xj/common/download/` (30 files) — `UxDownloadManage`,
  `UxDownloadUtils`, `CommonDownloadCtrl`, `DownloadNotification`,
  `SimpleDownloadCallback`, listeners/events (Android service + UI plumbing,
  not ported).
- `sources/com/xj/winemu/download/` (106 files), notably
  `WinEmuDownloadManager.java`, `WinEmuFileExplorer.java`,
  `explorer/UnZipComponent.java`, `action/*DownloadAction.java` — env/container
  download orchestration, MD5 checks, unzip pipeline.
- `sources/com/xj/winemu/download/service/UnzipService.java` and
  `UnzipServiceConfig.java` — 8192-byte unzip buffer, default 3 retries,
  `unzip_state` event reporting, foreground-service gating.
- `sources/com/xj/winemu/data/bean/UnZipFileArgs.java` — archive path, dest path
  and expected MD5 payload.
- `sources/com/xj/standalone/steam/contentdownloader/FileMerger.java` —
  atomic `.tmp` move + `e(file1, file2)` byte comparison used for merge/verify.
- `sources/net/lingala/zip4j/` (98 files), notably
  `headers/HeaderSignature.java` — ZIP magic signatures (0x04034B50 etc.)
  used for magic-byte detection.
- `apktool/assets/aria_config.xml` (and `resources/assets/aria_config.xml`) —
  `threadNum=3`, `reTryNum=3`, `reTryInterval=5000`, `useBlock=true`,
  `buffSize=8192`, `iOTimeOut=10000`, `connectTimeOut=5000`, `queueMod=wait`,
  and the documented min-thread/min-buffer/min-IO-timeout invariants.
- `resources/res/values/strings.xml` — `comm_unzipping` ("正在解压中") and
  `llauncher_pc_game_file_merge` ("Files are merging") used as the unzip/merge
  stage labels.

## Relationship to other Rust modules

- `sources/com/xj/common/data/common_download_task_state.rs` already defines an
  equivalent `CommonDownloadTaskState`. `JobState` here is intentionally a
  separate, self-contained copy so this module can be wired independently; the
  integer codes match exactly.
- `apktool/assets/config.rs` defines `AriaConfig` (a generic XML value reader).
  `AriaTaskConfig` here consumes the same XML semantics but is independently
  parsed and adds the documented constraints/effective-thread logic.
- `sources/org/compression.rs` is an inventory enum only; `ArchiveFormat` here
  adds runtime detection by extension and magic bytes.
- The OTA downloader at `sources/com/xj/ota/download.rs` was **not modified**.

## Out of scope (not implemented)

- Any actual network I/O, HTTP/Aria engine, coroutine scheduling, Android
  `Service`/`Intent`/`Notification`/foreground permission handling.
- Real decompression / extraction (zip4j, `java.util.zip`, 7z/RAR/etc.) — only
  format *detection* and the ordered *plan* are modelled.
- MD5 computation — hashes are opaque precomputed strings supplied by the
  caller; no crypto crate is used.
- Real file moves/overwrites (atomic `.tmp` + `ATOMIC_MOVE`) — `content_equal`
  and the manifest `verify()` model the comparison only.
- Steam / cloud gaming / remote play paths; WinEmu native `.so` engines.
- XML parser generality — `from_xml()` handles only the fixed `<key
  value="..."/>` / `<key>...</key>` subset used by `aria_config.xml`.

## Wiring

Not performed here (lib.rs is outside this write area). Suggested entry in
`app/src/lib.rs`:

```rust
#[path = "../sources/com/xj/common/download/mod.rs"]
pub mod download;
```
