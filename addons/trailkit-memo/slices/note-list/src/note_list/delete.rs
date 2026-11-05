//! Deleting from the note list (memo M133, M141): the Delete key and the
//! rule that the note in the editor and its groups stay.

use nexus::imgui::{Key, Ui};
use trailkit_memo_shared::note_tree::{NodeId, NoteId, NotesDocument};

use super::NoteListAction;

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

/// The Delete key deletes the `selected` entry while the list's window has
/// the focus and no text input is active (M133). Call it inside the list's
/// window.
pub(super) fn delete_key_action(
    ui: &Ui,
    document: &NotesDocument,
    selected: Option<NodeId>,
    edited: Option<NoteId>,
) -> Option<NoteListAction> {
    let node = selected?;
    let pressed =
        ui.is_window_focused() && !ui.io().want_text_input && ui.is_key_pressed(Key::Delete);
    (pressed && !blocks_delete(document, node, edited)).then_some(NoteListAction::Delete(node))
}
