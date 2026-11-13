//! What the user did in the main window (`WindowAction`) and how it is
//! applied: a dispatcher that first protects unsaved edits (M54), then one
//! function per area — tree, editor, view (ROADMAP R76 question 58).

use std::sync::Arc;

use jiff::Timestamp;
use trailkit_ui::{
    markdown,
    markdown_view::{MarkdownAction, open_link},
};

use trailkit_memo_editor::{
    editor::Editor,
    note_editing::{EditSession, FormatCommand},
};
use trailkit_memo_shared::{
    i18n::{Language, LocalizedText, Text},
    note_tree::{
        Group, GroupId, InsertionPoint, NodeId, Note, NoteId, NoteKind, ResetPeriod,
        TreeNode as Node,
    },
    settings::UnsavedChanges,
    undo::UndoStep,
};
use trailkit_memo_trash::trash_view::TrashAction;

use trailkit_memo_shared::i18n::chip_text;
use trailkit_ui::{chat_link::ChipText, toast::Toast};

use super::{MemoWindow, WindowToast, WindowToastKind};

/// Something the user did while the window was drawn.
#[derive(Debug, Clone, PartialEq)]
pub(super) enum WindowAction {
    /// Select a note or group.
    Select(NodeId),
    /// A click on a group's row: selects it and toggles it (M172).
    ClickGroup(GroupId),
    /// A group was expanded or collapsed.
    SetExpanded(GroupId, bool),
    /// Rename a note or group (inline rename committed, M44).
    Rename(NodeId, String),
    /// Create a note at a point, or by the selection for `None` (M43, M96).
    NewNote(Option<InsertionPoint>),
    /// Create a group at a point, or by the selection for `None`.
    NewGroup(Option<InsertionPoint>),
    /// Hide or show the note list column (window menu, M166).
    ToggleNoteList,
    /// Show the search field and focus it (window menu, M166).
    ShowSearch,
    /// Close the search and clear its text (its `x`, M169).
    HideSearch,
    /// Back to the regular notes page from wherever the user is (M166).
    ShowNotes,
    /// The compact note view toggle (M139).
    ToggleCompactNoteView,
    /// Toggle a task checkbox in the shown note.
    Markdown(MarkdownAction),
    /// Open the addon folder in the file explorer.
    OpenFolder,
    /// Start editing the selected note (M51).
    StartEditing,
    /// Start editing the selected note in its title, selected (M161).
    StartRenaming,
    /// Commit the edit ("Save", Ctrl+Enter).
    FinishEditing,
    /// Leave the editor without saving; asks first if something changed.
    CancelEditing,
    /// Move a note or group with its children (drag & drop, "Move to",
    /// M106–M112).
    Move(NodeId, InsertionPoint),
    /// Change a note's reset period (`None` = normal note, M86).
    SetResetPeriod(NoteId, Option<ResetPeriod>),
    /// Apply a formatting button to the editor (M53).
    Format(FormatCommand),
    /// Show or hide the editor preview (M60).
    TogglePreview,
    /// Continue editing a note from its draft (M54).
    RestoreDraft(NoteId),
    /// Delete a note's draft (M54).
    DiscardDraft(NoteId),
    /// Unsaved changes dialog: save, then continue.
    LeaveSaving,
    /// Unsaved changes dialog: discard, then continue.
    LeaveDiscarding,
    /// Unsaved changes dialog: stay in the editor.
    LeaveCancelled,
    /// Collapse (`true`) or expand a group only for the current search
    /// (not saved, M124).
    SetCollapsedInSearch(GroupId, bool),
    /// Empty the search field (empty search result, M126).
    ClearSearch,
    /// Delete a note or group: to the trash, a group with notes asks first
    /// (M133, M134).
    Delete(NodeId),
    /// Show the trash from the window menu or the toast (M158, M166).
    OpenTrash,
    /// Take the latest delete or move back (Ctrl+Z, the toast, M135).
    Undo,
    /// Do the latest undone step again (Ctrl+Y, Ctrl+Shift+Z, M135).
    Redo,
    /// The toast's time is up (M148).
    DismissToast,
    /// Something done in the trash view (M145–M147).
    Trash(TrashAction),
    /// The open delete question was confirmed (M134, M146).
    ConfirmAccepted,
    /// The open delete question was cancelled.
    ConfirmCancelled,
    /// "Try again" in the error toast: one write, or a worker restart (M180).
    RetrySave,
    /// The error toast's close button (Q164).
    CloseSaveToast,
    /// Save-failed dialog: discard the editor's text, then continue (Q157).
    SaveFailedDiscard,
    /// Save-failed dialog: stay in the editor.
    SaveFailedKeepEditing,
    /// Save-failed dialog: write the editor's text once more.
    SaveFailedRetry,
}

