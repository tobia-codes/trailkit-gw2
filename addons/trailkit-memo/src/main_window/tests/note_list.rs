//! Tests of the note list: context menu, drag and drop, revealing a note,
//! scrolling, clicking groups and the Delete key (memo M30–M50, M107, M144).

use super::*;

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

/// "Go to note" reveals a note in a collapsed group: the group expands (and
/// is saved so), the note is selected and shown (M185).
#[test]
fn revealing_a_note_expands_its_group_and_selects_it() {
    let mut document = sample_document();
    for node in &mut document.root {
        if let Node::Group(group) = node {
            group.expanded = false;
        }
    }
    let Some(NodeId::Note(weekly)) = weekly_note(&document) else {
        panic!("the sample has a weekly note in a group");
    };
    let (mut window, _worker, _directory) = memo_window(document, UiState::default());
    window.apply(&WindowAction::RevealNote(weekly), Language::English);
    assert_eq!(expanded_flags(&window.document), vec![true]);
    assert_eq!(window.ui_state.selected, Some(NodeId::Note(weekly)));
    assert_eq!(window.ui_state.shown_note, Some(weekly));
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
