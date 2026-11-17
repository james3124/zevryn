//! Rust model of `com.xj.winemu.settings.models.GroupSetting` (GameHub Lite
//! v5.1.8). Source: `sources/com/xj/winemu/settings/models/GroupSetting.java`.
//!
//! Java fields: `id`, `name`, `icon: Icons?`, `childs: List<ChildSetting>`.
//! The two `Function1` callback slots and `Parcelable` plumbing are Android UI
//! wiring and are intentionally excluded.

use super::child_setting::ChildSetting;
use super::icons::Icons;

/// A named group of child settings (one card/section in the Java UI).
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct GroupSetting {
    /// Stable group identifier (`id`).
    pub id: String,
    /// Human-facing group title (`name`).
    pub title: String,
    /// Optional group icon (`icon`).
    pub icon: Option<Icons>,
    /// The child rows in display order (`childs`).
    pub children: Vec<ChildSetting>,
}

impl GroupSetting {
    /// Construct a group with an explicit id and title.
    pub fn new(id: impl Into<String>, title: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            title: title.into(),
            icon: None,
            children: Vec::new(),
        }
    }

    /// Construct a group from just a title, deriving an empty id (the common
    /// DSL shape where `GroupSetting(id="", name=title, ...)` is used).
    pub fn named(title: impl Into<String>) -> Self {
        Self::new("", title)
    }

    /// Append a child row.
    pub fn push(&mut self, child: ChildSetting) {
        self.children.push(child);
    }

    /// Insert a child row at `index` (clamped to the end).
    pub fn insert(&mut self, index: usize, child: ChildSetting) {
        let at = index.min(self.children.len());
        self.children.insert(at, child);
    }

    /// Find a child by key.
    pub fn find(&self, key: &str) -> Option<&ChildSetting> {
        self.children.iter().find(|c| c.key == key)
    }

    /// Find a child by key, mutably.
    pub fn find_mut(&mut self, key: &str) -> Option<&mut ChildSetting> {
        self.children.iter_mut().find(|c| c.key == key)
    }

    /// Number of child rows.
    pub fn len(&self) -> usize {
        self.children.len()
    }

    /// True when the group has no child rows.
    pub fn is_empty(&self) -> bool {
        self.children.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn push_find_and_insert() {
        let mut group = GroupSetting::new("general", "General");
        group.push(ChildSetting::action("a", "A"));
        group.push(ChildSetting::action("b", "B"));
        assert_eq!(group.len(), 2);
        assert_eq!(group.find("b").unwrap().label, "B");
        assert!(group.find("missing").is_none());

        group.insert(1, ChildSetting::action("x", "X"));
        let keys: Vec<_> = group.children.iter().map(|c| c.key.as_str()).collect();
        assert_eq!(keys, ["a", "x", "b"]);
    }

    #[test]
    fn named_and_mut_find() {
        let mut group = GroupSetting::named("Compatibility");
        assert_eq!(group.title, "Compatibility");
        group.push(ChildSetting::action("dxvk", "DXVK"));
        group.find_mut("dxvk").unwrap().set_enabled(false);
        assert!(!group.find("dxvk").unwrap().enabled);
        assert!(group.is_empty() == false);
    }
}
