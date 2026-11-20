//! The indent guides of tree rows (ROADMAP R156): like VS Code, a 1 px
//! vertical line in `border.subtle` below the chevron of every expanded
//! group, from its row to its last descendant, one per level.
//!
//! Every row at depth `level` draws the segments of its `level` ancestors
//! over its own height and the gap above it, so the segments join into one
//! line that ends with the group's last descendant.

use nexus::imgui::Ui;

use super::RowInk;
use crate::{drawing::draw_list::window_draw_list, theme::Theme};

/// The screen x of the guide of tree depth `depth` in a list whose rows
/// start at `row_left`: the middle of where a row at that depth draws its
/// chevron, the frame padding plus half the font size from its indentation.
fn guide_x(ui: &Ui, row_left: f32, depth: usize) -> f32 {
    let style = ui.clone_style();
    let chevron_middle = style.frame_padding[0] + ui.current_font_size() / 2.0;
    #[expect(
        clippy::cast_precision_loss,
        reason = "a tree is never deep enough to lose precision"
    )]
    let indentation = depth as f32 * style.indent_spacing;
    (row_left + indentation + chevron_middle).floor()
}

/// Where the fill of a row at depth `level` starts (R156): `space.xs` right
/// of the nearest guide, so no fill covers a line; a top-level row fills
/// from `row_left`.
pub(super) fn fill_left(ui: &Ui, theme: &Theme, row_left: f32, level: usize) -> f32 {
    match level.checked_sub(1) {
        Some(parent_depth) => {
            guide_x(ui, row_left, parent_depth) + guide_width(theme) + theme.spacing.extra_small
        }
        None => row_left,
    }
}

/// The width of a guide line.
fn guide_width(theme: &Theme) -> f32 {
    theme.border.width.max(1.0)
}

/// Draws the guide segments of a row at depth `level` spanning `[min, max]`
/// (screen; `min[0]` is the list's left edge), one per ancestor.
pub(super) fn draw_indent_guides(
    ui: &Ui,
    ink: &RowInk<'_>,
    level: usize,
    [min, max]: [[f32; 2]; 2],
) {
    // Up into the gap to the row above, so the segments join.
    let top = min[1] - ui.clone_style().item_spacing[1];
    let color = ink.color(ink.theme.border.subtle);
    let width = guide_width(ink.theme);
    let draw_list = window_draw_list(ui);
    for depth in 0..level {
        let x = guide_x(ui, min[0], depth);
        draw_list
            .add_rect([x, top], [x + width, max[1]], color)
            .filled(true)
            .build();
    }
}
