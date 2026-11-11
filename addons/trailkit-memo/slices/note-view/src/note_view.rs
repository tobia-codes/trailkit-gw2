//! The reading view of a note: title (with an optional trailing widget,
//! M102), separator and the rendered content in its own scroll region, so
//! the title stays in place (M81). Search hits are marked (M122).

use nexus::imgui::{ChildWindow, MouseButton, Ui};
use trailkit_memo_shared::{
    i18n::{Language, LocalizedText},
    note_tree::{Note, NoteId},
};
use trailkit_ui::{
    chat_link_chip::ChipContext,
    fonts::FontRole,
    glyph_metrics::capital_middle,
    markdown::{self, MarkdownDocument},
    markdown_view::{MarkdownAction, SearchHighlight, draw_markdown, first_line_font},
    separator::draw_separator_above_text,
    text::{TextStyle, draw_text, draw_text_wrapped},
    text_search::SearchQuery,
    theme::Theme,
};

use crate::i18n::NoteViewText;

/// The parsed markdown of the shown note, re-parsed only when it changes.
#[derive(Debug)]
struct ParsedNote {
    /// The note the cache belongs to.
    note: NoteId,
    /// The body the document was parsed from.
    source: String,
    /// The parsed body.
    document: MarkdownDocument,
}

/// A widget another slice puts right behind the title (e.g. the kind
/// dropdown), centered on the title's capitals; it moves below a title that
/// leaves no room.
pub struct TitleTrailing<'a> {
    /// The widget's size, measured before drawing.
    pub size: [f32; 2],
    /// Whether the widget sits at the right end of the title row instead of
    /// right behind the title (the compact view toggle, memo M139).
    pub at_row_end: bool,
    /// Draws the widget at the cursor.
    pub draw: &'a mut dyn FnMut(&Ui),
}

/// What the view shows this frame.
pub struct NoteViewInput<'a> {
    /// The selected note, `None` when nothing is selected.
    pub note: Option<&'a Note>,
    /// The active search, whose hits are marked.
    pub search: Option<&'a SearchQuery>,
    /// Texts and copy state of chat-link chips.
    pub chips: ChipContext<'a>,
    /// The widget behind the title.
    pub title_trailing: Option<TitleTrailing<'a>>,
    /// A read-only note (in the trash, memo M145): the title keeps its role
    /// (Q280), the `subtitle` line follows it and a double-click does not
    /// edit.
    pub read_only: Option<ReadOnly<'a>>,
}

/// How a read-only note is shown (memo M145).
#[derive(Debug, Clone, Copy)]
pub struct ReadOnly<'a> {
    /// The line below the title, e.g. "In the trash · deleted today".
    pub subtitle: &'a str,
}

/// What the user did in the view.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NoteViewAction {
    /// Double-click on the note: start editing it.
    StartEditing,
    /// A click on rendered markdown (task, link, chat-link chip).
    Markdown(MarkdownAction),
}

/// The result of drawing the view for one frame.
#[derive(Debug, Default)]
pub struct NoteViewOutput {
    /// The user's actions, in order.
    pub actions: Vec<NoteViewAction>,
    /// Whether the content's scroll position changed and should be saved.
    pub scroll_changed: bool,
}

/// State of the view that outlives a frame.
#[derive(Debug)]
pub struct NoteView {
    /// The parsed markdown of the shown note.
    parsed: Option<ParsedNote>,
    /// Apply the saved scroll position on the next frame (after loading or
    /// selecting a note).
    restore_scroll: bool,
    /// Scroll to the first search hit on the next frame (M122).
    scroll_to_first_hit: bool,
}

impl Default for NoteView {
    /// A view that restores the saved scroll position on its first frame.
    fn default() -> Self {
        Self {
            parsed: None,
            restore_scroll: true,
            scroll_to_first_hit: false,
        }
    }
}

impl NoteView {
    /// Another note is shown: apply its scroll position on the next frame
    /// and, during a search, jump to its first hit.
    pub fn show_other_note(&mut self, is_searching: bool) {
        self.restore_scroll = true;
        self.scroll_to_first_hit = is_searching;
    }

