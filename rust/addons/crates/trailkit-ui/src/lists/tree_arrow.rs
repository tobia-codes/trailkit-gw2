//! The expand arrow of list rows with children: a Lucide chevron in
//! `text.disabled` instead of ImGui's filled triangle (trailkit-ui skill §4,
//! ROADMAP R71), drawn by [`super::list_row`] where ImGui would put its
//! triangle.

use nexus::imgui::Ui;

use crate::{
    controls::dropdown::{draw_icon_into, icon_width, packed},
    text::glyph_metrics::capital_middle,
    text::icons::Icon,
};

/// Draws the chevron of a tree row whose top-left corner is `row_min` in
/// `color` (RGBA, `text.disabled` faded like the row):
/// pointing down when `is_open`, right otherwise. It sits where ImGui puts
/// its triangle: `frame_padding` in from the row's corner, in a square of
/// the font size, centered on the row's frame height.
pub fn draw_tree_arrow(
    ui: &Ui,
    color: [f32; 4],
    is_open: bool,
    row_min: [f32; 2],
    frame_padding: [f32; 2],
) {
    let icon = if is_open {
        Icon::CHEVRON_DOWN
    } else {
        Icon::CHEVRON_RIGHT
    };
    let box_size = ui.current_font_size();
    let x = (row_min[0] + frame_padding[0] + (box_size - icon_width(ui)) / 2.0).round();
    // On the capitals of the row's title, drawn at the line top inside the
    // frame padding (R147).
    let middle = row_min[1] + frame_padding[1] + capital_middle().unwrap_or(box_size / 2.0);
    // SAFETY: called on the render thread inside a frame, where ImGui has a
    // current window and returns its draw list.
    let draw_list = unsafe { nexus::imgui::sys::igGetWindowDrawList() };
    let color = packed(crate::color::Color::from_array(color));
    draw_icon_into(draw_list, ui, icon, x, middle, color);
}
