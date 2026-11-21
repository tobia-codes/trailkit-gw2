//! The title bar moves the window; nothing else does (memo M217 follow-up,
//! user wish 2026-11-21).
//!
//! Run from WSL with `cargo xtask ui-test` (Windows only).
#![cfg(windows)]

use nexus::imgui::{Condition, Ui, Window};
use trailkit_ui::{
    drawing::style_scope::StyleScope,
    layout::title_bar::{TitleBarState, TitleBarTexts, draw_window},
    theme::ThemeKind,
};
use trailkit_ui_testing::harness::{FrameInput, UiHarness};

/// The title bar's texts.
const TEXTS: TitleBarTexts<'static> = TitleBarTexts {
    title: "Notes",
    collapse: "Collapse",
    expand: "Expand",
    close: "Close",
};

/// Drags the window's content from `from` by `by` and returns where the
/// window ends up.
fn window_position_after_drag(from: [f32; 2], by: [f32; 2]) -> [f32; 2] {
    let theme = ThemeKind::Dark.theme();
    let mut harness = UiHarness::new([600.0, 400.0]);
    let mut state = TitleBarState::default();
    let mut position = [0.0, 0.0];
    let mut draw = |ui: &Ui| {
        let _style = StyleScope::push_with_cursor_extent(ui, &theme, None);
        let mut open = true;
        let _ = draw_window(
            ui,
            &theme,
            Window::new("window")
                .position([20.0, 20.0], Condition::FirstUseEver)
                .size([300.0, 200.0], Condition::FirstUseEver),
            TEXTS,
            &[],
            &mut state,
            &mut open,
            || position = ui.window_pos(),
        );
    };
    let mouse = |at: [f32; 2], left_button_down: bool| FrameInput {
        mouse_position: at,
        left_button_down,
        ..FrameInput::default()
    };
    harness.step(&mouse(from, false), &mut draw);
    harness.step(&mouse(from, true), &mut draw);
    for step in 1..=4_u8 {
        let fraction = f32::from(step) / 4.0;
        let at = [from[0] + by[0] * fraction, from[1] + by[1] * fraction];
        harness.step(&mouse(at, true), &mut draw);
    }
    let end = [from[0] + by[0], from[1] + by[1]];
    harness.step(&mouse(end, false), &mut draw);
    harness.step(&mouse(end, false), &mut draw);
    drop(draw);
    position
}

#[test]
fn only_the_title_bar_moves_the_window() {
    assert_eq!(
        window_position_after_drag([160.0, 38.0], [60.0, 40.0]),
        [80.0, 60.0],
        "a drag on the title moves the window"
    );
    assert_eq!(
        window_position_after_drag([160.0, 150.0], [60.0, 40.0]),
        [20.0, 20.0],
        "a drag on the content leaves it in place"
    );
}
