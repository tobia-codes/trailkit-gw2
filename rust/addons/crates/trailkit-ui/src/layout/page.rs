//! The head of a page inside a pane (ROADMAP R101): the title in Heading1
//! with an optional widget behind it (memo M102), an optional Secondary line
//! below it, the separator, and the content in its own scroll region, so
//! title and separator stay in place (memo M81). Used by memo's note view
//! and trash preview and by the help window's topics and problem page.

use nexus::imgui::{ChildWindow, Ui};

use crate::{
    chat_link::chat_link_chip::ChipContext,
    layout::separator::{LineSpacing, capital_top},
    layout::stack::Stack,
    markdown::{MarkdownDocument, markdown_view::first_line_ink_top},
    text::fonts::FontRole,
    text::glyph_metrics::capital_middle,
    text::{TextStyle, draw_text, draw_text_wrapped},
    theme::Theme,
};

/// A widget right behind the title (e.g. memo's kind dropdown), centered on
/// the title's capitals; it moves below a title that leaves no room.
pub struct TitleTrailing<'a> {
    /// The widget's size, measured before drawing.
    pub size: [f32; 2],
    /// Whether the widget sits at the right end of the title row instead of
    /// right behind the title (memo's compact view toggle, M139).
    pub at_row_end: bool,
    /// Draws the widget at the cursor.
    pub draw: &'a mut dyn FnMut(&Ui),
}

/// What the first line of a page's content is, so its highest ink keeps
/// `space.sm` from the divider above (R148).
#[derive(Clone, Copy)]
pub enum FirstLine<'a> {
    /// Running text in this font, with the typography's leading above it.
    Running(FontRole),
    /// One label drawn with `draw_text` in this style, without leading.
    Label(TextStyle),
    /// A markdown document drawn at the content's top; its first line is
    /// measured as it will wrap, checkboxes and chips included (Q525).
    Markdown {
        /// The document.
        document: &'a MarkdownDocument,
        /// The texts its chat-link chips show.
        chips: ChipContext<'a>,
    },
}

impl FirstLine<'_> {
    /// How far below the top of the content the highest ink of this line
    /// is; call it where the content will be drawn.
    fn ink_top(self, ui: &Ui, theme: &Theme) -> f32 {
        match self {
            Self::Running(font) => capital_top(ui, theme, font, LineSpacing::Running),
            Self::Label(style) => capital_top(ui, theme, style.font(), LineSpacing::Label),
            Self::Markdown { document, chips } => first_line_ink_top(ui, document, theme, chips),
        }
    }
}

/// What the head of a page shows; the first line may borrow from shorter-lived
/// data than the title widget (`'line`).
pub struct PageHead<'a, 'line> {
    /// The title, drawn in Heading1.
    pub title: &'a str,
    /// The widget behind the title.
    pub trailing: Option<TitleTrailing<'a>>,
    /// The Secondary line below the title, e.g. a help topic's lead
    /// sentence or the state of a read-only note.
    pub subtitle: Option<&'a str>,
    /// The content's first line, which sets the gap below the separator.
    pub first_line: FirstLine<'line>,
}

/// Draws `head`, the separator and then `content` in the borderless scroll
/// region `id` that fills the rest of the pane, as one stack (R148); returns
/// what `content` returned, or `None` if the region is clipped and `content`
/// did not run.
pub fn draw_page<R>(
    ui: &Ui,
    theme: &Theme,
    id: &str,
    head: PageHead<'_, '_>,
    content: impl FnOnce() -> R,
) -> Option<R> {
    Stack::vertical(theme).build(ui, |stack| {
        stack.add(|| draw_title(ui, theme, head.title, head.trailing));
        if let Some(subtitle) = head.subtitle {
            stack.add(|| draw_text_wrapped(ui, theme, TextStyle::Secondary, subtitle));
        }
        stack.add_separator_before_region(theme);
        // Only the content scrolls, so its scrollbar starts below the
        // separator. The region starts at the line, so scrolled text stops
        // there, and moves its first line so that line's highest ink keeps
        // `space.sm` from the line, measured inside the region at its width.
        stack.add(|| {
            ChildWindow::new(id)
                .size([0.0, 0.0])
                .draw_background(false)
                .build(ui, || {
                    let shift = theme.spacing.small - head.first_line.ink_top(ui, theme);
                    let [x, y] = ui.cursor_pos();
                    ui.set_cursor_pos([x, (y + shift).round()]);
                    content()
                })
        })
    })
}

/// Draws the title with `trailing` right behind it (or at the row end),
/// centered on the title's capitals (memo M102); the widget moves below a
/// title that leaves no room.
fn draw_title(ui: &Ui, theme: &Theme, title: &str, trailing: Option<TitleTrailing<'_>>) {
    let trailing_size = trailing.as_ref().map_or([0.0, 0.0], |widget| widget.size);
    let gap = theme.spacing.medium;
    let available = ui.content_region_avail()[0];
    let [left, top] = ui.cursor_screen_pos();
    let (title_middle, is_beside) = {
        let _heading = TextStyle::Heading1.push(ui, theme);
        let middle = capital_middle().unwrap_or(ui.text_line_height() / 2.0);
        let is_beside = ui.calc_text_size(title)[0] + gap + trailing_size[0] <= available;
        (middle, is_beside)
    };
    if is_beside {
        draw_text(ui, theme, TextStyle::Heading1, title);
    } else {
        draw_text_wrapped(ui, theme, TextStyle::Heading1, title);
    }
    let Some(trailing) = trailing else {
        return;
    };
    if is_beside {
        ui.same_line_with_spacing(0.0, gap);
        let x = if trailing.at_row_end {
            left + available - trailing_size[0]
        } else {
            ui.cursor_screen_pos()[0]
        };
        ui.set_cursor_screen_pos([x, (top + title_middle - trailing_size[1] / 2.0).round()]);
    }
    (trailing.draw)(ui);
}
