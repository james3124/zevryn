//! Minimal `.reg` text parser/serializer for the Wine registry format
//! (port of `com.winemu.core.regedit.RegistryParser` and the text half of
//! `RegistryWriter`, GameHub Lite v5.1.8).
//!
//! The reference parser is a hand-written lexer for `WINE REGISTRY Version 2`
//! files. Its portable, observable behavior is:
//!
//! * the first line is a version header (`RegistryParser.k`);
//! * `;; All keys relative to X` records the relative base key;
//! * `#arch=win32|win64` records the target architecture (`h`);
//! * `[HKEY_CURRENT_USER\...]` selects/creates a key (`i`);
//! * `"Name"="value"` / `@="value"` add a value to the current key (`v`, `l`);
//! * data types are `"..."` (REG_SZ), `dword:XXXXXXXX` (type 4),
//!   `hex:XX,XX` (type 3), `hex(7):...` (REG_MULTI_SZ, type 7) and
//!   `hex(2):...` (REG_EXPAND_SZ, folded into [`RegValue::String`]);
//! * `\` at end of a line continues a `hex:` value onto the next line.
//!
//! Only dependency-free text handling is kept. File I/O, `BufferedReader`,
//! UTF-16 byte-array objects and Android resources stay out of scope; the
//! parser works on a `&str` and the writer returns a `String`.

use std::fmt;

use super::architecture::Architecture;
use super::key::{split_reg_path, RegHive, RegKey};
use super::value::RegValue;

/// Parse error carrying the 1-based physical line number, mirroring
/// `RegistryParser.ParseException` / `TextDiffParser.ParseException`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RegParseError {
    pub message: String,
    pub line: usize,
}

impl RegParseError {
    fn new(line: usize, message: impl Into<String>) -> Self {
        RegParseError {
            message: message.into(),
            line,
        }
    }
}

impl fmt::Display for RegParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} at line {}", self.message, self.line)
    }
}

impl std::error::Error for RegParseError {}

/// Result of parsing a `.reg` file, mirroring `RegistryParser.LoadResult`:
/// the populated hive, the `;; All keys relative to` text and the architecture
/// from `#arch=`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RegLoadResult {
    pub hive: RegHive,
    pub relative_base_key: String,
    pub architecture: Architecture,
}

/// Parse `.reg` text into an in-memory hive.
pub fn parse_reg(text: &str) -> Result<RegLoadResult, RegParseError> {
    let lines = logical_lines(text);
    let mut hive = RegHive::new();
    let mut relative_base = String::new();
    let mut architecture = Architecture::Unknown;
    let mut current: Option<String> = None;

    // First non-empty line must be a version header (RegistryParser.k).
    let mut idx = 0;
    while idx < lines.len() && lines[idx].1.is_empty() {
        idx += 1;
    }
    if idx >= lines.len() {
        return Err(RegParseError::new(1, "Empty registry file"));
    }
    let header_line = lines[idx].0;
    let header = lines[idx].1.to_ascii_lowercase();
    if !header.contains("wine registry") && !header.contains("windows registry editor") {
        return Err(RegParseError::new(
            header_line,
            format!("Invalid registry file format: '{}'", lines[idx].1),
        ));
    }
    idx += 1;

    while idx < lines.len() {
        let (line_no, line) = &lines[idx];
        let line_no = *line_no;
        let trimmed = line.trim();
        if trimmed.is_empty() {
            idx += 1;
            continue;
        }

        if let Some(rest) = trimmed.strip_prefix(";; All keys relative to") {
            relative_base = rest.trim().to_owned();
            idx += 1;
            continue;
        }

        if trimmed.starts_with(';') {
            idx += 1;
            continue;
        }

        if let Some(rest) = trimmed.strip_prefix('#') {
            if let Some(token) = rest.strip_prefix("arch=") {
                architecture = Architecture::parse(token).ok_or_else(|| {
                    RegParseError::new(line_no, format!("Unknown architecture: {token}"))
                })?;
            }
            // #time=, #class= and #link are metadata not modeled here.
            idx += 1;
            continue;
        }

        if trimmed.starts_with('[') {
            let path = parse_key_header(trimmed, line_no)?;
            if path.is_empty() {
                current = None;
            } else {
                hive.ensure_key(&path);
                current = Some(split_reg_path(&path).join("\\"));
            }
            idx += 1;
            continue;
        }

        let (name, value) = parse_value_line(trimmed, line_no)?;
        let key_path = current.as_deref().ok_or_else(|| {
            RegParseError::new(line_no, "Value without key")
        })?;
        hive.ensure_key(key_path).insert_value(name, value);
        idx += 1;
    }

    Ok(RegLoadResult {
        hive,
        relative_base_key: relative_base,
        architecture,
    })
}

