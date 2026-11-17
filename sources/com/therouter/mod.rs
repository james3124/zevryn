//! Rust model of the portable parts of `com.therouter`.
//!
//! Mirrors the framework types exercised by GameHub Lite v5.1.8's generated
//! route/service tables: a [`Route`] with `{name}` placeholders, a [`RouteTable`]
//! with register/resolve, and a [`ServiceProvider`] interface->implementation
//! registry. Android navigation (`Navigator`, `Intent`, `Context`) and
//! annotation processing are intentionally out of scope.

pub mod route;
pub mod route_table;
pub mod service_provider;

pub use self::route::{extract_placeholders, Route};
pub use self::route_table::{ResolvedRoute, RouteTable};
pub use self::service_provider::{default_provider, ServiceProvider};
