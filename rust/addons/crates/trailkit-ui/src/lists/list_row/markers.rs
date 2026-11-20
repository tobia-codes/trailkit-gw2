//! The markers at the right end of a list row (ROADMAP R155): icons such as
//! memo's reset badge (M193) and unsaved-changes mark (M185), or short texts
//! such as the number of search hits in a note (M122).

use nexus::imgui::Ui;

use super::RowInk;
use crate::{
    color::Color,
    drawing::draw_list::window_draw_list,
    text::font_registry::FontScope,
    text::fonts::FontRole,
    text::glyph_metrics::{capital_middle, icon_visible_box},
    text::icons::Icon,
};

/// One marker of a list row.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum RowMarker<'text> {
    /// A Lucide icon in `color`, its middle on the title's capitals.
    Icon {
        /// The icon.
        icon: Icon,
        /// Its color, e.g. `text.secondary` or `warning`.
        color: Color,
    },
    /// A text in `text.secondary` on the title's line.
    Text(&'text str),
}

/// Draws `markers` right to left, the last one ending at `end`, `space.sm`
/// apart, for a title line whose text starts at `text_top`; `row_span` is
/// the row's top and bottom. Returns where the first marker starts (`end`
/// without markers) and the index of the marker under the mouse.
pub(super) fn draw_markers(
    ui: &Ui,
    ink: &RowInk<'_>,
    markers: &[RowMarker<'_>],
    end: f32,
    text_top: f32,
    row_span: [f32; 2],
) -> (f32, Option<usize>) {
    let mut end = end;
    let mut start = end;
    let mut hovered = None;
    for (index, marker) in markers.iter().enumerate().rev() {
        let is_hovered = match *marker {
            RowMarker::Icon { icon, color } => {
                let (left, is_hovered) =
                    draw_icon_marker(ui, ink.color(color), icon, end, text_top);
                start = left;
                is_hovered
            }
            RowMarker::Text(text) => {
                start = end - ui.calc_text_size(text)[0];
                window_draw_list(ui).add_text(
                    [start, text_top],
                    ink.color(ink.theme.text.secondary),
                    text,
                );
                ui.is_mouse_hovering_rect([start, row_span[0]], [end, row_span[1]])
            }
        };
        if is_hovered {
            hovered = Some(index);
        }
        end = start - ink.theme.spacing.small;
    }
    (start, hovered)
}

/// Draws `icon` in `color` (RGBA) ending at `end`, its middle on the middle of the
/// capitals of a title drawn at `text_top`. Returns its left edge and
/// whether the mouse is over it.
fn draw_icon_marker(ui: &Ui, color: [f32; 4], icon: Icon, end: f32, text_top: f32) -> (f32, bool) {
    let line_height = ui.text_line_height();
    let text_middle = capital_middle().unwrap_or(line_height / 2.0);
    let _icons = FontScope::push(ui, FontRole::Icons);
    let mut buffer = [0; 4];
    let glyph = icon.as_str(&mut buffer);
    let width = ui.calc_text_size(glyph)[0];
    let ink = icon_visible_box(icon.glyph(), ui.text_line_height());
    let left = (end - width).round();
    let top = (text_top + text_middle - f32::midpoint(ink[1], ink[3])).round();
    window_draw_list(ui).add_text([left, top], color, glyph);
    let is_hovered = ui.is_mouse_hovering_rect([left, top], [end, top + ui.text_line_height()]);
    (left, is_hovered)
}
