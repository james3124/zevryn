//! Plugin lifecycle model and ordered startup list.
//!
//! SOURCE NOTES
//! ------------
//! - `environment/ProgramPlugin.java` is the abstract base: it holds a
//!   `ProgramEnvironment` reference and declares `a()` (start) and `b()`
//!   (stop). We model the two lifecycle transitions as `PluginState`.
//! - `environment/ProgramEnvironment.java` stores plugins in a
//!   `CopyOnWriteArrayList` (insertion order preserved), starts them by calling
//!   `a()` on each, and stops them by calling `b()` on each.
//! - The concrete plugins live in `environment/plugins/*`:
//!   `ALSAServerPlugin`, `PulseAudioPlugin`, `SysVSharedMemoryPlugin`,
//!   `PerfPlugin`, `RenderDocPlugin`, `WMPlugin`, `ProgramLauncher`,
//!   `NetworkInfoUpdatePlugin`.
//! - The **registration/start order** is in
//!   `controller/ProgramController.java`, method `i(String)`, in this sequence:
//!   PerfPlugin, WMPlugin, sound backend (PulseAudio when config mapping == 1,
//!   otherwise ALSAServerPlugin), RenderDocPlugin (when enabled),
//!   SysVSharedMemoryPlugin, NetworkInfoUpdatePlugin, then the
//!   ProgramLauncher(s). This is the order used for [`PLUGIN_STARTUP_ORDER`].
//! - Unix-socket paths are the exact arguments to `UnixSocketConfig.a(root,
//!   path)` in that same file.

/// Portable lifecycle states of a plugin.
///
/// Maps to the source's plugin operations: a freshly constructed plugin is
/// `Loaded`; `ProgramPlugin.a()` moves it to `Started`; `ProgramPlugin.b()`
/// moves it back to `Stopped`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PluginState {
    /// Plugin object constructed but its server/process not started yet.
    Loaded,
    /// `ProgramPlugin.a()` has been called (server thread/process running).
    Started,
    /// `ProgramPlugin.b()` has been called (server/stopped, resources freed).
    Stopped,
}

/// Identifies each concrete plugin from `environment/plugins/*`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PluginKind {
    /// `ALSAServerPlugin` - ALSA over unix socket.
    AlsaServer,
    /// `PulseAudioPlugin` - PulseAudio daemon (alternative to ALSA).
    PulseAudio,
    /// `SysVSharedMemoryPlugin` - System-V shared memory server.
    SysVSharedMemory,
    /// `PerfPlugin` - performance monitoring server.
    Perf,
    /// `RenderDocPlugin` - RenderDoc server.
    RenderDoc,
    /// `WMPlugin` - window manager callback server.
    Wm,
    /// `ProgramLauncher` - launches wine/box64/jwm/steam processes.
    ProgramLauncher,
    /// `NetworkInfoUpdatePlugin` - writes network info files on connectivity.
    NetworkInfo,
}

impl PluginKind {
    /// Source class name for this plugin kind.
    pub fn class_name(self) -> &'static str {
        match self {
            Self::AlsaServer => "ALSAServerPlugin",
            Self::PulseAudio => "PulseAudioPlugin",
            Self::SysVSharedMemory => "SysVSharedMemoryPlugin",
            Self::Perf => "PerfPlugin",
            Self::RenderDoc => "RenderDocPlugin",
            Self::Wm => "WMPlugin",
            Self::ProgramLauncher => "ProgramLauncher",
            Self::NetworkInfo => "NetworkInfoUpdatePlugin",
        }
    }
}

/// Exact unix-socket path suffixes passed to `UnixSocketConfig.a(root, path)`
/// in `ProgramController.java` / plugin files.
pub const SOCKET_PATH_PERF: &str = "/tmp/.perf/V0";
pub const SOCKET_PATH_WM: &str = "/tmp/.wm/V0";
pub const SOCKET_PATH_SOUND_PULSE: &str = "/tmp/.sound/PS0";
pub const SOCKET_PATH_SOUND_ALSA: &str = "/tmp/.sound/AS0";
pub const SOCKET_PATH_RENDERDOC: &str = "/tmp/.rdc/V0";
pub const SOCKET_PATH_SYSV: &str = "/tmp/.sysvshm/SM0";

/// Source registration order from `ProgramController.i(String)`.
///
/// `PulseAudio` and `AlsaServer` are mutually exclusive: exactly one of them is
/// selected (`j(String)` returns PulseAudio when the config mapping equals 1,
/// otherwise ALSA). Both slots are shown here so the ordered list covers every
/// plugin in `environment/plugins/*`. RenderDoc is registered only when
/// enabled; trailing `ProgramLauncher` entries may be repeated (jwm, main
/// program, steam).
pub const PLUGIN_STARTUP_ORDER: [PluginKind; 8] = [
    PluginKind::Perf,
    PluginKind::Wm,
    PluginKind::AlsaServer,
    PluginKind::PulseAudio,
    PluginKind::RenderDoc,
    PluginKind::SysVSharedMemory,
    PluginKind::NetworkInfo,
    PluginKind::ProgramLauncher,
];

