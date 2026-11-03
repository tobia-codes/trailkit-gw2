//! Unticking the tasks of reset notes whose reset passed (M88–M90).

use jiff::Timestamp;
use trailkit_memo_shared::note_tree::{NoteId, NoteKind, NotesDocument};
use trailkit_ui::markdown::uncheck_all_tasks;

use crate::schedule::ResetTimes;

/// Applies every due reset at `now` (M88–M90): unchecks all tasks of a
/// reset note whose reset passed and records `now` as its reset state.
/// Notes for which `is_being_edited` is true wait (M89). `updated_at`
/// stays. Returns whether anything changed and must be saved.
pub fn apply_due_resets(
    document: &mut NotesDocument,
    now: Timestamp,
    is_being_edited: impl Fn(NoteId) -> bool,
) -> bool {
    let mut changed = false;
    document.for_each_note_mut(|note| {
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

#[cfg(test)]
mod tests {
    use trailkit_memo_shared::note_tree::{Group, Note, TreeNode};

    use super::*;

    /// A fixed creation time of the sample notes.
    fn noon() -> Timestamp {
        "2026-11-01T12:00:00Z".parse().unwrap_or_default()
    }

    #[test]
    fn due_resets_uncheck_tasks_except_while_editing() {
        let mut weekly = Note::new(
            "Weekly clears".to_owned(),
            NoteKind::WeeklyReset {
                last_reset_applied_at: noon(),
            },
            noon(),
        );
        let mut ideas = Note::new("Ideas".to_owned(), NoteKind::Normal, noon());
        "- [x] boss\n".clone_into(&mut weekly.body);
        "- [x] boss\n".clone_into(&mut ideas.body);
        let (weekly_id, ideas_id) = (weekly.id, ideas.id);
        let mut group = Group::new("Raids".to_owned());
        group.children.push(TreeNode::Note(weekly));
        let mut document = NotesDocument {
            root: vec![TreeNode::Note(ideas), TreeNode::Group(group)],
            ..NotesDocument::default()
        };

        let next_week = "2026-11-02T08:00:00Z".parse().unwrap_or_default();
        assert!(!apply_due_resets(&mut document, next_week, |id| id == weekly_id));
        assert_eq!(
            document.note(weekly_id).map(|note| note.body.as_str()),
            Some("- [x] boss\n")
        );

        assert!(apply_due_resets(&mut document, next_week, |_| false));
        let note = document.note(weekly_id).expect("sample note");
        assert_eq!(note.body, "- [ ] boss\n");
        assert_eq!(note.kind.last_reset_applied_at(), Some(next_week));
        assert_eq!(note.updated_at, noon());
        // Normal notes are never reset; a second run finds nothing due.
        assert_eq!(
            document.note(ideas_id).map(|note| note.body.as_str()),
            Some("- [x] boss\n")
        );
        assert!(!apply_due_resets(&mut document, next_week, |_| false));
    }
}
