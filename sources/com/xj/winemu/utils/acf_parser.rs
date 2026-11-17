//! Minimal dependency-free parser for Valve VDF / Steam `.acf` text.
//!
//! Native Steam manifests (`appmanifest_<appid>.acf`) are VDF key-value trees:
//!
//! ```text
//! "AppState"
//! {
//!     "appid"      "570"
//!     "name"       "Dota 2"
//!     "installdir" "dota 2"
//!     "UserConfig"
//!     {
//!         "Language" "english"
//!     }
//! }
//! ```
//!
//! This is a pure text parser: quoted strings (with the common `\" \\ \n \t \r`
//! escapes), braces for nested objects and bare words as a lenient fallback.
//! Comments, conditionals and expansions used by full Steam VDF files are not
//! modelled — only the three keys needed by the portable import pipeline
//! (`appid`, `name`, `installdir`).

use super::local_pc_game_manifest::LocalPcGameManifest;
use std::fmt;

/// A value in a VDF tree: either a quoted string or a nested object.
///
/// Object entries are kept as an ordered vector because VDF permits repeated
/// keys; lookups return the first match.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum VdfValue {
    Str(String),
    Object(Vec<(String, VdfValue)>),
}

impl VdfValue {
    pub fn as_str(&self) -> Option<&str> {
        match self {
            VdfValue::Str(s) => Some(s),
            VdfValue::Object(_) => None,
        }
    }

    pub fn as_object(&self) -> Option<&[(String, VdfValue)]> {
        match self {
            VdfValue::Object(entries) => Some(entries),
            VdfValue::Str(_) => None,
        }
    }
}

/// A parsed ACF document. `root` normally holds a single `"AppState"` object.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AcfDocument {
    pub root: Vec<(String, VdfValue)>,
}

/// Errors from [`AcfDocument::parse`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AcfParseError {
    /// A quoted string was not closed.
    UnterminatedString,
    /// A key was expected but the input ended or held a non-token.
    ExpectedKey,
    /// A value was expected but the input ended.
    ExpectedValue,
    /// A closing `}` was expected before the input ended.
    ExpectedBrace,
    /// An unexpected character appeared where a value was expected.
    UnexpectedToken(char),
}

impl fmt::Display for AcfParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let msg = match self {
            AcfParseError::UnterminatedString => "unterminated string",
            AcfParseError::ExpectedKey => "expected key",
            AcfParseError::ExpectedValue => "expected value",
            AcfParseError::ExpectedBrace => "expected '}'",
            AcfParseError::UnexpectedToken(c) => return write!(f, "unexpected token '{}'", c),
        };
        f.write_str(msg)
    }
}

impl std::error::Error for AcfParseError {}

impl AcfDocument {
    /// Parse VDF text into a document.
    pub fn parse(input: &str) -> Result<Self, AcfParseError> {
        let mut parser = Parser::new(input);
        parser.skip_ws();
        let root_key = parser.read_token()?;
        parser.skip_ws();
        let root_value = match parser.peek() {
            Some('{') => {
                parser.pos += 1;
                VdfValue::Object(parser.parse_object_body()?)
            }
            Some(_) => VdfValue::Str(parser.read_token()?),
            None => return Err(AcfParseError::ExpectedValue),
        };
        parser.skip_ws();
        if let Some(c) = parser.peek() {
            return Err(AcfParseError::UnexpectedToken(c));
        }
        Ok(Self { root: vec![(root_key, root_value)] })
    }

    /// Look up a node by path, e.g. `&["AppState", "appid"]`.
    pub fn get_node(&self, path: &[&str]) -> Option<&VdfValue> {
        let (last, parents) = path.split_last()?;
        let mut entries: &[(String, VdfValue)] = &self.root;
        for key in parents {
            entries = entries.iter().find(|(k, _)| k == key)?.1.as_object()?;
        }
        entries.iter().find(|(k, _)| k == last).map(|(_, v)| v)
    }

    /// Look up a string value by path.
    pub fn get(&self, path: &[&str]) -> Option<&str> {
        self.get_node(path)?.as_str()
    }

    /// `AppState/appid` (kept as text; parse to `i64` when building a
    /// [`LocalPcGameManifest`]).
    pub fn app_id(&self) -> Option<&str> {
        self.get(&["AppState", "appid"])
    }

    /// `AppState/installdir`.
    pub fn install_dir(&self) -> Option<&str> {
        self.get(&["AppState", "installdir"])
    }

    /// `AppState/name`.
    pub fn name(&self) -> Option<&str> {
        self.get(&["AppState", "name"])
    }

    /// Project the three ACF fields onto a [`LocalPcGameManifest`].
    ///
    /// Every other GameHub-specific field is left at its empty/default value,
    /// matching `LocalPcGameManifest::empty()`. Returns `None` when `name` or
    /// `installdir` is absent; an unparsable `appid` becomes `0`.
    pub fn to_manifest(&self) -> Option<LocalPcGameManifest> {
        let mut manifest = LocalPcGameManifest::empty();
        manifest.name = self.name()?.to_owned();
        manifest.install_dir = self.install_dir()?.to_owned();
        manifest.app_id = self.app_id().and_then(|s| s.parse::<i64>().ok()).unwrap_or(0);
        Some(manifest)
    }
}

/// Small hand-written tokenizer/parser over `char`s.
struct Parser<'a> {
    input: &'a str,
    pos: usize,
}

