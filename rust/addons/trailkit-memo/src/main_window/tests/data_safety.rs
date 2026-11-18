//! Tests of the paths where notes could get lost: emptying the trash, drafts,
//! the answers of the unsaved changes dialog and a stopped storage worker
//! (memo M54, M55, M146, M180, Q151). What must survive is also checked on
//! disk: dropping the worker writes everything pending.

use trailkit_memo_shared::{drafts::Draft, settings::UnsavedChanges};
use trailkit_memo_trash::trash_view::TrashAction;
use trailkit_storage::VersionedDocument;

use super::{
    save_errors::{block_notes, edit_first_title, first_title, saved_first_title, wait_until},
    trash::document_with_trash,
    *,
};

/// The window size of the tests that click.
const SIZE: [f32; 2] = [640.0, 360.0];
/// "Discard" on the draft card of the first note, with the overview card above.
const DRAFT_DISCARD_BUTTON: [f32; 2] = [259.0, 199.0];
/// "Restore" on the draft card of the first note.
const DRAFT_RESTORE_BUTTON: [f32; 2] = [331.0, 199.0];

/// English, the language of these tests.
const ENGLISH: Language = Language::English;

/// The document of type `T` in `directory`, if one can be loaded.
fn saved<T: VersionedDocument>(directory: &tempfile::TempDir) -> Option<T> {
    DocumentStore::new(directory.path())
        .load::<T>(Timestamp::now())
        .into_document()
}

/// A draft of `note`.
fn draft_of(note: NoteId) -> Draft {
    Draft {
        note,
        title: "Draft title".to_owned(),
        body: "Draft body".to_owned(),
        saved_at: fixed_time(),
        note_updated_at: fixed_time(),
    }
}

/// The ids of the sample's two root notes.
fn root_notes(document: &NotesDocument) -> (NoteId, NoteId) {
    let mut notes = document.root.iter().filter_map(|node| match node {
        Node::Note(note) => Some(note.id),
        Node::Group(_) => None,
    });
    match (notes.next(), notes.next()) {
        (Some(first), Some(second)) => (first, second),
        _ => panic!("the sample has two root notes"),
    }
}

/// A window with the first note selected.
fn window() -> (MemoWindow, StorageWorker, tempfile::TempDir, NoteId, NoteId) {
    let document = sample_document();
    let (first, second) = root_notes(&document);
    let ui_state = selected_first(&document);
    let (memo, worker, directory) = memo_window(document, ui_state);
    (memo, worker, directory, first, second)
}

/// The note `id` of a trash entry.
fn trashed_note(document: &NotesDocument) -> NoteId {
    match &document.trash[0].node {
        Node::Note(note) => note.id,
        Node::Group(_) => panic!("the first trash entry is a note"),
    }
}

#[test]
fn emptying_the_trash_asks_and_then_removes_everything_for_good() {
    let document = document_with_trash([2, 3]);
    let old = document.trash[0].id();
    let in_trash = trashed_note(&document);
    let root_len = document.root.len();
    let (mut memo, worker, directory) = memo_window(document, UiState::default());
    memo.drafts.drafts.push(draft_of(in_trash));
    memo.apply(&WindowAction::OpenTrash, ENGLISH);
    memo.apply(&WindowAction::Trash(TrashAction::Select(old)), ENGLISH);

    memo.apply(&WindowAction::Trash(TrashAction::EmptyTrash), ENGLISH);
    assert!(memo.confirmation.is_some(), "it asks first (M146)");
    assert_eq!(memo.document.trash.len(), 2, "nothing is gone yet");
    memo.apply(&WindowAction::ConfirmCancelled, ENGLISH);
    assert!(memo.confirmation.is_none());
    assert_eq!(memo.document.trash.len(), 2, "Cancel keeps everything");
    assert!(memo.drafts.get(in_trash).is_some());

    memo.apply(&WindowAction::Trash(TrashAction::EmptyTrash), ENGLISH);
    memo.apply(&WindowAction::ConfirmAccepted, ENGLISH);
    assert!(memo.document.trash.is_empty());
    assert_eq!(memo.trash.selected, None);
    assert!(memo.drafts.get(in_trash).is_none(), "its drafts go with it");
    assert_eq!(memo.document.root.len(), root_len, "the notes stay");

    drop(worker);
    let notes = saved::<NotesDocument>(&directory).unwrap_or_else(|| panic!("notes on disk"));
    assert!(notes.trash.is_empty(), "also on disk");
    assert_eq!(notes.root.len(), root_len);
    let drafts = saved::<DraftsDocument>(&directory).unwrap_or_default();
    assert!(drafts.get(in_trash).is_none());
}

