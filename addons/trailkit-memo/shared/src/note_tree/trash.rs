//! The trash of the note tree (memo plan M12, M140, M147, M150) and moving
//! entries to an exact place, which undo needs (M135).

use jiff::{SignedDuration, Timestamp};

use super::{NodeId, Note, NoteId, NotesDocument, Place, TrashEntry, TreeNode};

/// How long a deleted entry stays in the trash before it is purged (M12,
/// M150).
pub const TRASH_RETENTION: SignedDuration = SignedDuration::from_hours(30 * 24);

/// Seconds of one day, for the day counts shown in the trash.
const SECONDS_PER_DAY: i64 = 24 * 60 * 60;

impl TrashEntry {
    /// The id of the deleted note or group.
    #[must_use]
    pub const fn id(&self) -> NodeId {
        self.node.id()
    }

    /// Whole days until the entry is purged at `now`, rounded up (`0` once
    /// it is due).
    #[must_use]
    pub fn days_left(&self, now: Timestamp) -> i64 {
        let remaining = (self.deleted_at + TRASH_RETENTION)
            .duration_since(now)
            .as_secs();
        if remaining <= 0 {
            0
        } else {
            (remaining + SECONDS_PER_DAY - 1) / SECONDS_PER_DAY
        }
    }

    /// Whole days since the entry was deleted at `now` (`0` = within the
    /// last 24 hours).
    #[must_use]
    pub fn days_since_deleted(&self, now: Timestamp) -> i64 {
        (now.duration_since(self.deleted_at).as_secs() / SECONDS_PER_DAY).max(0)
    }

    /// Whether the entry is due to be purged at `now` (M150).
    #[must_use]
    pub fn is_expired(&self, now: Timestamp) -> bool {
        now.duration_since(self.deleted_at) >= TRASH_RETENTION
    }
}

impl NotesDocument {
    /// The place of `node` in the tree, or `None` if it is not in the tree.
    #[must_use]
    pub fn place(&self, node: NodeId) -> Option<Place> {
        self.place_of(node)
    }

    /// Moves `node` to exactly `place` (undo and redo of a move, M135): the
    /// node is taken out first, then inserted at `place.index` (clamped) of
    /// `place.parent`; a missing parent means the end of the root level.
    /// Returns whether it moved; a group never moves into itself.
    pub fn move_to_place(&mut self, node: NodeId, place: Place) -> bool {
        let Some(current) = self.place_of(node) else {
            return false;
        };
        if current == place {
            return false;
        }
        if let NodeId::Group(moved) = node
            && self.is_within(place.parent, moved)
        {
            return false;
        }
        let Some(source) = self.children_mut(current.parent) else {
            return false;
        };
        let moved = source.remove(current.index);
        self.insert_at(place, moved);
        true
    }

    /// Moves `node` with all its children to the trash at `now`,
    /// remembering where it was (M12, M140). Returns whether it was in the
    /// tree.
    pub fn move_to_trash(&mut self, node: NodeId, now: Timestamp) -> bool {
        let Some(place) = self.place_of(node) else {
            return false;
        };
        let Some(source) = self.children_mut(place.parent) else {
            return false;
        };
        let removed = source.remove(place.index);
        self.trash.push(TrashEntry {
            node: removed,
            deleted_at: now,
            former_parent: place.parent,
            former_index: place.index,
        });
        true
    }

    /// The trash entry of `node`, if it is in the trash.
    #[must_use]
    pub fn trash_entry(&self, node: NodeId) -> Option<&TrashEntry> {
        self.trash.iter().find(|entry| entry.id() == node)
    }

    /// A note in the trash, on its own or inside a trashed group.
    #[must_use]
    pub fn note_in_trash(&self, id: NoteId) -> Option<&Note> {
        self.trash.iter().find_map(|entry| entry.node.find_note(id))
    }

    /// Puts a trashed entry back to its former parent at its former index
    /// (clamped), or at the end of the root level if that group is gone
    /// (M140). Groups come back whole (M147). Returns whether it was in the
    /// trash.
    pub fn restore_from_trash(&mut self, node: NodeId) -> bool {
        let Some(index) = self.trash.iter().position(|entry| entry.id() == node) else {
            return false;
        };
        let entry = self.trash.remove(index);
        let place = Place {
            parent: entry.former_parent,
            index: entry.former_index,
        };
        self.insert_at(place, entry.node);
        true
    }

    /// Removes an entry from the trash for good (M146) and returns it.
    pub fn delete_from_trash(&mut self, node: NodeId) -> Option<TrashEntry> {
        let index = self.trash.iter().position(|entry| entry.id() == node)?;
        Some(self.trash.remove(index))
    }

    /// Empties the trash for good (M146) and returns what was in it.
    pub fn empty_trash(&mut self) -> Vec<TrashEntry> {
        std::mem::take(&mut self.trash)
    }

    /// Removes the entries deleted at least [`TRASH_RETENTION`] before `now`
    /// (M150) and returns them.
    pub fn purge_trash(&mut self, now: Timestamp) -> Vec<TrashEntry> {
        let (expired, kept) = std::mem::take(&mut self.trash)
            .into_iter()
            .partition(|entry| entry.is_expired(now));
        self.trash = kept;
        expired
    }

