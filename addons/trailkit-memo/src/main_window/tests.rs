//! Snapshot tests of the memo main window (memo plan M65), run with
//! `cargo xtask ui-test` (`--update` rewrites `tests/snapshots/`).

mod note_list;
mod save_errors;
mod search;
mod states;
mod title_bar_help;
mod trash;

use std::time::Duration;

use nexus::imgui::{Condition, Ui, Window};
use trailkit_storage::DocumentStore;
use trailkit_ui::{
    style_scope::StyleScope,
    theme::ThemeKind,
    title_bar::{TitleBarState, draw_window},
};
use trailkit_ui_testing::{
    harness::{FrameInput, UiHarness},
    snapshot::assert_snapshot,
};

use super::*;
use trailkit_memo_shared::{
    note_tree::{Group, NoteKind, ResetPeriod, TreeNode as Node},
    storage_worker::{SaveConnection, SavingAllowed, SettingsSummary, StorageWorker, WorkerTiming},
};

/// The narrowest window (M168) at the test fonts.
const MINIMUM_WINDOW: [f32; 2] = [320.0, 360.0];

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
    let (worker, loaded) = StorageWorker::spawn(
        DocumentStore::new(directory.path()),
        WorkerTiming {
            debounce: Duration::from_secs(3600),
            retry_delay: Duration::from_secs(3600),
        },
        SavingAllowed { settings: false },
        SettingsSummary::default(),
    )
    .unwrap_or_else(|error| panic!("storage worker: {error}"));
    let reports = loaded
        .recv_timeout(Duration::from_secs(5))
        .unwrap_or_else(|error| panic!("storage worker load: {error}"))
        .reports;
    let window = MemoWindow::new(
        document,
        None,
        ui_state,
        SaveConnection {
            saver: worker.saver(),
            reports,
        },
        directory.path().to_path_buf(),
        DraftsDocument::default(),
        Settings::DEFAULT,
    );
    (window, worker, directory)
}

/// Draws `memo` in English like the addon does.
fn draw(ui: &Ui, memo: &mut MemoWindow, size: [f32; 2], theme: ThemeKind) {
    draw_in_language(ui, memo, size, theme, Language::English);
}

/// Draws `memo` like the addon does: theme, window, then its dialog.
fn draw_in_language(
    ui: &Ui,
    memo: &mut MemoWindow,
    size: [f32; 2],
    theme: ThemeKind,
    language: Language,
) {
    draw_like_the_addon(ui, memo, size, theme, language, false);
}

