//! A horizontal separator that respects the padding of the current window or
//! pane (`lay-pane-padding`).
//!
//! ImGui 1.80's `Separator` always spans the full window width, padding
//! included, so it touches the pane edges. [`draw_separator`] spans only the
//! available content width.

use nexus::imgui::Ui;

use crate::theme::Theme;

/// Draws a horizontal line in the `border.subtle` color across the available
/// content width and advances the cursor like an item of the line's
/// thickness. The line is snapped to whole pixels: between two pixel rows it
/// would blur into a soft band twice as thick (R71, in-game test).
pub fn draw_separator(ui: &Ui, theme: &Theme) {
    let start = ui.cursor_screen_pos();
    let width = ui.content_region_avail()[0].max(1.0);
    let thickness = theme.border.width.max(1.0).round();
    ui.dummy([width, thickness]);
    let middle = start[1].round() + thickness / 2.0;
    let left = start[0].round();
    ui.get_window_draw_list()
        .add_line(
            [left, middle],
            [left + width.round(), middle],
            theme.border.subtle.to_array(),
        )
        .thickness(thickness)
        .build();
}
