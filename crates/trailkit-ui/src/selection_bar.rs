//! The selection bar: the brass mark of the selected row in trees and lists
//! (trailkit-ui skill §4, ROADMAP R71).
//!
//! A selected row keeps the neutral `bg.active` fill that ImGui draws; this
//! module adds a short `accent` bar with round ends on its left edge, inset
//! `list.selection_bar_inset` from the top and bottom of the row.

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
    let row_height = bottom - top;
    let inset = theme
        .list
        .selection_bar_inset
        .min((row_height - MINIMUM_BAR_HEIGHT) / 2.0)
        .max(0.0);
    ui.get_window_draw_list()
        .add_rect(
            [left, top + inset],
            [left + width, bottom - inset],
            theme.accent.base.to_array(),
        )
        .filled(true)
        .rounding(width / 2.0)
        .build();
}
