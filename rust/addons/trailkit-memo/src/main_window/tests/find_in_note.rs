//! Tests of find in the open note driven through the window (memo M198): the
//! keys and buttons of the bar, the hits it counts, scrolling to the current
//! hit, the editor and switching notes.

use super::*;

/// The window size of these tests; the bar's buttons are placed for it.
const SIZE: [f32; 2] = [640.0, 360.0];
/// Center of the bar's "previous" arrow.
const PREVIOUS_BUTTON: [f32; 2] = [544.0, 109.0];
/// Center of the bar's "next" arrow.
const NEXT_BUTTON: [f32; 2] = [572.0, 109.0];
/// Center of the bar's close button.
const CLOSE_BUTTON: [f32; 2] = [600.0, 109.0];
/// Center of the note toolbar's `⋯` button.
const NOTE_MENU_BUTTON: [f32; 2] = [536.0, 64.0];
/// Center of the first row of the opened `⋯` menu.
const FIRST_NOTE_MENU_ROW: [f32; 2] = [470.0, 98.0];
/// Windows virtual-key code of Enter.
const KEY_ENTER: u32 = 0x0D;

/// A window and what must outlive it, with a harness to drive it.
struct Driven {
    /// The window.
    memo: MemoWindow,
    /// Runs its frames.
    harness: UiHarness,
    /// The storage worker behind it.
    _worker: StorageWorker,
    /// Its directory.
    _directory: tempfile::TempDir,
}

impl Driven {
    /// The sample with the first note selected and its body set to `body`.
    fn with_body(body: &str) -> Self {
        let mut document = sample_document();
        if let Some(Node::Note(first)) = document.root.first_mut() {
            first.body = body.to_owned();
        }
        let ui_state = selected_first(&document);
        let (memo, worker, directory) = memo_window(document, ui_state);
        Self {
            memo,
            harness: UiHarness::new(SIZE),
            _worker: worker,
            _directory: directory,
        }
    }

    /// Runs one frame with `input`.
    fn step(&mut self, input: &FrameInput) {
        let memo = &mut self.memo;
        self.harness
            .step(input, |ui| draw(ui, memo, SIZE, ThemeKind::Dark));
    }

    /// Runs a few idle frames, so focus changes and hit counts arrive.
    fn settle(&mut self) {
        for _ in 0..4 {
            self.step(&FrameInput::default());
        }
    }

    /// Opens the bar on `text` and lets it settle.
    fn find(&mut self, text: &str) {
        self.settle();
        self.memo.find_text(text);
        self.settle();
    }

    /// Clicks at `position` and lets the window settle.
    fn click(&mut self, position: [f32; 2]) {
        let at = |left_button_down| FrameInput {
            mouse_position: position,
            left_button_down,
            ..FrameInput::default()
        };
        self.step(&at(false));
        self.step(&at(false));
        self.step(&at(true));
        self.step(&at(false));
        self.settle();
    }

    /// Presses Enter, with Shift if `shift`.
    fn press_enter(&mut self, shift: bool) {
        self.step(&FrameInput {
            keys: vec![KEY_ENTER],
            shift,
            ..FrameInput::default()
        });
        self.settle();
    }

    /// Types `text` into whatever has the focus.
    fn type_text(&mut self, text: &str) {
        self.step(&FrameInput {
            characters: text.to_owned(),
            ..FrameInput::default()
        });
        self.settle();
    }

    /// The text selected in the open editor.
    fn selected_in_editor(&self) -> &str {
        self.memo
            .editor
            .as_ref()
            .map_or("", |editor| editor.selected_body_text())
    }

    /// The body of the open editor.
    fn editor_body(&self) -> &str {
        self.memo
            .editor
            .as_ref()
            .map_or("", |editor| editor.body_text())
    }
}

#[test]
fn the_bar_counts_the_hits_of_the_shown_note() {
    let mut driven = Driven::with_body("Ecto and ecto\n\nnothing\n\nECTO");
    driven.find("ecto");
    assert_eq!(driven.memo.find.hit_count, 3, "case does not matter");
    driven.memo.find_text("zzz");
    driven.settle();
    assert_eq!(driven.memo.find.hit_count, 0);
}

