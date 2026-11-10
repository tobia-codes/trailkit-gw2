//! Tests of save errors in the window (memo M180): the editor's "Save" that
//! waits for the disk (Q158), the dialog when leaving after a failed save
//! (Q157) and dropping a failed save when the text changes (Q153).

use std::fs;

use trailkit_memo_shared::{save_text::SaveText, storage_worker::SaveHealth};

use super::*;

/// How long a test waits for the storage worker.
const WAIT: Duration = Duration::from_secs(5);

/// Reads the worker's reports until `done` holds; panics after [`WAIT`].
fn wait_until(memo: &mut MemoWindow, what: &str, done: impl Fn(&MemoWindow) -> bool) {
    let deadline = std::time::Instant::now() + WAIT;
    while !done(memo) {
        assert!(
            std::time::Instant::now() < deadline,
            "timed out waiting for {what}"
        );
        std::thread::sleep(Duration::from_millis(5));
        memo.poll_save_reports(Language::English);
    }
}

/// A folder where the temporary notes file must go makes every notes save
/// fail; returns its path so the test can remove it again.
fn block_notes(directory: &tempfile::TempDir) -> std::path::PathBuf {
    let blocker = directory.path().join("notes.json.tmp");
    fs::create_dir(&blocker).unwrap_or_else(|error| panic!("blocker: {error}"));
    blocker
}

/// The title of the first root note on disk.
fn saved_first_title(directory: &tempfile::TempDir) -> Option<String> {
    let document = DocumentStore::new(directory.path())
        .load::<NotesDocument>(Timestamp::now())
        .into_document()?;
    document.root.iter().find_map(|node| match node {
        Node::Note(note) => Some(note.title.clone()),
        Node::Group(_) => None,
    })
}

/// The title of the first root note in the window.
fn first_title(memo: &MemoWindow) -> Option<&str> {
    memo.document.root.iter().find_map(|node| match node {
        Node::Note(note) => Some(note.title.as_str()),
        Node::Group(_) => None,
    })
}

/// Starts editing the first note and appends to its title.
fn edit_first_title(memo: &mut MemoWindow) {
    memo.apply(&WindowAction::StartEditing, Language::English);
    let Some(editor) = memo.editor.as_mut() else {
        panic!("editing did not start");
    };
    editor.title_mut().push_str(" (changed)");
}

#[test]
fn save_waits_for_the_disk_before_the_editor_closes() {
    let document = sample_document();
    let (mut memo, _worker, directory) = memo_window(document.clone(), selected_first(&document));
    let before = first_title(&memo).map(str::to_owned);
    edit_first_title(&mut memo);
    memo.apply(&WindowAction::FinishEditing, Language::English);
    assert!(memo.editor.is_some(), "the editor waits for the result");
    assert!(memo.editor_commit_running());
    assert_eq!(
        first_title(&memo).map(str::to_owned),
        before,
        "not yet in the tree"
    );

    wait_until(&mut memo, "the save", |memo| memo.editor.is_none());
    let expected = before.map(|title| format!("{title} (changed)"));
    assert_eq!(first_title(&memo).map(str::to_owned), expected);
    assert_eq!(saved_first_title(&directory), expected, "on disk");
    assert!(
        memo.toast.is_none(),
        "no toast without a shown error (Q160)"
    );
}

#[test]
fn a_failed_save_keeps_the_editor_and_asks_when_leaving() {
    let document = sample_document();
    let (mut memo, _worker, directory) = memo_window(document.clone(), selected_first(&document));
    let before = first_title(&memo).map(str::to_owned);
    block_notes(&directory);
    edit_first_title(&mut memo);
    memo.apply(&WindowAction::FinishEditing, Language::English);
    wait_until(&mut memo, "the failure", MemoWindow::editor_commit_failed);
    assert!(memo.editor.is_some(), "the editor stays open (Q158)");
    assert!(
        memo.save_status.error_shown,
        "a failed Save shows the toast at once"
    );

    assert!(!memo.request_close(), "leaving asks first");
    assert_eq!(
        memo.save_failed_prompt
            .as_ref()
            .map(|prompt| &prompt.reason),
        Some(&LeaveReason::CloseWindow)
    );
    memo.apply(&WindowAction::SaveFailedDiscard, Language::English);
    assert!(memo.editor.is_none());
    assert!(memo.take_close_approval(), "discarding continues closing");
    assert_eq!(
        first_title(&memo).map(str::to_owned),
        before,
        "nothing to roll back"
    );
}

