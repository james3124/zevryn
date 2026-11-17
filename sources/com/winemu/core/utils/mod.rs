//! Dependency-free Rust models of the portable `com.winemu.core.utils` classes
//! (GameHub Lite v5.1.8).
//!
//! Only pure data shapes and string/math transforms are kept. Android IPC,
//! real process spawn, sockets, filesystem traversal, native libraries and
//! compression are intentionally out of scope (see `REPORT.md`).
//!
//! Wire into the parent `com/winemu/core/mod.rs` with:
//! ```text
//! pub mod utils;
//! ```

mod battery;
mod cpu_info;
mod env_vars;
mod net;
mod port;
mod program;
mod tar_layout;
mod win_path;

pub use battery::{is_charging, normalize_current, voltage_from_extra};
pub use cpu_info::{
    cpu_affinity, core_count_from_names, is_cpu_dir_name, set_cpu_affinity, sysfs,
    Affinity, AllCoreFrequencyInfo, WINEMU_CPU_AFFINITY,
};
pub use env_vars::EnvVars;
pub use net::{ipv4_from_int, netmask_from_prefix};
pub use port::{
    find_free_port, PortRange, DEFAULT_PORT_END, DEFAULT_PORT_START,
};
pub use program::{
    split_command_line, Program, ProgramEntry,
};
pub use tar_layout::{
    CompressionType, TarEntry, TarEntryKind, TarLayout,
};
pub use win_path::{
    is_absolute, join, join_windows, strip_trailing_separators, to_unix_path,
    to_windows_path,
};