    /// A new search started: jump to the first hit on the next frame.
    pub fn jump_to_first_hit(&mut self) {
        self.scroll_to_first_hit = true;
    }

    /// Draws the selected note (or a hint without one). `note_scroll` is the
    /// persisted scroll position of the content.
    pub fn draw(
        &mut self,
        ui: &Ui,
        language: Language,
        theme: &Theme,
        input: NoteViewInput<'_>,
        note_scroll: &mut f32,
    ) -> NoteViewOutput {
        let mut output = NoteViewOutput::default();
        let Some(note) = input.note else {
            draw_text_wrapped(
                ui,
                theme,
                TextStyle::Secondary,
                NoteViewText::SelectNote.get(language),
            );
            return output;
        };
        let is_stale = self
            .parsed
            .as_ref()
            .is_none_or(|parsed| parsed.note != note.id || parsed.source != note.body);
        if is_stale {
            self.parsed = Some(ParsedNote {
                note: note.id,
                source: note.body.clone(),
                document: markdown::parse(&note.body),
            });
        }

        // A read-only note keeps its title role; the info line says its
        // state (Q280, `tokens.md` §7.1 rule 1).
        draw_title(ui, theme, &note.title, input.title_trailing);
        if let Some(read_only) = input.read_only {
            draw_text_wrapped(ui, theme, TextStyle::Secondary, read_only.subtitle);
        }
        let first_line = self
            .parsed
            .as_ref()
            .map_or(FontRole::Body, |parsed| first_line_font(&parsed.document));
        draw_separator_above_text(ui, theme, first_line);
        let editable = input.read_only.is_none();
        if editable && ui.is_window_hovered() && ui.is_mouse_double_clicked(MouseButton::Left) {
            output.actions.push(NoteViewAction::StartEditing);
        }
        // Title and separator stay at the top; only the content scrolls, so
        // its scrollbar starts below the separator (M81).
        ChildWindow::new("note_body")
            .size([0.0, 0.0])
            .draw_background(false)
            .build(ui, || {
                self.draw_content(
                    ui,
                    language,
                    theme,
                    (input.search, input.chips, editable),
                    note_scroll,
                    &mut output,
                );
            });
        output
    }

    /// Draws the rendered content in the scrolling region below the title.
    fn draw_content(
        &mut self,
        ui: &Ui,
        language: Language,
        theme: &Theme,
        (search, chips, editable): (Option<&SearchQuery>, ChipContext<'_>, bool),
        note_scroll: &mut f32,
        output: &mut NoteViewOutput,
    ) {
        if editable && ui.is_window_hovered() && ui.is_mouse_double_clicked(MouseButton::Left) {
            output.actions.push(NoteViewAction::StartEditing);
        }
        if self.restore_scroll {
            ui.set_scroll_y(*note_scroll);
            self.restore_scroll = false;
        }
        match &self.parsed {
            Some(parsed) if !parsed.document.blocks.is_empty() => {
                // Search hits are marked; a new search or a newly opened
                // note jumps to the first one (M122).
                let mut highlight = search.map(SearchHighlight::new);
                if let Some(action) =
                    draw_markdown(ui, &parsed.document, theme, chips, highlight.as_mut())
                {
                    output.actions.push(NoteViewAction::Markdown(action));
                }
                if std::mem::take(&mut self.scroll_to_first_hit)
                    && let Some(first_hit) =
                        highlight.as_ref().and_then(SearchHighlight::first_hit_y)
                {
                    ui.set_scroll_y((first_hit - ui.text_line_height()).max(0.0));
                }
            }
            _ => {
                draw_text_wrapped(
                    ui,
                    theme,
                    TextStyle::Secondary,
                    NoteViewText::EmptyNote.get(language),
                );
            }
        }
        let scroll = ui.scroll_y();
        if (scroll - *note_scroll).abs() >= 1.0 {
            *note_scroll = scroll;
            output.scroll_changed = true;
        }
    }
}

/// Draws the note title with `trailing` right behind it (or at the row end),
/// centered on the title's capitals (M102); the widget moves below a title
/// that leaves no room.
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
