//! The card of a dialog (`overlay-dialog-layout`): centered in the current
//! window, `bg.panel` with a `border.subtle` outline and `radius.lg`, and a
//! close icon at its top right (R141).

use nexus::imgui::Ui;

use crate::{
    controls::button::{draw_icon_button, icon_button_width},
    drawing::draw_list::window_draw_list,
    text::TextStyle,
    text::font_registry::FontScope,
    text::icons::Icon,
    theme::Theme,
};

/// Where a dialog card was drawn, in screen coordinates.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct DialogFrame {
    /// Top-left corner.
    pub min: [f32; 2],
    /// Bottom-right corner.
    pub max: [f32; 2],
}

impl DialogFrame {
    /// Whether `point` lies on the card.
    #[must_use]
    pub fn contains(&self, point: [f32; 2]) -> bool {
        (self.min[0]..=self.max[0]).contains(&point[0])
            && (self.min[1]..=self.max[1]).contains(&point[1])
    }
}

/// The room a dialog title leaves for the close icon at its right.
#[must_use]
pub fn dialog_close_width(ui: &Ui, theme: &Theme) -> f32 {
    icon_button_width(ui, Icon::X) + theme.spacing.small
}

/// Draws the close icon at the top right of `frame`, inside its `padding` and
/// centered on the first line of the title, and returns whether it was
/// clicked. The dialog treats that like Escape. The cursor stays where it was.
pub fn draw_dialog_close(
    ui: &Ui,
    theme: &Theme,
    frame: DialogFrame,
    padding: f32,
    tooltip: &str,
) -> bool {
    let cursor = ui.cursor_screen_pos();
    let title_height = {
        let _title = FontScope::push(ui, TextStyle::Heading3.font());
        ui.text_line_height()
    };
    ui.set_cursor_screen_pos([
        frame.max[0] - padding - icon_button_width(ui, Icon::X),
        frame.min[1] + padding + (title_height - ui.frame_height()) / 2.0,
    ]);
    let is_clicked = draw_icon_button(ui, theme, Icon::X, tooltip);
    ui.set_cursor_screen_pos(cursor);
    is_clicked
}

/// Draws an empty card of `size` centered in the current window. The dialog
/// draws its content on it afterwards.
pub fn draw_dialog_frame(ui: &Ui, theme: &Theme, size: [f32; 2]) -> DialogFrame {
    let position = ui.window_pos();
    let window = ui.window_size();
    let min = [
        position[0] + ((window[0] - size[0]) / 2.0).max(0.0),
        position[1] + ((window[1] - size[1]) / 2.0).max(0.0),
    ];
    let max = [min[0] + size[0], min[1] + size[1]];
    let draw_list = window_draw_list(ui);
    draw_list
        .add_rect(min, max, theme.surface.panel.to_array())
        .filled(true)
        .rounding(theme.radius.large)
        .build();
    draw_list
        .add_rect(min, max, theme.border.subtle.to_array())
        .thickness(theme.border.width.max(1.0))
        .rounding(theme.radius.large)
        .build();
    DialogFrame { min, max }
}
