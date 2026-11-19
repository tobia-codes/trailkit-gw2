//! The dialog that inserts or edits a link (memo plan M208): text and address
//! in labelled fields of a shared dialog over the editor's window, opened
//! from the link button and Ctrl+K.

use nexus::imgui::{StyleVar, Ui};
use trailkit_memo_shared::i18n::{Language, LocalizedText, Text, text_field_texts};
use trailkit_ui::{
    controls::button::{Button, ButtonKind, draw_popup_icon_button},
    controls::text_field::{TextField, move_below_last_item},
    drawing::screen_rect::ScreenRect,
    feedback::dialog::{Dialog, DialogHeight},
    text::icons::Icon,
    theme::Theme,
};

use crate::{
    i18n::EditorText,
    link_edit::{AddressProblem, LinkDraft, LinkField, normalized_address},
};

/// Placeholder of the address field.
const ADDRESS_PLACEHOLDER: &str = "https://";

/// Width of the fields in multiples of the font size.
const CONTENT_WIDTH_IN_FONT_SIZES: f32 = 24.0;

/// The largest share of the window width the card may take.
const MAXIMUM_WINDOW_FRACTION: f32 = 0.9;

/// How many frames a field asks for the keyboard focus; ImGui may drop a
/// request while the dialog appears.
const MAX_FOCUS_REQUESTS: u8 = 4;

/// How the dialog ended, if it did.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum LinkDialogOutcome {
    /// It stays open.
    Open,
    /// It was closed without a change.
    Cancelled,
    /// The link should be written.
    Submitted {
        /// The link text.
        text: String,
        /// The checked address.
        address: String,
    },
    /// The link should become plain text.
    Removed,
}

/// The open link dialog.
#[derive(Debug)]
pub(crate) struct LinkDialog {
    /// What it was opened for.
    draft: LinkDraft,
    /// The edited link text.
    text: String,
    /// The edited address.
    address: String,
    /// The field to focus.
    focus: LinkField,
    /// How often that field asked for the focus.
    focus_requests: u8,
    /// Whether the address field had the focus last frame.
    is_address_active: bool,
    /// Whether the user tried to confirm, so a missing address is shown.
    confirm_attempted: bool,
    /// Height of the dialog in the last frame.
    height: DialogHeight,
}

impl LinkDialog {
    /// A dialog with the values of `draft`.
    pub(crate) fn new(draft: LinkDraft) -> Self {
        Self {
            text: draft.text.clone(),
            address: draft.address.clone(),
            focus: draft.focus,
            draft,
            focus_requests: 0,
            is_address_active: false,
            confirm_attempted: false,
            height: DialogHeight::default(),
        }
    }

    /// What the dialog was opened for.
    pub(crate) const fn draft(&self) -> &LinkDraft {
        &self.draft
    }

    /// Draws the dialog over `window`, the editor's window on the screen.
    /// The close icon, Escape or a click on the dimmed area cancels.
    pub(crate) fn draw(
        &mut self,
        ui: &Ui,
        language: Language,
        theme: &Theme,
        window: ScreenRect,
        is_focused: bool,
    ) -> LinkDialogOutcome {
        let title = if self.draft.is_existing {
            EditorText::LinkEditTitle
        } else {
            EditorText::LinkInsertTitle
        };
        let mut height = self.height;
        let response = Dialog::new(
            "###trailkit-memo-link-dialog",
            title.get(language),
            Text::CloseWindow.get(language),
            window,
        )
        .focused(is_focused)
        .draw(
            ui,
            theme,
            &mut height,
            |window_width| {
                let padding = theme.spacing.large;
                (ui.current_font_size() * CONTENT_WIDTH_IN_FONT_SIZES)
                    .min(window_width * MAXIMUM_WINDOW_FRACTION - 2.0 * padding)
            },
            |width| {
                let spacing = theme.spacing.small;
                let _spacing = ui.push_style_var(StyleVar::ItemSpacing([spacing, spacing]));
                self.draw_content(ui, language, theme, width)
            },
        );
        self.height = height;
        match response {
            Some(response) if response.is_closed => LinkDialogOutcome::Cancelled,
            Some(response) => response.content,
            None => LinkDialogOutcome::Open,
        }
    }

