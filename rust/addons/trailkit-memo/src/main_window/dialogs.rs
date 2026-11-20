//! In-window dialogs and problem hints of the memo main window: the
//! dialog content, the unsaved-changes question (M70), the delete questions
//! (M134, M146) and storage problems.

pub(super) mod closing;
pub(super) mod save_status;

use nexus::imgui::{Key, Ui, WindowFocusedFlags};
use trailkit_ui::{
    controls::button::{Button, ButtonKind, ButtonState},
    controls::text_field::move_below_last_item,
    feedback::dialog::{Dialog, DialogHeight, maximum_content_width},
    feedback::status_card::{CardDetail, CardStatus, StatusCard, draw_status_card},
    feedback::status_line::{Status, draw_status_line},
    layout::stack::Stack,
    text::{TextStyle, draw_text_wrapped},
    theme::Theme,
};

use trailkit_memo_shared::i18n::LocalizedText;
use trailkit_memo_shared::i18n::{Language, Text};
use trailkit_memo_shared::save_text::SaveText;

use self::save_status::SaveFailedPrompt;
use super::{MemoWindow, NotesProblem, WindowAction, actions::AddonFolder, trash::Confirmation};

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

/// Minimum content width of a dialog card in average characters.
pub(super) const DIALOG_MINIMUM_CHARACTERS: f32 = 28.0;

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
    /// Tooltip of the close icon at the top right (R141).
    pub(super) close: &'a str,
    /// The harmless answer: the close icon, Escape and a click on the dimmed
    /// area.
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

/// The width of a dialog's content in a window `window_width` wide: the
/// question or the button row, at least [`DIALOG_MINIMUM_CHARACTERS`] and at
/// most the shared dialog maximum (R153, memo M70).
fn dialog_content_width(
    ui: &Ui,
    theme: &Theme,
    content: &DialogContent<'_>,
    window_width: f32,
) -> f32 {
    let maximum_content = maximum_content_width(ui, theme, window_width);
    dialog_row_width(ui, theme, content)
        .max(ui.calc_text_size(content.question)[0])
        .max(ui.calc_text_size("n")[0] * DIALOG_MINIMUM_CHARACTERS)
        .min(maximum_content)
}

/// The width the buttons take in one row: the one set apart, a gap, and the
/// buttons at the right end.
fn dialog_row_width(ui: &Ui, theme: &Theme, content: &DialogContent<'_>) -> f32 {
    let apart_width = content.apart.as_ref().map_or(0.0, |button| {
        dialog_button_width(ui, theme, button) + theme.spacing.extra_large
    });
    apart_width + right_group_width(ui, theme, content)
}

/// Draws the content of a dialog below its title, `content_width` wide (M54,
/// M56, M70, `overlay-dialog-layout`): the wrapped question, the status line,
/// then the button set apart on the left and the others at the right end,
/// stacked full width (main button first, the one set apart last) when they
/// do not fit. Returns the answer of a clicked button or of Enter.
fn draw_dialog_content(
    ui: &Ui,
    theme: &Theme,
    content: &DialogContent<'_>,
    content_width: f32,
) -> Option<WindowAction> {
    let left = ui.cursor_screen_pos()[0];
    let stacked = dialog_row_width(ui, theme, content) > content_width;
    {
        let _wrap = ui.push_text_wrap_pos_with_pos(ui.cursor_pos()[0] + content_width);
        draw_text_wrapped(ui, theme, TextStyle::Body, content.question);
    }
    if let Some(status) = content.status {
        draw_dialog_status(ui, theme, status, left);
    }
    move_below_last_item(ui, theme.spacing.large);
    // Whole pixels keep the button edges crisp.
    let [x, y] = ui.cursor_screen_pos();
    ui.set_cursor_screen_pos([x, y.round()]);
    let answer = draw_dialog_buttons(ui, theme, content, [left, content_width], stacked);
    answer.or_else(|| {
        content
            .enter
            .clone()
            .filter(|_| ui.is_key_pressed(Key::Enter))
    })
}

/// Draws the `danger` status line of a dialog card at `left` (screen x):
/// `circle-alert` and `status` (M180, Q157).
fn draw_dialog_status(ui: &Ui, theme: &Theme, status: &str, left: f32) {
    ui.set_cursor_screen_pos([left, ui.cursor_screen_pos()[1]]);
    draw_status_line(ui, theme, Status::Danger, status, TextStyle::Body);
}

