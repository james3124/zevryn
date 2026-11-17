//! Rust model of the portable path choices in `PageRouterUtils`.
//!
//! The Java class builds Android `Intent`s via `TheRouter.withRoutePath(path)`
//! (e.g. `f()` -> download manager, `g()`/`l()` -> game library, `k()` -> winemu
//! env settings). Only the route-path selection is modeled here; the
//! `Context`/`Intent` construction is out of scope.

use super::{route_path_for, DeepLinkPage, Route};

/// `/winemu/ui/download/DownloadManageActivity` (PageRouterUtils.f).
pub fn download_manager_route() -> Route {
    Route::new(
        "/winemu/ui/download/DownloadManageActivity",
        "com.xj.winemu.ui.download.DownloadManageActivity",
    )
}

/// `/game/ui/gameli_brary/GameLibraryActivity` (PageRouterUtils.g / l).
pub fn game_library_route() -> Route {
    Route::new(
        "/game/ui/gameli_brary/GameLibraryActivity",
        "com.xj.game.ui.game_library.GameLibraryActivity",
    )
}

/// `/winemu/ui/env_setting/WinEmuEnvSettingActivity` (PageRouterUtils.k).
pub fn winemu_env_setting_route() -> Route {
    Route::new(
        "/winemu/ui/env_setting/WinEmuEnvSettingActivity",
        "com.xj.winemu.ui.env_setting.WinEmuEnvSettingActivity",
    )
}

/// Route object for a deep-link page, using the same path mapping as
/// [`route_path_for`].
pub fn route_for_page(page: DeepLinkPage) -> Route {
    Route::new(route_path_for(page), route_path_for(page))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn helper_routes_use_real_paths() {
        assert_eq!(
            download_manager_route().path,
            "/winemu/ui/download/DownloadManageActivity"
        );
        assert_eq!(
            game_library_route().target,
            "com.xj.game.ui.game_library.GameLibraryActivity"
        );
        assert_eq!(
            route_for_page(DeepLinkPage::GameDetail).path,
            "/launcher/ui/gamedetail/GameDetailActivity"
        );
    }
}
