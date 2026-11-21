//! Draws a parsed [`MarkdownDocument`] with ImGui (Windows only; memo plan M3,
//! M19, M37, M38).
//!
//! Text is laid out word by word so bold, italic and links can share a line
//! and still wrap at the readable width (`lay-line-length`) or the window
//! edge, whichever is narrower (`lay-responsive`). Words wider than a whole
//! line are broken between characters. Nothing is allocated per frame except
//! the numbers of ordered list items and, while a search is active, the hit
//! ranges of each span (memo plan M122).
//!
//! The view never changes anything itself: clicks are returned as a
//! [`MarkdownAction`] for the caller to apply.

mod inline_text;
mod link_menu;

use crate::controls::hover::is_item_hovered;
use nexus::imgui::{ImColor32, MouseCursor, Ui};

use self::{inline_text::draw_spans, link_menu::draw_link_menu};
use crate::{
    chat_link::chat_link_chip::ChipContext,
    controls::dropdown::draw_icon_into,
    drawing::draw_list::window_draw_list,
    layout::separator::draw_separator,
    markdown::{Block, ListMarker, MarkdownDocument, TaskMarker},
    text::TextStyle,
    text::font_registry::FontScope,
    text::fonts::FontRole,
    text::glyph_metrics::capital_middle,
    text::icons::Icon,
    text::search_hits::HitMark,
    text::text_search::SearchQuery,
    theme::Theme,
};

/// Readable line length in average characters (`lay-line-length`, 60–80).
const READABLE_LINE_CHARACTERS: f32 = 75.0;

/// Search hits to mark while drawing (memo plan M122, M197, M198): matching
/// text is marked like a text marker, a matching chat-link chip gets an
/// `accent.text` frame. Find in a note also numbers the hits in drawing
/// order and marks only the current one filled, the others framed.
#[derive(Debug)]
pub struct SearchHighlight<'query> {
    /// What to mark.
    query: &'query SearchQuery,
    /// Window-content y of the first marked line, filled in by
    /// [`draw_markdown`] (scroll target for "jump to the first hit").
    first_hit_y: Option<f32>,
    /// Find in a note: the index of the current hit; `None` marks every
    /// hit filled (the list search).
    current: Option<usize>,
    /// Hits numbered so far while drawing.
    hit_count: usize,
    /// Window-content y of the current hit's line, once drawn.
    current_hit_y: Option<f32>,
}

impl<'query> SearchHighlight<'query> {
    /// Marks every hit of `query` filled (the list search, M197).
    #[must_use]
    pub const fn new(query: &'query SearchQuery) -> Self {
        Self {
            query,
            first_hit_y: None,
            current: None,
            hit_count: 0,
            current_hit_y: None,
        }
    }

    /// Marks the hit numbered `current` (0-based, drawing order) filled and
    /// the others framed (find in a note, M198).
    #[must_use]
    pub const fn with_current(query: &'query SearchQuery, current: usize) -> Self {
        Self {
            current: Some(current),
            ..Self::new(query)
        }
    }

    /// How many hits were drawn (text hits and chips), after
    /// [`draw_markdown`].
    #[must_use]
    pub const fn hit_count(&self) -> usize {
        self.hit_count
    }

    /// Window-content y (as for `set_scroll_y`) of the top of the current
    /// hit's line, if it was drawn.
    #[must_use]
    pub const fn current_hit_y(&self) -> Option<f32> {
        self.current_hit_y
    }

    /// How the next hit is marked, and whether it is the current one.
    fn next_mark(&self) -> (HitMark, bool) {
        self.mark_at(0)
    }

    /// How the hit `offset` places after the next one is marked, and
    /// whether it is the current one.
    fn mark_at(&self, offset: usize) -> (HitMark, bool) {
        let index = self.hit_count + offset;
        match self.current {
            Some(current) if current == index => (HitMark::Filled, true),
            Some(_) => (HitMark::Framed, false),
            None => (HitMark::Filled, false),
        }
    }

    /// Counts `count` drawn hits; `current_y` is the screen y of the current
    /// hit if it was among them.
    fn count_hits(&mut self, ui: &Ui, count: usize, current_y: Option<f32>) {
        self.hit_count += count;
        if let Some(y) = current_y {
            self.current_hit_y = Some(y - ui.window_pos()[1] + ui.scroll_y());
        }
    }

    /// Window-content y (as for `set_scroll_y`) of the top of the first
    /// marked line, if anything was marked.
    #[must_use]
    pub const fn first_hit_y(&self) -> Option<f32> {
        self.first_hit_y
    }

    /// Remembers the first hit at screen `y`.
    fn record_hit(&mut self, ui: &Ui, y: f32) {
        if self.first_hit_y.is_none() {
            self.first_hit_y = Some(y - ui.window_pos()[1] + ui.scroll_y());
        }
    }
}

