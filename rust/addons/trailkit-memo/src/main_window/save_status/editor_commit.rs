//! The editor's "Save" that waits for the disk (memo Q158) and the dialog
//! when the user leaves the editor after a failed save (Q157). The editor's
//! text reaches the window's tree only once it is on disk, so "Discard"
//! never needs a rollback.

use std::sync::Arc;

use jiff::Timestamp;
use trailkit_ui::busy::BusyState;

use trailkit_memo_editor::note_editing::EditSession;
use trailkit_memo_shared::{
    i18n::Language,
    note_tree::{NodeId, NotesDocument},
};

use super::WriteRequester;
use crate::main_window::{MemoWindow, actions::LeaveReason};

/// The editor's "Save" while it waits for the disk (Q158).
#[derive(Debug)]
pub(in crate::main_window) struct EditorCommit {
    /// The note tree with the editor's text, as handed to the worker.
    candidate: Arc<NotesDocument>,
    /// What to do once it is saved (leaving the editor, closing the window).
    then: Option<LeaveReason>,
    /// Whether the write runs or failed.
    pub(super) failed: bool,
}

impl EditorCommit {
    /// Whether the write still runs.
    pub(in crate::main_window) const fn is_running(&self) -> bool {
        !self.failed
    }
}

/// The dialog when the user leaves the editor after a failed save (Q157).
#[derive(Debug, Clone, PartialEq)]
pub(in crate::main_window) struct SaveFailedPrompt {
    /// Where the user wanted to go.
    pub(in crate::main_window) reason: LeaveReason,
    /// "Save again": busy while its write runs, and at least
    /// `MINIMUM_BUSY`. While busy the dialog's other answers are locked.
    pub(in crate::main_window) save: BusyState,
    /// Whether "Save again" failed.
    pub(in crate::main_window) failed_again: bool,
}

impl MemoWindow {
    /// The editor's write failed: the editor stays open (Q158). Leaving the
    /// editor asks with the save-failed dialog.
    pub(super) fn editor_write_failed(&mut self) {
        let Some(commit) = &mut self.editor_commit else {
            return;
        };
        commit.failed = true;
        if let Some(reason) = commit.then.take() {
            self.open_save_failed_prompt(reason);
        }
    }

    /// Starts saving the editor's text (Q158): the worker writes the tree
    /// with the text at once; the text reaches the window's tree only when
    /// it is on disk. `then` runs after a successful save. With nothing to
    /// write the editor closes at once.
    pub(in crate::main_window) fn begin_editor_commit(
        &mut self,
        now: Timestamp,
        then: Option<LeaveReason>,
        language: Language,
    ) {
        let Some(editor) = &self.editor else {
            self.continue_after_leaving(then, language);
            return;
        };
        let mut candidate = self.document.clone();
        let changed = apply_session(&mut candidate, editor.session(), now);
        if !changed && !self.save_status.failing() {
            self.close_editor_saved();
            self.continue_after_leaving(then, language);
            return;
        }
        let candidate = Arc::new(candidate);
        self.save_status.waiting.push_back(WriteRequester::Editor);
        self.saver.save_notes(Arc::clone(&candidate));
        self.saver.write_now();
        self.editor_commit = Some(EditorCommit {
            candidate,
            then,
            failed: false,
        });
    }

    /// The editor's text is on disk: it enters the tree, the editor closes
    /// and the pending way out continues.
    pub(super) fn finish_editor_commit(&mut self, language: Language) {
        let Some(commit) = self.editor_commit.take() else {
            return;
        };
        if let Some(editor) = &self.editor {
            let note = editor.session().note;
            if let (Some(saved), Some(target)) =
                (commit.candidate.note(note), self.document.note_mut(note))
            {
                target.title.clone_from(&saved.title);
                target.body.clone_from(&saved.body);
                target.updated_at = saved.updated_at;
            }
        }
        if self.document == *commit.candidate {
            // Already on disk; only the search index must see the new text.
            self.document_revision = self.document_revision.wrapping_add(1);
        } else {
            self.save_notes();
        }
        self.close_editor_saved();
        self.continue_after_leaving(commit.then, language);
    }

    /// Closes the editor after its text was saved and drops its draft (M55).
    fn close_editor_saved(&mut self) {
        let Some(editor) = self.editor.take() else {
            return;
        };
        if self.drafts.take(editor.session().note).is_some() {
            self.saver.save_drafts(self.drafts.clone());
        }
    }

