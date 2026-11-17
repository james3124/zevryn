# Area report — emulator exec chain (Rust port)

Reference: `sources/com/winemu/core/{controller,server/environment/plugins,trans_layer}/*.java`
and `sources/defpackage/{State,EmbeddedFileManager,ComponentRepo,RunEmbedded}.java`
of GameHub Lite v5.1.8. All modules are dependency-free (std only), plain
structs/enums with `#[cfg(test)]` unit tests. Layout follows `PLAN.md`: Rust
`mod.rs` package roots plus sibling `.rs` files sit at the mirrored Java paths,
to be wired via `#[path]` in `app/src/lib.rs`. Names are kept consistent with
the core crate's `src/emu.rs` (`ComponentState`, `EmbeddedFile`, `ElfMachine`,
`inspect_elf`, `ENV_KEYS`, `exec_argv`, and their method names).

## Files created

### `app/sources/defpackage/`

| File | Models | Contents |
|---|---|---|
| `state.rs` | `defpackage.State` | `ComponentState` enum (codes 0/1/2/4/8), `ALL`, `from_code`/`code`, `is_installed` |
| `embedded_file_manager.rs` | `EmbeddedFileManager` + `RunEmbedded` | `EmbeddedFile` (run/tabtip/winmonitor + versioned cache names), `is_cached_version` eviction predicate |
| `component_repo.rs` | portable part of `ComponentRepo` | `ComponentRepo` plain data struct (name, version, state, is_dep, is_base) |
| `mod.rs` | package root | submodule declarations + re-exports |

### `app/sources/com/winemu/core/`

| File | Models | Contents |
|---|---|---|
| `program_launcher.rs` | `ProgramLauncher` + env keys from `EnvironmentController` | `ElfMachine`, `inspect_elf` (20-byte ELF header, e_machine 3/62 ⇒ box64), `LINKER64`/`DEFAULT_BOX64`, `exec_argv` (`linker64 [box64] <cmd> [extra]`), `ENV_KEYS` |
| `box64_config.rs` | `trans_layer.Box64Config` | 28-field plain data struct, `Default` values, `set_compatibility_mode`/`set_performance_mode`, `export()` → ordered `BOX64_*` assignments |
| `wine.rs` | `Wine` / `WineArch` | arch enum, `root`, `is_arm64x`/`is_x86`, architecture-dependent `lib_dir()` |
| `boot_data.rs` | `BootData` | two-flag plain data struct with `a/b/c/d` accessors |
| `container.rs` | portable part of `Container` | `BottleMetadata` + `Container` data structs, file-name constants, pure `parse_patch_version` |
| `tricks.rs` | `Tricks` | game-id membership lists, `ConfigOverrides`, `env_tricks()`/`cef_files()` |
| `mod.rs` | package root | submodule declarations + re-exports |

## Intentionally not implemented (out of scope)

- **Native `.so` libraries / emulation runtime**: `libwinemu.so`, `libxserver.so`,
  `libvfs.so`, `libsandboxfs.so`, `libbox64` itself. No Wine, X server, VFS or
  Box64 dynarec — only the portable command/env shaping that surrounds them.
- **Process launch / process control**: `ProcessHelper.g/k/m`, `Process.killProcess`,
  PID tracking, callbacks, `PathUtils`/PATH resolution against the Android
  filesystem, `EnvVars` application and signal handling. `exec_argv` returns a
  `Vec<String>`; it never spawns a process.
- **Embedded PE payloads**: the Base64 blobs inside `RunEmbedded`,
  `TabtipEmbedded` and `WinMonitorEmbedded` are real Windows executables written
  to the cache dir; they are native assets, not portable logic, so only their
  filenames/version tokens and cache-naming rules are modeled. The `f(File,
  byte[])` write/rename logic is filesystem I/O and is excluded.
- **Android/JVM plumbing**: `Context`, `Intent`, `Parcelable`/`CREATOR`,
  Kotlin metadata/`@Keep`/`EnumEntries`, `@NotNull` assertions, logging.
- **Xj-specific beans**: `EnvLayerEntity` (`ComponentRepo.entry`) and
  `DependencyManager.Companion.Info` (`depInfo`) are omitted from `ComponentRepo`.
- **Gson JSON / metadata I/O**: `Container.e()/f()/g()` read/write
  `metadata.json` and `patch_version.txt`; only the data shape and the pure
  integer-parse helper are kept, not the file access.
- **Config / Wine runtime side effects**: `EnvironmentController` writes dozens
  of env vars based on a live `Config` object; only the representative
  `ENV_KEYS` set is modeled. Render/driver paths (DXVK, Gamescope, Mesa,
  Pulse/ALSA, Reshade, FEX) and gamescope/driver detection need a real device.
- **Steam / cloud gaming / remote play**: excluded by rebuild scope.

## Verification

Each module carries its own `#[cfg(test)]` tests (component codes, cache names
and eviction, ELF x86 detection incl. big-endian, argv building, Box64 defaults
and modes/export, Wine arch paths, BootData accessors, container patch parsing,
Tricks lookups). Wiring is via `#[path]` mod declarations in `app/src/lib.rs`,
which is owned by the orchestrating agent and was not modified here.
