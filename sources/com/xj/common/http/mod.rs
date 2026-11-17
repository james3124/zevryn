//! Rust model of the `com.xj.common.http` API layer (portable logic only).
//!
//! Mirrors `sources/com/xj/common/http/` from GameHub Lite v5.1.8: the base
//! URL config, the `clientparams` builder, the signer and the request-rewrite
//! shape of the token interceptors. Android/OkHttp plumbing (timeouts, cache,
//! cookie jar, coroutines, real network I/O) is intentionally omitted; see
//! `REPORT.md`.

// Share the single JSON reader from the data module (no duplicate copy).
use crate::data::json;

pub mod api_config;
pub mod client_params;
pub mod sign_utils;
pub mod signed_request;

pub use api_config::{ApiConfig, BASE_URL};
pub use client_params::ClientParams;
pub use sign_utils::{md5_hex, sign_string, SIGN_SALT};
pub use signed_request::{HttpMethod, SignedRequest};
