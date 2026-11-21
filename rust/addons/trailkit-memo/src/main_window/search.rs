//! The search above the note list as the window runs it: cached index and
//! results (M119), Ctrl+F (M124, M127), Escape in the field and stepping
//! through the hits of every matching note with the arrows (BL30, mock v9).

use nexus::imgui::Ui;
use trailkit_ui::text::text_search::SearchQuery;

use trailkit_memo_search::note_search::{HitStop, NoteSearchIndex, SearchResults};
use trailkit_memo_shared::note_tree::{NodeId, NoteId};

use super::MemoWindow;

/// Windows virtual-key code of `F` (Ctrl+F focuses the search, M124).
pub(super) const VIRTUAL_KEY_F: i32 = 0x46;

/// The search index and the results of the current query, each cached until
/// what it depends on changes (M119: never rebuilt per frame).
#[derive(Debug, Default)]
pub(super) struct SearchCache {
    /// The index and the document revision it was built from.
    pub(super) index: Option<(u64, NoteSearchIndex)>,
    /// The results of the current query, if the field holds one.
    pub(super) current: Option<CurrentSearch>,
    /// Which hit of the shown note the arrows are at.
    pub(super) cursor: HitCursor,
}

/// The current hit of the list search inside the shown note (mock v9): the
/// arrows step through it, then on to the next or previous matching note.
#[derive(Debug, Default)]
pub(super) struct HitCursor {
    /// The note the position belongs to; another shown note starts at its
    /// first hit.
    pub(super) note: Option<NoteId>,
    /// The current hit in the note, 0-based in reading order.
    pub(super) current: usize,
    /// How many hits the reading view drew in the note in the last frame;
    /// it replaces the index's count, which cannot see chips.
    pub(super) drawn: Option<usize>,
}

impl HitCursor {
    /// Starts over at the first hit of `note`.
    fn start_at(&mut self, note: Option<NoteId>) {
        self.note = note;
        self.current = 0;
        self.drawn = None;
    }
}

/// Where the shown note's current hit lies among all hits.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) struct HitPosition {
    /// The 1-based place of the current hit, `None` while the shown note is
    /// no stop (e.g. a note shown because its group's title matches).
    pub(super) current: Option<usize>,
    /// All hits of all matching notes.
    pub(super) total: usize,
}

/// Results of one query over one document revision.
#[derive(Debug)]
pub(super) struct CurrentSearch {
    /// The field text the results belong to.
    pub(super) text: String,
    /// The parsed query.
    pub(super) query: SearchQuery,
    /// The document revision the results belong to.
    pub(super) revision: u64,
    /// What the note list shows.
    pub(super) results: SearchResults,
}

impl MemoWindow {
    /// Whether Escape should clear the search instead of closing the window:
    /// the focused search field holds text (M124) and no menu lies above it.
    #[must_use]
    pub(in crate::main_window) fn search_takes_escape(&self) -> bool {
        !self.window_menu_open
            && self.search_field.is_active()
            && !self.search_field.text.is_empty()
    }

    /// Whether Shift pressed twice belongs to this window (M199): it has the
    /// focus, and the editor is not open, which hides the note list the
    /// search sits in (M167, Q512).
    #[must_use]
    pub const fn takes_search_key(&self) -> bool {
        self.window_focused && self.editor.is_none()
    }

    /// Toggles the search after Shift was pressed twice (M199, M176). The
    /// editor keeps the key from ending an edit (M167).
    pub fn toggle_search_by_key(&mut self) {
        if self.editor.is_some() {
            return;
        }
        self.window_menu_open = false;
        if self.ui_state.search_visible && !self.trash.visible {
            self.hide_search_field();
        } else {
            self.show_search_field();
        }
    }

    /// Clears the search after Escape (see [`Self::search_takes_escape`]);
    /// the field keeps the focus, so a second Escape closes the window.
    pub(in crate::main_window) fn clear_search_by_escape(&mut self) {
        self.search_field.clear(true);
    }

    /// Ctrl+F as ImGui sees it; in game `nexus_bridge::catch_window_keys` gets
    /// it first while it belongs to the window (M174). It opens or closes find
    /// in the open note (M198, M199), in the reading view and the editor.
    pub(super) fn handle_search_keys(&mut self, ui: &Ui) {
        if self.window_focused && ui.io().key_ctrl && ui.is_key_index_pressed(VIRTUAL_KEY_F) {
            self.toggle_find_by_key();
        }
    }

    /// Shows the search field (M138) with the note list it sits in, and moves
    /// the keyboard focus into it.
    pub(super) fn show_search_field(&mut self) {
        // The search filters the notes, so the trash makes way (M136).
        self.trash.visible = false;
        self.ui_state.search_visible = true;
        self.ui_state.note_list_hidden = false;
        self.saver.save_ui_state(&self.ui_state);
        self.search_field.request_focus();
    }

