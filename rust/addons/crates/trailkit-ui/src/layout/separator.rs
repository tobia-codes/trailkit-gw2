//! The one horizontal divider (ROADMAP R148): `space.sm` above and below,
//! measured to the visible edges of its neighbors, across the content width
//! of the current window or pane (`lay-pane-padding`).
//!
//! Below the line, text is measured from the top of its line box, as in CSS:
//! the empty space above the letters is part of the text, so the gap stays
//! the same fixed `space.sm` whatever the font or first line (Q528). ImGui
//! 1.80's `Separator` spans the full window width, padding included, so it
//! is never used.

use crate::drawing::draw_list::window_draw_list;
use nexus::imgui::Ui;

use crate::theme::Theme;

/// Draws the divider below the last item: `space.sm` from its edge to the
/// line and from the line to the next item's edge, such as a button, a field
/// or the fill of a list row.
pub fn draw_separator(ui: &Ui, theme: &Theme) {
    draw_line(ui, theme);
}

/// Draws the line `space.sm` below the last item and leaves the cursor
/// `space.sm` under it; ImGui's item spacing after each item is taken into
/// account.
fn draw_line(ui: &Ui, theme: &Theme) {
    let item_spacing = ui.clone_style().item_spacing[1];
    move_cursor_down(ui, theme.spacing.small - item_spacing);
    draw_rule(ui, theme);
    move_cursor_down(ui, theme.spacing.small - item_spacing);
}

/// Draws the line itself at the cursor in `border.subtle`, across the
/// available content width, as one item as tall as the line. It is snapped
/// to whole pixels: between two pixel rows it would blur into a soft band
/// twice as thick (R71, in-game test). ImGui's `AddLine` already adds half a
/// pixel, so the line goes through the whole pixel coordinate itself.
pub(crate) fn draw_rule(ui: &Ui, theme: &Theme) {
    let start = ui.cursor_screen_pos();
    let width = ui.content_region_avail()[0].max(1.0);
    let thickness = theme.border.width.max(1.0).round();
    ui.dummy([width, thickness]);
    let middle = start[1].round() + ((thickness - 1.0) / 2.0).floor();
    let left = start[0].round();
    window_draw_list(ui)
        .add_line(
            [left, middle],
            [left + width.round(), middle],
            theme.border.subtle.to_array(),
        )
        .thickness(thickness)
        .build();
}

/// Moves the cursor down by `distance` (up if negative).
fn move_cursor_down(ui: &Ui, distance: f32) {
    let [x, y] = ui.cursor_pos();
    ui.set_cursor_pos([x, y + distance]);
}