/// Parse just the tree, discarding the header metadata.
pub fn parse_hive(text: &str) -> Result<RegHive, RegParseError> {
    parse_reg(text).map(|r| r.hive)
}

/// Join physical lines that end with `\` into logical lines, returning
/// `(first_physical_line_number, joined_text)` pairs. Mirrors the continuation
/// handling in `RegistryParser.l`.
fn logical_lines(text: &str) -> Vec<(usize, String)> {
    let mut out = Vec::new();
    let mut buf = String::new();
    let mut start = 0usize;

    for (i, raw) in text.lines().enumerate() {
        let line_no = i + 1;
        let trimmed = raw.trim();
        if buf.is_empty() {
            start = line_no;
        }
        if let Some(head) = trimmed.strip_suffix('\\') {
            buf.push_str(head);
        } else if buf.is_empty() {
            out.push((line_no, trimmed.to_owned()));
        } else {
            buf.push_str(trimmed);
            out.push((start, std::mem::take(&mut buf)));
        }
    }
    if !buf.is_empty() {
        out.push((start, buf));
    }
    out
}

/// Parse `[path]` (anything after the first `]` is ignored, as in the
/// reference, which appends a timestamp). Escape sequences in the path are
/// decoded the same way as quoted strings, except unknown escapes keep their
/// backslash.
fn parse_key_header(line: &str, line_no: usize) -> Result<String, RegParseError> {
    let end = line.find(']').ok_or_else(|| {
        RegParseError::new(line_no, format!("Malformed key: {line}"))
    })?;
    let inner = &line[1..end];
    if inner.is_empty() {
        return Ok(String::new());
    }
    let chars: Vec<char> = inner.chars().collect();
    decode_content(&chars, line_no)
}

/// Parse a value line: a quoted name (or `@` for the default value), `=`, and
/// the data payload. Returns `(name, value)`.
fn parse_value_line(line: &str, line_no: usize) -> Result<(String, RegValue), RegParseError> {
    let chars: Vec<char> = line.chars().collect();
    let (name, data_start) = if chars.first() == Some(&'@') {
        if chars.get(1) != Some(&'=') {
            return Err(RegParseError::new(line_no, "Malformed default value"));
        }
        (String::new(), 2)
    } else if chars.first() == Some(&'"') {
        let (name, after_quote) = read_quoted(&chars, 1, line_no)?;
        let mut i = after_quote;
        while i < chars.len() && chars[i].is_whitespace() {
            i += 1;
        }
        if chars.get(i) != Some(&'=') {
            return Err(RegParseError::new(line_no, "Missing '=' after value name"));
        }
        (name, i + 1)
    } else {
        return Err(RegParseError::new(
            line_no,
            format!("Malformed value name: expected quoted name, got '{line}'"),
        ));
    };

    let data: String = chars[data_start..].iter().collect();
    let value = parse_data(data.trim(), line_no)?;
    Ok((name, value))
}

/// Read a `"`-delimited string starting at `start`, returning its decoded
/// content and the index just after the closing quote.
fn read_quoted(chars: &[char], start: usize, line_no: usize) -> Result<(String, usize), RegParseError> {
    let mut out = String::new();
    let mut i = start;
    while i < chars.len() {
        match chars[i] {
            '"' => return Ok((out, i + 1)),
            '\\' if i + 1 < chars.len() => {
                let (c, used) = decode_escape(&chars[i + 1..], line_no)?;
                out.push(c);
                i += 1 + used;
            }
            c => {
                out.push(c);
                i += 1;
            }
        }
    }
    Err(RegParseError::new(line_no, "Missing closing quote"))
}

/// Decode all characters in `chars` as string content (used for key paths).
fn decode_content(chars: &[char], line_no: usize) -> Result<String, RegParseError> {
    let mut out = String::new();
    let mut i = 0;
    while i < chars.len() {
        if chars[i] == '\\'
            && i + 1 < chars.len()
            && is_known_escape(chars[i + 1])
        {
            let (c, used) = decode_escape(&chars[i + 1..], line_no)?;
            out.push(c);
            i += 1 + used;
        } else {
            // Unknown escape or path separator: keep the backslash literally.
            out.push(chars[i]);
            i += 1;
        }
    }
    Ok(out)
}

