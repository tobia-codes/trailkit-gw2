//! Inline text of a markdown block: words with wrapping, links, chat-link
//! chips and search hits (memo plan M19, M37, M75, M122).

use std::ops::Range;

use nexus::imgui::{MouseCursor, StyleColor, Ui};

use super::{LineBounds, MarkdownAction, SearchHighlight};
use crate::{
    chat_link::ChatLinkChip,
    chat_link_chip::{ChipContext, chip_tooltip, chip_width, draw_chip},
    draw_list::window_draw_list,
    font_registry::FontScope,
    fonts::FontRole,
    markdown::Span,
    text::TextStyle,
    theme::Theme,
    tooltip::draw_tooltip,
};

/// Where the next inline element of a block goes (screen coordinates).
#[derive(Debug, Clone, Copy)]
struct LineCursor {
    /// Left edge of the next element.
    x: f32,
    /// Text top of the current line, below its leading.
    y: f32,
    /// Distance between two lines (`typography.line_height`).
    line_height: f32,
}

impl LineCursor {
    /// Moves to the start of the next line.
    fn new_line(&mut self, bounds: LineBounds) {
        self.x = bounds.left;
        self.y += self.line_height;
    }

    /// Starts a new line if an element `width` wide does not fit on the
    /// current one; an element at the line start stays, however wide.
    fn wrap_before(&mut self, width: f32, bounds: LineBounds) {
        if self.x > bounds.left && self.x + width > bounds.right {
            self.new_line(bounds);
        }
    }
}

/// Lays out `spans` word by word between `bounds`, starting at the current
/// line, and leaves the cursor below the last line. The block's text style
/// is `heading` or [`TextStyle::Body`] (Q276): it sets the color, and a
/// heading also the font of every span; other spans pick their body face
/// (bold, italic) themselves; chat codes are drawn as chips; `highlight` marks
/// search hits. Returns a link or chip click.
pub(super) fn draw_spans(
    ui: &Ui,
    spans: &[Span],
    heading: Option<TextStyle>,
    bounds: LineBounds,
    theme: &Theme,
    chips: ChipContext<'_>,
    mut highlight: Option<&mut SearchHighlight<'_>>,
) -> Option<MarkdownAction> {
    // Lines are `typography.line_height` apart; `y` is the text top, below
    // the leading of the line.
    let (line_height, leading) = {
        let _font = FontScope::push(ui, heading.map_or(FontRole::Body, TextStyle::font));
        let font_size = ui.current_font_size();
        (
            theme.typography.line_height_for(font_size),
            theme.typography.leading_for(font_size),
        )
    };
    let mut cursor = LineCursor {
        x: bounds.left,
        y: ui.cursor_screen_pos()[1] + leading,
        line_height,
    };
    let mut action = None;
    let mut hits: Vec<Range<usize>> = Vec::new();
    // The block's role sets the text color (Q276); links keep their own.
    let _color = ui.push_style_color(
        StyleColor::Text,
        heading.unwrap_or(TextStyle::Body).color(theme).to_array(),
    );

    for (index, span) in spans.iter().enumerate() {
        if span.is_line_break() {
            cursor.new_line(bounds);
            continue;
        }
        let role = heading.map_or_else(
            || FontRole::body(span.style.bold, span.style.italic),
            TextStyle::font,
        );
        let _font = FontScope::push(ui, role);
        let clicked = if let Some(chip) = &span.chip {
            let _id = ui.push_id(i32::try_from(index).unwrap_or(i32::MAX));
            draw_chat_link(
                ui,
                chip,
                bounds,
                theme,
                chips,
                highlight.as_deref_mut(),
                &mut cursor,
            )
        } else {
            draw_wrapped_text(
                ui,
                span,
                bounds,
                theme,
                highlight.as_deref_mut(),
                &mut hits,
                &mut cursor,
            )
        };
        action = action.or(clicked);
    }
    ui.set_cursor_screen_pos([bounds.left, cursor.y - leading + line_height]);
    action
}

