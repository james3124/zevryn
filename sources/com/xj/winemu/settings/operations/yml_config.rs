//! Portable descriptor for the YML list parsing in
//! `PcGameSettingOperations.getYmlList` (GameHub Lite v5.1.8).
//!
//! The decompiled helper `PcGameSettingOperations$getYmlList$listType$1`
//! extends `TypeToken<List<String>>`, so `getYmlList` deserializes a YAML list
//! into a `List<String>`. The only portable pieces are the descriptor name and
//! that list element type; the Gson/YAML parsing itself is out of scope.

/// Element type of the list `getYmlList` deserializes. The `TypeToken` in the
/// decompiled sources is `TypeToken<List<String>>`, i.e. string elements.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum YmlListType {
    StringList,
}

impl YmlListType {
    /// Descriptor as written in the decompiled `TypeToken<List<String>>`.
    pub fn descriptor(self) -> &'static str {
        match self {
            Self::StringList => "List<String>",
        }
    }
}

/// Descriptor of a named YML list consumed by `getYmlList`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct YmlConfig {
    /// Config/list name (the key the YML list is stored under).
    pub name: String,
    /// The `TypeToken` list type used for deserialization.
    pub list_type: YmlListType,
}

impl YmlConfig {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            list_type: YmlListType::StringList,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn yml_config_describes_string_list() {
        let config = YmlConfig::new("recommended_yml");
        assert_eq!(config.name, "recommended_yml");
        assert_eq!(config.list_type, YmlListType::StringList);
        assert_eq!(config.list_type.descriptor(), "List<String>");
    }
}
