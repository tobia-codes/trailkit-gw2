//! Deleting, the trash and undo as the main window runs them (memo M12,
//! M133–M151): the delete flow with its questions, the selection after it,
//! the trash view's actions, Ctrl+Z / Ctrl+Y and the hourly purge. Drawing
//! the trash and the toast lives in `trash_pane`.

use jiff::{SignedDuration, Timestamp};
use nexus::imgui::Ui;
use trailkit_ui::toast::Toast;

use trailkit_memo_shared::{
    i18n::{Language, LocalizedText},
    note_tree::{NodeId, TrashEntry, TreeNode},
    undo::UndoStep,
};
use trailkit_memo_trash::{i18n::TrashText, labels, trash_view::TrashAction};

use super::{MemoWindow, WindowAction};

/// How often the trash is checked for expired entries (M150).
const PURGE_INTERVAL: SignedDuration = SignedDuration::from_hours(1);

/// Windows virtual-key code of `Y` (Ctrl+Y redoes, M135).
const VIRTUAL_KEY_Y: i32 = 0x59;
/// Windows virtual-key code of `Z` (Ctrl+Z undoes, M135).
const VIRTUAL_KEY_Z: i32 = 0x5A;

/// What a delete question asks about.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum ConfirmationKind {
    /// Move a group with notes to the trash (M134).
    DeleteGroup(NodeId),
    /// Delete a trash entry for good (M146).
    DeletePermanently(NodeId),
    /// Empty the trash (M146).
    EmptyTrash,
}

/// An open delete question with its texts, formatted once when it opens.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct Confirmation {
    /// What is confirmed.
    pub(super) kind: ConfirmationKind,
    /// The heading.
    pub(super) title: String,
    /// The explanation.
    pub(super) text: String,
    /// The label of the confirm button.
    pub(super) confirm_label: &'static str,
}

impl Confirmation {
    /// Whether the confirmed action can be undone (only then Enter
    /// confirms).
    pub(super) const fn is_undoable(&self) -> bool {
        matches!(self.kind, ConfirmationKind::DeleteGroup(_))
    }
}

/// The trash view's state (M136): whether the trash tool is on and which
/// entry the note pane shows. Not saved; the window opens on the notes.
#[derive(Debug, Default)]
pub(super) struct TrashState {
    /// Whether the trash replaces the note list.
    pub(super) visible: bool,
    /// The entry shown in the note pane.
    pub(super) selected: Option<NodeId>,
    /// Scroll position of a previewed note.
    pub(super) scroll: f32,
    /// When the trash is checked for expired entries next (M150).
    pub(super) next_purge: Option<Timestamp>,
}

impl MemoWindow {
    /// Starts deleting `node` (M133): a group with notes asks first (M134),
    /// everything else goes to the trash at once. The note in the editor
    /// and the groups holding it stay (M141).
    pub(super) fn request_delete(&mut self, node: NodeId, language: Language) {
        if self.is_blocked_by_editor(node) {
            return;
        }
        let group_notes = match node {
            NodeId::Group(group) => self
                .document
                .group(group)
                .map(|group| (group.title.clone(), group.note_count())),
            NodeId::Note(_) => None,
        };
        match group_notes {
            Some((title, notes)) if notes > 0 => {
                let (title, text) = labels::delete_group_question(&title, notes, language);
                self.confirmation = Some(Confirmation {
                    kind: ConfirmationKind::DeleteGroup(node),
                    title,
                    text,
                    confirm_label: TrashText::Delete.get(language),
                });
            }
            _ => self.move_to_trash(node, language),
        }
    }

    /// Whether `node` is or holds the note in the editor (M141).
    fn is_blocked_by_editor(&self, node: NodeId) -> bool {
        let Some(edited) = self.editor.as_ref().map(|editor| editor.session().note) else {
            return false;
        };
        match node {
            NodeId::Note(note) => note == edited,
            NodeId::Group(group) => self.document.group(group).is_some_and(|group| {
                group
                    .children
                    .iter()
                    .any(|child| child.contains_note(edited))
            }),
        }
    }

    /// Moves `node` to the trash, records the undo step, moves the
    /// selection on (M149) and shows the toast (M148).
    fn move_to_trash(&mut self, node: NodeId, language: Language) {
        let Some(title) = self.node_title(node) else {
            return;
        };
        let next_selection = self.selection_after_removing(node);
        if !self.document.move_to_trash(node, Timestamp::now()) {
            return;
        }
        if let Some(selection) = next_selection {
            self.change_selection(selection);
        }
        self.undo.record(UndoStep::Delete(node));
        self.toast = Some(Toast::new(labels::deleted_message(&title, language)));
        self.save_notes();
    }

