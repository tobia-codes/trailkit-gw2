//! The reading view of a note: title (with an optional trailing widget,
//! M102), separator and the rendered content in its own scroll region, so
//! the title stays in place (M81), drawn with the shared page head of
//! `trailkit-ui` (R101). Search hits are marked (M122); the mouse selects
//! text to copy (M217).

use nexus::imgui::{MouseButton, Ui};
use trailkit_memo_shared::{
    i18n::{Language, LocalizedText},
    note_tree::{Note, NoteId},
};
use trailkit_ui::{
    chat_link::chat_link_chip::ChipContext,
    layout::page::{PageHead, TitleTrailing, draw_page},
    layout::pane::WindowDrag,
    markdown::markdown_view::{MarkdownAction, SearchHighlight, draw_selectable_markdown},
    markdown::text_selection::TextSelection,
    markdown::{self, MarkdownDocument},
    text::text_search::SearchQuery,
    text::{TextStyle, draw_text_wrapped},
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

/// What the view shows this frame.
pub struct NoteViewInput<'a> {
    /// The selected note, `None` when nothing is selected.
    pub note: Option<&'a Note>,
    /// The active search, whose hits are marked.
    pub search: Option<&'a SearchQuery>,
    /// Find in the note (M198); replaces `search` while it has a query.
    pub find: Option<FindInNote<'a>>,
    /// Texts and copy state of chat-link chips.
    pub chips: ChipContext<'a>,
    /// The widget behind the title.
    pub title_trailing: Option<TitleTrailing<'a>>,
    /// A read-only note (in the trash, memo M145): the title keeps its role
    /// (Q280), the `subtitle` line follows it and a double-click does not
    /// edit.
    pub read_only: Option<ReadOnly<'a>>,
}

/// Find in the open note (M198): what to find and which hit is current.
#[derive(Debug, Clone, Copy)]
pub struct FindInNote<'a> {
    /// The phrase to find.
    pub query: &'a SearchQuery,
    /// The current hit, 0-based in reading order.
    pub current: usize,
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
    /// How many find hits the note shows, while finding (M198).
    pub find_hit_count: Option<usize>,
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
    /// Scroll the current find hit into view on the next frame (M198).
    reveal_current_hit: bool,
    /// The selected text of the shown note (M217).
    selection: TextSelection,
    /// The search or find phrase marked when the selection was last drawn;
    /// another one selects nothing (Q553).
    marked_query: Option<SearchQuery>,
}

impl Default for NoteView {
    /// A view that restores the saved scroll position on its first frame.
    fn default() -> Self {
        Self {
            parsed: None,
            restore_scroll: true,
            scroll_to_first_hit: false,
            reveal_current_hit: false,
            selection: TextSelection::default(),
            marked_query: None,
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

    /// Selects nothing (M217, Q553): the search changed or editing starts.
    pub fn clear_selection(&mut self) {
        self.selection.clear();
    }

    /// Whether text of the shown note is selected, so Ctrl+C has something
    /// to copy (M217).
    #[must_use]
    pub fn has_selection(&self) -> bool {
        self.parsed.is_some() && !self.selection.is_empty()
    }

    /// The selected text as it is copied (Q554), if any.
    #[must_use]
    pub fn selected_text(&self) -> Option<String> {
        let parsed = self.parsed.as_ref()?;
        self.selection.text(&parsed.document)
    }

    /// Selects the whole shown note (Ctrl+A, Q555).
    pub fn select_all(&mut self) {
        if let Some(parsed) = &self.parsed {
            self.selection.select_all(&parsed.document);
        }
    }

    /// The current find hit changed: scroll it into view on the next frame
    /// unless it is visible already (M198).
    pub fn reveal_current_hit(&mut self) {
        self.reveal_current_hit = true;
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
            // Nothing shown, nothing to copy.
            self.parsed = None;
            self.selection.clear();
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
            // Another note selects nothing (Q553); a changed body keeps the
            // selection, clamped to the new text while drawing.
            if self
                .parsed
                .as_ref()
                .is_none_or(|parsed| parsed.note != note.id)
            {
                self.selection.clear();
            }
            self.parsed = Some(ParsedNote {
                note: note.id,
                source: note.body.clone(),
                document: markdown::parse(&note.body),
            });
        }

        let editable = input.read_only.is_none();
        if editable && ui.is_window_hovered() && ui.is_mouse_double_clicked(MouseButton::Left) {
            output.actions.push(NoteViewAction::StartEditing);
        }
        // A read-only note keeps its title role; the info line says its
        // state (Q280, `tokens.md` §7.1 rule 1). Title and separator stay at
        // the top; only the content scrolls (M81).
        let head = PageHead {
            title: &note.title,
            trailing: input.title_trailing,
            subtitle: input.read_only.map(|read_only| read_only.subtitle),
        };
        draw_page(
            ui,
            theme,
            "note_body",
            head,
            WindowDrag::KeepsWindow,
            || {
                self.draw_content(
                    ui,
                    language,
                    theme,
                    (input.search, input.find, input.chips, editable),
                    note_scroll,
                    &mut output,
                );
            },
        );
        output
    }

    /// Draws the rendered content in the scrolling region below the title.
    fn draw_content(
        &mut self,
        ui: &Ui,
        language: Language,
        theme: &Theme,
        (search, find, chips, editable): (
            Option<&SearchQuery>,
            Option<FindInNote<'_>>,
            ChipContext<'_>,
            bool,
        ),
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
                // note jumps to the first one (M122). Find in the note marks
                // its own hits instead, the current one filled (M198).
                let marked = find.map(|find| find.query).or(search);
                if marked != self.marked_query.as_ref() {
                    self.selection.clear();
                    self.marked_query = marked.cloned();
                }
                let mut highlight = match find {
                    Some(find) => Some(SearchHighlight::with_current(find.query, find.current)),
                    None => search.map(SearchHighlight::new),
                };
                let action = draw_selectable_markdown(
                    ui,
                    &parsed.document,
                    theme,
                    chips,
                    highlight.as_mut(),
                    &mut self.selection,
                );
                if let Some(action) = action {
                    output.actions.push(NoteViewAction::Markdown(action));
                }
                if find.is_some() {
                    output.find_hit_count = highlight.as_ref().map(SearchHighlight::hit_count);
                    if std::mem::take(&mut self.reveal_current_hit)
                        && let Some(hit) =
                            highlight.as_ref().and_then(SearchHighlight::current_hit_y)
                    {
                        reveal(ui, hit);
                    }
                } else if std::mem::take(&mut self.scroll_to_first_hit)
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

/// Scrolls the content so the line at window-content `y` is visible, a
/// third down the view, unless it is visible already (M198).
fn reveal(ui: &Ui, y: f32) {
    let line = ui.text_line_height();
    let (top, height) = (ui.scroll_y(), ui.window_size()[1]);
    if y < top || y + line > top + height {
        ui.set_scroll_y((y - height / 3.0).max(0.0));
    }
}