impl WindowAction {
    /// Whether this action leaves the editor of `note` (M54): choosing
    /// another entry or creating one. These ask first if something changed.
    pub(super) fn leaves_editor_of(&self, note: NoteId) -> bool {
        match self {
            Self::Select(node) => *node != NodeId::Note(note),
            Self::ClickGroup(_)
            | Self::NewNote(_)
            | Self::NewGroup(_)
            | Self::OpenTrash
            | Self::ShowNotes
            | Self::ShowSearch => true,
            _ => false,
        }
    }
}

/// What happens once the unsaved changes dialog is answered (M54).
#[derive(Debug, Clone, PartialEq)]
pub(super) enum LeaveReason {
    /// Continue with this action.
    Action(WindowAction),
    /// Close the window.
    CloseWindow,
    /// Leave the editor ("Cancel").
    Cancel,
}

impl MemoWindow {
    /// Applies one collected action.
    pub(super) fn apply(&mut self, action: &WindowAction, language: Language) {
        let now = Timestamp::now();
        if !self.leave_editor_for(action, now, language) {
            return;
        }
        match action {
            WindowAction::Select(_)
            | WindowAction::ClickGroup(_)
            | WindowAction::SetExpanded(..)
            | WindowAction::Move(..)
            | WindowAction::Rename(..)
            | WindowAction::NewNote(_)
            | WindowAction::NewGroup(_)
            | WindowAction::SetCollapsedInSearch(..)
            | WindowAction::SetResetPeriod(..) => self.apply_tree_action(action, language, now),
            WindowAction::StartEditing
            | WindowAction::StartRenaming
            | WindowAction::FinishEditing
            | WindowAction::CancelEditing
            | WindowAction::Format(_)
            | WindowAction::TogglePreview
            | WindowAction::RestoreDraft(_)
            | WindowAction::DiscardDraft(_)
            | WindowAction::LeaveSaving
            | WindowAction::LeaveDiscarding
            | WindowAction::LeaveCancelled => self.apply_editor_action(action, language, now),
            WindowAction::ClearSearch
            | WindowAction::ToggleNoteList
            | WindowAction::ShowSearch
            | WindowAction::HideSearch
            | WindowAction::ShowNotes
            | WindowAction::ToggleCompactNoteView
            | WindowAction::Markdown(_)
            | WindowAction::OpenFolder => self.apply_view_action(action, now, language),
            WindowAction::Delete(_)
            | WindowAction::Undo
            | WindowAction::Redo
            | WindowAction::DismissToast
            | WindowAction::OpenTrash
            | WindowAction::Trash(_)
            | WindowAction::ConfirmAccepted
            | WindowAction::ConfirmCancelled => self.apply_trash_action(action, language),
            WindowAction::RetrySave
            | WindowAction::CloseSaveToast
            | WindowAction::SaveFailedDiscard
            | WindowAction::SaveFailedKeepEditing
            | WindowAction::SaveFailedRetry => self.apply_save_action(action, language),
        }
        // Whatever opens the editor, and creating an entry, leaves the
        // compact note view (M139).
        let leaves_compact = self.editor.is_some()
            || matches!(action, WindowAction::NewNote(_) | WindowAction::NewGroup(_));
        if self.ui_state.compact_note_view && leaves_compact {
            self.ui_state.compact_note_view = false;
            self.saver.save_ui_state(self.ui_state);
        }
    }

    /// Protects unsaved edits before `action` leaves the editor (M54, M180):
    /// asks, saves first or opens the save-failed dialog. Returns whether
    /// `action` may run now; otherwise it follows once the question is
    /// answered or the text is on disk.
    fn leave_editor_for(
        &mut self,
        action: &WindowAction,
        now: Timestamp,
        language: Language,
    ) -> bool {
        if let Some(editor) = &self.editor
            && action.leaves_editor_of(editor.session().note)
        {
            if self.leave_prompt.is_some()
                || self.save_failed_prompt.is_some()
                || self.editor_commit_running()
            {
                return false;
            }
            let reason = LeaveReason::Action(action.clone());
            if self.editor_commit_failed() {
                self.open_save_failed_prompt(reason);
                return false;
            }
            match (
                editor.session().has_changes(),
                self.settings.unsaved_changes,
            ) {
                (true, UnsavedChanges::Ask) => {
                    self.leave_prompt = Some(reason);
                    return false;
                }
                (true, UnsavedChanges::SaveAutomatically) => {
                    // The action follows once the text is on disk (M180).
                    self.begin_editor_commit(now, Some(reason), language);
                    return false;
                }
                (false, _) => self.editor = None,
            }
        }
        true
    }

