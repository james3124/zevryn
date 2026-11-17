//! Rust model of the generated `ServiceProvider__TheRouter__*.java` classes.
//!
//! Every generated class implements `Interceptor.interception(Class, args)`,
//! returning a concrete implementation when the requested interface matches.
//! The Android object construction is out of scope; what is portable is the
//! interface-name -> implementation-name binding itself.

/// Registry mapping an interface name to its implementation class name.
#[derive(Debug, Clone, Default)]
pub struct ServiceProvider {
    bindings: Vec<(String, String)>,
}

impl ServiceProvider {
    pub fn new() -> Self {
        Self {
            bindings: Vec::new(),
        }
    }

    /// Register (or replace) the implementation bound to `interface`.
    pub fn register(
        &mut self,
        interface: impl Into<String>,
        implementation: impl Into<String>,
    ) {
        let interface = interface.into();
        if let Some(slot) = self.bindings.iter_mut().find(|(name, _)| name == &interface) {
            slot.1 = implementation.into();
        } else {
            self.bindings.push((interface, implementation.into()));
        }
    }

    /// Look up the implementation name for `interface`.
    pub fn get(&self, interface: &str) -> Option<&str> {
        self.bindings
            .iter()
            .find(|(name, _)| name == interface)
            .map(|(_, implementation)| implementation.as_str())
    }

    pub fn len(&self) -> usize {
        self.bindings.len()
    }

    pub fn is_empty(&self) -> bool {
        self.bindings.is_empty()
    }

    pub fn bindings(&self) -> &[(String, String)] {
        &self.bindings
    }
}

/// Registry pre-loaded with every interface -> implementation binding found
/// in the eleven generated `ServiceProvider__TheRouter__*.java` files.
pub fn default_provider() -> ServiceProvider {
    let mut provider = ServiceProvider::new();

    // ServiceProvider__TheRouter__549289766
    provider.register("IHomeService", "com.xj.landscape.launcher.soa.HomeService");
    provider.register(
        "IDeviceManagerService",
        "com.xj.landscape.launcher.soa.IDeviceManagerServiceImpl",
    );
    provider.register(
        "ILandscapeLauncherNavService",
        "com.xj.landscape.launcher.soa.ILandscapeLauncherNavServiceImpl",
    );

    // ServiceProvider__TheRouter__573075289
    provider.register("IAppService", "com.xj.app.soa.IAppServiceImpl");

    // ServiceProvider__TheRouter__634971008
    provider.register("IUmengService", "com.xj.umeng.soa.IUmengServiceImpl");

    // ServiceProvider__TheRouter__742311270
    provider.register("IGameModuleService", "com.xj.game.service.GameModuleService");

    // ServiceProvider__TheRouter__892010023
    provider.register("IWinEmuDataService", "com.xj.winemu.WinEmuDataService");
    provider.register("IWinEmuService", "com.xj.winemu.WinEmuServiceImpl");
    provider.register(
        "IWinEmuDownloadService",
        "com.xj.winemu.download.WinEmuDownloadService",
    );

    // ServiceProvider__TheRouter__911463050
    provider.register("ISteamGameService", "com.xj.module.steam.SteamGameService");
    provider.register("ISteamService", "com.xj.module.steam.SteamServiceImpl");

    // ServiceProvider__TheRouter__923475518
    provider.register("IPushService", "com.xj.push.PushService");

    // ServiceProvider__TheRouter__969355976
    provider.register("IMappingService", "com.xj.mapping.IMappingServiceImpl");

    // ServiceProvider__TheRouter__1344554815
    provider.register(
        "IApkUpdateService",
        "com.xj.apk.update.soa.IApkUpdateServiceImpl",
    );

    // ServiceProvider__TheRouter__2000493684
    provider.register(
        "IDeviceSettingService",
        "com.xj.devicesetting.IDeviceSettingServiceImpl",
    );

    // ServiceProvider__TheRouter__2056161029
    provider.register(
        "IPCStreamService",
        "com.xj.module_pcstream.soa.IPCStreamServiceImpl",
    );
    provider.register(
        "IPCStreamHelperService",
        "com.xj.module_pcstream.soa.PCStreamHelperService",
    );

    provider
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lookup_known_interfaces() {
        let provider = default_provider();
        assert_eq!(
            provider.get("IHomeService"),
            Some("com.xj.landscape.launcher.soa.HomeService")
        );
        assert_eq!(
            provider.get("ILandscapeLauncherNavService"),
            Some("com.xj.landscape.launcher.soa.ILandscapeLauncherNavServiceImpl")
        );
        assert_eq!(provider.get("INotRegistered"), None);
    }

    #[test]
    fn register_replaces_existing_binding() {
        let mut provider = ServiceProvider::new();
        assert!(provider.is_empty());
        provider.register("IFoo", "ImplA");
        provider.register("IFoo", "ImplB");
        assert_eq!(provider.len(), 1);
        assert_eq!(provider.get("IFoo"), Some("ImplB"));
    }

    #[test]
    fn default_registry_has_all_bindings() {
        assert_eq!(default_provider().len(), 17);
    }
}
