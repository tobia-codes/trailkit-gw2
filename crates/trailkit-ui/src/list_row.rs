//! The list row of the memo trash, the help topic list and every other
//! plain list (ROADMAP R96, Q272): a full-width selectable with the
//! rounded hover and selection fill, the selection bar, a truncated title,
//! an optional text at the right end of the title line and an optional
//! detail line below it.
//!
//! Rows that carry drag and drop, indentation or a tree arrow (the memo
//! note list) are tree nodes and stay in their slice; they share only the
//! row state through [`draw_row_state`].

use std::borrow::Cow;

use nexus::imgui::{Selectable, Ui};

use crate::{
    color::Color,
    font_registry::FontScope,
    selection_bar::{draw_row_state_in, hide_square_row_highlight},
    text::TextStyle,
    text_layout::truncate_to_width,
    theme::Theme,
    tooltip::draw_tooltip,
};

/// What a list row shows.
#[derive(Debug, Clone, Copy)]
pub struct ListRow<'text> {
    /// Id of the row's selectable; unique among the rows of one list unless
    /// the caller pushed an id per row.
    pub id: &'text str,
    /// Whether the row is selected.
    pub selected: bool,
    /// The title on the first line, in `text.primary`.
    pub title: &'text str,
    /// [`TextStyle::Body`] or [`TextStyle::BodyStrong`] (a group name).
    pub title_style: TextStyle,
    /// A text at the right end of the title line in `text.secondary`, e.g.
    /// the note count of a group.
    pub trailing: Option<&'text str>,
    /// A second line below the title.
    pub detail: Option<ListRowDetail<'text>>,
}

/// The second line of a [`ListRow`].
#[derive(Debug, Clone, Copy)]
pub struct ListRowDetail<'text> {
    /// The text, shortened to fit.
    pub text: &'text str,
    /// Its color: `text.secondary`, or a status color for an urgent value.
    pub color: Color,
}

/// What happened to a row in this frame.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ListRowResponse {
    /// The row was clicked.
    pub clicked: bool,
    /// The title did not fit and was shortened; the tooltip with the full
    /// title is already drawn while the row is hovered.
    pub title_truncated: bool,
}

/// Draws `row` across the remaining content width at the cursor and returns
/// its response. After the call the row's selectable is the last item, so
/// the caller can ask [`Ui::is_item_hovered`] or open a context menu.
///
/// The row is one line high, two with a detail line.
pub fn draw_list_row(ui: &Ui, theme: &Theme, row: &ListRow<'_>) -> ListRowResponse {
    let padding = ui.clone_style().frame_padding;
    let line = ui.text_line_height();
    let lines = if row.detail.is_some() { 2.0 } else { 1.0 };
    let [left, top] = ui.cursor_screen_pos();
    let size = [
        ui.content_region_avail()[0],
        lines * line + 2.0 * padding[1],
    ];
    let clicked = {
        let _square = hide_square_row_highlight(ui);
        Selectable::new(row.id)
            .selected(row.selected)
            .size(size)
            .build(ui)
    };
    // The selectable's item rect reaches half the item spacing beyond the
    // row, where a list clips; the row state belongs on the row itself.
    let rect = [[left, top], [left + size[0], top + size[1]]];
    draw_row_state_in(ui, theme, row.selected, rect);
    let title_truncated = draw_row_texts(ui, theme, row, rect);
    if title_truncated && ui.is_item_hovered() {
        draw_tooltip(ui, theme, row.title);
    }
    ListRowResponse {
        clicked,
        title_truncated,
    }
}

/// Draws the texts of `row` inside `[min, max]`; returns whether the title
/// was shortened (`lay-truncation`).
fn draw_row_texts(ui: &Ui, theme: &Theme, row: &ListRow<'_>, [min, max]: [[f32; 2]; 2]) -> bool {
    let padding = ui.clone_style().frame_padding;
    let [left, top] = [min[0] + padding[0], min[1] + padding[1]];
    let right = max[0] - padding[0];
    let draw_list = ui.get_window_draw_list();
    let mut title_end = right;
    if let Some(trailing) = row.trailing {
        let trailing_x = right - ui.calc_text_size(trailing)[0];
        draw_list.add_text([trailing_x, top], theme.text.secondary.to_array(), trailing);
        title_end = trailing_x - theme.spacing.small;
    }
    let title = {
        let _font = FontScope::push(ui, row.title_style.font());
        let title = truncate_to_width(ui, row.title, title_end - left);
        draw_list.add_text([left, top], theme.text.primary.to_array(), &title);
        matches!(title, Cow::Owned(_))
    };
    if let Some(detail) = row.detail {
        draw_list.add_text(
            [left, top + ui.text_line_height()],
            detail.color.to_array(),
            truncate_to_width(ui, detail.text, right - left),
        );
    }
    title
}