#[test]
fn a_phrase_with_a_space_counts_as_one_hit() {
    let mut driven = Driven::with_body("Globs of ore, globs OF gold, globs");
    driven.find("globs of");
    assert_eq!(driven.memo.find.hit_count, 2);
}

#[test]
fn a_matching_chat_link_chip_counts_as_a_hit() {
    let mut driven = Driven::with_body("[&AdsnAAA=] and [&AdsnAAA=]");
    driven.find("adsn");
    assert_eq!(driven.memo.find.hit_count, 2);
}

#[test]
fn enter_steps_forward_and_shift_enter_back_with_the_field_focused() {
    let mut driven = Driven::with_body("ecto ecto ecto");
    driven.find("ecto");
    assert!(driven.memo.find.field.is_active(), "the bar took the focus");
    driven.press_enter(false);
    assert_eq!(driven.memo.find.current, 1);
    assert!(driven.memo.find.field.is_active(), "Enter keeps the focus");
    driven.press_enter(true);
    assert_eq!(driven.memo.find.current, 0);
    driven.press_enter(true);
    assert_eq!(driven.memo.find.current, 2, "back from the first wraps");
}

#[test]
fn the_arrows_step_and_the_x_closes_the_bar() {
    let mut driven = Driven::with_body("ecto ecto");
    driven.find("ecto");
    driven.click(NEXT_BUTTON);
    assert_eq!(driven.memo.find.current, 1);
    driven.click(PREVIOUS_BUTTON);
    assert_eq!(driven.memo.find.current, 0);
    driven.click(CLOSE_BUTTON);
    assert!(!driven.memo.find.visible);
}

#[test]
fn typing_starts_again_at_the_first_hit() {
    let mut driven = Driven::with_body("ecto ecto ectoplasm");
    driven.find("ecto");
    driven.press_enter(false);
    assert_eq!(driven.memo.find.current, 1);
    driven.type_text("p");
    assert_eq!(driven.memo.find.current, 0);
    assert_eq!(driven.memo.find.hit_count, 1);
}

#[test]
fn reopening_selects_the_old_text_so_typing_replaces_it() {
    let mut driven = Driven::with_body("ecto mithril");
    driven.find("ecto");
    driven.memo.close_find();
    driven.settle();
    driven.memo.open_find();
    driven.settle();
    assert_eq!(
        driven.memo.find.field.text, "ecto",
        "the last find is offered"
    );
    driven.type_text("mithril");
    assert_eq!(driven.memo.find.field.text, "mithril");
    assert_eq!(driven.memo.find.hit_count, 1);
}

#[test]
fn the_note_menu_opens_the_bar() {
    let mut driven = Driven::with_body("ecto");
    driven.settle();
    driven.click(NOTE_MENU_BUTTON);
    driven.click(FIRST_NOTE_MENU_ROW);
    assert!(driven.memo.find.visible, "first row of the menu (M198)");
}

/// A note with `needle` in its first and last of many paragraphs.
fn long_note_with_two_hits() -> String {
    let filler: String = (1..=80).map(|line| format!("line {line}\n\n")).collect();
    format!("needle first\n\n{filler}needle last")
}

#[test]
fn stepping_scrolls_to_a_hit_outside_the_view_and_back() {
    let mut driven = Driven::with_body(&long_note_with_two_hits());
    driven.find("needle");
    assert_eq!(driven.memo.find.hit_count, 2);
    assert!(
        driven.memo.ui_state.note_scroll < 1.0,
        "the first is visible"
    );
    driven.memo.step_find(true);
    driven.settle();
    assert!(driven.memo.ui_state.note_scroll > 100.0, "down to the last");
    driven.memo.step_find(true);
    driven.settle();
    assert!(
        driven.memo.ui_state.note_scroll < 1.0,
        "wrapped to the first"
    );
}

#[test]
fn a_visible_hit_does_not_scroll_the_note() {
    let filler: String = (1..=40).map(|line| format!("line {line}\n\n")).collect();
    let mut driven = Driven::with_body(&format!("{filler}needle one\n\nneedle two\n\n{filler}"));
    driven.find("needle");
    let scroll = driven.memo.ui_state.note_scroll;
    assert!(scroll > 100.0, "the first hit was scrolled to");
    driven.memo.step_find(true);
    driven.settle();
    assert!(
        (driven.memo.ui_state.note_scroll - scroll).abs() < 1.0,
        "the next hit is already visible"
    );
}

