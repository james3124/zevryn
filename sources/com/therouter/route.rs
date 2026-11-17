//! Rust model of `com.therouter.router.RouteItem` / the `@Route` annotation.
//!
//! A route binds a path pattern to a target class name. Path segments written
//! as `{name}` are placeholders that capture a single segment at resolve time
//! (e.g. `/user/{id}`). This is a std-only, framework-free stand-in for the
//! `RouteItem` objects registered by TheRouter's generated route maps.

/// A single route: a path pattern, a target class name and declared params.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Route {
    /// Path pattern. A `{name}` segment matches any single non-empty segment.
    pub path: String,
    /// Fully-qualified target class name (the generated `className` field).
    pub target: String,
    /// Placeholder / declared parameter names, in declaration order.
    pub params: Vec<String>,
}

impl Route {
    /// Build a route, automatically extracting `{name}` placeholders from the
    /// path into `params`.
    pub fn new(path: impl Into<String>, target: impl Into<String>) -> Self {
        let path = path.into();
        let params = extract_placeholders(&path);
        Self {
            path,
            target: target.into(),
            params,
        }
    }

    /// Attach extra declared parameter names. Mirrors the `params` field of
    /// the generated route JSON, which is empty `{}` for every GameHub route.
    pub fn with_params(mut self, params: &[&str]) -> Self {
        for param in params {
            if !self.params.iter().any(|existing| existing == param) {
                self.params.push((*param).to_string());
            }
        }
        self
    }

    /// Whether this path pattern contains at least one `{name}` placeholder.
    pub fn has_placeholders(&self) -> bool {
        !self.params.is_empty()
    }
}

impl core::fmt::Display for Route {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_str(&self.path)
    }
}

/// Collect every `{name}` placeholder found in `pattern`, in first-seen order.
pub fn extract_placeholders(pattern: &str) -> Vec<String> {
    let mut out: Vec<String> = Vec::new();
    let bytes = pattern.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'{' {
            if let Some(offset) = pattern[i + 1..].find('}') {
                let name = &pattern[i + 1..i + 1 + offset];
                if !name.is_empty() && !out.iter().any(|existing| existing == name) {
                    out.push(name.to_string());
                }
                i += offset + 2; // skip past the closing '}'
                continue;
            }
        }
        i += 1;
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extracts_placeholders_in_order_and_deduplicates() {
        assert_eq!(
            extract_placeholders("/game/{id}/player/{name}/game/{id}"),
            vec!["id", "name"]
        );
        assert!(extract_placeholders("/launcher/ui/gamedetail/GameDetailActivity").is_empty());
    }

    #[test]
    fn route_new_records_placeholders() {
        let route = Route::new("/user/{uid}", "Demo");
        assert_eq!(route.params, vec!["uid".to_string()]);
        assert!(route.has_placeholders());
    }
}
