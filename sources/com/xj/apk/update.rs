//! Portable logic of `com.xj.apk.update` (APK self-update module).
//!
//! Sources:
//! * `config.Constant` — cache path template and file-provider authority.
//! * `config.DownloadStatus` — download result sealed class (modeled in
//!   [`download`], shared with the OTA area).
//! * `manager.DownloadManager` — builder fields and the `a()` / `b()` /
//!   `download()` decision logic.
//! * `listener.OnDownloadListener` / `OnDownloadListenerAdapter` — callbacks.
//!
//! Android service startup (`DownloadService`), notifications, FileProvider,
//! Activity lifecycle callbacks and view binding are **not** implemented.

use std::fmt;

/// `Constant.f22694c` cache directory template.
pub const CACHE_DIR_TEMPLATE: &str = "/storage/emulated/0/Android/data/%s/cache";
/// `DownloadManager.a()` appends this to the package name for the provider authority.
pub const FILE_PROVIDER_SUFFIX: &str = ".apkUpdateFileProvider";
/// Value used by `Builder` for an unset version code.
pub const UNSET_VERSION_CODE: i32 = i32::MIN;

/// `config.Constant` — portable constants and formatting.
pub struct UpdatePaths;

impl UpdatePaths {
    pub fn cache_dir(package_name: &str) -> String {
        CACHE_DIR_TEMPLATE.replacen("%s", package_name, 1)
    }

    pub fn file_provider_authority(package_name: &str) -> String {
        format!("{}{}", package_name, FILE_PROVIDER_SUFFIX)
    }
}

/// Configuration carried by `DownloadManager` / its `Builder`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ApkUpdateConfig {
    pub apk_url: String,
    pub apk_name: String,
    pub apk_md5: String,
    pub apk_size: String,
    pub apk_description: String,
    pub apk_version_code: i32,
    pub apk_version_name: String,
    pub download_path: String,
    pub show_newer_toast: bool,
    pub show_notification: bool,
    pub forced_upgrade: bool,
    pub jump_install_page: bool,
    pub show_bgd_toast: bool,
    pub small_icon: i32,
    pub notify_id: i32,
}

impl Default for ApkUpdateConfig {
    fn default() -> Self {
        Self {
            apk_url: String::new(),
            apk_name: String::new(),
            apk_md5: String::new(),
            apk_size: String::new(),
            apk_description: String::new(),
            apk_version_code: UNSET_VERSION_CODE,
            apk_version_name: String::new(),
            download_path: String::new(),
            show_newer_toast: false,
            show_notification: true,
            forced_upgrade: false,
            jump_install_page: true,
            show_bgd_toast: true,
            small_icon: -1,
            notify_id: -1,
        }
    }
}

/// Validation failures from `DownloadManager.a()` / `b()`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConfigError {
    /// `apkUrl 不能为空！`
    EmptyUrl,
    /// `apkName 不能为空！`
    EmptyName,
    /// `apkName 必须以 .apk 结尾！`
    InvalidName,
    /// `smallIcon 不能为空！`
    MissingIcon,
    /// `apkDescription 不能为空！`
    MissingDescription,
}

impl fmt::Display for ConfigError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let msg = match self {
            ConfigError::EmptyUrl => "apkUrl must not be empty",
            ConfigError::EmptyName => "apkName must not be empty",
            ConfigError::InvalidName => "apkName must end with .apk",
            ConfigError::MissingIcon => "smallIcon must be set",
            ConfigError::MissingDescription => "apkDescription must not be empty",
        };
        f.write_str(msg)
    }
}

impl std::error::Error for ConfigError {}

impl ApkUpdateConfig {
    /// `DownloadManager.a()` — validate URLs / file name / icon.
    pub fn validate_download(&self) -> Result<(), ConfigError> {
        if self.apk_url.is_empty() {
            return Err(ConfigError::EmptyUrl);
        }
        if self.apk_name.is_empty() {
            return Err(ConfigError::EmptyName);
        }
        if !self.apk_name.to_ascii_lowercase().ends_with(".apk") {
            return Err(ConfigError::InvalidName);
        }
        if self.small_icon == -1 {
            return Err(ConfigError::MissingIcon);
        }
        Ok(())
    }

