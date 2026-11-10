//! Save errors in the main window (memo M180): the worker's reports, the
//! error toast, the editor's "Save" that waits for the disk (Q158) and the
//! dialog when the user leaves the editor after a failed save (Q157).
//!
//! The toast shows after the second failed write of a cycle or at once when
//! a write the user asked for fails (Q149, Q163); it stays until the save
//! succeeds, then "Changes saved" replaces it (Q160). A regular toast or an
//! open dialog hides it meanwhile (Q146, Q154); the close button hides it
//! until the next change fails (Q164). The editor's text reaches the note
//! tree only once it is on disk, so "Discard" never needs a rollback.

use std::{collections::VecDeque, sync::Arc, sync::mpsc::TryRecvError};

use jiff::Timestamp;
use nexus::imgui::Ui;
use trailkit_ui::{
    theme::Theme,
    toast::{Toast, ToastAction, ToastExtras, ToastTone, draw_toast_with_extras},
};

use trailkit_memo_editor::note_editing::EditSession;
use trailkit_memo_shared::{
    i18n::{Language, LocalizedText},
    note_tree::{NodeId, NotesDocument},
    save_text::{SaveText, attempt_counter},
    storage_worker::{SaveConnection, SaveHealth, SaveReport},
};

use super::{
    MemoWindow, WindowToast, WindowToastKind,
    actions::{LeaveReason, WindowAction},
};

/// Who asked the worker for a write that is still running; the worker
/// answers in order.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum WriteRequester {
    /// "Try again" in the toast.
    Toast,
    /// The editor's "Save" (or leaving with automatic saving).
    Editor,
    /// "Save again" in the dialog.
    Dialog,
}

/// What the window knows about saving (memo M180).
#[derive(Debug)]
pub(super) struct SaveStatus {
    /// Reports from the storage worker.
    reports: std::sync::mpsc::Receiver<SaveReport>,
    /// The latest state of the shown documents.
    pub(super) health: SaveHealth,
    /// Whether the worker has ended (it panicked, Q151).
    worker_stopped: bool,
    /// Whether the error toast was shown since everything was last saved.
    pub(super) error_shown: bool,
    /// Whether the user closed the error toast (Q164).
    pub(super) closed: bool,
    /// Writes asked for and not answered yet, oldest first.
    waiting: VecDeque<WriteRequester>,
    /// Whether the user asked to restart the stopped worker.
    restart_requested: bool,
    /// The error toast (no timer), kept for its hover state.
    toast: Toast,
}

impl SaveStatus {
    /// The status of a freshly loaded window reading `reports`.
    pub(super) fn new(reports: std::sync::mpsc::Receiver<SaveReport>) -> Self {
        Self {
            reports,
            health: SaveHealth::Healthy,
            worker_stopped: false,
            error_shown: false,
            closed: false,
            waiting: VecDeque::new(),
            restart_requested: false,
            toast: Toast::with_tone(String::new(), ToastTone::Error),
        }
    }

    /// Whether something the user sees is not on disk.
    fn failing(&self) -> bool {
        self.worker_stopped || self.health != SaveHealth::Healthy
    }

    /// Whether the toast's "Try again" runs.
    fn toast_write_running(&self) -> bool {
        self.restart_requested || self.waiting.contains(&WriteRequester::Toast)
    }

    /// Marks the error as shown and the toast as visible again, for a
    /// failure the user caused or must know about at once.
    fn show_error(&mut self) {
        self.error_shown = true;
        self.closed = false;
    }
}

/// The editor's "Save" while it waits for the disk (Q158).
#[derive(Debug)]
pub(super) struct EditorCommit {
    /// The note tree with the editor's text, as handed to the worker.
    candidate: Arc<NotesDocument>,
    /// What to do once it is saved (leaving the editor, closing the window).
    then: Option<LeaveReason>,
    /// Whether the write runs or failed.
    failed: bool,
}

impl EditorCommit {
    /// Whether the write still runs.
    pub(super) const fn is_running(&self) -> bool {
        !self.failed
    }
}

/// The dialog when the user leaves the editor after a failed save (Q157).
#[derive(Debug, Clone, PartialEq)]
pub(super) struct SaveFailedPrompt {
    /// Where the user wanted to go.
    pub(super) reason: LeaveReason,
    /// Whether "Save again" runs.
    pub(super) saving: bool,
    /// Whether "Save again" failed.
    pub(super) failed_again: bool,
}

