# REPORT — `com.winemu.core.utils` Rust model

Task: dependency-free (std-only) Rust reimplementation of the portable parts of
the GameHub Lite v5.1.8 `com.winemu.core.utils` package. Work was confined to
this write area (`app/sources/com/winemu/core/utils/`); `Cargo.toml`,
`app/src/lib.rs`, the parent `core/mod.rs` and other agents' directories were
not touched.

## Files written

| File | Models | Key API |
|---|---|---|
| `mod.rs` | package root, module declarations + re-exports | `pub use ...` for every public type/fn |
| `env_vars.rs` | `EnvVars` | `new`, `from_pairs`, `get`, `set`/`put`, `remove`, `extend`, `set_path`, `export`/`to_string_array`, `to_env_string`, `to_export_script`, `keys`, `iter`, `clear` |
| `win_path.rs` | `PathUtils`, `WinUtils`, `StringUtils` | `is_absolute`, `to_windows_path`, `to_unix_path`, `join_windows`, `join`, `strip_trailing_separators` |
| `cpu_info.rs` | `CpuInfoCollector`, `EnvironmentController.d()` affinity | `is_cpu_dir_name`, `core_count_from_names`, `cpu_affinity`, `set_cpu_affinity`, `Affinity`, `AllCoreFrequencyInfo`, `sysfs::*`, `WINEMU_CPU_AFFINITY` |
| `port.rs` | `PortUtils` | `PortRange`, `find_free_port`, `DEFAULT_PORT_START/END` |
| `tar_layout.rs` | `TarCompressorUtils` | `CompressionType`, `TarEntry`, `TarEntryKind`, `TarLayout` |
| `program.rs` | `Program`, `ProcessHelper.l()` | `Program` (path/args/env/cwd), `ProgramEntry` (label/icon/command), `split_command_line` |
| `net.rs` | `NetworkHelper.Companion` | `ipv4_from_int`, `netmask_from_prefix` |
| `battery.rs` | `BatteryUtils` | `normalize_current`, `voltage_from_extra`, `is_charging` |

Each file has inline `#[cfg(test)]` tests (36 total). Validation:

```
cd app/sources/com/winemu/core/utils
rustc --test --edition 2021 -D warnings -o /tmp/utils_test mod.rs && /tmp/utils_test
# 36 passed; 0 failed; 0 warnings
```

## Sources studied (reference, read-only)

All under `reference/.../sources/com/winemu/core/utils/`:
`EnvVars.java`, `ProcessHelper.java`, `ProcessUtils.java`, `WinUtils.java`,
`PathUtils.java`, `FileUtils.java`, `CpuInfoCollector.java`, `PortUtils.java`,
`NetworkHelper.java`, `Program.java`, `StreamUtils.java`, `StringUtils.java`,
`TTCHelper.java`, `TarCompressorUtils.java`, `BatteryUtils.java`,
`JWMConfig.java`, `RefreshRateHelper.java`.

Plus two callers for cross-file behavior:
- `sources/com/winemu/core/controller/EnvironmentController.java` — the
  `WINEMU_CPU_AFFINITY = ~((1 << iB) - 1)` computation at line 179, where
  `iB = coreCount - container.x()` and the write is skipped unless
  `container.x() > 0 && iB > 0`.
- `sources/com/winemu/core/program_launcher.rs` (other agent) — read only to
  confirm the existing `ENV_KEYS` list; it was **not** modified. This package's
  `EnvVars` is the fuller map implementation.

## Behaviors modeled (faithfully)

- **EnvVars**: Java wraps a `LinkedHashMap`, so iteration/export order is
  insertion order and re-`set`ting a key keeps its original position. Ported
  `a` (get), `c` (set), `g` (putAll/extend), `h`/`i` (drop empty components +
  join with `:`), `toStringArray` (`export()` → ordered `Vec<"KEY=VALUE">`),
  `toString` (space-joined), and `b()` (shell `export KEY="VALUE"` lines with
  value quotes escaped). Iterator over keys is supported.
- **Paths**: `PathUtils.a` = `starts_with("/")`; `WinUtils.b` =
  `drive + "\\" + path(backslash-converted)` with trailing separators trimmed;
  `StringUtils.a` = strip all trailing `/` or `\`. Added `to_unix_path` (the
  inverse) and a generic `join`.
- **CPU affinity**: `~((1<<iB)-1)` as a signed 32-bit Java `int` (e.g. 8 cores,
  2 reserved → `-64` = `0xFFFF_FFC0`), with the early-return guards. Also the
  `CpuInfoCollector` `cpu[0-9]` name predicate and the `AllCoreFrequencyInfo`
  usage-percentage math (`e`, `d`).
- **Ports**: inclusive `PortRange` with validation and an ordered first-free
  scan against a used set — the deterministic stand-in for
  `DatagramSocket(0).getLocalPort()`.
- **Tar layout**: ordered `TarEntry` list capturing `TarArchiveEntry`'s
  `p/s/u/H/k` fields, plus the `Type` tag (None/XZ/ZSTD/BZ2).
- **Program**: full descriptor `path`/`args`/`env`/`cwd`, plus a verbatim
  `ProgramEntry` (label/icon/command) with Java-equivalent `equals`/hashCode/
  toString, and `ProcessHelper.l()` command-line tokenization (quotes, `\ `
  escapes, space splitting).
- **Network/Battery**: pure static transforms only (int→IPv4, prefix→netmask;
  current normalization, voltage scaling, charging flag).

## Out of scope (intentionally not implemented)

- **Real process spawn / signals / PID reflection** — `ProcessHelper.g/f/e/d`,
  `Runtime.exec`, `Process.sendSignal` (`k`/`m`), thread-pool stream pumping.
  Only the pure command-line tokenizer is kept.
- **Sockets / network I/O** — `PortUtils.a()`'s real `DatagramSocket(0)` and
  `NetworkHelper`'s `WifiManager`/`DhcpInfo`/`NetworkInterface` probing; only
  pure range and string conversions are modeled.
- **Compression / archive parsing / extraction** — Commons Compress streams
  (XZ/ZSTD/BZip2), tar byte parsing, `FileUtils` symlink/permission/`unzip`
  helpers, `StreamUtils` byte pumping. `TarLayout` is data only.
- **Android platform & IPC** — `Parcelable`/`CREATOR`/`Log.d` in `EnvVars`,
  `Context`/`BatteryManager`/`IntentFilter` receivers in `BatteryUtils`,
  `JWMConfig` (Android preferences/system), `RefreshRateHelper` (display
  manager), `ProcessUtils` (`/proc` scan), `FileUtils` filesystem walks.
- **Native code** — `TTCHelper`'s `System.loadLibrary("winemu")` and its
  `extractTTC` JNI method.

## Wiring note for the coordinator

`mod.rs` is a self-contained package root. Add `pub mod utils;` to
`app/sources/com/winemu/core/mod.rs` to expose it under `winemu_core::utils`
(the module uses `super::EnvVars` internally, so it compiles both as a
standalone crate root and when nested under `winemu_core`). `Cargo.toml` and
`app/src/lib.rs` require no changes because `winemu_core` is already declared.
