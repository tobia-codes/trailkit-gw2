//! A vertical splitter in the gap between two panes (`lay-pane-padding`):
//! invisible until hovered or dragged, then an accent line with the resize
//! cursor (Windows only).

use nexus::imgui::{MouseCursor, Ui};

use crate::theme::Theme;

/// What the user did with a splitter in this frame.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SplitterResponse {
    /// The new width of the left pane while dragging.
    pub left_width: Option<f32>,
    /// The drag ended; a good moment to save the position.
    pub released: bool,
}

/// Draws a splitter of `pane.gap` width and `height` at the cursor, between a
/// left pane of `left_width` and the pane that follows. While dragged it
/// reports the new left width, clamped to `left_range`.
pub fn draw_vertical_splitter(
    ui: &Ui,
    theme: &Theme,
    id: &str,
    left_width: f32,
    left_range: (f32, f32),
    height: f32,
) -> SplitterResponse {
    let gap = theme.pane.gap;
    let start = ui.cursor_screen_pos();
    ui.invisible_button(id, [gap, height.max(1.0)]);
    let is_dragged = ui.is_item_active();
    if ui.is_item_hovered() || is_dragged {
        ui.set_mouse_cursor(Some(MouseCursor::ResizeEW));
        let color = if is_dragged {
            theme.accent.active
        } else {
            theme.accent.hover
        };
        let line_x = start[0] + gap / 2.0;
        ui.get_window_draw_list()
            .add_line(
                [line_x, start[1]],
                [line_x, start[1] + height],
                color.to_array(),
            )
            .thickness(theme.border.width.max(1.0))
            .build();
    }
    let (minimum, maximum) = left_range;
    SplitterResponse {
        left_width: is_dragged
            .then(|| (left_width + ui.io().mouse_delta[0]).clamp(minimum, maximum.max(minimum))),
        released: ui.is_item_deactivated(),
    }
}
