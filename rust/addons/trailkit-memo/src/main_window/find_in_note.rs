//! Find in the open note (memo plan M198): the state of the find bar below
//! the note toolbar, opening it with Ctrl+F, stepping through the hits with
//! wrap-around, and how it drives the reading view and the editor.

use nexus::imgui::{Key, Ui};
use trailkit_ui::{text::text_search::SearchQuery, theme::Theme};

use trailkit_memo_search::{
    find_bar::{FindBarStatus, FindBarTexts, draw_find_bar},
    i18n::SearchText,
    search_field::SearchField,
};
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
    /// Whether the field had the keyboard focus in the last frame (Enter in
    /// the field ends its activation in the same frame).
    field_was_active: bool,
}

impl FindInNote {
    /// Parses the field text again and starts at the first hit.
    fn text_changed(&mut self) {
        self.query = SearchQuery::phrase(&self.field.text);
        self.current = 0;
        self.reveal_pending = true;
    }

    /// The bar's status for the count and the arrows.
    fn status(&self) -> FindBarStatus {
        match (&self.query, self.hit_count) {
            (None, _) => FindBarStatus::Empty,
            (Some(_), 0) => FindBarStatus::NoResults,
            (Some(_), _) => FindBarStatus::Hits,
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

    /// Whether a note is open that find could search: a selected note or
    /// the edited one, not the trash.
    fn finds_in_shown_note(&self) -> bool {
        !self.trash.visible && (self.editor.is_some() || self.selected_note().is_some())
    }

    /// Whether the find bar is open over a note it can search.
    pub(super) fn find_bar_shown(&self) -> bool {
        self.find.visible && self.finds_in_shown_note()
    }

    /// Opens the find bar after Ctrl+F (M198) and focuses its field with the
    /// whole text selected. The field offers the editor's selected text, else
    /// the list search, else the last find (Q507).
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
                let search = self.search_field.text.trim();
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
        let note = self
            .editor
            .as_ref()
            .map(|editor| editor.session().note)
            .or(self.ui_state.shown_note);
        if note != self.find.note {
            self.find.note = note;
            self.find.current = 0;
        }
        self.find.visible = true;
        self.find.reveal_pending = true;
        self.find.field.request_focus_selecting_all();
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

    /// Closes the find bar (Ctrl+F, its `x` or Escape, M198). In the editor the body
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
    /// typing, the arrows, close, and Enter / Shift+Enter in the field.
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
        let texts = FindBarTexts {
            hint: SearchText::FindHint.get(language),
            count: &count,
            no_results: SearchText::FindNoResults.get(language),
            previous: SearchText::FindPrevious.get(language),
            next: SearchText::FindNext.get(language),
            close: SearchText::FindClose.get(language),
        };
        let status = self.find.status();
        let response = draw_find_bar(ui, theme, &mut self.find.field, status, texts);
        if response.text_changed {
            self.find.text_changed();
        }
        // ImGui ends the field's activation on Enter; it keeps the focus.
        if self.find.field_was_active && ui.is_key_pressed(Key::Enter) {
            self.step_find(!ui.io().key_shift);
            self.find.field.request_focus();
        }
        self.find.field_was_active = self.find.field.is_active();
        if response.previous {
            self.step_find(false);
        }
        if response.next {
            self.step_find(true);
        }
        if response.close {
            self.close_find();
        }
    }
}