#[test]
fn accepting_without_a_question_deletes_nothing() {
    let document = document_with_trash([2, 3]);
    let (mut memo, _worker, _directory) = memo_window(document, UiState::default());
    memo.apply(&WindowAction::ConfirmAccepted, ENGLISH);
    assert_eq!(memo.document.trash.len(), 2);
}

#[test]
fn deleting_for_good_selects_the_neighbour_in_the_trash() {
    let document = document_with_trash([2, 3]);
    let (old, fractals) = (document.trash[0].id(), document.trash[1].id());
    let delete = |memo: &mut MemoWindow, node| {
        memo.apply(&WindowAction::Trash(TrashAction::Select(node)), ENGLISH);
        memo.apply(
            &WindowAction::Trash(TrashAction::DeletePermanently(node)),
            ENGLISH,
        );
        memo.apply(&WindowAction::ConfirmAccepted, ENGLISH);
    };
    // The list shows the newest entry first: "Fractals", then the old note.
    let (mut memo, _worker, _directory) = memo_window(document.clone(), UiState::default());
    memo.apply(&WindowAction::OpenTrash, ENGLISH);
    delete(&mut memo, fractals);
    assert_eq!(memo.trash.selected, Some(old), "the next row");
    delete(&mut memo, old);
    assert_eq!(memo.trash.selected, None, "nothing is left");

    let (mut memo, _worker, _directory) = memo_window(document, UiState::default());
    memo.apply(&WindowAction::OpenTrash, ENGLISH);
    delete(&mut memo, old);
    assert_eq!(
        memo.trash.selected,
        Some(fractals),
        "the last row: the one before"
    );
}

#[test]
fn restoring_a_draft_opens_it_in_the_editor_and_uses_it_up() {
    let (mut memo, worker, directory, first, _) = window();
    let original = first_title(&memo).map(str::to_owned);
    memo.drafts.drafts.push(draft_of(first));
    // As after a restart: the worker holds the draft.
    memo.saver.save_drafts(memo.drafts.clone());
    memo.apply(&WindowAction::RestoreDraft(first), ENGLISH);
    let Some(editor) = memo.editor.as_ref() else {
        panic!("the draft opens in the editor");
    };
    assert_eq!(editor.session().title, "Draft title");
    assert_eq!(editor.body_text(), "Draft body");
    assert!(memo.has_unsaved_changes(), "it still has to be saved");
    assert!(memo.drafts.get(first).is_none());
    assert_eq!(
        first_title(&memo).map(str::to_owned),
        original,
        "the note itself is unchanged until saving"
    );
    drop(worker);
    let drafts = saved::<DraftsDocument>(&directory).unwrap_or_default();
    assert!(drafts.get(first).is_none(), "also on disk");
}

#[test]
fn a_draft_is_not_restored_over_an_open_editor() {
    let (mut memo, _worker, _directory, first, second) = window();
    memo.drafts.drafts.push(draft_of(second));
    edit_first_title(&mut memo);
    memo.apply(&WindowAction::RestoreDraft(second), ENGLISH);
    assert_eq!(
        memo.editor.as_ref().map(|editor| editor.session().note),
        Some(first),
        "the open edit stays"
    );
    assert!(memo.drafts.get(second).is_some(), "the draft is kept");
}

#[test]
fn discarding_a_draft_removes_only_that_draft() {
    let (mut memo, worker, directory, first, second) = window();
    memo.drafts.drafts.push(draft_of(first));
    memo.drafts.drafts.push(draft_of(second));
    memo.apply(&WindowAction::DiscardDraft(first), ENGLISH);
    assert!(memo.drafts.get(first).is_none());
    assert!(memo.drafts.get(second).is_some());
    assert!(memo.editor.is_none(), "discarding opens nothing");
    drop(worker);
    let drafts = saved::<DraftsDocument>(&directory).unwrap_or_default();
    assert!(drafts.get(first).is_none(), "also on disk");
    assert!(drafts.get(second).is_some());
}

#[test]
fn saving_the_edit_uses_up_the_notes_draft() {
    let (mut memo, worker, directory, first, _) = window();
    memo.drafts.drafts.push(draft_of(first));
    memo.apply(&WindowAction::RestoreDraft(first), ENGLISH);
    memo.drafts.drafts.push(draft_of(first));
    memo.apply(&WindowAction::FinishEditing, ENGLISH);
    wait_until(&mut memo, "the save", |memo| memo.editor.is_none());
    assert!(memo.drafts.get(first).is_none());
    assert_eq!(first_title(&memo), Some("Draft title"));
    drop(worker);
    assert_eq!(
        saved_first_title(&directory).as_deref(),
        Some("Draft title")
    );
}

