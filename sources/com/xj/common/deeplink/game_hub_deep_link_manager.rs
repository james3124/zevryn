//! Portable portion of `com.xj.common.deeplink.GameHubDeepLinkManager`.
//!
//! The original manager is an Android coroutine dispatcher: it caches a
//! pending `Intent`, asks `IHomeService` whether the main page is ready, then
//! forwards the query parameters to a registered `DeepLinkHandler`. None of
//! that is portable. The part we keep here is pure URI parsing: turn a
//! `gamesir://gamehub?page_type=...` string into a [`DeepLink`] with
//! percent-decoded parameters. This mirrors the core crate's `deeplink`
//! module so the two implementations stay compatible.

use super::gh_deep_link_page_type::DeepLinkPage;

/// Prefix every handled deep link must start with (scheme + host).
pub const SCHEME_HOST: &str = "gamesir://gamehub";

/// A parsed deep link: the page plus every query parameter (percent-decoded).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DeepLink {
    pub page: DeepLinkPage,
    pub params: Vec<(String, String)>,
}

impl DeepLink {
    /// First value bound to `key`, if present.
    pub fn param(&self, key: &str) -> Option<&str> {
        self.params
            .iter()
            .find(|(k, _)| k == key)
            .map(|(_, v)| v.as_str())
    }
}

/// Why a link could not be parsed.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParseError(pub String);

/// Parse a `gamesir://gamehub?...` deep link.
pub fn parse(input: &str) -> Result<DeepLink, ParseError> {
    let rest = input
        .strip_prefix(SCHEME_HOST)
        .ok_or_else(|| ParseError("not a gamesir://gamehub link".into()))?;
    let query = rest.strip_prefix('?').unwrap_or("");
    let mut params = Vec::new();
    for pair in query.split('&') {
        if pair.is_empty() {
            continue;
        }
        let (k, v) = pair.split_once('=').unwrap_or((pair, ""));
        params.push((percent_decode(k), percent_decode(v)));
    }
    let page = params
        .iter()
        .find(|(k, _)| k == "page_type")
        .and_then(|(_, v)| DeepLinkPage::from_str(v))
        .ok_or_else(|| ParseError("missing or unknown page_type".into()))?;
    Ok(DeepLink { page, params })
}

/// Decode `%XX` escapes with UTF-8 lossy fallback (std only).
fn percent_decode(s: &str) -> String {
    let bytes = s.as_bytes();
    let mut out = Vec::with_capacity(bytes.len());
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'%' && i + 2 < bytes.len() {
            if let (Some(a), Some(b)) = (hex(bytes[i + 1]), hex(bytes[i + 2])) {
                out.push(a * 16 + b);
                i += 3;
                continue;
            }
        }
        out.push(bytes[i]);
        i += 1;
    }
    String::from_utf8_lossy(&out).into_owned()
}

fn hex(b: u8) -> Option<u8> {
    match b {
        b'0'..=b'9' => Some(b - b'0'),
        b'a'..=b'f' => Some(b - b'a' + 10),
        b'A'..=b'F' => Some(b - b'A' + 10),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_deep_link() {
        let d = parse("gamesir://gamehub?page_type=game_detail&game_id=123&title=A%20Game").unwrap();
        assert_eq!(d.page, DeepLinkPage::GameDetail);
        assert_eq!(d.param("game_id"), Some("123"));
        assert_eq!(d.param("title"), Some("A Game"));
    }

    #[test]
    fn parses_every_page_type() {
        for page in DeepLinkPage::ALL {
            let input = format!("gamesir://gamehub?page_type={}", page.as_str());
            assert_eq!(parse(&input).unwrap().page, page);
        }
    }

    #[test]
    fn percent_decodes_keys_and_values_including_uppercase_hex() {
        let d = parse("gamesir://gamehub?page%5Ftype=web%5Furl&url=https%3A%2F%2Fexample.com%2Fa%3Fb%3D1").unwrap();
        assert_eq!(d.page, DeepLinkPage::WebUrl);
        assert_eq!(d.param("page_type"), Some("web_url"));
        assert_eq!(d.param("url"), Some("https://example.com/a?b=1"));
    }

    #[test]
    fn leaves_unknown_escapes_and_plus_signs_untouched() {
        let d = parse("gamesir://gamehub?page_type=user_info&x=%ZZ&y=a+b").unwrap();
        assert_eq!(d.param("x"), Some("%ZZ"));
        assert_eq!(d.param("y"), Some("a+b"));
    }

    #[test]
    fn accepts_no_query_and_missing_values() {
        let d = parse("gamesir://gamehub?page_type=main_page&flag").unwrap();
        assert_eq!(d.page, DeepLinkPage::MainPage);
        assert_eq!(d.param("flag"), Some(""));
    }

    #[test]
    fn rejects_bad_links() {
        assert!(parse("https://example.com").is_err());
        assert!(parse("gamesir://other?page_type=game_detail").is_err());
        assert!(parse("gamesir://gamehub").is_err());
        assert!(parse("gamesir://gamehub?page_type=nope").is_err());
        assert!(parse("gamesir://gamehub?foo=bar").is_err());
    }
}