    /// Draws the fields and the buttons below the title, `width` wide.
    fn draw_content(
        &mut self,
        ui: &Ui,
        language: Language,
        theme: &Theme,
        width: f32,
    ) -> LinkDialogOutcome {
        let wanted = (self.focus_requests < MAX_FOCUS_REQUESTS).then_some(self.focus);
        let error = normalized_address(&self.address)
            .err()
            .filter(|problem| {
                self.confirm_attempted
                    || (*problem == AddressProblem::Invalid && !self.is_address_active)
            })
            .map(|problem| problem_text(problem).get(language));

        let field_texts = text_field_texts(language);
        let text = TextField::new(
            "text",
            EditorText::LinkText.get(language),
            &mut self.text,
            width,
        )
        .optional(true)
        .focus(wanted == Some(LinkField::Text))
        .draw(ui, theme, field_texts);
        move_below_last_item(ui, theme.spacing.medium);
        let address = TextField::new(
            "address",
            EditorText::LinkAddress.get(language),
            &mut self.address,
            width,
        )
        .placeholder(ADDRESS_PLACEHOLDER)
        .error(error)
        .focus(wanted == Some(LinkField::Address))
        .draw(ui, theme, field_texts);
        self.is_address_active = address.is_active;
        let is_focused = match self.focus {
            LinkField::Text => text.is_active,
            LinkField::Address => address.is_active,
        };
        self.focus_requests = if is_focused {
            MAX_FOCUS_REQUESTS
        } else {
            self.focus_requests.saturating_add(1)
        };

        move_below_last_item(ui, theme.spacing.large);
        let buttons = self.draw_buttons(ui, language, theme, width);
        if buttons.cancelled {
            LinkDialogOutcome::Cancelled
        } else if buttons.removed {
            LinkDialogOutcome::Removed
        } else if text.submitted || address.submitted || buttons.confirmed {
            self.confirm()
        } else {
            LinkDialogOutcome::Open
        }
    }

    /// Draws the button row: "remove link" at the left of an existing link,
    /// "Cancel" and the primary button at the right.
    fn draw_buttons(&self, ui: &Ui, language: Language, theme: &Theme, width: f32) -> ButtonClicks {
        let start = ui.cursor_screen_pos();
        let removed = self.draft.is_existing
            && draw_popup_icon_button(
                ui,
                theme,
                Icon::UNLINK,
                EditorText::LinkRemove.get(language),
            );
        let confirm_label = if self.draft.is_existing {
            EditorText::LinkSave
        } else {
            EditorText::LinkInsert
        }
        .get(language);
        let confirm = Button::new(confirm_label).kind(ButtonKind::Primary);
        let cancel = Button::new(EditorText::LinkCancel.get(language));
        let confirm_width = confirm.measure(ui, theme);
        let cancel_width = cancel.measure(ui, theme);
        let cancel_x = start[0] + width - confirm_width - theme.spacing.small - cancel_width;
        ui.set_cursor_screen_pos([cancel_x, start[1]]);
        let cancelled = cancel.draw(ui, theme);
        ui.same_line_with_spacing(0.0, theme.spacing.small);
        let confirmed = confirm.draw(ui, theme);
        ButtonClicks {
            removed,
            cancelled,
            confirmed,
        }
    }

    /// Accepts a usable address; otherwise shows why and returns to its field.
    fn confirm(&mut self) -> LinkDialogOutcome {
        match normalized_address(&self.address) {
            Ok(address) => LinkDialogOutcome::Submitted {
                text: self.text.clone(),
                address,
            },
            Err(_) => {
                self.confirm_attempted = true;
                self.focus = LinkField::Address;
                self.focus_requests = 0;
                LinkDialogOutcome::Open
            }
        }
    }
}

/// The buttons clicked in this frame.
#[derive(Debug, Clone, Copy)]
struct ButtonClicks {
    /// "Remove link".
    removed: bool,
    /// "Cancel".
    cancelled: bool,
    /// "Insert" or "Save".
    confirmed: bool,
}

/// The text that explains `problem` and how to solve it.
const fn problem_text(problem: AddressProblem) -> EditorText {
    match problem {
        AddressProblem::Missing => EditorText::LinkAddressMissing,
        AddressProblem::Invalid => EditorText::LinkAddressInvalid,
    }
}
