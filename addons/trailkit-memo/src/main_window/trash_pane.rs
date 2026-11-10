//! Drawing the trash view and the undo toast in the memo main window (memo
//! M136, M145, M148): the trash column in place of the note list, the
//! read-only preview in the note pane and the toast at the bottom.

use jiff::Timestamp;
use nexus::imgui::{ChildWindow, StyleColor, Ui};
use trailkit_ui::{
    button::ButtonState,
    markdown_view::{MarkdownAction, open_web_link},
    separator::draw_separator,
    theme::Theme,
    toast::{ToastAction, draw_toast},
};

use trailkit_memo_note_view::note_view::{NoteViewAction, NoteViewInput, ReadOnly};
use trailkit_memo_shared::{
    i18n::{Language, LocalizedText, chip_context},
    note_tree::TreeNode,
};
use trailkit_memo_trash::{
    i18n::TrashText,
    labels,
    trash_view::{draw_group_preview, draw_preview_toolbar, draw_trash_entries, draw_trash_header},
};

use super::{MemoWindow, WindowAction, WindowToastKind};

impl MemoWindow {
    /// Draws the trash column in place of the note list (M136): header with
    /// "Empty" and the scrolling entries.
    pub(super) fn draw_trash_column(&mut self, ui: &Ui, language: Language, theme: &Theme) {
        let has_entries = !self.document.trash.is_empty();
        if let Some(action) = draw_trash_header(ui, language, theme, has_entries) {
            self.actions.push(WindowAction::Trash(action));
        }
        let now = Timestamp::now();
        let mut clicked = None;
        ChildWindow::new("trash_list")
            .size([0.0, 0.0])
            .draw_background(false)
            .build(ui, || {
                clicked = draw_trash_entries(
                    ui,
                    language,
                    theme,
                    &self.document.trash,
                    self.trash.selected,
                    now,
                );
            });
        if let Some(action) = clicked {
            self.actions.push(WindowAction::Trash(action));
        }
    }

    /// Draws the selected trash entry read-only in the note pane (M145):
    /// toolbar with "Delete permanently" and "Restore", muted title, info
    /// line and the content.
    pub(super) fn draw_trash_preview(&mut self, ui: &Ui, language: Language, theme: &Theme) {
        let now = Timestamp::now();
        let Some(entry) = self
            .trash
            .selected
            .and_then(|node| self.document.trash_entry(node))
        else {
            let _muted = ui.push_style_color(StyleColor::Text, theme.text.secondary.to_array());
            ui.text_wrapped(TrashText::SelectTrashEntry.get(language));
            return;
        };
        if let Some(action) = draw_preview_toolbar(ui, language, theme, entry.id()) {
            self.actions.push(WindowAction::Trash(action));
        }
        draw_separator(ui, theme);
        let TreeNode::Note(note) = &entry.node else {
            draw_group_preview(ui, language, theme, entry, now);
            return;
        };
        let info = labels::info_line(entry, now, language);
        let input = NoteViewInput {
            note: Some(note),
            search: None,
            chips: chip_context(language),
            title_trailing: None,
            read_only: Some(ReadOnly { subtitle: &info }),
        };
        let output = self
            .note_view
            .draw(ui, language, theme, input, &mut self.trash.scroll);
        // A trashed note cannot change: only links open.
        for action in output.actions {
            if let NoteViewAction::Markdown(MarkdownAction::OpenLink(url)) = action {
                open_web_link(&url);
            }
        }
    }

    /// Draws the toast at the bottom of the window; after a delete it offers
    /// "Undo" and "Open trash" (M142, M148, M158).
    pub(super) fn draw_toast_message(&mut self, ui: &Ui, language: Language, theme: &Theme) {
        let Some(toast) = &mut self.toast else {
            return;
        };
        let trash_actions = [
            ToastAction {
                label: TrashText::Undo.get(language),
                tooltip: Some(TrashText::UndoTooltip.get(language)),
                state: ButtonState::Ready,
            },
            ToastAction {
                label: TrashText::OpenTrash.get(language),
                tooltip: None,
                state: ButtonState::Ready,
            },
        ];
        let actions: &[ToastAction<'_>] = match toast.kind {
            WindowToastKind::Message => &[],
            WindowToastKind::MovedToTrash => &trash_actions,
        };
        let outcome = draw_toast(ui, theme, &mut toast.toast, actions);
        match outcome.clicked_action {
            Some(0) => self.actions.push(WindowAction::Undo),
            Some(_) => self.actions.push(WindowAction::OpenTrash),
            None if outcome.expired => self.actions.push(WindowAction::DismissToast),
            None => {}
        }
    }
}
