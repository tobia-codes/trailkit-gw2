//! The note tree and its persisted form `notes.json` (memo plan M23–M25).
//!
//! The tree is stored nested: a group owns an ordered list of children, and
//! the order of notes and groups is their position in that list (M24). Notes
//! and groups may live on every level, including the root (M23).
//!
//! Platform-independent so it can be unit-tested on any host (ROADMAP R23).

mod ids;
mod node;
mod traversal;

use jiff::Timestamp;
use serde::{Deserialize, Serialize};
use trailkit_storage::{SchemaVersion, VersionedDocument};

pub use ids::{GroupId, NodeId, NoteId};
pub use node::{Group, Note, NoteKind, ResetPeriod, TrashEntry, TreeNode};
use traversal::{
    Place, find_group, find_group_mut, find_note, find_note_mut, for_each_note_mut, insert_next_to,
    parent_of, place_in,
};

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

    /// Calls `visit` for every note in the tree, nested ones included (the
    /// trash is not part of the tree).
    pub fn for_each_note_mut(&mut self, mut visit: impl FnMut(&mut Note)) {
        for_each_note_mut(&mut self.root, &mut visit);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use node::tests::noon;

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