/// Every plugin kind, for enumeration purposes.
pub const ALL_PLUGIN_KINDS: [PluginKind; 8] = PLUGIN_STARTUP_ORDER;

/// A portable descriptor for one plugin instance.
///
/// Mirrors the fields that matter for lifecycle modeling: which plugin it is,
/// its unix-socket path (if any) as configured in `ProgramController`, and its
/// current [`PluginState`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProgramPlugin {
    pub kind: PluginKind,
    /// Unix-socket path suffix, for plugins that create one.
    pub socket_path: Option<&'static str>,
    pub state: PluginState,
}

impl ProgramPlugin {
    /// Construct a plugin descriptor in the `Loaded` state.
    pub fn new(kind: PluginKind) -> Self {
        Self {
            kind,
            socket_path: socket_path_for(kind),
            state: PluginState::Loaded,
        }
    }

    /// Build descriptors for all plugins in source startup order.
    pub fn startup_sequence() -> Vec<Self> {
        PLUGIN_STARTUP_ORDER.iter().copied().map(Self::new).collect()
    }

    /// Model `ProgramPlugin.a()` (start). Idempotent: a started plugin stays
    /// started, matching the source guards like `if (this.f20749b != null) return;`.
    pub fn start(&mut self) {
        if self.state != PluginState::Started {
            self.state = PluginState::Started;
        }
    }

    /// Model `ProgramPlugin.b()` (stop). Idempotent.
    pub fn stop(&mut self) {
        self.state = PluginState::Stopped;
    }
}

/// Returns the configured socket path suffix for a plugin kind, or `None` for
/// plugins that do not open a unix socket (`NetworkInfoUpdatePlugin`,
/// `ProgramLauncher`).
pub fn socket_path_for(kind: PluginKind) -> Option<&'static str> {
    match kind {
        PluginKind::Perf => Some(SOCKET_PATH_PERF),
        PluginKind::Wm => Some(SOCKET_PATH_WM),
        PluginKind::PulseAudio => Some(SOCKET_PATH_SOUND_PULSE),
        PluginKind::AlsaServer => Some(SOCKET_PATH_SOUND_ALSA),
        PluginKind::RenderDoc => Some(SOCKET_PATH_RENDERDOC),
        PluginKind::SysVSharedMemory => Some(SOCKET_PATH_SYSV),
        PluginKind::NetworkInfo | PluginKind::ProgramLauncher => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn startup_order_matches_program_controller() {
        assert_eq!(
            PLUGIN_STARTUP_ORDER,
            [
                PluginKind::Perf,
                PluginKind::Wm,
                PluginKind::AlsaServer,
                PluginKind::PulseAudio,
                PluginKind::RenderDoc,
                PluginKind::SysVSharedMemory,
                PluginKind::NetworkInfo,
                PluginKind::ProgramLauncher,
            ]
        );
    }

    #[test]
    fn lifecycle_transitions() {
        let mut p = ProgramPlugin::new(PluginKind::Perf);
        assert_eq!(p.state, PluginState::Loaded);
        p.start();
        assert_eq!(p.state, PluginState::Started);
        p.start(); // idempotent
        assert_eq!(p.state, PluginState::Started);
        p.stop();
        assert_eq!(p.state, PluginState::Stopped);
    }

    #[test]
    fn socket_paths_are_exact() {
        assert_eq!(socket_path_for(PluginKind::Perf), Some("/tmp/.perf/V0"));
        assert_eq!(socket_path_for(PluginKind::Wm), Some("/tmp/.wm/V0"));
        assert_eq!(socket_path_for(PluginKind::PulseAudio), Some("/tmp/.sound/PS0"));
        assert_eq!(socket_path_for(PluginKind::AlsaServer), Some("/tmp/.sound/AS0"));
        assert_eq!(socket_path_for(PluginKind::RenderDoc), Some("/tmp/.rdc/V0"));
        assert_eq!(socket_path_for(PluginKind::SysVSharedMemory), Some("/tmp/.sysvshm/SM0"));
        assert_eq!(socket_path_for(PluginKind::NetworkInfo), None);
        assert_eq!(socket_path_for(PluginKind::ProgramLauncher), None);
    }

    #[test]
    fn startup_sequence_uses_order_and_starts_loaded() {
        let seq = ProgramPlugin::startup_sequence();
        assert_eq!(seq.len(), 8);
        assert_eq!(seq[0].kind, PluginKind::Perf);
        assert_eq!(seq[7].kind, PluginKind::ProgramLauncher);
        assert!(seq.iter().all(|p| p.state == PluginState::Loaded));
    }

    #[test]
    fn class_names_match_source_classes() {
        assert_eq!(PluginKind::Wm.class_name(), "WMPlugin");
        assert_eq!(PluginKind::RenderDoc.class_name(), "RenderDocPlugin");
        assert_eq!(PluginKind::NetworkInfo.class_name(), "NetworkInfoUpdatePlugin");
    }
}
