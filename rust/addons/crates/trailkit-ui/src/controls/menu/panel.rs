//! A menu drawn as a panel instead of a popup (ROADMAP R155-1), e.g. memo's
//! window menu (M166): taller rows over the panel's width, icons centered
//! in a square, and a selected row that looks like a selected list row.

use nexus::imgui::{StyleVar, Ui};

use super::{MenuItem, draw_menu_row_label};
use crate::{
    drawing::draw_list::window_draw_list,
    layout::pane::draw_pane,
    layout::separator::draw_separator,
    lists::selection_bar::draw_row_state,
    text::font_registry::FontScope,
    text::fonts::FontRole,
    text::glyph_metrics::{capital_middle, icon_visible_box},
    theme::Theme,
};

/// One entry of a menu panel.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MenuEntry<'text> {
    /// A row to choose.
    Item(MenuItem<'text>),
    /// A divider that sets the rows below apart.
    Separator,
}

/// Draws a menu panel `id` at the cursor with `entries` and returns the
/// index (into `entries`) of the row clicked this frame. The panel is as
/// wide as its widest row with padding, at least `minimum_width` and at most
/// `maximum_width`.
///
/// A panel row shows its icon, label, shortcut, tone and selection; check
/// marks and disabled rows exist only in popups so far (debug builds assert
/// it).
pub fn draw_menu_panel(
    ui: &Ui,
    theme: &Theme,
    id: &str,
    [minimum_width, maximum_width]: [f32; 2],
    entries: &[MenuEntry<'_>],
) -> Option<usize> {
    let width = panel_width(ui, theme, entries, minimum_width).min(maximum_width);
    let mut chosen = None;
    draw_pane(ui, theme, id, [width, 0.0], || {
        let _spacing = ui.push_style_var(StyleVar::ItemSpacing([0.0, theme.spacing.extra_small]));
        for (index, entry) in entries.iter().enumerate() {
            match entry {
                MenuEntry::Separator => draw_separator(ui, theme),
                MenuEntry::Item(item) => {
                    if draw_panel_row(ui, theme, item) {
                        chosen = Some(index);
                    }
                }
            }
        }
    });
    chosen
}

/// The side of the square the rows' icons are centered in.
fn icon_square_side(ui: &Ui) -> f32 {
    let _icons = FontScope::push(ui, FontRole::Icons);
    ui.text_line_height()
}

/// The width of the panel: `minimum`, or the widest row with its padding.
fn panel_width(ui: &Ui, theme: &Theme, entries: &[MenuEntry<'_>], minimum: f32) -> f32 {
    let gap = theme.spacing.small;
    let widest_row = entries
        .iter()
        .filter_map(|entry| match entry {
            MenuEntry::Item(item) => Some(item),
            MenuEntry::Separator => None,
        })
        .map(|item| {
            let shortcut = item
                .shortcut
                .map_or(0.0, |text| 2.0 * gap + ui.calc_text_size(text)[0]);
            ui.calc_text_size(item.label)[0] + shortcut
        })
        .fold(0.0, f32::max);
    (widest_row + icon_square_side(ui) + 3.0 * gap + 2.0 * theme.spacing.small).max(minimum)
}

/// Draws one row over the available width: icon, label and the shortcut at
/// its right end; a selected row looks like a selected list row. Returns
/// whether it was clicked.
fn draw_panel_row(ui: &Ui, theme: &Theme, item: &MenuItem<'_>) -> bool {
    debug_assert!(
        !item.checked && item.is_enabled(),
        "menu panel rows have no check mark or disabled state yet"
    );
    let width = ui.content_region_avail()[0];
    let height = ui.frame_height() + theme.spacing.extra_small;
    let clicked = ui.invisible_button(item.label, [width, height]);
    draw_row_state(ui, theme, item.selected);
    let [left, top] = ui.item_rect_min();
    let right = ui.item_rect_max()[0];
    let middle = top + height / 2.0;
    let gap = theme.spacing.small;
    let square = icon_square_side(ui);
    let color = item.color(theme);
    if let Some(icon) = item.icon {
        let _icons = FontScope::push(ui, FontRole::Icons);
        let ink = icon_visible_box(icon.glyph(), square);
        let position = [
            left + gap + (square - (ink[2] - ink[0])) / 2.0 - ink[0],
            middle - (ink[1] + ink[3]) / 2.0,
        ];
        let mut buffer = [0; 4];
        window_draw_list(ui).add_text(position, color.to_array(), icon.as_str(&mut buffer));
    }
    let text_y = middle - capital_middle().unwrap_or(ui.text_line_height() / 2.0);
    draw_menu_row_label([left + 2.0 * gap + square, text_y], color, item.label);
    if let Some(shortcut) = item.shortcut {
        draw_menu_row_label(
            [right - gap - ui.calc_text_size(shortcut)[0], text_y],
            theme.text.secondary,
            shortcut,
        );
    }
    clicked && item.is_enabled()
}
