//! Tests of the search: filtering the list, hits, the search field and its
//! keys (memo M122, M129, M138).

use super::*;

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
