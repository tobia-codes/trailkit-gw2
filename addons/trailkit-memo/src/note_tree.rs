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

impl NotesDocument {
    /// Finds a note anywhere in the tree (not in the trash).
    #[must_use]
    pub fn note(&self, id: NoteId) -> Option<&Note> {
        find_note(&self.root, id)
    }

    /// Finds a note anywhere in the tree (not in the trash) for changing it.
    pub fn note_mut(&mut self, id: NoteId) -> Option<&mut Note> {
        find_note_mut(&mut self.root, id)
    }

    /// Finds a group anywhere in the tree for changing it.
    pub fn group_mut(&mut self, id: GroupId) -> Option<&mut Group> {
        find_group_mut(&mut self.root, id)
    }

    /// Whether `id` exists in the tree (not in the trash).
    #[must_use]
    pub fn contains(&self, id: NodeId) -> bool {
        match id {
            NodeId::Note(note) => self.note(note).is_some(),
            NodeId::Group(group) => parent_of(&self.root, NodeId::Group(group)).is_some(),
        }
    }

    /// The group a new note or group goes into when `selected` is selected
    /// (M43): the selected group itself, the group of a selected note, or
    /// `None` for the root level.
    #[must_use]
    pub fn insertion_parent(&self, selected: Option<NodeId>) -> Option<GroupId> {
        match selected? {
            NodeId::Group(group) if self.contains(NodeId::Group(group)) => Some(group),
            NodeId::Group(_) => None,
            node @ NodeId::Note(_) => parent_of(&self.root, node).flatten(),
        }
    }

    /// Appends `node` at the end of `parent`'s children (or the root level if
    /// `parent` is `None` or no longer exists) and expands that parent.
    pub fn append(&mut self, parent: Option<GroupId>, node: TreeNode) {
        match parent.and_then(|parent| self.group_mut(parent)) {
            Some(group) => {
                group.expanded = true;
                group.children.push(node);
            }
            None => self.root.push(node),
        }
    }

    /// Renames a note or group; an empty or blank title keeps the old one
    /// (M44). Returns whether anything changed. `now` updates a note's
    /// `updated_at`.
    pub fn rename(&mut self, id: NodeId, title: &str, now: Timestamp) -> bool {
        let title = title.trim();
        if title.is_empty() {
            return false;
        }
        match id {
            NodeId::Note(id) => self.note_mut(id).is_some_and(|note| {
                if note.title == title {
                    return false;
                }
                title.clone_into(&mut note.title);
                note.updated_at = now;
                true
            }),
            NodeId::Group(id) => self.group_mut(id).is_some_and(|group| {
                if group.title == title {
                    return false;
                }
                title.clone_into(&mut group.title);
                true
            }),
        }
    }
}

/// Searches `nodes` recursively for the note `id`.
fn find_note(nodes: &[TreeNode], id: NoteId) -> Option<&Note> {
    nodes.iter().find_map(|node| match node {
        TreeNode::Note(note) if note.id == id => Some(note),
        TreeNode::Note(_) => None,
        TreeNode::Group(group) => find_note(&group.children, id),
    })
}

/// Searches `nodes` recursively for the note `id`, mutably.
fn find_note_mut(nodes: &mut [TreeNode], id: NoteId) -> Option<&mut Note> {
    nodes.iter_mut().find_map(|node| match node {
        TreeNode::Note(note) if note.id == id => Some(note),
        TreeNode::Note(_) => None,
        TreeNode::Group(group) => find_note_mut(&mut group.children, id),
    })
}

/// Searches `nodes` recursively for the group `id`, mutably.
fn find_group_mut(nodes: &mut [TreeNode], id: GroupId) -> Option<&mut Group> {
    for node in nodes {
        let TreeNode::Group(group) = node else {
            continue;
        };
        if group.id == id {
            return Some(group);
        }
        if let Some(found) = find_group_mut(&mut group.children, id) {
            return Some(found);
        }
    }
    None
}

/// Finds the parent of `id`: `Some(None)` = root level, `Some(Some(group))`
/// = inside `group`, `None` = not in the tree.
fn parent_of(nodes: &[TreeNode], id: NodeId) -> Option<Option<GroupId>> {
    if nodes.iter().any(|node| node.id() == id) {
        return Some(None);
    }
    nodes.iter().find_map(|node| match node {
        TreeNode::Note(_) => None,
        TreeNode::Group(group) => match parent_of(&group.children, id)? {
            None => Some(Some(group.id)),
            inner => Some(inner),
        },
    })
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

    #[test]
    fn finds_and_renames_nested_nodes() {
        let mut document = sample_document();
        let TreeNode::Group(group) = &document.root[1] else {
            panic!("expected a group");
        };
        let group_id = group.id;
        let TreeNode::Note(nested) = &group.children[0] else {
            panic!("expected a note");
        };
        let nested_id = nested.id;
        let later = Timestamp::now();

        assert!(document.rename(NodeId::Note(nested_id), "  Raid clears ", later));
        let renamed = document.note(nested_id);
        assert_eq!(renamed.map(|note| note.title.as_str()), Some("Raid clears"));
        assert_eq!(renamed.map(|note| note.updated_at), Some(later));
        assert!(
            !document.rename(NodeId::Note(nested_id), "   ", later),
            "blank keeps the title"
        );
        assert!(document.rename(NodeId::Group(group_id), "Raids & strikes", later));
        assert!(!document.rename(NodeId::Group(GroupId::new_unique()), "x", later));
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

    #[test]
    fn insertion_parent_follows_the_selection() {
        let document = sample_document();
        let TreeNode::Note(root_note) = &document.root[0] else {
            panic!("expected a note");
        };
        let TreeNode::Group(group) = &document.root[1] else {
            panic!("expected a group");
        };
        let TreeNode::Note(nested) = &group.children[0] else {
            panic!("expected a note");
        };

        assert_eq!(document.insertion_parent(None), None);
        assert_eq!(
            document.insertion_parent(Some(NodeId::Note(root_note.id))),
            None
        );
        assert_eq!(
            document.insertion_parent(Some(NodeId::Group(group.id))),
            Some(group.id)
        );
        assert_eq!(
            document.insertion_parent(Some(NodeId::Note(nested.id))),
            Some(group.id)
        );
        assert_eq!(
            document.insertion_parent(Some(NodeId::Group(GroupId::new_unique()))),
            None
        );
    }

    #[test]
    fn append_goes_to_the_end_and_expands_the_parent() {
        let mut document = sample_document();
        let TreeNode::Group(group) = &mut document.root[1] else {
            panic!("expected a group");
        };
        group.expanded = false;
        let group_id = group.id;
        let note = Note::new("New".to_owned(), NoteKind::Normal, noon());
        let note_id = note.id;

        document.append(Some(group_id), TreeNode::Note(note));
        let TreeNode::Group(group) = &document.root[1] else {
            panic!("expected a group");
        };
        assert!(group.expanded);
        assert_eq!(
            group.children.last().map(TreeNode::id),
            Some(NodeId::Note(note_id))
        );

        document.append(None, TreeNode::Group(Group::new("Root".to_owned())));
        assert_eq!(document.root.last().map(TreeNode::title), Some("Root"));
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
