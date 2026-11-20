//! Tests of the `Stack` layout helper (ROADMAP R148): one gap between the
//! visible edges of neighboring elements, whatever ImGui's item spacing is.
//!
//! Run from WSL with `cargo xtask ui-test` (Windows only; the ImGui harness
//! needs nexus-rs' ImGui).
#![cfg(windows)]

use std::cell::RefCell;

use nexus::imgui::{Condition, Ui, Window};
use trailkit_ui::{
    drawing::style_scope::StyleScope,
    layout::stack::{Stack, StackLayout},
    theme::{Theme, ThemeKind},
};
use trailkit_ui_testing::harness::{FrameInput, UiHarness};

/// A screen rectangle as `[min, max]`.
type Rect = [[f32; 2]; 2];

/// Draws `content` in a themed window for a few frames (ImGui settles item
/// rects after the first) and returns the rectangles it recorded last.
fn record(content: impl Fn(&Ui, &Theme, &RefCell<Vec<Rect>>)) -> Vec<Rect> {
    let theme = ThemeKind::Dark.theme();
    let rects = RefCell::new(Vec::new());
    let mut harness = UiHarness::new([400.0, 300.0]);
    for _ in 0..3 {
        rects.borrow_mut().clear();
        harness.step(&FrameInput::default(), |ui: &Ui| {
            let _style = StyleScope::push(ui, &theme);
            Window::new("window")
                .position([0.0, 0.0], Condition::Always)
                .size([400.0, 300.0], Condition::Always)
                .build(ui, || content(ui, &theme, &rects));
        });
    }
    rects.into_inner()
}

/// Adds a button labelled `label` to `stack` and records its rectangle.
fn add_button(ui: &Ui, stack: &mut StackLayout<'_>, rects: &RefCell<Vec<Rect>>, label: &str) {
    stack.add(|| {
        ui.button(label);
        rects
            .borrow_mut()
            .push([ui.item_rect_min(), ui.item_rect_max()]);
    });
}

#[test]
fn a_vertical_stack_puts_space_sm_between_elements() {
    let rects = record(|ui, theme, rects| {
        Stack::vertical(theme).build(ui, |stack| {
            add_button(ui, stack, rects, "First");
            add_button(ui, stack, rects, "Second");
        });
    });
    let theme = ThemeKind::Dark.theme();
    assert_eq!(rects[1][0][1] - rects[0][1][1], theme.spacing.small);
    assert_eq!(rects[1][0][0], rects[0][0][0], "left edges stay aligned");
}

#[test]
fn a_horizontal_stack_takes_the_given_gap() {
    let rects = record(|ui, theme, rects| {
        Stack::horizontal(theme)
            .gap(theme.spacing.extra_small)
            .build(ui, |stack| {
                add_button(ui, stack, rects, "First");
                add_button(ui, stack, rects, "Second");
            });
    });
    let theme = ThemeKind::Dark.theme();
    assert_eq!(rects[1][0][0] - rects[0][1][0], theme.spacing.extra_small);
    assert_eq!(rects[1][0][1], rects[0][0][1], "top edges stay aligned");
}

#[test]
fn the_gap_counts_from_the_last_item_of_an_element() {
    let rects = record(|ui, theme, rects| {
        Stack::vertical(theme).build(ui, |stack| {
            stack.add(|| {
                ui.button("Upper");
                ui.button("Lower");
                rects
                    .borrow_mut()
                    .push([ui.item_rect_min(), ui.item_rect_max()]);
            });
            add_button(ui, stack, rects, "Next");
        });
    });
    let theme = ThemeKind::Dark.theme();
    assert_eq!(rects[1][0][1] - rects[0][1][1], theme.spacing.small);
}

#[test]
fn an_empty_element_takes_no_gap() {
    let rects = record(|ui, theme, rects| {
        Stack::vertical(theme).build(ui, |stack| {
            stack.add(|| {});
            add_button(ui, stack, rects, "First");
            stack.add(|| {});
            add_button(ui, stack, rects, "Second");
        });
    });
    let theme = ThemeKind::Dark.theme();
    assert_eq!(rects[1][0][1] - rects[0][1][1], theme.spacing.small);
}

#[test]
fn a_nested_stack_counts_as_one_element() {
    let rects = record(|ui, theme, rects| {
        let start = ui.cursor_screen_pos();
        Stack::vertical(theme).build(ui, |outer| {
            outer.add(|| {
                Stack::horizontal(theme).build(ui, |row| {
                    add_button(ui, row, rects, "Left");
                    add_button(ui, row, rects, "Right");
                });
            });
            add_button(ui, outer, rects, "Below");
        });
        rects.borrow_mut().push([start, start]);
    });
    let theme = ThemeKind::Dark.theme();
    let [left, right, below, [start, _]] = rects[..] else {
        panic!("four rectangles, got {rects:?}");
    };
    assert_eq!(left[0], start, "the first element starts at the cursor");
    assert_eq!(right[0][0] - left[1][0], theme.spacing.small);
    assert_eq!(
        below[0][1] - left[1][1].max(right[1][1]),
        theme.spacing.small
    );
    assert_eq!(below[0][0], start[0]);
}