/// Characters that introduce a real escape sequence. Everything else after
/// `\` is a literal path separator (e.g. `\Software`).
fn is_known_escape(c: char) -> bool {
    matches!(c, 'a' | 'b' | 'e' | 'f' | 'n' | 'r' | 't' | 'v' | '\\' | '"' | '\'' | 'x')
        || ('0'..='7').contains(&c)
}

/// Decode one escape sequence from `rest` (the characters after `\`).
/// Returns the produced char and how many characters were consumed.
fn decode_escape(rest: &[char], line_no: usize) -> Result<(char, usize), RegParseError> {
    let Some(&c) = rest.first() else {
        return Err(RegParseError::new(line_no, "Dangling escape"));
    };
    let simple = match c {
        'a' => Some('\u{0007}'),
        'b' => Some('\u{0008}'),
        'e' => Some('\u{001b}'),
        'f' => Some('\u{000c}'),
        'n' => Some('\n'),
        'r' => Some('\r'),
        't' => Some('\t'),
        'v' => Some('\u{000b}'),
        '\\' => Some('\\'),
        '"' => Some('"'),
        '\'' => Some('\''),
        _ => None,
    };
    if let Some(ch) = simple {
        return Ok((ch, 1));
    }
    if c == 'x' {
        let mut value = 0u32;
        let mut used = 1usize;
        while used < rest.len() && used < 5 && rest[used].is_ascii_hexdigit() {
            value = value * 16 + rest[used].to_digit(16).unwrap();
            used += 1;
        }
        if used == 1 {
            return Ok(('x', 1));
        }
        return Ok((char::from_u32(value).unwrap_or('\u{fffd}'), used));
    }
    if ('0'..='7').contains(&c) {
        let mut value = c.to_digit(8).unwrap();
        let mut used = 1usize;
        while used < rest.len() && used < 4 && ('0'..='7').contains(&rest[used]) {
            value = value * 8 + rest[used].to_digit(8).unwrap();
            used += 1;
        }
        return Ok((char::from_u32(value).unwrap_or('\u{fffd}'), used));
    }
    // Unknown escape: emit the character itself (RegistryParser.m behaves
    // similarly for unrecognized sequences).
    Ok((c, 1))
}

/// Parse the data payload after `=`, mirroring `RegistryParser.c` / `l`.
fn parse_data(data: &str, line_no: usize) -> Result<RegValue, RegParseError> {
    let lower = data.to_ascii_lowercase();
    if let Some(hex) = lower.strip_prefix("dword:") {
        let raw = u64::from_str_radix(hex.trim(), 16)
            .map_err(|_| RegParseError::new(line_no, format!("Invalid DWORD: {hex}")))?;
        return Ok(RegValue::Dword(raw as u32));
    }
    if let Some(hex) = lower.strip_prefix("qword:") {
        let raw = u64::from_str_radix(hex.trim(), 16)
            .map_err(|_| RegParseError::new(line_no, format!("Invalid QWORD: {hex}")))?;
        return Ok(RegValue::Qword(raw));
    }
    if let Some(hex) = lower.strip_prefix("hex(7):") {
        return Ok(RegValue::MultiString(parse_multi_string(&parse_hex_bytes(hex))));
    }
    if let Some(hex) = lower.strip_prefix("hex(2):") {
        return Ok(RegValue::String(utf16le_to_string(&parse_hex_bytes(hex))));
    }
    if let Some(hex) = lower.strip_prefix("hex:") {
        return Ok(RegValue::Binary(parse_hex_bytes(hex)));
    }
    if let Some(rest) = lower.strip_prefix("str:") {
        return parse_quoted_data(rest, data, line_no);
    }
    if data.starts_with('"') {
        return parse_quoted_data(data, data, line_no);
    }
    Err(RegParseError::new(
        line_no,
        format!("Unknown data type: {data}"),
    ))
}

/// Helper for `str:"..."` and raw `"..."` data. `rest` is the lowercased
/// prefix-stripped form used only for detection; `data` is the original text.
fn parse_quoted_data(rest: &str, data: &str, line_no: usize) -> Result<RegValue, RegParseError> {
    // Locate the opening quote in the original text (after `str:` if present).
    let offset = if data.to_ascii_lowercase().starts_with("str:") {
        4
    } else {
        0
    };
    let _ = rest;
    let chars: Vec<char> = data.chars().collect();
    if chars.get(offset) != Some(&'"') {
        return Err(RegParseError::new(line_no, "Malformed string value"));
    }
    let (value, _) = read_quoted(&chars, offset + 1, line_no)?;
    Ok(RegValue::String(value))
}

