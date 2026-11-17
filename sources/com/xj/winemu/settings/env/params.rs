//! Environment-variable parameter model.
//!
//! The app stores game variables as a comma-separated `KEY=VALUE` string.
//! `NewInputEnvParamsDialog.p` builds it (skip fully-blank rows, de-duplicate
//! by key keeping the first, join with `,`), and
//! `PcGameSettingDataHelper.R` parses it back into an ordered map, allowing
//! values to contain commas by concatenating later comma-separated segments
//! until the next `=`. Both behaviours are reproduced here without I/O.
//!
//! `EditType` (from `EditType.java`) records which dialog mode produced a
//! parameter: `EditGameVars` parses/serializes the `KEY=VALUE` string, while
//! `EditTranslationConfigName` treats the value as an opaque config name.

use std::collections::HashSet;

/// Which dialog edit mode a parameter belongs to.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum EditType {
    /// Game environment variables (`KEY=VALUE,KEY2=VALUE2`).
    EditGameVars,
    /// A translation-config name, passed through as an opaque string.
    EditTranslationConfigName,
}

impl EditType {
    pub fn as_str(self) -> &'static str {
        match self {
            EditType::EditGameVars => "EditGameVars",
            EditType::EditTranslationConfigName => "EditTranslationConfigName",
        }
    }
}

/// A single environment parameter (the portable form of `PcSettingEnvEntity`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EnvParam {
    pub key: String,
    pub value: String,
    pub edit_type: EditType,
}

impl EnvParam {
    pub fn new(key: impl Into<String>, value: impl Into<String>, edit_type: EditType) -> Self {
        Self {
            key: key.into(),
            value: value.into(),
            edit_type,
        }
    }

    /// Game-variable shorthand (the common case).
    pub fn game_var(key: impl Into<String>, value: impl Into<String>) -> Self {
        Self::new(key, value, EditType::EditGameVars)
    }

    /// True when both key and value are blank (a row `p()` drops).
    pub fn is_blank(&self) -> bool {
        self.key.trim().is_empty() && self.value.trim().is_empty()
    }

    /// `key=value`, matching `NewInputEnvParamsDialog.q`.
    pub fn render(&self) -> String {
        format!("{}={}", self.key, self.value)
    }
}

/// Error returned by [`validate_env_string`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EnvParseError {
    /// No usable `KEY=VALUE` pair was found (the "input tips" toast case).
    Empty,
}

/// An ordered collection of environment parameters.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct EnvParams {
    pub params: Vec<EnvParam>,
}

impl EnvParams {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn from_vec(params: Vec<EnvParam>) -> Self {
        Self { params }
    }

    pub fn push(&mut self, param: EnvParam) {
        self.params.push(param);
    }

    pub fn len(&self) -> usize {
        self.params.len()
    }

    pub fn is_empty(&self) -> bool {
        self.params.is_empty()
    }

    pub fn iter(&self) -> std::slice::Iter<'_, EnvParam> {
        self.params.iter()
    }

    /// Look up the first value for `key`.
    pub fn get(&self, key: &str) -> Option<&str> {
        self.params
            .iter()
            .find(|p| p.key == key)
            .map(|p| p.value.as_str())
    }

    /// Serialize to the comma-separated `KEY=VALUE` string.
    ///
    /// Mirrors `NewInputEnvParamsDialog.p`: drop fully-blank rows, keep the
    /// first entry for each key, join with `,`. Values may contain commas —
    /// [`EnvParams::parse`] reconstructs them.
    pub fn to_env_string(&self) -> String {
        let mut seen: HashSet<&str> = HashSet::new();
        let mut parts: Vec<String> = Vec::new();
        for param in &self.params {
            if param.is_blank() {
                continue;
            }
            if !seen.insert(param.key.as_str()) {
                continue;
            }
            parts.push(param.render());
        }
        parts.join(",")
    }

    /// Parse a `KEY=VALUE,KEY2=VALUE2` string, mirroring
    /// `PcGameSettingDataHelper.R`.
    ///
    /// Segments are split on `,`. A segment containing `=` starts a new key
    /// (split with a limit of two, so values may themselves contain `=`).
    /// Segments without `=` are appended to the current value, so a value may
    /// contain commas. Keys and values are trimmed, and order is preserved.
    pub fn parse(input: &str) -> Self {
        Self::parse_with_edit_type(input, EditType::EditGameVars)
    }

    /// Parse with an explicit [`EditType`] assigned to each produced param.
    pub fn parse_with_edit_type(input: &str, edit_type: EditType) -> Self {
        let mut params: Vec<EnvParam> = Vec::new();
        let mut current_key: Option<String> = None;
        let mut value_buf = String::new();

        for segment in input.split(',') {
            if let Some(eq) = segment.find('=') {
                // Flush the previously accumulated key/value pair.
                if let Some(key) = current_key.take() {
                    let value = value_buf.trim().to_string();
                    params.push(EnvParam::new(key, value, edit_type));
                    value_buf.clear();
                }
                let key = segment[..eq].trim().to_string();
                let value = segment[eq + 1..].trim().to_string();
                value_buf.push_str(&value);
                current_key = Some(key);
            } else if current_key.is_some() {
                // Continuation of the current value (may contain commas).
                value_buf.push(',');
                value_buf.push_str(segment.trim());
            }
            // Segments before the first '=' with no key are ignored, as in R().
        }

        if let Some(key) = current_key {
            let value = value_buf.trim().to_string();
            params.push(EnvParam::new(key, value, edit_type));
        }

        Self { params }
    }
}

