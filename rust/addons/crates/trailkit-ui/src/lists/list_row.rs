//! The one row of every list and tree (ROADMAP R96, R155): the memo note
//! list, the trash, the help topic list. A full-width selectable with the
//! rounded hover and selection fill, the selection bar, indent guides in a
//! tree (R156), an optional chevron, a truncated title with marked search
//! hits, markers and a text at the right end, and an optional detail line
//! below.
//!
//! The row draws only through the draw list after its selectable, so the
//! selectable stays the last item: the caller attaches drag and drop, a
//! context menu or an inline rename input to it.

mod guides;
mod markers;

pub use self::markers::RowMarker;

use crate::drawing::draw_list::window_draw_list;
use std::borrow::Cow;

use nexus::imgui::{Key, MouseButton, Selectable, Ui};

use self::{guides::draw_indent_guides, markers::draw_markers};
use crate::{
    color::Color,
    controls::tooltip::draw_tooltip,
    lists::selection_bar::{draw_row_state_in, hide_square_row_highlight, is_row_hovered},
    lists::tree_arrow::draw_tree_arrow,
    text::TextStyle,
    text::font_registry::FontScope,
    text::search_hits::{HitMark, mark_search_hits},
    text::text_layout::truncate_to_width,
    text::text_search::SearchQuery,
    theme::Theme,
};

/// The chevron before a row's title, for a row that has children.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum RowChevron {
    /// No chevron; the title starts where a sibling's chevron does.
    #[default]
    None,
    /// The children are hidden (`chevron-right`).
    Closed,
    /// The children are shown (`chevron-down`).
    Open,
}

/// What a list row shows.
#[derive(Debug, Clone, Copy)]
pub struct ListRow<'text> {
    /// ID of the row's selectable (start it with `##`); unique among the
    /// rows of one list unless the caller pushed an ID per row.
    pub id: &'text str,
    /// Whether the row is selected.
    pub selected: bool,
    /// The title on the first line, in `text.primary`.
    pub title: &'text str,
    /// [`TextStyle::Body`] or [`TextStyle::BodyStrong`] (a group name).
    pub title_style: TextStyle,
    /// The search whose hits are marked in the visible title (M122).
    pub title_hits: Option<&'text SearchQuery>,
    /// Whether the title is being renamed: it is left out, and the caller
    /// draws its input over [`ListRowResponse::title_rect`].
    pub renaming: bool,
    /// The depth in a tree; the row's content and its fill are indented by
    /// ImGui's indent spacing per level, with one indent guide per level in
    /// the indentation (R156). The whole width stays clickable (memo M144).
    pub level: usize,
    /// The chevron before the title.
    pub chevron: RowChevron,
    /// Markers between the title and [`Self::trailing`], left to right,
    /// `space.sm` apart, e.g. a badge or an unsaved-changes mark.
    pub markers: &'text [RowMarker<'text>],
    /// A text at the right end of the title line in `text.secondary`, e.g.
    /// the note count of a group.
    pub trailing: Option<&'text str>,
    /// A second line below the title.
    pub detail: Option<ListRowDetail<'text>>,
}

impl<'text> ListRow<'text> {
    /// An unselected one-line row `id` with a `Body` title and nothing else.
    #[must_use]
    pub const fn new(id: &'text str, title: &'text str) -> Self {
        Self {
            id,
            selected: false,
            title,
            title_style: TextStyle::Body,
            title_hits: None,
            renaming: false,
            level: 0,
            chevron: RowChevron::None,
            markers: &[],
            trailing: None,
            detail: None,
        }
    }
}

/// The second line of a [`ListRow`].
#[derive(Debug, Clone, Copy)]
pub struct ListRowDetail<'text> {
    /// The text, shortened to fit.
    pub text: &'text str,
    /// Its color: `text.secondary`, or a status color for an urgent value.
    pub color: Color,
}

/// What happened to a row in this frame, and where its parts are.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ListRowResponse {
    /// The row was clicked (on release).
    pub clicked: bool,
    /// The mouse is on the row itself, not on the gap to the next row that
    /// ImGui counts to the selectable; ask this instead of the item.
    pub hovered: bool,
    /// The chevron was clicked, or Left/Right closed/opened the focused row.
    pub chevron_clicked: bool,
    /// The title did not fit and was shortened; the tooltip with the full
    /// title is already drawn while the row is hovered.
    pub title_truncated: bool,
    /// The marker (index into [`ListRow::markers`]) under the mouse while the
    /// row is hovered; the caller draws its tooltip.
    pub hovered_marker: Option<usize>,
    /// The row (screen), without the half item spacing ImGui adds around
    /// the selectable.
    pub row_rect: [[f32; 2]; 2],
    /// Where the row's content starts after the indentation (screen x).
    pub content_left: f32,
    /// Where the title is drawn (screen): from its start to where markers or
    /// the trailing text begin, the row's height.
    pub title_rect: [[f32; 2]; 2],
}