    /// Continues where the user wanted to go once the editor is left.
    pub(in crate::main_window) fn continue_after_leaving(
        &mut self,
        then: Option<LeaveReason>,
        language: Language,
    ) {
        match then {
            Some(LeaveReason::Action(next)) => self.apply(&next, language),
            Some(LeaveReason::CloseWindow) => self.close_approved = true,
            Some(LeaveReason::Cancel) | None => {}
        }
    }

    /// Whether the editor's "Save" waits for the disk.
    pub(in crate::main_window) fn editor_commit_running(&self) -> bool {
        self.editor_commit
            .as_ref()
            .is_some_and(EditorCommit::is_running)
    }

    /// Whether the editor's last "Save" failed and its text is unchanged.
    pub(in crate::main_window) fn editor_commit_failed(&self) -> bool {
        self.editor_commit
            .as_ref()
            .is_some_and(|commit| commit.failed)
    }

    /// Opens the save-failed dialog for leaving with `reason` (Q157). The
    /// editor's text is no longer written automatically ("the retry ends").
    pub(in crate::main_window) fn open_save_failed_prompt(&mut self, reason: LeaveReason) {
        self.editor_commit = None;
        self.save_notes();
        self.save_failed_prompt = Some(SaveFailedPrompt {
            reason,
            save: BusyState::default(),
            failed_again: false,
        });
    }

    /// Drops a failed "Save" once the user changes the text (Q153): the
    /// tree without the text replaces it at the worker.
    pub(in crate::main_window) fn drop_failed_commit_on_edit(&mut self) {
        let (Some(commit), Some(editor)) = (&self.editor_commit, &self.editor) else {
            return;
        };
        if !commit.failed {
            return;
        }
        let session = editor.session();
        let unchanged = commit.candidate.note(session.note).is_some_and(|note| {
            // An empty title keeps the old one (`apply_session`).
            let title_kept = session.title.trim().is_empty() || note.title == session.title;
            title_kept && note.body == session.body
        });
        if !unchanged {
            self.editor_commit = None;
            self.save_notes();
        }
    }

    /// Whether the save-failed dialog is open and "Save again" is not busy.
    pub(in crate::main_window) fn save_failed_prompt_idle(&self) -> bool {
        self.save_failed_prompt
            .as_ref()
            .is_some_and(|prompt| !prompt.save.is_busy())
    }

    /// "Save again" in the dialog: one write of the editor's text (Q157).
    pub(super) fn retry_from_save_failed_prompt(&mut self) {
        let Some(prompt) = &mut self.save_failed_prompt else {
            return;
        };
        let Some(editor) = &self.editor else {
            return;
        };
        if prompt.save.is_busy() || self.save_status.worker_stopped {
            return;
        }
        prompt.save.start();
        let mut candidate = self.document.clone();
        apply_session(&mut candidate, editor.session(), Timestamp::now());
        let candidate = Arc::new(candidate);
        self.save_status.waiting.push_back(WriteRequester::Dialog);
        self.saver.save_notes(Arc::clone(&candidate));
        self.saver.write_now();
        self.editor_commit = Some(EditorCommit {
            candidate,
            then: Some(prompt.reason.clone()),
            failed: false,
        });
    }
}

#[cfg(test)]
impl MemoWindow {
    /// Puts the editor into a running "Save" without asking the worker, so
    /// a snapshot can show "Saving…" (the real worker answers at once).
    pub(in crate::main_window) fn show_editor_commit_running(&mut self) {
        self.editor_commit = Some(EditorCommit {
            candidate: Arc::new(self.document.clone()),
            then: None,
            failed: false,
        });
    }
}

/// Writes the editor `session`'s title and body into its note in `document`
/// (M52); an empty title keeps the old one. Returns whether the note
/// changed.
pub(in crate::main_window) fn apply_session(
    document: &mut NotesDocument,
    session: &EditSession,
    now: Timestamp,
) -> bool {
    let mut changed = document.rename(NodeId::Note(session.note), &session.title, now);
    if let Some(note) = document.note_mut(session.note)
        && note.body != session.body
    {
        note.body.clone_from(&session.body);
        note.updated_at = now;
        changed = true;
    }
    changed
}
