//! Fields a newer addon version wrote that this version does not know
//! (memo plan M202).

use serde::{Deserialize, Serialize};

use crate::DocumentFields;

/// The fields of a stored object that this build does not know, kept
/// unchanged so that saving does not drop what a newer version added.
///
/// Embed it with `#[serde(flatten)]` next to the known fields: reading
/// collects every remaining field, writing puts them back. Newer versions add
/// optional fields without a `schema_version` bump (M202), so an older
/// version can still save such a file without losing those fields. Like a
/// TypeScript object spread `{ ...unknownFields, ...knownFields }`.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(transparent)]
pub struct UnknownFields(DocumentFields);

impl UnknownFields {
    /// Whether no unknown field was read.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    /// Keeps `value` under `key`, replacing an earlier value. Used when a
    /// known field holds a value this build does not know (M202).
    pub fn insert(&mut self, key: String, value: serde_json::Value) {
        self.0.insert(key, value);
    }

    /// The kept fields in key order, for types that write them by hand.
    pub fn iter(&self) -> impl Iterator<Item = (&String, &serde_json::Value)> {
        self.0.iter()
    }
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;

    /// A stored object with one known field.
    #[derive(Debug, Serialize, Deserialize)]
    struct Stored {
        /// The known field.
        title: String,
        /// Everything else.
        #[serde(flatten)]
        unknown_fields: UnknownFields,
    }

    #[test]
    fn keeps_unknown_fields_across_a_round_trip() -> serde_json::Result<()> {
        let original = json!({"title": "A", "color": "red", "pinned": {"at": 3}});
        let stored: Stored = serde_json::from_value(original.clone())?;
        assert_eq!(stored.title, "A");
        assert!(!stored.unknown_fields.is_empty());
        assert_eq!(serde_json::to_value(&stored)?, original);
        Ok(())
    }

    #[test]
    fn known_only_object_has_no_unknown_fields() -> serde_json::Result<()> {
        let stored: Stored = serde_json::from_value(json!({"title": "A"}))?;
        assert!(stored.unknown_fields.is_empty());
        assert_eq!(serde_json::to_value(&stored)?, json!({"title": "A"}));
        Ok(())
    }
}
