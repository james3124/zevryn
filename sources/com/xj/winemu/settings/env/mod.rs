//! Rust model of the portable environment-variable / exe-path logic of
//! `com.xj.winemu.settings` (GameHub Lite v5.1.8).
//!
//! Mirrors `InputEnvParamsDialog.java`, `NewInputEnvParamsDialog.java`,
//! `EditType.java` and the `PcGameSettingDataHelper.R` / `V1` helpers, plus
//! `GameSettingViewModel.getLocalPcExePath` / `updateLocalPcExePath`.
//!
//! Dialogs, focus handling and repository I/O are intentionally out of scope;
//! only the data model and its pure transformations live here — see
//! `REPORT.md`.

pub mod exe_path;
pub mod params;

pub use self::exe_path::{ExePathError, ExePathUpdate, LocalExePath};
pub use self::params::{validate_env_string, EnvParam, EnvParams, EnvParseError, EditType};
