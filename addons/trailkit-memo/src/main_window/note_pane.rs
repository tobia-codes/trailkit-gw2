//! The note pane of the memo main window: note view, editor with its
//! formatting toolbar (M51–M53, M94), title with kind (M102) and draft hint (from the drafts slice).

use nexus::imgui::{ChildWindow, Key, MouseButton, StyleColor, Ui};
use trailkit_ui::{
    button::{
        button_width, draw_button, draw_icon_button, draw_primary_button_with_icon,
        icon_label_button_width,
    },
    chat_link_chip::ChipContext,
    font_registry::FontScope,
    fonts::FontRole,
    glyph_metrics::capital_middle,
    icons::Icon,
    markdown::{self},
    markdown_view::{MarkdownAction, SearchHighlight, draw_markdown},
    pane::draw_pane,
    separator::draw_separator,
    splitter::draw_vertical_splitter,
    theme::Theme,
};

use crate::note_editing::{ActiveFormats, FormatCommand};
use trailkit_memo_drafts::draft_hint::DraftHintAction;
use trailkit_memo_reset_schedule::note_kind::{draw_kind_dropdown, kind_dropdown_size};
use trailkit_memo_shared::i18n::LocalizedText;
use trailkit_memo_shared::{
    i18n::{Language, Text, chip_text},
    note_tree::{NoteId, NoteKind},
};

