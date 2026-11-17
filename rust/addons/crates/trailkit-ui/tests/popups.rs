//! Tests of the popup bookkeeping that lets Escape close a menu before its
//! window (memo plan M195).
//!
//! Run from WSL with `cargo xtask ui-test` (Windows only; the ImGui harness
//! needs nexus-rs' ImGui).
#![cfg(windows)]

use nexus::imgui::{Condition, Ui, Window};
use trailkit_ui::{
    controls::menu::{close_open_popups, draw_popup, popup_drawn_this_frame},
    theme::Theme,
};
use trailkit_ui_testing::harness::{FrameInput, UiHarness};

/// Draws a window with the popup `menu`, opened when `open` is set and
/// closed beforehand when `close` is set. Returns whether the popup content
/// was drawn and whether the bookkeeping reports an open popup.
fn draw(ui: &Ui, open: bool, close: bool) -> (bool, bool) {
    let theme = Theme::default();
    let mut drawn = false;
    if close {
        close_open_popups(ui);
    }
    Window::new("window")
        .position([0.0, 0.0], Condition::FirstUseEver)
        .size([200.0, 200.0], Condition::Always)
        .build(ui, || {
            if open {
                ui.open_popup("menu");
            }
            draw_popup(ui, &theme, "menu", || {
                drawn = true;
                ui.text("entry");
            });
        });
    (drawn, popup_drawn_this_frame(ui))
}

#[test]
fn reports_an_open_popup_and_closes_it_on_request() {
    let mut harness = UiHarness::new([200.0, 200.0]);
    let input = FrameInput::default();
    let mut result = (false, false);

    harness.step(&input, |ui| result = draw(ui, false, false));
    assert_eq!(result, (false, false), "no popup at first");

    harness.step(&input, |ui| result = draw(ui, true, false));
    assert_eq!(result, (true, true), "an open popup is reported");

    harness.step(&input, |ui| result = draw(ui, false, true));
    assert!(!result.1, "a popup closed by request is not reported");

    harness.step(&input, |ui| result = draw(ui, false, false));
    assert_eq!(result, (false, false), "and stays closed");
}
