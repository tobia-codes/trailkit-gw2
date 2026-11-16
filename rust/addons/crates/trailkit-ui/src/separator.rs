//! A horizontal separator that respects the padding of the current window or
//! pane (`lay-pane-padding`).
//!
//! ImGui 1.80's `Separator` always spans the full window width, padding
//! included, so it touches the pane edges. [`draw_separator`] spans only the
//! available content width.

use crate::draw_list::window_draw_list;
use nexus::imgui::Ui;

use crate::{font_registry::FontScope, fonts::FontRole, glyph_metrics::glyph_bounds, theme::Theme};

/// A divider between sections: `spacing.medium` above and below (R83), for
/// neighbors that show their own edge, such as buttons and input fields.
pub fn draw_separator(ui: &Ui, theme: &Theme) {
    draw_line(ui, theme, theme.spacing.medium, theme.spacing.medium);
}

/// A section divider above list rows: their text keeps about `spacing.medium`
/// from the line, while the fill of a hovered or selected row stays
/// `list.item_spacing` away from it, like from the next row (R83).
pub fn draw_separator_above_rows(ui: &Ui, theme: &Theme) {
    draw_line(ui, theme, theme.spacing.medium, theme.list.item_spacing);
}

/// A section divider above running text whose first line uses `first_line_font`:
/// the empty space on top of that line counts as part of the gap, so its capital
/// letters keep `spacing.medium` from the line (R83).
pub fn draw_separator_above_text(ui: &Ui, theme: &Theme, first_line_font: FontRole) {
    let empty_top = {
        let _font = FontScope::push(ui, first_line_font);
        let leading = theme.typography.leading_for(ui.current_font_size());
        leading + glyph_bounds('H').map_or(0.0, |(top, _)| top)
    };
    let below = (theme.spacing.medium - empty_top).max(0.0);
    draw_line(ui, theme, theme.spacing.medium, below);
}

/// A compact divider inside a tight group such as a menu or a note's body:
/// `spacing.small` above and below (R83).
pub fn draw_compact_separator(ui: &Ui, theme: &Theme) {
    draw_line(ui, theme, theme.spacing.small, theme.spacing.small);
}

/// Draws a horizontal line in the `border.subtle` color across the available
/// content width with the gaps `above` and `below` it;
/// the item spacing ImGui adds around an item is taken into account. The line
/// is snapped to whole pixels: between two pixel rows it would blur into a soft
/// band twice as thick (R71, in-game test). ImGui's `AddLine` already adds half
/// a pixel, so the line goes through the whole pixel coordinate itself.
fn draw_line(ui: &Ui, theme: &Theme, above: f32, below: f32) {
    let item_spacing = ui.clone_style().item_spacing[1];
    move_cursor_down(ui, above - item_spacing);
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
    move_cursor_down(ui, below - item_spacing);
}

/// Moves the cursor down by `distance` (up if negative).
fn move_cursor_down(ui: &Ui, distance: f32) {
    let [x, y] = ui.cursor_pos();
    ui.set_cursor_pos([x, y + distance]);
}