    /// `DownloadManager.b()` — a description is only required when a version
    /// code was supplied (i.e. the "ask the user" dialog path is used).
    pub fn validate_description(&self) -> Result<(), ConfigError> {
        if self.apk_version_code == UNSET_VERSION_CODE {
            return Ok(());
        }
        if self.apk_description.is_empty() {
            return Err(ConfigError::MissingDescription);
        }
        Ok(())
    }

    pub fn validate(&self) -> Result<(), ConfigError> {
        self.validate_download()?;
        self.validate_description()
    }

    /// True when the available version code is newer than the installed one.
    pub fn is_newer_than(&self, installed_code: i32) -> bool {
        self.apk_version_code != UNSET_VERSION_CODE && self.apk_version_code > installed_code
    }

    /// Portable form of `DownloadManager.download()`: decide what should happen
    /// after validation, without starting any Android service or Activity.
    pub fn decision(&self, installed_code: i32) -> UpdateDecision {
        if self.validate().is_err() {
            return UpdateDecision::Invalid;
        }
        if self.apk_version_code == UNSET_VERSION_CODE || self.apk_description.is_empty() {
            return UpdateDecision::StartDownload;
        }
        if self.is_newer_than(installed_code) {
            UpdateDecision::ShowDialog
        } else if self.show_newer_toast {
            UpdateDecision::ShowNewerToast
        } else {
            UpdateDecision::AlreadyLatest
        }
    }
}

/// Outcomes of `DownloadManager.download()`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UpdateDecision {
    /// Start the download service directly.
    StartDownload,
    /// Show the version dialog and let the user choose.
    ShowDialog,
    /// User is already on the latest version (toast was requested).
    ShowNewerToast,
    /// Already latest, no UI action.
    AlreadyLatest,
    /// Builder validation failed; nothing started.
    Invalid,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn valid() -> ApkUpdateConfig {
        ApkUpdateConfig {
            apk_url: "https://example.test/app.apk".into(),
            apk_name: "gamehub.apk".into(),
            small_icon: 1,
            ..ApkUpdateConfig::default()
        }
    }

    #[test]
    fn constant_helpers_follow_java_templates() {
        assert_eq!(
            UpdatePaths::cache_dir("com.xj.gamehub"),
            "/storage/emulated/0/Android/data/com.xj.gamehub/cache"
        );
        assert_eq!(
            UpdatePaths::file_provider_authority("com.xj.gamehub"),
            "com.xj.gamehub.apkUpdateFileProvider"
        );
    }

    #[test]
    fn download_validation_checks_url_name_and_icon() {
        let mut c = valid();
        assert_eq!(c.validate_download(), Ok(()));
        c.apk_url.clear();
        assert_eq!(c.validate_download(), Err(ConfigError::EmptyUrl));
        c = valid();
        c.apk_name.clear();
        assert_eq!(c.validate_download(), Err(ConfigError::EmptyName));
        c = valid();
        c.apk_name = "update.zip".into();
        assert_eq!(c.validate_download(), Err(ConfigError::InvalidName));
        c = valid();
        c.small_icon = -1;
        assert_eq!(c.validate_download(), Err(ConfigError::MissingIcon));
    }

    #[test]
    fn description_required_only_with_version_code() {
        let mut c = valid();
        assert_eq!(c.validate_description(), Ok(()));
        c.apk_version_code = 518;
        assert_eq!(c.validate_description(), Err(ConfigError::MissingDescription));
        c.apk_description = "new features".into();
        assert_eq!(c.validate(), Ok(()));
    }

    #[test]
    fn update_decision_matches_download_manager() {
        let mut c = valid();
        assert_eq!(c.decision(0), UpdateDecision::StartDownload);

        c.apk_version_code = 518;
        c.apk_description = "new".into();
        assert_eq!(c.decision(517), UpdateDecision::ShowDialog);
        assert_eq!(c.decision(518), UpdateDecision::AlreadyLatest);

        c.show_newer_toast = true;
        assert_eq!(c.decision(518), UpdateDecision::ShowNewerToast);

        c.apk_url.clear();
        assert_eq!(c.decision(0), UpdateDecision::Invalid);
    }

    #[test]
    fn validation_errors_display_readably() {
        assert!(ConfigError::InvalidName.to_string().contains(".apk"));
        assert!(ConfigError::EmptyUrl.to_string().contains("apkUrl"));
    }
}
