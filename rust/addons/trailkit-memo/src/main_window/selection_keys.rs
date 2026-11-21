//! Ctrl+C and Ctrl+A for the selected text of the reading view (M217, Q555):
//! copying shows the "Copied" toast like a chip click, and any left click in
//! the window ends the selection (user wishes 2026-11-21).

use nexus::imgui::{MouseButton, Ui, WindowHoveredFlags};
use trailkit_memo_shared::i18n::Language;

use super::MemoWindow;

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

    /// Ends the selection on a left click anywhere in the window (user wish
    /// 2026-11-21, replaces "a click in the note" of Q553). Shift+click
    /// extends it instead; a press in the note's text starts a new one there.
    /// Must run before the note view draws.
    pub(super) fn clear_selection_on_click(&mut self, ui: &Ui) {
        if ui.is_mouse_clicked(MouseButton::Left)
            && !ui.io().key_shift
            && ui.is_window_hovered_with_flags(WindowHoveredFlags::ROOT_AND_CHILD_WINDOWS)
        {
            self.note_view.clear_selection();
        }
    }

    /// Ctrl+C and Ctrl+A as ImGui sees them; in game the window message
    /// callback gets them first while they belong to the window.
    pub(super) fn handle_selection_keys(&mut self, ui: &Ui, language: Language) {
        if !ui.io().key_ctrl {
            return;
        }
        if ui.is_key_index_pressed(VIRTUAL_KEY_C) {
            self.copy_selection_by_key(ui, language);
        } else if ui.is_key_index_pressed(VIRTUAL_KEY_A) {
            self.select_all_by_key();
        }
    }

    /// Puts the selected text on the clipboard after Ctrl+C and confirms it
    /// with the toast at once: the window message callback's keys are applied
    /// before the frame's actions are collected, so an action would be lost.
    pub fn copy_selection_by_key(&mut self, ui: &Ui, language: Language) {
        if self.takes_copy_key()
            && let Some(text) = self.note_view.selected_text()
        {
            ui.set_clipboard_text(&text);
            self.show_copied_toast(language);
        }
    }

    /// Selects the whole shown note after Ctrl+A.
    pub fn select_all_by_key(&mut self) {
        if self.takes_select_all_key() {
            self.note_view.select_all();
        }
    }
}
