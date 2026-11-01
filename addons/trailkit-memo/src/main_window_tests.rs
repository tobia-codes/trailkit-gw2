//! Snapshot tests of the memo main window (memo plan M65), run with
//! `cargo xtask ui-test` (`--update` rewrites `tests/snapshots/`).

use std::time::Duration;

use nexus::imgui::{Condition, Ui, Window};
use trailkit_storage::DocumentStore;
use trailkit_ui::{style_scope::StyleScope, theme::ThemeKind};
use trailkit_ui_testing::{
    harness::{FrameInput, UiHarness},
    snapshot::assert_snapshot,
};

use super::*;
use crate::storage_worker::{SavingAllowed, StorageWorker};

/// A fixed point in time, so rendered dates never change.
fn fixed_time() -> Timestamp {
    Timestamp::from_second(1_793_520_000).unwrap_or(Timestamp::UNIX_EPOCH)
}

/// A note with `title`, `body` and `kind`.
fn note(title: &str, body: &str, kind: NoteKind) -> Note {
    let mut note = Note::new(title.to_owned(), kind, fixed_time());
    note.body = body.to_owned();
    note
}

/// A small tree: two root notes (one daily), a group with a weekly note.
fn sample_document() -> NotesDocument {
    let mut raids = Group::new("Raids".to_owned());
    raids.children.push(Node::Note(note(
        "Weekly clears",
        "- [x] Spirit Vale\n- [ ] Salvation Pass",
        NoteKind::WeeklyReset {
            last_reset_applied_at: fixed_time(),
        },
    )));
    NotesDocument {
        root: vec![
            Node::Note(note(
                "Shopping list",
                "## Ectos\n- [x] **250** Globs of Ectoplasm\n- [ ] Mystic Coins\nCheck the [wiki](https://wiki.guildwars2.com) first.",
                NoteKind::Normal,
            )),
            Node::Note(note(
                "Dailies",
                "- [ ] Wizard's Vault",
                NoteKind::DailyReset {
                    last_reset_applied_at: fixed_time(),
                },
            )),
            Node::Group(raids),
        ],
        trash: Vec::new(),
    }
}

/// A memo window for `document` with `ui_state`, backed by a storage worker
/// in a temporary directory (returned so it outlives the window).
fn memo_window(
    document: NotesDocument,
    ui_state: UiState,
) -> (MemoWindow, StorageWorker, tempfile::TempDir) {
    let directory = tempfile::tempdir().unwrap_or_else(|error| panic!("temp dir: {error}"));
    let (worker, _loaded) = StorageWorker::spawn(
        DocumentStore::new(directory.path()),
        Duration::from_secs(3600),
        SavingAllowed { settings: false },
    )
    .unwrap_or_else(|error| panic!("storage worker: {error}"));
    let window = MemoWindow::new(
        document,
        None,
        ui_state,
        worker.saver(),
        directory.path().to_path_buf(),
        DraftsDocument::default(),
        UnsavedChanges::Ask,
    );
    (window, worker, directory)
}

/// Draws `memo` like the addon does: theme, window, then its dialog.
fn draw(ui: &Ui, memo: &mut MemoWindow, size: [f32; 2], theme: ThemeKind) {
    let theme = theme.theme();
    let _style = StyleScope::push(ui, &theme);
    Window::new("Notes###memo")
        .position([0.0, 0.0], Condition::Always)
        .size(size, Condition::Always)
        .build(ui, || memo.draw(ui, Language::English, &theme));
    memo.draw_dialog(ui, Language::English, &theme);
}

/// Renders `memo` for a few frames (layouts settle) and compares it with the
/// reference `name`.
fn assert_window(memo: &mut MemoWindow, size: [f32; 2], theme: ThemeKind, name: &str) {
    let mut harness = UiHarness::new(size);
    let image = harness.settle(4, &FrameInput::default(), |ui| {
        draw(ui, memo, size, theme);
    });
    assert_snapshot(env!("CARGO_MANIFEST_DIR"), name, &image);
}

/// The id of the first root note of `document`.
fn first_note(document: &NotesDocument) -> Option<NodeId> {
    document.root.iter().find_map(|node| match node {
        Node::Note(note) => Some(NodeId::Note(note.id)),
        Node::Group(_) => None,
    })
}

/// Window state with the first note of `document` selected.
fn selected_first(document: &NotesDocument) -> UiState {
    UiState {
        selected: first_note(document),
        ..UiState::default()
    }
}

#[test]
fn wide_window_in_both_themes() {
    let document = sample_document();
    let ui_state = selected_first(&document);
    for (theme, name) in [
        (ThemeKind::Dark, "memo-wide-dark"),
        (ThemeKind::Light, "memo-wide-light"),
    ] {
        let (mut memo, _worker, _directory) = memo_window(document.clone(), ui_state);
        assert_window(&mut memo, [640.0, 360.0], theme, name);
    }
}

#[test]
fn narrow_window_shows_the_drawer_without_a_selected_note() {
    let (mut memo, _worker, _directory) = memo_window(sample_document(), UiState::default());
    assert_window(
        &mut memo,
        [360.0, 360.0],
        ThemeKind::Dark,
        "memo-narrow-drawer",
    );
}

#[test]
fn empty_state() {
    let (mut memo, _worker, _directory) = memo_window(NotesDocument::default(), UiState::default());
    assert_window(&mut memo, [480.0, 300.0], ThemeKind::Dark, "memo-empty");
}

