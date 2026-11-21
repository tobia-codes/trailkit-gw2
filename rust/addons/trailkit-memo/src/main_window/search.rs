//! The search above the note list as the window runs it: the cached
//! [`ListSearch`] (M119), Ctrl+F (M124, M127), Escape in the field and
//! stepping through the hits of every matching note with the arrows (BL30,
//! mock v9).

use nexus::imgui::Ui;

use trailkit_memo_search::list_search::SearchUpdate;
use trailkit_memo_shared::note_tree::{NodeId, NoteId};
use trailkit_ui::text::search_steps::HitStep;

use super::MemoWindow;

/// Windows virtual-key code of `F` (Ctrl+F focuses the search, M124).
pub(super) const VIRTUAL_KEY_F: i32 = 0x46;

impl MemoWindow {
    /// Whether Escape should clear the search instead of closing the window:
    /// the focused search field of the note list or the trash holds text
    /// (M124, BL31) and no menu lies above it.
    #[must_use]
    pub(in crate::main_window) fn search_takes_escape(&self) -> bool {
        !self.window_menu_open
            && [&self.search_field, &self.trash.search_field]
                .into_iter()
                .any(|field| field.is_active() && !field.text.is_empty())
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

    /// Opens the note search after Shift twice in the help window (Q582):
    /// the main window comes forward with the search field focused. The
    /// editor keeps the key, as it hides the note list (M167).
    pub fn open_search_by_key(&mut self) {
        if self.editor.is_some() {
            return;
        }
        self.window_menu_open = false;
        self.show_search_field();
    }

    /// Clears the search after Escape (see [`Self::search_takes_escape`]);
    /// the field keeps the focus, so a second Escape closes the window.
    pub(in crate::main_window) fn clear_search_by_escape(&mut self) {
        if self.trash.search_field.is_active() {
            self.trash.search_field.clear(true);
        } else {
            self.search_field.clear(true);
        }
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

    /// Brings the search up to date with the field text, the notes and the
    /// open note (M119). Cheap when nothing changed.
    pub(super) fn update_search(&mut self) {
        let update = self.search.update(
            &self.search_field.text,
            &self.document,
            self.document_revision,
            self.ui_state.shown_note,
        );
        match update {
            SearchUpdate::Ended => self.collapsed_in_search.clear(),
            SearchUpdate::NewQuery => {
                // A new search starts with every matching group expanded and
                // jumps to the first hit of the open note (M122).
                self.collapsed_in_search.clear();
                self.note_view.jump_to_first_hit();
            }
            SearchUpdate::Unchanged | SearchUpdate::Refreshed => {}
        }
    }

    /// Goes to the next hit (`forward`) or the previous one over all
    /// matching notes, wrapping at either end (BL30, Q505): inside the shown
    /// note first, then into the next or previous note in list order, which
    /// is selected, shown and revealed in the list.
    pub(super) fn step_list_search(&mut self, forward: bool) {
        match self.search.step(self.ui_state.shown_note, forward) {
            Some(HitStep::InsidePage) => self.note_view.reveal_current_hit(),
            Some(HitStep::ToPage { page: note, .. }) => self.show_search_stop(note),
            None => {}
        }
    }

    /// Selects and shows `note` during a search: the groups collapsed in the
    /// search around it open, and the list scrolls to its row (M185); the
    /// saved expand state stays untouched (M124).
    fn show_search_stop(&mut self, note: NoteId) {
        let node = NodeId::Note(note);
        for group in self.document.ancestors(node) {
            self.collapsed_in_search.remove(&group);
        }
        self.change_selected(node);
        self.note_list.reveal(node);
        self.note_view.reveal_current_hit();
    }
}
