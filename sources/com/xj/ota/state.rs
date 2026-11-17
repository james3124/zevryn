//! Portable OTA upgrade state machine.
//!
//! Modeled on `com.xj.ota.set.BaseOTA` (`g()` / `c()` / `d()` / `e()` / `i()` /
//! `j()`) and the `IUpListener` callback contract of GameHub Lite v5.1.8.
//!
//! The original flow (`BaseOTA.g()`) is:
//!
//! ```text
//!   otaInfor == null? --yes--> fail(7)
//!        | no
//!   cached firmware valid? (c(): file exists, and when mode==0 also MD5)
//!        | no                              | yes
//!   download (d()) --fail--> fail(7)       |
//!        | ok                              |
//!   re-verify (c()) --fail--> fail(7) <----+
//!        | ok
//!   pre-install hook (e()) --false--> direct flash (j())
//!        | true
//!   unzip (i()) --fail--> fail(3)
//!        | ok
//!   flash (j()) --> done
//! ```
//!
//! Android specifics (Context, OkHttp, ConditionVariable, Handler threads, the
//! concrete per-device flash routines) are intentionally absent; only the
//! *ordering* and *allowed transitions* are portable and are reproduced here.

use super::download::DownloadTask;

/// Failure reasons that can terminate an OTA session.
///
/// `b(int)` on `IUpListener` carries a numeric error code. The codes actually
/// emitted by the reference sources are preserved here; `Code` keeps any
/// device-specific code (e.g. 711, 79, 71, 110 seen in G8/T4nLite/X3Pro OTA
/// classes) representable without losing information.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum OtaFailure {
    /// Null `OTAInfor` passed to `BaseOTA.g()` -> `b(7)`.
    NullUpdateInfo,
    /// Update archive / up-file missing -> `b(3)`.
    FileNotFound,
    /// MD5 of the cached package does not match -> `b(7)`.
    ChecksumMismatch,
    /// Firmware download failed -> `b(7)`.
    DownloadFailed,
    /// Post-download verification failed -> `b(7)`.
    VerifyFailed,
    /// Flash / unzip step failed -> `b(7)`.
    InstallFailed,
    /// A raw `IUpListener.b(int)` code from a device-specific OTA class.
    Code(i32),
}

impl OtaFailure {
    /// The numeric `IUpListener.b(int)` code carried by this failure.
    pub const fn code(self) -> i32 {
        match self {
            OtaFailure::Code(c) => c,
            OtaFailure::FileNotFound => 3,
            OtaFailure::NullUpdateInfo
            | OtaFailure::ChecksumMismatch
            | OtaFailure::DownloadFailed
            | OtaFailure::VerifyFailed
            | OtaFailure::InstallFailed => 7,
        }
    }
}

impl std::fmt::Display for OtaFailure {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let name = match self {
            OtaFailure::NullUpdateInfo => "null update info",
            OtaFailure::FileNotFound => "firmware file not found",
            OtaFailure::ChecksumMismatch => "firmware checksum mismatch",
            OtaFailure::DownloadFailed => "firmware download failed",
            OtaFailure::VerifyFailed => "firmware verification failed",
            OtaFailure::InstallFailed => "firmware install failed",
            OtaFailure::Code(_) => "device-specific error",
        };
        write!(f, "{} (code {})", name, self.code())
    }
}

/// States of a portable OTA upgrade session.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum OtaState {
    /// No upgrade in progress.
    Idle,
    /// Checking whether an update is available / whether a cached package is valid.
    Checking,
    /// Downloading the firmware archive.
    Downloading,
    /// Verifying the downloaded archive (existence + MD5).
    Verifying,
    /// Unzipping and flashing the firmware.
    Installing,
    /// Upgrade finished successfully.
    Completed,
    /// Upgrade aborted with a failure reason.
    Failed(OtaFailure),
    /// Upgrade cancelled by the user / caller.
    Cancelled,
}

