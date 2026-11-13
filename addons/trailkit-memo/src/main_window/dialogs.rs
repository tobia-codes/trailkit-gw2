//! In-window dialogs and problem hints of the memo main window: the
//! dialog card, the unsaved-changes question (M70), the delete questions
//! (M134, M146) and storage problems.

use nexus::imgui::{Condition, Key, MouseButton, StyleColor, StyleVar, Ui, Window, WindowFlags};
use trailkit_ui::draw_list::window_draw_list;
use trailkit_ui::{
    button::{Button, ButtonKind, ButtonState},
    font_registry::FontScope,
    fonts::FontRole,
    status_card::{CardDetail, CardStatus, StatusCard, draw_status_card},
    status_line::{Status, draw_status_line},
    text::{TextStyle, draw_text_wrapped},
    theme::Theme,
};

use trailkit_memo_shared::i18n::LocalizedText;
use trailkit_memo_shared::i18n::{Language, Text};
use trailkit_memo_shared::save_text::SaveText;

use super::{
    MemoWindow, NotesProblem, WindowAction, actions::AddonFolder, save_status::SaveFailedPrompt,
    trash::Confirmation,
};

impl MemoWindow {
    /// Draws the storage problem hint, if any, as a status card (M31,
    /// M184). Only the recovered hint can be closed, for this session: the
    /// damaged file was moved away, so the next load is clean (Q336, Q337).
    pub(super) fn draw_problem(&mut self, ui: &Ui, language: Language, theme: &Theme) {
        let Some(problem) = &self.problem else {
            return;
        };
        let open_folder = [Button::new(Text::OpenFolder.get(language))];
        let card = match problem {
            NotesProblem::Recovered(path) => StatusCard {
                status: CardStatus::Danger,
                text: Text::NotesRecovered.get(language),
                detail: Some(CardDetail {
                    text: path,
                    style: TextStyle::Body,
                }),
                buttons: &open_folder,
                close_tooltip: Some(Text::CloseWindow.get(language)),
            },
            NotesProblem::NewerVersion => StatusCard {
                status: CardStatus::Warning,
                text: Text::NotesNewerVersion.get(language),
                detail: None,
                buttons: &[],
                close_tooltip: None,
            },
            NotesProblem::Inaccessible => StatusCard {
                status: CardStatus::Danger,
                text: Text::NotesInaccessible.get(language),
                detail: None,
                buttons: &open_folder,
                close_tooltip: None,
            },
        };
        // The damaged file lies in `backups`; the other problems concern
        // `notes.json` itself.
        let folder = match problem {
            NotesProblem::Recovered(_) => AddonFolder::Backups,
            NotesProblem::NewerVersion | NotesProblem::Inaccessible => AddonFolder::Notes,
        };
        let outcome = draw_status_card(ui, theme, &card);
        if outcome.clicked_button.is_some() {
            self.actions.push(WindowAction::OpenFolder(folder));
        }
        if outcome.closed {
            self.problem = None;
        }
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

/// A button of a dialog card and the answer it gives.
#[derive(Debug, Clone)]
pub(super) struct DialogButton<'a> {
    /// The label.
    pub(super) label: &'a str,
    /// How it looks while it can be used.
    pub(super) kind: ButtonKind,
    /// What it can do now (R93): busy while its action runs, disabled
    /// with a reason while another answer runs.
    pub(super) state: ButtonState<'a>,
    /// The answer applied when it is clicked.
    pub(super) answer: WindowAction,
}

/// What a dialog card shows and how keys answer it.
#[derive(Debug, Clone)]
pub(super) struct DialogContent<'a> {
    /// The heading.
    pub(super) title: &'a str,
    /// The wrapped question below it.
    pub(super) question: &'a str,
    /// A `danger` line with `circle-alert` below the question, e.g. "Failed
    /// again" (M180, Q157).
    pub(super) status: Option<&'a str>,
    /// A button set apart on the left (e.g. "Discard").
    pub(super) apart: Option<DialogButton<'a>>,
    /// The buttons at the right end, the main one last.
    pub(super) buttons: Vec<DialogButton<'a>>,
    /// The harmless answer: Escape and a click on the dimmed area.
    pub(super) escape: WindowAction,
    /// The answer of Enter, if Enter answers at all.
    pub(super) enter: Option<WindowAction>,
}