/// Collect hexadecimal digits (ignoring separators) into bytes. An odd number
/// of nibbles is padded with a leading zero, matching `RegistryParser.q`.
fn parse_hex_bytes(data: &str) -> Vec<u8> {
    let nibbles: Vec<u8> = data
        .chars()
        .filter_map(|c| c.to_digit(16).map(|d| d as u8))
        .collect();
    let odd = nibbles.len() % 2 == 1;
    let mut out = Vec::with_capacity((nibbles.len() + 1) / 2);
    let mut high = 0u8;
    let mut z3 = odd;
    for n in nibbles {
        if z3 {
            out.push((high << 4) | n);
            z3 = false;
        } else {
            high = n;
            z3 = true;
        }
    }
    out
}

/// Decode UTF-16LE bytes (the format of REG_SZ / REG_MULTI_SZ payloads).
fn utf16le_to_string(bytes: &[u8]) -> String {
    let units: Vec<u16> = bytes
        .chunks_exact(2)
        .map(|c| u16::from_le_bytes([c[0], c[1]]))
        .collect();
    String::from_utf16_lossy(&units)
}

/// Split a REG_MULTI_SZ payload on NULs, dropping the trailing empty pieces.
fn parse_multi_string(bytes: &[u8]) -> Vec<String> {
    let text = utf16le_to_string(bytes);
    let mut parts: Vec<String> = text.split('\0').map(str::to_owned).collect();
    while parts.last().map_or(false, |s| s.is_empty()) {
        parts.pop();
    }
    parts
}

/// Serialize a hive to `WINE REGISTRY Version 2` text. This is the inverse of
/// [`parse_reg`] for the value types modeled here and is used by round-trip
/// tests; the reference writer additionally handles file I/O and timestamps.
pub fn write_reg(hive: &RegHive, relative_base: &str, architecture: Architecture) -> String {
    let mut out = String::new();
    out.push_str("WINE REGISTRY Version 2\n");
    out.push_str(&format!(";; All keys relative to {relative_base}\n\n"));
    if architecture != Architecture::Unknown {
        out.push_str(&format!("#arch={}\n\n", architecture.as_str()));
    }
    for key in hive.keys() {
        if is_root(key) {
            continue;
        }
        out.push_str(&format!("[{}]\n", export_path(&key.path)));
        for (name, value) in &key.values {
            out.push_str(&format!("{}={}\n", export_name(name), export_data(value)));
        }
        out.push('\n');
    }
    out
}

fn is_root(key: &RegKey) -> bool {
    split_reg_path(&key.path).is_empty() || key.path.eq_ignore_ascii_case("\\REGISTRY")
}

fn export_path(path: &str) -> String {
    let p = path.trim_start_matches('\\');
    if p.eq_ignore_ascii_case("REGISTRY") {
        return String::new();
    }
    // The hive root is named `\REGISTRY` (RegistryKey.Companion.a()); its
    // children's paths carry that prefix but .reg files write paths relative
    // to the root, so strip it before emitting `[...]` headers.
    if p.len() > 8 && p[..8].eq_ignore_ascii_case("REGISTRY") && p.as_bytes()[8] == b'\\' {
        return p[9..].to_owned();
    }
    p.to_owned()
}

fn export_name(name: &str) -> String {
    if name.is_empty() {
        "@".to_owned()
    } else {
        format!("\"{}\"", escape_string(name))
    }
}

fn export_data(value: &RegValue) -> String {
    match value {
        RegValue::String(s) => format!("\"{}\"", escape_string(s)),
        RegValue::Dword(v) => format!("dword:{v:08x}"),
        RegValue::Qword(v) => format!("qword:{v:016x}"),
        RegValue::Binary(b) => format!("hex:{}", hex_bytes(b)),
        RegValue::None => "hex:".to_owned(),
        RegValue::MultiString(items) => {
            let mut bytes = Vec::new();
            for s in items {
                bytes.extend(s.encode_utf16().flat_map(u16::to_le_bytes));
                bytes.extend_from_slice(&[0, 0]);
            }
            bytes.extend_from_slice(&[0, 0]);
            format!("hex(7):{}", hex_bytes(&bytes))
        }
    }
}

fn hex_bytes(bytes: &[u8]) -> String {
    bytes
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect::<Vec<_>>()
        .join(",")
}