/// Clicks at `position` in the window drawn by `harness`.
fn click(harness: &mut UiHarness, memo: &mut MemoWindow, position: [f32; 2]) {
    let input = |left_button_down| FrameInput {
        mouse_position: position,
        left_button_down,
        ..FrameInput::default()
    };
    let mut frame = |harness: &mut UiHarness, input: &FrameInput| {
        harness.step(input, |ui| draw(ui, memo, SIZE, ThemeKind::Dark));
    };
    for _ in 0..3 {
        frame(harness, &input(false));
    }
    frame(harness, &input(true));
    for _ in 0..2 {
        frame(harness, &input(false));
    }
}

#[test]
fn the_buttons_of_the_draft_card_restore_and_discard() {
    let (mut memo, _worker, _directory, first, _) = window();
    memo.drafts.drafts.push(draft_of(first));
    let mut harness = UiHarness::new(SIZE);
    click(&mut harness, &mut memo, DRAFT_DISCARD_BUTTON);
    assert!(memo.drafts.get(first).is_none(), "\"Discard\"");
    assert!(memo.editor.is_none());

    memo.drafts.drafts.push(draft_of(first));
    click(&mut harness, &mut memo, DRAFT_RESTORE_BUTTON);
    assert!(memo.editor.is_some(), "\"Restore\"");
    assert!(memo.drafts.get(first).is_none());
}

#[test]
fn unloading_keeps_unsaved_changes_as_a_draft() {
    let (mut memo, worker, directory, first, _) = window();
    assert!(!memo.keep_draft_for_unload(), "no editor, nothing to keep");
    memo.apply(&WindowAction::StartEditing, ENGLISH);
    assert!(
        !memo.keep_draft_for_unload(),
        "an unchanged edit is no draft"
    );
    assert!(memo.drafts.get(first).is_none());
    let updated_at = memo.document.note(first).map(|note| note.updated_at);
    if let Some(editor) = memo.editor.as_mut() {
        editor.title_mut().push_str(" (changed)");
    }
    assert!(memo.keep_draft_for_unload());
    assert!(memo.keep_draft_for_unload(), "again replaces the draft");
    assert_eq!(memo.drafts.drafts.len(), 1);
    let kept = memo.drafts.get(first).cloned();
    assert!(
        kept.as_ref()
            .is_some_and(|draft| draft.title.ends_with(" (changed)"))
    );
    assert_eq!(kept.map(|draft| draft.note_updated_at), updated_at);
    drop(worker);
    let drafts = saved::<DraftsDocument>(&directory).unwrap_or_default();
    assert!(
        drafts
            .get(first)
            .is_some_and(|draft| draft.title.ends_with(" (changed)")),
        "the worker wrote it"
    );
}

#[test]
fn a_draft_is_not_kept_for_a_note_that_is_gone() {
    let (mut memo, _worker, _directory, first, _) = window();
    edit_first_title(&mut memo);
    memo.document.root.remove(0);
    assert!(!memo.keep_draft_for_unload());
    assert!(memo.drafts.get(first).is_none());
}

/// A window editing the first note with a change, asked to go to the second
/// note: the unsaved changes dialog is open.
fn asked_when_leaving() -> (MemoWindow, StorageWorker, tempfile::TempDir, NoteId) {
    let (mut memo, worker, directory, _, second) = window();
    edit_first_title(&mut memo);
    memo.apply(&WindowAction::Select(NodeId::Note(second)), ENGLISH);
    assert!(memo.leave_prompt.is_some(), "leaving asks first (M54)");
    assert!(memo.editor.is_some());
    (memo, worker, directory, second)
}

#[test]
fn discarding_when_leaving_drops_the_text_and_continues() {
    let (mut memo, worker, directory, second) = asked_when_leaving();
    let before = first_title(&memo).map(str::to_owned);
    memo.apply(&WindowAction::LeaveDiscarding, ENGLISH);
    assert!(memo.editor.is_none());
    assert!(memo.leave_prompt.is_none());
    assert_eq!(memo.ui_state.selected, Some(NodeId::Note(second)));
    assert_eq!(first_title(&memo).map(str::to_owned), before);
    drop(worker);
    let on_disk = saved_first_title(&directory);
    assert!(
        on_disk.is_none_or(|title| !title.ends_with(" (changed)")),
        "the discarded text never reaches the disk"
    );
}