    /// Inserts `node` at `place` (index clamped); a missing parent group
    /// means the end of the root level. The parent group is expanded so the
    /// entry is visible.
    fn insert_at(&mut self, place: Place, node: TreeNode) {
        if let Some(parent) = place.parent.and_then(|parent| self.group_mut(parent)) {
            parent.expanded = true;
        }
        match self.children_mut(place.parent) {
            Some(children) => children.insert(place.index.min(children.len()), node),
            None => self.root.push(node),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::note_tree::{Group, GroupId, NoteKind, node::tests::noon};

    /// A tree `[First, Raids[Weekly, Daily], Last]`; returns the document
    /// and the ids of `First`, `Raids` and `Weekly`.
    fn sample() -> (NotesDocument, NodeId, GroupId, NodeId) {
        let note = |title: &str| Note::new(title.to_owned(), NoteKind::Normal, noon());
        let first = note("First");
        let weekly = note("Weekly");
        let mut raids = Group::new("Raids".to_owned());
        let (first_id, raids_id, weekly_id) =
            (NodeId::Note(first.id), raids.id, NodeId::Note(weekly.id));
        raids.children = vec![TreeNode::Note(weekly), TreeNode::Note(note("Daily"))];
        let document = NotesDocument {
            root: vec![
                TreeNode::Note(first),
                TreeNode::Group(raids),
                TreeNode::Note(note("Last")),
            ],
            trash: Vec::new(),
        };
        (document, first_id, raids_id, weekly_id)
    }

    /// The titles of the root level, for compact assertions.
    fn root_titles(document: &NotesDocument) -> Vec<&str> {
        document.root.iter().map(TreeNode::title).collect()
    }

    #[test]
    fn trashing_and_restoring_returns_to_the_former_place() {
        let (mut document, _, raids, weekly) = sample();
        assert!(document.move_to_trash(weekly, noon()));
        assert!(!document.contains(weekly));
        assert_eq!(document.trash.len(), 1);
        assert!(document.restore_from_trash(weekly));
        assert_eq!(
            document.place(weekly),
            Some(Place {
                parent: Some(raids),
                index: 0
            })
        );
        assert!(document.trash.is_empty());
    }

    #[test]
    fn a_restored_entry_without_its_group_goes_to_the_root_end() {
        let (mut document, _, raids, weekly) = sample();
        assert!(document.move_to_trash(weekly, noon()));
        assert!(document.move_to_trash(NodeId::Group(raids), noon()));
        assert!(document.delete_from_trash(NodeId::Group(raids)).is_some());
        assert!(document.restore_from_trash(weekly));
        assert_eq!(root_titles(&document), ["First", "Last", "Weekly"]);
    }

    #[test]
    fn a_trashed_group_keeps_its_notes() {
        let (mut document, _, raids, weekly) = sample();
        assert!(document.move_to_trash(NodeId::Group(raids), noon()));
        let NodeId::Note(weekly_note) = weekly else {
            unreachable!("the sample's weekly entry is a note")
        };
        assert!(document.note_in_trash(weekly_note).is_some());
        assert!(document.restore_from_trash(NodeId::Group(raids)));
        assert_eq!(root_titles(&document), ["First", "Raids", "Last"]);
        assert!(document.contains(weekly));
    }

    #[test]
    fn moving_to_a_place_and_back_restores_the_order() {
        let (mut document, first, _, _) = sample();
        let from = document.place(first).unwrap_or(Place {
            parent: None,
            index: 0,
        });
        let to = Place {
            parent: None,
            index: 2,
        };
        assert!(document.move_to_place(first, to));
        assert_eq!(root_titles(&document), ["Raids", "Last", "First"]);
        assert!(document.move_to_place(first, from));
        assert_eq!(root_titles(&document), ["First", "Raids", "Last"]);
    }

    #[test]
    fn a_group_never_moves_into_itself() {
        let (mut document, _, raids, _) = sample();
        let inside = Place {
            parent: Some(raids),
            index: 0,
        };
        assert!(!document.move_to_place(NodeId::Group(raids), inside));
    }

    #[test]
    fn purging_removes_only_expired_entries() {
        let (mut document, first, raids, _) = sample();
        assert!(document.move_to_trash(first, noon()));
        let later = noon() + SignedDuration::from_hours(24 * 10);
        assert!(document.move_to_trash(NodeId::Group(raids), later));
        let purged = document.purge_trash(noon() + TRASH_RETENTION);
        assert_eq!(purged.len(), 1);
        assert_eq!(purged[0].id(), first);
        assert_eq!(document.trash.len(), 1);
    }

    #[test]
    fn days_left_round_up_and_end_at_zero() {
        let (mut document, first, _, _) = sample();
        assert!(document.move_to_trash(first, noon()));
        let entry = &document.trash[0];
        assert_eq!(entry.days_left(noon()), 30);
        assert_eq!(entry.days_left(noon() + SignedDuration::from_hours(1)), 30);
        assert_eq!(entry.days_left(noon() + SignedDuration::from_hours(24)), 29);
        assert_eq!(entry.days_left(noon() + TRASH_RETENTION), 0);
        assert_eq!(
            entry.days_since_deleted(noon() + SignedDuration::from_hours(25)),
            1
        );
    }
}
