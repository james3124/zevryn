//! Portable model of the request-building shape in the signing interceptors
//! (`EggGameTokenInterceptor` and the adb/wifiui `TokenInterceptor`).
//!
//! Both interceptors take a JSON POST body, add `clientparams`, `time` and
//! `sign`, re-serialize it, and send it as the new body. The active one
//! (`com.xj.common.http.EggGameTokenInterceptor`) computes `sign` from the
//! canonical `key=value` string via `SignUtils`; the legacy adb interceptor
//! does the same but skips the exact `https://clientgsw.vgabc.com/clientapi/`
//! URL. No actual HTTP transport is modeled — see `REPORT.md`.

use std::collections::BTreeMap;

use super::api_config::ApiConfig;
use super::json::object_to_json;
use super::sign_utils::sign_string;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HttpMethod {
    Get,
    Post,
}

impl HttpMethod {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Get => "GET",
            Self::Post => "POST",
        }
    }
}

/// A request after the interceptor has rewritten it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SignedRequest {
    pub method: HttpMethod,
    pub url: String,
    /// Every body field, including `clientparams`, `time` and `sign`.
    pub fields: BTreeMap<String, String>,
    /// The signature computed over the fields before `sign` was inserted.
    pub sign: String,
}

impl SignedRequest {
    /// Build the canonical `key=value&...` string used as the MD5 input.
    /// `sign` itself is excluded (it is not present yet at signing time).
    pub fn canonical_string(fields: &BTreeMap<String, String>) -> String {
        fields
            .iter()
            .map(|(k, v)| format!("{}={}", k, v))
            .collect::<Vec<_>>()
            .join("&")
    }

    /// Apply the signing interceptor to a POST body.
    ///
    /// Returns `None` for a sign-exempt URL (the legacy clientapi endpoint),
    /// matching the adb interceptor's early "no sign" branch.
    pub fn build_post(
        url: impl Into<String>,
        mut fields: BTreeMap<String, String>,
        clientparams: &str,
        time_ms: i64,
    ) -> Option<Self> {
        let url = url.into();
        if ApiConfig::is_sign_exempt(&url) {
            return None;
        }
        fields.insert("clientparams".into(), clientparams.to_owned());
        fields.insert("time".into(), time_ms.to_string());
        let sign = sign_string(&fields);
        fields.insert("sign".into(), sign.clone());
        Some(Self {
            method: HttpMethod::Post,
            url,
            fields,
            sign,
        })
    }

    /// Add the extra fields used by the deprecated common `TokenInterceptor`
    /// (`uid`, `session`, `version`, `language`, `format`). These are added
    /// after signing, so they do not affect `sign`.
    pub fn with_session_fields(mut self, extra: &[(&str, &str)]) -> Self {
        for (k, v) in extra {
            self.fields.insert((*k).to_owned(), (*v).to_owned());
        }
        self
    }

    /// Serialize the rewritten body as JSON (string values), as the
    /// interceptors hand it back to OkHttp's `RequestBody.create(..., JSON)`.
    pub fn body_json(&self) -> String {
        let pairs: Vec<(String, String)> = self
            .fields
            .iter()
            .map(|(k, v)| (k.clone(), v.clone()))
            .collect();
        object_to_json(&pairs)
    }

    /// The body fields in insertion order used for logging (sorted, like the
    /// `TreeMap` the signer iterates).
    pub fn field(&self, key: &str) -> Option<&str> {
        self.fields.get(key).map(String::as_str)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn builds_signed_post_body() {
        let mut fields = BTreeMap::new();
        fields.insert("game_name".into(), "Hollow Knight".into());
        let req = SignedRequest::build_post(
            "https://gamehub-lite-api.emuready.workers.dev/card/getGameDetail",
            fields,
            "5.1.8|14|zh|model",
            1_700_000_000_000,
        )
        .unwrap();

        assert_eq!(req.method, HttpMethod::Post);
        assert_eq!(req.field("clientparams"), Some("5.1.8|14|zh|model"));
        assert_eq!(req.field("time"), Some("1700000000000"));
        assert_eq!(req.field("sign"), Some(req.sign.as_str()));
        // sign is derived from canonical string WITHOUT the sign field itself.
        let mut without_sign = req.fields.clone();
        without_sign.remove("sign");
        assert_eq!(req.sign, sign_string(&without_sign));
    }

    #[test]
    fn exempt_url_is_not_signed() {
        let fields = BTreeMap::new();
        assert!(SignedRequest::build_post(
            super::super::api_config::LEGACY_CLIENT_API,
            fields,
            "cp",
            1,
        )
        .is_none());
    }

    #[test]
    fn body_json_contains_all_fields() {
        let req = SignedRequest::build_post("https://x/", BTreeMap::new(), "cp", 7).unwrap();
        let json = req.body_json();
        assert!(json.contains("\"clientparams\":\"cp\""));
        assert!(json.contains("\"time\":\"7\""));
        assert!(json.contains("\"sign\""));
    }

    #[test]
    fn extra_fields_do_not_change_sign() {
        let req = SignedRequest::build_post("https://x/", BTreeMap::new(), "cp", 7).unwrap();
        let before = req.sign.clone();
        let req = req.with_session_fields(&[("uid", "42"), ("format", "JSON")]);
        assert_eq!(req.sign, before);
        assert_eq!(req.field("uid"), Some("42"));
    }
}
