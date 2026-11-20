//! Tests of `markdown_view::first_line_ink_top` (ROADMAP R148, Q525): the
//! highest ink of a document's first line, wrapped as drawing wraps it.
//!
//! Run from WSL with `cargo xtask ui-test` (Windows only; the ImGui harness
//! needs nexus-rs' ImGui).
#![cfg(windows)]

use std::cell::Cell;

use nexus::imgui::{Condition, Ui, Window};
use trailkit_ui::{
    chat_link::ChipText, chat_link::chat_link_chip::ChipContext, drawing::style_scope::StyleScope,
    markdown, markdown::markdown_view::first_line_ink_top, theme::ThemeKind,
};
use trailkit_ui_testing::harness::{FrameInput, UiHarness};

/// One chip text for every case: only the chip's width matters here.
fn chip_text(_: ChipText) -> &'static str {
    "Map point"
}

/// The first line's ink top of `source` in a window `width` wide.
fn ink_top(source: &str, width: f32) -> f32 {
    let theme = ThemeKind::Dark.theme();
    let document = markdown::parse(source);
    let measured = Cell::new(None);
    let mut harness = UiHarness::new([width, 200.0]);
    for _ in 0..2 {
        harness.step(&FrameInput::default(), |ui: &Ui| {
            let _style = StyleScope::push(ui, &theme);
            Window::new("window")
                .position([0.0, 0.0], Condition::Always)
                .size([width, 200.0], Condition::Always)
                .build(ui, || {
                    let chips = ChipContext { text: &chip_text };
                    measured.set(Some(first_line_ink_top(ui, &document, &theme, chips)));
                });
        });
    }
    let Some(top) = measured.get() else {
        panic!("the document was not measured");
    };
    top
}

#[test]
fn a_checkbox_and_a_chip_reach_above_the_capitals() {
    let text = ink_top("Wizard's Vault", 400.0);
    let task = ink_top("- [ ] Wizard's Vault", 400.0);
    let chip = ink_top("Go to [&BDgAAAA=] now", 400.0);
    assert!(task < text, "checkbox {task} not above capitals {text}");
    assert!(chip < task, "chip {chip} not above checkbox {task}");
}

#[test]
fn a_chip_wrapped_to_the_second_line_does_not_count() {
    let text = ink_top("Words words words words words", 160.0);
    let wrapped = ink_top("Words words words words words [&BDgAAAA=]", 160.0);
    let first = ink_top("[&BDgAAAA=] Words words words words words", 160.0);
    assert!(
        (wrapped - text).abs() < f32::EPSILON,
        "a chip on line two moved the first line: {wrapped} vs {text}"
    );
    assert!(first < text, "a chip on line one must count");
}