/// The shared button of a dialog button, `width` wide (`0.0` fits).
fn button_of<'a>(button: &DialogButton<'a>, width: f32) -> Button<'a> {
    Button::new(button.label)
        .kind(button.kind)
        .state(button.state)
        .width(width)
}

/// The width a dialog button takes in a row.
fn dialog_button_width(ui: &Ui, theme: &Theme, button: &DialogButton<'_>) -> f32 {
    button_of(button, 0.0).measure(ui, theme)
}

/// Draws one dialog button `size[0]` wide (`0.0` fits) and returns whether
/// it was clicked.
fn draw_dialog_button(ui: &Ui, theme: &Theme, button: &DialogButton<'_>, size: [f32; 2]) -> bool {
    button_of(button, size[0]).draw(ui, theme)
}

/// Draws a dialog card centered in the current window (M54, M56, M70,
/// `overlay-dialog-layout`): `bg.panel` with a `border` frame and
/// `space.lg` padding; heading, wrapped question, then the button set apart
/// on the left and the others at the right end, stacked full width (main
/// button first, the one set apart last) when they do not fit.
/// `last_height` is the card height measured in the previous frame.
pub(super) fn draw_dialog_card(
    ui: &Ui,
    theme: &Theme,
    content: &DialogContent<'_>,
    last_height: Option<f32>,
) -> DialogCard {
    let spacing = ui.clone_style().item_spacing;
    let padding = theme.spacing.large;
    let window_position = ui.window_pos();
    let window_size = ui.window_size();
    let right_group = right_group_width(ui, theme, content);
    let apart_width = content.apart.as_ref().map_or(0.0, |button| {
        dialog_button_width(ui, theme, button) + theme.spacing.extra_large
    });
    let row_width = apart_width + right_group;
    let maximum_content = (window_size[0] * DIALOG_MAXIMUM_FRACTION - 2.0 * padding).max(1.0);
    let content_width = row_width
        .max(ui.calc_text_size(content.question)[0])
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
    let draw_list = window_draw_list(ui);
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
        let _wrap = ui.push_text_wrap_pos_with_pos(left - window_position[0] + content_width);
        draw_text_wrapped(ui, theme, TextStyle::Heading3, content.title);
    }
    {
        // Every new line starts at the window padding; move into the card.
        ui.set_cursor_screen_pos([left, ui.cursor_screen_pos()[1]]);
        let _wrap = ui.push_text_wrap_pos_with_pos(left - window_position[0] + content_width);
        draw_text_wrapped(ui, theme, TextStyle::Body, content.question);
    }
    if let Some(status) = content.status {
        draw_dialog_status(ui, theme, status, left);
    }
    ui.dummy([0.0, theme.spacing.small]);
    let mut answer = draw_dialog_buttons(ui, theme, content, [left, content_width], stacked);
    if ui.is_key_pressed(Key::Escape) {
        answer = Some(content.escape.clone());
    } else if let Some(enter) = &content.enter
        && ui.is_key_pressed(Key::Enter)
    {
        answer = Some(enter.clone());
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

/// Draws the `danger` status line of a dialog card at `left` (screen x):
/// `circle-alert` and `status` (M180, Q157).
fn draw_dialog_status(ui: &Ui, theme: &Theme, status: &str, left: f32) {
    ui.set_cursor_screen_pos([left, ui.cursor_screen_pos()[1]]);
    draw_status_line(ui, theme, Status::Danger, status, TextStyle::Body);
}

/// The width of the buttons at the right end of a dialog card.
fn right_group_width(ui: &Ui, theme: &Theme, content: &DialogContent<'_>) -> f32 {
    let spacing = ui.clone_style().item_spacing[0];
    content
        .buttons
        .iter()
        .map(|button| dialog_button_width(ui, theme, button))
        .sum::<f32>()
        + spacing * content.buttons.len().saturating_sub(1) as f32
}

/// Draws the buttons of a dialog card in the column starting at
/// `[left, width]` (screen x, width): side by side, or stacked full width
/// (main button first, the one set apart last). Returns the clicked answer.
fn draw_dialog_buttons(
    ui: &Ui,
    theme: &Theme,
    content: &DialogContent<'_>,
    [left, content_width]: [f32; 2],
    stacked: bool,
) -> Option<WindowAction> {
    let mut answer = None;
    let right_group = right_group_width(ui, theme, content);
    if stacked {
        let full = [content_width, 0.0];
        for button in content.buttons.iter().rev() {
            ui.set_cursor_screen_pos([left, ui.cursor_screen_pos()[1]]);
            if draw_dialog_button(ui, theme, button, full) {
                answer = Some(button.answer.clone());
            }
        }
        if let Some(button) = &content.apart {
            ui.dummy([0.0, theme.spacing.small]);
            ui.set_cursor_screen_pos([left, ui.cursor_screen_pos()[1]]);
            if draw_dialog_button(ui, theme, button, full) {
                answer = Some(button.answer.clone());
            }
        }
    } else {
        ui.set_cursor_screen_pos([left, ui.cursor_screen_pos()[1]]);
        if let Some(button) = &content.apart {
            if draw_dialog_button(ui, theme, button, [0.0, 0.0]) {
                answer = Some(button.answer.clone());
            }
            ui.same_line();
        }
        ui.set_cursor_screen_pos([
            left + content_width - right_group,
            ui.cursor_screen_pos()[1],
        ]);
        for (index, button) in content.buttons.iter().enumerate() {
            if index > 0 {
                ui.same_line();
            }
            if draw_dialog_button(ui, theme, button, [0.0, 0.0]) {
                answer = Some(button.answer.clone());
            }
        }
    }
    answer
}

/// The unsaved changes dialog (M54, M56, M70): the red "Discard" (R106) set apart on the
/// left, "Keep editing" and the primary "Save" on the right. Enter saves,
/// Escape keeps editing.
pub(super) fn unsaved_changes_dialog(language: Language) -> DialogContent<'static> {
    DialogContent {
        title: Text::UnsavedChangesTitle.get(language),
        question: Text::UnsavedChangesQuestion.get(language),
        status: None,
        apart: Some(DialogButton {
            label: Text::DiscardChanges.get(language),
            kind: ButtonKind::DangerFill,
            state: ButtonState::Ready,
            answer: WindowAction::LeaveDiscarding,
        }),
        buttons: vec![
            DialogButton {
                label: Text::KeepEditing.get(language),
                kind: ButtonKind::Regular,
                state: ButtonState::Ready,
                answer: WindowAction::LeaveCancelled,
            },
            DialogButton {
                label: Text::KeepChanges.get(language),
                kind: ButtonKind::Primary,
                state: ButtonState::Ready,
                answer: WindowAction::LeaveSaving,
            },
        ],
        escape: WindowAction::LeaveCancelled,
        enter: Some(WindowAction::LeaveSaving),
    }
}

