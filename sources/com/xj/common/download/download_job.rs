//! Portable model of the general Aria download job.
//!
//! Mirrors `com.xj.common.download.bean.CommonDownloadTask` from the reference
//! sources. That class is a Kotlin data class with an integer-backed state code
//! (`CommonDownloadTaskState`), byte counts, a percentage and a speed that the
//! setter only keeps while the job is actively downloading.
//!
//! No networking, Android service, notification or coroutine code is present —
//! only the state model and the byte/progress bookkeeping that the UI observes.

use std::fmt;

/// Lifecycle state of a [`DownloadJob`].
///
/// Codes and names are taken verbatim from the decompiled
/// `CommonDownloadTaskState` enum. The enum is redefined here (instead of
/// importing `crate::data::CommonDownloadTaskState`) so this module stays
/// self-contained and can be wired independently.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum JobState {
    Normal,            // 0
    Waiting,           // 1
    Downloading,       // 2
    Paused,            // 3
    Completed,         // 4
    Fail,              // 5
    DownloadingConfig, // 6  (DownloadingConfigFile)
    Cancel,            // 7
    VerifyingFiles,    // 8
    MergingFiles,      // 9
    Pending,           // 10
}

impl JobState {
    /// Every state in declaration order (`values()` in Kotlin).
    pub const ALL: [JobState; 11] = [
        JobState::Normal,
        JobState::Waiting,
        JobState::Downloading,
        JobState::Paused,
        JobState::Completed,
        JobState::Fail,
        JobState::DownloadingConfig,
        JobState::Cancel,
        JobState::VerifyingFiles,
        JobState::MergingFiles,
        JobState::Pending,
    ];

    /// The `getType()` integer code.
    pub const fn code(self) -> i32 {
        match self {
            JobState::Normal => 0,
            JobState::Waiting => 1,
            JobState::Downloading => 2,
            JobState::Paused => 3,
            JobState::Completed => 4,
            JobState::Fail => 5,
            JobState::DownloadingConfig => 6,
            JobState::Cancel => 7,
            JobState::VerifyingFiles => 8,
            JobState::MergingFiles => 9,
            JobState::Pending => 10,
        }
    }

    /// Reverse lookup by `getType()` code. Unknown codes return `None`.
    pub fn from_code(code: i32) -> Option<Self> {
        Self::ALL.iter().copied().find(|s| s.code() == code)
    }

    /// Mirrors `CommonDownloadTask.isDownloading()`: the job is still in the
    /// active download pipeline (transfer, config fetch, verify, merge, or a
    /// queued/pending task). Notably `Waiting`, `Paused` and the terminal
    /// states are excluded.
    pub const fn is_downloading(self) -> bool {
        matches!(
            self,
            JobState::Downloading
                | JobState::DownloadingConfig
                | JobState::VerifyingFiles
                | JobState::MergingFiles
                | JobState::Pending
        )
    }

    /// A state after which the job can no longer make progress.
    pub const fn is_terminal(self) -> bool {
        matches!(self, JobState::Completed | JobState::Fail | JobState::Cancel)
    }

    /// Canonical enum name (matches `toString()` of the Kotlin enum).
    pub const fn as_str(self) -> &'static str {
        match self {
            JobState::Normal => "Normal",
            JobState::Waiting => "Waiting",
            JobState::Downloading => "Downloading",
            JobState::Paused => "Paused",
            JobState::Completed => "Completed",
            JobState::Fail => "Fail",
            JobState::DownloadingConfig => "DownloadingConfigFile",
            JobState::Cancel => "Cancel",
            JobState::VerifyingFiles => "VerifyingFiles",
            JobState::MergingFiles => "MergingFiles",
            JobState::Pending => "Pending",
        }
    }
}

