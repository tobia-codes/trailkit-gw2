//! Button variants shared by all addons (trailkit-ui skill §3).
//!
//! `lay-one-primary`: at most one primary button per window or dialog.

use nexus::imgui::{ImColor32, StyleColor, Ui};

use crate::{glyph_metrics::capital_middle, theme::Theme};

/// Draws a button with `label` and returns whether it was clicked. Unlike
/// `ui.button` the label is centered by its capital letters, not by the font
/// line box, in which Open Sans sits low (memo plan M85).
pub fn draw_button(ui: &Ui, label: &str) -> bool {
    draw_button_with_size(ui, label, [0.0, 0.0])
}

/// Like [`draw_button`] with an explicit size (ImGui sizing: `0.0` fits the
/// label, negative values fill the available space).
pub fn draw_button_with_size(ui: &Ui, label: &str, size: [f32; 2]) -> bool {
    let padding = ui.clone_style().frame_padding;
    let text_size = ui.calc_text_size(label);
    let fitting = [
        text_size[0] + 2.0 * padding[0],
        text_size[1] + 2.0 * padding[1],
    ];
    let size = [
        if size[0] == 0.0 { fitting[0] } else { size[0] },
        if size[1] == 0.0 { fitting[1] } else { size[1] },
    ];
    // The label is the ID only; ImGui draws the frame, the label follows.
    let _id = ui.push_id(label);
    let clicked = ui.button_with_size("##button", size);
    let [min, max] = [ui.item_rect_min(), ui.item_rect_max()];
    let middle = capital_middle().unwrap_or(text_size[1] / 2.0);
    let position = [
        min[0] + ((max[0] - min[0] - text_size[0]) / 2.0).max(padding[0]),
        ((min[1] + max[1]) / 2.0 - middle).round(),
    ];
    let color = ImColor32::from(ui.style_color(StyleColor::Text)).to_bits();
    let begin = label.as_ptr().cast::<std::ffi::c_char>();
    // The raw call works while a caller holds the window's `DrawListMut`
    // (imgui-rs allows only one at a time, e.g. in a dialog card).
    // SAFETY: called on the render thread inside a frame, so the current
    // window's draw list is valid; `begin..begin + len` is the UTF-8 label,
    // which ImGui only reads.
    unsafe {
        nexus::imgui::sys::ImDrawList_AddText_Vec2(
            nexus::imgui::sys::igGetWindowDrawList(),
            nexus::imgui::sys::ImVec2 {
                x: position[0],
                y: position[1],
            },
            color,
            begin,
            begin.add(label.len()),
        );
    }
    clicked
}

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
    draw_button_with_size(ui, label, size)
}

/// The width a button with `label` takes, for right-aligning button rows.
#[must_use]
pub fn button_width(ui: &Ui, label: &str) -> f32 {
    ui.calc_text_size(label)[0] + 2.0 * ui.clone_style().frame_padding[0]
}
