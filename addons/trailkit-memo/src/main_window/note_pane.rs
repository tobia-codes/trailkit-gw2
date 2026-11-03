//! The note pane of the memo main window: note view, editor with its
//! formatting toolbar (M51–M53, M94), title with kind (M102) and draft hint (from the drafts slice).

use nexus::imgui::{Key, StyleColor, Ui};
use trailkit_ui::{
    button::icon_button_width,
    button::{
        button_width, draw_button, draw_icon_button, draw_primary_button_with_icon,
        icon_label_button_width,
    },
    font_registry::FontScope,
    fonts::FontRole,
    icons::Icon,
    markdown::{self},
    markdown_view::{MarkdownAction, SearchHighlight, draw_markdown},
    pane::draw_pane,
    separator::draw_separator,
    splitter::draw_vertical_splitter,
    theme::Theme,
    toolbar::{begin_toolbar_group, move_to_toolbar_end},
};

use crate::note_editing::{ActiveFormats, FormatCommand};
use trailkit_memo_drafts::draft_hint::DraftHintAction;
use trailkit_memo_note_view::note_view::{NoteViewAction, NoteViewInput, TitleTrailing};
use trailkit_memo_reset_schedule::note_kind::{draw_kind_dropdown, kind_dropdown_size};
use trailkit_memo_shared::i18n::LocalizedText;
use trailkit_memo_shared::i18n::{Language, Text, chip_context};

use super::{
    MINIMUM_PREVIEW_SPLIT_CHARACTERS, MemoWindow, VIRTUAL_KEY_B, VIRTUAL_KEY_I, VIRTUAL_KEY_S,
    WindowAction, selected_note_in,
};

