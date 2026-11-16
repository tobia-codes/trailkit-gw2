//! The head of a page inside a pane (ROADMAP R101): the title in Heading1
//! with an optional widget behind it (memo M102), an optional Secondary line
//! below it, the separator, and the content in its own scroll region, so
//! title and separator stay in place (memo M81). Used by memo's note view
//! and trash preview and by the help window's topics and problem page.

use nexus::imgui::{ChildWindow, Ui};

use crate::{
    fonts::FontRole,
    glyph_metrics::capital_middle,
    separator::draw_separator_above_text,
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

/// What the head of a page shows.
pub struct PageHead<'a> {
    /// The title, drawn in Heading1.
    pub title: &'a str,
    /// The widget behind the title.
    pub trailing: Option<TitleTrailing<'a>>,
    /// The Secondary line below the title, e.g. a help topic's lead
    /// sentence or the state of a read-only note.
    pub subtitle: Option<&'a str>,
    /// The font of the content's first line, which sets the gap below the
    /// separator (R83).
    pub first_line: FontRole,
}

/// Draws `head`, the separator and then `content` in the borderless scroll
/// region `id` that fills the rest of the pane; returns what `content`
/// returned, or `None` if the region is clipped and `content` did not run.
pub fn draw_page<R>(
    ui: &Ui,
    theme: &Theme,
    id: &str,
    head: PageHead<'_>,
    content: impl FnOnce() -> R,
) -> Option<R> {
    draw_title(ui, theme, head.title, head.trailing);
    if let Some(subtitle) = head.subtitle {
        draw_text_wrapped(ui, theme, TextStyle::Secondary, subtitle);
    }
    draw_separator_above_text(ui, theme, head.first_line);
    // Only the content scrolls, so its scrollbar starts below the separator.
    ChildWindow::new(id)
        .size([0.0, 0.0])
        .draw_background(false)
        .build(ui, content)
}

/// Draws the title with `trailing` right behind it (or at the row end),
/// centered on the title's capitals (memo M102); the widget moves below a
/// title that leaves no room.
fn draw_title(ui: &Ui, theme: &Theme, title: &str, trailing: Option<TitleTrailing<'_>>) {
    let trailing_size = trailing.as_ref().map_or([0.0, 0.0], |widget| widget.size);
    let gap = theme.spacing.medium;
    let available = ui.content_region_avail()[0];
    let [left, top] = ui.cursor_screen_pos();
    let (title_height, title_middle, is_beside) = {
        let _heading = TextStyle::Heading1.push(ui, theme);
        let height = ui.text_line_height();
        let middle = capital_middle().unwrap_or(height / 2.0);
        let is_beside = ui.calc_text_size(title)[0] + gap + trailing_size[0] <= available;
        (height, middle, is_beside)
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
    if is_beside {
        // Continue below the title, not below the centered widget.
        let spacing = ui.clone_style().item_spacing[1];
        ui.set_cursor_screen_pos([left, top + title_height + spacing]);
    }
}
