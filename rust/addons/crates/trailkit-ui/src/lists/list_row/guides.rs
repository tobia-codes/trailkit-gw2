//! The indent guides of tree rows (ROADMAP R156): like VS Code, a 1 px
//! vertical line in `border.subtle` below the chevron of every expanded
//! group, from its row to its last descendant, one per level.
//!
//! Every row at depth `level` draws the segments of its `level` ancestors
//! over its own height and the gap above it, so the segments join into one
//! line that ends with the group's last descendant.

use nexus::imgui::Ui;

use super::RowInk;
use crate::drawing::draw_list::window_draw_list;

/// Draws the guide segments of a row at depth `level` spanning `[min, max]`
/// (screen; `min[0]` is the list's left edge). Each guide runs through the
/// middle of where its ancestor draws its chevron: the frame padding plus
/// half the font size from that ancestor's indentation.
pub(super) fn draw_indent_guides(
    ui: &Ui,
    ink: &RowInk<'_>,
    level: usize,
    [min, max]: [[f32; 2]; 2],
) {
    if level == 0 {
        return;
    }
    let style = ui.clone_style();
    // Up into the gap to the row above, so the segments join.
    let top = min[1] - style.item_spacing[1];
    let chevron_middle = style.frame_padding[0] + ui.current_font_size() / 2.0;
    let color = ink.color(ink.theme.border.subtle);
    let width = ink.theme.border.width.max(1.0);
    let draw_list = window_draw_list(ui);
    for depth in 0..level {
        #[expect(
            clippy::cast_precision_loss,
            reason = "a tree is never deep enough to lose precision"
        )]
        let x = (min[0] + depth as f32 * style.indent_spacing + chevron_middle).floor();
        draw_list
            .add_rect([x, top], [x + width, max[1]], color)
            .filled(true)
            .build();
    }
}