#[test]
fn save_again_in_the_dialog_leaves_the_editor_once_it_works() {
    let document = sample_document();
    let (mut memo, _worker, directory) = memo_window(document.clone(), selected_first(&document));
    let blocker = block_notes(&directory);
    edit_first_title(&mut memo);
    memo.apply(&WindowAction::FinishEditing, Language::English);
    wait_until(&mut memo, "the failure", MemoWindow::editor_commit_failed);
    memo.apply(&WindowAction::CancelEditing, Language::English);
    assert!(memo.save_failed_prompt.is_some(), "Cancel asks (Q157)");

    memo.apply(&WindowAction::SaveFailedRetry, Language::English);
    wait_until(&mut memo, "the second failure", |memo| {
        memo.save_failed_prompt
            .as_ref()
            .is_some_and(|prompt| prompt.failed_again)
    });
    assert!(
        memo.editor_commit.is_none(),
        "no automatic writes after it (Q157)"
    );

    fs::remove_dir(&blocker).unwrap_or_else(|error| panic!("blocker: {error}"));
    memo.apply(&WindowAction::SaveFailedRetry, Language::English);
    wait_until(&mut memo, "the save", |memo| memo.editor.is_none());
    assert!(memo.save_failed_prompt.is_none());
    assert!(
        saved_first_title(&directory).is_some_and(|title| title.ends_with(" (changed)")),
        "on disk"
    );
    wait_until(&mut memo, "the success toast", |memo| memo.toast.is_some());
    assert_eq!(
        memo.toast.as_ref().map(|toast| toast.toast.message()),
        Some(SaveText::ChangesSaved.get(Language::English))
    );
}

#[test]
fn changing_the_text_drops_a_failed_save() {
    let document = sample_document();
    let (mut memo, _worker, directory) = memo_window(document.clone(), selected_first(&document));
    block_notes(&directory);
    edit_first_title(&mut memo);
    memo.apply(&WindowAction::FinishEditing, Language::English);
    wait_until(&mut memo, "the failure", MemoWindow::editor_commit_failed);
    memo.drop_failed_commit_on_edit();
    assert!(memo.editor_commit_failed(), "unchanged text keeps it");

    if let Some(editor) = memo.editor.as_mut() {
        editor.title_mut().push('!');
    }
    memo.drop_failed_commit_on_edit();
    assert!(memo.editor_commit.is_none(), "the edit drops it (Q153)");
}

#[test]
fn the_toast_waits_for_the_second_failure_and_returns_after_closing() {
    let document = sample_document();
    let (mut memo, _worker, _directory) = memo_window(document.clone(), selected_first(&document));
    let report = |memo: &mut MemoWindow, health| memo.apply_save_health(health, Language::English);
    report(&mut memo, SaveHealth::Retrying { failed_attempts: 1 });
    assert!(
        !memo.save_status.error_shown,
        "the first failure is silent (Q149)"
    );
    report(&mut memo, SaveHealth::Retrying { failed_attempts: 2 });
    assert!(memo.save_status.error_shown);

    memo.apply(&WindowAction::CloseSaveToast, Language::English);
    report(&mut memo, SaveHealth::GaveUp);
    assert!(
        memo.save_status.closed,
        "closing hides it while the save still fails"
    );
    report(&mut memo, SaveHealth::Retrying { failed_attempts: 0 });
    report(&mut memo, SaveHealth::Retrying { failed_attempts: 1 });
    assert!(
        !memo.save_status.closed,
        "the next failed change brings it back (Q164)"
    );

    report(&mut memo, SaveHealth::Healthy);
    assert!(!memo.save_status.error_shown);
    assert_eq!(
        memo.toast.as_ref().map(|toast| toast.toast.tone()),
        Some(trailkit_ui::toast::ToastTone::Success)
    );
}
