//! Tests of editing a note's text through the whole window: what the editor,
//! its preview and the toolbar draw while the text changes (memo M207).

use super::*;

/// The window size of these tests.
const SIZE: [f32; 2] = [720.0, 360.0];
/// Windows virtual-key code of Backspace.
const KEY_BACKSPACE: u32 = 0x08;
/// Windows virtual-key code of Delete.
const KEY_DELETE: u32 = 0x2E;
/// Windows virtual-key code of Home (start of the line).
const KEY_HOME: u32 = 0x24;
/// Windows virtual-key code of the right arrow.
const KEY_RIGHT: u32 = 0x27;
/// A line with a link whose label is edited.
const LINK_LINE: &str = "Vorher [Wiki](https://wiki.guildwars2.com) nachher";

/// Opens the editor on [`LINK_LINE`] (with the preview if `preview`), puts
/// the cursor behind `cursor_after` and presses `key` `times` times, drawing
/// a frame after each press. Returns the editor's text.
fn edit_link_line(preview: bool, cursor_after: &str, key: u32, times: usize) -> String {
    let mut document = sample_document();
    if let Some(Node::Note(first)) = document.root.first_mut() {
        first.body = LINK_LINE.to_owned();
    }
    let ui_state = UiState {
        preview_visible: preview,
        ..selected_first(&document)
    };
    let (mut memo, _worker, _directory) = memo_window(document, ui_state);
    memo.apply(&WindowAction::StartEditing, Language::English);
    if let Some(editor) = memo.editor.as_mut() {
        editor.focus_body();
    }
    let mut harness = UiHarness::new(SIZE);
    let idle = FrameInput::default();
    harness.settle(3, &idle, |ui| draw(ui, &mut memo, SIZE, ThemeKind::Dark));
    let mut press = |key: u32| {
        let input = FrameInput {
            keys: vec![key],
            ..FrameInput::default()
        };
        harness.frame(&input, |ui| draw(ui, &mut memo, SIZE, ThemeKind::Dark));
        harness.frame(&idle, |ui| draw(ui, &mut memo, SIZE, ThemeKind::Dark));
    };
    press(KEY_HOME);
    for _ in cursor_after.chars() {
        press(KEY_RIGHT);
    }
    for _ in 0..times {
        press(key);
    }
    memo.editor
        .as_ref()
        .map(|editor| editor.body_text().to_owned())
        .unwrap_or_default()
}

#[test]
fn backspace_inside_a_link_label_deletes_characters_and_keeps_drawing() {
    for preview in [false, true] {
        assert_eq!(
            edit_link_line(preview, "Vorher [Wiki", KEY_BACKSPACE, 4),
            "Vorher [](https://wiki.guildwars2.com) nachher",
            "preview {preview}"
        );
    }
}

#[test]
fn delete_inside_a_link_label_deletes_characters_and_keeps_drawing() {
    for preview in [false, true] {
        assert_eq!(
            edit_link_line(preview, "Vorher [", KEY_DELETE, 4),
            "Vorher [](https://wiki.guildwars2.com) nachher",
            "preview {preview}"
        );
    }
}