impl OtaState {
    /// True for states in which no further progress events are accepted.
    pub const fn is_terminal(self) -> bool {
        matches!(
            self,
            OtaState::Completed | OtaState::Failed(_) | OtaState::Cancelled
        )
    }

    /// True while work is actively in flight.
    pub const fn is_active(self) -> bool {
        matches!(
            self,
            OtaState::Checking | OtaState::Downloading | OtaState::Verifying | OtaState::Installing
        )
    }
}

/// Events that drive [`OtaState`] transitions.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OtaEvent {
    /// Start checking for an update (`BaseOTA.g()` entry).
    Check,
    /// No update available (version check returned false).
    CheckNoUpdate,
    /// A cached package is already valid; skip download.
    CheckCacheValid,
    /// No valid cached package; a download is required.
    CheckNeedsDownload,
    /// Check step failed with a raw error.
    CheckFailed(OtaFailure),
    /// Begin downloading.
    DownloadStart,
    /// Download progress tick (0..=100), mirrors `IDownloadListener.b(int)`.
    DownloadProgress(u8),
    /// Download finished, mirrors `IDownloadListener.a()`.
    DownloadDone,
    /// Download failed, mirrors `IDownloadListener.c()`.
    DownloadFail(OtaFailure),
    /// Begin post-download verification.
    VerifyStart,
    /// Verification succeeded.
    VerifyOk,
    /// Verification failed.
    VerifyFail(OtaFailure),
    /// Begin unzip + flash.
    InstallStart,
    /// Flash progress tick (0..=100).
    InstallProgress(u8),
    /// Flash finished.
    InstallDone,
    /// Flash failed.
    InstallFail(OtaFailure),
    /// Cancel an in-flight upgrade.
    Cancel,
    /// Return a terminal state to `Idle` for a retry.
    Reset,
}

/// Error returned when an event is not legal in the current state.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct InvalidTransition {
    pub from: OtaState,
    pub event: OtaEvent,
}

impl std::fmt::Display for InvalidTransition {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "event {:?} is not valid in state {:?}", self.event, self.from)
    }
}

impl std::error::Error for InvalidTransition {}

/// A side effect requested by a successful transition. Kept as data so callers
/// can decide how (or whether) to perform it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OtaEffect {
    /// Nothing to do (state bookkeeping only).
    None,
    /// Kick off a firmware download.
    Download,
    /// Verify the cached / downloaded package.
    Verify,
    /// Unzip and flash the package.
    Install,
    /// Report a failure through `IUpListener.b(int)`.
    ReportFailure(OtaFailure),
    /// The upgrade completed successfully.
    Done,
}

/// Apply `event` to `state`, returning the resulting state and the single side
/// effect that the transition requests.
pub fn transition(
    state: OtaState,
    event: OtaEvent,
) -> Result<(OtaState, OtaEffect), InvalidTransition> {
    use OtaEffect as E;
    use OtaEvent as V;
    use OtaState as S;

    let result = match (state, event) {
        // --- check phase ---------------------------------------------------
        (S::Idle, V::Check) => (S::Checking, E::None),
        (S::Checking, V::CheckNoUpdate) => (S::Completed, E::Done),
        (S::Checking, V::CheckCacheValid) => (S::Verifying, E::Verify),
        (S::Checking, V::CheckNeedsDownload) => (S::Downloading, E::Download),
        (S::Checking, V::CheckFailed(f)) => (S::Failed(f), E::ReportFailure(f)),

        // --- download phase ------------------------------------------------
        (S::Downloading, V::DownloadStart) => (S::Downloading, E::None),
        (S::Downloading, V::DownloadProgress(_)) => (S::Downloading, E::None),
        (S::Downloading, V::DownloadDone) => (S::Verifying, E::Verify),
        (S::Downloading, V::DownloadFail(f)) => (S::Failed(f), E::ReportFailure(f)),

        // --- verify phase --------------------------------------------------
        (S::Verifying, V::VerifyStart) => (S::Verifying, E::None),
        (S::Verifying, V::VerifyOk) => (S::Installing, E::Install),
        (S::Verifying, V::VerifyFail(f)) => (S::Failed(f), E::ReportFailure(f)),

        // --- install phase -------------------------------------------------
        (S::Installing, V::InstallStart) => (S::Installing, E::None),
        (S::Installing, V::InstallProgress(_)) => (S::Installing, E::None),
        (S::Installing, V::InstallDone) => (S::Completed, E::Done),
        (S::Installing, V::InstallFail(f)) => (S::Failed(f), E::ReportFailure(f)),

        // --- cancellation from any active phase ----------------------------
        (s, V::Cancel) if s.is_active() => (S::Cancelled, E::None),

        // --- retry from a terminal state -----------------------------------
        (S::Completed, V::Reset) | (S::Failed(_), V::Reset) | (S::Cancelled, V::Reset) => {
            (S::Idle, E::None)
        }

        _ => return Err(InvalidTransition { from: state, event }),
    };
    Ok(result)
}