/// Draws `row` across the remaining content width at the cursor and returns
/// its response. After the call the row's selectable is the last item, so
/// the caller can attach a context menu or drag and drop; whether the row is
/// hovered is [`ListRowResponse::hovered`].
///
/// The row is one line high, two with a detail line.
pub fn draw_list_row(ui: &Ui, theme: &Theme, row: &ListRow<'_>) -> ListRowResponse {
    let style = ui.clone_style();
    let padding = style.frame_padding;
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
    #[expect(
        clippy::cast_precision_loss,
        reason = "a tree is never deep enough to lose precision"
    )]
    let content_left = left + row.level as f32 * style.indent_spacing;
    // The fill leaves the guides free, `space.xs` right of the nearest (R156).
    let fill_left = guides::fill_left(ui, theme, left, row.level);
    draw_row_state_in(ui, theme, row.selected, rect, fill_left);
    let is_hovered = is_row_hovered(ui, rect);
    // Content follows ImGui's style alpha, e.g. a dragged row (memo M106);
    // the fill keeps the row's place visible.
    let ink = RowInk {
        theme,
        alpha: style.alpha,
    };
    draw_indent_guides(ui, &ink, row.level, rect);
    let texts = draw_row_texts(ui, &ink, row, rect, content_left, is_hovered);
    if texts.title_truncated && is_hovered {
        draw_tooltip(ui, theme, row.title);
    }
    ListRowResponse {
        clicked,
        hovered: is_hovered,
        chevron_clicked: is_chevron_clicked(ui, row.chevron, content_left, is_hovered),
        title_truncated: texts.title_truncated,
        hovered_marker: texts.hovered_marker,
        row_rect: rect,
        content_left,
        title_rect: texts.title_rect,
    }
}

/// Whether the chevron of a row was used this frame: clicked where ImGui's
/// tree arrow takes clicks (the font size plus twice the frame padding from
/// `content_left`), or Left/Right on the focused row closing/opening it.
fn is_chevron_clicked(ui: &Ui, chevron: RowChevron, content_left: f32, is_hovered: bool) -> bool {
    if chevron == RowChevron::None {
        return false;
    }
    let padding = ui.clone_style().frame_padding[0];
    let arrow_end = content_left + ui.current_font_size() + 2.0 * padding;
    let mouse_x = ui.io().mouse_pos[0];
    let clicked = is_hovered
        && ui.is_mouse_clicked(MouseButton::Left)
        && (content_left..arrow_end).contains(&mouse_x);
    let key = match chevron {
        RowChevron::Open => Key::LeftArrow,
        RowChevron::Closed | RowChevron::None => Key::RightArrow,
    };
    clicked || (ui.is_item_focused() && ui.is_key_pressed(key))
}

/// The colors of a row's content: the theme's, faded by ImGui's style alpha.
pub(super) struct RowInk<'theme> {
    /// The active theme.
    pub(super) theme: &'theme Theme,
    /// ImGui's style alpha, below 1 while a row is faded.
    pub(super) alpha: f32,
}

impl RowInk<'_> {
    /// `color` faded by the style alpha.
    pub(super) fn color(&self, color: Color) -> [f32; 4] {
        let [red, green, blue, alpha] = color.to_array();
        [red, green, blue, alpha * self.alpha]
    }
}

/// What drawing a row's texts found out.
struct RowTexts {
    /// Whether the title was shortened.
    title_truncated: bool,
    /// The marker under the mouse, while the row is hovered.
    hovered_marker: Option<usize>,
    /// Where the title is drawn.
    title_rect: [[f32; 2]; 2],
}

/// Draws the chevron, title, markers, trailing text and detail line of
/// `row` inside `[min, max]`, its content starting at `content_left`.
fn draw_row_texts(
    ui: &Ui,
    ink: &RowInk<'_>,
    row: &ListRow<'_>,
    [min, max]: [[f32; 2]; 2],
    content_left: f32,
    is_hovered: bool,
) -> RowTexts {
    let theme = ink.theme;
    let padding = ui.clone_style().frame_padding;
    let top = min[1] + padding[1];
    let right = max[0] - padding[0];
    let draw_list = window_draw_list(ui);
    let mut title_end = right;
    if let Some(trailing) = row.trailing {
        let trailing_x = right - ui.calc_text_size(trailing)[0];
        draw_list.add_text([trailing_x, top], ink.color(theme.text.secondary), trailing);
        title_end = trailing_x - theme.spacing.small;
    }
    let row_span = [min[1], max[1]];
    let (markers_start, hovered_marker) =
        draw_markers(ui, ink, row.markers, title_end, top, row_span);
    if !row.markers.is_empty() {
        title_end = markers_start - theme.spacing.small;
    }
    // The chevron is drawn in the title's font, centered on its capitals.
    let (title_x, title_truncated) = {
        let _font = FontScope::push(ui, row.title_style.font());
        let title_x = if row.chevron == RowChevron::None {
            content_left + padding[0]
        } else {
            draw_tree_arrow(
                ui,
                ink.color(theme.text.disabled),
                row.chevron == RowChevron::Open,
                [content_left, min[1]],
                padding,
            );
            content_left + ui.current_font_size() + 2.0 * padding[0]
        };
        let truncated = !row.renaming && draw_title(ui, ink, row, [title_x, top], title_end);
        (title_x, truncated)
    };
    if let Some(detail) = row.detail {
        draw_list.add_text(
            [content_left + padding[0], top + ui.text_line_height()],
            ink.color(detail.color),
            truncate_to_width(ui, detail.text, right - content_left - padding[0]),
        );
    }
    RowTexts {
        title_truncated,
        hovered_marker: hovered_marker.filter(|_| is_hovered),
        title_rect: [[title_x, min[1]], [title_end, max[1]]],
    }
}

/// Draws the title of `row` at `origin` in the current font, shortened to
/// end before `end`, with its search hits marked. Returns whether it was
/// shortened (`lay-truncation`).
fn draw_title(ui: &Ui, ink: &RowInk<'_>, row: &ListRow<'_>, origin: [f32; 2], end: f32) -> bool {
    let title = truncate_to_width(ui, row.title, end - origin[0]);
    window_draw_list(ui).add_text(origin, ink.color(ink.theme.text.primary), &title);
    if let Some(query) = row.title_hits {
        let hits = query.hit_ranges(&title);
        let marks = hits.into_iter().map(|range| (range, HitMark::Filled));
        mark_search_hits(ui, ink.theme, &title, marks, origin);
    }
    matches!(title, Cow::Owned(_))
}
