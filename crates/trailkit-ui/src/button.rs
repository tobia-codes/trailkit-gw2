//! Button variants shared by all addons (trailkit-ui skill §3).
//!
//! `lay-one-primary`: at most one primary button per window or dialog.

use nexus::imgui::{StyleColor, Ui};

use crate::theme::Theme;

/// Draws a primary button (accent background, `text.on_accent` label) and
/// returns whether it was clicked.
pub fn draw_primary_button(ui: &Ui, theme: &Theme, label: &str) -> bool {
    let _colors = [
        ui.push_style_color(StyleColor::Button, theme.accent.base.to_array()),
        ui.push_style_color(StyleColor::ButtonHovered, theme.accent.hover.to_array()),
        ui.push_style_color(StyleColor::ButtonActive, theme.accent.active.to_array()),
        ui.push_style_color(StyleColor::Text, theme.text.on_accent.to_array()),
    ];
    ui.button(label)
}

/// The width a button with `label` takes, for right-aligning button rows.
#[must_use]
pub fn button_width(ui: &Ui, label: &str) -> f32 {
    ui.calc_text_size(label)[0] + 2.0 * ui.clone_style().frame_padding[0]
}
