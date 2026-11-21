//! Tests of selecting and copying text in the reading view (M217, Q551–Q556):
//! the mouse selects, Ctrl+C copies the text as shown, Ctrl+A selects all,
//! a drag that starts on a chip selects instead of copying the chip, and the
//! selection ends with the note.

use super::*;

/// The usual window size.
const SIZE: [f32; 2] = [640.0, 360.0];
/// Inside "Globs" of the sample note, before " of Ectoplasm".
const BEFORE_OF: [f32; 2] = [300.0, 195.0];
/// After "first." on the sample note's last line.
const AFTER_FIRST: [f32; 2] = [372.0, 243.0];
/// The item chip of [`CHIP_BODY`].
const ITEM_CHIP: [f32; 2] = [240.0, 152.0];
/// The right end of the same chip, past the drag threshold.
const ITEM_CHIP_END: [f32; 2] = [330.0, 152.0];
/// A note body that starts with an item chip.
const CHIP_BODY: &str = "[&AvoJTQAA] and more text here";

/// The mouse at `position` with the left button `down`.
fn mouse(position: [f32; 2], down: bool) -> FrameInput {
    FrameInput {
        mouse_position: position,
        left_button_down: down,
        ..FrameInput::default()
    }
}

/// A press at `from`, a drag to `to` and the release there.
fn drag_select(from: [f32; 2], to: [f32; 2]) -> Vec<FrameInput> {
    let mut steps = vec![mouse(from, false), mouse(from, true)];
    for step in 1..=4_u8 {
        let fraction = f32::from(step) / 4.0;
        let at = [
            from[0] + (to[0] - from[0]) * fraction,
            from[1] + (to[1] - from[1]) * fraction,
        ];
        steps.push(mouse(at, true));
    }
    steps.push(mouse(to, false));
    steps
}

/// A click at `position`, released there.
fn click_at(position: [f32; 2]) -> Vec<FrameInput> {
    vec![
        mouse(position, false),
        mouse(position, true),
        mouse(position, false),
    ]
}

/// Ctrl and the key `key` pressed with the mouse resting at `position`.
fn ctrl_key(key: u32, position: [f32; 2]) -> FrameInput {
    FrameInput {
        mouse_position: position,
        control: true,
        keys: vec![key],
        ..FrameInput::default()
    }
}

/// Runs `steps` on `memo` in a focused window and returns the clipboard
/// afterwards.
fn run(memo: &mut MemoWindow, steps: &[FrameInput]) -> Option<String> {
    let mut harness = UiHarness::new(SIZE);
    let mut clipboard = None;
    let mut draw_frame = |ui: &Ui| {
        draw(ui, memo, SIZE, ThemeKind::Dark);
        clipboard = ui.clipboard_text();
    };
    for step in std::iter::repeat_n(&FrameInput::default(), 2).chain(steps) {
        harness.step(step, &mut draw_frame);
    }
    harness.step(&FrameInput::default(), &mut draw_frame);
    clipboard
}

/// The sample with its first note shown.
fn sample() -> (MemoWindow, StorageWorker, tempfile::TempDir) {
    let document = sample_document();
    let ui_state = selected_first(&document);
    memo_window(document, ui_state)
}

/// The sample with the first note's body replaced by `body`.
fn sample_with_body(body: &str) -> (MemoWindow, StorageWorker, tempfile::TempDir) {
    let mut document = sample_document();
    if let Some(Node::Note(first)) = document.root.first_mut() {
        first.body = body.to_owned();
    }
    let ui_state = selected_first(&document);
    memo_window(document, ui_state)
}

#[test]
fn dragging_selects_and_ctrl_c_copies_the_text_as_shown() {
    let (mut memo, _worker, _directory) = sample();
    let mut steps = drag_select(BEFORE_OF, AFTER_FIRST);
    steps.push(ctrl_key(0x43, AFTER_FIRST));
    let clipboard = run(&mut memo, &steps);
    let expected = "of Ectoplasm\n[ ] Mystic Coins\nCheck the wiki first.";
    assert_eq!(memo.note_view.selected_text().as_deref(), Some(expected));
    assert_eq!(clipboard.as_deref(), Some(expected), "Ctrl+C copies (Q554)");
    assert!(memo.takes_copy_key(), "the game does not get Ctrl+C now");
}

#[test]
fn a_click_without_drag_and_another_note_select_nothing() {
    let (mut memo, _worker, _directory) = sample();
    run(&mut memo, &drag_select(BEFORE_OF, AFTER_FIRST));
    assert!(memo.note_view.has_selection());
    run(&mut memo, &click_at([330.0, 219.0]));
    assert!(!memo.note_view.has_selection(), "a click ends it (Q553)");
    assert!(!memo.takes_copy_key(), "the game keeps Ctrl+C");

    run(&mut memo, &drag_select(BEFORE_OF, AFTER_FIRST));
    let dailies = memo.document.root.get(1).map(Node::id).expect("Dailies");
    memo.apply(&WindowAction::Select(dailies), Language::English);
    run(&mut memo, &[]);
    assert!(
        !memo.note_view.has_selection(),
        "another note ends it (Q553)"
    );
}

#[test]
fn shift_click_extends_and_ctrl_a_selects_the_whole_note() {
    let (mut memo, _worker, _directory) = sample();
    let mut steps = click_at(BEFORE_OF);
    steps.push(FrameInput {
        shift: true,
        ..mouse(AFTER_FIRST, false)
    });
    steps.push(FrameInput {
        shift: true,
        ..mouse(AFTER_FIRST, true)
    });
    steps.push(mouse(AFTER_FIRST, false));
    run(&mut memo, &steps);
    assert_eq!(
        memo.note_view.selected_text().as_deref(),
        Some("of Ectoplasm\n[ ] Mystic Coins\nCheck the wiki first."),
        "Shift+click extends from the click (Q555)"
    );
    run(&mut memo, &[ctrl_key(0x41, [500.0, 300.0])]);
    assert_eq!(
        memo.note_view.selected_text().as_deref(),
        Some("Ectos\n\n[x] 250 Globs of Ectoplasm\n[ ] Mystic Coins\nCheck the wiki first."),
        "Ctrl+A selects all (Q555)"
    );
}

#[test]
fn a_drag_from_a_chip_selects_while_a_click_still_copies_it() {
    let (mut memo, _worker, _directory) = sample_with_body(CHIP_BODY);
    let clipboard = run(&mut memo, &drag_select(ITEM_CHIP, ITEM_CHIP_END));
    // Released on the chip itself: still a drag, so no click (Q551).
    assert!(memo.note_view.has_selection(), "the drag selects (Q551)");
    assert_ne!(
        clipboard.as_deref(),
        Some("[&AvoJTQAA]"),
        "and does not copy the chip"
    );
    let clipboard = run(&mut memo, &click_at(ITEM_CHIP));
    assert_eq!(clipboard.as_deref(), Some("[&AvoJTQAA]"), "a click copies");
}

#[test]
fn editing_ends_the_selection() {
    let (mut memo, _worker, _directory) = sample();
    run(&mut memo, &drag_select(BEFORE_OF, AFTER_FIRST));
    memo.apply(&WindowAction::StartEditing, Language::English);
    run(&mut memo, &[]);
    assert!(!memo.note_view.has_selection(), "Q553");
    assert!(!memo.takes_select_all_key(), "Ctrl+A belongs to the editor");
}
