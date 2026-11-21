//! Drawing the trash view and the undo toast in the memo main window (memo
//! M136, M145, M148): the trash column in place of the note list, the
//! read-only preview in the note pane and the toast at the bottom.

use jiff::Timestamp;
use nexus::imgui::{ChildWindow, Ui};
use trailkit_ui::{
    controls::button::ButtonState,
    feedback::toast::{ToastAction, draw_toast},
    layout::stack::Stack,
    markdown::markdown_view::{MarkdownAction, open_link},
    text::{TextStyle, draw_text_wrapped},
    theme::Theme,
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

use crate::main_window::{MemoWindow, WindowAction, WindowToastKind};

impl MemoWindow {
    /// Draws the trash column in place of the note list (M136): header with
    /// "Empty" and the scrolling entries.
    pub(in crate::main_window) fn draw_trash_column(
        &mut self,
        ui: &Ui,
        language: Language,
        theme: &Theme,
    ) {
        let has_entries = !self.document.trash.is_empty();
        let now = Timestamp::now();
        let mut clicked = None;
        Stack::vertical(theme).build(ui, |stack| {
            if let Some(action) = stack.add(|| draw_trash_header(ui, language, theme, has_entries))
            {
                self.actions.push(WindowAction::Trash(action));
            }
            stack.add_separator(theme);
            stack.add(|| {
                ChildWindow::new("trash_list")
                    .size([0.0, 0.0])
                    .draw_background(false)
                    .build(ui, || {
                        clicked = draw_trash_entries(
                            ui,
                            language,
                            theme,
                            &self.document,
                            self.trash.selected,
                            &self.trash.open_groups,
                            now,
                        );
                    });
            });
        });
        if let Some(action) = clicked {
            self.actions.push(WindowAction::Trash(action));
        }
    }

    /// Draws the selected trash entry read-only in the note pane (M145):
    /// toolbar with "Delete permanently" and "Restore", title, info
    /// line and the content.
    pub(in crate::main_window) fn draw_trash_preview(
        &mut self,
        ui: &Ui,
        language: Language,
        theme: &Theme,
    ) {
        let now = Timestamp::now();
        let selected = self.trash.selected;
        // The selected entry may sit inside a trashed group (M218): it shows
        // the deadline of the entry holding it and names its group.
        let found = selected.and_then(|node| {
            let shown = self.document.find_in_trash(node)?;
            let holding = self.document.trash_entry_holding(node)?;
            let group = self
                .document
                .trashed_parent(node)
                .map(|group| group.title.as_str());
            Some((
                node,
                shown,
                labels::info_line(holding, group, now, language),
            ))
        });
        let Some((entry_id, shown, info)) = found else {
            draw_text_wrapped(
                ui,
                theme,
                TextStyle::Secondary,
                TrashText::SelectTrashEntry.get(language),
            );
            return;
        };
        let mut toolbar_action = None;
        let mut view_actions = Vec::new();
        Stack::vertical(theme).build(ui, |stack| {
            toolbar_action = stack.add(|| draw_preview_toolbar(ui, language, theme, entry_id));
            stack.add_separator(theme);
            let note = match shown {
                TreeNode::Note(note) => note,
                TreeNode::Group(group) => {
                    stack.add(|| draw_group_preview(ui, language, theme, group, &info));
                    return;
                }
            };
            let input = NoteViewInput {
                note: Some(note),
                search: None,
                find: None,
                chips: chip_context(language),
                title_trailing: None,
                read_only: Some(ReadOnly { subtitle: &info }),
            };
            let output = stack.add(|| {
                self.note_view
                    .draw(ui, language, theme, input, &mut self.trash.scroll)
            });
            view_actions = output.actions;
        });
        if let Some(action) = toolbar_action {
            self.actions.push(WindowAction::Trash(action));
        }
        // A trashed note cannot change: links open, and copying (a chip, the
        // menu's Copy) is confirmed like anywhere else (M217).
        for action in view_actions {
            match action {
                NoteViewAction::Markdown(MarkdownAction::OpenLink(url)) => open_link(&url),
                NoteViewAction::Markdown(MarkdownAction::Copied(text)) => {
                    self.actions
                        .push(WindowAction::Markdown(MarkdownAction::Copied(text)));
                }
                NoteViewAction::Markdown(MarkdownAction::ToggleTask(_))
                | NoteViewAction::StartEditing => {}
            }
        }
    }

    /// Draws the toast at the bottom of the window; after a delete it offers
    /// "Undo" and "Open trash" (M142, M148, M158).
    pub(in crate::main_window) fn draw_toast_message(
        &mut self,
        ui: &Ui,
        language: Language,
        theme: &Theme,
    ) {
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
