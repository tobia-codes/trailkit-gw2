//! Draws a parsed [`MarkdownDocument`] with ImGui (Windows only; memo plan M3,
//! M19, M37, M38).
//!
//! Text is laid out word by word so bold, italic and links can share a line
//! and still wrap at the readable width (`lay-line-length`) or the window
//! edge, whichever is narrower (`lay-responsive`). Words wider than a whole
//! line are broken between characters. Nothing is allocated per frame except
//! the numbers of ordered list items.
//!
//! The view never changes anything itself: clicks are returned as a
//! [`MarkdownAction`] for the caller to apply.

use nexus::imgui::{MouseCursor, Ui};

use crate::{
    font_registry::FontScope,
    fonts::FontRole,
    icons::Icon,
    markdown::{Block, ListMarker, MarkdownDocument, Span, TaskMarker},
    theme::Theme,
};

/// Readable line length in average characters (`lay-line-length`, 60–80).
const READABLE_LINE_CHARACTERS: f32 = 75.0;

/// A user interaction with the markdown view.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MarkdownAction {
    /// A task checkbox was clicked; toggle it with `markdown::toggle_task`.
    ToggleTask(TaskMarker),
    /// A web link was clicked; open it with [`open_web_link`].
    OpenLink(String),
}

/// Horizontal layout of one block: where text starts and where it must wrap.
#[derive(Debug, Clone, Copy)]
struct LineBounds {
    /// Left edge of every line (screen x).
    left: f32,
    /// Right edge no text may cross (screen x).
    right: f32,
}

/// Draws `document` at the cursor position and returns what the user clicked.
pub fn draw_markdown(
    ui: &Ui,
    document: &MarkdownDocument,
    theme: &Theme,
) -> Option<MarkdownAction> {
    let _body_font = FontScope::push(ui, FontRole::Body);
    let origin_x = ui.cursor_screen_pos()[0];
    let average_character_width = ui.calc_text_size("n")[0];
    let width = ui.content_region_avail()[0]
        .min(average_character_width * READABLE_LINE_CHARACTERS)
        .max(average_character_width);
    let right = origin_x + width;
    let indent = theme.list.indent;

    let mut action = None;
    for (index, block) in document.blocks.iter().enumerate() {
        let _id = ui.push_id(i32::try_from(index).unwrap_or(i32::MAX));
        if index > 0 {
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
                draw_spans(ui, spans, Some(FontRole::heading(*level)), bounds, theme)
            }
            Block::Paragraph { depth, spans } => {
                let bounds = LineBounds {
                    left: origin_x + f32::from(*depth) * indent,
                    right,
                };
                draw_spans(ui, spans, None, bounds, theme)
            }
            Block::ListItem {
                depth,
                marker,
                spans,
            } => {
                let left = origin_x + f32::from(depth.saturating_sub(1)) * indent;
                let marker_click = draw_list_marker(ui, *marker, left, indent, theme);
                let bounds = LineBounds {
                    left: left + indent,
                    right,
                };
                let text_click = draw_spans(ui, spans, None, bounds, theme);
                marker_click.or(text_click)
            }
            Block::Rule => {
                ui.separator();
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
    let line_start = ui.cursor_screen_pos();
    ui.set_cursor_screen_pos([left, line_start[1]]);
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
            let size = {
                let _icons = FontScope::push(ui, FontRole::Icons);
                let glyph = icon.as_str(&mut buffer);
                let size = ui.calc_text_size(glyph);
                let color = if checked {
                    theme.accent.base
                } else {
                    theme.text.secondary
                };
                ui.text_colored(color.to_array(), glyph);
                size
            };
            ui.set_cursor_screen_pos([left, line_start[1]]);
            let clicked = ui.invisible_button("task", [size[0].max(1.0), size[1].max(1.0)]);
            if ui.is_item_hovered() {
                ui.set_mouse_cursor(Some(MouseCursor::Hand));
            }
            clicked.then_some(MarkdownAction::ToggleTask(marker))
        }
    };
    // Text continues on the same line, right of the marker column.
    ui.set_cursor_screen_pos([left + marker_width, line_start[1]]);
    action
}

/// Lays out `spans` word by word between `bounds`, starting at the current
/// line, and leaves the cursor below the last line. `heading` overrides the
/// font of every span. Returns a link click.
fn draw_spans(
    ui: &Ui,
    spans: &[Span],
    heading: Option<FontRole>,
    bounds: LineBounds,
    theme: &Theme,
) -> Option<MarkdownAction> {
    let line_height = {
        let _font = FontScope::push(ui, heading.unwrap_or(FontRole::Body));
        ui.text_line_height_with_spacing()
    };
    let mut x = bounds.left;
    let mut y = ui.cursor_screen_pos()[1];
    let mut action = None;

    for span in spans {
        if span.is_line_break() {
            x = bounds.left;
            y += line_height;
            continue;
        }
        let role = heading.unwrap_or_else(|| FontRole::body(span.style.bold, span.style.italic));
        let _font = FontScope::push(ui, role);
        for token in span.text.split_inclusive(' ') {
            let word = token.trim_end_matches(' ');
            let word_width = ui.calc_text_size(word)[0];
            if x > bounds.left && x + word_width > bounds.right {
                x = bounds.left;
                y += line_height;
            }
            if !word.is_empty() {
                let clicked = draw_word(ui, word, span, bounds, theme, &mut x, &mut y, line_height);
                action = action.or(clicked);
            }
            let trailing_spaces = &token[word.len()..];
            if x > bounds.left {
                x += ui.calc_text_size(trailing_spaces)[0];
            }
        }
    }
    ui.set_cursor_screen_pos([bounds.left, y + line_height]);
    action
}

/// Draws one word at `(x, y)`, breaking it between characters if it is wider
/// than a whole line, and advances `x`/`y`. Returns a link click.
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
    x: &mut f32,
    y: &mut f32,
    line_height: f32,
) -> Option<MarkdownAction> {
    let mut action = None;
    let mut rest = word;
    while !rest.is_empty() {
        let available = bounds.right - *x;
        let piece = fitting_prefix(ui, rest, available, *x <= bounds.left);
        let piece_width = ui.calc_text_size(piece)[0];
        ui.set_cursor_screen_pos([*x, *y]);
        let clicked = draw_piece(ui, piece, span, theme);
        action = action.or(clicked);
        *x += piece_width;
        rest = &rest[piece.len()..];
        if !rest.is_empty() {
            *x = bounds.left;
            *y += line_height;
        }
    }
    action
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
        ui.tooltip_text(url);
    }
    ui.is_item_clicked()
        .then(|| MarkdownAction::OpenLink(url.clone()))
}
