//! Portable, dependency-free (std only) model of the GameHub Lite v5.1.8
//! `com.winemu.core.server` portable logic.
//!
//! This module models **only** data/protocol/control logic that is portable to
//! Rust. Sockets, unix sockets, epoll, native `winemu` JNI calls, Android
//! services and real IPC are explicitly out of scope (see REPORT.md).
//!
//! Submodules:
//! - [`request_codes`] - numeric command codes per subsystem.
//! - [`server_message`] - code + payload message model.
//! - [`plugin`] - plugin lifecycle state and ordered startup list.
//! - [`winmonitor`] - WinMonitor data beans.

pub mod plugin;
pub mod request_codes;
pub mod server_message;
pub mod winmonitor;

// ---- Re-exports -----------------------------------------------------------

pub use plugin::{
    PluginKind, PluginState, ProgramPlugin, ALL_PLUGIN_KINDS, PLUGIN_STARTUP_ORDER,
    SOCKET_PATH_RENDERDOC, SOCKET_PATH_SOUND_ALSA, SOCKET_PATH_SOUND_PULSE, SOCKET_PATH_PERF,
    SOCKET_PATH_SYSV, SOCKET_PATH_WM,
};
pub use request_codes::{
    AlsaRequestCode, PerfRequestCode, RenderDocRequestCode, SysvRequestCode, WmRequestCode,
    WinMonitorRequestCode,
};
pub use server_message::ServerMessage;
pub use winmonitor::{
    CommandRequest, KillProcessResponse, ProcessInfo, ProcessListResponse, StackFrame,
    ThreadDetailInfo, ThreadInfo, ThreadListResponse, ThreadStackResponse,
};