use super::layout::move_to_toolbar_end;
use super::{
    MINIMUM_PREVIEW_SPLIT_CHARACTERS, MemoWindow, ParsedNote, VIRTUAL_KEY_B, VIRTUAL_KEY_I,
    VIRTUAL_KEY_S, WindowAction, selected_note_in,
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
        let chip_texts = |text| chip_text(text, language);
        let chips = ChipContext {
            text: &chip_texts,
            copied: self
                .copied_chat_link
                .as_ref()
                .map(|copied| copied.code.as_str()),
        };
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

    /// Draws the selected note: its title and rendered content.
    pub(super) fn draw_note_body(&mut self, ui: &Ui, language: Language, theme: &Theme) {
        // Borrow only the document, so the markdown cache can be updated below.
        let Some(note) = selected_note_in(&self.document, self.ui_state.selected) else {
            ui.text_colored(
                theme.text.secondary.to_array(),
                Text::SelectNote.get(language),
            );
            return;
        };
        let note_id = note.id;
        let is_stale = self
            .parsed
            .as_ref()
            .is_none_or(|parsed| parsed.note != note_id || parsed.source != note.body);
        if is_stale {
            self.parsed = Some(ParsedNote {
                note: note_id,
                source: note.body.clone(),
                document: markdown::parse(&note.body),
            });
        }

        if let Some(draft) = self.drafts.get(note_id) {
            match self
                .draft_hint
                .draw(ui, language, theme, draft, note.updated_at)
            {
                Some(DraftHintAction::Restore) => {
                    self.actions.push(WindowAction::RestoreDraft(note_id));
                }
                Some(DraftHintAction::Discard) => {
                    self.actions.push(WindowAction::DiscardDraft(note_id));
                }
                None => {}
            }
        }

        draw_title_with_kind(
            ui,
            language,
            theme,
            &note.title,
            (note.id, note.kind),
            &mut self.actions,
        );
        draw_separator(ui, theme);
        if ui.is_window_hovered() && ui.is_mouse_double_clicked(MouseButton::Left) {
            self.actions.push(WindowAction::StartEditing);
        }
        // Title and separator stay at the top; only the content scrolls, so
        // its scrollbar starts below the separator (M81).
        ChildWindow::new("note_body")
            .size([0.0, 0.0])
            .draw_background(false)
            .build(ui, || self.draw_note_content(ui, language, theme));
    }

    /// Draws the rendered content of the selected note in the scrolling
    /// region below its title.
    pub(super) fn draw_note_content(&mut self, ui: &Ui, language: Language, theme: &Theme) {
        if ui.is_window_hovered() && ui.is_mouse_double_clicked(MouseButton::Left) {
            self.actions.push(WindowAction::StartEditing);
        }
        if self.restore_scroll {
            ui.set_scroll_y(self.ui_state.note_scroll);
            self.restore_scroll = false;
        }
        match &self.parsed {
            Some(parsed) if !parsed.document.blocks.is_empty() => {
                let chip_texts = |text| chip_text(text, language);
                let chips = ChipContext {
                    text: &chip_texts,
                    copied: self
                        .copied_chat_link
                        .as_ref()
                        .map(|copied| copied.code.as_str()),
                };
                // Search hits are marked; a new search or a newly opened
                // note jumps to the first one (M122).
                let mut highlight = self
                    .search
                    .current
                    .as_ref()
                    .map(|current| SearchHighlight::new(&current.query));
                if let Some(action) =
                    draw_markdown(ui, &parsed.document, theme, chips, highlight.as_mut())
                {
                    self.actions.push(WindowAction::Markdown(action));
                }
                if std::mem::take(&mut self.scroll_to_first_hit)
                    && let Some(first_hit) =
                        highlight.as_ref().and_then(SearchHighlight::first_hit_y)
                {
                    ui.set_scroll_y((first_hit - ui.text_line_height()).max(0.0));
                }
            }
            _ => ui.text_colored(
                theme.text.secondary.to_array(),
                Text::EmptyNote.get(language),
            ),
        }
        let scroll = ui.scroll_y();
        if (scroll - self.ui_state.note_scroll).abs() >= 1.0 {
            self.ui_state.note_scroll = scroll;
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

/// The width of an icon button showing `icon`.
pub(super) fn icon_button_width(ui: &Ui, icon: Icon) -> f32 {
    let _icons = FontScope::push(ui, FontRole::Icons);
    let mut buffer = [0; 4];
    ui.calc_text_size(icon.as_str(&mut buffer))[0] + 2.0 * ui.clone_style().frame_padding[0]
}

/// Starts a toolbar group of `width` on the current line, after a thin
/// vertical separator (M102); in a narrow pane the group moves to a new line
/// instead, without a separator (`lay-responsive`).
pub(super) fn begin_toolbar_group(ui: &Ui, theme: &Theme, width: f32) {
    let gap = theme.spacing.small;
    let height = ui.item_rect_size()[1];
    let right_edge = ui.window_content_region_max()[0] + ui.window_pos()[0];
    ui.same_line_with_spacing(0.0, gap);
    let [x, y] = ui.cursor_screen_pos();
    if x + 1.0 + gap + width > right_edge {
        ui.new_line();
        return;
    }
    let inset = (height * 0.2).round();
    ui.get_window_draw_list()
        .add_line(
            [x + 0.5, y + inset],
            [x + 0.5, y + height - inset],
            theme.border.color.to_array(),
        )
        .thickness(1.0)
        .build();
    ui.dummy([1.0, height]);
    ui.same_line_with_spacing(0.0, gap);
}

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

/// Draws the note title with the kind dropdown right behind it (M102, the
/// one shared dropdown since R71), centered on the title's capitals; the
/// dropdown moves below a title that leaves no room.
/// Pushes a chosen kind as an action.
pub(super) fn draw_title_with_kind(
    ui: &Ui,
    language: Language,
    theme: &Theme,
    title: &str,
    (note, kind): (NoteId, NoteKind),
    actions: &mut Vec<WindowAction>,
) {
    let dropdown_size = kind_dropdown_size(ui, language);
    let gap = theme.spacing.medium;
    let available = ui.content_region_avail()[0];
    let [left, top] = ui.cursor_screen_pos();
    let (title_height, title_middle, is_beside) = {
        let _heading = FontScope::push(ui, FontRole::Heading1);
        let height = ui.text_line_height();
        let middle = capital_middle().unwrap_or(height / 2.0);
        let is_beside = ui.calc_text_size(title)[0] + gap + dropdown_size[0] <= available;
        if is_beside {
            ui.text(title);
        } else {
            ui.text_wrapped(title);
        }
        (height, middle, is_beside)
    };
    if is_beside {
        ui.same_line_with_spacing(0.0, gap);
        let x = ui.cursor_screen_pos()[0];
        ui.set_cursor_screen_pos([x, (top + title_middle - dropdown_size[1] / 2.0).round()]);
    }
    if let Some(period) = draw_kind_dropdown(ui, theme, language, kind) {
        actions.push(WindowAction::SetResetPeriod(note, period));
    }
    if is_beside {
        // Continue below the title, not below the centered dropdown.
        let spacing = ui.clone_style().item_spacing[1];
        ui.set_cursor_screen_pos([left, top + title_height + spacing]);
    }
}
