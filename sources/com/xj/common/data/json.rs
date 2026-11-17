//! Minimal dependency-free JSON reader/writer for the API DTOs.
//!
//! This mirrors the core crate's `src/json.rs` reader (same `Value` model and
//! parsing rules) so this mirrored area stays self-contained: no external
//! crates, no reliance on how the harness wires `#[path]` modules into
//! `app/src/lib.rs`. A small string-object writer is added for building signed
//! request bodies.

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
        while self.pos < self.src.len() && matches!(self.src[self.pos], b' ' | b'\n' | b'\r' | b'\t') {
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

/// Serialize a flat object of string values into JSON, with the same escaping
/// behavior the app's Gson writer uses for the signed request body.
pub fn object_to_json(pairs: &[(String, String)]) -> String {
    let mut out = String::from("{");
    for (i, (k, v)) in pairs.iter().enumerate() {
        if i != 0 {
            out.push(',');
        }
        out.push('"');
        push_escaped(&mut out, k);
        out.push_str("\":\"");
        push_escaped(&mut out, v);
        out.push('"');
    }
    out.push('}');
    out
}

fn push_escaped(out: &mut String, s: &str) {
    for c in s.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            c if (c as u32) < 0x20 => out.push_str(&format!("\\u{:04x}", c as u32)),
            c => out.push(c),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_nested_value() {
        let v = Parser::new(r#"{"a":[1,-2.5,true,null,"x\n"],"b":{"c":1e2}}"#).parse().unwrap();
        assert_eq!(v.get("a").unwrap().as_array().unwrap().len(), 5);
        assert_eq!(v.get("a").unwrap().as_array().unwrap()[1].as_f64(), Some(-2.5));
        assert_eq!(v.get("b").unwrap().get("c").unwrap().as_f64(), Some(100.0));
    }

    #[test]
    fn writes_escaped_object() {
        let pairs = vec![("k".into(), "a\"b\n".into()), ("n".into(), "1".into())];
        assert_eq!(object_to_json(&pairs), r#"{"k":"a\"b\n","n":"1"}"#);
    }
}
