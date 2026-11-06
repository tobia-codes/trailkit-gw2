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

use crate::tooltip::draw_tooltip;
use std::ops::Range;

use nexus::imgui::{MouseCursor, Ui};

use crate::{
    chat_link_chip::{ChipContext, chip_tooltip, chip_width, draw_chip},
    font_registry::FontScope,
    fonts::FontRole,
    icons::Icon,
    markdown::{Block, ListMarker, MarkdownDocument, Span, TaskMarker},
    separator::draw_compact_separator,
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
    /// A web link was clicked; open it with [`open_web_link`].
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
    let right = origin_x + width;
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
    let marker_column = marker_column.max(indent);

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
        let clicked = match block {
            Block::Heading { level, spans } => {
                if index > 0 {
                    ui.dummy([0.0, theme.spacing.small]);
                }
                let bounds = LineBounds {
                    left: origin_x,
                    right,
                };
                draw_spans(
                    ui,
                    spans,
                    Some(FontRole::heading(*level)),
                    bounds,
                    theme,
                    chips,
                    highlight.as_deref_mut(),
                )
            }
            Block::Paragraph { depth, spans } => {
                let bounds = LineBounds {
                    left: origin_x + f32::from(*depth) * indent,
                    right,
                };
                draw_spans(
                    ui,
                    spans,
                    None,
                    bounds,
                    theme,
                    chips,
                    highlight.as_deref_mut(),
                )
            }
            Block::ListItem {
                depth,
                marker,
                spans,
            } => {
                let left = origin_x + f32::from(depth.saturating_sub(1)) * indent;
                let marker_click = draw_list_marker(ui, *marker, left, marker_column, theme);
                let bounds = LineBounds {
                    left: left + marker_column,
                    right,
                };
                let text_click = draw_spans(
                    ui,
                    spans,
                    None,
                    bounds,
                    theme,
                    chips,
                    highlight.as_deref_mut(),
                );
                marker_click.or(text_click)
            }
            Block::Rule => {
                draw_compact_separator(ui, theme);
                None
            }
        };
        action = action.or(clicked);
    }
    ui.dummy([0.0, 0.0]);
    action
}

/// Opens `url` in the default browser if it is a web link (M37); other
/// schemes are ignored. Logs a failure.
pub fn open_web_link(url: &str) {
    if !crate::markdown::is_web_link(url) {
        return;
    }
    // `url.dll` hands the URL to the default browser without a shell, so the
    // URL cannot inject commands.
    let result = std::process::Command::new("rundll32")
        .args(["url.dll,FileProtocolHandler", url])
        .spawn();
    if let Err(error) = result {
        log::warn!("cannot open link: {error}");
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
                ui.get_window_draw_list()
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
            ui.text_colored(color.to_array(), glyph);
            clicked.then_some(MarkdownAction::ToggleTask(marker))
        }
    };
    // Text continues on the same line, right of the marker column.
    ui.set_cursor_screen_pos([left + marker_width, line_top]);
    action
}

