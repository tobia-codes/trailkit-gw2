//! Tests of deleting, the trash and undo (memo M133–M151).

use jiff::SignedDuration;
use trailkit_memo_shared::note_tree::{InsertionPoint, TrashEntry};
use trailkit_memo_trash::trash_view::TrashAction;

use super::*;

/// The ids of the "Raids" group and its "Weekly clears" note.
fn raids_and_weekly(document: &NotesDocument) -> (NodeId, Option<NodeId>) {
    let raids = document
        .root
        .iter()
        .find_map(|node| match node {
            Node::Group(group) => Some(NodeId::Group(group.id)),
            Node::Note(_) => None,
        })
        .unwrap_or_else(|| panic!("the sample has a group"));
    (raids, weekly_note(document))
}

/// The sample with `days_ago`-old trash entries: a note and a group.
pub(super) fn document_with_trash(days_ago: [i64; 2]) -> NotesDocument {
    let mut document = sample_document();
    let now = Timestamp::now();
    let mut group = Group::new("Fractals".to_owned());
    group
        .children
        .push(Node::Note(note("Daily T4", "", NoteKind::Normal)));
    group
        .children
        .push(Node::Note(note("CMs", "", NoteKind::Normal)));
    let old = note(
        "Old shopping list",
        "- [ ] Mithril ore\n- [x] Ectos",
        NoteKind::Normal,
    );
    document.trash = [
        (Node::Note(old), days_ago[0]),
        (Node::Group(group), days_ago[1]),
    ]
    .into_iter()
    .map(|(node, days)| TrashEntry {
        node,
        deleted_at: now - SignedDuration::from_hours(24 * days),
        former_parent: None,
        former_index: 0,
    })
    .collect();
    document
}

#[test]
fn deleting_a_note_selects_the_next_row_and_can_be_undone() {
    let document = sample_document();
    let (mut memo, _worker, _directory) = memo_window(document.clone(), selected_first(&document));
    let first = first_note(&document).unwrap_or_else(|| panic!("the sample has a note"));
    memo.apply(&WindowAction::Delete(first), Language::English);
    assert_eq!(outline(&memo), "Dailies, Raids[Weekly clears]");
    assert_eq!(memo.document.trash.len(), 1);
    assert_ne!(memo.ui_state.selected, Some(first), "the row below (M149)");
    assert!(memo.toast.is_some(), "the undo toast shows (M148)");
    memo.apply(&WindowAction::Undo, Language::English);
    assert_eq!(
        outline(&memo),
        "Shopping list, Dailies, Raids[Weekly clears]"
    );
    assert_eq!(memo.ui_state.selected, Some(first));
    assert!(memo.toast.is_none());
    memo.apply(&WindowAction::Redo, Language::English);
    assert_eq!(memo.document.trash.len(), 1);
}

#[test]
fn deleting_a_group_with_notes_asks_first() {
    let document = sample_document();
    let (raids, _) = raids_and_weekly(&document);
    let (mut memo, _worker, _directory) = memo_window(document, UiState::default());
    memo.apply(&WindowAction::Delete(raids), Language::English);
    assert!(memo.confirmation.is_some(), "M134");
    assert!(memo.document.trash.is_empty());
    memo.apply(&WindowAction::ConfirmAccepted, Language::English);
    assert_eq!(outline(&memo), "Shopping list, Dailies");
    assert_eq!(memo.document.trash.len(), 1);
}

#[test]
fn the_edited_note_cannot_be_deleted() {
    let document = sample_document();
    let (raids, weekly) = raids_and_weekly(&document);
    let ui_state = UiState {
        selected: weekly,
        ..UiState::default()
    };
    let (mut memo, _worker, _directory) = memo_window(document, ui_state);
    memo.apply(&WindowAction::StartEditing, Language::English);
    for node in [raids].into_iter().chain(weekly) {
        memo.apply(&WindowAction::Delete(node), Language::English);
    }
    assert!(
        memo.confirmation.is_none() && memo.document.trash.is_empty(),
        "M141"
    );
}

#[test]
fn moves_can_be_undone() {
    let document = sample_document();
    let first = first_note(&document).unwrap_or_else(|| panic!("the sample has a note"));
    let (mut memo, _worker, _directory) = memo_window(document, UiState::default());
    memo.apply(
        &WindowAction::Move(first, InsertionPoint::End(None)),
        Language::English,
    );
    assert_eq!(
        outline(&memo),
        "Dailies, Raids[Weekly clears], Shopping list"
    );
    memo.apply(&WindowAction::Undo, Language::English);
    assert_eq!(
        outline(&memo),
        "Shopping list, Dailies, Raids[Weekly clears]"
    );
}

