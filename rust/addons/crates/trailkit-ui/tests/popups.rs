//! Tests of the popup bookkeeping that lets Escape close a menu before its
//! window (memo plan M195).
//!
//! Run from WSL with `cargo xtask ui-test` (Windows only; the ImGui harness
//! needs nexus-rs' ImGui).
#![cfg(windows)]

use std::cell::Cell;

use nexus::imgui::{Condition, Ui, Window};
use trailkit_ui::{
    controls::menu::{
        close_open_popups, draw_menu_item, draw_popup, menu_row_highlight_rect,
        popup_drawn_this_frame,
    },
    drawing::style_scope::StyleScope,
    theme::{Theme, ThemeKind},
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

/// A highlighted row keeps the same distance to every edge of its menu.
#[test]
fn menu_rows_have_the_same_gap_on_every_side() {
    let theme = ThemeKind::Dark.theme();
    let gaps = Cell::new(None);
    let mut harness = UiHarness::new([300.0, 200.0]);
    let mut frame = |open: bool| {
        harness.step(&FrameInput::default(), |ui: &Ui| {
            let _style = StyleScope::push(ui, &theme);
            Window::new("window")
                .position([0.0, 0.0], Condition::Always)
                .size([300.0, 200.0], Condition::Always)
                .build(ui, || {
                    if open {
                        ui.open_popup("menu");
                    }
                    draw_popup(ui, &theme, "menu", || {
                        let position = ui.window_pos();
                        let size = ui.window_size();
                        draw_menu_item(ui, &theme, "First", false);
                        let [first, _] = menu_row_highlight_rect(ui, &theme);
                        draw_menu_item(ui, &theme, "Last", false);
                        let [_, last] = menu_row_highlight_rect(ui, &theme);
                        gaps.set(Some([
                            first[0] - position[0],
                            position[0] + size[0] - last[0],
                            first[1] - position[1],
                            position[1] + size[1] - last[1],
                        ]));
                    });
                });
        });
    };
    frame(true);
    for _ in 0..3 {
        frame(false);
    }
    let gaps = gaps.get().expect("the menu was drawn");
    let expected = theme.spacing.small + theme.border.width;
    for gap in gaps {
        assert!((gap - expected).abs() < 0.5, "uneven gaps {gaps:?}");
    }
}
