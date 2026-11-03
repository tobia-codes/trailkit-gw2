//! Snapshot tests of the memo main window (memo plan M65), run with
//! `cargo xtask ui-test` (`--update` rewrites `tests/snapshots/`).

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
        Settings::DEFAULT,
    );
    (window, worker, directory)
}

/// Draws `memo` like the addon does: theme, window, then its dialog.
fn draw(ui: &Ui, memo: &mut MemoWindow, size: [f32; 2], theme: ThemeKind) {
    let theme = theme.theme();
    let _style = StyleScope::push_with_cursor_extent(ui, &theme, None);
    let mut title_bar = TitleBarState::default();
    let mut open = true;
    draw_window(
        ui,
        &theme,
        Window::new("Notes###memo")
            .position([0.0, 0.0], Condition::Always)
            .size(size, Condition::Always),
        title_bar_texts(Language::English),
        &mut title_bar,
        &mut open,
        || memo.draw(ui, Language::English, &theme),
    );
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
    let field = [400.0, 106.0];
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

    let daily_row = [400.0, 160.0];
    harness.frame(&at(daily_row, false), &mut draw_light);
    harness.frame(&at(daily_row, true), &mut draw_light);
    harness.frame(&at(daily_row, false), &mut draw_light);
    let image = harness.settle(2, &at([300.0, 300.0], false), &mut draw_light);
    // The chip takes the color of the daily badge (M104).
    assert_snapshot(env!("CARGO_MANIFEST_DIR"), "memo-daily-chip-light", &image);
    let kind = memo.document.note(note).map(|note| note.kind.period());
    assert_eq!(kind, Some(Some(ResetPeriod::Daily)));
}

#[test]
fn right_click_on_an_empty_list_spot_creates_a_root_note() {
    let document = sample_document();
    let root_count = document.root.len();
    let ui_state = selected_first(&document);
    let (mut memo, _worker, _directory) = memo_window(document, ui_state);
    let size = [640.0, 360.0];
    let mut harness = UiHarness::new(size);
    let empty_spot = [100.0, 300.0];
    let input = |position: [f32; 2], left: bool, right: bool| FrameInput {
        mouse_position: position,
        left_button_down: left,
        right_button_down: right,
        ..FrameInput::default()
    };
    let mut draw_dark = |ui: &Ui| draw(ui, &mut memo, size, ThemeKind::Dark);
    harness.settle(2, &input(empty_spot, false, false), &mut draw_dark);
    harness.frame(&input(empty_spot, false, true), &mut draw_dark);
    let image = harness.settle(2, &input(empty_spot, false, false), &mut draw_dark);
    assert_snapshot(env!("CARGO_MANIFEST_DIR"), "memo-list-context-menu", &image);
    let new_note_item = [empty_spot[0] + 30.0, empty_spot[1] + 14.0];
    harness.frame(&input(new_note_item, false, false), &mut draw_dark);
    harness.frame(&input(new_note_item, true, false), &mut draw_dark);
    harness.settle(2, &input(new_note_item, false, false), &mut draw_dark);
    assert_eq!(memo.document.root.len(), root_count + 1);
    assert!(matches!(memo.document.root.last(), Some(Node::Note(_))));
}

#[test]
fn reset_type_submenu_opens_beside_the_context_menu() {
    let document = sample_document();
    let ui_state = selected_first(&document);
    let (mut memo, _worker, _directory) = memo_window(document, ui_state);
    let size = [640.0, 360.0];
    let mut harness = UiHarness::new(size);
    let row = [100.0, 149.0];
    let input = |position: [f32; 2], right: bool| FrameInput {
        mouse_position: position,
        right_button_down: right,
        ..FrameInput::default()
    };
    let mut draw_dark = |ui: &Ui| draw(ui, &mut memo, size, ThemeKind::Dark);
    harness.settle(2, &input(row, false), &mut draw_dark);
    harness.frame(&input(row, true), &mut draw_dark);
    harness.frame(&input(row, false), &mut draw_dark);
    let reset_type_item = [row[0] + 30.0, row[1] + 38.0];
    let image = harness.settle(6, &input(reset_type_item, false), &mut draw_dark);
    assert_snapshot(
        env!("CARGO_MANIFEST_DIR"),
        "memo-reset-type-submenu",
        &image,
    );
}