#[test]
fn restoring_and_deleting_for_good_in_the_trash() {
    let document = document_with_trash([2, 3]);
    let (old, fractals) = (document.trash[0].id(), document.trash[1].id());
    let (mut memo, _worker, _directory) = memo_window(document, UiState::default());
    memo.apply(&WindowAction::ToggleTrash, Language::English);
    memo.apply(
        &WindowAction::Trash(TrashAction::Restore(old)),
        Language::English,
    );
    assert_eq!(
        outline(&memo),
        "Old shopping list, Shopping list, Dailies, Raids[Weekly clears]",
        "back at its former place (M140)"
    );
    memo.apply(
        &WindowAction::Trash(TrashAction::DeletePermanently(fractals)),
        Language::English,
    );
    assert_eq!(memo.document.trash.len(), 1, "asks first (M146)");
    memo.apply(&WindowAction::ConfirmAccepted, Language::English);
    assert!(memo.document.trash.is_empty());
}

#[test]
fn expired_entries_are_purged() {
    let document = document_with_trash([31, 3]);
    let (mut memo, _worker, _directory) = memo_window(document, UiState::default());
    memo.purge_trash(Timestamp::now());
    assert_eq!(memo.document.trash.len(), 1, "M150");
}

#[test]
fn the_back_arrow_leaves_the_trash() {
    let (mut memo, _worker, _directory) = memo_window(sample_document(), UiState::default());
    memo.apply(&WindowAction::ToggleTrash, Language::English);
    assert!(memo.trash.visible);
    memo.apply(
        &WindowAction::Trash(TrashAction::LeaveTrash),
        Language::English,
    );
    assert!(!memo.trash.visible, "M154");
}

#[test]
fn the_trash_shows_its_entries_and_the_selected_one() {
    let document = document_with_trash([0, 25]);
    let selected = document.trash[0].id();
    let (mut memo, _worker, _directory) = memo_window(document, UiState::default());
    memo.apply(&WindowAction::ToggleTrash, Language::English);
    memo.apply(
        &WindowAction::Trash(TrashAction::Select(selected)),
        Language::English,
    );
    assert_window(&mut memo, [640.0, 360.0], ThemeKind::Dark, "memo-trash");
}

#[test]
fn deleting_for_good_asks_in_a_dialog() {
    let document = document_with_trash([0, 25]);
    let selected = document.trash[0].id();
    let (mut memo, _worker, _directory) = memo_window(document, UiState::default());
    memo.apply(&WindowAction::ToggleTrash, Language::English);
    memo.apply(
        &WindowAction::Trash(TrashAction::DeletePermanently(selected)),
        Language::English,
    );
    assert_window(
        &mut memo,
        [640.0, 360.0],
        ThemeKind::Dark,
        "memo-delete-permanently",
    );
}

#[test]
fn deleting_shows_the_undo_toast() {
    let document = sample_document();
    let (mut memo, _worker, _directory) = memo_window(document.clone(), selected_first(&document));
    let first = first_note(&document).unwrap_or_else(|| panic!("the sample has a note"));
    memo.apply(&WindowAction::Delete(first), Language::English);
    assert_window(
        &mut memo,
        [640.0, 360.0],
        ThemeKind::Dark,
        "memo-delete-toast",
    );
}

#[test]
fn the_toast_opens_the_trash_and_shows_the_hidden_list() {
    let document = sample_document();
    let ui_state = UiState {
        note_list_hidden: true,
        ..selected_first(&document)
    };
    let (mut memo, _worker, _directory) = memo_window(document.clone(), ui_state);
    let first = first_note(&document).unwrap_or_else(|| panic!("the sample has a note"));
    memo.apply(&WindowAction::Delete(first), Language::English);
    assert!(memo.toast.is_some());
    memo.apply(&WindowAction::OpenTrash, Language::English);
    assert!(memo.trash.visible, "the way to the trash (M158)");
    assert!(memo.toast.is_none(), "the toast has done its job");
    assert!(
        !memo.ui_state.note_list_hidden,
        "the trash needs its column"
    );
}

#[test]
fn deleting_and_undoing_keep_the_drawer_open() {
    let document = sample_document();
    let (mut memo, _worker, _directory) = memo_window(document.clone(), selected_first(&document));
    let first = first_note(&document).unwrap_or_else(|| panic!("the sample has a note"));
    memo.drawer_open = true;
    memo.apply(&WindowAction::Delete(first), Language::English);
    assert!(memo.drawer_open, "more entries may follow (M164)");
    assert_ne!(
        memo.ui_state.selected,
        Some(first),
        "the selection moves on"
    );
    memo.apply(&WindowAction::Undo, Language::English);
    assert!(memo.drawer_open, "undoing keeps it open too (M164)");
    assert_eq!(memo.ui_state.selected, Some(first));
}

#[test]
fn deleting_from_the_note_view_leaves_the_drawer_closed() {
    let document = sample_document();
    let (mut memo, _worker, _directory) = memo_window(document.clone(), selected_first(&document));
    let first = first_note(&document).unwrap_or_else(|| panic!("the sample has a note"));
    memo.apply(&WindowAction::Delete(first), Language::English);
    assert!(!memo.drawer_open, "the note menu does not open the list");
}
