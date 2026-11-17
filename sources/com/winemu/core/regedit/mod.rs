//! Dependency-free Rust model of `com.winemu.core.regedit` (GameHub Lite
//! v5.1.8).
//!
//! This package mirrors the portable parts of the decompiled package:
//!
//! * [`value::RegValue`] — value payloads from `RegistryValue` (String, Dword,
//!   Qword, Binary, MultiString, None);
//! * [`key::RegKey`] / [`key::RegHive`] — the `RegistryKey` tree and an
//!   in-memory hive with insert / lookup / delete;
//! * [`patch::RegPatch`] / [`patch::RegChange`] — `RegistryPatcher` and
//!   `RegistryComparator` change models, with deterministic ordering and a
//!   [`patch::RegResult`] report from `apply()`;
//! * [`parser::parse_reg`] / [`parser::write_reg`] — a minimal `.reg` text
//!   parser and serializer (`RegistryParser` / the text half of
//!   `RegistryWriter`);
//! * [`architecture::Architecture`] — the `Architecture` enum.
//!
//! `RegistryEditor`, `RegistryResult` and the `RegistryDsl*` family are thin
//! wrappers over the parser/writer/tree; their portable behavior is exposed
//! directly here rather than as separate pass-through types. Real Windows
//! registry access and JNI/Android integration are intentionally out of scope.
//!
//! Only `std` is used. Wire into the crate with:
//!
//! ```ignore
//! #[path = "../sources/com/winemu/core/regedit/mod.rs"]
//! pub mod regedit;
//! ```

pub mod architecture;
pub mod key;
pub mod parser;
pub mod patch;
pub mod value;

pub use architecture::Architecture;
pub use key::{canonical_path, split_reg_path, RegHive, RegKey};
pub use parser::{parse_hive, parse_reg, write_reg, RegLoadResult, RegParseError};
pub use patch::{
    KeyPropertyChange, PatchFailure, PatchOptions, RegChange, RegPatch, RegResult,
};
pub use value::RegValue;
