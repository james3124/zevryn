//! Rust model of `com.winemu.core.regedit.RegistryValue` (GameHub Lite
//! v5.1.8).
//!
//! In the reference each registry value is a subclass of the abstract
//! `RegistryValue` (StringValue, DwordValue, QwordValue, BinaryValue,
//! MultiStringValue, ExpandStringValue). Each subclass carries a name, raw
//! bytes (`a()`), a Windows REG_* type code (`c()`) and a string name (`b()`).
//!
//! For the portable model the value **name** lives in the owning key's
//! `BTreeMap` (`RegKey::values`), so `RegValue` only needs to hold the payload.
//! `ExpandStringValue` (REG_EXPAND_SZ, type 2) is folded into [`RegValue::String`]
//! because the distinction is only meaningful to Win32 APIs; the `.reg` parser
//! decodes `hex(2):` into a plain string. `None` models REG_NONE / an empty
//! default value.

use std::fmt;

/// Payload of a registry value, keyed by name inside a [`RegKey`].
///
/// The discriminant names mirror the Java subclasses; type codes match the
/// Windows REG_* constants reported by `RegistryValue.c()`.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum RegValue {
    /// REG_SZ (type 1) — `StringValue` / `ExpandStringValue`.
    String(String),
    /// REG_DWORD (type 4) — `DwordValue`.
    Dword(u32),
    /// REG_QWORD (type 11) — `QwordValue`.
    Qword(u64),
    /// REG_BINARY (type 3) — `BinaryValue`.
    Binary(Vec<u8>),
    /// REG_MULTI_SZ (type 7) — `MultiStringValue`.
    MultiString(Vec<String>),
    /// REG_NONE (type 0) — no payload / the default value with no data.
    None,
}

impl RegValue {
    /// Windows registry type code, matching `RegistryValue.c()` in the
    /// decompiled sources (`StringValue` = 1, `BinaryValue` = 3,
    /// `DwordValue` = 4, `MultiStringValue` = 7, `QwordValue` = 11).
    pub fn type_code(&self) -> u32 {
        match self {
            RegValue::String(_) => 1,
            RegValue::Binary(_) => 3,
            RegValue::Dword(_) => 4,
            RegValue::MultiString(_) => 7,
            RegValue::Qword(_) => 11,
            RegValue::None => 0,
        }
    }

    /// `true` only for the REG_NONE / no-data variant.
    pub fn is_none(&self) -> bool {
        matches!(self, RegValue::None)
    }

    /// Little-endian bytes of the payload, mirroring `RegistryValue.a()`.
    /// Strings are encoded as UTF-16LE plus the `00 00` terminator used by
    /// `StringValue.a()`; multi-strings append one terminator per element and
    /// a final extra one (as `MultiStringValue.a()` does).
    pub fn to_le_bytes(&self) -> Vec<u8> {
        match self {
            RegValue::None => Vec::new(),
            RegValue::String(s) => {
                let mut out: Vec<u8> = s.encode_utf16().flat_map(u16::to_le_bytes).collect();
                out.extend_from_slice(&[0, 0]);
                out
            }
            RegValue::Dword(v) => v.to_le_bytes().to_vec(),
            RegValue::Qword(v) => v.to_le_bytes().to_vec(),
            RegValue::Binary(b) => b.clone(),
            RegValue::MultiString(items) => {
                let mut out = Vec::new();
                for s in items {
                    out.extend(s.encode_utf16().flat_map(u16::to_le_bytes));
                    out.extend_from_slice(&[0, 0]);
                }
                out.extend_from_slice(&[0, 0]);
                out
            }
        }
    }
}

impl fmt::Display for RegValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            RegValue::String(s) => write!(f, "String({s:?})"),
            RegValue::Dword(v) => write!(f, "Dword(0x{v:08x})"),
            RegValue::Qword(v) => write!(f, "Qword(0x{v:016x})"),
            RegValue::Binary(b) => write!(f, "Binary({} bytes)", b.len()),
            RegValue::MultiString(items) => write!(f, "MultiString({items:?})"),
            RegValue::None => f.write_str("None"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::RegValue;

    #[test]
    fn type_codes_match_windows_reg_constants() {
        assert_eq!(RegValue::String("x".into()).type_code(), 1);
        assert_eq!(RegValue::Binary(vec![1]).type_code(), 3);
        assert_eq!(RegValue::Dword(1).type_code(), 4);
        assert_eq!(RegValue::MultiString(vec![]).type_code(), 7);
        assert_eq!(RegValue::Qword(1).type_code(), 11);
        assert_eq!(RegValue::None.type_code(), 0);
        assert!(RegValue::None.is_none());
        assert!(!RegValue::Dword(0).is_none());
    }

    #[test]
    fn little_endian_bytes() {
        assert_eq!(RegValue::Dword(0x0102_0304).to_le_bytes(), vec![4, 3, 2, 1]);
        assert_eq!(
            RegValue::Qword(0x0102_0304_0506_0708).to_le_bytes(),
            vec![8, 7, 6, 5, 4, 3, 2, 1]
        );
        assert_eq!(
            RegValue::String("A".into()).to_le_bytes(),
            vec![b'A', 0, 0, 0]
        );
        assert_eq!(
            RegValue::MultiString(vec!["A".into()]).to_le_bytes(),
            vec![b'A', 0, 0, 0, 0, 0]
        );
    }
}
