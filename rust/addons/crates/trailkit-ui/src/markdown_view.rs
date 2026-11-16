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

use nexus::imgui::{MouseCursor, Ui};

use self::inline_text::draw_spans;
use crate::{
    chat_link_chip::ChipContext,
    draw_list::window_draw_list,
    font_registry::FontScope,
    fonts::FontRole,
    icons::Icon,
    markdown::{Block, ListMarker, MarkdownDocument, TaskMarker},
    separator::draw_compact_separator,
    text::{TextStyle, draw_icon},
    text_search::SearchQuery,
    theme::Theme,
};

/// Readable line length in average characters (`lay-line-length`, 60–80).
const READABLE_LINE_CHARACTERS: f32 = 75.0;

/// Search hits to mark while drawing (memo plan M122): matching text gets an
/// `accent.highlight` background, a matching chat-link chip an
/// `accent.text` frame.
#[derive(Debug)]
pub struct SearchHighlight<'query> {
    /// What to mark.
    query: &'query SearchQuery,
    /// Window-content y of the first marked line, filled in by
    /// [`draw_markdown`] (scroll target for "jump to the first hit").
    first_hit_y: Option<f32>,
}

impl<'query> SearchHighlight<'query> {
    /// Marks the hits of `query`.
    #[must_use]
    pub const fn new(query: &'query SearchQuery) -> Self {
        Self {
            query,
            first_hit_y: None,
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
    /// A chat-link chip was clicked and its code copied to the clipboard
    /// (memo plan M75); show "Copied" by passing it as
    /// [`ChipContext::copied`] for a moment.
    ChatLinkCopied(String),
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

/// The font of the first line of `document`, e.g. to keep an even gap above it.
#[must_use]
pub fn first_line_font(document: &MarkdownDocument) -> FontRole {
    match document.blocks.first() {
        Some(Block::Heading { level, .. }) => FontRole::heading(*level),
        _ => FontRole::Body,
    }
}

/// Draws `document` at the cursor position and returns what the user
/// clicked. `chips` supplies the texts of chat-link chips; `highlight`
/// marks search hits.
pub fn draw_markdown(
    ui: &Ui,
    document: &MarkdownDocument,
    theme: &Theme,
    chips: ChipContext<'_>,
    mut highlight: Option<&mut SearchHighlight<'_>>,
) -> Option<MarkdownAction> {
    let _body_font = FontScope::push(ui, FontRole::Body);
    let origin_x = ui.cursor_screen_pos()[0];
    let average_character_width = ui.calc_text_size("n")[0];
    let width = ui.content_region_avail()[0]
        .min(average_character_width * READABLE_LINE_CHARACTERS)
        .max(average_character_width);
    let indent = theme.list.indent;
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
        // gets the paragraph gap (memo plan M66).
        let continues_list = matches!(block, Block::ListItem { .. })
            && index
                .checked_sub(1)
                .and_then(|previous| document.blocks.get(previous))
                .is_some_and(|previous| matches!(previous, Block::ListItem { .. }));
        if index > 0 && !continues_list {
            ui.dummy([0.0, theme.spacing.small]);
        }
        let clicked = draw_block(
            ui,
            block,
            index > 0,
            layout,
            theme,
            chips,
            highlight.as_deref_mut(),
        );
        action = action.or(clicked);
    }
    ui.dummy([0.0, 0.0]);
    action
}

/// Draws one `block` on the current line with the document's `layout`;
/// `follows_block` adds the gap a heading keeps from what is above it.
/// Returns what the user clicked.
fn draw_block(
    ui: &Ui,
    block: &Block,
    follows_block: bool,
    layout: BlockLayout,
    theme: &Theme,
    chips: ChipContext<'_>,
    highlight: Option<&mut SearchHighlight<'_>>,
) -> Option<MarkdownAction> {
    match block {
        Block::Heading { level, spans } => {
            if follows_block {
                ui.dummy([0.0, theme.spacing.small]);
            }
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
            draw_compact_separator(ui, theme);
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
            let mut buffer = [0; 4];
            let _icons = FontScope::push(ui, FontRole::Icons);
            let glyph = icon.as_str(&mut buffer);
            let size = ui.calc_text_size(glyph);
            // Only the box is clickable, the text next to it may hold links
            // and chips (R85); the target reaches a little beyond the glyph.
            let margin = theme.spacing.extra_small;
            let target_min = [left - margin, text_top - margin];
            let target_size = [
                size[0].max(1.0) + 2.0 * margin,
                size[1].max(1.0) + 2.0 * margin,
            ];
            ui.set_cursor_screen_pos(target_min);
            let clicked = ui.invisible_button("task", target_size);
            let fill = if ui.is_item_active() {
                Some(theme.surface.active)
            } else if ui.is_item_hovered() {
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
            ui.set_cursor_screen_pos([left, text_top]);
            draw_icon(ui, icon, color);
            clicked.then_some(MarkdownAction::ToggleTask(marker))
        }
    };
    // Text continues on the same line, right of the marker column.
    ui.set_cursor_screen_pos([left + marker_width, line_top]);
    action
}