/// Presses the left button at `from`, drags to `to` in a few frames and
/// returns the image of the last frame while the button is still down.
fn drag_rows(
    harness: &mut UiHarness,
    from: [f32; 2],
    to: [f32; 2],
    draw: &mut impl FnMut(&Ui),
) -> trailkit_ui_testing::raster::Image {
    let input = |position: [f32; 2], down: bool| FrameInput {
        mouse_position: position,
        left_button_down: down,
        ..FrameInput::default()
    };
    harness.settle(2, &input(from, false), &mut *draw);
    harness.frame(&input(from, true), &mut *draw);
    for step in 1..=4_u8 {
        let fraction = f32::from(step) / 4.0;
        let position = [
            from[0] + (to[0] - from[0]) * fraction,
            from[1] + (to[1] - from[1]) * fraction,
        ];
        harness.frame(&input(position, true), &mut *draw);
    }
    harness.settle(3, &input(to, true), &mut *draw)
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

#[test]
fn dragging_a_note_into_an_expanded_group_moves_it_to_the_line() {
    let document = sample_document();
    let ui_state = selected_first(&document);
    let (mut memo, _worker, _directory) = memo_window(document, ui_state);
    let size = [640.0, 360.0];
    let mut harness = UiHarness::new(size);
    {
        let mut draw_dark = |ui: &Ui| draw(ui, &mut memo, size, ThemeKind::Dark);
        // Lower half of the expanded "Raids" row = first place inside (M107).
        let image = drag_rows(&mut harness, [100.0, 149.0], [100.0, 220.0], &mut draw_dark);
        assert_snapshot(env!("CARGO_MANIFEST_DIR"), "memo-drag-into-group", &image);
        let release = FrameInput {
            mouse_position: [100.0, 220.0],
            ..FrameInput::default()
        };
        harness.settle(2, &release, &mut draw_dark);
    }
    assert_eq!(
        outline(&memo),
        "Dailies, Raids[Shopping list, Weekly clears]"
    );
}

#[test]
fn a_group_cannot_be_dropped_into_itself() {
    let document = sample_document();
    let ui_state = selected_first(&document);
    let (mut memo, _worker, _directory) = memo_window(document, ui_state);
    let size = [640.0, 360.0];
    let mut harness = UiHarness::new(size);
    {
        let mut draw_dark = |ui: &Ui| draw(ui, &mut memo, size, ThemeKind::Dark);
        // "Raids" over its own child: ban icon, no line (M109).
        let image = drag_rows(&mut harness, [100.0, 213.0], [100.0, 248.0], &mut draw_dark);
        assert_snapshot(env!("CARGO_MANIFEST_DIR"), "memo-drag-impossible", &image);
        let release = FrameInput {
            mouse_position: [100.0, 248.0],
            ..FrameInput::default()
        };
        harness.settle(2, &release, &mut draw_dark);
    }
    assert_eq!(
        outline(&memo),
        "Shopping list, Dailies, Raids[Weekly clears]"
    );
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

/// Focuses the search with Ctrl+F, types `query` and lets the window
/// settle; returns the last image.
fn search_for(
    memo: &mut MemoWindow,
    size: [f32; 2],
    query: &str,
) -> trailkit_ui_testing::raster::Image {
    let mut harness = UiHarness::new(size);
    let mut draw_frame = |harness: &mut UiHarness, input: &FrameInput| {
        harness.frame(input, |ui| draw(ui, memo, size, ThemeKind::Dark))
    };
    draw_frame(&mut harness, &FrameInput::default());
    draw_frame(
        &mut harness,
        &FrameInput {
            control: true,
            keys: vec![0x46],
            ..FrameInput::default()
        },
    );
    draw_frame(&mut harness, &FrameInput::default());
    draw_frame(
        &mut harness,
        &FrameInput {
            characters: query.to_owned(),
            ..FrameInput::default()
        },
    );
    let mut image = draw_frame(&mut harness, &FrameInput::default());
    for _ in 0..3 {
        image = draw_frame(&mut harness, &FrameInput::default());
    }
    image
}

#[test]
fn search_filters_the_list_and_marks_the_hits() {
    let document = sample_document();
    let ui_state = UiState {
        selected: weekly_note(&document),
        ..UiState::default()
    };
    let saved_expanded = expanded_flags(&document);
    let (mut memo, _worker, _directory) = memo_window(document, ui_state);
    // "clears" matches the title, "spirit" the content: the row shows one
    // content hit, the group counts one matching note (M119, M121, M122).
    let image = search_for(&mut memo, [640.0, 360.0], "clears SPIRIT");
    assert_eq!(memo.search_field.text, "clears SPIRIT");
    assert!(memo.search_takes_escape());
    assert_snapshot(env!("CARGO_MANIFEST_DIR"), "memo-search-hits", &image);

    // Escape (sent by Nexus as hiding the window) clears the search, and
    // the saved expand state of the group is untouched (M124).
    memo.clear_search_by_escape();
    assert!(memo.search_field.text.is_empty());
    assert_eq!(
        expanded_flags(&memo.document),
        saved_expanded,
        "the saved expand state is untouched"
    );
}

#[test]
fn search_without_hits_offers_to_clear_it() {
    let (mut memo, _worker, _directory) = memo_window(sample_document(), UiState::default());
    let image = search_for(&mut memo, [640.0, 360.0], "zzz");
    assert_snapshot(env!("CARGO_MANIFEST_DIR"), "memo-search-empty", &image);
}
