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
use trailkit_ui::markdown::uncheck_all_tasks;
use uuid::Uuid;

use crate::reset_schedule::ResetPeriod;

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

/// Where a new or moved note or group goes in the tree (M43, M96,
/// M106–M112).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InsertionPoint {
    /// At the end of a group's children, or of the root level for `None`.
    End(Option<GroupId>),
    /// At the first position of a group's children (M107).
    Start(GroupId),
    /// Right before a note or group, in the same parent.
    Before(NodeId),
    /// Right after a note or group, in the same parent.
    After(NodeId),
}

/// Whether a note or group can be moved to an [`InsertionPoint`]
/// (M109, M110).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MoveCheck {
    /// The move changes the node's place.
    Changes,
    /// The point is the node's current place; moving changes nothing.
    Unchanged,
    /// The move is impossible: a group into itself or one of its
    /// descendants, or the node or the point no longer exists.
    Impossible,
}

/// A place in the tree: the parent (`None` = root level) and the index in
/// its children.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Place {
    /// The parent group, or `None` for the root level.
    parent: Option<GroupId>,
    /// The position in the parent's children.
    index: usize,
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

    /// Finds a group anywhere in the tree.
    #[must_use]
    pub fn group(&self, id: GroupId) -> Option<&Group> {
        find_group(&self.root, id)
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

    /// Inserts `node` at `point`. A group or sibling that no longer exists
    /// falls back to the end of the root level; a parent group is expanded.
    pub fn insert(&mut self, point: InsertionPoint, node: TreeNode) {
        let result = match point {
            InsertionPoint::End(parent) => {
                self.append(parent, node);
                Ok(())
            }
            InsertionPoint::Start(parent) => match self.group_mut(parent) {
                Some(group) => {
                    group.expanded = true;
                    group.children.insert(0, node);
                    Ok(())
                }
                None => Err(node),
            },
            InsertionPoint::Before(sibling) => insert_next_to(&mut self.root, sibling, 0, node),
            InsertionPoint::After(sibling) => insert_next_to(&mut self.root, sibling, 1, node),
        };
        if let Err(node) = result {
            self.root.push(node);
        }
    }

    /// The parent of `node`: `Some(None)` = root level, `Some(Some(group))`
    /// = inside `group`, `None` = not in the tree.
    #[must_use]
    pub fn parent(&self, node: NodeId) -> Option<Option<GroupId>> {
        parent_of(&self.root, node)
    }

    /// Checks whether `node` can be moved to `point` (M109, M110).
    #[must_use]
    pub fn check_move(&self, node: NodeId, point: InsertionPoint) -> MoveCheck {
        let (Some(current), Some(target)) = (self.place_of(node), self.resolve(point)) else {
            return MoveCheck::Impossible;
        };
        if let NodeId::Group(moved) = node
            && self.is_within(target.parent, moved)
        {
            return MoveCheck::Impossible;
        }
        let is_same_place = target.parent == current.parent
            && (target.index == current.index || target.index == current.index + 1);
        if is_same_place {
            MoveCheck::Unchanged
        } else {
            MoveCheck::Changes
        }
    }

    /// Moves `node` with all its children to `point` (M106–M112); the
    /// target group is expanded. Returns whether anything changed; an
    /// unchanged or impossible move ([`Self::check_move`]) does nothing.
    pub fn move_node(&mut self, node: NodeId, point: InsertionPoint) -> bool {
        if self.check_move(node, point) != MoveCheck::Changes {
            return false;
        }
        let (Some(current), Some(target)) = (self.place_of(node), self.resolve(point)) else {
            return false;
        };
        let Some(source) = self.children_mut(current.parent) else {
            return false;
        };
        let moved = source.remove(current.index);
        // Removing the node shifts later siblings in the same list up.
        let index = if target.parent == current.parent && target.index > current.index {
            target.index - 1
        } else {
            target.index
        };
        if let Some(parent) = target.parent.and_then(|parent| self.group_mut(parent)) {
            parent.expanded = true;
        }
        match self.children_mut(target.parent) {
            Some(children) => children.insert(index.min(children.len()), moved),
            None => self.root.push(moved),
        }
        true
    }

    /// The place of `node` in the tree, or `None` if it does not exist.
    fn place_of(&self, node: NodeId) -> Option<Place> {
        place_in(&self.root, None, node)
    }

    /// The place `point` stands for, or `None` if its group or sibling no
    /// longer exists.
    fn resolve(&self, point: InsertionPoint) -> Option<Place> {
        match point {
            InsertionPoint::End(parent) => Some(Place {
                parent,
                index: self.children(parent)?.len(),
            }),
            InsertionPoint::Start(parent) => {
                self.group(parent)?;
                Some(Place {
                    parent: Some(parent),
                    index: 0,
                })
            }
            InsertionPoint::Before(sibling) => self.place_of(sibling),
            InsertionPoint::After(sibling) => self.place_of(sibling).map(|place| Place {
                index: place.index + 1,
                ..place
            }),
        }
    }

    /// Whether `group` is `ancestor` or nested somewhere inside it; the root
    /// level (`None`) is inside no group.
    fn is_within(&self, group: Option<GroupId>, ancestor: GroupId) -> bool {
        let Some(group) = group else {
            return false;
        };
        group == ancestor
            || self.group(ancestor).is_some_and(|ancestor| {
                parent_of(&ancestor.children, NodeId::Group(group)).is_some()
            })
    }

    /// The children of `parent`, or the root level for `None`.
    fn children(&self, parent: Option<GroupId>) -> Option<&[TreeNode]> {
        match parent {
            None => Some(&self.root),
            Some(parent) => self.group(parent).map(|group| group.children.as_slice()),
        }
    }

    /// The children of `parent` for changing them, or the root level for
    /// `None`.
    fn children_mut(&mut self, parent: Option<GroupId>) -> Option<&mut Vec<TreeNode>> {
        match parent {
            None => Some(&mut self.root),
            Some(parent) => self.group_mut(parent).map(|group| &mut group.children),
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

    /// Changes the reset period of note `id` (`None` = normal note, M86).
    /// A new reset period starts at `now` (M90); `updated_at` stays. Returns
    /// whether anything changed.
    pub fn set_reset_period(
        &mut self,
        id: NoteId,
        period: Option<ResetPeriod>,
        now: Timestamp,
    ) -> bool {
        self.note_mut(id).is_some_and(|note| {
            if note.kind.period() == period {
                return false;
            }
            note.kind = NoteKind::with_period(period, now);
            true
        })
    }

    /// Applies every due reset at `now` (M88–M90): unchecks all tasks of a
    /// reset note whose reset passed and records `now` as its reset state.
    /// Notes for which `is_being_edited` is true wait (M89). `updated_at`
    /// stays. Returns whether anything changed and must be saved.
    pub fn apply_due_resets(
        &mut self,
        now: Timestamp,
        is_being_edited: impl Fn(NoteId) -> bool,
    ) -> bool {
        let mut changed = false;
        for_each_note_mut(&mut self.root, &mut |note| {
            let (Some(period), Some(last_reset_applied_at)) =
                (note.kind.period(), note.kind.last_reset_applied_at())
            else {
                return;
            };
            if !period.is_due(last_reset_applied_at, now) || is_being_edited(note.id) {
                return;
            }
            if let Some(body) = uncheck_all_tasks(&note.body) {
                note.body = body;
            }
            note.kind = NoteKind::with_period(Some(period), now);
            changed = true;
        });
        changed
    }
}

/// Inserts `node` next to `sibling` (`offset` 0 = before, 1 = after) in
/// whichever list of `nodes` (or their descendants) holds it; hands `node`
/// back if `sibling` is missing.
fn insert_next_to(
    nodes: &mut Vec<TreeNode>,
    sibling: NodeId,
    offset: usize,
    node: TreeNode,
) -> Result<(), TreeNode> {
    if let Some(index) = nodes.iter().position(|candidate| candidate.id() == sibling) {
        nodes.insert(index + offset, node);
        return Ok(());
    }
    let mut node = node;
    for candidate in nodes.iter_mut() {
        if let TreeNode::Group(group) = candidate {
            match insert_next_to(&mut group.children, sibling, offset, node) {
                Ok(()) => return Ok(()),
                Err(returned) => node = returned,
            }
        }
    }
    Err(node)
}

/// Calls `visit` for every note in `nodes`, nested ones included.
fn for_each_note_mut(nodes: &mut [TreeNode], visit: &mut impl FnMut(&mut Note)) {
    for node in nodes {
        match node {
            TreeNode::Note(note) => visit(note),
            TreeNode::Group(group) => for_each_note_mut(&mut group.children, visit),
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

/// Searches `nodes` recursively for the group `id`.
fn find_group(nodes: &[TreeNode], id: GroupId) -> Option<&Group> {
    nodes.iter().find_map(|node| match node {
        TreeNode::Note(_) => None,
        TreeNode::Group(group) if group.id == id => Some(group),
        TreeNode::Group(group) => find_group(&group.children, id),
    })
}

/// Finds the place of `id` in `nodes` (the children of `parent`) or their
/// descendants.
fn place_in(nodes: &[TreeNode], parent: Option<GroupId>, id: NodeId) -> Option<Place> {
    if let Some(index) = nodes.iter().position(|node| node.id() == id) {
        return Some(Place { parent, index });
    }
    nodes.iter().find_map(|node| match node {
        TreeNode::Note(_) => None,
        TreeNode::Group(group) => place_in(&group.children, Some(group.id), id),
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

    /// The id of the first note titled `title` in the root or first group.
    fn note_id(document: &NotesDocument, title: &str) -> Option<NoteId> {
        let mut found = None;
        let mut copy = document.clone();
        for_each_note_mut(&mut copy.root, &mut |note| {
            if note.title == title && found.is_none() {
                found = Some(note.id);
            }
        });
        found
    }

    #[test]
    fn kind_change_starts_a_new_period_and_keeps_updated_at() {
        let mut document = sample_document();
        let id = note_id(&document, "Ideas").expect("sample note");
        let later = "2026-11-01T18:00:00Z".parse().unwrap_or_default();
        assert!(document.set_reset_period(id, Some(ResetPeriod::Daily), later));
        assert!(!document.set_reset_period(id, Some(ResetPeriod::Daily), later));
        let note = document.note(id).expect("sample note");
        assert_eq!(note.kind.last_reset_applied_at(), Some(later));
        assert_eq!(note.updated_at, noon());
        assert!(document.set_reset_period(id, None, later));
        assert_eq!(
            document.note(id).map(|note| note.kind),
            Some(NoteKind::Normal)
        );
    }

    #[test]
    fn due_resets_uncheck_tasks_except_while_editing() {
        let mut document = sample_document();
        let weekly = note_id(&document, "Weekly clears").expect("sample note");
        let ideas = note_id(&document, "Ideas").expect("sample note");
        for id in [weekly, ideas] {
            if let Some(note) = document.note_mut(id) {
                "- [x] boss\n".clone_into(&mut note.body);
            }
        }
        let next_week = "2026-11-02T08:00:00Z".parse().unwrap_or_default();
        assert!(!document.apply_due_resets(next_week, |id| id == weekly));
        assert_eq!(
            document.note(weekly).map(|note| note.body.as_str()),
            Some("- [x] boss\n")
        );

        assert!(document.apply_due_resets(next_week, |_| false));
        let note = document.note(weekly).expect("sample note");
        assert_eq!(note.body, "- [ ] boss\n");
        assert_eq!(note.kind.last_reset_applied_at(), Some(next_week));
        assert_eq!(note.updated_at, noon());
        // Normal notes are never reset; a second run finds nothing due.
        assert_eq!(
            document.note(ideas).map(|note| note.body.as_str()),
            Some("- [x] boss\n")
        );
        assert!(!document.apply_due_resets(next_week, |_| false));
    }

    #[test]
    fn insert_after_a_nested_note_and_fall_back_to_the_root() {
        let mut document = sample_document();
        let weekly = note_id(&document, "Weekly clears").expect("sample note");
        let new = Note::new("New".to_owned(), NoteKind::Normal, noon());
        let new_id = new.id;
        document.insert(
            InsertionPoint::After(NodeId::Note(weekly)),
            TreeNode::Note(new),
        );
        let Some(TreeNode::Group(group)) = document.root.get(1) else {
            panic!("the sample has a group second");
        };
        assert_eq!(
            group.children.get(1).map(TreeNode::id),
            Some(NodeId::Note(new_id))
        );

        let orphan = Note::new("Orphan".to_owned(), NoteKind::Normal, noon());
        let orphan_id = orphan.id;
        document.insert(
            InsertionPoint::After(NodeId::Note(NoteId::new_unique())),
            TreeNode::Note(orphan),
        );
        assert_eq!(
            document.root.last().map(TreeNode::id),
            Some(NodeId::Note(orphan_id))
        );
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

    /// The tree `A G1[B G2[C]] D` with G2 collapsed, plus the ids of
    /// G1, G2, A, B and C.
    fn move_document() -> (NotesDocument, [GroupId; 2], [NoteId; 3]) {
        let note = |title: &str| Note::new(title.to_owned(), NoteKind::Normal, noon());
        let (a, b, c, d) = (note("A"), note("B"), note("C"), note("D"));
        let ids = [a.id, b.id, c.id];
        let mut inner = Group::new("G2".to_owned());
        inner.expanded = false;
        inner.children.push(TreeNode::Note(c));
        let mut outer = Group::new("G1".to_owned());
        outer.children.push(TreeNode::Note(b));
        let groups = [outer.id, inner.id];
        outer.children.push(TreeNode::Group(inner));
        let document = NotesDocument {
            root: vec![TreeNode::Note(a), TreeNode::Group(outer), TreeNode::Note(d)],
            trash: Vec::new(),
        };
        (document, groups, ids)
    }

    /// The tree as text, e.g. `A G1[B G2[C]] D`; a collapsed group is
    /// marked with `-`.
    fn outline(nodes: &[TreeNode]) -> String {
        nodes
            .iter()
            .map(|node| match node {
                TreeNode::Note(note) => note.title.clone(),
                TreeNode::Group(group) => format!(
                    "{}{}[{}]",
                    group.title,
                    if group.expanded { "" } else { "-" },
                    outline(&group.children)
                ),
            })
            .collect::<Vec<_>>()
            .join(" ")
    }

    #[test]
    fn moves_notes_to_every_kind_of_point() {
        let (mut document, [outer, inner], [a, b, c]) = move_document();
        let [a, b, c] = [a, b, c].map(NodeId::Note);
        assert!(document.move_node(a, InsertionPoint::After(b)));
        assert_eq!(outline(&document.root), "G1[B A G2-[C]] D");
        assert!(document.move_node(a, InsertionPoint::Start(outer)));
        assert_eq!(outline(&document.root), "G1[A B G2-[C]] D");
        assert!(document.move_node(c, InsertionPoint::End(None)));
        assert_eq!(outline(&document.root), "G1[A B G2-[]] D C");
        assert!(document.move_node(b, InsertionPoint::End(Some(inner))));
        assert_eq!(outline(&document.root), "G1[A G2[B]] D C");
        assert!(document.move_node(c, InsertionPoint::Before(NodeId::Group(outer))));
        assert_eq!(outline(&document.root), "C G1[A G2[B]] D");
    }

    #[test]
    fn moving_down_in_the_same_list_accounts_for_the_removed_node() {
        let (mut document, _, [a, ..]) = move_document();
        let d = document.root[2].id();
        assert!(document.move_node(NodeId::Note(a), InsertionPoint::After(d)));
        assert_eq!(outline(&document.root), "G1[B G2-[C]] D A");
    }

    #[test]
    fn a_group_moves_with_its_children() {
        let (mut document, [_, inner], [a, ..]) = move_document();
        assert!(document.move_node(
            NodeId::Group(inner),
            InsertionPoint::Before(NodeId::Note(a))
        ));
        assert_eq!(outline(&document.root), "G2-[C] A G1[B] D");
    }

    #[test]
    fn checks_impossible_and_unchanged_moves() {
        let (mut document, [outer, inner], [a, b, c]) = move_document();
        let outer_node = NodeId::Group(outer);
        // A group cannot go into itself or one of its descendants (M109).
        for point in [
            InsertionPoint::Start(outer),
            InsertionPoint::End(Some(inner)),
            InsertionPoint::After(NodeId::Note(b)),
            InsertionPoint::Before(NodeId::Note(c)),
        ] {
            assert_eq!(
                document.check_move(outer_node, point),
                MoveCheck::Impossible
            );
        }
        // Next to its own place changes nothing (M110).
        for point in [
            InsertionPoint::Before(outer_node),
            InsertionPoint::After(outer_node),
            InsertionPoint::After(NodeId::Note(a)),
            InsertionPoint::Before(document.root[2].id()),
        ] {
            assert_eq!(document.check_move(outer_node, point), MoveCheck::Unchanged);
        }
        assert_eq!(
            document.check_move(NodeId::Note(b), InsertionPoint::Start(outer)),
            MoveCheck::Unchanged
        );
        assert_eq!(
            document.check_move(NodeId::Note(c), InsertionPoint::End(Some(inner))),
            MoveCheck::Unchanged
        );
        let before = document.clone();
        assert!(!document.move_node(outer_node, InsertionPoint::End(Some(inner))));
        assert!(!document.move_node(outer_node, InsertionPoint::After(NodeId::Note(a))));
        assert_eq!(document, before);
        assert_eq!(
            document.check_move(
                NodeId::Note(NoteId::new_unique()),
                InsertionPoint::End(None)
            ),
            MoveCheck::Impossible
        );
    }

    #[test]
    fn insert_before_and_at_the_start() {
        let (mut document, [outer, _], [_, b, _]) = move_document();
        let note =
            |title: &str| TreeNode::Note(Note::new(title.to_owned(), NoteKind::Normal, noon()));
        document.insert(InsertionPoint::Before(NodeId::Note(b)), note("X"));
        document.insert(InsertionPoint::Start(outer), note("Y"));
        document.insert(InsertionPoint::Start(GroupId::new_unique()), note("Z"));
        assert_eq!(outline(&document.root), "A G1[Y X B G2-[C]] D Z");
    }
}
