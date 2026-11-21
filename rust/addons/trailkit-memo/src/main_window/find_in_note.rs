//! Find in the open note (memo plan M198): the state of the find bar below
//! the note toolbar, opening it with Ctrl+F, stepping through the hits with
//! wrap-around, and how it drives the reading view and the editor.

use nexus::imgui::Ui;
use trailkit_ui::{
    controls::search::{
        bar::{CountPlacement, SearchBarTexts, SearchStatus, draw_search_bar},
        field::{SearchField, SearchFieldTexts},
    },
    text::text_search::SearchQuery,
    theme::Theme,
};

use trailkit_memo_search::i18n::SearchText;
use trailkit_memo_shared::{
    i18n::{Language, LocalizedText},
    note_tree::NoteId,
};

use super::MemoWindow;

/// State of the find bar that outlives a frame.
#[derive(Debug, Default)]
pub(super) struct FindInNote {
    /// Whether the bar is open.
    pub(super) visible: bool,
    /// The find field; its text survives closing (the next Ctrl+F offers it).
    pub(super) field: SearchField,
    /// The parsed phrase of the field text, `None` while it is blank.
    pub(super) query: Option<SearchQuery>,
    /// The current hit, 0-based in reading or text order.
    pub(super) current: usize,
    /// How many hits the shown note had in the last frame.
    pub(super) hit_count: usize,
    /// The current hit must be scrolled into view (and selected in the
    /// editor) once the hits are known.
    pub(super) reveal_pending: bool,
    /// The note the hits belong to; another note starts at its first hit.
    note: Option<NoteId>,
}

impl FindInNote {
    /// Parses the field text again and starts at the first hit.
    fn text_changed(&mut self) {
        self.query = SearchQuery::phrase(&self.field.text);
        self.current = 0;
        self.reveal_pending = true;
    }

    /// The bar's status for the count and the arrows.
    fn status(&self) -> SearchStatus {
        match (&self.query, self.hit_count) {
            (None, _) => SearchStatus::Empty,
            (Some(_), 0) => SearchStatus::NoResults,
            (Some(_), _) => SearchStatus::Hits,
        }
    }
}

impl MemoWindow {
    /// Whether Ctrl+F belongs to the window (M198): it has the focus and a
    /// note is open, in the reading view or the editor; without an open note
    /// Ctrl+F does nothing (Q513).
    #[must_use]
    pub fn takes_find_key(&self) -> bool {
        self.window_focused && self.finds_in_shown_note()
    }

    /// Whether F3 belongs to the window: it has the focus and the find bar
    /// is shown (M198).
    #[must_use]
    pub fn takes_find_step_key(&self) -> bool {
        self.window_focused && self.find_bar_shown()
    }

    /// Whether a note is open that find could search: a selected note, the
    /// edited one, or a trashed note in the trash preview (mock v9).
    fn finds_in_shown_note(&self) -> bool {
        if self.trash.visible {
            self.trash_preview_note().is_some()
        } else {
            self.editor.is_some() || self.selected_note().is_some()
        }
    }

    /// The note find searches: the trash preview's, the edited or the shown one.
    fn find_target(&self) -> Option<NoteId> {
        if self.trash.visible {
            return self.trash_preview_note();
        }
        self.editor
            .as_ref()
            .map(|editor| editor.session().note)
            .or(self.ui_state.shown_note)
    }

    /// Whether the find bar is open over a note it can search.
    pub(super) fn find_bar_shown(&self) -> bool {
        self.find.visible && self.finds_in_shown_note()
    }