/// The dialog when the user leaves the editor after a failed save (M180,
/// Q157, mockup V4): the red "Discard" (R106) set apart on the left, "Keep editing" and
/// the primary "Save again" (busy while it writes) on the right; after a
/// failed "Save again" a red "Failed again" line. While "Save again" is
/// busy, the other answers are disabled and name the reason
/// (`ix-busy-buttons`, R93). Enter saves again, Escape keeps editing.
pub(super) fn save_failed_dialog(
    prompt: &SaveFailedPrompt,
    language: Language,
) -> DialogContent<'static> {
    // A write that failed at once keeps "Saving…" for its minimum time;
    // meanwhile the other answers wait and say why (R93).
    let save_state = prompt.save.button_state();
    let saving = save_state == ButtonState::Busy;
    let save_label = if saving {
        SaveText::Saving.get(language)
    } else {
        SaveText::SaveAgain.get(language)
    };
    let others = if saving {
        ButtonState::Disabled {
            reason: SaveText::AvailableAfterSaving.get(language),
        }
    } else {
        ButtonState::Ready
    };
    DialogContent {
        title: SaveText::SaveFailed.get(language),
        question: SaveText::DiscardDeletesChanges.get(language),
        status: (prompt.failed_again && !saving).then(|| SaveText::FailedAgain.get(language)),
        apart: Some(DialogButton {
            label: Text::DiscardChanges.get(language),
            kind: ButtonKind::DangerFill,
            state: others,
            answer: WindowAction::SaveFailedDiscard,
        }),
        buttons: vec![
            DialogButton {
                label: Text::KeepEditing.get(language),
                kind: ButtonKind::Regular,
                state: others,
                answer: WindowAction::SaveFailedKeepEditing,
            },
            DialogButton {
                label: save_label,
                kind: ButtonKind::Primary,
                state: save_state,
                answer: WindowAction::SaveFailedRetry,
            },
        ],
        escape: WindowAction::SaveFailedKeepEditing,
        enter: (!saving).then_some(WindowAction::SaveFailedRetry),
    }
}

