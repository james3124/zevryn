//! Rust model of `com.xj.winemu.bean.LocalPcGameManifest`.
//!
//! Mirrors the Gson-backed data class declared in
//! `sources/com/xj/winemu/bean/LocalPcGameManifest.java`. The class is the
//! on-disk model for GameHub Lite's own `appmanifest_<id>.json` files (see
//! `ImportPcGameHelper.m`/`v`), *not* Steam's native VDF `.acf` format — a
//! parser for the latter lives in [`super::acf_parser`].
//!
//! Only portable data/accessors are modeled: no Gson annotation, no Android
//! types, no coroutines.

/// On-disk manifest for a locally imported PC game.
///
/// Field order matches the primary Kotlin constructor
/// (`LocalPcGameManifest.java:285-306`). The serialized names are the
/// `@SerializedName` values at lines 26-69.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LocalPcGameManifest {
    /// `@SerializedName("ManifestVersion")` — `getManifestVersion()`.
    pub manifest_version: String,
    /// `@SerializedName("AppId")` — `getAppId()`.
    pub app_id: i64,
    /// `@SerializedName("Name")` — `getName()`.
    pub name: String,
    /// `@SerializedName("Executable")` — `getExecutable()`.
    pub executable: String,
    /// `@SerializedName("Arguments")` — `getArguments()`.
    pub arguments: String,
    /// `@SerializedName("InstallDir")` — `getInstallDir()`.
    pub install_dir: String,
    /// `@SerializedName("SizeOnDisk")` — `getSizeOnDisk()`.
    pub size_on_disk: i64,
    /// `@SerializedName("Source")` — `getSource()`.
    pub source: String,
    /// `@SerializedName("UserId")` — `getUserId()`.
    pub user_id: String,
    /// `@SerializedName("UserName")` — `getUserName()`.
    pub user_name: String,
    /// `@SerializedName("Packaged")` — `getPackaged()`.
    pub packaged: bool,
    /// `@SerializedName("PackagedFiles")` — `getPackagedFiles()`.
    pub packaged_files: Option<Vec<String>>,
}

impl LocalPcGameManifest {
    /// Primary constructor (`LocalPcGameManifest.java:285-306`).
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        manifest_version: impl Into<String>,
        app_id: i64,
        name: impl Into<String>,
        executable: impl Into<String>,
        arguments: impl Into<String>,
        install_dir: impl Into<String>,
        size_on_disk: i64,
        source: impl Into<String>,
        user_id: impl Into<String>,
        user_name: impl Into<String>,
        packaged: bool,
        packaged_files: Option<Vec<String>>,
    ) -> Self {
        Self {
            manifest_version: manifest_version.into(),
            app_id,
            name: name.into(),
            executable: executable.into(),
            arguments: arguments.into(),
            install_dir: install_dir.into(),
            size_on_disk,
            source: source.into(),
            user_id: user_id.into(),
            user_name: user_name.into(),
            packaged,
            packaged_files,
        }
    }

    /// The no-argument constructor (`LocalPcGameManifest.java:71-73`) and the
    /// synthetic defaults applied by the Kotlin default-argument constructor
    /// (lines 308-310): every string is `""`, numbers are `0`, `packaged` is
    /// `false` and `packagedFiles` is `null`.
    pub fn empty() -> Self {
        Self {
            manifest_version: String::new(),
            app_id: 0,
            name: String::new(),
            executable: String::new(),
            arguments: String::new(),
            install_dir: String::new(),
            size_on_disk: 0,
            source: String::new(),
            user_id: String::new(),
            user_name: String::new(),
            packaged: false,
            packaged_files: None,
        }
    }

    // --- Getters (LocalPcGameManifest.java:156-216) -----------------------

    pub fn get_app_id(&self) -> i64 {
        self.app_id
    }
    pub fn get_arguments(&self) -> &str {
        &self.arguments
    }
    pub fn get_executable(&self) -> &str {
        &self.executable
    }
    pub fn get_install_dir(&self) -> &str {
        &self.install_dir
    }
    pub fn get_manifest_version(&self) -> &str {
        &self.manifest_version
    }
    pub fn get_name(&self) -> &str {
        &self.name
    }
    pub fn get_packaged(&self) -> bool {
        self.packaged
    }
    pub fn get_packaged_files(&self) -> Option<&[String]> {
        self.packaged_files.as_deref()
    }
    pub fn get_size_on_disk(&self) -> i64 {
        self.size_on_disk
    }
    pub fn get_source(&self) -> &str {
        &self.source
    }
    pub fn get_user_id(&self) -> &str {
        &self.user_id
    }
    pub fn get_user_name(&self) -> &str {
        &self.user_name
    }

    /// `getHandleExecutable()` (lines 170-173): `replace('\\', '/',
    /// ignoreCase = false)`.
    pub fn get_handle_executable(&self) -> String {
        self.executable.replace('\\', "/")
    }

    // --- Behaviour recovered from ImportPcGameHelper ----------------------

    /// `ImportPcGameHelper.n(...)` (lines 232-238) selects the Downloads
    /// `/SteamGame` root when the manifest version is exactly `"1.0.0"`,
    /// otherwise the external-files root.
    pub fn is_portable_v1(&self) -> bool {
        self.manifest_version == "1.0.0"
    }

    /// `ImportPcGameHelper.s(...)` (lines 256-280) finds the first packaged
    /// file whose name ends with `".zip"` (case-sensitive) and uses it for the
    /// temporary marker file.
    pub fn zip_marker_file(&self) -> Option<&str> {
        self.packaged_files.as_ref()?.iter().find(|f| f.ends_with(".zip")).map(String::as_str)
    }

    // --- Setters (LocalPcGameManifest.java:224-278) -----------------------

    pub fn set_app_id(&mut self, v: i64) {
        self.app_id = v;
    }
    pub fn set_arguments(&mut self, v: impl Into<String>) {
        self.arguments = v.into();
    }
    pub fn set_executable(&mut self, v: impl Into<String>) {
        self.executable = v.into();
    }
    pub fn set_install_dir(&mut self, v: impl Into<String>) {
        self.install_dir = v.into();
    }
    pub fn set_manifest_version(&mut self, v: impl Into<String>) {
        self.manifest_version = v.into();
    }
    pub fn set_name(&mut self, v: impl Into<String>) {
        self.name = v.into();
    }
    pub fn set_packaged(&mut self, v: bool) {
        self.packaged = v;
    }
    pub fn set_packaged_files(&mut self, v: Option<Vec<String>>) {
        self.packaged_files = v;
    }
    pub fn set_size_on_disk(&mut self, v: i64) {
        self.size_on_disk = v;
    }
    pub fn set_source(&mut self, v: impl Into<String>) {
        self.source = v.into();
    }
    pub fn set_user_id(&mut self, v: impl Into<String>) {
        self.user_id = v.into();
    }
    pub fn set_user_name(&mut self, v: impl Into<String>) {
        self.user_name = v.into();
    }
}

