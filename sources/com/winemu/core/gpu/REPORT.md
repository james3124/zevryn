# Area report — `com.winemu.core.gpu` + `com.winemu.core.pe` (Rust port)

This report covers both write areas assigned for this port:
`app/sources/com/winemu/core/gpu/` and `app/sources/com/winemu/core/pe/`.

## Files written

### `gpu/`
- `mod.rs` — package root: `pub mod driver; pub mod direct_rendering;` with
  re-exports of all models, enums, env-name constants and builder functions.
- `driver.rs` — `GpuDriver`, `GpuInfo`, `GpuDriverConfig`, `DriverEnvError`,
  `HudLevel`; env constants and the functions
  `driver_replacement_env`, `renderdoc_env`, `dxvk_env`, `mesa_zink_env`,
  `vkd3d_env`, `all_gpu_env`. 10 unit tests.
- `direct_rendering.rs` — `DirectRenderingState`, `NativeRenderingMode`,
  `DirectRenderingController`, `DirectRenderingEvent`; decision/state logic and
  GameScope control-byte encoding. 6 unit tests.
- `REPORT.md` — this file.

### `pe/`
- `mod.rs` — package root: `pub mod bitmap_info_header; pub mod exe_info;
  pub mod icon_image;` with re-exports.
- `bitmap_info_header.rs` — `BitmapInfoHeader`, `HeaderError`,
  `BitmapCompression`, `BitmapOriginDirection`, `ColorFormat`; little-endian
  40-byte DIB header reader. 4 unit tests.
- `icon_image.rs` — `PeIcon`, `IcoDirectoryEntry`, `IcoError` and
  `parse_ico_header` (ICONDIR + first entry + DIB header). 3 unit tests.
- `exe_info.rs` — `ExeInfo`, `ExeFileInfo` and the 7-key version mapping.
  3 unit tests.
- `REPORT.md` — cross-reference to this report.

Style follows the neighbouring `box64_config.rs` / `program_launcher.rs`:
plain `pub` structs/enums, `Vec<(String, String)>` for environment pairs,
`mod.rs` re-exports, `#[cfg(test)]` inline test modules, `std` only (no
dependencies, no Android/JNI).

## Reference sources studied (read-only)

All under
`reference/gamehub-lite-v518-decompiled/gamehub-lite-v518-decompiled/sources/`:

| File | What was taken |
| --- | --- |
| `com/winemu/core/GPUInfoQuery.java` | `getGPUVendor`, `getGPUDeviceName`, `getGPUDriverVersion`, `getGPUId`; `a()` — `"Adreno (TM)"` prefix strip + int parse (returns 0 on failure). |
| `com/xj/common/utils/GetGpuInfo.java` | EGL probe; `GL_VENDOR` 0x1F00 / `GL_RENDERER` 0x1F01 strings cached as `gpu_renderer`; unknown fallback. |
| `com/xj/winemu/data/repository/EnvLayerRepository$getGameConfigByScript$2.java` | `gpu_vendor`, `gpu_version`, `gpu_device_name`, `gpu_system_driver_version` field shapes. |
| `com/winemu/core/controller/EnvironmentController.java` | methods `e()` (driver replacement + RenderDoc), `f()` (DXVK), `s()` (zink/Mesa), `j()` (vkd3d); `WhenMappings` for `Config.InGameHud`. |
| `com/winemu/openapi/Config.java` | `GPUDriver(isCustomDriver, customPath)` bean; `GPUConfig`, `InGameHud` (NONE/FPS/FULL). |
| `com/winemu/openapi/DirectRenderingMode.java` | enum `Auto(0)`, `Never(1)`, `Always(2)`. |
| `com/xj/winemu/bean/NativeRendering.java`, `NativeRenderingMode.java` | `trans2NativeRenderingMode` mapping (Auto/Never/Always). |
| `com/winemu/core/DirectRendering.java`, `DirectRenderingStateListener.java` | `onDirectRenderingStateChanged(boolean)`, `XServer.setRenderingEnabled(z3 ^ true)`, listener `a(boolean)`. |
| `com/winemu/openapi/WinUIBridge.java` | listener uses `setRenderingEnabled(!z3)`. |
| `com/winemu/core/controller/GameScopeController.java` | 4-byte control block: version at bytes 0..2, toggle at byte 2, mode at byte 3. |
| `com/winemu/core/pe/BitmapInfoHeader.java` | all 11 DIB fields and the exact read order (DWORD/LONG/WORD), compression/origin/color-format enums and `fromValue` fallbacks. |
| `com/winemu/core/pe/IconImage.java` | `c(byte[], int)` → `width = header.g()`, `height = abs(header.d()/2)`, available length; `f() == 40` DIB branch. |
| `com/winemu/core/ExeFileInfo.java` | `Info` bean fields + the 7 string keys mapped in `b()`; `ExeFileInfo` fields (path, bitmap, `pe.i()`, VersionInfo). |
| `com/winemu/core/CabFile.java`, `FontHelper.java`, `ImageFsHelper.java` | read as context: native-only helpers (`libwinemu.so` CAB API), empty singleton, tar.xz/zst extraction — no portable driver/PE logic to port. |
| `in/dragonbra/javasteam/protobufs/steamclient/Enums.java` | `EGpuDriverId` numeric enum (0–19) reviewed but judged a protobuf wire type, not the Android GPU model; `GpuDriver` here is string-detection based. |

## Exact environment variable names found

From `EnvironmentController.java`, driver/graphics-related ones (all reproduced
as `pub const` in `gpu/driver.rs`):

