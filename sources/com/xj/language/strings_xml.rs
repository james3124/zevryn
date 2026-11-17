//! Minimal, dependency-free reader for Android `res/values/strings.xml`.
//!
//! Only the portable subset needed by the Rust port is implemented:
//! * `<string name="...">value</string>` elements (the only element the
//!   catalog consumes), including self-closing `<string name="..." />`.
//! * single- or double-quoted attribute values, with extra attributes such as
//!   `formatted="false"` / `translatable="false"` ignored.
//! * XML character entities (`&amp;`, `&lt;`, `&gt;`, `&quot;`, `&apos;`,
//!   `&nbsp;`, `&#NN;`, `&#xHH;`) and Android backslash escapes
//!   (`\'`, `\"`, `\\`, `\n`, `\t`, `\r`, `\@`, `\?`, `\uXXXX`).
//! * `<!-- ... -->` comments and other resource elements are skipped.
//!
//! Decoding is done in a single pass so an entity produced by another entity
//! (e.g. `"&amp;apos;"`) is *not* decoded twice, matching aapt behaviour.

use std::collections::HashMap;
use std::fmt;

/// A malformed `<string>` element.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParseError {
    pub message: String,
    pub byte_offset: usize,
}

impl ParseError {
    fn new(message: impl Into<String>, byte_offset: usize) -> Self {
        Self {
            message: message.into(),
            byte_offset,
        }
    }
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "strings.xml parse error at byte {}: {}",
            self.byte_offset, self.message
        )
    }
}

impl std::error::Error for ParseError {}

/// Parse an Android `strings.xml` document into a `name -> value` map.
///
/// Duplicate names keep the last occurrence, mirroring how aapt would report
/// a duplicate resource (the catalog only ever needs one value per name).
pub fn parse(xml: &str) -> Result<HashMap<String, String>, ParseError> {
    let mut out = HashMap::new();
    let bytes = xml.as_bytes();
    let len = bytes.len();
    let mut i = 0usize;

    while i < len {
        let Some(rel) = xml[i..].find('<') else {
            break;
        };
        let tag_start = i + rel;

        // Skip XML comments entirely (they may contain fake <string> tags).
        if xml[tag_start..].starts_with("<!--") {
            let Some(end) = xml[tag_start + 4..].find("-->") else {
                return Err(ParseError::new("unterminated XML comment", tag_start));
            };
            i = tag_start + 4 + end + 3;
            continue;
        }

        // Only interested in <string ...> (not <string-array>, <plurals>, ...).
        if !xml[tag_start..].starts_with("<string") {
            i = tag_start + 1;
            continue;
        }
        let after = tag_start + "<string".len();
        if after >= len
            || !(bytes[after].is_ascii_whitespace() || bytes[after] == b'/' || bytes[after] == b'>')
        {
            i = tag_start + 1;
            continue;
        }

        // Find the end of the opening tag, respecting quoted attribute values.
        let mut j = after;
        let mut quote: Option<u8> = None;
        while j < len {
            let b = bytes[j];
            match quote {
                Some(q) => {
                    if b == q {
                        quote = None;
                    }
                }
                None => {
                    if b == b'"' || b == b'\'' {
                        quote = Some(b);
                    } else if b == b'>' {
                        break;
                    }
                }
            }
            j += 1;
        }
        if j >= len {
            return Err(ParseError::new(
                "unterminated <string> opening tag",
                tag_start,
            ));
        }

        let open_tag = &xml[tag_start..=j]; // includes the '>'
        let self_closing = open_tag[..open_tag.len() - 1].trim_end().ends_with('/');
        let name = parse_name_attr(open_tag, tag_start)?;

        let (value, next) = if self_closing {
            (String::new(), j + 1)
        } else {
            let content_start = j + 1;
            let Some(close_rel) = xml[content_start..].find("</string>") else {
                return Err(ParseError::new("missing </string> closing tag", tag_start));
            };
            let content_end = content_start + close_rel;
            let value = decode(&xml[content_start..content_end]);
            let next = content_end + "</string>".len();
            (value, next)
        };

        out.insert(name, value);
        i = next;
    }

    Ok(out)
}