impl Default for LocalPcGameManifest {
    fn default() -> Self {
        Self::empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample() -> LocalPcGameManifest {
        LocalPcGameManifest::new(
            "1.0.0",
            570,
            "Dota 2",
            r"bin\win64\dota2.exe",
            "-console",
            "dota 2",
            1_234,
            "Steam",
            "u1",
            "Player",
            true,
            Some(vec!["patch.zip".into(), "readme.txt".into()]),
        )
    }

    #[test]
    fn getters_match_constructor_arguments() {
        let m = sample();
        assert_eq!(m.get_manifest_version(), "1.0.0");
        assert_eq!(m.get_app_id(), 570);
        assert_eq!(m.get_name(), "Dota 2");
        assert_eq!(m.get_executable(), r"bin\win64\dota2.exe");
        assert_eq!(m.get_arguments(), "-console");
        assert_eq!(m.get_install_dir(), "dota 2");
        assert_eq!(m.get_size_on_disk(), 1_234);
        assert_eq!(m.get_source(), "Steam");
        assert_eq!(m.get_user_id(), "u1");
        assert_eq!(m.get_user_name(), "Player");
        assert!(m.get_packaged());
        assert_eq!(m.get_packaged_files().unwrap().len(), 2);
    }

    #[test]
    fn handle_executable_normalizes_backslashes() {
        assert_eq!(sample().get_handle_executable(), "bin/win64/dota2.exe");
    }

    #[test]
    fn portable_v1_and_zip_marker() {
        let m = sample();
        assert!(m.is_portable_v1());
        assert_eq!(m.zip_marker_file(), Some("patch.zip"));
        let mut other = m.clone();
        other.set_manifest_version("2.0.0");
        assert!(!other.is_portable_v1());
        other.set_packaged_files(None);
        assert_eq!(other.zip_marker_file(), None);
    }

    #[test]
    fn default_matches_no_arg_constructor() {
        let m = LocalPcGameManifest::default();
        assert_eq!(m.get_manifest_version(), "");
        assert_eq!(m.get_app_id(), 0);
        assert!(!m.get_packaged());
        assert_eq!(m.get_packaged_files(), None);
    }
}
