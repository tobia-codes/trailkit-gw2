//! The search above the note list as the window runs it: cached index and
//! results (M119), Ctrl+F (M124, M127) and Escape in the field.

use nexus::imgui::Ui;
use trailkit_ui::text_search::SearchQuery;

use trailkit_memo_search::note_search::{NoteSearchIndex, SearchResults};
use trailkit_memo_shared::note_tree::NodeId;

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
    /// The node kept visible (the open note, M125).
    pub(super) keep_visible: Option<NodeId>,
    /// What the note list shows.
    pub(super) results: SearchResults,
}

impl MemoWindow {
    /// Whether Escape should clear the search instead of closing the window:
    /// the focused search field holds text (M124) and no menu lies above it.
    #[must_use]
    pub fn search_takes_escape(&self) -> bool {
        !self.window_menu_open
            && self.search_field.is_active()
            && !self.search_field.text.is_empty()
    }

    /// Clears the search after Escape (see [`Self::search_takes_escape`]);
    /// the field keeps the focus, so a second Escape closes the window.
    pub fn clear_search_by_escape(&mut self) {
        self.search_field.clear(true);
    }

    /// Ctrl+F shows and focuses the search field (M124, M138). The search sits
    /// in the note list, which the editor hides, so it does nothing there
    /// (M167): a shortcut must not end an edit.
    pub(super) fn handle_search_keys(&mut self, ui: &Ui) {
        if self.editor.is_some()
            || !(self.window_focused && ui.io().key_ctrl && ui.is_key_index_pressed(VIRTUAL_KEY_F))
        {
            return;
        }
        self.show_search_field();
    }

    /// Shows the search field (M138) with the note list it sits in, and moves
    /// the keyboard focus into it.
    pub(super) fn show_search_field(&mut self) {
        // The search filters the notes, so the trash makes way (M136).
        self.trash.visible = false;
        self.ui_state.search_visible = true;
        self.ui_state.note_list_hidden = false;
        self.saver.save_ui_state(self.ui_state);
        self.search_field.request_focus();
    }

    /// Hides the search field and clears its text, because a hidden field
    /// must not keep filtering the list (M138, M169).
    pub(super) fn hide_search_field(&mut self) {
        self.search_field.clear(false);
        self.ui_state.search_visible = false;
        self.saver.save_ui_state(self.ui_state);
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
        let keep_visible = self.ui_state.selected;
        if let Some(current) = &self.search.current
            && current.revision == revision
            && current.keep_visible == keep_visible
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
        }
        let results = SearchResults::compute(&self.document, index, &query, keep_visible);
        self.search.current = Some(CurrentSearch {
            text: self.search_field.text.clone(),
            query,
            revision,
            keep_visible,
            results,
        });
    }
}