/// Non-mutating query: is `event` legal in `state`?
pub fn can_transition(state: OtaState, event: OtaEvent) -> bool {
    transition(state, event).is_ok()
}

/// Stateful wrapper around [`transition`] that owns the state, an optional
/// [`DownloadTask`] and the last failure, mirroring the fields held by
/// `BaseOTA` / `BaseBleOTAActivity`.
#[derive(Debug, Clone)]
pub struct OtaStateMachine {
    pub state: OtaState,
    pub task: Option<DownloadTask>,
    pub last_failure: Option<OtaFailure>,
}

impl Default for OtaStateMachine {
    fn default() -> Self {
        Self::new()
    }
}

impl OtaStateMachine {
    pub fn new() -> Self {
        Self {
            state: OtaState::Idle,
            task: None,
            last_failure: None,
        }
    }

    /// Drive the machine with an event. Download progress events update the
    /// owned [`DownloadTask`] when one is present.
    pub fn handle(&mut self, event: OtaEvent) -> Result<OtaEffect, InvalidTransition> {
        // Keep the download task's progress in sync with progress events even
        // if a transition is otherwise rejected (progress is informational).
        match event {
            OtaEvent::DownloadProgress(p) => {
                if let Some(task) = self.task.as_mut() {
                    task.set_progress(p);
                }
            }
            OtaEvent::DownloadStart => {
                if let Some(task) = self.task.as_mut() {
                    task.start();
                }
            }
            OtaEvent::DownloadDone => {
                if let Some(task) = self.task.as_mut() {
                    task.complete();
                }
            }
            OtaEvent::CheckNeedsDownload => {
                // A task is created lazily by the caller; nothing to do here.
            }
            _ => {}
        }

        let (next, effect) = transition(self.state, event)?;
        match effect {
            OtaEffect::ReportFailure(f) => self.last_failure = Some(f),
            OtaEffect::Done => self.last_failure = None,
            _ => {}
        }
        // A reset starts a fresh session: drop the previous task and failure.
        if event == OtaEvent::Reset {
            self.last_failure = None;
            self.task = None;
        }
        self.state = next;
        Ok(effect)
    }

    /// Attach a download task (used when the check step finds an update).
    pub fn attach_task(&mut self, task: DownloadTask) {
        self.task = Some(task);
    }

