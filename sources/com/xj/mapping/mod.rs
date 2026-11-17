//! Portable logic for `com.xj.mapping` — the `.icp` input-control profile format.
//!
//! This module mirrors the dependency-free model in
//! `/home/Project/Zevryn/src/lib.rs`: the same type names, the same JSON
//! reader approach and the same validation rules. It is intentionally
//! self-contained (only `std`) so it can be wired into the `app` crate with
//! a single declaration, e.g.
//!
//! ```ignore
//! #[path = "../sources/com/xj/mapping/mod.rs"]
//! pub mod mapping;
//! ```
//!
//! Android/JNI specifics (injection, sockets, touch dispatch, UI) are out of
//! scope; only the portable data model and its binding helpers live here.

pub mod keymap;

use json::Value;

/// Minimal dependency-free JSON reader for the subset used by `.icp` profiles.
/// Copied from `src/json.rs` so this area stays self-contained.
mod json {
    //! Minimal dependency-free JSON reader for the subset used by GameHub `.icp` profiles.
    //! Supports objects, arrays, strings (incl. \u escapes), f64 numbers, bools and null.

    #[derive(Debug, Clone, PartialEq)]
    pub enum Value {
        Null,
        Bool(bool),
        Number(f64),
        String(String),
        Array(Vec<Value>),
        Object(Vec<(String, Value)>),
    }

    impl Value {
        pub fn get(&self, key: &str) -> Option<&Value> {
            match self {
                Value::Object(fields) => fields.iter().find(|(k, _)| k == key).map(|(_, v)| v),
                _ => None,
            }
        }
        pub fn as_str(&self) -> Option<&str> {
            if let Value::String(s) = self { Some(s) } else { None }
        }
        pub fn as_f64(&self) -> Option<f64> {
            if let Value::Number(n) = self { Some(*n) } else { None }
        }
        pub fn as_bool(&self) -> Option<bool> {
            if let Value::Bool(b) = self { Some(*b) } else { None }
        }
        pub fn as_array(&self) -> Option<&[Value]> {
            if let Value::Array(a) = self { Some(a) } else { None }
        }
    }

    #[derive(Debug, Clone, PartialEq, Eq)]
    pub struct ParseError {
        pub message: String,
        pub offset: usize,
    }