#[test]
fn saving_when_leaving_writes_the_text_and_then_continues() {
    let (mut memo, worker, directory, second) = asked_when_leaving();
    memo.apply(&WindowAction::LeaveSaving, ENGLISH);
    assert!(memo.editor.is_some(), "it waits for the disk (M180)");
    assert_ne!(memo.ui_state.selected, Some(NodeId::Note(second)));
    wait_until(&mut memo, "the save", |memo| memo.editor.is_none());
    assert_eq!(memo.ui_state.selected, Some(NodeId::Note(second)));
    assert!(first_title(&memo).is_some_and(|title| title.ends_with(" (changed)")));
    drop(worker);
    assert!(saved_first_title(&directory).is_some_and(|title| title.ends_with(" (changed)")));
}

#[test]
fn a_failed_save_when_leaving_keeps_the_text_and_stays() {
    let (mut memo, _worker, directory, second) = asked_when_leaving();
    block_notes(&directory);
    memo.apply(&WindowAction::LeaveSaving, ENGLISH);
    wait_until(&mut memo, "the failure", |memo| {
        memo.save_failed_prompt.is_some()
    });
    assert_ne!(memo.ui_state.selected, Some(NodeId::Note(second)));
    assert!(
        memo.editor
            .as_ref()
            .is_some_and(|editor| editor.session().title.ends_with(" (changed)")),
        "the text is still in the editor (Q157)"
    );
    memo.apply(&WindowAction::SaveFailedDiscard, ENGLISH);
    assert!(memo.editor.is_none(), "only the explicit answer drops it");
    assert_eq!(memo.ui_state.selected, Some(NodeId::Note(second)));
}

#[test]
fn keep_editing_answers_leave_everything_as_it_was() {
    let (mut memo, _worker, _directory, second) = asked_when_leaving();
    memo.apply(&WindowAction::LeaveCancelled, ENGLISH);
    assert!(memo.leave_prompt.is_none());
    assert!(memo.has_unsaved_changes());
    assert_ne!(memo.ui_state.selected, Some(NodeId::Note(second)));

    assert!(!memo.request_close(), "closing asks");
    assert!(!memo.request_close(), "Escape answers \"keep editing\"");
    assert!(memo.leave_prompt.is_none());
    assert!(memo.has_unsaved_changes());
    assert!(!memo.take_close_approval());
}

#[test]
fn the_answers_when_closing_approve_the_close() {
    let (mut memo, _worker, _directory, ..) = window();
    edit_first_title(&mut memo);
    assert!(!memo.request_close());
    memo.apply(&WindowAction::LeaveDiscarding, ENGLISH);
    assert!(memo.take_close_approval(), "discarded: it may close");
    assert!(!memo.take_close_approval(), "reading clears it");
    assert!(memo.editor.is_none());

    edit_first_title(&mut memo);
    assert!(!memo.request_close());
    memo.apply(&WindowAction::LeaveSaving, ENGLISH);
    assert!(
        !memo.take_close_approval(),
        "not before the text is on disk"
    );
    wait_until(&mut memo, "the save", |memo| memo.close_approved);
    assert!(memo.take_close_approval());
    assert!(first_title(&memo).is_some_and(|title| title.ends_with(" (changed)")));
}

#[test]
fn cancel_asks_with_changes_and_closes_without() {
    let (mut memo, _worker, _directory, ..) = window();
    memo.apply(&WindowAction::StartEditing, ENGLISH);
    memo.apply(&WindowAction::CancelEditing, ENGLISH);
    assert!(memo.editor.is_none(), "nothing to lose");

    let before = first_title(&memo).map(str::to_owned);
    edit_first_title(&mut memo);
    memo.apply(&WindowAction::CancelEditing, ENGLISH);
    assert_eq!(memo.leave_prompt, Some(LeaveReason::Cancel));
    memo.apply(&WindowAction::LeaveDiscarding, ENGLISH);
    assert!(memo.editor.is_none());
    assert_eq!(first_title(&memo).map(str::to_owned), before);
}

