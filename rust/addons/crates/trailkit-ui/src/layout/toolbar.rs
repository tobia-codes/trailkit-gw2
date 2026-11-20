//! Toolbar layout (`lay-responsive`): groups separated by a short vertical
//! line, and items aligned to the toolbar's right end; both wrap onto a new
//! line in narrow windows.

use crate::drawing::draw_list::window_draw_list;
use nexus::imgui::Ui;

use crate::theme::Theme;

/// Starts a toolbar group of `width` on the current line, after a thin
/// vertical separator (M102); in a narrow pane the group moves to a new line
/// instead, without a separator (`lay-responsive`).
pub fn begin_toolbar_group(ui: &Ui, theme: &Theme, width: f32) {
    let gap = theme.spacing.small;
    let height = ui.item_rect_size()[1];
    let right_edge = ui.window_content_region_max()[0] + ui.window_pos()[0];
    ui.same_line_with_spacing(0.0, gap);
    let [x, y] = ui.cursor_screen_pos();
    if x + 1.0 + gap + width > right_edge {
        wrap_line(ui, theme);
        return;
    }
    let inset = (height * 0.2).round();
    window_draw_list(ui)
        .add_line(
            [x + 0.5, y + inset],
            [x + 0.5, y + height - inset],
            theme.border.color.to_array(),
        )
        .thickness(1.0)
        .build();
    ui.dummy([1.0, height]);
    ui.same_line_with_spacing(0.0, gap);
}

/// Continues the toolbar line at its right end for `width` of items, at
/// least `space.sm` after the last item, wrapping onto a new line when they
/// do not fit (`lay-responsive`).
pub fn move_to_toolbar_end(ui: &Ui, theme: &Theme, width: f32) {
    let right_edge = ui.window_content_region_max()[0] + ui.window_pos()[0];
    ui.same_line_with_spacing(0.0, theme.spacing.small);
    if ui.cursor_screen_pos()[0] + width > right_edge {
        wrap_line(ui, theme);
    }
    let y = ui.cursor_screen_pos()[1];
    ui.set_cursor_screen_pos([right_edge - width, y]);
}

/// Starts a new toolbar line `space.sm` below the current one (R148).
fn wrap_line(ui: &Ui, theme: &Theme) {
    ui.new_line();
    let [x, y] = ui.cursor_pos();
    ui.set_cursor_pos([x, y + theme.spacing.small]);
}
