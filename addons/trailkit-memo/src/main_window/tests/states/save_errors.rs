//! Snapshots of save errors (memo M180, mockup V4): the error toast over
//! the note and the editor, the editor's "Saving…" and the dialog when
//! leaving after a failed save, also in German at the narrowest window.

use trailkit_memo_shared::storage_worker::SaveHealth;

use super::*;
use crate::main_window::save_status::SaveFailedPrompt;

/// Marks the save as failing with `health` and the error toast as shown.
fn failing(mut fixture: Fixture, health: SaveHealth) -> Fixture {
    fixture.0.save_status.health = health;
    fixture.0.save_status.error_shown = true;
    fixture
}

/// The error toast with "Try again" over the note.
fn toast_gave_up() -> Fixture {
    failing(selected(), SaveHealth::GaveUp)
}

/// The error toast during the automatic writes over the editor (mockup 3c).
fn toast_retrying_in_editor() -> Fixture {
    failing(editing(), SaveHealth::Retrying { failed_attempts: 2 })
}

/// The editor while its "Save" waits for the disk (Q158).
fn editor_saving() -> Fixture {
    let mut fixture = editing();
    fixture.0.show_editor_commit_running();
    fixture
}

/// The editor with changed text and the save-failed dialog (Q157).
fn dialog(saving: bool, failed_again: bool) -> Fixture {
    let mut fixture = failing(editing(), SaveHealth::GaveUp);
    if let Some(editor) = fixture.0.editor.as_mut() {
        editor.title_mut().push_str(" (changed)");
    }
    fixture.0.save_failed_prompt = Some(SaveFailedPrompt {
        reason: LeaveReason::CloseWindow,
        saving,
        failed_again,
    });
    fixture
}

#[test]
fn save_errors() {
    let states: [NamedFixture; 6] = [
        ("save-toast", toast_gave_up),
        ("save-toast-editing", toast_retrying_in_editor),
        ("editor-saving", editor_saving),
        ("dialog-save-failed", || dialog(false, false)),
        ("dialog-save-failed-saving", || dialog(true, false)),
        ("dialog-save-failed-again", || dialog(false, true)),
    ];
    for (name, make) in states {
        assert_state(name, WIDE, make, &[]);
    }
}

#[test]
fn save_errors_in_german_at_the_minimum_width() {
    let states: [NamedFixture; 2] = [
        ("save-toast", toast_gave_up),
        ("dialog-save-failed", || dialog(false, true)),
    ];
    for (name, make) in states {
        assert_state_in(
            Language::German,
            &format!("min-de-{name}"),
            MINIMUM_WINDOW,
            make,
            &[],
        );
    }
}
