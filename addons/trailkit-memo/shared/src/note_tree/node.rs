//! Notes, groups, tree entries and trash entries (memo plan M12, M13,
//! M23–M25).

use jiff::Timestamp;
use serde::{Deserialize, Serialize};

use super::ids::{GroupId, NodeId, NoteId};

/// How often a reset note is reset.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ResetPeriod {
    /// Every day at 00:00 UTC.
    Daily,
    /// Every Monday at 07:30 UTC.
    Weekly,
}

impl ResetPeriod {
    /// Both periods, in the order the kind dropdown lists them.
    pub const ALL: [Self; 2] = [Self::Daily, Self::Weekly];
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
    /// When the note's title or content was last changed by the user; kind
    /// changes and resets do not count (M86, M88).
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

impl NoteKind {
    /// A note kind with `period` (`None` = normal note) whose reset state
    /// starts at `now`, so checks made before `now` stay until the next
    /// reset (M90).
    #[must_use]
    pub const fn with_period(period: Option<ResetPeriod>, now: Timestamp) -> Self {
        match period {
            None => Self::Normal,
            Some(ResetPeriod::Daily) => Self::DailyReset {
                last_reset_applied_at: now,
            },
            Some(ResetPeriod::Weekly) => Self::WeeklyReset {
                last_reset_applied_at: now,
            },
        }
    }

    /// The reset period, or `None` for a normal note.
    #[must_use]
    pub const fn period(self) -> Option<ResetPeriod> {
        match self {
            Self::Normal => None,
            Self::DailyReset { .. } => Some(ResetPeriod::Daily),
            Self::WeeklyReset { .. } => Some(ResetPeriod::Weekly),
        }
    }

    /// The reset state, or `None` for a normal note.
    #[must_use]
    pub const fn last_reset_applied_at(self) -> Option<Timestamp> {
        match self {
            Self::Normal => None,
            Self::DailyReset {
                last_reset_applied_at,
            }
            | Self::WeeklyReset {
                last_reset_applied_at,
            } => Some(last_reset_applied_at),
        }
    }
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

    /// Number of notes in this group, including those in nested groups
    /// (shown next to the group title, M50).
    #[must_use]
    pub fn note_count(&self) -> usize {
        self.children
            .iter()
            .map(|child| match child {
                TreeNode::Note(_) => 1,
                TreeNode::Group(group) => group.note_count(),
            })
            .sum()
    }
}

impl TreeNode {
    /// The id of this note or group.
    #[must_use]
    pub const fn id(&self) -> NodeId {
        match self {
            Self::Note(note) => NodeId::Note(note.id),
            Self::Group(group) => NodeId::Group(group.id),
        }
    }

    /// The title shown in the note list.
    #[must_use]
    pub fn title(&self) -> &str {
        match self {
            Self::Note(note) => &note.title,
            Self::Group(group) => &group.title,
        }
    }
}

/// Tests of the node types; `noon` is shared with the document tests.
#[cfg(test)]
pub(super) mod tests {
    use super::*;

    /// A fixed point in time.
    pub(in crate::note_tree) fn noon() -> Timestamp {
        "2026-11-01T12:00:00Z".parse().unwrap_or_default()
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
    fn note_count_includes_nested_groups() {
        let mut outer = Group::new("Outer".to_owned());
        assert_eq!(outer.note_count(), 0);
        let mut inner = Group::new("Inner".to_owned());
        inner.children.push(TreeNode::Note(Note::new(
            "A".to_owned(),
            NoteKind::Normal,
            noon(),
        )));
        inner
            .children
            .push(TreeNode::Group(Group::new("Empty".to_owned())));
        outer.children.push(TreeNode::Group(inner));
        outer.children.push(TreeNode::Note(Note::new(
            "B".to_owned(),
            NoteKind::Normal,
            noon(),
        )));
        assert_eq!(outer.note_count(), 2);
    }
}
