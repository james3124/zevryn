//! Portable model of `com.xj.common.download.bean.CommonDownloadTaskState`.
//!
//! A Kotlin enum with an integer `type` code used in download DTOs
//! (`commonDownloadState`, `GameDetailDownloadInfo`). Codes are taken verbatim
//! from the decompiled enum.

use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CommonDownloadTaskState {
    Normal,            // 0
    Waiting,           // 1
    Downloading,       // 2
    Paused,            // 3
    Completed,         // 4
    Fail,              // 5
    DownloadingConfig, // 6
    Cancel,            // 7
    VerifyingFiles,    // 8
    MergingFiles,      // 9
    Pending,           // 10
}

impl CommonDownloadTaskState {
    pub const ALL: [Self; 11] = [
        Self::Normal,
        Self::Waiting,
        Self::Downloading,
        Self::Paused,
        Self::Completed,
        Self::Fail,
        Self::DownloadingConfig,
        Self::Cancel,
        Self::VerifyingFiles,
        Self::MergingFiles,
        Self::Pending,
    ];

    /// The `getType()` value.
    pub fn code(self) -> i32 {
        match self {
            Self::Normal => 0,
            Self::Waiting => 1,
            Self::Downloading => 2,
            Self::Paused => 3,
            Self::Completed => 4,
            Self::Fail => 5,
            Self::DownloadingConfig => 6,
            Self::Cancel => 7,
            Self::VerifyingFiles => 8,
            Self::MergingFiles => 9,
            Self::Pending => 10,
        }
    }

    /// `values()[ordinal]`-style lookup. Unknown codes return `None`.
    pub fn from_code(code: i32) -> Option<Self> {
        Self::ALL.iter().copied().find(|s| s.code() == code)
    }

    /// Whether a state means the download has stopped/finished.
    pub fn is_terminal(self) -> bool {
        matches!(self, Self::Completed | Self::Fail | Self::Cancel)
    }
}

impl fmt::Display for CommonDownloadTaskState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let name = match self {
            Self::Normal => "Normal",
            Self::Waiting => "Waiting",
            Self::Downloading => "Downloading",
            Self::Paused => "Paused",
            Self::Completed => "Completed",
            Self::Fail => "Fail",
            Self::DownloadingConfig => "DownloadingConfigFile",
            Self::Cancel => "Cancel",
            Self::VerifyingFiles => "VerifyingFiles",
            Self::MergingFiles => "MergingFiles",
            Self::Pending => "Pending",
        };
        f.write_str(name)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn codes_round_trip() {
        for state in CommonDownloadTaskState::ALL {
            assert_eq!(CommonDownloadTaskState::from_code(state.code()), Some(state));
        }
        assert_eq!(CommonDownloadTaskState::from_code(99), None);
        assert_eq!(CommonDownloadTaskState::from_code(-1), None);
    }

    #[test]
    fn known_codes() {
        assert_eq!(CommonDownloadTaskState::Normal.code(), 0);
        assert_eq!(CommonDownloadTaskState::Downloading.code(), 2);
        assert_eq!(CommonDownloadTaskState::Fail.code(), 5);
        assert_eq!(CommonDownloadTaskState::Pending.code(), 10);
    }

    #[test]
    fn terminal_states() {
        assert!(CommonDownloadTaskState::Completed.is_terminal());
        assert!(CommonDownloadTaskState::Fail.is_terminal());
        assert!(!CommonDownloadTaskState::Downloading.is_terminal());
        assert_eq!(CommonDownloadTaskState::DownloadingConfig.to_string(), "DownloadingConfigFile");
    }
}
