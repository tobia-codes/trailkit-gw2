//! The note pane of the memo main window: the note toolbar with the compact
//! view toggle (M139) and its end (Edit / Cancel / Save, M51, M94), the
//! wiring of the editor slice, the note-view slice with the kind dropdown
//! (M102) and the draft hint.

use nexus::imgui::Ui;
use trailkit_ui::{
    button::{
        button_width, draw_button, draw_icon_button, draw_icon_toggle,
        draw_primary_button_with_icon, icon_button_width, icon_label_button_width,
    },
    icons::Icon,
    separator::draw_separator,
    theme::Theme,
    toolbar::move_to_toolbar_end,
};

use trailkit_memo_drafts::draft_hint::DraftHintAction;
use trailkit_memo_editor::editor::{EditorAction, EditorInput, ToolbarAction};
use trailkit_memo_note_view::note_view::{NoteViewAction, NoteViewInput, TitleTrailing};
use trailkit_memo_reset_schedule::note_kind::{draw_kind_dropdown, kind_dropdown_size};
use trailkit_memo_shared::i18n::{Language, LocalizedText, Text, chip_context};

use super::{MemoWindow, WindowAction, selected_note_in};

impl MemoWindow {
    /// Draws the note pane: the note toolbar, then the note in its own
    /// scroll region so the toolbar stays in place. The compact note view
    /// (M139) has no toolbar; its toggle sits behind the note title.
    pub(super) fn draw_note_pane(&mut self, ui: &Ui, language: Language, theme: &Theme) {
        if self.trash.visible {
            self.draw_trash_preview(ui, language, theme);
            return;
        }
        if self.ui_state.compact_note_view && self.editor.is_none() {
            self.draw_note_body(ui, language, theme);
            return;
        }
        if self.editor.is_none()
            && self.selected_note().is_some()
            && draw_icon_button(
                ui,
                theme,
                Icon::MAXIMIZE_2,
                Text::CompactNoteView.get(language),
            )
        {
            self.actions.push(WindowAction::ToggleCompactNoteView);
        }
        // One toolbar for the note (M94, M102): the formatting groups while
        // editing and the edit or Cancel/Save buttons at the end; it wraps in
        // narrow panes (`lay-responsive`).
        if let Some(editor) = self.editor.as_mut() {
            match editor.draw_toolbar(ui, language, theme, self.ui_state.preview_visible) {
                Some(ToolbarAction::Format(command)) => {
                    self.actions.push(WindowAction::Format(command));
                }
                Some(ToolbarAction::TogglePreview) => {
                    self.actions.push(WindowAction::TogglePreview);
                }
                None => {}
            }
        }
        self.draw_note_toolbar_end(ui, language, theme);
        // Without a note there is no toolbar above that a line could set off.
        if self.editor.is_some() || self.selected_note().is_some() {
            draw_separator(ui, theme);
        }
        if self.editor.is_some() {
            self.draw_editor(ui, language, theme);
        } else {
            self.draw_note_body(ui, language, theme);
        }
    }

    /// Draws the right end of the note toolbar (M51): the edit button while
    /// viewing, "Cancel" and the primary "Save" while editing.
    pub(super) fn draw_note_toolbar_end(&mut self, ui: &Ui, language: Language, theme: &Theme) {
        let spacing = ui.clone_style().item_spacing[0];
        if self.editor.is_some() {
            let cancel = Text::Cancel.get(language);
            let save = Text::Save.get(language);
            let width = button_width(ui, theme, cancel)
                + spacing
                + icon_label_button_width(ui, theme, save);
            move_to_toolbar_end(ui, width);
            if draw_button(ui, theme, cancel) {
                self.actions.push(WindowAction::CancelEditing);
            }
            ui.same_line();
            if draw_primary_button_with_icon(ui, theme, Icon::SAVE, save) {
                self.actions.push(WindowAction::FinishEditing);
            }
            if ui.is_item_hovered() {
                ui.tooltip_text(Text::SaveTooltip.get(language));
            }
        } else if self.selected_note().is_some() {
            // Editing is the main action of the reading view: the one brass
            // button of the view (R71).
            let edit = Text::EditNote.get(language);
            move_to_toolbar_end(ui, icon_label_button_width(ui, theme, edit));
            if draw_primary_button_with_icon(ui, theme, Icon::PENCIL, edit) {
                self.actions.push(WindowAction::StartEditing);
            }
        }
    }

