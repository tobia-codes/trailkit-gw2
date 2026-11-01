//! Button variants shared by all addons (trailkit-ui skill §3).
//!
//! `lay-one-primary`: at most one primary button per window or dialog.

use nexus::imgui::{StyleColor, Ui};

use crate::theme::Theme;

/// Draws a primary button (accent background, `text.on_accent` label) and
/// returns whether it was clicked.
pub fn draw_primary_button(ui: &Ui, theme: &Theme, label: &str) -> bool {
    draw_primary_button_with_size(ui, theme, label, [0.0, 0.0])
}

/// Like [`draw_primary_button`] with an explicit size (ImGui sizing: `0.0`
/// fits the label), e.g. full width in a stacked button column.
pub fn draw_primary_button_with_size(ui: &Ui, theme: &Theme, label: &str, size: [f32; 2]) -> bool {
    let _colors = [
        ui.push_style_color(StyleColor::Button, theme.accent.base.to_array()),
        ui.push_style_color(StyleColor::ButtonHovered, theme.accent.hover.to_array()),
        ui.push_style_color(StyleColor::ButtonActive, theme.accent.active.to_array()),
        ui.push_style_color(StyleColor::Text, theme.text.on_accent.to_array()),
    ];
    ui.button_with_size(label, size)
}

/// The width a button with `label` takes, for right-aligning button rows.
#[must_use]
pub fn button_width(ui: &Ui, label: &str) -> f32 {
    ui.calc_text_size(label)[0] + 2.0 * ui.clone_style().frame_padding[0]
}