    /// Hides the search field and clears its text, because a hidden field
    /// must not keep filtering the list (M138, M169).
    pub(super) fn hide_search_field(&mut self) {
        self.search_field.clear(false);
        self.ui_state.search_visible = false;
        self.saver.save_ui_state(&self.ui_state);
    }

    /// Brings the search index and results up to date with the field text,
    /// the notes and the open note (M119). Cheap when nothing changed.
    pub(super) fn update_search(&mut self) {
        let Some(query) = SearchQuery::parse(&self.search_field.text) else {
            if self.search.current.take().is_some() {
                self.collapsed_in_search.clear();
            }
            return;
        };
        let revision = self.document_revision;
        if let Some(current) = &self.search.current
            && current.revision == revision
            && current.text == self.search_field.text
        {
            return;
        }
        if self
            .search
            .index
            .as_ref()
            .is_none_or(|(built_from, _)| *built_from != revision)
        {
            self.search.index = Some((revision, NoteSearchIndex::build(&self.document)));
        }
        let Some((_, index)) = &self.search.index else {
            return;
        };
        let query_changed = self
            .search
            .current
            .as_ref()
            .is_none_or(|current| current.query != query);
        if query_changed {
            // A new search starts with every matching group expanded and
            // jumps to the first hit of the open note (M122).
            self.collapsed_in_search.clear();
            self.note_view.jump_to_first_hit();
            self.search.cursor.start_at(self.ui_state.shown_note);
        }
        let results = SearchResults::compute(&self.document, index, &query);
        self.search.current = Some(CurrentSearch {
            text: self.search_field.text.clone(),
            query,
            revision,
            results,
        });
    }

    /// The hits of the shown note in this search: the drawn count once the
    /// reading view knows it, else the index's.
    fn stop_hits(&self, stop: &HitStop) -> usize {
        let cursor = &self.search.cursor;
        match cursor.drawn {
            Some(drawn) if cursor.note == Some(stop.note) => drawn.max(1),
            _ => stop.hits,
        }
    }

    /// Where the current hit lies among all hits of the search, `None`
    /// without a search or without a matching note.
    #[must_use]
    pub(super) fn hit_position(&self) -> Option<HitPosition> {
        let stops = self.search.current.as_ref()?.results.hit_stops();
        let mut total = 0;
        let mut current = None;
        for stop in stops {
            let hits = self.stop_hits(stop);
            if Some(stop.note) == self.ui_state.shown_note {
                current = Some(total + self.search.cursor.current.min(hits - 1) + 1);
            }
            total += hits;
        }
        (total > 0).then_some(HitPosition { current, total })
    }

    /// Goes to the next hit (`forward`) or the previous one over all
    /// matching notes, wrapping at either end (BL30, Q505): inside the shown
    /// note first, then into the next or previous note in list order, which
    /// is selected, shown and revealed in the list.
    pub(super) fn step_list_search(&mut self, forward: bool) {
        let Some(current) = &self.search.current else {
            return;
        };
        let stops = current.results.hit_stops();
        if stops.is_empty() {
            return;
        }
        let shown = self.ui_state.shown_note;
        let place = stops.iter().position(|stop| Some(stop.note) == shown);
        if let Some(place) = place {
            let hits = self.stop_hits(&stops[place]);
            let inside = self.search.cursor.current.min(hits - 1);
            let next_inside = if forward {
                (inside + 1 < hits).then_some(inside + 1)
            } else {
                inside.checked_sub(1)
            };
            if let Some(next_inside) = next_inside {
                self.search.cursor.current = next_inside;
                self.note_view.reveal_current_hit();
                return;
            }
        }
        let count = stops.len();
        let target = match (place, forward) {
            (Some(place), true) => (place + 1) % count,
            (Some(place), false) => (place + count - 1) % count,
            (None, true) => 0,
            (None, false) => count - 1,
        };
        let stop = stops[target];
        let last = if forward { 0 } else { stop.hits - 1 };
        self.show_search_stop(stop.note, last);
    }

    /// Selects and shows `note` at its hit `current` during a search: the
    /// groups collapsed in the search around it open, and the list scrolls
    /// to its row (M185); the saved expand state stays untouched (M124).
    fn show_search_stop(&mut self, note: NoteId, current: usize) {
        let node = NodeId::Note(note);
        for group in self.document.ancestors(node) {
            self.collapsed_in_search.remove(&group);
        }
        self.change_selected(node);
        self.note_list.reveal(node);
        self.search.cursor.start_at(Some(note));
        self.search.cursor.current = current;
        self.note_view.reveal_current_hit();
    }

    /// Takes the hits the reading view drew in the shown note: another note
    /// starts at its first hit, and the current hit stays in range.
    pub(super) fn record_search_hits(&mut self, note: Option<NoteId>, drawn: usize) {
        let cursor = &mut self.search.cursor;
        if note != cursor.note {
            cursor.start_at(note);
        }
        cursor.drawn = Some(drawn);
        if drawn > 0 && cursor.current >= drawn {
            cursor.current = drawn - 1;
        }
    }
}
