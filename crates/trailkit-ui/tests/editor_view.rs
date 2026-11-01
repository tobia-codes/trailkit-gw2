//! Visual and interaction tests of the text editor (memo plan M65).
//!
//! Run from WSL with `cargo xtask ui-test` (Windows only; the ImGui harness
//! needs nexus-rs' ImGui). `cargo xtask ui-test --update` rewrites the
//! reference images in `tests/snapshots/`.
#![cfg(windows)]

use nexus::imgui::{Condition, Ui, Window, WindowFlags};
use trailkit_ui::{editor::view::TextEditor, style_scope::StyleScope, theme::ThemeKind};
use trailkit_ui_testing::{
    harness::{FrameInput, UiHarness},
    snapshot::assert_snapshot,
};

/// Size of the rendered images.
const SIZE: [f32; 2] = [360.0, 180.0];

/// A note that wraps and uses every highlighted construct.
const SAMPLE: &str = "# Raids\n- [x] **Ectos** kaufen und danach noch ganz viele Wörter, damit die Zeile umbricht\n- [ ] [Wiki](https://wiki.guildwars2.com) `code`";

/// Draws `editor` in a borderless window filling the image; returns the
/// window position.
fn draw(ui: &Ui, editor: &mut TextEditor, theme: ThemeKind) -> [f32; 2] {
    let theme = theme.theme();
    let _style = StyleScope::push(ui, &theme);
    let mut position = [0.0; 2];
    Window::new("editor")
        .position([0.0, 0.0], Condition::FirstUseEver)
        .size(SIZE, Condition::Always)
        .flags(WindowFlags::NO_TITLE_BAR | WindowFlags::NO_RESIZE)
        .build(ui, || {
            position = ui.window_pos();
            editor.draw(ui, &theme, "body", [0.0, 0.0]);
        });
    position
}

/// Input with the mouse at `position`, optionally holding the left button.
fn mouse(position: [f32; 2], down: bool) -> FrameInput {
    FrameInput {
        mouse_position: position,
        left_button_down: down,
        ..FrameInput::default()
    }
}

#[test]
fn wraps_and_highlights_in_both_themes() {
    let mut harness = UiHarness::new(SIZE);
    for (theme, name) in [
        (ThemeKind::Dark, "editor-dark"),
        (ThemeKind::Light, "editor-light"),
    ] {
        let mut editor = TextEditor::new(SAMPLE);
        editor.unfocus();
        let image = harness.settle(3, &FrameInput::default(), |ui| {
            draw(ui, &mut editor, theme);
        });
        assert_snapshot(env!("CARGO_MANIFEST_DIR"), name, &image);
    }
}

#[test]
fn typing_and_backspace_edit_the_text() {
    let mut harness = UiHarness::new(SIZE);
    let mut editor = TextEditor::new("Hallo");
    let idle = FrameInput::default();
    harness.settle(2, &idle, |ui| {
        draw(ui, &mut editor, ThemeKind::Dark);
    });
    let typing = FrameInput {
        characters: " Tyriä".to_owned(),
        ..FrameInput::default()
    };
    harness.frame(&typing, |ui| {
        draw(ui, &mut editor, ThemeKind::Dark);
    });
    let backspace = FrameInput {
        keys: vec![0x08],
        ..FrameInput::default()
    };
    harness.frame(&backspace, |ui| {
        draw(ui, &mut editor, ThemeKind::Dark);
    });
    harness.frame(&idle, |ui| {
        draw(ui, &mut editor, ThemeKind::Dark);
    });
    assert_eq!(editor.text(), "Hallo Tyri");
}

#[test]
fn dragging_selects_text_and_keeps_the_window_in_place() {
    let mut harness = UiHarness::new(SIZE);
    let mut editor = TextEditor::new("one two three four");
    let mut positions = Vec::new();
    let steps = [
        mouse([20.0, 20.0], false),
        mouse([20.0, 20.0], true),
        mouse([90.0, 20.0], true),
        mouse([90.0, 20.0], false),
    ];
    for input in &steps {
        harness.frame(input, |ui| {
            positions.push(draw(ui, &mut editor, ThemeKind::Dark));
        });
    }
    assert!(!editor.selection().is_empty(), "the drag selects text");
    assert!(
        positions.windows(2).all(|pair| pair[0] == pair[1]),
        "the window did not move: {positions:?}"
    );
}