| Name | Value / source | Where |
| --- | --- | --- |
| `WINEMU_REPLACED_DRIVER` | `GPUDriver.customPath` (only when not freedreno, path non-empty, `isDirectory()`) | `e()` line 195 |
| `GAMESCOPE_DRIVER_PATH` | custom path, or fallback `libvulkan_freedreno.so` when empty | `e()` line 189 |
| `TU_DEBUG` | `sysmem,noconform` | `e()` line 190 |
| `ENABLE_VULKAN_RENDERDOC_CAPTURE` | `1`/`0` | `e()` line 197 |
| `RENDERDOC_CAPTURE_PATH` | `/sdcard/gpu_captures` | `e()` line 206 |
| `DXVK_LOG_LEVEL` | `warn` | `f()` line 212 |
| `DXVK_ASYNC` | `1` | `f()` line 213 |
| `DXVK_STATE_CACHE_PATH` | `WinAPI.k(root)` | `f()` line 214 |
| `DXVK_HUD` | `fps,gpuload` (FPS) or the long `scale=0.8,...` string (FULL); absent for NONE | `f()` lines 218/223 |
| `DXVK_CONFIG_FILE` | `<root>/dxvk.conf` (only when maxDeviceMemory set) | `f()` line 235 |
| `ZINK_DESCRIPTORS` | `lazy` | `s()` line 420 |
| `ZINK_DEBUG` | `compact` | `s()` line 421 |
| `MESA_SHADER_CACHE_DISABLE` | `false` | `s()` line 422 |
| `MESA_SHADER_CACHE_MAX_SIZE` | `512MB` | `s()` line 423 |
| `mesa_glthread` | `true` (lowercase key is intentional) | `s()` line 424 |
| `GALLIUM_DRIVER` | `zink` | `s()` line 425 |
| `VKD3D_DEBUG` | `err` | `j()` line 373 |
| `VKD3D_FEATURE_LEVEL` | `12_0` | `j()` line 374 |
| `VKD3D_SHADER_MODEL` | `6_6` | `j()` line 375 |

Other env names observed in `EnvironmentController` but belonging to other
agents' areas and **not** reimplemented here: `WINEMU_ROOT_FS`, `WINEMU_VFS`,
`WINEMU_CPU_AFFINITY`, `WINEMU_MEMORY_LIMIT`, `WINEMU_DISABLE_RESHADE` family,
`LC_*`, `HOME`, `USER`, `DISPLAY`, `TMPDIR`, `XDG_*`, `GSETTINGS_SCHEMA_DIR`,
`BOX64_*`, `GAMESCOPE_SURFACE_USING_BGRA`, `GAMESCOPE_CONTROL_PATH`, `MANGOHUD`,
`PULSE_*`, `ALSA_*`, `ANDROID_ALSA_SERVER`, `ANDROID_ASERVER_USE_SHM`,
`ANDROID_SYSVSHM_SERVER`, `WM_SERVER_PATH`, `PERF_SERVER_PATH`, `RDC_SERVER_PATH`,
`SteamGameId`, `WINEPREFIX`, `WINEPREFIX_BASE`, `WINEUSERNAME`, `WINEDEBUG`,
`WINELOADERNOEXEC`, `WINE_DISABLE_FULLSCREEN_HACK`,
`PROTON_DISABLE_LSTEAMCLIENT`, `GST_PLUGIN_PATH`, `WINEESYNC`,
`MEDIACONV_BLANK_*`, `XCURSOR_*`.

`Config.GPUDriver.customPath` empty + `isCustomDriver=true` also implies the
literal fallback library name `libvulkan_freedreno.so` (source line 187).

## Out of scope

- Real GPU/EGL/Vulkan queries: `libgpuinfo.so` (`GPUInfoQuery` natives),
  `EGL14`/`GLES20.glGetString`, `GetGpuInfo`'s EGL context setup, the
  `/sys/class/kgsl/kgsl-3d0/gpubusy` Adreno profile probe.
- Driver installation/management and actual filesystem checks
  (`new File(path).isDirectory()`, CAB extraction, tar.xz/zst image extraction in
  `CabFile`/`ImageFsHelper`). The directory check is passed in as a boolean.
- Real process environment mutation (`EnvVars.c(...)`); env pairs are returned
  as data.
- Android surfaces / native renderer: `SurfaceControl`, `Surface`,
  `XServer.setRenderingEnabled`, `libwinemu.so` natives (`nativeInitialize`,
  `nativeSetSurfaceFormat`, `nativeStartTestClient`), listener registration.
- Real PE/COFF parsing (pecoff4j `PEParser`, `ResourceHelper`, `VersionInfo`,
  `StringTable`), RT_GROUP_ICON/RT_ICON resource walking.
- Image decoding: `IconImage.b()/d()`, color quads, `BitmapFactory`,
  `Bitmap.createBitmap`, PNG decode. Only the 40-byte `BITMAPINFOHEADER` and the
  small ICO container header are read; pixel bytes are ignored.
- `Config` Parcelable IPC, Kotlin coroutines, `FontHelper` (empty singleton).

## Verification

This agent cannot edit `Cargo.toml`, `app/src/lib.rs` or the parent
`com/winemu/core/mod.rs`; wiring the two new `pub mod gpu;` / `pub mod pe;`
declarations belongs to the parent agent. Once wired, compile and run with:

```
cd /home/Project/Zevryn/app && cargo test
```

26 unit tests are included across the six Rust files (10 driver + 6 rendering +
4 header + 3 ICO + 3 EXE). A temporary external harness compiled all modules
with `rustc --test` and reported 26/26 passing.
