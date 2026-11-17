//! Portable download-task model.
//!
//! Combines the two download abstractions found in the reference sources:
//!
//! * `com.xj.ota.http.HttpHandle` / `IDownloadListener` (firmware download):
//!   `a()` = finished, `b(int)` = percentage, `c()` = failed. Progress is
//!   computed as `(downloaded * 100) / content_length`.
//! * `com.xj.apk.update.config.DownloadStatus` (APK update): a sealed family of
//!   `Start`, `Downloading(content, max, progress)`, `Done(File)`, `Error`,
//!   `Cancel`, mirrored by [`DownloadStatus`].
//!
//! No networking, file I/O, Android service or notification code is present.

use std::fmt;

/// Lifecycle state of a [`DownloadTask`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DownloadTaskState {
    /// Created, not started.
    Pending,
    /// Actively transferring bytes.
    Downloading,
    /// Bytes received; integrity check pending.
    Verifying,
    /// Transfer and verification finished.
    Completed,
    /// Transfer failed.
    Failed,
    /// Cancelled by the caller.
    Cancelled,
}

impl DownloadTaskState {
    pub const fn is_running(self) -> bool {
        matches!(self, DownloadTaskState::Downloading | DownloadTaskState::Verifying)
    }

    pub const fn is_terminal(self) -> bool {
        matches!(
            self,
            DownloadTaskState::Completed | DownloadTaskState::Failed | DownloadTaskState::Cancelled
        )
    }
}

/// A downloadable artifact with id, url, progress and state.
///
/// Field origins:
/// * `id` — synthetic handle (the Java code tracks one active task at a time
///   via singletons `HttpHandle.f33604a` / `DownloadManager.f22710a`).
/// * `url` — `OTAInfor.d()` / `DownloadManager.apkUrl`.
/// * `destination` — `OTAInfor.e()` / `DownloadManager.downloadPath`.
/// * `expected_md5` — `OTAInfor.f()` / `DownloadManager.apkMD5`.
/// * `content_length` — `response.body().contentLength()` in `HttpHandle`.
/// * `downloaded` / `progress` — running byte count and its percentage.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DownloadTask {
    pub id: u64,
    pub url: String,
    pub destination: String,
    pub expected_md5: Option<String>,
    pub content_length: Option<u64>,
    pub downloaded: u64,
    /// Integer percentage in `0..=100`.
    pub progress: u8,
    pub state: DownloadTaskState,
}

impl DownloadTask {
    pub fn new(id: u64, url: impl Into<String>, destination: impl Into<String>) -> Self {
        Self {
            id,
            url: url.into(),
            destination: destination.into(),
            expected_md5: None,
            content_length: None,
            downloaded: 0,
            progress: 0,
            state: DownloadTaskState::Pending,
        }
    }

    pub fn with_md5(mut self, md5: impl Into<String>) -> Self {
        self.expected_md5 = Some(md5.into());
        self
    }

    pub fn with_content_length(mut self, len: u64) -> Self {
        self.content_length = Some(len);
        self
    }

    pub fn start(&mut self) {
        if !self.state.is_terminal() {
            self.state = DownloadTaskState::Downloading;
        }
    }

    /// Clamp and store a percentage from `IDownloadListener.b(int)`.
    pub fn set_progress(&mut self, percent: u8) {
        if !self.state.is_terminal() {
            self.progress = percent.min(100);
        }
    }

    /// Add received bytes and recompute the percentage from the known content
    /// length (the exact formula used by `HttpHandle.onResponse`).
    pub fn add_bytes(&mut self, n: u64) {
        if self.state.is_terminal() {
            return;
        }
        self.downloaded = self.downloaded.saturating_add(n);
        if let Some(total) = self.content_length {
            if total > 0 {
                let pct = ((self.downloaded as f64 / total as f64) * 100.0).round();
                self.progress = (pct as i64).clamp(0, 100) as u8;
            }
        }
    }

    pub fn complete(&mut self) {
        if !self.state.is_terminal() {
            self.progress = 100;
            self.state = DownloadTaskState::Verifying;
        }
    }

    pub fn verified(&mut self) {
        if self.state == DownloadTaskState::Verifying {
            self.state = DownloadTaskState::Completed;
        }
    }

    pub fn fail(&mut self) {
        if !self.state.is_terminal() {
            self.state = DownloadTaskState::Failed;
        }
    }

    pub fn cancel(&mut self) {
        if !self.state.is_terminal() {
            self.state = DownloadTaskState::Cancelled;
        }
    }

    /// True when the result has been verified and is usable.
    pub fn is_complete(&self) -> bool {
        self.state == DownloadTaskState::Completed
    }
}

impl fmt::Display for DownloadTask {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "DownloadTask#{} {} -> {} [{}% {:?}]",
            self.id, self.url, self.destination, self.progress, self.state
        )
    }
}

/// Kotlin sealed hierarchy `com.xj.apk.update.config.DownloadStatus`, reduced
/// to a plain, dependency-free enum.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DownloadStatus {
    /// `DownloadStatus.Start` (singleton).
    Start,
    /// `DownloadStatus.Downloading(content, max, progress)`.
    Downloading {
        content: String,
        max: u64,
        progress: u64,
    },
    /// `DownloadStatus.Done(File apk)` — the `File` is kept as a path string.
    Done { apk_path: String },
    /// `DownloadStatus.Error(Throwable e)` — the throwable is kept as text.
    Error { reason: String },
    /// `DownloadStatus.Cancel` (singleton).
    Cancel,
}

