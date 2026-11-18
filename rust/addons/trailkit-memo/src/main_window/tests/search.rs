//! Tests of the search: filtering the list, hits, the search field and its
//! keys (memo M122, M129, M138).

use super::*;
use trailkit_memo_shared::list_filter::ListFilter;

/// Shows the search as Shift pressed twice does (M199), types `query` and
/// lets the window settle; returns the last image.
fn search_for(
    memo: &mut MemoWindow,
    size: [f32; 2],
    query: &str,
) -> trailkit_ui_testing::raster::Image {
    memo.toggle_search_by_key();
    let mut harness = UiHarness::new(size);
    let mut draw_frame = |harness: &mut UiHarness, input: &FrameInput| {
        harness.frame(input, |ui| draw(ui, memo, size, ThemeKind::Dark))
    };
    draw_frame(&mut harness, &FrameInput::default());
    draw_frame(&mut harness, &FrameInput::default());
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

/// Presses Ctrl+F in a focused window showing `memo`.
fn press_ctrl_f(memo: &mut MemoWindow) {
    let size = [640.0, 360.0];
    let mut harness = UiHarness::new(size);
    let mut draw_frame = |ui: &Ui| draw(ui, memo, size, ThemeKind::Dark);
    harness.step(&FrameInput::default(), &mut draw_frame);
    harness.step(
        &FrameInput {
            control: true,
            keys: vec![0x46],
            ..FrameInput::default()
        },
        &mut draw_frame,
    );
    harness.step(&FrameInput::default(), &mut draw_frame);
}

#[test]
fn ctrl_f_finds_in_the_open_note_also_in_the_editor() {
    let document = sample_document();
    let (mut memo, _worker, _directory) = memo_window(document.clone(), selected_first(&document));
    press_ctrl_f(&mut memo);
    assert!(memo.find.visible, "Ctrl+F opens find in the note (M198)");
    assert!(!memo.ui_state.search_visible, "not the list search (M199)");
    press_ctrl_f(&mut memo);
    assert!(!memo.find.visible, "Ctrl+F again closes it");
    memo.apply(&WindowAction::StartEditing, Language::English);
    press_ctrl_f(&mut memo);
    assert!(memo.find.visible, "also in the editor");
    assert!(memo.editor.is_some(), "a shortcut must not end an edit");
    press_ctrl_f(&mut memo);
    assert!(!memo.find.visible, "and closes there too");
    assert!(memo.editor.is_some(), "closing keeps the edit");
}

#[test]
fn ctrl_f_does_nothing_without_an_open_note() {
    let (mut memo, _worker, _directory) = memo_window(sample_document(), UiState::default());
    press_ctrl_f(&mut memo);
    assert!(
        !memo.find.visible,
        "only an open note can be searched (Q513)"
    );
    assert!(!memo.takes_find_key(), "the game keeps the key");
}

#[test]
fn find_steps_through_the_hits_and_wraps() {
    let document = sample_document();
    let shown = first_note(&document).and_then(|node| match node {
        NodeId::Note(note) => Some(note),
        NodeId::Group(_) => None,
    });
    let (mut memo, _worker, _directory) = memo_window(document.clone(), selected_first(&document));
    memo.find_text("ecto");
    memo.record_find_hits(shown, 3);
    memo.step_find(false);
    assert_eq!(
        memo.find.current, 2,
        "back from the first wraps to the last (Q505)"
    );
    memo.step_find(true);
    assert_eq!(memo.find.current, 0, "on from the last wraps to the first");
    memo.step_find(true);
    memo.record_find_hits(shown, 1);
    assert_eq!(
        memo.find.current, 0,
        "fewer hits keep the current one in range"
    );
    memo.find_text("zzz");
    memo.record_find_hits(shown, 0);
    memo.step_find(true);
    assert_eq!(memo.find.current, 0, "without hits nothing moves");
}

#[test]
fn find_offers_the_selection_then_the_list_search() {
    let document = sample_document();
    let (mut memo, _worker, _directory) = memo_window(document.clone(), selected_first(&document));
    memo.search_field.text = "coins".to_owned();
    memo.open_find();
    assert_eq!(memo.find.field.text, "coins", "the list search (Q507)");
    memo.close_find();
    memo.apply(&WindowAction::StartEditing, Language::English);
    if let Some(editor) = memo.editor.as_mut() {
        let start = editor.body_text().find("Globs").unwrap_or(0);
        editor.select_in_body(start..start + "Globs".len());
    }
    memo.open_find();
    assert_eq!(
        memo.find.field.text, "Globs",
        "the editor's selection first"
    );
}

#[test]
fn escape_closes_the_find_bar_before_the_window() {
    let document = sample_document();
    let (mut memo, _worker, _directory) = memo_window(document.clone(), selected_first(&document));
    memo.find_text("ecto");
    assert!(memo.takes_escape(), "the open bar takes Escape (Q508)");
    let size = [640.0, 360.0];
    let mut harness = UiHarness::new(size);
    harness.step(&FrameInput::default(), |ui| {
        memo.handle_escape(ui);
    });
    assert!(!memo.find.visible);
    assert!(!memo.takes_escape(), "the next Escape closes the window");
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
fn a_click_on_a_row_selects_it_during_a_search() {
    let document = sample_document();
    let shopping = first_note(&document);
    let ui_state = UiState {
        selected: weekly_note(&document),
        ..UiState::default()
    };
    let (mut memo, _worker, _directory) = memo_window(document, ui_state);
    let size = [640.0, 360.0];
    // "Shopping" leaves the shopping list as the first row; the open weekly
    // note stays visible below it.
    search_for(&mut memo, size, "Shopping");
    let row = [100.0, 145.0];
    let input = |down: bool| FrameInput {
        mouse_position: row,
        left_button_down: down,
        ..FrameInput::default()
    };
    let mut harness = UiHarness::new(size);
    let mut draw_dark = |ui: &Ui| draw(ui, &mut memo, size, ThemeKind::Dark);
    harness.settle(2, &input(false), &mut draw_dark);
    harness.step(&input(true), &mut draw_dark);
    harness.settle(2, &input(false), &mut draw_dark);
    assert_eq!(
        memo.ui_state.selected, shopping,
        "a press and release in place selects, even though dragging is blocked (M125)"
    );
}

#[test]
fn the_open_note_leaves_the_list_but_stays_shown_during_a_search() {
    let document = sample_document();
    let weekly = weekly_note(&document);
    let ui_state = UiState {
        selected: weekly,
        search_visible: true,
        ..UiState::default()
    };
    let (mut memo, _worker, _directory) = memo_window(document, ui_state);
    memo.search_field.text = "Shopping".to_owned();
    memo.update_search();
    let Some(search) = &memo.search.current else {
        panic!("the search runs");
    };
    assert!(
        weekly.is_some_and(|node| !search.results.is_visible(node)),
        "the open note does not match, so the list hides it (M196)"
    );
    assert_eq!(memo.ui_state.selected, weekly, "but it stays open");
    memo.note_list_takes_delete_key = true;
    assert!(
        !memo.takes_delete_key(),
        "the Delete key cannot delete the hidden note"
    );
    memo.search_field.text.clear();
    memo.update_search();
    assert!(memo.takes_delete_key(), "it is back in the list");
}
