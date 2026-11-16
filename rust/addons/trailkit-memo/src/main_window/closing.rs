//! Closing the window and unloading the addon with unsaved changes: the
//! close request and its approval (M54, M166, M180), drafts kept on unload
//! (M54, R104) and the card about notes with drafts (M185).

use jiff::Timestamp;
use nexus::imgui::Ui;
use trailkit_ui::theme::Theme;

use trailkit_memo_drafts::draft_overview::DraftOverviewAction;
use trailkit_memo_shared::{i18n::Language, settings::UnsavedChanges};

use super::{
    MemoWindow,
    actions::{LeaveReason, WindowAction},
};

impl MemoWindow {
    /// Whether the editor holds changes that are not saved yet (M54).
    #[must_use]
    pub fn has_unsaved_changes(&self) -> bool {
        self.editor
            .as_ref()
            .is_some_and(|editor| editor.session().has_changes())
    }

    /// Asks to close the window (close button, keybind, quick access icon,
    /// Escape). Returns `true` if it may close now; otherwise the unsaved
    /// changes dialog opens and [`Self::take_close_approval`] reports the
    /// answer later (M54).
    pub fn request_close(&mut self) -> bool {
        // Escape while dragging cancels the drag (trailkit-ui skill §4).
        if self.note_list.cancel_drag() {
            return false;
        }
        // Escape (sent by Nexus as a close request) while the dialog is shown
        // picks the harmless answer (`overlay-dialog-layout`).
        if self.leave_prompt.is_some() {
            self.leave_prompt = None;
            return false;
        }
        // Escape keeps editing in the save-failed dialog, unless it saves (Q157).
        if self.save_failed_prompt.is_some() {
            if self.save_failed_prompt_idle() {
                self.save_failed_prompt = None;
            }
            return false;
        }
        // Escape closes the open window menu first (M166).
        if self.window_menu_open {
            self.window_menu_open = false;
            return false;
        }
        // The window closes once the running save is done (M180).
        if self.editor_commit_running() {
            return false;
        }
        if self.editor_commit_failed() {
            self.open_save_failed_prompt(LeaveReason::CloseWindow);
            return false;
        }
        if self.has_unsaved_changes()
            && self.settings.unsaved_changes == UnsavedChanges::SaveAutomatically
        {
            // Closing needs no language; it only continues an action.
            self.begin_editor_commit(
                Timestamp::now(),
                Some(LeaveReason::CloseWindow),
                Language::default(),
            );
            return false;
        }
        if self.has_unsaved_changes() {
            if self.leave_prompt.is_none() {
                self.leave_prompt = Some(LeaveReason::CloseWindow);
            }
            return false;
        }
        self.editor = None;
        true
    }

    /// Closes the window menu, so the close button closes the window at once
    /// instead of only the menu.
    pub fn close_window_menu(&mut self) {
        self.window_menu_open = false;
    }

    /// Whether the unsaved changes dialog allowed closing the window since the
    /// last call.
    pub fn take_close_approval(&mut self) -> bool {
        std::mem::take(&mut self.close_approved)
    }

    /// Lists the notes with a draft in list order (M185); notes in the trash
    /// do not count. Cheap without drafts, the usual case.
    pub(super) fn collect_unsaved_notes(&mut self) {
        self.unsaved_notes.clear();
        if self.drafts.drafts.is_empty() {
            return;
        }
        let (drafts, unsaved_notes) = (&self.drafts, &mut self.unsaved_notes);
        self.document.for_each_note(|note| {
            if drafts.get(note.id).is_some() {
                unsaved_notes.push(note.id);
            }
        });
    }

    /// Draws the card about notes with unsaved changes (M185). It stays while
    /// a draft is left and it is not closed; while the only such note is the
    /// one shown, it has no "Go to note" (Q354). The button reveals the next
    /// such note after the shown one, wrapping around.
    pub(super) fn draw_draft_overview(&mut self, ui: &Ui, language: Language, theme: &Theme) {
        let shown = self.ui_state.shown_note;
        let count = self.unsaved_notes.len();
        let can_go_to_note =
            !matches!(self.unsaved_notes.as_slice(), [only] if Some(*only) == shown);
        if self
            .draft_overview
            .draw(ui, language, theme, count, can_go_to_note)
            == Some(DraftOverviewAction::GoToNote)
        {
            let after_shown = shown
                .and_then(|shown| {
                    let mut passed = false;
                    let mut next = None;
                    self.document.for_each_note(|note| {
                        if passed && next.is_none() && self.unsaved_notes.contains(&note.id) {
                            next = Some(note.id);
                        }
                        passed |= note.id == shown;
                    });
                    next
                })
                .or_else(|| self.unsaved_notes.first().copied());
            if let Some(note) = after_shown {
                self.actions.push(WindowAction::RevealNote(note));
            }
        }
    }

    /// Keeps unsaved changes as a draft before the addon unloads (M54) or
    /// after it stopped on an error (R104, Q360); the storage worker writes
    /// it. Returns whether a draft was kept.
    pub fn keep_draft_for_unload(&mut self) -> bool {
        let Some(editor) = &self.editor else {
            return false;
        };
        if !editor.session().has_changes() {
            return false;
        }
        let Some(note) = self.document.note(editor.session().note) else {
            return false;
        };
        let draft = editor.session().to_draft(Timestamp::now(), note.updated_at);
        self.drafts.upsert(draft);
        self.saver.save_drafts(self.drafts.clone());
        true
    }
}