/// The delete question `confirmation` (M134, M146): "Cancel" and the
/// `danger` confirm button. Enter confirms only what can be undone.
fn confirmation_dialog<'a>(
    confirmation: &'a Confirmation,
    language: Language,
) -> DialogContent<'a> {
    DialogContent {
        title: &confirmation.title,
        question: &confirmation.text,
        status: None,
        apart: None,
        buttons: vec![
            DialogButton {
                label: Text::Cancel.get(language),
                kind: ButtonKind::Regular,
                state: ButtonState::Ready,
                answer: WindowAction::ConfirmCancelled,
            },
            DialogButton {
                label: confirmation.confirm_label,
                kind: ButtonKind::DangerFill,
                state: ButtonState::Ready,
                answer: WindowAction::ConfirmAccepted,
            },
        ],
        escape: WindowAction::ConfirmCancelled,
        enter: confirmation
            .is_undoable()
            .then_some(WindowAction::ConfirmAccepted),
    }
}

impl MemoWindow {
    /// Draws the open dialog (the unsaved changes question, M54, or a
    /// delete question, M134, M146) and applies the answer
    /// (`overlay-dialog-scope`). Call it after the main window is finished:
    /// the dialog is its own borderless window exactly over the main window,
    /// so it dims only that window (title bar included) while the game and
    /// other windows stay usable. Clicking the main window brings the dialog
    /// back to the front; a click on the dimmed area picks the harmless
    /// answer, like Escape.
    pub fn draw_dialog(&mut self, ui: &Ui, language: Language, theme: &Theme) {
        let content = if self.leave_prompt.is_some() {
            unsaved_changes_dialog(language)
        } else if let Some(prompt) = &self.save_failed_prompt {
            save_failed_dialog(prompt, language)
        } else if let Some(confirmation) = &self.confirmation {
            confirmation_dialog(confirmation, language)
        } else {
            self.dialog_height = None;
            return;
        };
        let _body_font = FontScope::push(ui, FontRole::Body);
        let Some(window_rect) = self.window_rect else {
            return;
        };
        let _background =
            ui.push_style_color(StyleColor::WindowBg, theme.surface.modal_overlay.to_array());
        let _border = ui.push_style_var(StyleVar::WindowBorderSize(0.0));
        let mut answer = None;
        let mut measured = self.dialog_height;
        Window::new("###trailkit-memo-dialog")
            .position(window_rect[0], Condition::Always)
            .size(window_rect[1], Condition::Always)
            .flags(
                WindowFlags::NO_DECORATION
                    | WindowFlags::NO_MOVE
                    | WindowFlags::NO_SAVED_SETTINGS
                    | WindowFlags::NO_NAV,
            )
            .focused(self.window_focused)
            .build(ui, || {
                let card = draw_dialog_card(ui, theme, &content, measured);
                measured = Some(card.height);
                answer = card.answer;
                // A click on the dimmed area picks the harmless answer
                // (`overlay-dialog-layout`).
                let mouse = ui.io().mouse_pos;
                let on_card = (card.min[0]..=card.max[0]).contains(&mouse[0])
                    && (card.min[1]..=card.max[1]).contains(&mouse[1]);
                if answer.is_none()
                    && !on_card
                    && ui.is_window_hovered()
                    && ui.is_mouse_clicked(MouseButton::Left)
                {
                    answer = Some(content.escape.clone());
                }
            });
        self.dialog_height = measured;
        if let Some(answer) = answer {
            self.apply(&answer, language);
        }
    }
}
