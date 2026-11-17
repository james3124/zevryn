# REPORT — Injection decoder + Aria config parser (Rust reimplementation)

Subagent area report for GameHub Lite v5.1.8. Both deliverables are dependency-free
(std only) and mirror the API of the existing core crate files `src/inject.rs` and
`src/config.rs`.

## Files written (only inside my assigned write areas)

| File | Purpose |
|---|---|
| `app/sources/com/xiaoji/inject/mod.rs` | Byte-reversed `.xj` payload decoder + shell-variable extraction |
| `app/apktool/assets/config.rs` | `AriaConfig` parser for `aria_config.xml` |
| `app/sources/com/xiaoji/inject/REPORT.md` | This report (inject area copy) |
| `app/apktool/assets/REPORT.md` | This report (assets area copy) |

Nothing outside these two areas was touched (`Cargo.toml`, `app/src/lib.rs`, and other
agents' directories were left untouched).

## 1. Injection decoder — `.../inject/mod.rs`

Reference studied: `apktool/assets/inject_{1,2,3}.xj` and
`sources/com/xiaoji/inject/utils/InjectActivationUtils.java` / `AssetsFileUtils.java`
(read-only). `InjectActivationUtils.n()` confirms the runtime reverses the bytes of
each `.xj` asset before writing the decoded file.

API (kept identical to `src/inject.rs`):

- `pub struct InjectPayload { script, device, model, version, package_placeholder }`
- `pub fn decode(&[u8]) -> Result<String, std::string::FromUtf8Error>` — reverse bytes, decode UTF-8.
- `InjectPayload::parse(&[u8]) -> Result<Self, String>` — decode + `shell_var` extraction.
- private `fn shell_var(&str, &str) -> Option<String>` — extracts `NAME="value"` assignments.

Findings from the real assets:

- `inject_1.xj` (1151 B) reverses to a valid UTF-8 `/system/bin/sh` script containing
  `device="$1"`, `XJMODEL="$2"`, `XJVER="$3"`, `PACKAGE="package_name"`, `xjServer.jar`,
  `xjServer_a`, and `app_process ... XiaojiInjectServer`.
- `inject_2.xj` (179192 B) reverses to a DEX (`dex\n035`), not UTF-8 → `decode` errors.
- `inject_3.xj` (1163908 B) reverses to a ZIP/APK (`PK\x03\x04`), not UTF-8 → `decode` errors.
  The parser therefore rejects those two gracefully rather than panicking.

Tests: roundtrip, synthetic payload with all four variables, missing-variable case,
real `inject_1.xj`, and graceful UTF-8 failure for `inject_2/3`.

## 2. Aria config parser — `.../assets/config.rs`

Reference studied: `apktool/assets/aria_config.xml` (read-only). The real file uses
`<key value="..." />` (with Chinese XML comments); the parser supports both that form
and `<key>...</key>`.

API (kept identical to `src/config.rs`):

- `pub struct AriaConfig` with all 11 fields:
  `max_speed, thread_num, re_try_num, re_try_interval, connect_time_out, io_time_out,
  use_block, use_head_request, net_check, log_level, max_task_num`.
- `AriaConfig::parse(&str) -> Self`; private `tag_value`, `attr_value`, `parse_bool`.
- `parse_bool` accepts `true/1` and `false/0`; bad/missing values become `None`.

Values read from the real file's first (`<app>` / `<download>`) sections:
`maxSpeed=0`, `threadNum=3`, `reTryNum=3`, `reTryInterval=5000`, `connectTimeOut=5000`,
`iOTimeOut=10000`, `useBlock=true`, `useHeadRequest=false`, `netCheck=true`,
`logLevel=1`, `maxTaskNum=2`.

Tests: attribute form (all fields), text form (all fields), missing/malformed values,
and the real file via absolute reference path.

## Test files fall back gracefully

Every "real file" test reads the absolute reference path first
(`/home/Project/Zevryn/reference/gamehub-lite-v518-decompiled/gamehub-lite-v518-decompiled/apktool/assets/...`),
then the mirrored local copy, and returns early if neither exists. No test depends on
another agent's work being complete.

## Validation performed

Compiled each module standalone with `rustc --test` (edition 2021) and ran tests:

- inject: **5 passed, 0 failed**
- config: **4 passed, 0 failed**

Also compiled both modules together in one crate with `#![deny(warnings)]` via
`#[path = "..."] mod ...;` — clean, no warnings.

## Wiring note for the coordinator

The files are ready to be declared in `app/src/lib.rs` as:

```rust
#[path = "../sources/com/xiaoji/inject/mod.rs"]
pub mod inject;
#[path = "../apktool/assets/config.rs"]
pub mod config;
```

(Paths shown are relative to `app/src/lib.rs`; the coordinator may choose its own.)
