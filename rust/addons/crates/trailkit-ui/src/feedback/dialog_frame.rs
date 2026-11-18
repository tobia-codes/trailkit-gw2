//! The card of a dialog (`overlay-dialog-layout`): centered in the current
//! window, `bg.panel` with a `border.subtle` outline and `radius.lg`.

use nexus::imgui::Ui;

use crate::{drawing::draw_list::window_draw_list, theme::Theme};

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