/// The width of the buttons at the right end of a dialog card.
fn right_group_width(ui: &Ui, theme: &Theme, content: &DialogContent<'_>) -> f32 {
    let spacing = theme.spacing.small;
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
        // Stacked buttons are `space.sm` apart; the one set apart keeps
        // `space.xl` from them (R148).
        let full = [content_width, 0.0];
        ui.set_cursor_screen_pos([left, ui.cursor_screen_pos()[1]]);
        Stack::vertical(theme)
            .gap(theme.spacing.extra_large)
            .build(ui, |outer| {
                outer.add(|| {
                    Stack::vertical(theme).build(ui, |stack| {
                        for button in content.buttons.iter().rev() {
                            stack.add(|| {
                                if draw_dialog_button(ui, theme, button, full) {
                                    answer = Some(button.answer.clone());
                                }
                            });
                        }
                    });
                });
                if let Some(button) = &content.apart {
                    outer.add(|| {
                        if draw_dialog_button(ui, theme, button, full) {
                            answer = Some(button.answer.clone());
                        }
                    });
                }
            });
    } else {
        let spacing = theme.spacing.small;
        ui.set_cursor_screen_pos([left, ui.cursor_screen_pos()[1]]);
        if let Some(button) = &content.apart {
            if draw_dialog_button(ui, theme, button, [0.0, 0.0]) {
                answer = Some(button.answer.clone());
            }
            ui.same_line_with_spacing(0.0, spacing);
        }
        ui.set_cursor_screen_pos([
            (left + content_width - right_group).round(),
            ui.cursor_screen_pos()[1],
        ]);
        for (index, button) in content.buttons.iter().enumerate() {
            if index > 0 {
                ui.same_line_with_spacing(0.0, spacing);
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
        close: Text::CloseWindow.get(language),
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
        close: Text::CloseWindow.get(language),
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
        enter: save_failed_enter(prompt),
    }
}

/// Enter's answer in the save failed dialog: save again, unless it is saving
/// already (M180).
fn save_failed_enter(prompt: &SaveFailedPrompt) -> Option<WindowAction> {
    (prompt.save.button_state() != ButtonState::Busy).then_some(WindowAction::SaveFailedRetry)
}

/// Enter's answer in the delete question `confirmation`: confirm only what
/// can be undone (M152).
fn confirmation_enter(confirmation: &Confirmation) -> Option<WindowAction> {
    confirmation
        .is_undoable()
        .then_some(WindowAction::ConfirmAccepted)
}

/// The delete question `confirmation` (M134, M146): "Cancel" and the
/// `danger` confirm button. Enter confirms only what can be undone.
fn confirmation_dialog<'a>(
    confirmation: &'a Confirmation,
    language: Language,
) -> DialogContent<'a> {
    DialogContent {
        title: &confirmation.title,
        close: Text::CloseWindow.get(language),
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
        enter: confirmation_enter(confirmation),
    }
}

impl MemoWindow {
    /// The answer Enter gives in the open dialog, if any (M56, M152, M180);
    /// the same order as [`Self::draw_dialog`] picks the dialog in.
    fn dialog_enter_answer(&self) -> Option<WindowAction> {
        if self.leave_prompt.is_some() {
            Some(WindowAction::LeaveSaving)
        } else if let Some(prompt) = &self.save_failed_prompt {
            save_failed_enter(prompt)
        } else {
            self.confirmation.as_ref().and_then(confirmation_enter)
        }
    }

    /// Whether Enter belongs to the open dialog: memo or its dialog has the
    /// focus, no text input is active (Nexus would hand it Enter itself) and
    /// Enter answers the dialog. The window message callback catches Enter
    /// only then, because the game would otherwise open its chat (in-game
    /// test 35).
    #[must_use]
    pub fn takes_dialog_enter_key(&self) -> bool {
        (self.window_focused || self.dialog_focused)
            && !self.text_input_active
            && self.dialog_enter_answer().is_some()
    }

    /// Answers the open dialog as Enter does after the window message
    /// callback caught it.
    pub fn confirm_dialog_by_key(&mut self, language: Language) {
        if self.takes_dialog_enter_key()
            && let Some(answer) = self.dialog_enter_answer()
        {
            self.apply(&answer, language);
        }
    }

    /// Draws the open dialog (the unsaved changes question, M54, or a
    /// delete question, M134, M146) and applies the answer
    /// (`overlay-dialog-scope`). Call it after the main window is finished:
    /// the dialog is its own borderless window exactly over the main window,
    /// so it dims only that window (title bar included) while the game and
    /// other windows stay usable. Clicking the main window brings the dialog
    /// back to the front; a click on the dimmed area picks the harmless
    /// answer, like Escape.
    pub fn draw_dialog(&mut self, ui: &Ui, language: Language, theme: &Theme) {
        self.dialog_focused = false;
        let content = if self.leave_prompt.is_some() {
            unsaved_changes_dialog(language)
        } else if let Some(prompt) = &self.save_failed_prompt {
            save_failed_dialog(prompt, language)
        } else if let Some(confirmation) = &self.confirmation {
            confirmation_dialog(confirmation, language)
        } else {
            self.dialog_height = DialogHeight::default();
            self.draw_link_dialog(ui, language, theme);
            return;
        };
        let Some([position, size]) = self.window_rect else {
            return;
        };
        let mut dialog_focused = false;
        let response = Dialog::new(
            "###trailkit-memo-dialog",
            content.title,
            content.close,
            position,
            size,
        )
        .focused(self.window_focused)
        .draw(
            ui,
            theme,
            &mut self.dialog_height,
            |window_width| dialog_content_width(ui, theme, &content, window_width),
            |content_width| {
                dialog_focused =
                    ui.is_window_focused_with_flags(WindowFocusedFlags::ROOT_AND_CHILD_WINDOWS);
                draw_dialog_content(ui, theme, &content, content_width)
            },
        );
        self.dialog_focused = dialog_focused;
        let answer = response.and_then(|response| {
            if response.is_closed {
                Some(content.escape.clone())
            } else {
                response.content
            }
        });
        if let Some(answer) = answer {
            self.apply(&answer, language);
        }
    }

    /// Draws the link dialog of the editor, if open, over the dimmed window
    /// like the other dialogs (M208).
    fn draw_link_dialog(&mut self, ui: &Ui, language: Language, theme: &Theme) {
        let Some(window) = self.window_rect else {
            return;
        };
        let is_focused = self.window_focused;
        if let Some(editor) = self.editor.as_mut() {
            editor.draw_link_dialog(ui, language, theme, window, is_focused);
        }
    }
}