impl fmt::Display for JobState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A single general download job.
///
/// Field origins in `CommonDownloadTask`:
/// * `id` — `getId()` (String handle in the original).
/// * `url` — `getUrl()`.
/// * `dest` — destination path kept in `AriaDownloadArgs.fileName` /
///   `UnZipFileArgs.unzipFilePath`; the original task object itself does not
///   store a path, so it is modelled explicitly here.
/// * `total` / `downloaded` — `getSize()` / `getDownloadedSize()`.
/// * `state` — `getState()`.
/// * `speed` — `getSpeed()` (bytes/second).
/// * `progress` — `getPercent()` (integer `0..=100`), recomputed from the byte
///   counts the same way the OTA `HttpHandle` listener computes its percentage.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DownloadJob {
    pub id: String,
    pub url: String,
    pub dest: String,
    pub total: u64,
    pub downloaded: u64,
    pub state: JobState,
    /// Bytes per second. Only retained while `state == Downloading`, mirroring
    /// `CommonDownloadTask.setSpeed`.
    pub speed: u64,
    /// Integer percentage in `0..=100`.
    pub progress: u8,
    /// Mirrors `getFailCode()`; `0` means no error (`UN_KNOW`).
    pub fail_code: i32,
}

impl DownloadJob {
    /// Create a job. The original synthetic constructor defaults the state to
    /// `Waiting`; the same default is used here.
    pub fn new(id: impl Into<String>, url: impl Into<String>, dest: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            url: url.into(),
            dest: dest.into(),
            total: 0,
            downloaded: 0,
            state: JobState::Waiting,
            speed: 0,
            progress: 0,
            fail_code: 0,
        }
    }

    pub fn with_total(mut self, total: u64) -> Self {
        self.total = total;
        self
    }

    /// Clamp an externally reported percentage (`IDownloadListener.b(int)`).
    pub fn set_progress(&mut self, percent: u8) {
        if !self.state.is_terminal() {
            self.progress = percent.min(100);
        }
    }

    /// `CommonDownloadTask.setSpeed`: a non-downloading job reports zero speed.
    pub fn set_speed(&mut self, speed: u64) {
        self.speed = if self.state == JobState::Downloading {
            speed
        } else {
            0
        };
    }

    /// Add received bytes and recompute `progress` from `total`.
    ///
    /// Uses integer arithmetic with a `u128` intermediate so large byte counts
    /// cannot overflow, and clamps to `0..=100` when the reported bytes exceed
    /// the declared size. Bytes are only accepted while actively downloading;
    /// paused, queued and terminal jobs ignore them.
    pub fn add_bytes(&mut self, n: u64) {
        if self.state != JobState::Downloading {
            return;
        }
        self.downloaded = self.downloaded.saturating_add(n);
        if self.total > 0 {
            let pct = (self.downloaded as u128 * 100 / self.total as u128) as u64;
            self.progress = pct.min(100) as u8;
        }
    }

    /// Estimated remaining seconds, mirroring `CommonDownloadTask.getLeftTime()`:
    /// `(total - downloaded) / speed` for an Aria download, or `0` when the
    /// speed is unknown or the job is not actively downloading.
    pub fn left_time(&self) -> u64 {
        if self.state != JobState::Downloading || self.speed == 0 || self.downloaded >= self.total {
            return 0;
        }
        (self.total - self.downloaded) / self.speed
    }

    // --- state transitions (the download -> unzip -> merge -> verify pipeline) ---

    /// Begin (or resume after `Waiting`/`Pending`/`Normal`).
    pub fn start(&mut self) {
        if !self.state.is_terminal() {
            self.state = JobState::Downloading;
        }
    }

    pub fn pause(&mut self) {
        if self.state == JobState::Downloading || self.state == JobState::DownloadingConfig {
            self.state = JobState::Paused;
            self.speed = 0;
        }
    }

    pub fn resume(&mut self) {
        if self.state == JobState::Paused {
            self.state = JobState::Downloading;
        }
    }

    /// Fetch auxiliary config files before the main archive (state 6).
    pub fn download_config(&mut self) {
        if !self.state.is_terminal() {
            self.state = JobState::DownloadingConfig;
            self.speed = 0;
        }
    }

    /// Transition to file verification (state 8), e.g. MD5 checks.
    pub fn verify(&mut self) {
        if !self.state.is_terminal() {
            self.state = JobState::VerifyingFiles;
            self.speed = 0;
        }
    }

    /// Transition to file merging (state 9), before verification completes.
    pub fn merge(&mut self) {
        if !self.state.is_terminal() {
            self.state = JobState::MergingFiles;
            self.speed = 0;
        }
    }

    pub fn complete(&mut self) {
        if !self.state.is_terminal() {
            self.state = JobState::Completed;
            self.progress = 100;
            self.speed = 0;
        }
    }

    pub fn fail(&mut self, code: i32) {
        if !self.state.is_terminal() {
            self.state = JobState::Fail;
            self.fail_code = code;
            self.speed = 0;
        }
    }

    pub fn cancel(&mut self) {
        if !self.state.is_terminal() {
            self.state = JobState::Cancel;
            self.speed = 0;
        }
    }

    /// Copy the volatile fields of `new` into this job, mirroring
    /// `CommonDownloadTask.upgrade`.
    pub fn upgrade(&mut self, new: &DownloadJob) {
        self.set_speed(new.speed);
        self.state = new.state;
        self.url = new.url.clone();
        self.downloaded = new.downloaded;
        self.total = new.total;
        self.progress = new.progress;
        self.fail_code = new.fail_code;
    }

    pub fn is_complete(&self) -> bool {
        self.state == JobState::Completed
    }
}

