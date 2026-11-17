//! A tiny, std-only route registry with `{name}` placeholder matching.
//!
//! Resolution follows the spirit of `RegexpKeyedMap.get`: a `?query` suffix is
//! split off before matching, exact paths win over placeholder patterns, and
//! the captured placeholder values plus the query pairs are exposed on the
//! resolved result.

use super::route::Route;

/// A route together with the parameter values captured while resolving it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResolvedRoute {
    /// The registered route (pattern, target and declared params).
    pub route: Route,
    /// `(name, value)` pairs: `{name}` captures first, then `?query` pairs.
    pub params: Vec<(String, String)>,
}

impl ResolvedRoute {
    /// First value bound to `key`, if any.
    pub fn param(&self, key: &str) -> Option<&str> {
        self.params
            .iter()
            .find(|(name, _)| name == key)
            .map(|(_, value)| value.as_str())
    }

    pub fn target(&self) -> &str {
        &self.route.target
    }

    pub fn path(&self) -> &str {
        &self.route.path
    }
}

/// Registry of routes, keyed by path pattern.
#[derive(Debug, Clone, Default)]
pub struct RouteTable {
    routes: Vec<Route>,
}

impl RouteTable {
    pub fn new() -> Self {
        Self { routes: Vec::new() }
    }

    /// Register a route. Re-registering the same path pattern replaces it.
    pub fn register(&mut self, route: Route) {
        if let Some(slot) = self.routes.iter_mut().find(|r| r.path == route.path) {
            *slot = route;
        } else {
            self.routes.push(route);
        }
    }

    /// Register many routes at once.
    pub fn register_all<I: IntoIterator<Item = Route>>(&mut self, routes: I) {
        for route in routes {
            self.register(route);
        }
    }

    pub fn len(&self) -> usize {
        self.routes.len()
    }

    pub fn is_empty(&self) -> bool {
        self.routes.is_empty()
    }

    pub fn routes(&self) -> &[Route] {
        &self.routes
    }

    /// Resolve `path` (optionally with a `?query`) against the registered
    /// routes. Exact patterns are tried before `{name}` placeholder patterns.
    pub fn resolve(&self, path: &str) -> Option<ResolvedRoute> {
        let (base, query) = match path.find('?') {
            Some(index) => (&path[..index], &path[index + 1..]),
            None => (path, ""),
        };
        let parts = split_segments(base);

        // Fast path: a verbatim registered path (RegexpKeyedMap's super.get).
        for route in &self.routes {
            if !route.params.is_empty() {
                continue;
            }
            if split_segments(&route.path) == parts {
                return Some(build_resolved(route, Vec::new(), query));
            }
        }

        // Fallback: placeholder patterns, tried in registration order.
        for route in &self.routes {
            if route.params.is_empty() {
                continue;
            }
            if let Some(captured) = match_pattern(&route.path, &parts) {
                return Some(build_resolved(route, captured, query));
            }
        }

        None
    }
}

/// Split a path into non-empty segments, ignoring leading/trailing slashes.
fn split_segments(path: &str) -> Vec<&str> {
    path.trim_matches('/')
        .split('/')
        .filter(|segment| !segment.is_empty())
        .collect()
}

/// If `segment` is exactly `{name}`, return `name`.
fn placeholder_name(segment: &str) -> Option<&str> {
    if segment.len() >= 2 && segment.starts_with('{') && segment.ends_with('}') {
        let name = &segment[1..segment.len() - 1];
        if !name.is_empty() {
            return Some(name);
        }
    }
    None
}

/// Match a placeholder pattern against already-split path segments.
fn match_pattern(pattern: &str, parts: &[&str]) -> Option<Vec<(String, String)>> {
    let pattern_parts = split_segments(pattern);
    if pattern_parts.len() != parts.len() {
        return None;
    }
    let mut captured = Vec::new();
    for (pattern_part, value) in pattern_parts.iter().zip(parts.iter()) {
        if let Some(name) = placeholder_name(pattern_part) {
            if value.is_empty() {
                return None;
            }
            captured.push((name.to_string(), (*value).to_string()));
        } else if pattern_part != value {
            return None;
        }
    }
    Some(captured)
}

/// Combine captured placeholders with `?query` key/value pairs.
fn build_resolved(
    route: &Route,
    mut params: Vec<(String, String)>,
    query: &str,
) -> ResolvedRoute {
    for pair in query.split('&') {
        if pair.is_empty() {
            continue;
        }
        let (key, value) = pair.split_once('=').unwrap_or((pair, ""));
        params.push((key.to_string(), value.to_string()));
    }
    ResolvedRoute {
        route: route.clone(),
        params,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn resolves_exact_route_with_query_params() {
        let mut table = RouteTable::new();
        table.register(Route::new(
            "/launcher/ui/gamedetail/GameDetailActivity",
            "com.xj.landscape.launcher.ui.gamedetail.GameDetailActivity",
        ));

        let got = table
            .resolve("/launcher/ui/gamedetail/GameDetailActivity?id=42&type=1")
            .expect("exact route should resolve");

        assert_eq!(
            got.target(),
            "com.xj.landscape.launcher.ui.gamedetail.GameDetailActivity"
        );
        assert_eq!(got.param("id"), Some("42"));
        assert_eq!(got.param("type"), Some("1"));
        assert_eq!(got.param("missing"), None);
    }

    #[test]
    fn resolves_placeholder_params() {
        let mut table = RouteTable::new();
        table.register(Route::new("/user/{uid}/game/{game_id}", "DemoTarget"));

        let got = table
            .resolve("/user/7/game/abc?from=deeplink")
            .expect("placeholder route should resolve");

        assert_eq!(got.target(), "DemoTarget");
        assert_eq!(got.param("uid"), Some("7"));
        assert_eq!(got.param("game_id"), Some("abc"));
        assert_eq!(got.param("from"), Some("deeplink"));

        // A missing segment must not match.
        assert!(table.resolve("/user/7/game").is_none());
        assert!(table.resolve("/user/7/game/abc/extra").is_none());
    }

    #[test]
    fn exact_route_wins_over_placeholder() {
        let mut table = RouteTable::new();
        table.register(Route::new("/a/{x}", "Placeholder"));
        table.register(Route::new("/a/b", "Exact"));
        assert_eq!(table.resolve("/a/b").unwrap().target(), "Exact");
        assert_eq!(table.resolve("/a/c").unwrap().target(), "Placeholder");
    }
}
