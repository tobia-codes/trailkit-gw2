//! The note pane of the memo main window: the note toolbar with the compact
//! view toggle (M139) and its end (Edit / Cancel / Save, M51, M94), the
//! wiring of the editor slice, the note-view slice with the kind dropdown
//! (M102) and the draft hint.

use nexus::imgui::Ui;
use trailkit_ui::controls::hover::is_item_hovered;
use trailkit_ui::controls::tooltip::draw_tooltip;
use trailkit_ui::{
    controls::button::{
        Button, ButtonKind, ButtonState, button_width, draw_button, draw_icon_button,
        draw_icon_toggle, draw_primary_button_with_icon, icon_button_width,
        icon_label_button_width,
    },
    controls::menu::place_next_popup,
    controls::text_field::move_below_last_item,
    layout::stack::{Stack, StackLayout},
    layout::toolbar::move_to_toolbar_end,
    text::icons::Icon,
    theme::Theme,
};

use trailkit_memo_drafts::draft_hint::DraftHintAction;
use trailkit_memo_editor::editor::{EditorAction, EditorInput, ToolbarAction};
use trailkit_memo_note_list::note_list::{EntryMenu, LeadingItem, draw_note_menu};
use trailkit_memo_note_view::note_view::{FindInNote, NoteViewAction, NoteViewInput};
use trailkit_memo_reset_schedule::note_kind::{draw_kind_dropdown, kind_dropdown_size};
use trailkit_memo_shared::{
    i18n::{Language, LocalizedText, Text, chip_context},
    note_tree::Note,
    save_text::SaveText,
};
use trailkit_ui::layout::page::TitleTrailing;

use crate::main_window::{
    MemoWindow, WindowAction, panes::note_list::draw_reset_type_submenu, shown_note_in,
};

impl MemoWindow {
    /// Draws the note pane: the note toolbar, then the note in its own
    /// scroll region so the toolbar stays in place. The compact note view
    /// (M139) has no toolbar; its toggle sits behind the note title.
    pub(in crate::main_window) fn draw_note_pane(
        &mut self,
        ui: &Ui,
        language: Language,
        theme: &Theme,
    ) {
        if self.trash.visible {
            self.draw_trash_preview(ui, language, theme);
            return;
        }
        if self.ui_state.compact_note_view && self.editor.is_none() {
            Stack::vertical(theme).build(ui, |stack| {
                self.add_find_bar(stack, ui, language, theme);
                stack.add(|| self.draw_note_body(ui, language, theme));
            });
            return;
        }
        Stack::vertical(theme).build(ui, |stack| {
            stack.add(|| self.draw_note_toolbar(ui, language, theme));
            // Without a note there is no toolbar above that a line could set off.
            if self.editor.is_some() || self.selected_note().is_some() {
                stack.add_separator(theme);
            }
            // Find in the note sits fixed below the toolbar (M198, variant A).
            self.add_find_bar(stack, ui, language, theme);
            stack.add(|| {
                if self.editor.is_some() {
                    self.draw_editor(ui, language, theme);
                } else {
                    self.draw_note_body(ui, language, theme);
                }
            });
        });
    }

    /// Adds the find bar to `stack` while it is shown, with the divider below
    /// it (M198, R148).
    fn add_find_bar(
        &mut self,
        stack: &mut StackLayout<'_>,
        ui: &Ui,
        language: Language,
        theme: &Theme,
    ) {
        if !self.find_bar_shown() {
            return;
        }
        stack.add(|| self.draw_find_bar(ui, language, theme));
        stack.add_separator(theme);
    }

    /// Draws the note toolbar row: the compact view toggle while viewing, the
    /// formatting groups while editing, and its right end.
    fn draw_note_toolbar(&mut self, ui: &Ui, language: Language, theme: &Theme) {
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
    }

    /// Draws the right end of the note toolbar (M51): the edit button while
    /// viewing, "Cancel" and the primary "Save" while editing.
    pub(in crate::main_window) fn draw_note_toolbar_end(
        &mut self,
        ui: &Ui,
        language: Language,
        theme: &Theme,
    ) {
        let spacing = theme.spacing.small;
        if self.editor.is_some() {
            let cancel = Text::Cancel.get(language);
            // While "Save" waits for the disk it shows "Saving…" (Q158).
            let saving = self.editor_commit_running();
            let save = if saving {
                SaveText::Saving.get(language)
            } else {
                Text::Save.get(language)
            };
            let width = button_width(ui, theme, cancel)
                + spacing
                + icon_label_button_width(ui, theme, save);
            move_to_toolbar_end(ui, theme, width);
            if draw_button(ui, theme, cancel) {
                self.actions.push(WindowAction::CancelEditing);
            }
            ui.same_line_with_spacing(0.0, spacing);
            let state = if saving {
                ButtonState::Busy
            } else {
                ButtonState::Ready
            };
            let save_button = Button::new(save)
                .kind(ButtonKind::Primary)
                .icon(Icon::SAVE)
                .state(state);
            if save_button.draw(ui, theme) {
                self.actions.push(WindowAction::FinishEditing);
            }
            if !saving && is_item_hovered(ui) {
                draw_tooltip(ui, theme, Text::SaveTooltip.get(language));
            }
        } else if self.selected_note().is_some() {
            // Editing is the main action of the reading view: the one accent
            // button of the view (R71); the note's other actions wait in the
            // menu beside it (M160).
            let edit = Text::EditNote.get(language);
            move_to_toolbar_end(
                ui,
                theme,
                icon_button_width(ui, Icon::ELLIPSIS)
                    + spacing
                    + icon_label_button_width(ui, theme, edit),
            );
            self.draw_note_menu_button(ui, language, theme);
            ui.same_line_with_spacing(0.0, spacing);
            if draw_primary_button_with_icon(ui, theme, Icon::PENCIL, edit) {
                self.actions.push(WindowAction::StartEditing);
            }
        }
    }