/// Escape a string for a `.reg` file, mirroring the writer's escape table.
fn escape_string(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for c in s.chars() {
        match c {
            '\\' => out.push_str("\\\\"),
            '"' => out.push_str("\\\""),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            '\u{0007}' => out.push_str("\\a"),
            '\u{0008}' => out.push_str("\\b"),
            '\u{000b}' => out.push_str("\\v"),
            '\u{000c}' => out.push_str("\\f"),
            '\u{001b}' => out.push_str("\\e"),
            c if (c as u32) < 0x20 || c == '\u{007f}' => {
                out.push_str(&format!("\\x{:02x}", c as u32));
            }
            c => out.push(c),
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::{parse_hive, parse_reg, write_reg, RegParseError};
    use super::super::architecture::Architecture;
    use super::super::value::RegValue;

    #[test]
    fn parse_key_and_values() {
        let text = "WINE REGISTRY Version 2\n\
                    ;; All keys relative to \\\\Machine\\Software\\Wine\n\
                    #arch=win64\n\
                    \n\
                    [HKEY_CURRENT_USER\\Software\\Foo]\n\
                    \"Name\"=\"hello world\"\n\
                    @=\"default\"\n\
                    \"Flag\"=dword:00000001\n\
                    \"Big\"=dword:ffffffff\n\
                    \"Data\"=hex:01,02,ff\n";
        let result = parse_reg(text).unwrap();
        assert_eq!(result.architecture, Architecture::Win64);
        assert!(result.relative_base_key.contains("Wine"));
        let hive = result.hive;
        let key = hive.lookup("HKEY_CURRENT_USER\\Software\\Foo").unwrap();
        assert_eq!(key.get_value("Name"), Some(&RegValue::String("hello world".into())));
        assert_eq!(key.get_value(""), Some(&RegValue::String("default".into())));
        assert_eq!(key.get_value("Flag"), Some(&RegValue::Dword(1)));
        assert_eq!(key.get_value("Big"), Some(&RegValue::Dword(u32::MAX)));
        assert_eq!(key.get_value("Data"), Some(&RegValue::Binary(vec![1, 2, 0xff])));
    }

    #[test]
    fn parse_multistring_and_continuation() {
        let text = "WINE REGISTRY Version 2\n\
                    [Software\\Multi]\n\
                    \"Items\"=hex(7):41,00,00,00,\\\n\
                    42,00,00,00,00,00\n";
        let hive = parse_hive(text).unwrap();
        assert_eq!(
            hive.get_value("Software\\Multi", "Items"),
            Some(&RegValue::MultiString(vec!["A".into(), "B".into()]))
        );
    }

    #[test]
    fn parse_escapes() {
        let text = "WINE REGISTRY Version 2\n\
                    [Software\\Esc]\n\
                    \"A\"=\"line\\nquote\\\"slash\\\\\"\n";
        let hive = parse_hive(text).unwrap();
        assert_eq!(
            hive.get_value("Software\\Esc", "A"),
            Some(&RegValue::String("line\nquote\"slash\\".into()))
        );
    }

    #[test]
    fn parse_errors_carry_line() {
        let err = parse_reg("WINE REGISTRY Version 2\n[Software]\nBad line\n").unwrap_err();
        assert_eq!(err.line, 3);
        let err = parse_reg("garbage\n").unwrap_err();
        assert_eq!(err.line, 1);
        let _: &dyn std::error::Error = &err;
        assert!(format!("{err}").contains("line 1"));
    }

    #[test]
    fn round_trip_all_value_types() {
        let text = "WINE REGISTRY Version 2\n\
                    ;; All keys relative to HKEY_CURRENT_USER\n\
                    #arch=win32\n\
                    [Software\\RoundTrip]\n\
                    \"S\"=\"a\\tb\"\n\
                    @=\"default\"\n\
                    \"D\"=dword:0000002a\n\
                    \"Q\"=qword:0000000000000123\n\
                    \"Bin\"=hex:00,10,ff\n\
                    \"Multi\"=hex(7):41,00,00,00,42,00,00,00,00,00\n";
        let first = parse_reg(text).unwrap();
        let serialized = write_reg(&first.hive, &first.relative_base_key, first.architecture);
        let second = parse_reg(&serialized).unwrap();

        assert_eq!(first.hive, second.hive);
        assert_eq!(first.architecture, second.architecture);
        assert_eq!(first.relative_base_key, second.relative_base_key);
        // Serializing again yields the exact same text (stable round-trip).
        let serialized2 = write_reg(&second.hive, &second.relative_base_key, second.architecture);
        assert_eq!(serialized, serialized2);
        assert!(serialized.contains("#arch=win32"));
    }

    #[test]
    fn unused_error_constructor_is_available() {
        let e = RegParseError::new(7, "custom");
        assert_eq!(e.line, 7);
    }
}