/// A user interaction with the markdown view.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MarkdownAction {
    /// A task checkbox was clicked; toggle it with `markdown::toggle_task`.
    ToggleTask(TaskMarker),
    /// A web or e-mail link was clicked; open it with [`open_link`].
    OpenLink(String),
    /// A chip was clicked, or a copy entry of the right-click menu chosen, and
    /// this text is on the clipboard (memo plan M75, M210): confirm it.
    Copied(String),
}

/// Horizontal layout of one block: where text starts and where it must wrap.
#[derive(Debug, Clone, Copy)]
struct LineBounds {
    /// Left edge of every line (screen x).
    left: f32,
    /// Right edge no text may cross (screen x).
    right: f32,
}

/// Horizontal layout shared by every block of a document.
#[derive(Debug, Clone, Copy)]
struct BlockLayout {
    /// Left edge of unindented text (screen x).
    left: f32,
    /// Right edge no text may cross (screen x).
    right: f32,
    /// Indent per nesting level.
    indent: f32,
    /// Width of the list marker column, gap included.
    marker_column: f32,
}

/// Draws `document` at the cursor position and returns what the user
/// clicked. `chips` supplies the texts of chat-link chips; `highlight`
/// marks search hits.
pub fn draw_markdown(
    ui: &Ui,
    document: &MarkdownDocument,
    theme: &Theme,
    chips: ChipContext<'_>,
    highlight: Option<&mut SearchHighlight<'_>>,
) -> Option<MarkdownAction> {
    draw_document(ui, document, theme, chips, highlight, None)
}

/// Like [`draw_markdown`] without search hits, and also returns the
/// window-content y (as for `set_scroll_y`) of the top of every heading of
/// `level`, in order: the scroll targets of the help's sections (R155 c).
pub fn draw_markdown_with_heading_tops(
    ui: &Ui,
    document: &MarkdownDocument,
    theme: &Theme,
    chips: ChipContext<'_>,
    level: u8,
) -> (Option<MarkdownAction>, Vec<f32>) {
    let mut tops = Vec::new();
    let action = draw_document(ui, document, theme, chips, None, Some((level, &mut tops)));
    (action, tops)
}

/// Draws `document` for [`draw_markdown`]; `heading_tops` collects the
/// window-content y of every heading of its level.
fn draw_document(
    ui: &Ui,
    document: &MarkdownDocument,
    theme: &Theme,
    chips: ChipContext<'_>,
    mut highlight: Option<&mut SearchHighlight<'_>>,
    mut heading_tops: Option<(u8, &mut Vec<f32>)>,
) -> Option<MarkdownAction> {
    let _body_font = FontScope::push(ui, FontRole::Body);
    let origin_x = ui.cursor_screen_pos()[0];
    let average_character_width = ui.calc_text_size("n")[0];
    let width = ui.content_region_avail()[0]
        .min(average_character_width * READABLE_LINE_CHARACTERS)
        .max(average_character_width);
    let indent = theme.spacing.large;
    // The marker column must fit the widest marker (a task box) plus a gap,
    // whatever the font size; otherwise the text touches the checkbox.
    let marker_column = {
        let _icons = FontScope::push(ui, FontRole::Icons);
        let mut buffer = [0; 4];
        ui.calc_text_size(Icon::SQUARE.as_str(&mut buffer))[0]
    }
    .max(ui.calc_text_size("00.")[0])
        + theme.spacing.extra_small;
    let layout = BlockLayout {
        left: origin_x,
        right: origin_x + width,
        indent,
        marker_column: marker_column.max(indent),
    };

    let mut action = None;
    for (index, block) in document.blocks.iter().enumerate() {
        let _id = ui.push_id(i32::try_from(index).unwrap_or(i32::MAX));
        // Items of one list stand close together; every other block boundary
        // gets the paragraph gap `space.lg`, a heading `space.xl` above it
        // (memo plan M66, Q530).
        let continues_list = matches!(block, Block::ListItem { .. })
            && index
                .checked_sub(1)
                .and_then(|previous| document.blocks.get(previous))
                .is_some_and(|previous| matches!(previous, Block::ListItem { .. }));
        if index > 0 && !continues_list {
            let gap = if matches!(block, Block::Heading { .. }) {
                theme.spacing.extra_large
            } else {
                theme.spacing.large
            };
            ui.dummy([0.0, gap]);
        }
        if let (Block::Heading { level, .. }, Some((wanted, tops))) = (block, heading_tops.as_mut())
            && *level == *wanted
        {
            tops.push(ui.cursor_screen_pos()[1] - ui.window_pos()[1] + ui.scroll_y());
        }
        let clicked = draw_block(ui, block, layout, theme, chips, highlight.as_deref_mut());
        action = action.or(clicked);
    }
    ui.dummy([0.0, 0.0]);
    action.or(draw_link_menu(ui, theme, chips))
}