impl fmt::Display for DownloadJob {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "DownloadJob#{} {} -> {} [{}% {}]",
            self.id, self.url, self.dest, self.progress, self.state
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn state_codes_round_trip() {
        for state in JobState::ALL {
            assert_eq!(JobState::from_code(state.code()), Some(state));
        }
        assert_eq!(JobState::from_code(-1), None);
        assert_eq!(JobState::from_code(99), None);
        assert_eq!(JobState::DownloadingConfig.code(), 6);
        assert_eq!(JobState::DownloadingConfig.as_str(), "DownloadingConfigFile");
    }

    #[test]
    fn is_downloading_matches_java_helper() {
        // Java isDownloading(): Downloading, DownloadingConfigFile,
        // VerifyingFiles, MergingFiles, Pending.
        assert!(JobState::Downloading.is_downloading());
        assert!(JobState::DownloadingConfig.is_downloading());
        assert!(JobState::VerifyingFiles.is_downloading());
        assert!(JobState::MergingFiles.is_downloading());
        assert!(JobState::Pending.is_downloading());
        assert!(!JobState::Waiting.is_downloading());
        assert!(!JobState::Paused.is_downloading());
        assert!(!JobState::Completed.is_downloading());
    }

    #[test]
    fn lifecycle_transitions() {
        let mut j = DownloadJob::new("a", "https://x/f.zip", "/d/f.zip").with_total(200);
        assert_eq!(j.state, JobState::Waiting);
        j.start();
        assert_eq!(j.state, JobState::Downloading);
        j.add_bytes(50);
        assert_eq!(j.progress, 25);
        j.pause();
        assert_eq!(j.state, JobState::Paused);
        j.add_bytes(50);
        assert_eq!(j.downloaded, 50, "paused jobs ignore bytes");
        j.resume();
        j.set_speed(10);
        assert_eq!(j.speed, 10);
        assert_eq!(j.left_time(), 15, "(200-50)/10");
        j.verify();
        assert_eq!(j.speed, 0, "speed is cleared when leaving Downloading");
        j.merge();
        assert_eq!(j.state, JobState::MergingFiles);
        j.complete();
        assert_eq!(j.progress, 100);
        assert!(j.is_complete());
        assert!(j.state.is_terminal());
    }

    #[test]
    fn terminal_states_ignore_updates() {
        let mut j = DownloadJob::new("b", "u", "d");
        j.fail(1);
        assert_eq!(j.state, JobState::Fail);
        assert_eq!(j.fail_code, 1);
        j.start();
        j.complete();
        j.cancel();
        assert_eq!(j.state, JobState::Fail);

        let mut k = DownloadJob::new("c", "u", "d");
        k.cancel();
        assert!(k.state.is_terminal());
        k.set_progress(80);
        assert_eq!(k.progress, 0);
    }

    #[test]
    fn progress_is_clamped_and_safe_without_total() {
        let mut j = DownloadJob::new("d", "u", "d").with_total(100);
        j.start();
        j.add_bytes(250);
        assert_eq!(j.progress, 100, "never exceeds 100");
        let mut z = DownloadJob::new("e", "u", "d");
        z.start();
        z.add_bytes(999);
        assert_eq!(z.progress, 0, "unknown total leaves progress untouched");
        z.set_progress(200);
        assert_eq!(z.progress, 100);
    }

    #[test]
    fn speed_only_applies_while_downloading() {
        let mut j = DownloadJob::new("f", "u", "d");
        j.set_speed(999);
        assert_eq!(j.speed, 0, "Waiting job has zero speed");
        j.start();
        j.set_speed(999);
        assert_eq!(j.speed, 999);
    }
}