    /// Draws the editor slice below the toolbar and applies what it reports.
    pub(super) fn draw_editor(&mut self, ui: &Ui, language: Language, theme: &Theme) {
        let Some(editor) = self.editor.as_mut() else {
            return;
        };
        let input = EditorInput {
            preview_visible: self.ui_state.preview_visible,
            preview_split_fraction: self.ui_state.clamped_preview_split_fraction(),
            // While the unsaved changes dialog is shown, typing must not
            // change what the dialog asks about.
            is_dialog_open: self.leave_prompt.is_some(),
            search: self.search.current.as_ref().map(|current| &current.query),
            chips: chip_context(
                language,
                self.copied_chat_link
                    .as_ref()
                    .map(|copied| copied.code.as_str()),
            ),
        };
        let output = editor.draw(ui, language, theme, input);
        for action in output.actions {
            self.actions.push(match action {
                EditorAction::Save => WindowAction::FinishEditing,
                EditorAction::Markdown(action) => WindowAction::Markdown(action),
            });
        }
        if let Some(fraction) = output.preview_split_fraction {
            self.ui_state.preview_split_fraction = fraction;
        }
        if output.split_released {
            self.saver.save_ui_state(self.ui_state);
        }
    }

    /// Draws the selected note through the note-view slice: the draft hint
    /// (drafts slice) above it and the kind dropdown (reset-schedule slice)
    /// behind its title.
    pub(super) fn draw_note_body(&mut self, ui: &Ui, language: Language, theme: &Theme) {
        // Borrow only the document, so the view's state can change below.
        let note = selected_note_in(&self.document, self.ui_state.selected);
        if let Some(note) = note
            && let Some(draft) = self.drafts.get(note.id)
        {
            match self
                .draft_hint
                .draw(ui, language, theme, draft, note.updated_at)
            {
                Some(DraftHintAction::Restore) => {
                    self.actions.push(WindowAction::RestoreDraft(note.id));
                }
                Some(DraftHintAction::Discard) => {
                    self.actions.push(WindowAction::DiscardDraft(note.id));
                }
                None => {}
            }
        }
        let compact = self.ui_state.compact_note_view;
        let mut chosen_kind = None;
        let mut leave_compact = false;
        let mut draw_trailing = |ui: &Ui| {
            if compact {
                // The only control of the compact view (M139).
                leave_compact = draw_icon_toggle(
                    ui,
                    theme,
                    Icon::MINIMIZE_2,
                    Text::RegularNoteView.get(language),
                    true,
                );
            } else if let Some(note) = note {
                chosen_kind = draw_kind_dropdown(ui, theme, language, note.kind);
            }
        };
        let trailing_size = if compact {
            [icon_button_width(ui, Icon::MINIMIZE_2), ui.frame_height()]
        } else {
            kind_dropdown_size(ui, language)
        };
        let input = NoteViewInput {
            note,
            search: self.search.current.as_ref().map(|current| &current.query),
            chips: chip_context(
                language,
                self.copied_chat_link
                    .as_ref()
                    .map(|copied| copied.code.as_str()),
            ),
            read_only: None,
            title_trailing: Some(TitleTrailing {
                size: trailing_size,
                at_row_end: compact,
                draw: &mut draw_trailing,
            }),
        };
        let output =
            self.note_view
                .draw(ui, language, theme, input, &mut self.ui_state.note_scroll);
        if leave_compact {
            self.actions.push(WindowAction::ToggleCompactNoteView);
        }
        if let (Some(note), Some(period)) = (note, chosen_kind) {
            self.actions
                .push(WindowAction::SetResetPeriod(note.id, period));
        }
        for action in output.actions {
            self.actions.push(match action {
                NoteViewAction::StartEditing => WindowAction::StartEditing,
                NoteViewAction::Markdown(action) => WindowAction::Markdown(action),
            });
        }
        if output.scroll_changed {
            self.saver.save_ui_state(self.ui_state);
        }
    }
}