#[test]
fn automatic_saving_writes_before_leaving_and_closing() {
    let (mut memo, _worker, _directory, _, second) = window();
    memo.settings.unsaved_changes = UnsavedChanges::SaveAutomatically;
    edit_first_title(&mut memo);
    memo.apply(&WindowAction::Select(NodeId::Note(second)), ENGLISH);
    assert!(memo.leave_prompt.is_none(), "it does not ask");
    wait_until(&mut memo, "the save", |memo| memo.editor.is_none());
    assert_eq!(memo.ui_state.selected, Some(NodeId::Note(second)));
    assert!(first_title(&memo).is_some_and(|title| title.ends_with(" (changed)")));

    memo.apply(&WindowAction::StartEditing, ENGLISH);
    if let Some(editor) = memo.editor.as_mut() {
        editor.title_mut().push_str(" again");
    }
    assert!(!memo.request_close(), "closing waits for the disk");
    wait_until(&mut memo, "the save", |memo| memo.close_approved);
    assert!(memo.editor.is_none());
}

#[test]
fn cancel_discards_without_asking_under_automatic_saving() {
    let (mut memo, _worker, _directory, ..) = window();
    memo.settings.unsaved_changes = UnsavedChanges::SaveAutomatically;
    let before = first_title(&memo).map(str::to_owned);
    edit_first_title(&mut memo);
    memo.apply(&WindowAction::CancelEditing, ENGLISH);
    assert!(memo.leave_prompt.is_none());
    assert!(memo.editor.is_none());
    assert_eq!(first_title(&memo).map(str::to_owned), before);
}

#[test]
fn a_stopped_worker_is_reported_and_try_again_restarts_it() {
    let (mut memo, worker, directory, first, _) = window();
    memo.drafts.drafts.push(draft_of(first));
    drop(worker);
    memo.poll_save_reports(ENGLISH);
    assert!(
        memo.save_status.error_shown,
        "the error toast is due (Q151)"
    );
    assert!(!memo.take_worker_restart_request(), "nobody asked yet");

    memo.apply(&WindowAction::RetrySave, ENGLISH);
    assert!(memo.take_worker_restart_request());
    assert!(!memo.take_worker_restart_request(), "reading clears it");
    memo.storage_restart_failed();
    wait_until(&mut memo, "the busy time", |memo| {
        !memo.save_status.toast_write_running()
    });
    memo.apply(&WindowAction::RetrySave, ENGLISH);
    assert!(memo.take_worker_restart_request(), "it can be tried again");

    if let Some(note) = memo.document.note_mut(first) {
        note.title = "Written after the restart".to_owned();
    }
    let (restarted, connection) = storage_worker_in(directory.path());
    memo.reconnect_storage(connection);
    wait_until(&mut memo, "the write", |memo| {
        memo.save_status.waiting.is_empty()
    });
    memo.apply(&WindowAction::RetrySave, ENGLISH);
    assert!(
        !memo.take_worker_restart_request(),
        "the new worker runs: no restart"
    );
    drop(restarted);
    assert_eq!(
        saved_first_title(&directory).as_deref(),
        Some("Written after the restart"),
        "the window's documents reach the disk"
    );
    let drafts = saved::<DraftsDocument>(&directory).unwrap_or_default();
    assert!(drafts.get(first).is_some(), "the drafts too");
}

#[test]
fn a_save_running_when_the_worker_stops_fails_and_keeps_the_text() {
    let (mut memo, worker, _directory, ..) = window();
    drop(worker);
    edit_first_title(&mut memo);
    memo.apply(&WindowAction::FinishEditing, ENGLISH);
    memo.poll_save_reports(ENGLISH);
    assert!(memo.editor_commit_failed());
    assert!(memo.has_unsaved_changes(), "the text stays in the editor");
}

#[test]
fn saving_after_the_worker_stopped_fails_instead_of_waiting_forever() {
    let (mut memo, worker, _directory, ..) = window();
    drop(worker);
    memo.poll_save_reports(ENGLISH);
    edit_first_title(&mut memo);
    memo.apply(&WindowAction::FinishEditing, ENGLISH);
    assert!(memo.editor_commit_failed(), "no answer will ever come");
    assert!(!memo.editor_commit_running());
    assert!(memo.save_status.waiting.is_empty(), "nothing waits for it");
    assert!(memo.editor.is_some(), "the text stays in the editor");
}

#[test]
fn leaving_after_the_worker_stopped_asks_with_the_save_failed_dialog() {
    let (mut memo, worker, _directory, _, second) = window();
    drop(worker);
    memo.poll_save_reports(ENGLISH);
    edit_first_title(&mut memo);
    memo.apply(&WindowAction::Select(NodeId::Note(second)), ENGLISH);
    memo.apply(&WindowAction::LeaveSaving, ENGLISH);
    assert!(memo.save_failed_prompt.is_some());
    assert!(memo.has_unsaved_changes(), "the text stays in the editor");
    assert_ne!(memo.ui_state.selected, Some(NodeId::Note(second)));
}