#[test]
fn editor_with_preview() {
    let document = sample_document();
    let ui_state = UiState {
        preview_visible: true,
        ..selected_first(&document)
    };
    let (mut memo, _worker, _directory) = memo_window(document, ui_state);
    memo.apply(&WindowAction::StartEditing, Language::English);
    if let Some(editor) = memo.editor.as_mut() {
        editor.body.unfocus();
    }
    assert_window(
        &mut memo,
        [720.0, 360.0],
        ThemeKind::Dark,
        "memo-editor-preview",
    );
}

#[test]
fn narrow_editor_keeps_editor_and_preview_side_by_side() {
    let document = sample_document();
    let ui_state = UiState {
        preview_visible: true,
        note_list_hidden: true,
        ..selected_first(&document)
    };
    let (mut memo, _worker, _directory) = memo_window(document, ui_state);
    memo.apply(&WindowAction::StartEditing, Language::English);
    if let Some(editor) = memo.editor.as_mut() {
        editor.body.unfocus();
    }
    assert_window(
        &mut memo,
        [280.0, 360.0],
        ThemeKind::Dark,
        "memo-editor-preview-narrow",
    );
}

#[test]
fn toolbar_highlights_the_formatting_at_the_cursor() {
    let document = sample_document();
    let ui_state = selected_first(&document);
    let (mut memo, _worker, _directory) = memo_window(document, ui_state);
    memo.apply(&WindowAction::StartEditing, Language::English);
    let Some(editor) = memo.editor.as_mut() else {
        panic!("editing did not start");
    };
    // Cursor inside the bold "250" of a task line: bold + checkbox active.
    let position = editor.body.text().find("250").map_or(0, |index| index + 1);
    editor.body.select(position..position);
    editor.body.unfocus();
    assert_window(
        &mut memo,
        [640.0, 300.0],
        ThemeKind::Dark,
        "memo-toolbar-active",
    );
}

#[test]
fn unsaved_changes_dialog() {
    let document = sample_document();
    let ui_state = selected_first(&document);
    let (mut memo, _worker, _directory) = memo_window(document, ui_state);
    memo.apply(&WindowAction::StartEditing, Language::English);
    let Some(editor) = memo.editor.as_mut() else {
        panic!("editing did not start");
    };
    editor.session.title.push_str(" (changed)");
    assert!(!memo.request_close(), "closing must ask first");
    assert_window(
        &mut memo,
        [640.0, 360.0],
        ThemeKind::Dark,
        "memo-unsaved-dialog",
    );
}

#[test]
fn unsaved_changes_dialog_stacks_buttons_in_a_narrow_window() {
    let document = sample_document();
    let ui_state = selected_first(&document);
    let (mut memo, _worker, _directory) = memo_window(document, ui_state);
    memo.apply(&WindowAction::StartEditing, Language::English);
    let Some(editor) = memo.editor.as_mut() else {
        panic!("editing did not start");
    };
    editor.session.title.push_str(" (changed)");
    assert!(!memo.request_close(), "closing must ask first");
    assert_window(
        &mut memo,
        [220.0, 420.0],
        ThemeKind::Dark,
        "memo-unsaved-dialog-narrow",
    );
}

#[test]
fn narrow_window_dims_the_edited_note_behind_the_drawer() {
    let document = sample_document();
    let ui_state = selected_first(&document);
    let (mut memo, _worker, _directory) = memo_window(document, ui_state);
    memo.apply(&WindowAction::StartEditing, Language::English);
    memo.drawer_open = true;
    assert_window(
        &mut memo,
        [360.0, 420.0],
        ThemeKind::Dark,
        "memo-narrow-drawer-editing",
    );
}

#[test]
fn note_list_can_be_dragged_narrower_than_the_note_pane_minimum() {
    let document = sample_document();
    let ui_state = UiState {
        note_list_characters: 10.0,
        ..selected_first(&document)
    };
    let (mut memo, _worker, _directory) = memo_window(document, ui_state);
    assert_window(
        &mut memo,
        [640.0, 360.0],
        ThemeKind::Dark,
        "memo-narrow-note-list",
    );
}

#[test]
fn editing_with_preview_turns_the_note_list_into_a_drawer_earlier() {
    let document = sample_document();
    let ui_state = UiState {
        preview_visible: true,
        ..selected_first(&document)
    };
    let (mut memo, _worker, _directory) = memo_window(document, ui_state);
    memo.apply(&WindowAction::StartEditing, Language::English);
    if let Some(editor) = memo.editor.as_mut() {
        editor.body.unfocus();
    }
    assert_window(
        &mut memo,
        [600.0, 360.0],
        ThemeKind::Dark,
        "memo-editor-preview-hides-note-list",
    );
}

#[test]
fn chat_links_show_as_chips_in_the_note_view() {
    let mut document = sample_document();
    if let Some(Node::Note(first)) = document.root.first_mut() {
        first.body = "## Route\n- Start at [&BDgAAAA=], buy [&AvoJTQAA]\n- Budget: [&AdsnAAA=]\n- Build: [&DQc=]\n- Gear: [&CwEAAAA=] [&BnMVAAA=] [&CgEAAAA=] [&DgEAAAA=]\n- Broken: [&KgAAAAA=]".to_owned();
    }
    let ui_state = selected_first(&document);
    let (mut memo, _worker, _directory) = memo_window(document, ui_state);
    assert_window(
        &mut memo,
        [640.0, 360.0],
        ThemeKind::Dark,
        "memo-chat-link-chips",
    );
}
