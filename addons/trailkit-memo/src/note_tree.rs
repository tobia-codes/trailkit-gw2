//! The note tree and its persisted form `notes.json` (memo plan M23–M25).
//!
//! The tree is stored nested: a group owns an ordered list of children, and
//! the order of notes and groups is their position in that list (M24). Notes
//! and groups may live on every level, including the root (M23).
//!
//! Platform-independent so it can be unit-tested on any host (ROADMAP R23).

use jiff::Timestamp;
use serde::{Deserialize, Serialize};
use trailkit_storage::{SchemaVersion, VersionedDocument};
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
}

impl GroupId {
    /// Creates a new, unique group id.
    #[must_use]
    pub fn new_unique() -> Self {
        Self(Uuid::now_v7())
    }
}

/// What happens to a note's checkboxes at the game's reset times (M13).
///
/// Stored flat in the note as `"kind"` plus, for reset kinds,
/// `"last_reset_applied_at"`; a normal note cannot carry a reset time.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum NoteKind {
    /// A plain note; nothing is reset.
    Normal,
    /// All checkboxes are unchecked at the daily reset (00:00 UTC).
    DailyReset {
        /// The reset that was last applied to this note.
        last_reset_applied_at: Timestamp,
    },
    /// All checkboxes are unchecked at the weekly reset (Monday 07:30 UTC).
    WeeklyReset {
        /// The reset that was last applied to this note.
        last_reset_applied_at: Timestamp,
    },
}

/// A markdown note.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Note {
    /// Stable unique id.
    pub id: NoteId,
    /// Title shown in the note list.
    pub title: String,
    /// The note content as markdown.
    pub body: String,
    /// Normal or reset note, with its reset state.
    #[serde(flatten)]
    pub kind: NoteKind,
    /// When the note was created.
    pub created_at: Timestamp,
    /// When the note's title, content or kind was last changed.
    pub updated_at: Timestamp,
}

/// A group of notes and subgroups.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Group {
    /// Stable unique id.
    pub id: GroupId,
    /// Title shown in the note list.
    pub title: String,
    /// Notes and subgroups in display order.
    pub children: Vec<TreeNode>,
    /// Whether the group is expanded in the note list (kept across sessions).
    pub expanded: bool,
}

/// One entry of the tree: a note or a group.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum TreeNode {
    /// A note.
    Note(Note),
    /// A group with its own children.
    Group(Group),
}

/// A deleted note or group waiting in the trash (M12, M25).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TrashEntry {
    /// The deleted note or group, including all its children.
    pub node: TreeNode,
    /// When it was deleted; it is purged 30 days later (M12).
    pub deleted_at: Timestamp,
    /// The group it was in, or `None` if it was on the root level.
    pub former_parent: Option<GroupId>,
    /// Its position within the former parent's children.
    pub former_index: usize,
}

/// Everything stored in `notes.json`.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct NotesDocument {
    /// Notes and groups on the root level, in display order.
    pub root: Vec<TreeNode>,
    /// Deleted notes and groups.
    pub trash: Vec<TrashEntry>,
}

impl VersionedDocument for NotesDocument {
    const FILE_STEM: &'static str = "notes";
    const CURRENT_SCHEMA_VERSION: SchemaVersion = SchemaVersion::FIRST;
}

impl Note {
    /// Creates a new, empty note of `kind` created at `now`.
    ///
    /// For reset kinds, the reset state starts at `now`, so the first reset
    /// after creation is applied.
    #[must_use]
    pub fn new(title: String, kind: NoteKind, now: Timestamp) -> Self {
        Self {
            id: NoteId::new_unique(),
            title,
            body: String::new(),
            kind,
            created_at: now,
            updated_at: now,
        }
    }
}

impl Group {
    /// Creates a new, empty and expanded group.
    #[must_use]
    pub fn new(title: String) -> Self {
        Self {
            id: GroupId::new_unique(),
            title,
            children: Vec::new(),
            expanded: true,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A fixed point in time.
    fn noon() -> Timestamp {
        "2026-11-01T12:00:00Z".parse().unwrap_or_default()
    }

    /// A small tree with every node and kind variant.
    fn sample_document() -> NotesDocument {
        let mut group = Group::new("Raids".to_owned());
        group.children.push(TreeNode::Note(Note::new(
            "Weekly clears".to_owned(),
            NoteKind::WeeklyReset {
                last_reset_applied_at: noon(),
            },
            noon(),
        )));
        NotesDocument {
            root: vec![
                TreeNode::Note(Note::new("Ideas".to_owned(), NoteKind::Normal, noon())),
                TreeNode::Group(group),
            ],
            trash: vec![TrashEntry {
                node: TreeNode::Note(Note::new(
                    "Old".to_owned(),
                    NoteKind::DailyReset {
                        last_reset_applied_at: noon(),
                    },
                    noon(),
                )),
                deleted_at: noon(),
                former_parent: None,
                former_index: 0,
            }],
        }
    }

    #[test]
    fn json_round_trip_keeps_everything() -> serde_json::Result<()> {
        let document = sample_document();
        let json = serde_json::to_string(&document)?;
        assert_eq!(serde_json::from_str::<NotesDocument>(&json)?, document);
        Ok(())
    }

    #[test]
    fn note_is_stored_flat_with_kind_and_type_tags() -> serde_json::Result<()> {
        let note = Note::new(
            "Dailies".to_owned(),
            NoteKind::DailyReset {
                last_reset_applied_at: noon(),
            },
            noon(),
        );
        let value = serde_json::to_value(TreeNode::Note(note))?;
        assert_eq!(value["type"], "note");
        assert_eq!(value["kind"], "daily_reset");
        assert_eq!(value["last_reset_applied_at"], "2026-11-01T12:00:00Z");
        assert_eq!(value["created_at"], "2026-11-01T12:00:00Z");
        Ok(())
    }

    #[test]
    fn normal_note_has_no_reset_time() -> serde_json::Result<()> {
        let note = Note::new("Ideas".to_owned(), NoteKind::Normal, noon());
        let value = serde_json::to_value(note)?;
        assert_eq!(value["kind"], "normal");
        assert!(value.get("last_reset_applied_at").is_none());
        Ok(())
    }

    #[test]
    fn ids_are_unique() {
        assert_ne!(NoteId::new_unique(), NoteId::new_unique());
        assert_ne!(GroupId::new_unique(), GroupId::new_unique());
    }
}
