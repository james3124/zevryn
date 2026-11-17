//! Rust model of the portable `com.winemu.core` logic (GameHub Lite v5.1.8).
//!
//! Mirrors `sources/com/winemu/core/` from the reference. The native emulation
//! runtime (`libwinemu.so`, `libxserver.so`, `libvfs.so`), the Wine/X server,
//! actual process launch and Android IPC are intentionally out of scope; only
//! the dependency-free data model and command/env shaping stay here. Layout
//! follows `PLAN.md`: a `mod.rs` package root with sibling `.rs` files for each
//! modeled Java class.

pub mod boot_data;
pub mod box64_config;
pub mod container;
pub mod program_launcher;
pub mod tricks;
pub mod wine;

// Driver/server layer (added by follow-up agents).
pub mod gpu;
pub mod input;
pub mod pe;
pub mod regedit;
pub mod server;
pub mod utils;
pub mod gamepad_server;

pub use boot_data::BootData;
pub use box64_config::Box64Config;
pub use container::{BottleMetadata, Container};
pub use program_launcher::{exec_argv, inspect_elf, ElfMachine, ENV_KEYS, DEFAULT_BOX64, LINKER64};
pub use tricks::{ConfigOverrides, Tricks};
pub use wine::{Wine, WineArch};