    pub fn is_terminal(&self) -> bool {
        self.state.is_terminal()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn task() -> DownloadTask {
        DownloadTask::new(1, "https://example.test/fw.zip", "/cache/fw.zip")
    }

    #[test]
    fn happy_path_check_download_verify_install() {
        let mut m = OtaStateMachine::new();
        assert_eq!(m.state, OtaState::Idle);
        assert_eq!(m.handle(OtaEvent::Check).unwrap(), OtaEffect::None);
        assert_eq!(m.state, OtaState::Checking);
        assert_eq!(
            m.handle(OtaEvent::CheckNeedsDownload).unwrap(),
            OtaEffect::Download
        );
        m.attach_task(task());
        assert_eq!(m.handle(OtaEvent::DownloadStart).unwrap(), OtaEffect::None);
        assert_eq!(m.task.as_ref().unwrap().state.is_running(), true);
        assert_eq!(
            m.handle(OtaEvent::DownloadProgress(42)).unwrap(),
            OtaEffect::None
        );
        assert_eq!(m.task.as_ref().unwrap().progress, 42);
        assert_eq!(
            m.handle(OtaEvent::DownloadDone).unwrap(),
            OtaEffect::Verify
        );
        assert_eq!(m.state, OtaState::Verifying);
        assert_eq!(m.handle(OtaEvent::VerifyOk).unwrap(), OtaEffect::Install);
        assert_eq!(m.state, OtaState::Installing);
        assert_eq!(m.handle(OtaEvent::InstallDone).unwrap(), OtaEffect::Done);
        assert_eq!(m.state, OtaState::Completed);
        assert!(m.is_terminal());
        assert_eq!(m.last_failure, None);
    }

    #[test]
    fn cached_firmware_skips_download() {
        let mut m = OtaStateMachine::new();
        m.handle(OtaEvent::Check).unwrap();
        assert_eq!(
            m.handle(OtaEvent::CheckCacheValid).unwrap(),
            OtaEffect::Verify
        );
        assert_eq!(m.state, OtaState::Verifying);
        assert!(m.task.is_none(), "no download task must be created");
        m.handle(OtaEvent::VerifyOk).unwrap();
        m.handle(OtaEvent::InstallDone).unwrap();
        assert_eq!(m.state, OtaState::Completed);
    }

    #[test]
    fn no_update_finishes_without_download() {
        let mut m = OtaStateMachine::new();
        m.handle(OtaEvent::Check).unwrap();
        assert_eq!(
            m.handle(OtaEvent::CheckNoUpdate).unwrap(),
            OtaEffect::Done
        );
        assert_eq!(m.state, OtaState::Completed);
    }

    #[test]
    fn failures_at_each_stage_record_the_code() {
        // Download failure -> code 7.
        let mut m = OtaStateMachine::new();
        m.handle(OtaEvent::Check).unwrap();
        m.handle(OtaEvent::CheckNeedsDownload).unwrap();
        assert_eq!(
            m.handle(OtaEvent::DownloadFail(OtaFailure::DownloadFailed))
                .unwrap(),
            OtaEffect::ReportFailure(OtaFailure::DownloadFailed)
        );
        assert_eq!(m.state, OtaState::Failed(OtaFailure::DownloadFailed));
        assert_eq!(m.last_failure.unwrap().code(), 7);

        // Verify failure -> code 7.
        let mut m = OtaStateMachine::new();
        m.handle(OtaEvent::Check).unwrap();
        m.handle(OtaEvent::CheckCacheValid).unwrap();
        m.handle(OtaEvent::VerifyFail(OtaFailure::ChecksumMismatch))
            .unwrap();
        assert_eq!(m.state, OtaState::Failed(OtaFailure::ChecksumMismatch));

        // Install failure with a device-specific code.
        let mut m = OtaStateMachine::new();
        m.handle(OtaEvent::Check).unwrap();
        m.handle(OtaEvent::CheckCacheValid).unwrap();
        m.handle(OtaEvent::VerifyOk).unwrap();
        m.handle(OtaEvent::InstallFail(OtaFailure::Code(711)))
            .unwrap();
        assert_eq!(m.last_failure.unwrap().code(), 711);

        // Missing up-file -> code 3.
        let mut m = OtaStateMachine::new();
        m.handle(OtaEvent::Check).unwrap();
        m.handle(OtaEvent::CheckFailed(OtaFailure::FileNotFound))
            .unwrap();
        assert_eq!(m.last_failure.unwrap().code(), 3);
    }

    #[test]
    fn cancel_is_allowed_from_every_active_state() {
        for next_after_check in [OtaEvent::CheckNeedsDownload, OtaEvent::CheckCacheValid] {
            let mut m = OtaStateMachine::new();
            m.handle(OtaEvent::Check).unwrap();
            m.handle(next_after_check).unwrap();
            assert!(m.state.is_active());
            m.handle(OtaEvent::Cancel).unwrap();
            assert_eq!(m.state, OtaState::Cancelled);
        }

        let mut m = OtaStateMachine::new();
        m.handle(OtaEvent::Check).unwrap();
        m.handle(OtaEvent::CheckCacheValid).unwrap();
        m.handle(OtaEvent::VerifyOk).unwrap();
        assert_eq!(m.state, OtaState::Installing);
        m.handle(OtaEvent::Cancel).unwrap();
        assert_eq!(m.state, OtaState::Cancelled);
    }

    #[test]
    fn invalid_transitions_are_rejected() {
        // Cannot download without checking.
        assert!(!can_transition(OtaState::Idle, OtaEvent::DownloadDone));
        // Cannot verify before download/verify state.
        assert!(!can_transition(OtaState::Checking, OtaEvent::VerifyOk));
        // Cannot install before verification.
        assert!(!can_transition(OtaState::Downloading, OtaEvent::InstallStart));
        // Terminal states reject progress events until reset.
        assert!(!can_transition(
            OtaState::Completed,
            OtaEvent::DownloadProgress(10)
        ));
        assert!(!can_transition(OtaState::Failed(OtaFailure::Code(1)), OtaEvent::Cancel));

        let err = transition(OtaState::Idle, OtaEvent::InstallDone).unwrap_err();
        assert_eq!(err.from, OtaState::Idle);
        assert_eq!(err.event, OtaEvent::InstallDone);
        assert!(err.to_string().contains("not valid"));
    }

    #[test]
    fn reset_allows_a_retry_after_terminal_state() {
        for terminal in [
            OtaState::Completed,
            OtaState::Failed(OtaFailure::DownloadFailed),
            OtaState::Cancelled,
        ] {
            assert_eq!(transition(terminal, OtaEvent::Reset).unwrap().0, OtaState::Idle);
            assert!(!can_transition(terminal, OtaEvent::Check));
        }

        let mut m = OtaStateMachine::new();
        m.handle(OtaEvent::Check).unwrap();
        m.handle(OtaEvent::CheckNeedsDownload).unwrap();
        m.handle(OtaEvent::DownloadFail(OtaFailure::DownloadFailed))
            .unwrap();
        m.handle(OtaEvent::Reset).unwrap();
        assert_eq!(m.state, OtaState::Idle);
        assert_eq!(m.last_failure, None);
        m.handle(OtaEvent::Check).unwrap();
        assert_eq!(m.state, OtaState::Checking);
    }

    #[test]
    fn progress_events_are_ignored_where_not_applicable() {
        assert!(!can_transition(OtaState::Checking, OtaEvent::DownloadProgress(5)));
        assert!(!can_transition(OtaState::Installing, OtaEvent::DownloadProgress(5)));
        assert!(can_transition(
            OtaState::Downloading,
            OtaEvent::DownloadProgress(200)
        ));
    }

    #[test]
    fn failure_codes_match_iup_listener_contract() {
        assert_eq!(OtaFailure::FileNotFound.code(), 3);
        assert_eq!(OtaFailure::NullUpdateInfo.code(), 7);
        assert_eq!(OtaFailure::ChecksumMismatch.code(), 7);
        assert_eq!(OtaFailure::Code(110).code(), 110);
        assert!(OtaFailure::Code(79).to_string().contains("79"));
    }
}
