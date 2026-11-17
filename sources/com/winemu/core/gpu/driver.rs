//! Rust model of the portable GPU/driver pieces of GameHub Lite v5.1.8.
//!
//! Sources mirrored here:
//! * `sources/com/winemu/core/GPUInfoQuery.java` — native `gpuinfo` queries
//!   (`getGPUVendor`, `getGPUDeviceName`, `getGPUDriverVersion`, `getGPUId`) plus
//!   the pure `a()` helper that extracts the Adreno model number from
//!   `"Adreno (TM) <n>"`.
//! * `sources/com/xj/common/utils/GetGpuInfo.java` — EGL/GLES probe that reads
//!   `GL_VENDOR` (0x1F00) / `GL_RENDERER` (0x1F01) strings; the `gpu_renderer`
//!   SharedPreferences cache.
//! * `sources/com/xj/winemu/data/repository/EnvLayerRepository$getGameConfigByScript$2.java`
//!   — the vendor / version / device / driver-version fields reported upstream.
//! * `sources/com/winemu/core/controller/EnvironmentController.java` — methods
//!   `e()`, `f()`, `s()` and `j()` that shape the driver-related environment.
//!
//! The native `libgpuinfo.so` query itself, EGL context creation, filesystem
//! directory checks and actual process environment mutation are out of scope.
//! Here the environment is returned as `Vec<(String, String)>` pairs, and the
//! "is this a directory" check is supplied by the caller as a boolean.

use std::fmt;

// ---------------------------------------------------------------------------
// Exact environment variable names found in EnvironmentController.java
// ---------------------------------------------------------------------------

/// `EnvironmentController.e()` — set from `Config.GPUDriver.customPath`.
pub const ENV_WINEMU_REPLACED_DRIVER: &str = "WINEMU_REPLACED_DRIVER";
/// `EnvironmentController.e()` — gamescope Vulkan driver (custom or fallback).
pub const ENV_GAMESCOPE_DRIVER_PATH: &str = "GAMESCOPE_DRIVER_PATH";
/// `EnvironmentController.e()` — set alongside the freedreno fallback driver.
pub const ENV_TU_DEBUG: &str = "TU_DEBUG";
/// `EnvironmentController.s()` — zink descriptor mode.
pub const ENV_ZINK_DESCRIPTORS: &str = "ZINK_DESCRIPTORS";
/// `EnvironmentController.s()` — zink debug flags.
pub const ENV_ZINK_DEBUG: &str = "ZINK_DEBUG";
/// `EnvironmentController.s()` — Mesa gallium driver selection.
pub const ENV_GALLIUM_DRIVER: &str = "GALLIUM_DRIVER";
/// `EnvironmentController.s()` — Mesa shader cache toggle (string `"false"`).
pub const ENV_MESA_SHADER_CACHE_DISABLE: &str = "MESA_SHADER_CACHE_DISABLE";
/// `EnvironmentController.s()` — Mesa shader cache size.
pub const ENV_MESA_SHADER_CACHE_MAX_SIZE: &str = "MESA_SHADER_CACHE_MAX_SIZE";
/// `EnvironmentController.s()` — note the deliberately lowercase key.
pub const ENV_MESA_GLTHREAD: &str = "mesa_glthread";
/// `EnvironmentController.f()` — DXVK log level.
pub const ENV_DXVK_LOG_LEVEL: &str = "DXVK_LOG_LEVEL";
/// `EnvironmentController.f()` — DXVK async pipeline compilation.
pub const ENV_DXVK_ASYNC: &str = "DXVK_ASYNC";
/// `EnvironmentController.f()` — DXVK pipeline cache location.
pub const ENV_DXVK_STATE_CACHE_PATH: &str = "DXVK_STATE_CACHE_PATH";
/// `EnvironmentController.f()` — DXVK HUD string, dependent on `InGameHud`.
pub const ENV_DXVK_HUD: &str = "DXVK_HUD";
/// `EnvironmentController.f()` — generated `dxvk.conf` location.
pub const ENV_DXVK_CONFIG_FILE: &str = "DXVK_CONFIG_FILE";
/// `EnvironmentController.j()` — vkd3d-proton debug level.
pub const ENV_VKD3D_DEBUG: &str = "VKD3D_DEBUG";
/// `EnvironmentController.j()` — vkd3d-proton feature level.
pub const ENV_VKD3D_FEATURE_LEVEL: &str = "VKD3D_FEATURE_LEVEL";
/// `EnvironmentController.j()` — vkd3d-proton shader model.
pub const ENV_VKD3D_SHADER_MODEL: &str = "VKD3D_SHADER_MODEL";
/// `EnvironmentController.e()` — RenderDoc Vulkan capture toggle.
pub const ENV_ENABLE_VULKAN_RENDERDOC_CAPTURE: &str = "ENABLE_VULKAN_RENDERDOC_CAPTURE";
/// `EnvironmentController.e()` — RenderDoc capture output directory.
pub const ENV_RENDERDOC_CAPTURE_PATH: &str = "RENDERDOC_CAPTURE_PATH";