    pub struct Parser<'a> {
        src: &'a [u8],
        pos: usize,
    }

    impl<'a> Parser<'a> {
        pub fn new(src: &'a str) -> Self {
            Self { src: src.as_bytes(), pos: 0 }
        }

        pub fn parse(mut self) -> Result<Value, ParseError> {
            self.ws();
            let v = self.value()?;
            self.ws();
            if self.pos != self.src.len() {
                return Err(self.err("trailing data"));
            }
            Ok(v)
        }

        fn err(&self, msg: &str) -> ParseError {
            ParseError { message: msg.into(), offset: self.pos }
        }

        fn ws(&mut self) {
            while self.pos < self.src.len()
                && matches!(self.src[self.pos], b' ' | b'\n' | b'\r' | b'\t')
            {
                self.pos += 1;
            }
        }

        fn peek(&self) -> Option<u8> {
            self.src.get(self.pos).copied()
        }

        fn value(&mut self) -> Result<Value, ParseError> {
            self.ws();
            match self.peek() {
                Some(b'{') => self.object(),
                Some(b'[') => self.array(),
                Some(b'"') => Ok(Value::String(self.string()?)),
                Some(b't') => { self.literal(b"true")?; Ok(Value::Bool(true)) }
                Some(b'f') => { self.literal(b"false")?; Ok(Value::Bool(false)) }
                Some(b'n') => { self.literal(b"null")?; Ok(Value::Null) }
                Some(b'-' | b'0'..=b'9') => self.number(),
                _ => Err(self.err("expected value")),
            }
        }

        fn literal(&mut self, lit: &[u8]) -> Result<(), ParseError> {
            if self.src.get(self.pos..self.pos + lit.len()) == Some(lit) {
                self.pos += lit.len();
                Ok(())
            } else {
                Err(self.err("invalid literal"))
            }
        }

        fn object(&mut self) -> Result<Value, ParseError> {
            self.pos += 1; // {
            let mut fields = Vec::new();
            self.ws();
            if self.peek() == Some(b'}') {
                self.pos += 1;
                return Ok(Value::Object(fields));
            }
            loop {
                self.ws();
                if self.peek() != Some(b'"') {
                    return Err(self.err("expected object key"));
                }
                let key = self.string()?;
                self.ws();
                if self.peek() != Some(b':') {
                    return Err(self.err("expected ':'"));
                }
                self.pos += 1;
                let val = self.value()?;
                fields.push((key, val));
                self.ws();
                match self.peek() {
                    Some(b',') => self.pos += 1,
                    Some(b'}') => { self.pos += 1; break; }
                    _ => return Err(self.err("expected ',' or '}'")),
                }
            }
            Ok(Value::Object(fields))
        }

        fn array(&mut self) -> Result<Value, ParseError> {
            self.pos += 1; // [
            let mut items = Vec::new();
            self.ws();
            if self.peek() == Some(b']') {
                self.pos += 1;
                return Ok(Value::Array(items));
            }
            loop {
                items.push(self.value()?);
                self.ws();
                match self.peek() {
                    Some(b',') => self.pos += 1,
                    Some(b']') => { self.pos += 1; break; }
                    _ => return Err(self.err("expected ',' or ']'")),
                }
            }
            Ok(Value::Array(items))
        }

        fn string(&mut self) -> Result<String, ParseError> {
            self.pos += 1; // opening quote
            let mut out = String::new();
            while self.pos < self.src.len() {
                let b = self.src[self.pos];
                self.pos += 1;
                match b {
                    b'"' => return Ok(out),
                    b'\\' => {
                        let esc = *self.src.get(self.pos).ok_or_else(|| self.err("bad escape"))?;
                        self.pos += 1;
                        match esc {
                            b'"' => out.push('"'),
                            b'\\' => out.push('\\'),
                            b'/' => out.push('/'),
                            b'b' => out.push('\u{0008}'),
                            b'f' => out.push('\u{000C}'),
                            b'n' => out.push('\n'),
                            b'r' => out.push('\r'),
                            b't' => out.push('\t'),
                            b'u' => {
                                let hi = self.hex4()?;
                                let cp = if (0xD800..=0xDBFF).contains(&hi) {
                                    if self.src.get(self.pos..self.pos + 2) != Some(&b"\\u"[..]) {
                                        return Err(self.err("missing low surrogate"));
                                    }
                                    self.pos += 2;
                                    let lo = self.hex4()?;
                                    0x10000 + ((hi - 0xD800) << 10) + (lo - 0xDC00)
                                } else {
                                    hi
                                };
                                out.push(char::from_u32(cp).ok_or_else(|| self.err("bad codepoint"))?);
                            }
                            _ => return Err(self.err("unknown escape")),
                        }
                    }
                    0x00..=0x1F => return Err(self.err("control character in string")),
                    _ => {
                        let start = self.pos - 1;
                        let len = utf8_len(b);
                        if self.pos - 1 + len > self.src.len() {
                            return Err(self.err("truncated utf-8"));
                        }
                        self.pos = start + len;
                        let s = std::str::from_utf8(&self.src[start..self.pos])
                            .map_err(|_| self.err("invalid utf-8"))?;
                        out.push_str(s);
                    }
                }
            }
            Err(self.err("unterminated string"))
        }

        fn hex4(&mut self) -> Result<u32, ParseError> {
            let bytes = self.src.get(self.pos..self.pos + 4).ok_or_else(|| self.err("short unicode escape"))?;
            let s = std::str::from_utf8(bytes).map_err(|_| self.err("bad hex"))?;
            let n = u32::from_str_radix(s, 16).map_err(|_| self.err("bad hex"))?;
            self.pos += 4;
            Ok(n)
        }

        fn number(&mut self) -> Result<Value, ParseError> {
            let start = self.pos;
            while let Some(b) = self.peek() {
                if matches!(b, b'0'..=b'9' | b'-' | b'+' | b'.' | b'e' | b'E') {
                    self.pos += 1;
                } else {
                    break;
                }
            }
            let s = std::str::from_utf8(&self.src[start..self.pos]).map_err(|_| self.err("bad number"))?;
            let n = s.parse::<f64>().map_err(|_| self.err("bad number"))?;
            Ok(Value::Number(n))
        }
    }

    fn utf8_len(first: u8) -> usize {
        match first {
            0x00..=0x7F => 1,
            0xC0..=0xDF => 2,
            0xE0..=0xEF => 3,
            0xF0..=0xF7 => 4,
            _ => 1,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ElementType {
    DPad,
    Button,
    Stick,
    RangeButton,
    Other,
}

impl ElementType {
    fn from_str(s: &str) -> Self {
        match s {
            "D_PAD" => Self::DPad,
            "BUTTON" => Self::Button,
            "STICK" => Self::Stick,
            "RANGE_BUTTON" => Self::RangeButton,
            _ => Self::Other,
        }
    }

    /// Number of directional/action bindings this element type declares.
    pub fn binding_count(self) -> Option<usize> {
        match self {
            Self::DPad | Self::Button | Self::Stick => Some(4),
            Self::RangeButton => Some(6),
            Self::Other => None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Shape {
    Circle,
    Square,
    Other,
}

impl Shape {
    fn from_str(s: &str) -> Self {
        match s {
            "CIRCLE" => Self::Circle,
            "SQUARE" => Self::Square,
            _ => Self::Other,
        }
    }
}

/// One on-screen control from an `.icp` profile.
#[derive(Debug, Clone, PartialEq)]
pub struct Element {
    pub kind: ElementType,
    pub shape: Shape,
    /// Up/right/down/left or action bindings, e.g. `GAMEPAD_BUTTON_X`, `NONE`.
    pub bindings: Vec<String>,
    pub scale: f64,
    /// Normalized anchor position in [0.0, 1.0].
    pub x: f64,
    pub y: f64,
    pub toggle_switch: bool,
    pub text: String,
    pub icon_id: i64,
    /// Range-button mode, e.g. `FROM_0_TO_9`.
    pub range: Option<String>,
}

/// A localized, portable input-control profile (`assets/inputcontrols/profiles/*.icp`).
#[derive(Debug, Clone, PartialEq)]
pub struct Profile {
    pub id: i64,
    pub name: String,
    pub cn_name: String,
    pub ja_name: String,
    pub ru_name: String,
    pub description: String,
    pub cn_description: String,
    pub ja_description: String,
    pub ru_description: String,
    pub cursor_speed: f64,
    pub game_id: String,
    pub elements: Vec<Element>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProfileError {
    pub message: String,
}

impl ProfileError {
    fn new(msg: impl Into<String>) -> Self {
        Self { message: msg.into() }
    }
}

fn req_str<'a>(obj: &'a Value, key: &str) -> Result<&'a str, ProfileError> {
    obj.get(key)
        .and_then(Value::as_str)
        .ok_or_else(|| ProfileError::new(format!("missing or invalid string field '{key}'")))
}

fn req_num(obj: &Value, key: &str) -> Result<f64, ProfileError> {
    obj.get(key)
        .and_then(Value::as_f64)
        .ok_or_else(|| ProfileError::new(format!("missing or invalid numeric field '{key}'")))
}

fn opt_str<'a>(obj: &'a Value, key: &str) -> &'a str {
    obj.get(key).and_then(Value::as_str).unwrap_or("")
}

fn opt_bool(obj: &Value, key: &str) -> bool {
    obj.get(key).and_then(Value::as_bool).unwrap_or(false)
}

fn opt_num(obj: &Value, key: &str, default: f64) -> f64 {
    obj.get(key).and_then(Value::as_f64).unwrap_or(default)
}

impl Profile {
    /// Parse an `.icp` profile from JSON text.
    pub fn parse(input: &str) -> Result<Self, ProfileError> {
        let root = json::Parser::new(input)
            .parse()
            .map_err(|e| ProfileError::new(format!("JSON error at {}: {}", e.offset, e.message)))?;

        let elements = match root.get("elements").and_then(Value::as_array) {
            Some(items) => items.iter().map(parse_element).collect::<Result<Vec<_>, _>>()?,
            None => Vec::new(),
        };

        Ok(Self {
            id: req_num(&root, "id")? as i64,
            name: req_str(&root, "name")?.into(),
            cn_name: opt_str(&root, "cnName").into(),
            ja_name: opt_str(&root, "jaName").into(),
            ru_name: opt_str(&root, "ruName").into(),
            description: opt_str(&root, "description").into(),
            cn_description: opt_str(&root, "cnDescription").into(),
            ja_description: opt_str(&root, "jaDescription").into(),
            ru_description: opt_str(&root, "ruDescription").into(),
            cursor_speed: opt_num(&root, "cursorSpeed", 1.0),
            game_id: opt_str(&root, "gameId").into(),
            elements,
        })
    }

    /// Validate normalized coordinates and binding arity.
    pub fn validate(&self) -> Result<(), ProfileError> {
        for (i, e) in self.elements.iter().enumerate() {
            if !(0.0..=1.0).contains(&e.x) || !(0.0..=1.0).contains(&e.y) {
                return Err(ProfileError::new(format!("element {i}: position outside [0,1]")));
            }
            if let Some(expected) = e.kind.binding_count() {
                if e.bindings.len() != expected {
                    return Err(ProfileError::new(format!(
                        "element {i}: {:?} expects {expected} bindings, got {}",
                        e.kind,
                        e.bindings.len()
                    )));
                }
            }
            if e.scale <= 0.0 {
                return Err(ProfileError::new(format!("element {i}: non-positive scale")));
            }
        }
        Ok(())
    }

    /// Convenience: number of on-screen controls declared by this profile.
    pub fn element_count(&self) -> usize {
        self.elements.len()
    }
}

fn parse_element(v: &Value) -> Result<Element, ProfileError> {
    let bindings = match v.get("bindings").and_then(Value::as_array) {
        Some(items) => items
            .iter()
            .map(|x| x.as_str().map(str::to_owned).ok_or_else(|| ProfileError::new("non-string binding")))
            .collect::<Result<Vec<_>, _>>()?,
        None => Vec::new(),
    };
    Ok(Element {
        kind: ElementType::from_str(opt_str(v, "type")),
        shape: Shape::from_str(opt_str(v, "shape")),
        bindings,
        scale: opt_num(v, "scale", 1.0),
        x: req_num(v, "x")?,
        y: req_num(v, "y")?,
        toggle_switch: opt_bool(v, "toggleSwitch"),
        text: opt_str(v, "text").into(),
        icon_id: opt_num(v, "iconId", 0.0) as i64,
        range: v.get("range").and_then(Value::as_str).map(str::to_owned),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    const FIXTURE: &str = r#"{
      "id": 1,
      "name": "Default Controller Layout",
      "cnName": "手柄默认布局",
      "cursorSpeed": 1,
      "gameId": "",
      "elements": [
        {"type":"D_PAD","shape":"CIRCLE","bindings":["GAMEPAD_DPAD_UP","GAMEPAD_DPAD_RIGHT","GAMEPAD_DPAD_DOWN","GAMEPAD_DPAD_LEFT"],"scale":0.85,"x":0.31,"y":0.73,"toggleSwitch":false,"text":"","iconId":0},
        {"type":"BUTTON","shape":"CIRCLE","bindings":["GAMEPAD_BUTTON_X","NONE","NONE","NONE"],"scale":1,"x":0.81,"y":0.73,"toggleSwitch":false,"text":"","iconId":0}
      ]
    }"#;

    #[test]
    fn parses_profile() {
        let p = Profile::parse(FIXTURE).unwrap();
        assert_eq!(p.id, 1);
        assert_eq!(p.name, "Default Controller Layout");
        assert_eq!(p.cn_name, "手柄默认布局");
        assert_eq!(p.cursor_speed, 1.0);
        assert_eq!(p.elements.len(), 2);
        assert_eq!(p.element_count(), 2);
        assert_eq!(p.elements[0].kind, ElementType::DPad);
        assert_eq!(p.elements[0].shape, Shape::Circle);
        assert_eq!(p.elements[0].bindings[0], "GAMEPAD_DPAD_UP");
        assert_eq!(p.elements[1].kind, ElementType::Button);
        assert!(!p.elements[1].toggle_switch);
    }

    #[test]
    fn validates_profile() {
        let p = Profile::parse(FIXTURE).unwrap();
        assert!(p.validate().is_ok());
    }

    #[test]
    fn rejects_bad_coordinates() {
        let bad = FIXTURE.replace("\"x\":0.31", "\"x\":1.5");
        let p = Profile::parse(&bad).unwrap();
        assert!(p.validate().is_err());
    }

    #[test]
    fn rejects_missing_name() {
        let bad = FIXTURE.replace("\"name\": \"Default Controller Layout\",", "");
        assert!(Profile::parse(&bad).is_err());
    }

    #[test]
    fn rejects_wrong_binding_arity() {
        let bad = FIXTURE.replace(
            "\"bindings\":[\"GAMEPAD_BUTTON_X\",\"NONE\",\"NONE\",\"NONE\"]",
            "\"bindings\":[\"GAMEPAD_BUTTON_X\"]",
        );
        let p = Profile::parse(&bad).unwrap();
        assert!(p.validate().is_err());
    }

    #[test]
    fn element_type_binding_counts() {
        assert_eq!(ElementType::DPad.binding_count(), Some(4));
        assert_eq!(ElementType::Button.binding_count(), Some(4));
        assert_eq!(ElementType::Stick.binding_count(), Some(4));
        assert_eq!(ElementType::RangeButton.binding_count(), Some(6));
        assert_eq!(ElementType::Other.binding_count(), None);
    }

    #[test]
    fn parses_all_real_apk_profiles_when_present() {
        let dir = "/home/Project/Zevryn/reference/gamehub-lite-v518-decompiled/gamehub-lite-v518-decompiled/apktool/assets/inputcontrols/profiles";
        let Ok(entries) = std::fs::read_dir(dir) else { return; };
        let mut count = 0;
        for entry in entries.flatten() {
            let text = std::fs::read_to_string(entry.path()).unwrap();
            let p = Profile::parse(&text)
                .unwrap_or_else(|e| panic!("{}: {}", entry.path().display(), e.message));
            p.validate()
                .unwrap_or_else(|e| panic!("{}: {}", entry.path().display(), e.message));
            assert!(!p.elements.is_empty());
            count += 1;
        }
        assert_eq!(count, 4, "expected 4 bundled .icp profiles");
    }

    #[test]
    fn real_profiles_have_expected_kinds_and_ranges() {
        let dir = "/home/Project/Zevryn/reference/gamehub-lite-v518-decompiled/gamehub-lite-v518-decompiled/apktool/assets/inputcontrols/profiles";
        let Ok(entries) = std::fs::read_dir(dir) else { return; };
        let mut range_buttons = 0;
        let mut range_modes = 0;
        for entry in entries.flatten() {
            let text = std::fs::read_to_string(entry.path()).unwrap();
            let p = Profile::parse(&text).unwrap();
            for e in &p.elements {
                if e.kind == ElementType::RangeButton {
                    range_buttons += 1;
                    if e.range.as_deref() == Some("FROM_0_TO_9") {
                        range_modes += 1;
                    }
                }
            }
        }
        assert_eq!(range_buttons, 2, "expected 2 RANGE_BUTTON elements in the 4 profiles");
        assert_eq!(range_modes, 1, "expected 1 FROM_0_TO_9 range mode");
    }
}