/// Extract the `name` attribute from a `<string ...>` opening tag.
fn parse_name_attr(tag: &str, base_offset: usize) -> Result<String, ParseError> {
    let bytes = tag.as_bytes();
    let len = bytes.len();
    let mut i = "<string".len();

    while i < len {
        while i < len && bytes[i].is_ascii_whitespace() {
            i += 1;
        }
        if i >= len || bytes[i] == b'>' || bytes[i] == b'/' {
            break;
        }

        let name_start = i;
        while i < len
            && (bytes[i].is_ascii_alphanumeric()
                || bytes[i] == b'_'
                || bytes[i] == b'-'
                || bytes[i] == b':')
        {
            i += 1;
        }
        if i == name_start {
            i += 1; // defensive: always make progress
            continue;
        }
        let attr_name = &tag[name_start..i];

        while i < len && bytes[i].is_ascii_whitespace() {
            i += 1;
        }
        if i >= len || bytes[i] != b'=' {
            continue;
        }
        i += 1;
        while i < len && bytes[i].is_ascii_whitespace() {
            i += 1;
        }
        if i >= len || (bytes[i] != b'"' && bytes[i] != b'\'') {
            continue;
        }
        let quote = bytes[i];
        i += 1;
        let value_start = i;
        while i < len && bytes[i] != quote {
            i += 1;
        }
        let value = &tag[value_start..i];
        if i < len {
            i += 1;
        }

        if attr_name == "name" {
            return Ok(value.to_string());
        }
    }

    Err(ParseError::new(
        "<string> element has no name attribute",
        base_offset,
    ))
}

/// Decode Android XML text: expand character entities and backslash escapes.
pub fn decode(raw: &str) -> String {
    let mut out = String::with_capacity(raw.len());
    let mut chars = raw.chars().peekable();

    while let Some(c) = chars.next() {
        match c {
            '&' => {
                // Peek ahead for "&...;" without consuming on failure.
                let mut probe = chars.clone();
                let mut token = String::from("&");
                let mut count = 0usize;
                while let Some(&nc) = probe.peek() {
                    if nc == ';' {
                        token.push(';');
                        probe.next();
                        break;
                    }
                    if nc.is_ascii_whitespace() || nc == '&' || nc == '<' || count > 12 {
                        break;
                    }
                    token.push(nc);
                    probe.next();
                    count += 1;
                }
                if token.len() > 2 && token.ends_with(';') {
                    if let Some(decoded) = decode_entity(&token) {
                        out.push_str(&decoded);
                        chars = probe;
                        continue;
                    }
                }
                // Not a recognised entity: keep the '&' and rescan the rest.
                out.push('&');
            }
            '\\' => match chars.peek().copied() {
                Some('\'') => {
                    chars.next();
                    out.push('\'');
                }
                Some('"') => {
                    chars.next();
                    out.push('"');
                }
                Some('\\') => {
                    chars.next();
                    out.push('\\');
                }
                Some('n') => {
                    chars.next();
                    out.push('\n');
                }
                Some('t') => {
                    chars.next();
                    out.push('\t');
                }
                Some('r') => {
                    chars.next();
                    out.push('\r');
                }
                Some('@') => {
                    chars.next();
                    out.push('@');
                }
                Some('?') => {
                    chars.next();
                    out.push('?');
                }
                Some('u') => {
                    let mut probe = chars.clone();
                    probe.next(); // consume 'u'
                    let mut hex = String::new();
                    for _ in 0..4 {
                        match probe.peek() {
                            Some(&h) if h.is_ascii_hexdigit() => {
                                hex.push(h);
                                probe.next();
                            }
                            _ => break,
                        }
                    }
                    if hex.len() == 4 {
                        if let Ok(cp) = u32::from_str_radix(&hex, 16) {
                            if let Some(ch) = char::from_u32(cp) {
                                out.push(ch);
                                chars = probe;
                                continue;
                            }
                        }
                    }
                    out.push('\\');
                }
                _ => out.push('\\'),
            },
            _ => out.push(c),
        }
    }

    out
}