/// Fallback Vulkan driver used by `EnvironmentController.e()` when a custom
/// driver is requested but its path string is empty.
pub const FREEDRENO_VULKAN_LIBRARY: &str = "libvulkan_freedreno.so";

/// HUD strings assigned to `DXVK_HUD` by `EnvironmentController.f()`, keyed off
/// `Config.InGameHud` (`NONE`, `FPS`, `FULL`).
pub const DXVK_HUD_FPS: &str = "fps,gpuload";
pub const DXVK_HUD_FULL: &str = "scale=0.8,version,fps,api,devinfo,gpuload,frametimes,submissions,drawcalls,pipelines,descriptors,memory,allocations,cs,compiler";

// ---------------------------------------------------------------------------
// GpuInfo / GpuDriver
// ---------------------------------------------------------------------------

/// GPU vendor / driver family. Detected from the strings exposed by
/// `GPUInfoQuery.getGPUVendor()` and `getGPUDeviceName()` (and by the GLES
/// `GL_RENDERER` string cached in `GetGpuInfo`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum GpuDriver {
    /// Qualcomm Adreno ("Adreno (TM)" in `GPUInfoQuery.a()`).
    Adreno,
    /// ARM Mali.
    Mali,
    /// Imagination PowerVR.
    PowerVr,
    /// NVIDIA.
    Nvidia,
    /// AMD / Radeon.
    Amd,
    /// Intel.
    Intel,
    /// Generic Mesa software / open-source driver (llvmpipe, softpipe, ...).
    Mesa,
    /// Google SwiftShader software rasterizer.
    SwiftShader,
    /// Broadcom VideoCore.
    Broadcom,
    /// Nothing identifiable in the reported strings.
    Unknown,
}

impl GpuDriver {
    /// Detection from `GPUInfoQuery.getGPUDeviceName()` / the GLES renderer
    /// string. Case-insensitive substring match, mirroring the string based
    /// heuristics used elsewhere in the reference sources.
    pub fn from_device_name(name: &str) -> Self {
        let n = name.to_ascii_lowercase();
        if n.contains("adreno") {
            GpuDriver::Adreno
        } else if n.contains("mali") {
            GpuDriver::Mali
        } else if n.contains("powervr") || n.contains("imagination") {
            GpuDriver::PowerVr
        } else if n.contains("swiftshader") {
            GpuDriver::SwiftShader
        } else if n.contains("llvmpipe") || n.contains("softpipe") || n.contains("mesa") {
            GpuDriver::Mesa
        } else if n.contains("nvidia") || n.contains("geforce") {
            GpuDriver::Nvidia
        } else if n.contains("radeon") || n.contains("amd") {
            GpuDriver::Amd
        } else if n.contains("intel") {
            GpuDriver::Intel
        } else if n.contains("broadcom") || n.contains("videocore") {
            GpuDriver::Broadcom
        } else {
            GpuDriver::Unknown
        }
    }

    /// Detection from `GPUInfoQuery.getGPUVendor()` / the GLES vendor string.
    pub fn from_vendor(vendor: &str) -> Self {
        let v = vendor.to_ascii_lowercase();
        if v.contains("qualcomm") {
            GpuDriver::Adreno
        } else if v.contains("arm") {
            GpuDriver::Mali
        } else if v.contains("imagination") {
            GpuDriver::PowerVr
        } else if v.contains("nvidia") {
            GpuDriver::Nvidia
        } else if v.contains("amd") || v.contains("ati") {
            GpuDriver::Amd
        } else if v.contains("intel") {
            GpuDriver::Intel
        } else if v.contains("google") && v.contains("swiftshader") {
            GpuDriver::SwiftShader
        } else if v.contains("mesa") || v.contains("broadcom") {
            GpuDriver::Mesa
        } else {
            GpuDriver::Unknown
        }
    }