    /// Draws the `⋯` button of the note toolbar and its menu, whose right edge
    /// lines up with the button (M160).
    fn draw_note_menu_button(&mut self, ui: &Ui, language: Language, theme: &Theme) {
        const POPUP_ID: &str = "note_menu";
        if draw_icon_button(ui, theme, Icon::ELLIPSIS, Text::MoreActions.get(language)) {
            ui.open_popup(POPUP_ID);
        }
        let Some(note) = shown_note_in(&self.document, self.ui_state.shown_note) else {
            return;
        };
        let (right, bottom) = (ui.item_rect_max()[0], ui.item_rect_max()[1]);
        place_next_popup([right, bottom], 1.0);
        let window_actions = &mut self.actions;
        let mut note_items = |ui: &Ui, note: &Note| {
            draw_reset_type_submenu(ui, theme, language, note, window_actions);
        };
        let mut list_actions = Vec::new();
        // Find in the note comes first, with its shortcut (M198).
        let find = LeadingItem {
            icon: Icon::SEARCH,
            label: Text::FindInNote.get(language),
            shortcut: Text::FindInNoteShortcut.get(language),
        };
        let choice = draw_note_menu(
            ui,
            POPUP_ID,
            note,
            find,
            EntryMenu {
                document: &self.document,
                edited: None,
                language,
                theme,
                note_items: &mut note_items,
                actions: &mut list_actions,
            },
        );
        self.actions
            .extend(list_actions.into_iter().map(WindowAction::from));
        if choice.rename {
            self.actions.push(WindowAction::StartRenaming);
        }
        if choice.leading {
            self.open_find();
        }
    }

    /// Draws the editor slice below the toolbar and applies what it reports.
    pub(in crate::main_window) fn draw_editor(
        &mut self,
        ui: &Ui,
        language: Language,
        theme: &Theme,
    ) {
        let Some(editor) = self.editor.as_mut() else {
            return;
        };
        // Find in the note marks its hits in the body; the current one is
        // selected once it is known (M198).
        let find = if self.find.visible && !self.trash.visible {
            self.find.query.as_ref()
        } else {
            None
        };
        let hit_count = editor.update_find(find, self.find.current);
        let edited = editor.session().note;
        if find.is_some() {
            if std::mem::take(&mut self.find.reveal_pending) && hit_count > 0 {
                editor.select_find_hit(self.find.current.min(hit_count - 1));
            }
            self.record_find_hits(Some(edited), hit_count);
        }
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
            chips: chip_context(language),
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
            self.saver.save_ui_state(&self.ui_state);
        }
    }

    /// Draws the selected note through the note-view slice: the draft hint
    /// (drafts slice) above it and the kind dropdown (reset-schedule slice)
    /// behind its title.
    pub(in crate::main_window) fn draw_note_body(
        &mut self,
        ui: &Ui,
        language: Language,
        theme: &Theme,
    ) {
        // Borrow only the document, so the view's state can change below.
        let note = shown_note_in(&self.document, self.ui_state.shown_note);
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
            // The note keeps `space.md` from the hint above it (R148).
            move_below_last_item(ui, theme.spacing.medium);
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
                chosen_kind = draw_kind_dropdown(ui, theme, language, note.reset_schedule);
            }
        };
        let trailing_size = if compact {
            [icon_button_width(ui, Icon::MINIMIZE_2), ui.frame_height()]
        } else {
            kind_dropdown_size(ui, theme, language)
        };
        // Find in the note replaces the search marks while it has a phrase;
        // the view scrolls to the current hit once it is known (M198).
        let finding = self.find_bar_shown() && self.find.query.is_some();
        if finding && std::mem::take(&mut self.find.reveal_pending) {
            self.note_view.reveal_current_hit();
        }
        let current = self.find.current;
        let find = self
            .find
            .query
            .as_ref()
            .filter(|_| finding)
            .map(|query| FindInNote { query, current });
        let input = NoteViewInput {
            note,
            search: self.search.current.as_ref().map(|current| &current.query),
            find,
            chips: chip_context(language),
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
            self.saver.save_ui_state(&self.ui_state);
        }
        if finding {
            let shown = note.map(|note| note.id);
            self.record_find_hits(shown, output.find_hit_count.unwrap_or(0));
        }
    }
}