    /// The title of `node` in the tree.
    fn node_title(&self, node: NodeId) -> Option<String> {
        match node {
            NodeId::Note(id) => self.document.note(id).map(|note| note.title.clone()),
            NodeId::Group(id) => self.document.group(id).map(|group| group.title.clone()),
        }
    }

    /// The selection once `node` is gone (M149), if the selection is `node`
    /// or inside it: the next visible row below, else the one above, else
    /// nothing. `None` keeps the selection.
    fn selection_after_removing(&self, node: NodeId) -> Option<Option<NodeId>> {
        let selected = self.ui_state.selected?;
        let mut rows = Vec::new();
        visible_rows(&self.document.root, &mut rows);
        let start = rows.iter().position(|row| *row == node)?;
        let mut inside = Vec::new();
        if let Some(entry) = self.tree_node(node) {
            all_ids(entry, &mut inside);
        }
        if !inside.contains(&selected) {
            return None;
        }
        let below = rows[start..].iter().find(|row| !inside.contains(row));
        Some(below.or_else(|| rows[..start].last()).copied())
    }

    /// The tree node `node` refers to.
    fn tree_node(&self, node: NodeId) -> Option<&TreeNode> {
        let place = self.document.place(node)?;
        let children = match place.parent {
            None => &self.document.root,
            Some(group) => &self.document.group(group)?.children,
        };
        children.get(place.index)
    }

    /// Selects `selection` or clears the selection, and remembers it.
    fn change_selection(&mut self, selection: Option<NodeId>) {
        match selection {
            Some(node) => self.select(node),
            None => {
                self.ui_state.selected = None;
                self.saver.save_ui_state(self.ui_state);
            }
        }
    }

    /// Takes the latest delete or move back (M135) and selects what came
    /// back.
    pub(super) fn undo_last_step(&mut self) {
        self.toast = None;
        if let Some(step) = self.undo.undo(&mut self.document, Timestamp::now()) {
            let (UndoStep::Delete(node) | UndoStep::Move { node, .. }) = step;
            self.select(node);
            self.save_notes();
        }
    }

    /// Does the latest undone step again (M135).
    pub(super) fn redo_last_step(&mut self) {
        self.toast = None;
        let next_selection = match self.undo.next_redo() {
            Some(UndoStep::Delete(node)) => self.selection_after_removing(node),
            _ => None,
        };
        if self
            .undo
            .redo(&mut self.document, Timestamp::now())
            .is_some()
        {
            if let Some(selection) = next_selection {
                self.change_selection(selection);
            }
            self.save_notes();
        }
    }

    /// Ctrl+Z undoes, Ctrl+Y and Ctrl+Shift+Z redo while the window has the
    /// focus and no text input is active (M135).
    pub(super) fn handle_undo_keys(&mut self, ui: &Ui) {
        let io = ui.io();
        if !self.window_focused
            || !io.key_ctrl
            || io.want_text_input
            || self.leave_prompt.is_some()
            || self.confirmation.is_some()
        {
            return;
        }
        if ui.is_key_index_pressed(VIRTUAL_KEY_Z) {
            if io.key_shift {
                self.actions.push(WindowAction::Redo);
            } else {
                self.actions.push(WindowAction::Undo);
            }
        } else if ui.is_key_index_pressed(VIRTUAL_KEY_Y) {
            self.actions.push(WindowAction::Redo);
        }
    }