/// Lays out `spans` word by word between `bounds`, starting at the current
/// line, and leaves the cursor below the last line. `heading` overrides the
/// font of every span; chat codes are drawn as chips; `highlight` marks
/// search hits. Returns a link or chip click.
fn draw_spans(
    ui: &Ui,
    spans: &[Span],
    heading: Option<FontRole>,
    bounds: LineBounds,
    theme: &Theme,
    chips: ChipContext<'_>,
    mut highlight: Option<&mut SearchHighlight<'_>>,
) -> Option<MarkdownAction> {
    // Lines are `typography.line_height` apart; `y` is the text top, below
    // the leading of the line.
    let (line_height, leading) = {
        let _font = FontScope::push(ui, heading.unwrap_or(FontRole::Body));
        let font_size = ui.current_font_size();
        (
            theme.typography.line_height_for(font_size),
            theme.typography.leading_for(font_size),
        )
    };
    let mut x = bounds.left;
    let mut y = ui.cursor_screen_pos()[1] + leading;
    let mut action = None;
    let mut hits: Vec<Range<usize>> = Vec::new();

    for (index, span) in spans.iter().enumerate() {
        if span.is_line_break() {
            x = bounds.left;
            y += line_height;
            continue;
        }
        let role = heading.unwrap_or_else(|| FontRole::body(span.style.bold, span.style.italic));
        let _font = FontScope::push(ui, role);
        if let Some(chip) = &span.chip {
            let width = chip_width(ui, chip, theme, chips);
            if x > bounds.left && x + width > bounds.right {
                x = bounds.left;
                y += line_height;
            }
            let _id = ui.push_id(i32::try_from(index).unwrap_or(i32::MAX));
            ui.set_cursor_screen_pos([x, y]);
            ui.invisible_button("##chip", [width, ui.text_line_height()]);
            let hovered = ui.is_item_hovered();
            let clicked = ui.is_item_clicked();
            draw_chip(ui, &ui.get_window_draw_list(), chip, [x, y], theme, chips);
            if let Some(highlight) = highlight.as_deref_mut()
                && !highlight.query.hit_ranges(&chip.code).is_empty()
            {
                highlight.record_hit(ui, y);
                ui.get_window_draw_list()
                    .add_rect(
                        [x, y],
                        [x + width, y + ui.text_line_height()],
                        theme.accent.text.to_array(),
                    )
                    .rounding(theme.radius.small)
                    .thickness(theme.border.width)
                    .build();
            }
            if hovered {
                ui.set_mouse_cursor(Some(MouseCursor::Hand));
                chip_tooltip(ui, chip, theme, chips, true);
            }
            if clicked {
                ui.set_clipboard_text(&chip.code);
                action = action.or_else(|| Some(MarkdownAction::ChatLinkCopied(chip.code.clone())));
            }
            x += width;
            continue;
        }
        match highlight.as_deref() {
            Some(highlight) => highlight.query.collect_hit_ranges(&span.text, &mut hits),
            None => hits.clear(),
        }
        let mut token_offset = 0;
        for token in span.text.split_inclusive(' ') {
            let word = token.trim_end_matches(' ');
            let word_offset = token_offset;
            token_offset += token.len();
            let word_width = ui.calc_text_size(word)[0];
            if x > bounds.left && x + word_width > bounds.right {
                x = bounds.left;
                y += line_height;
            }
            if !word.is_empty() {
                let word_hits = WordHits {
                    ranges: &hits,
                    offset: word_offset,
                };
                let clicked = draw_word(
                    ui,
                    word,
                    span,
                    bounds,
                    theme,
                    (&mut x, &mut y),
                    line_height,
                    word_hits,
                );
                if let Some(first_hit) = clicked.first_hit_y
                    && let Some(highlight) = highlight.as_deref_mut()
                {
                    highlight.record_hit(ui, first_hit);
                }
                action = action.or(clicked.action);
            }
            let trailing_spaces = &token[word.len()..];
            if x > bounds.left {
                x += ui.calc_text_size(trailing_spaces)[0];
            }
        }
    }
    ui.set_cursor_screen_pos([bounds.left, y - leading + line_height]);
    action
}

/// Search hits of a span, seen from one word of it.
#[derive(Debug, Clone, Copy)]
struct WordHits<'ranges> {
    /// Hit byte ranges in the span text, sorted.
    ranges: &'ranges [Range<usize>],
    /// Byte offset of the word in the span text.
    offset: usize,
}

/// What drawing one word produced.
#[derive(Debug, Default)]
struct WordOutcome {
    /// A link click.
    action: Option<MarkdownAction>,
    /// Screen y of the first marked piece, if any.
    first_hit_y: Option<f32>,
}

