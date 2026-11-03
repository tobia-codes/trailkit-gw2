//! The expand arrow of tree rows: a Lucide chevron in `text.disabled`
//! instead of ImGui's filled triangle (trailkit-ui skill §4, ROADMAP R71).
//!
//! ImGui draws its triangle in the text color of the tree node. A row that
//! wants the chevron pushes a transparent text color while it pushes the
//! node (so the triangle is invisible), draws its title itself and then
//! calls [`draw_tree_arrow`] over the same spot.

use nexus::imgui::Ui;

use crate::{
    dropdown::{draw_icon_into, icon_width, packed},
    icons::Icon,
    theme::Theme,
};

/// Draws the chevron of a tree row whose top-left corner is `row_min`:
/// pointing down when `is_open`, right otherwise. It sits where ImGui puts
/// its triangle: `frame_padding` in from the row's corner, in a square of
/// the font size, centered on the row's frame height.
pub fn draw_tree_arrow(
    ui: &Ui,
    theme: &Theme,
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
    let span = [row_min[1], row_min[1] + ui.frame_height()];
    // SAFETY: called on the render thread inside a frame, where ImGui has a
    // current window and returns its draw list.
    let draw_list = unsafe { nexus::imgui::sys::igGetWindowDrawList() };
    draw_icon_into(draw_list, ui, icon, x, span, packed(theme.text.disabled));
}
