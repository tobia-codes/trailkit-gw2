//! In-window dialogs and problem hints of the memo main window: the
//! dialog card, the unsaved-changes question (M70) and storage problems.

use nexus::imgui::{Key, Ui};
use trailkit_ui::{
    button::{
        button_width, draw_button, draw_button_with_size, draw_primary_button,
        draw_primary_button_with_size,
    },
    font_registry::FontScope,
    fonts::FontRole,
    icons::Icon,
    separator::draw_separator,
    theme::Theme,
};

use trailkit_memo_shared::i18n::{Language, Text};

use super::{MemoWindow, NotesProblem, WindowAction};

impl MemoWindow {
    /// Draws the storage problem hint, if any (M31).
    pub(super) fn draw_problem(&mut self, ui: &Ui, language: Language, theme: &Theme) {
        let Some(problem) = &self.problem else {
            return;
        };
        {
            let _icons = FontScope::push(ui, FontRole::Icons);
            let mut buffer = [0; 4];
            ui.text_colored(
                theme.status.warning.to_array(),
                Icon::TRIANGLE_ALERT.as_str(&mut buffer),
            );
        }
        ui.same_line();
        let offers_folder = match problem {
            NotesProblem::Recovered(path) => {
                ui.text_wrapped(Text::NotesRecovered.get(language));
                ui.text_wrapped(path);
                true
            }
            NotesProblem::NewerVersion => {
                ui.text_wrapped(Text::NotesNewerVersion.get(language));
                false
            }
            NotesProblem::Inaccessible => {
                ui.text_wrapped(Text::NotesInaccessible.get(language));
                true
            }
        };
        if offers_folder && draw_button(ui, theme, Text::OpenFolder.get(language)) {
            self.actions.push(WindowAction::OpenFolder);
        }
        draw_separator(ui, theme);
    }
}

/// Where the dialog card was drawn and what the user chose.
pub(super) struct DialogCard {
    /// Top-left corner (screen).
    pub(super) min: [f32; 2],
    /// Bottom-right corner (screen).
    pub(super) max: [f32; 2],
    /// The measured height, for centering in the next frame.
    pub(super) height: f32,
    /// The chosen answer, if any.
    pub(super) answer: Option<WindowAction>,
}

/// Minimum content width of a dialog card in average characters.
pub(super) const DIALOG_MINIMUM_CHARACTERS: f32 = 28.0;

/// Largest share of the window a dialog card may take.
pub(super) const DIALOG_MAXIMUM_FRACTION: f32 = 0.9;

/// Draws the card of the unsaved changes dialog centered in the current
/// window (M54, M56, M70, `overlay-dialog-layout`): `bg.panel` with a
/// `border` frame and `space.lg` padding; heading, wrapped question, then
/// "Discard" on the left and "Keep editing" plus the primary "Save" on the
/// right, stacked full width (Save, Keep editing, Discard) when they do not
/// fit. `last_height` is the card height measured in the previous frame.
/// Enter saves, Escape keeps editing.
pub(super) fn draw_unsaved_changes_card(
    ui: &Ui,
    language: Language,
    theme: &Theme,
    last_height: Option<f32>,
) -> DialogCard {
    let title = Text::UnsavedChangesTitle.get(language);
    let question = Text::UnsavedChangesQuestion.get(language);
    let discard = Text::DiscardChanges.get(language);
    let keep_editing = Text::KeepEditing.get(language);
    let save = Text::KeepChanges.get(language);
    let spacing = ui.clone_style().item_spacing;
    let padding = theme.spacing.large;
    let window_position = ui.window_pos();
    let window_size = ui.window_size();
    let row_width = button_width(ui, theme, discard)
        + theme.spacing.extra_large
        + button_width(ui, theme, keep_editing)
        + spacing[0]
        + button_width(ui, theme, save);
    let maximum_content = (window_size[0] * DIALOG_MAXIMUM_FRACTION - 2.0 * padding).max(1.0);
    let content_width = row_width
        .max(ui.calc_text_size(question)[0])
        .max(ui.calc_text_size("n")[0] * DIALOG_MINIMUM_CHARACTERS)
        .min(maximum_content);
    let stacked = row_width > content_width;
    let width = content_width + 2.0 * padding;
    let height = last_height.unwrap_or(window_size[1] / 3.0);
    let min = [
        window_position[0] + ((window_size[0] - width) / 2.0).max(0.0),
        window_position[1] + ((window_size[1] - height) / 2.0).max(0.0),
    ];
    let max = [min[0] + width, min[1] + height];
    let draw_list = ui.get_window_draw_list();
    draw_list
        .add_rect(min, max, theme.surface.panel.to_array())
        .filled(true)
        .rounding(theme.radius.large)
        .build();
    draw_list
        .add_rect(min, max, theme.border.subtle.to_array())
        .thickness(theme.border.width.max(1.0))
        .rounding(theme.radius.large)
        .build();
    let left = min[0] + padding;
    ui.set_cursor_screen_pos([left, min[1] + padding]);
    {
        let _heading = FontScope::push(ui, FontRole::Heading3);
        ui.text(title);
    }
    {
        // Every new line starts at the window padding; move into the card.
        ui.set_cursor_screen_pos([left, ui.cursor_screen_pos()[1]]);
        let _wrap = ui.push_text_wrap_pos_with_pos(left - window_position[0] + content_width);
        ui.text_wrapped(question);
    }
    ui.dummy([0.0, theme.spacing.small]);
    let mut answer = None;
    if stacked {
        let full = [content_width, 0.0];
        ui.set_cursor_screen_pos([left, ui.cursor_screen_pos()[1]]);
        if draw_primary_button_with_size(ui, theme, save, full) {
            answer = Some(WindowAction::LeaveSaving);
        }
        ui.set_cursor_screen_pos([left, ui.cursor_screen_pos()[1]]);
        if draw_button_with_size(ui, theme, keep_editing, full) {
            answer = Some(WindowAction::LeaveCancelled);
        }
        ui.dummy([0.0, theme.spacing.small]);
        ui.set_cursor_screen_pos([left, ui.cursor_screen_pos()[1]]);
        if draw_button_with_size(ui, theme, discard, full) {
            answer = Some(WindowAction::LeaveDiscarding);
        }
    } else {
        ui.set_cursor_screen_pos([left, ui.cursor_screen_pos()[1]]);
        if draw_button(ui, theme, discard) {
            answer = Some(WindowAction::LeaveDiscarding);
        }
        let right_group =
            button_width(ui, theme, keep_editing) + spacing[0] + button_width(ui, theme, save);
        ui.same_line_with_pos(left + content_width - right_group - window_position[0]);
        if draw_button(ui, theme, keep_editing) {
            answer = Some(WindowAction::LeaveCancelled);
        }
        ui.same_line();
        if draw_primary_button(ui, theme, save) {
            answer = Some(WindowAction::LeaveSaving);
        }
    }
    if ui.is_key_pressed(Key::Escape) {
        answer = Some(WindowAction::LeaveCancelled);
    } else if ui.is_key_pressed(Key::Enter) {
        answer = Some(WindowAction::LeaveSaving);
    }
    // The last item adds item spacing below itself; the card ends there plus padding.
    let measured = ui.cursor_screen_pos()[1] - spacing[1] - min[1] + padding;
    DialogCard {
        min,
        max,
        height: measured,
        answer,
    }
}
