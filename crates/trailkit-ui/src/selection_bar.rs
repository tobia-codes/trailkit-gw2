//! The selection bar: the brass mark of the selected row in trees and lists
//! (trailkit-ui skill §4, ROADMAP R71).
//!
//! A selected row keeps the neutral `bg.active` fill that ImGui draws; this
//! module adds a short `accent` bar with round ends on its left edge. The
//! bar takes the same share of every row's height: in a one-line row it is
//! inset `list.selection_bar_inset` from top and bottom, taller rows (the
//! two-line rows) get a proportionally longer bar.
//!
//! ImGui 1.80 clamps the rounding of a rectangle to half its width minus one
//! pixel, so a 3 px bar drawn with `rounding` stays square; the round ends
//! are drawn as circles instead.

use nexus::imgui::Ui;

use crate::theme::Theme;

/// The smallest bar height in pixels, so very small text sizes still show a
/// visible mark.
const MINIMUM_BAR_HEIGHT: f32 = 4.0;

/// Draws the selection bar on the left edge of the last drawn item (the
/// row's tree node or selectable). Call it right after that item.
pub fn draw_selection_bar(ui: &Ui, theme: &Theme) {
    let [left, top] = ui.item_rect_min();
    let bottom = ui.item_rect_max()[1];
    let width = theme.list.selection_bar_width;
    let [bar_top, bar_bottom] = bar_span(
        top,
        bottom,
        ui.frame_height(),
        theme.list.selection_bar_inset,
    );
    let radius = width / 2.0;
    let color = theme.accent.base.to_array();
    let draw_list = ui.get_window_draw_list();
    draw_list
        .add_rect(
            [left, bar_top + radius],
            [left + width, bar_bottom - radius],
            color,
        )
        .filled(true)
        .build();
    for center_y in [bar_top + radius, bar_bottom - radius] {
        draw_list
            .add_circle([left + radius, center_y], radius, color)
            .filled(true)
            .num_segments(12)
            .build();
    }
}

/// The top and bottom of the bar in a row from `top` to `bottom`: the bar
/// covers the same share of the row as it does of a one-line row
/// (`line_height` high, `inset` gap above and below), centered, and is at
/// least [`MINIMUM_BAR_HEIGHT`] high.
fn bar_span(top: f32, bottom: f32, line_height: f32, inset: f32) -> [f32; 2] {
    let row_height = bottom - top;
    let share = if line_height > 0.0 {
        ((line_height - 2.0 * inset) / line_height).clamp(0.0, 1.0)
    } else {
        1.0
    };
    let height = (row_height * share).max(MINIMUM_BAR_HEIGHT).min(row_height);
    let bar_top = top + (row_height - height) / 2.0;
    [bar_top, bar_top + height]
}

#[cfg(test)]
mod tests {
    use super::bar_span;

    #[test]
    fn a_one_line_row_keeps_the_inset() {
        assert_eq!(bar_span(10.0, 36.0, 26.0, 6.0), [16.0, 30.0]);
    }

    #[test]
    fn a_taller_row_gets_the_same_share() {
        let [top, bottom] = bar_span(0.0, 52.0, 26.0, 6.0);
        assert!((bottom - top - 28.0).abs() < 1e-4);
        assert!((top - 12.0).abs() < 1e-4);
    }
}