impl DownloadStatus {
    /// Integer percentage for the `Downloading` variant, guarding division by
    /// zero (the Kotlin code divides `progress / max` directly).
    pub fn percent(&self) -> Option<u64> {
        match self {
            DownloadStatus::Downloading { max, progress, .. } => {
                if *max == 0 {
                    None
                } else {
                    Some((*progress).min(*max) * 100 / *max)
                }
            }
            _ => None,
        }
    }
}

/// Listener events from `OnDownloadListener` / `IDownloadListener`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DownloadEvent {
    /// `start()` / `IDownloadListener` download begin.
    Start,
    /// `a(String content, int max, int progress)` / `b(int percent)`.
    Progress {
        content: String,
        max: u64,
        progress: u64,
    },
    /// `b(File apk)` / `a()`.
    Done { apk_path: String },
    /// `error(Throwable)` / `c()`.
    Error { reason: String },
    /// `cancel()`.
    Cancel,
}

impl DownloadEvent {
    /// Map a listener event to the corresponding `DownloadStatus`.
    pub fn to_status(self) -> DownloadStatus {
        match self {
            DownloadEvent::Start => DownloadStatus::Start,
            DownloadEvent::Progress {
                content,
                max,
                progress,
            } => DownloadStatus::Downloading {
                content,
                max,
                progress,
            },
            DownloadEvent::Done { apk_path } => DownloadStatus::Done { apk_path },
            DownloadEvent::Error { reason } => DownloadStatus::Error { reason },
            DownloadEvent::Cancel => DownloadStatus::Cancel,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn progress_is_clamped_to_0_100() {
        let mut t = DownloadTask::new(1, "https://x/fw.zip", "/c/fw.zip");
        t.start();
        t.set_progress(42);
        assert_eq!(t.progress, 42);
        t.set_progress(200);
        assert_eq!(t.progress, 100);
        t.set_progress(0);
        assert_eq!(t.progress, 0);
    }

    #[test]
    fn bytes_derive_progress_from_content_length() {
        let mut t = DownloadTask::new(2, "u", "d").with_content_length(200);
        t.start();
        t.add_bytes(50);
        assert_eq!(t.progress, 25);
        t.add_bytes(50);
        assert_eq!(t.progress, 50);
        t.add_bytes(1_000);
        assert_eq!(t.progress, 100, "never exceeds 100%");
    }

    #[test]
    fn zero_content_length_does_not_change_progress() {
        let mut t = DownloadTask::new(3, "u", "d").with_content_length(0);
        t.start();
        t.add_bytes(999);
        assert_eq!(t.progress, 0);
    }

    #[test]
    fn lifecycle_terminal_states_ignore_further_updates() {
        let mut t = DownloadTask::new(4, "u", "d");
        assert_eq!(t.state, DownloadTaskState::Pending);
        t.start();
        assert!(t.state.is_running());
        t.complete();
        assert_eq!(t.state, DownloadTaskState::Verifying);
        t.verified();
        assert!(t.is_complete());
        assert!(t.state.is_terminal());
        t.set_progress(10);
        t.fail();
        t.cancel();
        assert_eq!(t.state, DownloadTaskState::Completed);
        assert_eq!(t.progress, 100);
    }

    #[test]
    fn failed_and_cancelled_tasks_are_terminal() {
        let mut a = DownloadTask::new(5, "u", "d");
        a.start();
        a.fail();
        assert_eq!(a.state, DownloadTaskState::Failed);
        assert!(a.state.is_terminal());
        a.start();
        assert_eq!(a.state, DownloadTaskState::Failed);

        let mut b = DownloadTask::new(6, "u", "d");
        b.cancel();
        assert_eq!(b.state, DownloadTaskState::Cancelled);
        assert!(b.state.is_terminal());
    }

    #[test]
    fn download_status_percent_handles_zero_max() {
        let s = DownloadStatus::Downloading {
            content: "fw".into(),
            max: 0,
            progress: 5,
        };
        assert_eq!(s.percent(), None);
        let s = DownloadStatus::Downloading {
            content: "fw".into(),
            max: 200,
            progress: 50,
        };
        assert_eq!(s.percent(), Some(25));
        assert_eq!(DownloadStatus::Start.percent(), None);
    }

    #[test]
    fn listener_events_map_to_status() {
        assert_eq!(DownloadEvent::Start.to_status(), DownloadStatus::Start);
        assert_eq!(
            DownloadEvent::Done { apk_path: "/x.apk".into() }.to_status(),
            DownloadStatus::Done { apk_path: "/x.apk".into() }
        );
        assert_eq!(
            DownloadEvent::Error { reason: "boom".into() }.to_status(),
            DownloadStatus::Error { reason: "boom".into() }
        );
        assert_eq!(DownloadEvent::Cancel.to_status(), DownloadStatus::Cancel);
    }

    #[test]
    fn display_contains_id_url_and_state() {
        let t = DownloadTask::new(9, "https://x/f.zip", "/f.zip");
        let s = t.to_string();
        assert!(s.contains("DownloadTask#9"));
        assert!(s.contains("https://x/f.zip"));
        assert!(s.contains("Pending"));
    }
}