    /// Prefer the device name, then the vendor string; `Mesa` named in the
    /// vendor but `Unknown` device stays Mesa rather than collapsing to
    /// `Unknown`.
    pub fn detect(info: &GpuInfo) -> Self {
        let by_name = Self::from_device_name(&info.renderer);
        if by_name != GpuDriver::Unknown {
            return by_name;
        }
        let by_vendor = Self::from_vendor(&info.vendor);
        if by_vendor != GpuDriver::Unknown {
            return by_vendor;
        }
        Self::from_device_name(&info.device_name)
    }

    pub fn as_str(self) -> &'static str {
        match self {
            GpuDriver::Adreno => "Adreno",
            GpuDriver::Mali => "Mali",
            GpuDriver::PowerVr => "PowerVR",
            GpuDriver::Nvidia => "NVIDIA",
            GpuDriver::Amd => "AMD",
            GpuDriver::Intel => "Intel",
            GpuDriver::Mesa => "Mesa",
            GpuDriver::SwiftShader => "SwiftShader",
            GpuDriver::Broadcom => "Broadcom",
            GpuDriver::Unknown => "Unknown",
        }
    }
}

impl fmt::Display for GpuDriver {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

/// Snapshot of the GPU identity exposed by `GPUInfoQuery` and the GLES probe.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct GpuInfo {
    /// `GPUInfoQuery.getGPUVendor()` / `GL_VENDOR` (0x1F00).
    pub vendor: String,
    /// `GPUInfoQuery.getGPUDeviceName()` / `GL_RENDERER` (0x1F01).
    pub renderer: String,
    /// OpenGL / Vulkan version string.
    pub version: String,
    /// Space-separated extension names, split from `GL_EXTENSIONS`.
    pub extensions: Vec<String>,
    /// `GPUInfoQuery.getGPUDeviceName()` (kept separately so a GLES-only
    /// snapshot can still be classified).
    pub device_name: String,
    /// `GPUInfoQuery.getGPUDriverVersion()` (a packed 32-bit value upstream).
    pub driver_version: i32,
    /// `GPUInfoQuery.getGPUId()`.
    pub gpu_id: i32,
}

impl GpuInfo {
    /// Build a snapshot from the four strings the task calls out, splitting the
    /// extension blob on ASCII whitespace like a `GL_EXTENSIONS` string.
    pub fn from_gl_strings(
        vendor: &str,
        renderer: &str,
        version: &str,
        extensions_blob: &str,
    ) -> Self {
        Self {
            vendor: vendor.to_owned(),
            renderer: renderer.to_owned(),
            version: version.to_owned(),
            extensions: extensions_blob
                .split_whitespace()
                .map(str::to_owned)
                .collect(),
            device_name: renderer.to_owned(),
            driver_version: 0,
            gpu_id: 0,
        }
    }

    /// Port of `GPUInfoQuery.a()`: only a device name starting with
    /// `"Adreno (TM)"` is considered, the prefix is removed, the remainder is
    /// trimmed and parsed as an integer. Returns `None` where Java returns `0`.
    pub fn adreno_version(&self) -> Option<i32> {
        const PREFIX: &str = "Adreno (TM)";
        if !self.device_name.starts_with(PREFIX) {
            return None;
        }
        let suffix = self.device_name[PREFIX.len()..].trim();
        suffix.parse::<i32>().ok()
    }

    pub fn driver(&self) -> GpuDriver {
        GpuDriver::detect(self)
    }
}

// ---------------------------------------------------------------------------
// Driver replacement configuration (Config.GPUDriver)
// ---------------------------------------------------------------------------

/// Rust projection of `Config.GPUDriver(boolean isCustomDriver, String customPath)`:
/// `use_freedreno` is the Java `b()` / `isCustomDriver` flag and `custom_path`
/// is Java `a()` / `customPath`.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct GpuDriverConfig {
    pub use_freedreno: bool,
    pub custom_path: String,
}

impl GpuDriverConfig {
    /// `new GPUDriver(false, "")` — stock driver, no replacement.
    pub fn standard() -> Self {
        Self { use_freedreno: false, custom_path: String::new() }
    }