impl MemoWindow {
    /// Reads the worker's reports and acts on them. Never blocks.
    pub(super) fn poll_save_reports(&mut self, language: Language) {
        loop {
            match self.save_status.reports.try_recv() {
                Ok(SaveReport::Health(health)) => self.apply_save_health(health, language),
                Ok(SaveReport::WriteFinished { succeeded }) => {
                    let requester = self.save_status.waiting.pop_front();
                    self.apply_write_finished(requester, succeeded, language);
                }
                Err(TryRecvError::Empty) => return,
                Err(TryRecvError::Disconnected) => {
                    if !self.save_status.worker_stopped {
                        self.save_status.worker_stopped = true;
                        self.save_status.waiting.clear();
                        self.save_status.show_error();
                        self.editor_write_failed();
                    }
                    return;
                }
            }
        }
    }

    /// Takes a new state of the shown documents.
    pub(super) fn apply_save_health(&mut self, health: SaveHealth, language: Language) {
        let status = &mut self.save_status;
        status.health = health;
        match health {
            SaveHealth::Healthy => {
                if self
                    .editor_commit
                    .as_ref()
                    .is_some_and(|commit| commit.failed)
                {
                    // An automatic write put the editor's text on disk.
                    self.finish_editor_commit(language);
                }
                if std::mem::take(&mut self.save_status.error_shown) {
                    self.save_status.closed = false;
                    self.toast = Some(WindowToast {
                        toast: Toast::with_tone(
                            SaveText::ChangesSaved.get(language).to_owned(),
                            ToastTone::Success,
                        ),
                        kind: WindowToastKind::Message,
                    });
                }
            }
            SaveHealth::Retrying { failed_attempts } => {
                if failed_attempts == 1 {
                    // The next change failed: a closed toast returns (Q164).
                    status.closed = false;
                }
                if failed_attempts >= 2 {
                    // The same change still fails: a closed toast stays closed.
                    status.error_shown = true;
                }
            }
            SaveHealth::GaveUp => status.error_shown = true,
        }
    }

    /// Takes the result of a write someone asked for.
    fn apply_write_finished(
        &mut self,
        requester: Option<WriteRequester>,
        succeeded: bool,
        language: Language,
    ) {
        match requester {
            Some(WriteRequester::Editor | WriteRequester::Dialog) if succeeded => {
                self.save_failed_prompt = None;
                self.finish_editor_commit(language);
            }
            Some(WriteRequester::Editor) => {
                self.save_status.show_error();
                self.editor_write_failed();
            }
            Some(WriteRequester::Dialog) => {
                if let Some(prompt) = &mut self.save_failed_prompt {
                    prompt.saving = false;
                    prompt.failed_again = true;
                }
                // No automatic writes of the editor's text after this (Q157).
                self.editor_commit = None;
                self.save_notes();
            }
            Some(WriteRequester::Toast) | None => {}
        }
    }