/// Draws one `block` on the current line with the document's `layout`.
/// Returns what the user clicked.
fn draw_block(
    ui: &Ui,
    block: &Block,
    layout: BlockLayout,
    theme: &Theme,
    chips: ChipContext<'_>,
    highlight: Option<&mut SearchHighlight<'_>>,
) -> Option<MarkdownAction> {
    match block {
        Block::Heading { level, spans } => {
            let bounds = LineBounds {
                left: layout.left,
                right: layout.right,
            };
            let style = Some(TextStyle::heading(*level));
            draw_spans(ui, spans, style, bounds, theme, chips, highlight)
        }
        Block::Paragraph { depth, spans } => {
            let bounds = LineBounds {
                left: layout.left + f32::from(*depth) * layout.indent,
                right: layout.right,
            };
            draw_spans(ui, spans, None, bounds, theme, chips, highlight)
        }
        Block::ListItem {
            depth,
            marker,
            spans,
        } => {
            let left = layout.left + f32::from(depth.saturating_sub(1)) * layout.indent;
            let marker_click = draw_list_marker(ui, *marker, left, layout.marker_column, theme);
            let bounds = LineBounds {
                left: left + layout.marker_column,
                right: layout.right,
            };
            let text_click = draw_spans(ui, spans, None, bounds, theme, chips, highlight);
            marker_click.or(text_click)
        }
        Block::Rule => {
            draw_separator(ui, theme);
            None
        }
    }
}

/// Opens `url` if it is openable (M37, M183): a web link in the default
/// browser, an e-mail link in the default mail program; other schemes are
/// ignored. Logs a failure.
pub fn open_link(url: &str) {
    if !crate::markdown::is_openable_link(url) {
        return;
    }
    // `url.dll` hands the URL to its default program without a shell, so the
    // URL cannot inject commands.
    let result = std::process::Command::new("rundll32")
        .args(["url.dll,FileProtocolHandler", url])
        .spawn();
    if let Err(error) = result {
        log::warn!(error:% = error; "link_open_failed");
    }
}

/// Draws the bullet, number or checkbox of a list item at `left` on the
/// current line, without moving the cursor down. Returns a checkbox click.
fn draw_list_marker(
    ui: &Ui,
    marker: ListMarker,
    left: f32,
    marker_width: f32,
    theme: &Theme,
) -> Option<MarkdownAction> {
    // The marker sits on the text of the first line (see `draw_spans`).
    let line_top = ui.cursor_screen_pos()[1];
    let text_top = line_top + theme.typography.leading_for(ui.current_font_size());
    ui.set_cursor_screen_pos([left, text_top]);
    let action = match marker {
        ListMarker::Bullet => {
            ui.text("•");
            None
        }
        ListMarker::Number(number) => {
            ui.text(format!("{number}."));
            None
        }
        ListMarker::Task { checked, marker } => {
            let icon = if checked {
                Icon::SQUARE_CHECK
            } else {
                Icon::SQUARE
            };
            // One interface icon size (Q537), on the capitals of the item's
            // first line like every icon beside text (R147).
            let middle = text_top + capital_middle().unwrap_or(ui.text_line_height() / 2.0);
            let [width, height] = {
                let _icons = FontScope::push(ui, FontRole::Icons);
                let mut buffer = [0; 4];
                ui.calc_text_size(icon.as_str(&mut buffer))
            };
            // Only the box is clickable, the text next to it may hold links
            // and chips (R85); the target reaches a little beyond the glyph.
            let margin = theme.spacing.extra_small;
            let target_min = [left - margin, (middle - height / 2.0).round() - margin];
            let target_size = [
                width.max(1.0) + 2.0 * margin,
                height.max(1.0) + 2.0 * margin,
            ];
            ui.set_cursor_screen_pos(target_min);
            let clicked = ui.invisible_button("task", target_size);
            let fill = if ui.is_item_active() {
                Some(theme.surface.active)
            } else if is_item_hovered(ui) {
                ui.set_mouse_cursor(Some(MouseCursor::Hand));
                Some(theme.surface.hover)
            } else {
                None
            };
            if let Some(fill) = fill {
                window_draw_list(ui)
                    .add_rect(ui.item_rect_min(), ui.item_rect_max(), fill.to_array())
                    .filled(true)
                    .rounding(theme.radius.medium)
                    .build();
            }
            let color = if checked {
                theme.accent.text
            } else {
                theme.text.secondary
            };
            // SAFETY: called on the render thread inside a frame, where ImGui
            // has a current window and returns its draw list.
            let draw_list = unsafe { nexus::imgui::sys::igGetWindowDrawList() };
            draw_icon_into(
                draw_list,
                ui,
                icon,
                left,
                middle,
                ImColor32::from(color.to_array()).to_bits(),
            );
            clicked.then_some(MarkdownAction::ToggleTask(marker))
        }
    };
    // Text continues on the same line, right of the marker column.
    ui.set_cursor_screen_pos([left + marker_width, line_top]);
    action
}