    /// `new GPUDriver(true, path)` — route through gamescope/freedreno.
    pub fn freedreno(path: impl Into<String>) -> Self {
        Self { use_freedreno: true, custom_path: path.into() }
    }

    /// `new GPUDriver(false, path)` — replace via `WINEMU_REPLACED_DRIVER`.
    pub fn custom(path: impl Into<String>) -> Self {
        Self { use_freedreno: false, custom_path: path.into() }
    }
}

/// Mirrors the `IOException("Illegal driver dir!")` guard in
/// `EnvironmentController.e()` — the caller supplies the filesystem result.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DriverEnvError {
    IllegalDriverDir,
}

impl fmt::Display for DriverEnvError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("Illegal driver dir!")
    }
}

impl std::error::Error for DriverEnvError {}

/// Port of `EnvironmentController.e()`'s driver selection. `custom_path_is_dir`
/// stands in for `new File(path).isDirectory()`.
pub fn driver_replacement_env(
    config: &GpuDriverConfig,
    custom_path_is_dir: bool,
) -> Result<Vec<(String, String)>, DriverEnvError> {
    let mut env = Vec::new();

    if config.use_freedreno {
        let mut path = config.custom_path.clone();
        if path.is_empty() {
            path = FREEDRENO_VULKAN_LIBRARY.to_owned();
        }
        env.push((ENV_GAMESCOPE_DRIVER_PATH.to_owned(), path));
        env.push((ENV_TU_DEBUG.to_owned(), "sysmem,noconform".to_owned()));
    } else if !config.custom_path.is_empty() {
        if !custom_path_is_dir {
            return Err(DriverEnvError::IllegalDriverDir);
        }
        env.push((
            ENV_WINEMU_REPLACED_DRIVER.to_owned(),
            config.custom_path.clone(),
        ));
    }

    Ok(env)
}

/// RenderDoc part of `EnvironmentController.e()`.
pub fn renderdoc_env(enabled: bool, capture_path: &str) -> Vec<(String, String)> {
    let mut env = vec![(
        ENV_ENABLE_VULKAN_RENDERDOC_CAPTURE.to_owned(),
        if enabled { "1" } else { "0" }.to_owned(),
    )];
    if enabled {
        // Java creates `/sdcard/gpu_captures` when capture is enabled; the
        // caller supplies the resulting directory string.
        env.push((ENV_RENDERDOC_CAPTURE_PATH.to_owned(), capture_path.to_owned()));
    }
    env
}

/// `Config.InGameHud` values used by `EnvironmentController.f()`'s `WhenMappings`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HudLevel {
    /// `InGameHud.NONE` — no `DXVK_HUD` variable is written.
    None,
    /// `InGameHud.FPS` — `fps,gpuload`.
    Fps,
    /// `InGameHud.FULL` — the long overlay string.
    Full,
}

impl HudLevel {
    pub fn from_value(value: i32) -> Self {
        match value {
            1 => HudLevel::Fps,
            2 => HudLevel::Full,
            _ => HudLevel::None,
        }
    }

    pub fn value(self) -> i32 {
        match self {
            HudLevel::None => 0,
            HudLevel::Fps => 1,
            HudLevel::Full => 2,
        }
    }
}

/// Port of `EnvironmentController.f()`. The `dxvk.conf` file contents itself
/// (`dxgi.maxDeviceMemory` / `dxgi.maxSharedMemory`) is filesystem work and is
/// out of scope; pass the resolved config path to have `DXVK_CONFIG_FILE` set.
pub fn dxvk_env(
    hud: HudLevel,
    state_cache_path: &str,
    config_file: Option<&str>,
) -> Vec<(String, String)> {
    let mut env = vec![
        (ENV_DXVK_LOG_LEVEL.to_owned(), "warn".to_owned()),
        (ENV_DXVK_ASYNC.to_owned(), "1".to_owned()),
        (ENV_DXVK_STATE_CACHE_PATH.to_owned(), state_cache_path.to_owned()),
    ];
    match hud {
        HudLevel::None => {}
        HudLevel::Fps => env.push((ENV_DXVK_HUD.to_owned(), DXVK_HUD_FPS.to_owned())),
        HudLevel::Full => env.push((ENV_DXVK_HUD.to_owned(), DXVK_HUD_FULL.to_owned())),
    }
    if let Some(path) = config_file {
        env.push((ENV_DXVK_CONFIG_FILE.to_owned(), format!("{path}/dxvk.conf")));
    }
    env
}