    /// Applies an action on the note tree: selecting, renaming, creating,
    /// moving, expanding and the reset period of a note.
    fn apply_tree_action(&mut self, action: &WindowAction, language: Language, now: Timestamp) {
        match action {
            WindowAction::Select(node) => self.change_selected(*node),
            WindowAction::ClickGroup(group) => {
                self.change_selected(NodeId::Group(*group));
                self.toggle_group(*group);
            }
            WindowAction::SetExpanded(group, expanded) => {
                if let Some(group) = self.document.group_mut(*group)
                    && group.expanded != *expanded
                {
                    group.expanded = *expanded;
                    self.save_notes();
                }
            }
            WindowAction::Move(node, point) => {
                let from = self.document.place(*node);
                if self.document.move_node(*node, *point) {
                    // Moves are undoable (M114, M135).
                    if let (Some(from), Some(to)) = (from, self.document.place(*node)) {
                        self.undo.record(UndoStep::Move {
                            node: *node,
                            from,
                            to,
                        });
                    }
                    self.save_notes();
                }
            }
            WindowAction::Rename(node, title) => {
                if self.document.rename(*node, title, now) {
                    self.save_notes();
                }
            }
            WindowAction::NewNote(point) => {
                // The new entry must be visible: creating clears the search (M126).
                self.search_field.clear(false);
                let point = self.insertion_point(*point);
                let note = Note::new(
                    Text::NewNote.get(language).to_owned(),
                    NoteKind::Normal,
                    now,
                );
                let id = NodeId::Note(note.id);
                self.document.insert(point, Node::Note(note));
                self.change_selected(id);
                self.start_rename_of_new(id, Text::NewNote.get(language));
                self.save_notes();
            }
            WindowAction::NewGroup(point) => {
                self.search_field.clear(false);
                let point = self.insertion_point(*point);
                let group = Group::new(Text::NewGroup.get(language).to_owned());
                let id = NodeId::Group(group.id);
                self.document.insert(point, Node::Group(group));
                self.change_selected(id);
                self.start_rename_of_new(id, Text::NewGroup.get(language));
                self.save_notes();
            }
            WindowAction::SetCollapsedInSearch(group, collapsed) => {
                if *collapsed {
                    self.collapsed_in_search.insert(*group);
                } else {
                    self.collapsed_in_search.remove(group);
                }
            }
            WindowAction::SetResetPeriod(note, period) => {
                let changed = self.document.set_reset_period(*note, *period, now);
                if changed {
                    self.save_notes();
                }
            }
            // The other actions are dispatched by `apply`.
            _ => {}
        }
    }

    /// Applies an action of the editor: starting, saving, cancelling,
    /// formatting, the preview, drafts and the unsaved changes dialog.
    fn apply_editor_action(&mut self, action: &WindowAction, language: Language, now: Timestamp) {
        match action {
            WindowAction::StartEditing | WindowAction::StartRenaming => {
                if self.editor.is_none()
                    && let Some(note) = self.selected_note()
                {
                    let mut editor = Editor::new(EditSession::start(note));
                    if *action == WindowAction::StartRenaming {
                        editor.focus_title();
                    }
                    self.editor = Some(editor);
                }
            }
            WindowAction::FinishEditing => {
                if !self.editor_commit_running() {
                    self.begin_editor_commit(now, None, language);
                }
            }
            WindowAction::CancelEditing => {
                if self.editor_commit_running() || self.save_failed_prompt.is_some() {
                    // Wait for the running save; the dialog answers itself.
                } else if self.editor_commit_failed() {
                    self.open_save_failed_prompt(LeaveReason::Cancel);
                } else if self.has_unsaved_changes()
                    && self.settings.unsaved_changes == UnsavedChanges::Ask
                {
                    self.leave_prompt = Some(LeaveReason::Cancel);
                } else {
                    self.editor = None;
                }
            }
            WindowAction::Format(command) => {
                if let Some(editor) = self.editor.as_mut() {
                    editor.format(*command);
                }
            }
            WindowAction::TogglePreview => {
                self.ui_state.preview_visible = !self.ui_state.preview_visible;
                self.saver.save_ui_state(self.ui_state);
            }
            WindowAction::RestoreDraft(id) => {
                if self.editor.is_none()
                    && let Some(note) = self.document.note(*id)
                    && let Some(draft) = self.drafts.take(*id)
                {
                    self.editor = Some(Editor::new(EditSession::from_draft(note, draft)));
                    self.saver.save_drafts(self.drafts.clone());
                }
            }
            WindowAction::DiscardDraft(id) => {
                if self.drafts.take(*id).is_some() {
                    self.saver.save_drafts(self.drafts.clone());
                }
            }
            WindowAction::LeaveSaving | WindowAction::LeaveDiscarding => {
                let Some(reason) = self.leave_prompt.take() else {
                    return;
                };
                if *action == WindowAction::LeaveSaving {
                    // Continues once the text is on disk (M180).
                    self.begin_editor_commit(now, Some(reason), language);
                } else {
                    self.editor = None;
                    self.continue_after_leaving(Some(reason), language);
                }
            }
            WindowAction::LeaveCancelled => self.leave_prompt = None,
            // The other actions are dispatched by `apply`.
            _ => {}
        }
    }

