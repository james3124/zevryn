# Area report — emulator component state (Rust port)

This report covers the `defpackage` portion of the emulator exec-chain port;
the companion report for `com.winemu.core` is at
`app/sources/com/winemu/core/REPORT.md`. Reference:
`sources/defpackage/{State,EmbeddedFileManager,ComponentRepo,RunEmbedded}.java`
of GameHub Lite v5.1.8. Dependency-free (std only), with `#[cfg(test)]` tests.

## Files created

| File | Models | Contents |
|---|---|---|
| `state.rs` | `defpackage.State` | `ComponentState` enum (codes 0/1/2/4/8), `ALL`, `from_code`/`code`, `is_installed` |
| `embedded_file_manager.rs` | `EmbeddedFileManager` + `RunEmbedded` | `EmbeddedFile` (run/tabtip/winmonitor + versioned cache names), `is_cached_version` eviction predicate |
| `component_repo.rs` | portable part of `ComponentRepo` | `ComponentRepo` plain data struct (name, version, state, is_dep, is_base) |
| `mod.rs` | package root | submodule declarations + re-exports |

Names are kept consistent with the core crate's `src/emu.rs` (`ComponentState`,
`EmbeddedFile`, `from_code`/`code`, `filename`/`version`/`cached_name`).

## Out of scope

- Embedded PE payloads (Base64 `RunEmbedded`/`TabtipEmbedded`/`WinMonitorEmbedded`
  blobs) and the `f(File, byte[])` write/rename filesystem logic.
- Android/Kotlin plumbing: `@Keep`, `@Metadata`, `EnumEntries`, `Intrinsics`,
  `Log`, `File` I/O, `Base64` decoding.
- Xj-specific beans: `EnvLayerEntity` (`entry`) and
  `DependencyManager.Companion.Info` (`depInfo`) on `ComponentRepo`.
- Native `.so` libraries, Wine/X server, process launch — see the companion report.