impl<'a> Parser<'a> {
    fn new(input: &'a str) -> Self {
        Self { input, pos: 0 }
    }

    fn peek(&self) -> Option<char> {
        self.input[self.pos..].chars().next()
    }

    fn bump(&mut self) -> Option<char> {
        let c = self.peek()?;
        self.pos += c.len_utf8();
        Some(c)
    }

    fn skip_ws(&mut self) {
        while let Some(c) = self.peek() {
            if c.is_whitespace() {
                self.pos += c.len_utf8();
            } else {
                break;
            }
        }
    }

    fn read_quoted(&mut self) -> Result<String, AcfParseError> {
        if self.bump() != Some('"') {
            return Err(AcfParseError::ExpectedValue);
        }
        let mut out = String::new();
        loop {
            match self.bump() {
                None => return Err(AcfParseError::UnterminatedString),
                Some('"') => return Ok(out),
                Some('\\') => match self.bump() {
                    Some('n') => out.push('\n'),
                    Some('t') => out.push('\t'),
                    Some('r') => out.push('\r'),
                    Some('"') => out.push('"'),
                    Some('\\') => out.push('\\'),
                    Some(other) => {
                        out.push('\\');
                        out.push(other);
                    }
                    None => return Err(AcfParseError::UnterminatedString),
                },
                Some(c) => out.push(c),
            }
        }
    }

    fn read_token(&mut self) -> Result<String, AcfParseError> {
        self.skip_ws();
        match self.peek() {
            Some('"') => self.read_quoted(),
            Some(_) => {
                let mut out = String::new();
                while let Some(c) = self.peek() {
                    if c.is_whitespace() || matches!(c, '{' | '}' | '"') {
                        break;
                    }
                    out.push(c);
                    self.pos += c.len_utf8();
                }
                if out.is_empty() {
                    Err(AcfParseError::ExpectedKey)
                } else {
                    Ok(out)
                }
            }
            None => Err(AcfParseError::ExpectedKey),
        }
    }

    fn parse_object_body(&mut self) -> Result<Vec<(String, VdfValue)>, AcfParseError> {
        let mut entries = Vec::new();
        loop {
            self.skip_ws();
            match self.peek() {
                None => return Err(AcfParseError::ExpectedBrace),
                Some('}') => {
                    self.pos += 1;
                    return Ok(entries);
                }
                Some(_) => {}
            }
            let key = self.read_token()?;
            self.skip_ws();
            let value = match self.peek() {
                Some('{') => {
                    self.pos += 1;
                    VdfValue::Object(self.parse_object_body()?)
                }
                Some(_) => VdfValue::Str(self.read_token()?),
                None => return Err(AcfParseError::ExpectedValue),
            };
            entries.push((key, value));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const FIXTURE: &str = r#"
"AppState"
{
	"appid"		"570"
	"Universe"		"1"
	"name"		"Dota 2"
	"installdir"		"dota 2"
	"StateFlags"		"4"
	"UserConfig"
	{
		"Language"		"english"
	}
}
"#;

    #[test]
    fn parses_appid_name_and_installdir() {
        let doc = AcfDocument::parse(FIXTURE).unwrap();
        assert_eq!(doc.get(&["AppState", "appid"]), Some("570"));
        assert_eq!(doc.app_id(), Some("570"));
        assert_eq!(doc.name(), Some("Dota 2"));
        assert_eq!(doc.install_dir(), Some("dota 2"));
        assert_eq!(doc.get(&["AppState", "UserConfig", "Language"]), Some("english"));
        assert_eq!(doc.get(&["AppState", "missing"]), None);
        assert_eq!(doc.get(&["Missing", "appid"]), None);
    }

    #[test]
    fn nested_object_is_visible_as_a_node() {
        let doc = AcfDocument::parse(FIXTURE).unwrap();
        let node = doc.get_node(&["AppState", "UserConfig"]).unwrap();
        assert_eq!(node.as_object().unwrap().len(), 1);
        assert_eq!(node.as_str(), None);
    }

    #[test]
    fn projects_onto_local_manifest() {
        let manifest = AcfDocument::parse(FIXTURE).unwrap().to_manifest().unwrap();
        assert_eq!(manifest.get_app_id(), 570);
        assert_eq!(manifest.get_name(), "Dota 2");
        assert_eq!(manifest.get_install_dir(), "dota 2");
        assert_eq!(manifest.get_manifest_version(), "");
        assert_eq!(manifest.get_executable(), "");
    }

    #[test]
    fn handles_escapes_and_bare_words() {
        let doc = AcfDocument::parse(r#""AppState" { "name" "A \"quoted\" game" "path" C:\games\a\\b }"#).unwrap();
        assert_eq!(doc.name(), Some("A \"quoted\" game"));
        assert_eq!(doc.get(&["AppState", "path"]), Some(r"C:\games\a\\b"));
    }

    #[test]
    fn reports_malformed_input() {
        assert_eq!(AcfDocument::parse(r#""AppState" { "appid" "1""#), Err(AcfParseError::ExpectedBrace));
        assert_eq!(AcfDocument::parse(r#""AppState" { "appid" "unterminated }"#), Err(AcfParseError::UnterminatedString));
        assert_eq!(AcfDocument::parse(""), Err(AcfParseError::ExpectedKey));
    }
}