    /// The editor's write failed: the editor stays open (Q158). Leaving the
    /// editor asks with the save-failed dialog.
    fn editor_write_failed(&mut self) {
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
    pub(super) fn begin_editor_commit(
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
    fn finish_editor_commit(&mut self, language: Language) {
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
    pub(super) fn continue_after_leaving(&mut self, then: Option<LeaveReason>, language: Language) {
        match then {
            Some(LeaveReason::Action(next)) => self.apply(&next, language),
            Some(LeaveReason::CloseWindow) => self.close_approved = true,
            Some(LeaveReason::Cancel) | None => {}
        }
    }

    /// Whether the editor's "Save" waits for the disk.
    pub(super) fn editor_commit_running(&self) -> bool {
        self.editor_commit
            .as_ref()
            .is_some_and(EditorCommit::is_running)
    }

    /// Whether the editor's last "Save" failed and its text is unchanged.
    pub(super) fn editor_commit_failed(&self) -> bool {
        self.editor_commit
            .as_ref()
            .is_some_and(|commit| commit.failed)
    }

    /// Opens the save-failed dialog for leaving with `reason` (Q157). The
    /// editor's text is no longer written automatically ("the retry ends").
    pub(super) fn open_save_failed_prompt(&mut self, reason: LeaveReason) {
        self.editor_commit = None;
        self.save_notes();
        self.save_failed_prompt = Some(SaveFailedPrompt {
            reason,
            saving: false,
            failed_again: false,
        });
    }

    /// Drops a failed "Save" once the user changes the text (Q153): the
    /// tree without the text replaces it at the worker.
    pub(super) fn drop_failed_commit_on_edit(&mut self) {
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

    /// Applies an answer of the error toast or the save-failed dialog.
    pub(super) fn apply_save_action(&mut self, action: &WindowAction, language: Language) {
        match action {
            WindowAction::RetrySave => {
                if self.save_status.worker_stopped {
                    self.save_status.restart_requested = true;
                } else if !self.save_status.toast_write_running() {
                    self.save_status.waiting.push_back(WriteRequester::Toast);
                    self.saver.write_now();
                }
            }
            WindowAction::CloseSaveToast => self.save_status.closed = true,
            WindowAction::SaveFailedDiscard => {
                if let Some(prompt) = self.save_failed_prompt.take() {
                    self.editor = None;
                    self.continue_after_leaving(Some(prompt.reason), language);
                }
            }
            WindowAction::SaveFailedKeepEditing => {
                if self
                    .save_failed_prompt
                    .as_ref()
                    .is_some_and(|prompt| !prompt.saving)
                {
                    self.save_failed_prompt = None;
                }
            }
            WindowAction::SaveFailedRetry => self.retry_from_save_failed_prompt(),
            // The other actions are dispatched by `apply`.
            _ => {}
        }
    }

    /// "Save again" in the dialog: one write of the editor's text (Q157).
    fn retry_from_save_failed_prompt(&mut self) {
        let Some(prompt) = &mut self.save_failed_prompt else {
            return;
        };
        let Some(editor) = &self.editor else {
            return;
        };
        if prompt.saving || self.save_status.worker_stopped {
            return;
        }
        prompt.saving = true;
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

    /// Draws the error toast if it is due and the slot is free: no regular
    /// toast and no dialog (Q146, Q154). Call it after the window's content.
    pub(super) fn draw_save_toast(&mut self, ui: &Ui, language: Language, theme: &Theme) {
        let status = &self.save_status;
        let dialog_open = self.leave_prompt.is_some()
            || self.confirmation.is_some()
            || self.save_failed_prompt.is_some();
        if !status.failing()
            || !status.error_shown
            || status.closed
            || dialog_open
            || self.toast.is_some()
        {
            return;
        }
        let retrying = match status.health {
            SaveHealth::Retrying { failed_attempts } if !status.worker_stopped => {
                Some(failed_attempts)
            }
            _ => None,
        };
        let running = status.toast_write_running();
        let action = ToastAction {
            label: if running {
                SaveText::Saving.get(language)
            } else {
                SaveText::TryAgain.get(language)
            },
            tooltip: None,
            busy: running,
        };
        let actions: &[ToastAction<'_>] = if retrying.is_some() {
            &[]
        } else {
            std::slice::from_ref(&action)
        };
        let extras = ToastExtras {
            counter: retrying.and_then(attempt_counter),
            spinner: retrying.is_some(),
            close_tooltip: Some(SaveText::Close.get(language)),
        };
        let message = SaveText::SaveFailed.get(language);
        if self.save_status.toast.message() != message {
            self.save_status.toast = Toast::with_tone(message.to_owned(), ToastTone::Error);
        }
        let outcome =
            draw_toast_with_extras(ui, theme, &mut self.save_status.toast, actions, extras);
        if outcome.clicked_action.is_some() {
            self.actions.push(WindowAction::RetrySave);
        }
        if outcome.closed {
            self.actions.push(WindowAction::CloseSaveToast);
        }
    }

    /// Whether the addon should restart the stopped storage worker (Q151);
    /// reading it clears the request.
    pub fn take_worker_restart_request(&mut self) -> bool {
        std::mem::take(&mut self.save_status.restart_requested)
    }

    /// Connects the window to a restarted worker (Q151) and writes its
    /// current documents once ("one write per click").
    pub fn reconnect_storage(&mut self, connection: SaveConnection) {
        let SaveConnection { saver, reports } = connection;
        self.saver = saver;
        self.save_status.reports = reports;
        self.save_status.worker_stopped = false;
        self.save_status.health = SaveHealth::Healthy;
        self.saver.save_notes(Arc::new(self.document.clone()));
        self.saver.save_settings(self.settings);
        self.saver.save_ui_state(self.ui_state);
        self.saver.save_drafts(self.drafts.clone());
        self.save_status.waiting.push_back(WriteRequester::Toast);
        self.saver.write_now();
    }

    /// The restart of the stopped worker failed; "Try again" may ask again.
    pub fn storage_restart_failed(&mut self) {
        self.save_status.restart_requested = false;
    }
}

/// Writes the editor `session`'s title and body into its note in `document`
/// (M52); an empty title keeps the old one. Returns whether the note
/// changed.
pub(super) fn apply_session(
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

#[cfg(test)]
impl MemoWindow {
    /// Puts the editor into a running "Save" without asking the worker, so
    /// a snapshot can show "Saving…" (the real worker answers at once).
    pub(super) fn show_editor_commit_running(&mut self) {
        self.editor_commit = Some(EditorCommit {
            candidate: Arc::new(self.document.clone()),
            then: None,
            failed: false,
        });
    }
}
