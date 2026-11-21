//! Ctrl+C and Ctrl+A for the selected text of the reading view (M217, Q555):
//! copying shows the "Copied" toast like a chip click (user wish 2026-11-21).

use nexus::imgui::Ui;
use trailkit_ui::markdown::markdown_view::MarkdownAction;

use super::{MemoWindow, WindowAction};

/// Virtual-key code of A (Ctrl+A selects the whole note).
const VIRTUAL_KEY_A: i32 = 0x41;
/// Virtual-key code of C (Ctrl+C copies the selection).
const VIRTUAL_KEY_C: i32 = 0x43;

impl MemoWindow {
    /// Whether Ctrl+A belongs to the window: it has the focus, shows a note
    /// to read (no editor), no text field takes the keys and no dialog is
    /// open. The window message callback catches it only then.
    #[must_use]
    pub fn takes_select_all_key(&self) -> bool {
        self.window_focused
            && !self.text_input_active
            && self.editor.is_none()
            && self.leave_prompt.is_none()
            && self.confirmation.is_none()
    }

    /// Whether Ctrl+C belongs to the window: as [`Self::takes_select_all_key`]
    /// and text of the shown note is selected, so the game keeps Ctrl+C
    /// otherwise.
    #[must_use]
    pub fn takes_copy_key(&self) -> bool {
        self.takes_select_all_key() && self.note_view.has_selection()
    }

    /// Ctrl+C and Ctrl+A as ImGui sees them; in game the window message
    /// callback gets them first while they belong to the window.
    pub(super) fn handle_selection_keys(&mut self, ui: &Ui) {
        if !ui.io().key_ctrl {
            return;
        }
        if ui.is_key_index_pressed(VIRTUAL_KEY_C) {
            self.copy_selection_by_key(ui);
        } else if ui.is_key_index_pressed(VIRTUAL_KEY_A) {
            self.select_all_by_key();
        }
    }

    /// Puts the selected text on the clipboard after Ctrl+C and confirms it.
    pub fn copy_selection_by_key(&mut self, ui: &Ui) {
        if self.takes_copy_key()
            && let Some(text) = self.note_view.selected_text()
        {
            ui.set_clipboard_text(&text);
            self.actions
                .push(WindowAction::Markdown(MarkdownAction::Copied(text)));
        }
    }

    /// Selects the whole shown note after Ctrl+A.
    pub fn select_all_by_key(&mut self) {
        if self.takes_select_all_key() {
            self.note_view.select_all();
        }
    }
}