/// Decode a single `&...;` entity token, or `None` if it is not recognised.
fn decode_entity(token: &str) -> Option<String> {
    let body = &token[1..token.len() - 1]; // strip leading '&' and trailing ';'
    let ch = match body {
        "amp" => '&',
        "lt" => '<',
        "gt" => '>',
        "quot" => '"',
        "apos" => '\'',
        "nbsp" => '\u{00a0}',
        _ if body.starts_with("#x") || body.starts_with("#X") => {
            char::from_u32(u32::from_str_radix(&body[2..], 16).ok()?)?
        }
        _ if body.starts_with('#') => char::from_u32(body[1..].parse::<u32>().ok()?)?,
        _ => return None,
    };
    Some(ch.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    const FIXTURE: &str = r#"<?xml version="1.0" encoding="utf-8"?>
<resources>
    <string name="app_name">GameHub Lite</string>
    <string name="escaped">It\'s a \"quoted\" \\ path\nhere &amp; &lt;tag&gt; &quot;q&quot; &apos;a&apos;</string>
    <string name="empty" />
    <string name="formatted" formatted="false">Value %s with $1.00</string>
    <string name="multi">Line one
Line two</string>
    <string name="numeric">A&#66;C &#x1F600; \@\?</string>
    <!-- <string name="comment">must be ignored</string> -->
    <string-array name="arr"><item>x</item></string-array>
</resources>
"#;

    #[test]
    fn parses_inline_fixture() {
        let map = parse(FIXTURE).expect("fixture should parse");
        assert_eq!(map.len(), 6, "unexpected keys: {:?}", map.keys());
        assert_eq!(map.get("app_name").map(String::as_str), Some("GameHub Lite"));
        assert_eq!(
            map.get("escaped").map(String::as_str),
            Some("It's a \"quoted\" \\ path\nhere & <tag> \"q\" 'a'")
        );
        assert_eq!(map.get("empty").map(String::as_str), Some(""));
        assert_eq!(
            map.get("formatted").map(String::as_str),
            Some("Value %s with $1.00")
        );
        assert_eq!(map.get("multi").map(String::as_str), Some("Line one\nLine two"));
        assert_eq!(map.get("numeric").map(String::as_str), Some("ABC \u{1f600} @?"));
        assert!(!map.contains_key("comment"));
        assert!(!map.contains_key("arr"));
    }

    #[test]
    fn decode_handles_unknown_and_bare_entities() {
        assert_eq!(decode("&unknown;"), "&unknown;");
        assert_eq!(decode("AT&T and a < b"), "AT&T and a < b");
        assert_eq!(decode("&amp;apos;"), "&apos;"); // no double decoding
        assert_eq!(decode(r"\x"), r"\x"); // unknown escape preserved
    }

    #[test]
    fn missing_name_is_an_error() {
        let err = parse("<resources><string>no name</string></resources>").unwrap_err();
        assert!(err.message.contains("name"));
    }

    #[test]
    fn parses_real_reference_strings_xml() {
        const CANDIDATES: [&str; 2] = [
            "/home/Project/Zevryn/reference/gamehub-lite-v518-decompiled/gamehub-lite-v518-decompiled/resources/res/values/strings.xml",
            "/home/Project/Zevryn/app/resources/res/values/strings.xml",
        ];
        let mut xml = None;
        for path in CANDIDATES {
            if let Ok(content) = std::fs::read_to_string(path) {
                xml = Some(content);
                break;
            }
        }
        let Some(xml) = xml else {
            eprintln!("skip: reference strings.xml not present; test falls back gracefully");
            return;
        };

        let map = parse(&xml).expect("reference strings.xml must parse");
        assert!(map.len() > 3000, "only {} strings parsed", map.len());
        assert_eq!(map.get("app_name").map(String::as_str), Some("GameHub Lite"));
        assert!(map.contains_key("action_settings"));
        assert!(map
            .get("addpc_unknown_host")
            .unwrap()
            .contains("didn't make a typo"));
        assert!(map
            .get("adb_pairing_tutorial_content_miui")
            .unwrap()
            .contains("> Notifications & Control center >"));
    }
}