/// Draws one word at `position` (`x`, `y`), breaking it between characters if
/// it is wider than a whole line, marks its search hits and advances the
/// position. Returns a link click and where a hit was marked.
#[expect(
    clippy::too_many_arguments,
    reason = "layout state is threaded through explicitly to avoid per-frame allocation"
)]
fn draw_word(
    ui: &Ui,
    word: &str,
    span: &Span,
    bounds: LineBounds,
    theme: &Theme,
    position: (&mut f32, &mut f32),
    line_height: f32,
    hits: WordHits<'_>,
) -> WordOutcome {
    let (x, y) = position;
    let mut outcome = WordOutcome::default();
    let mut rest = word;
    let mut piece_offset = hits.offset;
    while !rest.is_empty() {
        let available = bounds.right - *x;
        let piece = fitting_prefix(ui, rest, available, *x <= bounds.left);
        let piece_width = ui.calc_text_size(piece)[0];
        if mark_hits(ui, piece, piece_offset, hits.ranges, [*x, *y], theme) {
            outcome.first_hit_y = outcome.first_hit_y.or(Some(*y));
        }
        ui.set_cursor_screen_pos([*x, *y]);
        let clicked = draw_piece(ui, piece, span, theme);
        outcome.action = outcome.action.or(clicked);
        *x += piece_width;
        piece_offset += piece.len();
        rest = &rest[piece.len()..];
        if !rest.is_empty() {
            *x = bounds.left;
            *y += line_height;
        }
    }
    outcome
}

/// Draws the `accent.highlight` background behind the parts of `piece`
/// (starting at byte `offset` of its span and drawn at `origin`) covered by
/// `ranges`. Returns whether anything was marked.
fn mark_hits(
    ui: &Ui,
    piece: &str,
    offset: usize,
    ranges: &[Range<usize>],
    origin: [f32; 2],
    theme: &Theme,
) -> bool {
    let piece_end = offset + piece.len();
    let mut marked = false;
    for range in ranges {
        if range.end <= offset || range.start >= piece_end {
            continue;
        }
        let start = range.start.max(offset) - offset;
        let end = range.end.min(piece_end) - offset;
        let left = origin[0] + ui.calc_text_size(&piece[..start])[0];
        let right = origin[0] + ui.calc_text_size(&piece[..end])[0];
        ui.get_window_draw_list()
            .add_rect(
                [left, origin[1]],
                [right, origin[1] + ui.text_line_height()],
                theme.accent.highlight.to_array(),
            )
            .filled(true)
            .rounding(theme.radius.small)
            .build();
        marked = true;
    }
    marked
}

/// The longest prefix of `text` (whole characters) narrower than `available`;
/// at least one character if `at_line_start`, so layout always progresses.
fn fitting_prefix<'text>(
    ui: &Ui,
    text: &'text str,
    available: f32,
    at_line_start: bool,
) -> &'text str {
    if ui.calc_text_size(text)[0] <= available {
        return text;
    }
    let mut end = 0;
    for (index, character) in text.char_indices() {
        let next = index + character.len_utf8();
        if ui.calc_text_size(&text[..next])[0] > available {
            break;
        }
        end = next;
    }
    if end == 0 && at_line_start {
        end = text.chars().next().map_or(text.len(), char::len_utf8);
    }
    &text[..end]
}

/// Draws a piece of text at the cursor, as a link if the span has one.
fn draw_piece(ui: &Ui, piece: &str, span: &Span, theme: &Theme) -> Option<MarkdownAction> {
    let Some(url) = &span.style.link else {
        ui.text(piece);
        return None;
    };
    ui.text_colored(theme.text.link.to_array(), piece);
    let [min, max] = [ui.item_rect_min(), ui.item_rect_max()];
    ui.get_window_draw_list()
        .add_line(
            [min[0], max[1]],
            [max[0], max[1]],
            theme.text.link.to_array(),
        )
        .build();
    if ui.is_item_hovered() {
        ui.set_mouse_cursor(Some(MouseCursor::Hand));
        draw_tooltip(ui, url);
    }
    ui.is_item_clicked()
        .then(|| MarkdownAction::OpenLink(url.clone()))
}
