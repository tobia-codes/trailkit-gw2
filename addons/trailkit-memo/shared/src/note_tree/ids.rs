//! Ids of notes and groups and their ImGui id-stack form (memo plan M23).

use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Stable unique id of a note (UUID v7, time-ordered).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct NoteId(Uuid);

/// Stable unique id of a group (UUID v7, time-ordered).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct GroupId(Uuid);

impl NoteId {
    /// Creates a new, unique note id.
    #[must_use]
    pub fn new_unique() -> Self {
        Self(Uuid::now_v7())
    }

    /// A stable 32-bit id for ImGui's id stack (collisions are harmless and
    /// practically impossible within one tree).
    #[must_use]
    pub const fn imgui_id(self) -> i32 {
        fold_to_i32(self.0)
    }
}

impl GroupId {
    /// Creates a new, unique group id.
    #[must_use]
    pub fn new_unique() -> Self {
        Self(Uuid::now_v7())
    }

    /// A stable 32-bit id for ImGui's id stack, see [`NoteId::imgui_id`].
    #[must_use]
    pub const fn imgui_id(self) -> i32 {
        fold_to_i32(self.0)
    }
}

/// Folds a UUID into 32 bits by XOR-ing its four 32-bit words.
#[expect(
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    reason = "deliberate folding into a hash"
)]
const fn fold_to_i32(uuid: Uuid) -> i32 {
    let value = uuid.as_u128();
    (value ^ (value >> 32) ^ (value >> 64) ^ (value >> 96)) as u32 as i32
}

/// A reference to a note or a group in the tree.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(tag = "type", content = "id", rename_all = "snake_case")]
pub enum NodeId {
    /// A note.
    Note(NoteId),
    /// A group.
    Group(GroupId),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ids_are_unique() {
        assert_ne!(NoteId::new_unique(), NoteId::new_unique());
        assert_ne!(GroupId::new_unique(), GroupId::new_unique());
    }
    #[test]
    fn node_id_serializes_tagged() -> serde_json::Result<()> {
        let id = NodeId::Group(GroupId::new_unique());
        let value = serde_json::to_value(id)?;
        assert_eq!(value["type"], "group");
        assert_eq!(serde_json::from_value::<NodeId>(value)?, id);
        Ok(())
    }
}