impl MemoWindow {
    /// Draws the note pane: the note toolbar with the menu button (M48),
    /// then the note in its own scroll region so the toolbar stays in place.
    /// `menu_tooltip` names what the menu button does.
    pub(super) fn draw_note_pane(
        &mut self,
        ui: &Ui,
        language: Language,
        theme: &Theme,
        menu_tooltip: Text,
    ) {
        if draw_icon_button(
            ui,
            theme,
            note_list_icon(menu_tooltip),
            menu_tooltip.get(language),
        ) {
            self.actions.push(WindowAction::ToggleNoteList);
        }
        // One toolbar for the note (M94, M102): the formatting groups while
        // editing and the edit or Cancel/Save buttons at the end; it wraps in
        // narrow panes (`lay-responsive`).
        if let Some(editor) = self.editor.as_mut() {
            let active = editor.active_formats();
            draw_format_toolbar(
                ui,
                language,
                theme,
                ToolbarState {
                    active,
                    preview_visible: self.ui_state.preview_visible,
                },
                &mut self.actions,
            );
        }
        self.draw_note_toolbar_end(ui, language, theme);
        draw_separator(ui, theme);
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

    /// Draws the editor (M51–M53): formatting toolbar, title input and the
    /// markdown body input filling the rest of the pane.
    pub(super) fn draw_editor(&mut self, ui: &Ui, language: Language, theme: &Theme) {
        // While the unsaved changes dialog is shown, typing must not change
        // what the dialog asks about.
        let is_dialog_open = self.leave_prompt.is_some();
        let Some(editor) = self.editor.as_mut() else {
            return;
        };
        // ImGui reverts an input to its content at activation when Escape is
        // pressed; keep the typed text instead (only copied on that frame).
        let escape_pressed = ui.is_key_pressed(Key::Escape);

        let title_backup =
            (escape_pressed && editor.title_was_active).then(|| editor.session.title.clone());
        {
            let _heading = FontScope::push(ui, FontRole::Heading1);
            ui.set_next_item_width(-f32::MIN_POSITIVE);
            ui.input_text("##title", &mut editor.session.title)
                .hint(Text::TitleHint.get(language))
                .read_only(is_dialog_open)
                .build();
        }
        editor.title_was_active = ui.is_item_active();
        if ui.is_item_deactivated()
            && let Some(title) = title_backup
        {
            editor.session.title = title;
        }

        if is_dialog_open {
            editor.body.unfocus();
        }
        // Ctrl+S works like "Save" from title and text (M93).
        if !is_dialog_open
            && (editor.body.is_focused() || editor.title_was_active)
            && ui.io().key_ctrl
            && ui.is_key_index_pressed(VIRTUAL_KEY_S)
        {
            self.actions.push(WindowAction::FinishEditing);
        }
        if editor.body.is_focused() && ui.io().key_ctrl {
            if ui.is_key_index_pressed(VIRTUAL_KEY_B) {
                editor.format(FormatCommand::Bold);
            } else if ui.is_key_index_pressed(VIRTUAL_KEY_I) {
                editor.format(FormatCommand::Italic);
            }
        }
        // Preview (M60, M72): editor and preview always stay side by side,
        // however narrow; the minimum only keeps the splitter usable.
        let available = ui.content_region_avail()[0];
        let gap = theme.pane.gap;
        let usable = (available - gap).max(0.0);
        let preview_minimum = (ui.calc_text_size("n")[0] * MINIMUM_PREVIEW_SPLIT_CHARACTERS
            + 2.0 * theme.pane.padding)
            .min(usable / 2.0);
        let editor_width = (usable * self.ui_state.clamped_preview_split_fraction())
            .clamp(preview_minimum, usable - preview_minimum);
        let width = if self.ui_state.preview_visible {
            editor_width
        } else {
            0.0
        };
        let chips = chip_context(
            language,
            self.copied_chat_link
                .as_ref()
                .map(|copied| copied.code.as_str()),
        );
        let response = editor.body.draw(ui, theme, "##body", [width, 0.0], chips);
        if response.changed {
            editor.sync_body();
        }
        if response.submitted {
            self.actions.push(WindowAction::FinishEditing);
        }
        if self.ui_state.preview_visible {
            ui.same_line_with_spacing(0.0, 0.0);
            let splitter = draw_vertical_splitter(
                ui,
                theme,
                "preview_splitter",
                editor_width,
                (preview_minimum, usable - preview_minimum),
                ui.content_region_avail()[1],
            );
            if let Some(width) = splitter.left_width {
                self.ui_state.preview_split_fraction = width / usable;
            }
            if splitter.released {
                self.saver.save_ui_state(self.ui_state);
            }
            ui.same_line_with_spacing(0.0, 0.0);
            let preview = editor
                .preview
                .get_or_insert_with(|| markdown::parse(&editor.session.body));
            let actions = &mut self.actions;
            let search_query = self.search.current.as_ref().map(|current| &current.query);
            draw_pane(ui, theme, "preview", [0.0, 0.0], || {
                // The preview is read-only: links open and chips copy, but
                // checkboxes do not toggle. It marks search hits (M122).
                let mut highlight = search_query.map(SearchHighlight::new);
                match draw_markdown(ui, preview, theme, chips, highlight.as_mut()) {
                    Some(
                        action @ (MarkdownAction::OpenLink(_) | MarkdownAction::ChatLinkCopied(_)),
                    ) => {
                        actions.push(WindowAction::Markdown(action));
                    }
                    Some(MarkdownAction::ToggleTask(_)) | None => {}
                }
            });
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
        let mut chosen_kind = None;
        let mut draw_kind = |ui: &Ui| {
            if let Some(note) = note {
                chosen_kind = draw_kind_dropdown(ui, theme, language, note.kind);
            }
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
            title_trailing: Some(TitleTrailing {
                size: kind_dropdown_size(ui, language),
                draw: &mut draw_kind,
            }),
        };
        let output =
            self.note_view
                .draw(ui, language, theme, input, &mut self.ui_state.note_scroll);
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

/// What the formatting toolbar shows as active.
#[derive(Debug, Clone, Copy)]
pub(super) struct ToolbarState {
    /// Formatting at the cursor (M69).
    active: ActiveFormats,
    /// Whether the preview is shown (M60).
    preview_visible: bool,
}

/// The sidebar icon of the note list toggle (M101): the icon itself shows
/// what a click does, so the button needs no accent "on" state (replaces
/// the gold menu button of M71).
pub(super) fn note_list_icon(tooltip: Text) -> Icon {
    if tooltip == Text::HideNoteList {
        Icon::PANEL_LEFT_CLOSE
    } else {
        Icon::PANEL_LEFT_OPEN
    }
}

/// One formatting button: icon, tooltip and command (`None` = disabled).
pub(super) type FormatButton = (Icon, Text, Option<FormatCommand>);

/// The formatting buttons in groups, like Word's ribbon (M53, M100, M102):
/// headings, emphasis, lists, inserts. The event insert is disabled until
/// trailkit-core exists.
pub(super) const FORMAT_GROUPS: [&[FormatButton]; 4] = [
    &[
        (
            Icon::HEADING_1,
            Text::FormatHeading1,
            Some(FormatCommand::Heading1),
        ),
        (
            Icon::HEADING_2,
            Text::FormatHeading2,
            Some(FormatCommand::Heading2),
        ),
    ],
    &[
        (Icon::BOLD, Text::FormatBold, Some(FormatCommand::Bold)),
        (
            Icon::ITALIC,
            Text::FormatItalic,
            Some(FormatCommand::Italic),
        ),
    ],
    &[
        (
            Icon::LIST,
            Text::FormatBulletList,
            Some(FormatCommand::BulletList),
        ),
        (
            Icon::LIST_TODO,
            Text::FormatCheckbox,
            Some(FormatCommand::Checkbox),
        ),
    ],
    &[
        (Icon::LINK, Text::FormatLink, Some(FormatCommand::Link)),
        (Icon::CALENDAR_CLOCK, Text::InsertEventRequiresCore, None),
    ],
];

/// Draws the formatting buttons of the editor in groups with separators
/// (M53, M102), then the preview toggle. Buttons of the formatting at the
/// cursor are drawn in the accent color (M69); the disabled event button in
/// `text.disabled`.
pub(super) fn draw_format_toolbar(
    ui: &Ui,
    language: Language,
    theme: &Theme,
    state: ToolbarState,
    actions: &mut Vec<WindowAction>,
) {
    let spacing = ui.clone_style().item_spacing[0];
    let mut id = 0;
    for group in FORMAT_GROUPS {
        let width = group
            .iter()
            .map(|(icon, _, _)| icon_button_width(ui, *icon))
            .sum::<f32>()
            + spacing * (group.len().saturating_sub(1)) as f32;
        begin_toolbar_group(ui, theme, width);
        for (index, (icon, tooltip, command)) in group.iter().enumerate() {
            if index > 0 {
                ui.same_line();
            }
            id += 1;
            let _id = ui.push_id(id);
            // ImGui 1.80 has no disabled items: a disabled button is drawn in
            // `text.disabled` and its clicks are ignored.
            let icon_color = match command {
                None => Some(theme.text.disabled),
                Some(command) if state.active.is_active(*command) => Some(theme.accent.text),
                Some(_) => None,
            };
            let _icon_color =
                icon_color.map(|color| ui.push_style_color(StyleColor::Text, color.to_array()));
            if draw_icon_button(ui, theme, *icon, tooltip.get(language))
                && let Some(command) = command
            {
                actions.push(WindowAction::Format(*command));
            }
        }
    }

    // The preview toggle is its own group; its icon is drawn in the accent
    // color while the preview is visible.
    begin_toolbar_group(ui, theme, icon_button_width(ui, Icon::EYE));
    let (tooltip, _selected) = if state.preview_visible {
        (
            Text::HidePreview,
            Some(ui.push_style_color(StyleColor::Text, theme.accent.text.to_array())),
        )
    } else {
        (Text::ShowPreview, None)
    };
    if draw_icon_button(ui, theme, Icon::EYE, tooltip.get(language)) {
        actions.push(WindowAction::TogglePreview);
    }
}