#[test]
fn another_note_starts_at_its_first_hit() {
    let mut driven = Driven::with_body("ecto ecto");
    let Some(Node::Note(dailies)) = driven.memo.document.root.get(1) else {
        panic!("the sample's second entry is a note");
    };
    let (dailies, id) = (dailies.id, NodeId::Note(dailies.id));
    if let Some(note) = driven.memo.document.note_mut(dailies) {
        note.body = "ecto ecto ecto".to_owned();
    }
    driven.find("ecto");
    driven.press_enter(false);
    assert_eq!(driven.memo.find.current, 1);
    driven
        .memo
        .apply(&WindowAction::Select(id), Language::English);
    driven.settle();
    assert_eq!(driven.memo.find.current, 0);
    assert_eq!(driven.memo.find.hit_count, 3);
    assert!(driven.memo.find.visible, "the bar stays open");
}

/// A window editing a note with two hits of "alpha", the bar open on it.
fn finding_in_the_editor() -> Driven {
    let mut driven = Driven::with_body("alpha one\nALPHA two");
    driven.settle();
    driven
        .memo
        .apply(&WindowAction::StartEditing, Language::English);
    driven.find("alpha");
    driven
}

#[test]
fn the_editor_selects_the_current_hit() {
    let mut driven = finding_in_the_editor();
    assert_eq!(driven.memo.find.hit_count, 2);
    assert_eq!(driven.selected_in_editor(), "alpha");
    driven.memo.step_find(true);
    driven.settle();
    assert_eq!(driven.selected_in_editor(), "ALPHA");
}

#[test]
fn closing_the_bar_lets_typing_replace_the_current_hit() {
    let mut driven = finding_in_the_editor();
    driven.memo.step_find(true);
    driven.settle();
    driven.memo.close_find();
    driven.settle();
    driven.type_text("Z");
    driven.type_text("Y");
    assert_eq!(
        driven.editor_body(),
        "alpha one\nZY two",
        "the cursor stood behind the selected hit"
    );
}

#[test]
fn editing_the_text_updates_the_hits() {
    let mut driven = finding_in_the_editor();
    if let Some(editor) = driven.memo.editor.as_mut() {
        editor.focus_body();
    }
    driven.settle();
    driven.type_text("Z");
    assert_eq!(driven.editor_body(), "Z one\nALPHA two");
    assert_eq!(driven.memo.find.hit_count, 1);
}

#[test]
fn the_trash_has_no_find() {
    let mut driven = Driven::with_body("ecto");
    driven.find("ecto");
    driven.memo.window_focused = true;
    assert!(driven.memo.takes_find_key());
    assert!(driven.memo.takes_find_step_key());
    driven
        .memo
        .apply(&WindowAction::OpenTrash, Language::English);
    assert!(!driven.memo.takes_find_key(), "the game keeps Ctrl+F");
    assert!(!driven.memo.takes_find_step_key(), "and F3");
    assert!(!driven.memo.takes_escape(), "the hidden bar leaves Escape");
}

#[test]
fn f3_belongs_to_the_window_only_while_the_bar_is_open() {
    let mut driven = Driven::with_body("ecto");
    driven.settle();
    driven.memo.window_focused = true;
    assert!(!driven.memo.takes_find_step_key());
    driven.memo.find_text("ecto");
    assert!(driven.memo.takes_find_step_key());
    driven.memo.window_focused = false;
    assert!(!driven.memo.takes_find_step_key(), "not without the focus");
}

#[test]
fn find_leaves_the_list_search_alone() {
    let mut driven = Driven::with_body("ecto coins");
    driven.memo.toggle_search_by_key();
    driven.memo.search_field.text = "coins".to_owned();
    driven.settle();
    driven.memo.find_text("ecto");
    driven.settle();
    assert_eq!(driven.memo.search_field.text, "coins");
    assert!(
        driven.memo.search.current().is_some(),
        "the list stays filtered"
    );
    driven.memo.close_find();
    assert_eq!(
        driven.memo.find.field.text, "ecto",
        "kept for the next time"
    );
}
