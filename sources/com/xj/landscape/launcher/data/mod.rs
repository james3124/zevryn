//! Rust model of `com.xj.landscape.launcher.data` (portable subset).
//!
//! The repositories themselves are coroutine/OkHttp wrappers; only their
//! endpoint catalogue and the signed-request shape are modeled here. See
//! `REPORT.md` in `com/xj/common/http/`.

#[path = "../../../common/http/mod.rs"]
pub mod http;

pub mod repository;

pub use repository::{build_repo_request, RepoEndpoint, REPO_ENDPOINTS};