    /// Applies an action of the trash view or a delete question.
    pub(super) fn apply_trash_action(&mut self, action: &WindowAction, language: Language) {
        match action {
            WindowAction::Delete(node) => self.request_delete(*node, language),
            WindowAction::Undo => self.undo_last_step(),
            WindowAction::Redo => self.redo_last_step(),
            WindowAction::DismissToast => self.toast = None,
            WindowAction::ToggleTrash => {
                self.trash.visible = !self.trash.visible;
                if self.trash.visible {
                    // The trash replaces the list, so the list must show.
                    if self.is_narrow {
                        self.drawer_open = true;
                    } else if self.ui_state.note_list_hidden {
                        self.ui_state.note_list_hidden = false;
                        self.saver.save_ui_state(self.ui_state);
                    }
                }
            }
            WindowAction::Trash(TrashAction::Select(node)) => {
                if self.trash.selected != Some(*node) {
                    self.trash.selected = Some(*node);
                    self.trash.scroll = 0.0;
                    self.note_view.show_other_note(false);
                }
                self.drawer_open = false;
            }
            WindowAction::Trash(TrashAction::Restore(node)) => {
                let next = self.trash_neighbour(*node);
                if self.document.restore_from_trash(*node) {
                    self.trash.selected = next;
                    self.save_notes();
                }
            }
            WindowAction::Trash(TrashAction::DeletePermanently(node)) => {
                if let Some(entry) = self.document.trash_entry(*node) {
                    let (title, text) = labels::delete_permanently_question(&entry.node, language);
                    self.confirmation = Some(Confirmation {
                        kind: ConfirmationKind::DeletePermanently(*node),
                        title,
                        text,
                        confirm_label: TrashText::DeletePermanently.get(language),
                    });
                }
            }
            WindowAction::Trash(TrashAction::LeaveTrash) => self.trash.visible = false,
            WindowAction::Trash(TrashAction::EmptyTrash) => {
                let (title, text) = labels::empty_trash_question(&self.document.trash, language);
                self.confirmation = Some(Confirmation {
                    kind: ConfirmationKind::EmptyTrash,
                    title,
                    text,
                    confirm_label: TrashText::DeletePermanently.get(language),
                });
            }
            WindowAction::ConfirmAccepted => self.apply_confirmation(language),
            WindowAction::ConfirmCancelled => self.confirmation = None,
            // The other actions are dispatched by `apply`.
            _ => {}
        }
    }

    /// Does what the open delete question asked about.
    fn apply_confirmation(&mut self, language: Language) {
        let Some(confirmation) = self.confirmation.take() else {
            return;
        };
        match confirmation.kind {
            ConfirmationKind::DeleteGroup(node) => self.move_to_trash(node, language),
            ConfirmationKind::DeletePermanently(node) => {
                let next = self.trash_neighbour(node);
                if let Some(entry) = self.document.delete_from_trash(node) {
                    self.forget_drafts(&[entry]);
                    self.trash.selected = next;
                    self.save_notes();
                }
            }
            ConfirmationKind::EmptyTrash => {
                let removed = self.document.empty_trash();
                if !removed.is_empty() {
                    self.forget_drafts(&removed);
                    self.trash.selected = None;
                    self.save_notes();
                }
            }
        }
    }

    /// The trash selection once `node` leaves the trash: unchanged unless
    /// `node` is selected, then the next entry down the list (older), else
    /// the one above, else nothing.
    fn trash_neighbour(&self, node: NodeId) -> Option<NodeId> {
        if self.trash.selected != Some(node) {
            return self.trash.selected;
        }
        // The list shows the newest entry first.
        let shown: Vec<NodeId> = self
            .document
            .trash
            .iter()
            .rev()
            .map(TrashEntry::id)
            .collect();
        let index = shown.iter().position(|entry| *entry == node)?;
        shown
            .get(index + 1)
            .or_else(|| index.checked_sub(1).and_then(|before| shown.get(before)))
            .copied()
    }

    /// Deletes the drafts of the notes in `entries`, deleted for good
    /// (M151).
    fn forget_drafts(&mut self, entries: &[TrashEntry]) {
        let mut changed = false;
        for entry in entries {
            entry.node.for_each_note_id(&mut |note| {
                changed |= self.drafts.take(note).is_some();
            });
        }
        if changed {
            self.saver.save_drafts(self.drafts.clone());
        }
    }

    /// Removes trash entries older than 30 days at start and then hourly
    /// (M150), with their drafts.
    pub(super) fn purge_trash(&mut self, now: Timestamp) {
        if self.trash.next_purge.is_some_and(|next| now < next) {
            return;
        }
        self.trash.next_purge = Some(now + PURGE_INTERVAL);
        let purged = self.document.purge_trash(now);
        if purged.is_empty() {
            return;
        }
        if self
            .trash
            .selected
            .is_some_and(|selected| self.document.trash_entry(selected).is_none())
        {
            self.trash.selected = None;
        }
        self.forget_drafts(&purged);
        self.save_notes();
    }
}

/// Collects the rows of `nodes` as the list shows them: groups, and the
/// children of expanded groups.
fn visible_rows(nodes: &[TreeNode], rows: &mut Vec<NodeId>) {
    for node in nodes {
        rows.push(node.id());
        if let TreeNode::Group(group) = node
            && group.expanded
        {
            visible_rows(&group.children, rows);
        }
    }
}

/// Collects the id of `node` and of everything inside it.
fn all_ids(node: &TreeNode, ids: &mut Vec<NodeId>) {
    ids.push(node.id());
    if let TreeNode::Group(group) = node {
        for child in &group.children {
            all_ids(child, ids);
        }
    }
}
