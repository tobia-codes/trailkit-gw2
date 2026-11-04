//! Undo and redo of deletes and moves in the note tree (memo plan M135):
//! 20 steps, kept in memory only.

use std::collections::VecDeque;

use jiff::Timestamp;

use crate::note_tree::{NodeId, NotesDocument, Place};

/// How many steps can be undone (M135).
pub const UNDO_LIMIT: usize = 20;

/// One undoable change of the note tree.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UndoStep {
    /// A note or group was moved to the trash.
    Delete(NodeId),
    /// A note or group was moved from one place to another (drag & drop,
    /// "Move to").
    Move {
        /// The moved note or group.
        node: NodeId,
        /// Where it was before.
        from: Place,
        /// Where it went.
        to: Place,
    },
}

/// Which direction a step is applied in.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Direction {
    /// Take the step back.
    Undo,
    /// Do the step again.
    Redo,
}

/// The steps that can be undone and redone.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct UndoHistory {
    /// Undoable steps, oldest first.
    undo: VecDeque<UndoStep>,
    /// Redoable steps, the next one last.
    redo: Vec<UndoStep>,
}

impl UndoHistory {
    /// Records a step the user just did; drops the oldest beyond
    /// [`UNDO_LIMIT`] and everything that could be redone.
    pub fn record(&mut self, step: UndoStep) {
        self.redo.clear();
        self.undo.push_back(step);
        while self.undo.len() > UNDO_LIMIT {
            self.undo.pop_front();
        }
    }

    /// The step the next undo takes back, if any.
    #[must_use]
    pub fn last(&self) -> Option<UndoStep> {
        self.undo.back().copied()
    }

    /// Takes the latest step back in `document` and returns it. Steps that
    /// no longer apply (e.g. the entry was deleted for good) are dropped.
    pub fn undo(&mut self, document: &mut NotesDocument, now: Timestamp) -> Option<UndoStep> {
        while let Some(step) = self.undo.pop_back() {
            if apply(step, Direction::Undo, document, now) {
                self.redo.push(step);
                return Some(step);
            }
        }
        None
    }

    /// Does the latest undone step again in `document` and returns it.
    pub fn redo(&mut self, document: &mut NotesDocument, now: Timestamp) -> Option<UndoStep> {
        while let Some(step) = self.redo.pop() {
            if apply(step, Direction::Redo, document, now) {
                self.undo.push_back(step);
                return Some(step);
            }
        }
        None
    }
}

/// Applies `step` in `direction`; returns whether the tree changed.
fn apply(
    step: UndoStep,
    direction: Direction,
    document: &mut NotesDocument,
    now: Timestamp,
) -> bool {
    match (step, direction) {
        (UndoStep::Delete(node), Direction::Undo) => document.restore_from_trash(node),
        (UndoStep::Delete(node), Direction::Redo) => document.move_to_trash(node, now),
        (UndoStep::Move { node, from, .. }, Direction::Undo) => document.move_to_place(node, from),
        (UndoStep::Move { node, to, .. }, Direction::Redo) => document.move_to_place(node, to),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::note_tree::{Note, NoteKind, TreeNode};

    /// A root level with three notes; returns the document and their ids.
    fn sample() -> (NotesDocument, [NodeId; 3]) {
        let notes = ["A", "B", "C"]
            .map(|title| Note::new(title.to_owned(), NoteKind::Normal, Timestamp::UNIX_EPOCH));
        let ids = [0, 1, 2].map(|index| NodeId::Note(notes[index].id));
        let document = NotesDocument {
            root: notes.into_iter().map(TreeNode::Note).collect(),
            trash: Vec::new(),
        };
        (document, ids)
    }

    /// The titles of the root level.
    fn titles(document: &NotesDocument) -> Vec<&str> {
        document.root.iter().map(TreeNode::title).collect()
    }

    #[test]
    fn a_delete_is_undone_and_redone() {
        let (mut document, [_, b, _]) = sample();
        let mut history = UndoHistory::default();
        assert!(document.move_to_trash(b, Timestamp::UNIX_EPOCH));
        history.record(UndoStep::Delete(b));
        assert_eq!(
            history.undo(&mut document, Timestamp::UNIX_EPOCH),
            Some(UndoStep::Delete(b))
        );
        assert_eq!(titles(&document), ["A", "B", "C"]);
        assert!(history.redo(&mut document, Timestamp::UNIX_EPOCH).is_some());
        assert_eq!(titles(&document), ["A", "C"]);
    }

    #[test]
    fn a_move_is_undone() {
        let (mut document, [a, _, _]) = sample();
        let mut history = UndoHistory::default();
        let from = document.place(a);
        let to = Place {
            parent: None,
            index: 2,
        };
        assert!(document.move_to_place(a, to));
        if let Some(from) = from {
            history.record(UndoStep::Move { node: a, from, to });
        }
        assert!(history.undo(&mut document, Timestamp::UNIX_EPOCH).is_some());
        assert_eq!(titles(&document), ["A", "B", "C"]);
    }

    #[test]
    fn steps_that_no_longer_apply_are_skipped() {
        let (mut document, [a, b, _]) = sample();
        let mut history = UndoHistory::default();
        for node in [a, b] {
            assert!(document.move_to_trash(node, Timestamp::UNIX_EPOCH));
            history.record(UndoStep::Delete(node));
        }
        assert!(document.delete_from_trash(b).is_some());
        assert_eq!(
            history.undo(&mut document, Timestamp::UNIX_EPOCH),
            Some(UndoStep::Delete(a))
        );
        assert!(history.undo(&mut document, Timestamp::UNIX_EPOCH).is_none());
    }

    #[test]
    fn the_history_keeps_the_latest_steps_and_a_new_step_clears_redo() {
        let (mut document, [a, _, _]) = sample();
        let mut history = UndoHistory::default();
        for _ in 0..UNDO_LIMIT + 5 {
            history.record(UndoStep::Delete(a));
        }
        assert_eq!(history.undo.len(), UNDO_LIMIT);
        assert!(document.move_to_trash(a, Timestamp::UNIX_EPOCH));
        assert!(history.undo(&mut document, Timestamp::UNIX_EPOCH).is_some());
        history.record(UndoStep::Delete(a));
        assert!(history.redo(&mut document, Timestamp::UNIX_EPOCH).is_none());
    }
}