    /// Applies an action of the window's view: search, note list,
    /// clicks on rendered markdown and the addon folder.
    fn apply_view_action(&mut self, action: &WindowAction, now: Timestamp, language: Language) {
        match action {
            WindowAction::ClearSearch => self.search_field.clear(false),
            WindowAction::ToggleNoteList => {
                self.ui_state.note_list_hidden = !self.ui_state.note_list_hidden;
                self.saver.save_ui_state(self.ui_state);
            }
            WindowAction::ShowSearch => self.show_search_field(),
            WindowAction::HideSearch => self.hide_search_field(),
            WindowAction::ShowNotes => {
                self.trash.visible = false;
                self.ui_state.note_list_hidden = false;
                self.ui_state.compact_note_view = false;
                self.hide_search_field();
            }
            WindowAction::ToggleCompactNoteView => {
                self.ui_state.compact_note_view = !self.ui_state.compact_note_view;
                self.saver.save_ui_state(self.ui_state);
            }
            WindowAction::Markdown(MarkdownAction::ToggleTask(marker)) => {
                let Some(id) = self.ui_state.shown_note else {
                    return;
                };
                let Some(note) = self.document.note_mut(id) else {
                    return;
                };
                if let Some(body) = markdown::toggle_task(&note.body, *marker) {
                    note.body = body;
                    note.updated_at = now;
                    self.save_notes();
                }
            }
            WindowAction::Markdown(MarkdownAction::OpenLink(url)) => open_link(url),
            WindowAction::Markdown(MarkdownAction::ChatLinkCopied(_)) => {
                self.toast = Some(WindowToast {
                    toast: Toast::new(chip_text(ChipText::Copied, language).to_owned()),
                    kind: WindowToastKind::Message,
                });
            }
            WindowAction::OpenFolder => {
                if let Err(error) = std::process::Command::new("explorer")
                    .arg(&self.addon_directory)
                    .spawn()
                {
                    log::warn!(error:% = error; "folder_open_failed");
                }
            }
            // The other actions are dispatched by `apply`.
            _ => {}
        }
    }

    /// Expands or collapses `group`, during a search only for that search (M124).
    fn toggle_group(&mut self, group: GroupId) {
        if self.search.current.is_some() {
            if !self.collapsed_in_search.remove(&group) {
                self.collapsed_in_search.insert(group);
            }
        } else if let Some(group) = self.document.group_mut(group) {
            group.expanded = !group.expanded;
            self.save_notes();
        }
    }

    /// Makes `node` the selected entry and remembers it. A note also moves into
    /// the note pane, shown from its top; a group leaves the pane as it is (M173).
    pub(super) fn change_selected(&mut self, node: NodeId) {
        if self.ui_state.selected == Some(node) {
            return;
        }
        self.ui_state.selected = Some(node);
        if let NodeId::Note(note) = node
            && self.ui_state.shown_note != Some(note)
        {
            self.ui_state.shown_note = Some(note);
            self.ui_state.note_scroll = 0.0;
            self.note_view
                .show_other_note(self.search.current.is_some());
        }
        self.saver.save_ui_state(self.ui_state);
    }

    /// Resolves where a new entry goes: `point` from a context menu (M96),
    /// or for the toolbar buttons the end of the selected group (M43).
    pub(super) fn insertion_point(&self, point: Option<InsertionPoint>) -> InsertionPoint {
        point.unwrap_or_else(|| {
            InsertionPoint::End(self.document.insertion_parent(self.ui_state.selected))
        })
    }

    /// Puts a just-created note or group into rename mode (M43).
    pub(super) fn start_rename_of_new(&mut self, node: NodeId, title: &str) {
        // The rename happens in the list, so it must be shown.
        if self.ui_state.note_list_hidden {
            self.ui_state.note_list_hidden = false;
            self.saver.save_ui_state(self.ui_state);
        }
        self.note_list.start_rename(node, title.to_owned());
    }

    /// Hands a copy of the note tree to the storage worker and marks the
    /// search index stale.
    pub(super) fn save_notes(&mut self) {
        self.document_revision = self.document_revision.wrapping_add(1);
        self.saver.save_notes(Arc::new(self.document.clone()));
    }
}