/// Validate a raw env string, returning the parsed collection or
/// [`EnvParseError::Empty`] when it contains no usable key/value pair.
pub fn validate_env_string(input: &str) -> Result<EnvParams, EnvParseError> {
    let parsed = EnvParams::parse(input);
    if parsed.params.iter().all(|p| p.key.is_empty()) {
        Err(EnvParseError::Empty)
    } else {
        Ok(parsed)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_splits_and_trims_key_value_pairs_in_order() {
        let parsed = EnvParams::parse(" A = 1 ,B=2, C = three ");
        assert_eq!(parsed.len(), 3);
        assert_eq!(parsed.get("A"), Some("1"));
        assert_eq!(parsed.get("B"), Some("2"));
        assert_eq!(parsed.get("C"), Some("three"));
        assert_eq!(parsed.params[0].edit_type, EditType::EditGameVars);
        assert_eq!(
            parsed.params.iter().map(|p| p.key.as_str()).collect::<Vec<_>>(),
            vec!["A", "B", "C"]
        );
    }

    #[test]
    fn parse_keeps_commas_and_equals_inside_values() {
        // Comma segments without '=' continue the current value; '=' inside a
        // segment is kept (split with a limit of two).
        let parsed = EnvParams::parse("FLAGS=a,b,c,MODE=x=y");
        assert_eq!(parsed.len(), 2);
        assert_eq!(parsed.get("FLAGS"), Some("a,b,c"));
        assert_eq!(parsed.get("MODE"), Some("x=y"));

        // A segment containing '=' always starts a new key, as in R().
        let with_key = EnvParams::parse("A=1,b=2");
        assert_eq!(with_key.len(), 2);
        assert_eq!(with_key.get("A"), Some("1"));
        assert_eq!(with_key.get("b"), Some("2"));
    }

    #[test]
    fn to_env_string_skips_blank_and_deduplicates_by_key() {
        let params = EnvParams::from_vec(vec![
            EnvParam::game_var("A", "1"),
            EnvParam::game_var("", ""),
            EnvParam::game_var("A", "overwritten"),
            EnvParam::game_var("B", "2"),
        ]);
        assert_eq!(params.to_env_string(), "A=1,B=2");
    }

    #[test]
    fn round_trip_preserves_values_with_commas() {
        let original = EnvParams::from_vec(vec![
            EnvParam::game_var("FLAGS", "a,b"),
            EnvParam::game_var("MODE", "x=y"),
            EnvParam::game_var("EMPTY", ""),
        ]);
        let encoded = original.to_env_string();
        let decoded = EnvParams::parse(&encoded);
        assert_eq!(decoded, original);
    }

    #[test]
    fn translate_config_edit_type_is_preserved_through_parse() {
        let parsed = EnvParams::parse_with_edit_type("NAME=cn", EditType::EditTranslationConfigName);
        assert_eq!(parsed.params[0].edit_type, EditType::EditTranslationConfigName);
        assert_eq!(parsed.params[0].value, "cn");
    }

    #[test]
    fn empty_or_keyless_input_reports_empty() {
        assert!(EnvParams::parse("").is_empty());
        assert_eq!(validate_env_string("   "), Err(EnvParseError::Empty));
        assert_eq!(validate_env_string("no_key_here"), Err(EnvParseError::Empty));
        assert!(validate_env_string("A=1").is_ok());
    }
}
