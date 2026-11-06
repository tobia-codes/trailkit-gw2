//! Snapshot tests of the memo main window (memo plan M65), run with
//! `cargo xtask ui-test` (`--update` rewrites `tests/snapshots/`).

mod states;
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
    storage_worker::{SavingAllowed, StorageWorker},
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
    let theme = theme.theme();
    let _style = StyleScope::push_with_cursor_extent(ui, &theme, None);
    let mut title_bar = TitleBarState::default();
    let mut open = true;
    let toggles = memo.title_bar_toggles(language);
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
    if let Some(index) = clicked {
        memo.apply_title_bar_toggle(index);
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

#[test]
fn right_click_on_an_empty_list_spot_creates_a_root_note() {
    let document = sample_document();
    let root_count = document.root.len();
    let ui_state = selected_first_with_search(&document);
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
    let ui_state = selected_first_with_search(&document);
    let (mut memo, _worker, _directory) = memo_window(document, ui_state);
    let size = [640.0, 360.0];
    let mut harness = UiHarness::new(size);
    let row = [100.0, 145.0];
    let input = |position: [f32; 2], right: bool| FrameInput {
        mouse_position: position,
        right_button_down: right,
        ..FrameInput::default()
    };
    let mut draw_dark = |ui: &Ui| draw(ui, &mut memo, size, ThemeKind::Dark);
    harness.settle(2, &input(row, false), &mut draw_dark);
    harness.frame(&input(row, true), &mut draw_dark);
    harness.frame(&input(row, false), &mut draw_dark);
    let rename_item = [row[0] + 30.0, row[1] + 14.0];
    let image = harness.settle(3, &input(rename_item, false), &mut draw_dark);
    assert_snapshot(env!("CARGO_MANIFEST_DIR"), "memo-menu-item-hover", &image);
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
    let ui_state = selected_first_with_search(&document);
    let (mut memo, _worker, _directory) = memo_window(document, ui_state);
    let size = [640.0, 360.0];
    let mut harness = UiHarness::new(size);
    {
        let mut draw_dark = |ui: &Ui| draw(ui, &mut memo, size, ThemeKind::Dark);
        // Lower half of the expanded "Raids" row = first place inside (M107).
        let image = drag_rows(&mut harness, [100.0, 145.0], [100.0, 216.0], &mut draw_dark);
        assert_snapshot(env!("CARGO_MANIFEST_DIR"), "memo-drag-into-group", &image);
        let release = FrameInput {
            mouse_position: [100.0, 216.0],
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
    let ui_state = selected_first_with_search(&document);
    let (mut memo, _worker, _directory) = memo_window(document, ui_state);
    let size = [640.0, 360.0];
    let mut harness = UiHarness::new(size);
    {
        let mut draw_dark = |ui: &Ui| draw(ui, &mut memo, size, ThemeKind::Dark);
        // "Raids" over its own child: ban icon, no line (M109).
        let image = drag_rows(&mut harness, [100.0, 209.0], [100.0, 244.0], &mut draw_dark);
        assert_snapshot(env!("CARGO_MANIFEST_DIR"), "memo-drag-impossible", &image);
        let release = FrameInput {
            mouse_position: [100.0, 244.0],
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
    // Rest the mouse on the closing `x`: its hover square must sit
    // centered on it (M128).
    let over_clear = FrameInput {
        mouse_position: [185.0, 70.0],
        ..FrameInput::default()
    };
    let mut image = draw_frame(&mut harness, &over_clear);
    for _ in 0..3 {
        image = draw_frame(&mut harness, &over_clear);
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

#[test]
fn only_the_tree_scrolls_below_the_fixed_list_header() {
    let mut document = sample_document();
    for number in 1..=20 {
        document.root.push(Node::Note(note(
            &format!("Note {number}"),
            "",
            NoteKind::Normal,
        )));
    }
    let ui_state = selected_first_with_search(&document);
    let (mut memo, _worker, _directory) = memo_window(document, ui_state);
    let size = [640.0, 360.0];
    let mut harness = UiHarness::new(size);
    let over_list = |mouse_wheel| FrameInput {
        mouse_position: [100.0, 250.0],
        mouse_wheel,
        ..FrameInput::default()
    };
    let mut draw_dark = |ui: &Ui| draw(ui, &mut memo, size, ThemeKind::Dark);
    harness.settle(2, &over_list(0.0), &mut draw_dark);
    harness.frame(&over_list(-5.0), &mut draw_dark);
    // Search field and buttons stay at the top (M129).
    let image = harness.settle(3, &over_list(0.0), &mut draw_dark);
    assert_snapshot(env!("CARGO_MANIFEST_DIR"), "memo-list-scrolled", &image);
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
fn search_field_stays_shown() {
    let document = sample_document();
    let ui_state = selected_first_with_search(&document);
    let (mut memo, _worker, _directory) = memo_window(document, ui_state);
    assert_window(
        &mut memo,
        [640.0, 360.0],
        ThemeKind::Dark,
        "memo-search-shown",
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
fn hiding_the_search_field_clears_the_search() {
    let document = sample_document();
    let (mut memo, _worker, _directory) =
        memo_window(document.clone(), selected_first_with_search(&document));
    memo.search_field.text = "ecto".to_owned();
    memo.apply(&WindowAction::HideSearch, Language::English);
    assert!(!memo.ui_state.search_visible);
    assert!(memo.search_field.text.is_empty(), "no hidden filter (M138)");
    memo.apply(&WindowAction::ShowSearch, Language::English);
    assert!(memo.ui_state.search_visible);
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
fn ctrl_f_does_nothing_in_the_editor() {
    let document = sample_document();
    let (mut memo, _worker, _directory) = memo_window(document.clone(), selected_first(&document));
    memo.apply(&WindowAction::StartEditing, Language::English);
    let size = [640.0, 360.0];
    let mut harness = UiHarness::new(size);
    let mut draw_frame = |ui: &Ui| draw(ui, &mut memo, size, ThemeKind::Dark);
    harness.step(&FrameInput::default(), &mut draw_frame);
    harness.step(
        &FrameInput {
            control: true,
            keys: vec![0x46],
            ..FrameInput::default()
        },
        &mut draw_frame,
    );
    assert!(
        memo.editor.is_some(),
        "a shortcut must not end an edit (M167)"
    );
    assert!(!memo.ui_state.search_visible);
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

/// The first group of `document`.
fn first_group(document: &NotesDocument) -> Option<GroupId> {
    document.root.iter().find_map(|node| match node {
        Node::Group(group) => Some(group.id),
        Node::Note(_) => None,
    })
}

#[test]
fn clicking_a_group_toggles_it_and_keeps_the_shown_note() {
    let document = sample_document();
    let Some(group) = first_group(&document) else {
        panic!("the sample has a group");
    };
    let shown = first_note(&document);
    let (mut memo, _worker, _directory) = memo_window(document.clone(), selected_first(&document));
    let is_expanded = |memo: &MemoWindow| memo.document.group(group).is_some_and(|g| g.expanded);
    let before = is_expanded(&memo);
    memo.apply(&WindowAction::ClickGroup(group), Language::English);
    assert_eq!(is_expanded(&memo), !before, "the click toggles (M172)");
    assert_eq!(memo.ui_state.selected, Some(NodeId::Group(group)));
    assert_eq!(
        memo.selected_note().map(|note| NodeId::Note(note.id)),
        shown,
        "the note stays (M173)"
    );
    memo.apply(&WindowAction::ClickGroup(group), Language::English);
    assert_eq!(is_expanded(&memo), before);
}

#[test]
fn clicking_a_group_during_a_search_changes_only_the_search() {
    let document = sample_document();
    let Some(group) = first_group(&document) else {
        panic!("the sample has a group");
    };
    let (mut memo, _worker, _directory) =
        memo_window(document.clone(), selected_first_with_search(&document));
    memo.search_field.text = "clears".to_owned();
    memo.update_search();
    let saved = memo.document.group(group).is_some_and(|g| g.expanded);
    memo.apply(&WindowAction::ClickGroup(group), Language::English);
    assert!(memo.collapsed_in_search.contains(&group));
    assert_eq!(
        memo.document.group(group).is_some_and(|g| g.expanded),
        saved,
        "the saved state stays (M124)"
    );
}

#[test]
fn the_shown_note_ticks_its_tasks_while_a_group_is_selected() {
    let document = sample_document();
    let Some(group) = first_group(&document) else {
        panic!("the sample has a group");
    };
    let (mut memo, _worker, _directory) = memo_window(document.clone(), selected_first(&document));
    memo.apply(&WindowAction::ClickGroup(group), Language::English);
    memo.apply(&WindowAction::StartEditing, Language::English);
    assert!(memo.editor.is_some(), "Edit acts on the shown note");
}

#[test]
fn the_search_key_toggles_the_search_except_in_the_editor() {
    let document = sample_document();
    let (mut memo, _worker, _directory) = memo_window(document.clone(), selected_first(&document));
    memo.window_menu_open = true;
    memo.toggle_search_by_key();
    assert!(
        memo.ui_state.search_visible,
        "the caught key shows it (M174)"
    );
    assert!(!memo.window_menu_open, "and closes the menu above it");
    memo.search_field.text = "ecto".to_owned();
    memo.toggle_search_by_key();
    assert!(!memo.ui_state.search_visible, "again closes it (M176)");
    assert!(memo.search_field.text.is_empty(), "no hidden filter (M138)");
    memo.apply(&WindowAction::OpenTrash, Language::English);
    memo.ui_state.search_visible = true;
    memo.toggle_search_by_key();
    assert!(memo.ui_state.search_visible, "the trash makes way for it");
    assert!(!memo.trash.visible);
    memo.apply(&WindowAction::HideSearch, Language::English);
    memo.apply(&WindowAction::StartEditing, Language::English);
    memo.toggle_search_by_key();
    assert!(
        !memo.ui_state.search_visible,
        "a key must not end an edit (M167)"
    );
    assert!(!memo.takes_search_key(), "and the game keeps it there");
}

#[test]
fn the_search_x_clears_the_text_and_then_closes_the_field() {
    let document = sample_document();
    let (mut memo, _worker, _directory) =
        memo_window(document.clone(), selected_first_with_search(&document));
    memo.search_field.text = "ecto".to_owned();
    let size = [640.0, 360.0];
    let mut harness = UiHarness::new(size);
    let end_of_field = [185.0, 68.0];
    let mut click_x = |memo: &mut MemoWindow| {
        for down in [false, false, true, false, false] {
            let input = FrameInput {
                mouse_position: end_of_field,
                left_button_down: down,
                ..FrameInput::default()
            };
            harness.step(&input, |ui| draw(ui, memo, size, ThemeKind::Dark));
        }
    };
    click_x(&mut memo);
    assert!(memo.search_field.text.is_empty(), "text first (M176)");
    assert!(memo.ui_state.search_visible, "the field stays");
    click_x(&mut memo);
    assert!(!memo.ui_state.search_visible, "the empty field closes");
}

#[test]
fn the_delete_key_belongs_to_a_focused_list_only() {
    let document = sample_document();
    let (mut memo, _worker, _directory) = memo_window(document.clone(), selected_first(&document));
    let size = [640.0, 360.0];
    let mut harness = UiHarness::new(size);
    let at = |position: [f32; 2], down: bool| FrameInput {
        mouse_position: position,
        left_button_down: down,
        ..FrameInput::default()
    };
    let mut draw_frame = |memo: &mut MemoWindow, input: &FrameInput| {
        harness.step(input, |ui| draw(ui, memo, size, ThemeKind::Dark));
    };
    let list_row = [100.0, 138.0];
    for input in [
        at(list_row, false),
        at(list_row, true),
        at(list_row, false),
        at(list_row, false),
    ] {
        draw_frame(&mut memo, &input);
    }
    assert!(
        memo.takes_delete_key(),
        "a click in the list gives it the key (M175)"
    );
    let note_text = [420.0, 300.0];
    for input in [
        at(note_text, false),
        at(note_text, true),
        at(note_text, false),
        at(note_text, false),
    ] {
        draw_frame(&mut memo, &input);
    }
    assert!(
        !memo.takes_delete_key(),
        "a click in the note takes it away"
    );
    memo.delete_selected_by_key(Language::English);
    assert!(
        memo.document.trash.is_empty(),
        "reading a note cannot delete it"
    );
    for input in [
        at(list_row, false),
        at(list_row, true),
        at(list_row, false),
        at(list_row, false),
    ] {
        draw_frame(&mut memo, &input);
    }
    memo.delete_selected_by_key(Language::English);
    assert_eq!(
        memo.document.trash.len(),
        1,
        "the caught key deletes (M133)"
    );
}
