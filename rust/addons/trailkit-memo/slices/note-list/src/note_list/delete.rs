//! Deleting from the note list (memo M133, M141): the rule that the note in the
//! editor and its groups stay.

use trailkit_memo_shared::note_tree::{NodeId, NoteId, NotesDocument};

/// Whether `node` is or holds the note open in the editor, which cannot be
/// deleted (M141).
pub(super) fn blocks_delete(
    document: &NotesDocument,
    node: NodeId,
    edited: Option<NoteId>,
) -> bool {
    let Some(edited) = edited else {
        return false;
    };
    match node {
        NodeId::Note(note) => note == edited,
        NodeId::Group(group) => document.group(group).is_some_and(|group| {
            group
                .children
                .iter()
                .any(|child| child.contains_note(edited))
        }),
    }
}