    /// Opens the find bar after Ctrl+F (M198) and focuses its field with the
    /// whole text selected. The field offers the editor's selected text, else
    /// the search of the list beside it, else the last find (Q507).
    pub fn open_find(&mut self) {
        if !self.finds_in_shown_note() {
            return;
        }
        let offered = self
            .editor
            .as_ref()
            .map(|editor| editor.selected_body_text())
            .filter(|text| !text.trim().is_empty() && !text.contains('\n'))
            .or_else(|| {
                let field = if self.trash.visible {
                    &self.trash.search_field
                } else {
                    &self.search_field
                };
                let search = field.text.trim();
                (!search.is_empty()).then_some(search)
            })
            .map(str::to_owned);
        if let Some(text) = offered
            && text != self.find.field.text
        {
            self.find.field.set_text(&text);
            self.find.text_changed();
        }
        if self.find.query.is_none() {
            self.find.text_changed();
        }
        // Another note than last time starts at its first hit.
        let note = self.find_target();
        if note != self.find.note {
            self.find.note = note;
            self.find.current = 0;
        }
        self.find.visible = true;
        self.find.reveal_pending = true;
        self.find.field.request_focus();
    }

    /// Opens the find bar with `text` in its field, as if typed (tests and
    /// snapshots).
    #[cfg(test)]
    pub(super) fn find_text(&mut self, text: &str) {
        self.open_find();
        self.find.field.set_text(text);
        self.find.text_changed();
    }

    /// Ctrl+F (M198): opens the find bar, or closes it while it is open.
    pub fn toggle_find_by_key(&mut self) {
        if self.find_bar_shown() {
            self.close_find();
        } else {
            self.open_find();
        }
    }

    /// Closes the find bar (Ctrl+F, the `x` of its empty field or Escape,
    /// M198, mock v9). In the editor the body
    /// takes the focus back with the current hit still selected.
    pub(super) fn close_find(&mut self) {
        self.find.visible = false;
        if let Some(editor) = self.editor.as_mut() {
            editor.focus_body();
        }
    }

    /// Goes to the next hit (`forward`) or the previous one, wrapping at
    /// either end (Q505).
    pub fn step_find(&mut self, forward: bool) {
        let count = self.find.hit_count;
        if count == 0 {
            return;
        }
        let current = self.find.current.min(count - 1);
        self.find.current = if forward {
            (current + 1) % count
        } else {
            (current + count - 1) % count
        };
        self.find.reveal_pending = true;
    }

    /// Takes the number of hits the shown note has, after drawing it: keeps
    /// the current hit in range and starts at the first hit of another note.
    pub(super) fn record_find_hits(&mut self, note: Option<NoteId>, count: usize) {
        if note != self.find.note {
            self.find.note = note;
            self.find.current = 0;
            self.find.reveal_pending = true;
        }
        self.find.hit_count = count;
        if self.find.current >= count {
            self.find.current = 0;
        }
    }

    /// Draws the find bar if it is shown and applies what the user did:
    /// typing, the arrows, the `x`, and Enter / Shift+Enter in the field.
    pub(super) fn draw_find_bar(&mut self, ui: &Ui, language: Language, theme: &Theme) {
        if !self.find_bar_shown() {
            return;
        }
        let current = (self.find.current + 1).to_string();
        let total = self.find.hit_count.to_string();
        let count = SearchText::FindCount
            .get(language)
            .replace("{current}", &current)
            .replace("{total}", &total);
        let texts = SearchBarTexts {
            field: SearchFieldTexts {
                hint: SearchText::FindHint.get(language),
                clear: SearchText::ClearSearch.get(language),
                close: Some(SearchText::FindClose.get(language)),
            },
            count: &count,
            no_results: SearchText::FindNoResults.get(language),
            previous: SearchText::FindPrevious.get(language),
            next: SearchText::FindNext.get(language),
        };
        let status = self.find.status();
        let response = draw_search_bar(
            ui,
            theme,
            &mut self.find.field,
            status,
            CountPlacement::Beside,
            texts,
        );
        if response.text_changed {
            self.find.text_changed();
        }
        if response.previous {
            self.step_find(false);
        }
        if response.next {
            self.step_find(true);
        }
        if response.close_clicked {
            self.close_find();
        }
    }
}