/// Draws a chat link as a clickable chip at the `cursor`, wrapping first if
/// it does not fit, frames it while it matches the search (M122) and advances
/// the cursor. A click copies the code (M75) and is returned.
fn draw_chat_link(
    ui: &Ui,
    chip: &ChatLinkChip,
    bounds: LineBounds,
    theme: &Theme,
    chips: ChipContext<'_>,
    highlight: Option<&mut SearchHighlight<'_>>,
    cursor: &mut LineCursor,
) -> Option<MarkdownAction> {
    let width = chip_width(ui, chip, theme, chips);
    cursor.wrap_before(width, bounds);
    let [x, y] = [cursor.x, cursor.y];
    ui.set_cursor_screen_pos([x, y]);
    ui.invisible_button("##chip", [width, ui.text_line_height()]);
    let hovered = ui.is_item_hovered();
    let clicked = ui.is_item_clicked();
    draw_chip(ui, window_draw_list(ui), chip, [x, y], theme, chips);
    if let Some(highlight) = highlight
        && !highlight.query.hit_ranges(&chip.code).is_empty()
    {
        highlight.record_hit(ui, y);
        window_draw_list(ui)
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
    cursor.x += width;
    clicked.then(|| {
        ui.set_clipboard_text(&chip.code);
        MarkdownAction::ChatLinkCopied(chip.code.clone())
    })
}

/// Draws the text of `span` word by word at the `cursor`, wrapping at
/// `bounds`, and marks its search hits; `hits` is a reused buffer for their
/// ranges. Returns a link click.
fn draw_wrapped_text(
    ui: &Ui,
    span: &Span,
    bounds: LineBounds,
    theme: &Theme,
    mut highlight: Option<&mut SearchHighlight<'_>>,
    hits: &mut Vec<Range<usize>>,
    cursor: &mut LineCursor,
) -> Option<MarkdownAction> {
    match highlight.as_deref() {
        Some(highlight) => highlight.query.collect_hit_ranges(&span.text, hits),
        None => hits.clear(),
    }
    let mut action = None;
    let mut token_offset = 0;
    for token in span.text.split_inclusive(' ') {
        let word = token.trim_end_matches(' ');
        let word_offset = token_offset;
        token_offset += token.len();
        cursor.wrap_before(ui.calc_text_size(word)[0], bounds);
        if !word.is_empty() {
            let word_hits = WordHits {
                ranges: hits,
                offset: word_offset,
            };
            let clicked = draw_word(ui, word, span, bounds, theme, cursor, word_hits);
            if let Some(first_hit) = clicked.first_hit_y
                && let Some(highlight) = highlight.as_deref_mut()
            {
                highlight.record_hit(ui, first_hit);
            }
            action = action.or(clicked.action);
        }
        let trailing_spaces = &token[word.len()..];
        if cursor.x > bounds.left {
            cursor.x += ui.calc_text_size(trailing_spaces)[0];
        }
    }
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

/// Draws one word at the `cursor`, breaking it between characters if it is
/// wider than a whole line, marks its search hits and advances the cursor.
/// Returns a link click and where a hit was marked.
fn draw_word(
    ui: &Ui,
    word: &str,
    span: &Span,
    bounds: LineBounds,
    theme: &Theme,
    cursor: &mut LineCursor,
    hits: WordHits<'_>,
) -> WordOutcome {
    let mut outcome = WordOutcome::default();
    let mut rest = word;
    let mut piece_offset = hits.offset;
    while !rest.is_empty() {
        let available = bounds.right - cursor.x;
        let piece = fitting_prefix(ui, rest, available, cursor.x <= bounds.left);
        let piece_width = ui.calc_text_size(piece)[0];
        let position = [cursor.x, cursor.y];
        if mark_hits(ui, piece, piece_offset, hits.ranges, position, theme) {
            outcome.first_hit_y = outcome.first_hit_y.or(Some(cursor.y));
        }
        ui.set_cursor_screen_pos(position);
        let clicked = draw_piece(ui, piece, span, theme);
        outcome.action = outcome.action.or(clicked);
        cursor.x += piece_width;
        piece_offset += piece.len();
        rest = &rest[piece.len()..];
        if !rest.is_empty() {
            cursor.new_line(bounds);
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
        window_draw_list(ui)
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
    window_draw_list(ui)
        .add_line(
            [min[0], max[1]],
            [max[0], max[1]],
            theme.text.link.to_array(),
        )
        .build();
    if ui.is_item_hovered() {
        ui.set_mouse_cursor(Some(MouseCursor::Hand));
        draw_tooltip(ui, theme, crate::markdown::link_target_label(url));
    }
    ui.is_item_clicked()
        .then(|| MarkdownAction::OpenLink(url.clone()))
}