/// Port of `EnvironmentController.s()` — fixed zink/Mesa environment.
pub fn mesa_zink_env() -> Vec<(String, String)> {
    vec![
        (ENV_ZINK_DESCRIPTORS.to_owned(), "lazy".to_owned()),
        (ENV_ZINK_DEBUG.to_owned(), "compact".to_owned()),
        (ENV_MESA_SHADER_CACHE_DISABLE.to_owned(), "false".to_owned()),
        (ENV_MESA_SHADER_CACHE_MAX_SIZE.to_owned(), "512MB".to_owned()),
        (ENV_MESA_GLTHREAD.to_owned(), "true".to_owned()),
        (ENV_GALLIUM_DRIVER.to_owned(), "zink".to_owned()),
    ]
}

/// Port of `EnvironmentController.j()` — fixed vkd3d-proton environment.
pub fn vkd3d_env() -> Vec<(String, String)> {
    vec![
        (ENV_VKD3D_DEBUG.to_owned(), "err".to_owned()),
        (ENV_VKD3D_FEATURE_LEVEL.to_owned(), "12_0".to_owned()),
        (ENV_VKD3D_SHADER_MODEL.to_owned(), "6_6".to_owned()),
    ]
}

/// All environment pairs this module models, in the order the
/// `EnvironmentController.g()` pipeline would add them.
pub fn all_gpu_env(
    config: &GpuDriverConfig,
    custom_path_is_dir: bool,
    hud: HudLevel,
    state_cache_path: &str,
    config_file_dir: Option<&str>,
) -> Result<Vec<(String, String)>, DriverEnvError> {
    let mut env = driver_replacement_env(config, custom_path_is_dir)?;
    env.extend(dxvk_env(hud, state_cache_path, config_file_dir));
    env.extend(mesa_zink_env());
    env.extend(vkd3d_env());
    Ok(env)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn adreno_version_parses_the_decompiled_prefix() {
        let info = GpuInfo {
            device_name: "Adreno (TM) 640".to_owned(),
            ..GpuInfo::default()
        };
        assert_eq!(info.adreno_version(), Some(640));
        assert_eq!(info.driver(), GpuDriver::Adreno);
    }

    #[test]
    fn adreno_version_rejects_other_names_and_bad_suffixes() {
        let mut info = GpuInfo::default();
        assert_eq!(info.adreno_version(), None);

        info.device_name = "adreno (tm) 640".to_owned(); // case-sensitive prefix
        assert_eq!(info.adreno_version(), None);

        info.device_name = "Adreno (TM) foo".to_owned();
        assert_eq!(info.adreno_version(), None);
    }

    #[test]
    fn driver_detection_prefers_renderer_then_vendor() {
        let info = GpuInfo::from_gl_strings(
            "Qualcomm",
            "Adreno (TM) 740",
            "OpenGL ES 3.2",
            "GL_EXT_foo GL_EXT_bar",
        );
        assert_eq!(info.driver(), GpuDriver::Adreno);
        assert_eq!(info.extensions, vec!["GL_EXT_foo", "GL_EXT_bar"]);

        let mali = GpuInfo::from_gl_strings("ARM", "Mali-G78", "v1", "");
        assert_eq!(mali.driver(), GpuDriver::Mali);

        let sw = GpuInfo::from_gl_strings("Google Inc.", "SwiftShader", "v1", "");
        assert_eq!(sw.driver(), GpuDriver::SwiftShader);
    }

    #[test]
    fn driver_env_custom_replacement_uses_winemu_replaced_driver() {
        let env = driver_replacement_env(&GpuDriverConfig::custom("/sdcard/driver"), true).unwrap();
        assert_eq!(
            env,
            vec![(
                ENV_WINEMU_REPLACED_DRIVER.to_owned(),
                "/sdcard/driver".to_owned()
            )]
        );
        assert!(env.iter().all(|(k, _)| k != ENV_GAMESCOPE_DRIVER_PATH));
    }

    #[test]
    fn driver_env_rejects_non_directory_custom_path() {
        let err = driver_replacement_env(&GpuDriverConfig::custom("/not/a/dir"), false).unwrap_err();
        assert_eq!(err, DriverEnvError::IllegalDriverDir);
        assert_eq!(err.to_string(), "Illegal driver dir!");
    }

    #[test]
    fn driver_env_freedreno_falls_back_to_freedreno_library() {
        let env = driver_replacement_env(&GpuDriverConfig::freedreno(""), true).unwrap();
        assert_eq!(
            env,
            vec![
                (
                    ENV_GAMESCOPE_DRIVER_PATH.to_owned(),
                    FREEDRENO_VULKAN_LIBRARY.to_owned()
                ),
                (ENV_TU_DEBUG.to_owned(), "sysmem,noconform".to_owned()),
            ]
        );
    }

    #[test]
    fn driver_env_standard_driver_sets_nothing() {
        let env = driver_replacement_env(&GpuDriverConfig::standard(), false).unwrap();
        assert!(env.is_empty());
    }

    #[test]
    fn zink_and_vkd3d_env_match_the_decompiled_literals() {
        let env = mesa_zink_env();
        assert_eq!(env.len(), 6);
        assert!(env.contains(&(ENV_ZINK_DESCRIPTORS.to_owned(), "lazy".to_owned())));
        assert!(env.contains(&(ENV_ZINK_DEBUG.to_owned(), "compact".to_owned())));
        assert!(env.contains(&(ENV_GALLIUM_DRIVER.to_owned(), "zink".to_owned())));
        assert!(env.contains(&(ENV_MESA_GLTHREAD.to_owned(), "true".to_owned())));

        assert_eq!(
            vkd3d_env(),
            vec![
                (ENV_VKD3D_DEBUG.to_owned(), "err".to_owned()),
                (ENV_VKD3D_FEATURE_LEVEL.to_owned(), "12_0".to_owned()),
                (ENV_VKD3D_SHADER_MODEL.to_owned(), "6_6".to_owned()),
            ]
        );
    }

    #[test]
    fn dxvk_hud_levels_and_config_file() {
        let none = dxvk_env(HudLevel::None, "/cache", None);
        assert!(!none.iter().any(|(k, _)| k == ENV_DXVK_HUD));
        assert!(none.contains(&(ENV_DXVK_LOG_LEVEL.to_owned(), "warn".to_owned())));
        assert!(none.contains(&(ENV_DXVK_ASYNC.to_owned(), "1".to_owned())));

        let fps = dxvk_env(HudLevel::from_value(1), "/cache", None);
        assert!(fps.contains(&(ENV_DXVK_HUD.to_owned(), DXVK_HUD_FPS.to_owned())));

        let full = dxvk_env(HudLevel::from_value(2), "/cache", Some("/prefix"));
        assert!(full.contains(&(ENV_DXVK_HUD.to_owned(), DXVK_HUD_FULL.to_owned())));
        assert!(full.contains(&(
            ENV_DXVK_CONFIG_FILE.to_owned(),
            "/prefix/dxvk.conf".to_owned()
        )));
    }

    #[test]
    fn all_gpu_env_composition_and_renderdoc() {
        let env = all_gpu_env(
            &GpuDriverConfig::freedreno(""),
            true,
            HudLevel::Fps,
            "/cache",
            None,
        )
        .unwrap();
        assert!(env.contains(&(ENV_GAMESCOPE_DRIVER_PATH.to_owned(), FREEDRENO_VULKAN_LIBRARY.to_owned())));
        assert!(env.contains(&(ENV_DXVK_HUD.to_owned(), DXVK_HUD_FPS.to_owned())));
        assert!(env.contains(&(ENV_ZINK_DESCRIPTORS.to_owned(), "lazy".to_owned())));
        assert!(env.contains(&(ENV_VKD3D_DEBUG.to_owned(), "err".to_owned())));

        let rdoc = renderdoc_env(true, "/sdcard/gpu_captures");
        assert_eq!(
            rdoc,
            vec![
                (ENV_ENABLE_VULKAN_RENDERDOC_CAPTURE.to_owned(), "1".to_owned()),
                (ENV_RENDERDOC_CAPTURE_PATH.to_owned(), "/sdcard/gpu_captures".to_owned()),
            ]
        );
        assert_eq!(
            renderdoc_env(false, "/sdcard/gpu_captures"),
            vec![(ENV_ENABLE_VULKAN_RENDERDOC_CAPTURE.to_owned(), "0".to_owned())]
        );
    }
}