/// Draws `memo` like the addon does: theme, window with the window menu
/// and the help `?` (active while `help_open`) in the title bar, then its
/// dialog. A click on the window menu toggle is applied.
fn draw_like_the_addon(
    ui: &Ui,
    memo: &mut MemoWindow,
    size: [f32; 2],
    theme: ThemeKind,
    language: Language,
    help_open: bool,
) {
    let theme = theme.theme();
    let _style = StyleScope::push_with_cursor_extent(ui, &theme, None);
    let mut title_bar = TitleBarState::default();
    let mut open = true;
    let [menu] = memo.title_bar_toggles(language);
    let toggles = [menu, help_title_bar_toggle(language, help_open)];
    let clicked = draw_window(
        ui,
        &theme,
        Window::new("Notes###memo")
            .position([0.0, 0.0], Condition::Always)
            .size(size, Condition::Always),
        title_bar_texts(language),
        &toggles,
        &mut title_bar,
        &mut open,
        || memo.draw(ui, language, &theme),
    );
    if clicked == Some(0) {
        memo.apply_title_bar_toggle(0);
    }
    memo.draw_dialog(ui, language, &theme);
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

/// Like [`selected_first`] with the search field shown (M138), the layout
/// the tests with fixed list coordinates were measured in.
fn selected_first_with_search(document: &NotesDocument) -> UiState {
    UiState {
        search_visible: true,
        ..selected_first(document)
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
        editor.unfocus_body();
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
        ..selected_first(&document)
    };
    let (mut memo, _worker, _directory) = memo_window(document, ui_state);
    memo.apply(&WindowAction::StartEditing, Language::English);
    if let Some(editor) = memo.editor.as_mut() {
        editor.unfocus_body();
    }
    assert_window(
        &mut memo,
        MINIMUM_WINDOW,
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
    let position = editor.body_text().find("250").map_or(0, |index| index + 1);
    editor.select_in_body(position..position);
    editor.unfocus_body();
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
    editor.title_mut().push_str(" (changed)");
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
    editor.title_mut().push_str(" (changed)");
    assert!(!memo.request_close(), "closing must ask first");
    assert_window(
        &mut memo,
        [220.0, 420.0],
        ThemeKind::Dark,
        "memo-unsaved-dialog-narrow",
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

#[test]
fn control_mouse_wheel_changes_the_text_scale_and_control_zero_resets_it() {
    let document = sample_document();
    let ui_state = selected_first(&document);
    let (mut memo, _worker, _directory) = memo_window(document, ui_state);
    let size = [480.0, 300.0];
    let mut harness = UiHarness::new(size);
    let over_window = [200.0, 150.0];
    let idle = FrameInput {
        mouse_position: over_window,
        ..FrameInput::default()
    };
    harness.settle(2, &idle, |ui| draw(ui, &mut memo, size, ThemeKind::Dark));
    let wheel_up = FrameInput {
        control: true,
        mouse_wheel: 1.0,
        ..idle.clone()
    };
    harness.frame(&wheel_up, |ui| draw(ui, &mut memo, size, ThemeKind::Dark));
    assert_eq!(memo.settings.text_scale, TextScale::DEFAULT.stepped(1.0));
    // The fonts follow only once the wheel rests.
    assert_eq!(memo.text_scale(), TextScale::DEFAULT);
    // Clicking focuses the window; Ctrl+0 then resets the size.
    let click = FrameInput {
        left_button_down: true,
        ..idle.clone()
    };
    harness.frame(&click, |ui| draw(ui, &mut memo, size, ThemeKind::Dark));
    harness.frame(&idle, |ui| draw(ui, &mut memo, size, ThemeKind::Dark));
    let reset = FrameInput {
        control: true,
        keys: vec![0x30],
        ..idle
    };
    harness.frame(&reset, |ui| draw(ui, &mut memo, size, ThemeKind::Dark));
    assert_eq!(memo.settings.text_scale, TextScale::DEFAULT);
}

#[test]
fn note_title_stays_at_the_top_while_the_content_scrolls() {
    let mut document = sample_document();
    if let Some(Node::Note(first)) = document.root.first_mut() {
        first.body = (1..=30)
            .map(|line| format!("- Line {line}"))
            .collect::<Vec<_>>()
            .join("\n");
    }
    let ui_state = UiState {
        note_scroll: 200.0,
        ..selected_first(&document)
    };
    let (mut memo, _worker, _directory) = memo_window(document, ui_state);
    assert_window(
        &mut memo,
        [640.0, 300.0],
        ThemeKind::Dark,
        "memo-scrolled-note",
    );
}

#[test]
fn kind_dropdown_lists_the_kinds_and_changes_the_note() {
    let document = sample_document();
    let ui_state = selected_first(&document);
    let Some(NodeId::Note(note)) = ui_state.selected else {
        panic!("the sample has a root note");
    };
    let (mut memo, _worker, _directory) = memo_window(document, ui_state);
    let size = [640.0, 360.0];
    let mut harness = UiHarness::new(size);
    let at = |position: [f32; 2], down: bool| FrameInput {
        mouse_position: position,
        left_button_down: down,
        ..FrameInput::default()
    };
    let field = [360.0, 106.0];
    let mut draw_light = |ui: &Ui| draw(ui, &mut memo, size, ThemeKind::Light);
    harness.settle(2, &at(field, false), &mut draw_light);
    harness.frame(&at(field, true), &mut draw_light);
    harness.frame(&at(field, false), &mut draw_light);
    let image = harness.settle(3, &at([300.0, 300.0], false), &mut draw_light);
    assert_snapshot(
        env!("CARGO_MANIFEST_DIR"),
        "memo-kind-dropdown-light",
        &image,
    );

    let daily_row = [360.0, 171.0];
    harness.frame(&at(daily_row, false), &mut draw_light);
    harness.frame(&at(daily_row, true), &mut draw_light);
    harness.frame(&at(daily_row, false), &mut draw_light);
    let image = harness.settle(2, &at([300.0, 300.0], false), &mut draw_light);
    // The chip takes the color of the daily badge (M104).
    assert_snapshot(env!("CARGO_MANIFEST_DIR"), "memo-daily-chip-light", &image);
    let kind = memo.document.note(note).map(|note| note.kind.period());
    assert_eq!(kind, Some(Some(ResetPeriod::Daily)));
}

/// The titles on the root level and in the first group, e.g.
/// `Shopping list, Dailies, Raids[Weekly clears]`.
fn outline(memo: &MemoWindow) -> String {
    memo.document
        .root
        .iter()
        .map(|node| match node {
            Node::Note(note) => note.title.clone(),
            Node::Group(group) => format!(
                "{}[{}]",
                group.title,
                group
                    .children
                    .iter()
                    .map(|child| child.title().to_owned())
                    .collect::<Vec<_>>()
                    .join(", ")
            ),
        })
        .collect::<Vec<_>>()
        .join(", ")
}

/// The id of the weekly note inside the "Raids" group of
/// [`sample_document`].
fn weekly_note(document: &NotesDocument) -> Option<NodeId> {
    document.root.iter().find_map(|node| match node {
        Node::Group(group) => group.children.iter().find_map(|child| match child {
            Node::Note(note) => Some(NodeId::Note(note.id)),
            Node::Group(_) => None,
        }),
        Node::Note(_) => None,
    })
}

/// The saved expand state of every root group of `document`.
fn expanded_flags(document: &NotesDocument) -> Vec<bool> {
    document
        .root
        .iter()
        .filter_map(|node| match node {
            Node::Group(group) => Some(group.expanded),
            Node::Note(_) => None,
        })
        .collect()
}

#[test]
fn compact_note_view_shows_only_the_note() {
    let document = sample_document();
    let ui_state = UiState {
        compact_note_view: true,
        note_list_hidden: true,
        ..selected_first(&document)
    };
    let (mut memo, _worker, _directory) = memo_window(document, ui_state);
    assert_window(&mut memo, [640.0, 360.0], ThemeKind::Dark, "memo-compact");
}

#[test]
fn rename_from_the_note_menu_opens_the_editor_in_the_title() {
    let document = sample_document();
    let (mut memo, _worker, _directory) = memo_window(document.clone(), selected_first(&document));
    memo.apply(&WindowAction::StartRenaming, Language::English);
    assert!(
        memo.editor.is_some(),
        "renaming happens in the editor (M161)"
    );
    assert!(!memo.note_list_shown(), "the editor hides the list (M167)");
    assert_window(
        &mut memo,
        [640.0, 360.0],
        ThemeKind::Dark,
        "memo-rename-title-focus",
    );
}

#[test]
fn editing_leaves_the_compact_note_view() {
    let document = sample_document();
    let ui_state = UiState {
        compact_note_view: true,
        ..selected_first(&document)
    };
    let (mut memo, _worker, _directory) = memo_window(document, ui_state);
    memo.apply(&WindowAction::StartEditing, Language::English);
    assert!(memo.editor.is_some());
    assert!(
        !memo.ui_state.compact_note_view,
        "the editor ends it (M139)"
    );
}

#[test]
fn title_bar_toggle_opens_and_closes_the_window_menu() {
    let document = sample_document();
    let (mut memo, _worker, _directory) = memo_window(document.clone(), selected_first(&document));
    assert!(!memo.title_bar_toggles(Language::English)[0].active);
    memo.apply_title_bar_toggle(0);
    assert!(memo.title_bar_toggles(Language::English)[0].active);
    memo.apply_title_bar_toggle(0);
    assert!(!memo.window_menu_open);
}

#[test]
fn escape_closes_the_window_menu_before_the_window() {
    let document = sample_document();
    let (mut memo, _worker, _directory) = memo_window(document.clone(), selected_first(&document));
    memo.apply_title_bar_toggle(0);
    assert!(!memo.request_close(), "the menu closes first (M166)");
    assert!(!memo.window_menu_open);
    assert!(memo.request_close(), "then the window closes");
}

#[test]
fn showing_the_notes_returns_to_the_regular_page() {
    let document = sample_document();
    let selected = selected_first(&document);
    let ui_state = UiState {
        note_list_hidden: true,
        search_visible: true,
        compact_note_view: true,
        ..selected
    };
    let (mut memo, _worker, _directory) = memo_window(document, ui_state);
    memo.search_field.text = "ecto".to_owned();
    memo.apply(&WindowAction::OpenTrash, Language::English);
    memo.apply(&WindowAction::ShowNotes, Language::English);
    assert!(!memo.trash.visible);
    assert!(memo.note_list_shown());
    assert!(!memo.ui_state.search_visible, "the search closes (Q120)");
    assert!(!memo.ui_state.compact_note_view, "Edit is back (Q123)");
    assert!(memo.search_field.text.is_empty());
    assert_eq!(memo.ui_state.selected, selected.selected, "the note stays");
}

#[test]
fn the_editor_hides_the_note_list_until_it_ends() {
    let document = sample_document();
    let (mut memo, _worker, _directory) = memo_window(document.clone(), selected_first(&document));
    assert!(memo.note_list_shown());
    memo.apply(&WindowAction::StartEditing, Language::English);
    assert!(!memo.note_list_shown(), "editing hides the list (M167)");
    memo.apply(&WindowAction::CancelEditing, Language::English);
    assert!(memo.note_list_shown(), "cancelling brings it back");
    memo.apply(&WindowAction::StartEditing, Language::English);
    memo.apply(&WindowAction::FinishEditing, Language::English);
    assert!(memo.note_list_shown(), "saving brings it back");
}

#[test]
fn the_editor_keeps_a_list_the_user_hid_hidden_after_it_ends() {
    let document = sample_document();
    let ui_state = UiState {
        note_list_hidden: true,
        ..selected_first(&document)
    };
    let (mut memo, _worker, _directory) = memo_window(document, ui_state);
    memo.apply(&WindowAction::StartEditing, Language::English);
    memo.apply(&WindowAction::CancelEditing, Language::English);
    assert!(!memo.note_list_shown(), "the user's choice stays (M167)");
}

#[test]
fn leaving_the_editor_from_the_menu_asks_about_unsaved_changes() {
    let document = sample_document();
    let (mut memo, _worker, _directory) = memo_window(document.clone(), selected_first(&document));
    memo.apply(&WindowAction::StartEditing, Language::English);
    if let Some(editor) = memo.editor.as_mut() {
        editor.title_mut().push_str(" (changed)");
    }
    for action in [WindowAction::ShowNotes, WindowAction::ShowSearch] {
        memo.apply(&action, Language::English);
        assert!(memo.editor.is_some(), "nothing is lost silently (M54)");
        assert_eq!(memo.leave_prompt, Some(LeaveReason::Action(action)));
        memo.apply(&WindowAction::LeaveCancelled, Language::English);
    }
}

#[test]
fn the_snapshots_use_the_minimum_window_width() {
    let mut measured = 0.0;
    let mut harness = UiHarness::new(MINIMUM_WINDOW);
    harness.step(&FrameInput::default(), |ui| {
        measured = minimum_window_width(ui);
    });
    assert!(
        (measured - MINIMUM_WINDOW[0]).abs() <= 24.0,
        "{measured} is far from {}",
        MINIMUM_WINDOW[0]
    );
}
