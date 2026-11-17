//! Rust port of the portable GPU / direct-rendering logic of
//! `com.winemu.core` (GameHub Lite v5.1.8).
//!
//! * [`driver`] — `GpuDriver` / `GpuInfo` from `GPUInfoQuery`, plus the
//!   driver-replacement and DXVK/zink/vkd3d environment shaping from
//!   `EnvironmentController`.
//! * [`direct_rendering`] — `DirectRenderingState` (Auto/Never/Always), the
//!   direct-rendering decision rule and the X-server inversion event.
//!
//! Real GPU/EGL queries, `libgpuinfo.so`, `libwinemu.so`, Android surfaces and
//! image decoding are intentionally not reimplemented.

pub mod direct_rendering;
pub mod driver;

pub use self::direct_rendering::{
    DirectRenderingController, DirectRenderingEvent, DirectRenderingState, NativeRenderingMode,
};
pub use self::driver::{
    all_gpu_env, driver_replacement_env, dxvk_env, mesa_zink_env, renderdoc_env, vkd3d_env,
    DriverEnvError, GpuDriver, GpuDriverConfig, GpuInfo, HudLevel,
};
pub use self::driver::{
    ENV_DXVK_ASYNC, ENV_DXVK_CONFIG_FILE, ENV_DXVK_HUD, ENV_DXVK_LOG_LEVEL,
    ENV_DXVK_STATE_CACHE_PATH, ENV_ENABLE_VULKAN_RENDERDOC_CAPTURE, ENV_GALLIUM_DRIVER,
    ENV_GAMESCOPE_DRIVER_PATH, ENV_MESA_GLTHREAD, ENV_MESA_SHADER_CACHE_DISABLE,
    ENV_MESA_SHADER_CACHE_MAX_SIZE, ENV_RENDERDOC_CAPTURE_PATH, ENV_TU_DEBUG,
    ENV_VKD3D_DEBUG, ENV_VKD3D_FEATURE_LEVEL, ENV_VKD3D_SHADER_MODEL,
    ENV_WINEMU_REPLACED_DRIVER, ENV_ZINK_DEBUG, ENV_ZINK_DESCRIPTORS,
    DXVK_HUD_FULL, DXVK_HUD_FPS, FREEDRENO_VULKAN_LIBRARY,
};
