//! Rust model of `com.xj.common.router` (portable routing logic only).
//!
//! Mirrors `sources/com/xj/common/router/` from GameHub Lite v5.1.8. The
//! framework-level router types are included by relative `#[path]` so this
//! area is self-contained; Android `Intent`/`Activity` navigation and
//! annotation processing are intentionally out of scope.

// Include the framework router with a path relative to this file so the
// module compiles regardless of how the crate root names its top-level
// modules. (app/sources/com/xj/common/router -> app/sources/com/therouter)
#[path = "../../../therouter/mod.rs"]
pub mod therouter;

pub mod common_router_table;
pub mod page_router_utils;
pub mod route_request;

pub use self::common_router_table::{
    build_route_table, RouteDef, ROUTES, THEROUTER_APT_VERSION,
};
pub use self::page_router_utils::{
    download_manager_route, game_library_route, route_for_page, winemu_env_setting_route,
};
pub use self::route_request::{route_path_for, DeepLinkPage, RouteRequest};
pub use self::therouter::{
    default_provider, extract_placeholders, ResolvedRoute, Route, RouteTable, ServiceProvider,
};
