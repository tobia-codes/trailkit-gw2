//! The popup that inserts or edits a link (memo plan M208): text and address
//! in labelled fields, opened from the link button and Ctrl+K.

use nexus::imgui::{StyleVar, Ui};
use trailkit_memo_shared::i18n::{Language, LocalizedText, text_field_texts};
use trailkit_ui::{
    controls::button::{Button, ButtonKind, ButtonState, draw_popup_icon_button},
    controls::menu::draw_popup_at,
    controls::text_field::TextField,
    text::icons::Icon,
    theme::Theme,
};

use crate::{
    i18n::EditorText,
    link_edit::{AddressProblem, LinkDraft, LinkField, normalized_address},
};

/// ImGui id of the popup.
const POPUP_ID: &str = "##link_popup";

/// Placeholder of the address field.
const ADDRESS_PLACEHOLDER: &str = "https://";

/// Width of the popup in multiples of the font size.
const WIDTH_IN_FONT_SIZES: f32 = 27.0;

/// Height assumed for placing the popup until it was drawn once.
const FIRST_HEIGHT_IN_FONT_SIZES: f32 = 13.0;

/// How many frames a field asks for the keyboard focus; ImGui may drop a
/// request while the popup appears.
const MAX_FOCUS_REQUESTS: u8 = 4;

/// Where the popup goes, in screen coordinates.
#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct LinkPopupPlace {
    /// Top-left corner of the cursor.
    pub(crate) cursor: [f32; 2],
    /// Height of the cursor's line.
    pub(crate) line_height: f32,
    /// Top-left corner of the area the popup stays in.
    pub(crate) area_min: [f32; 2],
    /// Bottom-right corner of that area.
    pub(crate) area_max: [f32; 2],
}

/// How the popup ended, if it did.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum LinkPopupOutcome {
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

/// The open link popup.
#[derive(Debug)]
pub(crate) struct LinkPopup {
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
    /// Whether ImGui was told to open the popup.
    is_opened: bool,
    /// Height of the popup in the last frame.
    height: Option<f32>,
}

impl LinkPopup {
    /// A popup with the values of `draft`.
    pub(crate) fn new(draft: LinkDraft) -> Self {
        Self {
            text: draft.text.clone(),
            address: draft.address.clone(),
            focus: draft.focus,
            draft,
            focus_requests: 0,
            is_address_active: false,
            confirm_attempted: false,
            is_opened: false,
            height: None,
        }
    }

    /// What the popup was opened for.
    pub(crate) const fn draft(&self) -> &LinkDraft {
        &self.draft
    }

    /// Draws the popup beside the cursor: above its line if there is room,
    /// else below it. Closing it by a click outside or Escape cancels.
    pub(crate) fn draw(
        &mut self,
        ui: &Ui,
        language: Language,
        theme: &Theme,
        place: LinkPopupPlace,
    ) -> LinkPopupOutcome {
        if !self.is_opened {
            ui.open_popup(POPUP_ID);
            self.is_opened = true;
        }
        let font_size = ui.current_font_size();
        let margin = theme.spacing.small;
        let padding = theme.menu.padding + theme.menu.row_padding_x;
        let room = (place.area_max[0] - place.area_min[0] - 2.0 * margin).max(1.0);
        let width = (font_size * WIDTH_IN_FONT_SIZES).min(room);
        let height = self
            .height
            .unwrap_or(font_size * FIRST_HEIGHT_IN_FONT_SIZES);
        let left = place.area_min[0] + margin;
        let top = place.area_min[1] + margin;
        let x = place.cursor[0].clamp(left, (place.area_max[0] - margin - width).max(left));
        let above = place.cursor[1] - theme.spacing.extra_small - height;
        let y = if above >= top {
            above
        } else {
            (place.cursor[1] + place.line_height + theme.spacing.extra_small)
                .min((place.area_max[1] - margin - height).max(top))
        };

        let mut outcome = LinkPopupOutcome::Cancelled;
        draw_popup_at(ui, theme, POPUP_ID, [x, y], || {
            self.height = Some(ui.window_size()[1]);
            outcome = self.draw_content(ui, language, theme, (width - 2.0 * padding).max(1.0));
            if outcome != LinkPopupOutcome::Open {
                ui.close_current_popup();
            }
        });
        outcome
    }

    /// Draws the fields and buttons, `width` wide.
    fn draw_content(
        &mut self,
        ui: &Ui,
        language: Language,
        theme: &Theme,
        width: f32,
    ) -> LinkPopupOutcome {
        let _spacing = ui.push_style_var(StyleVar::ItemSpacing([
            theme.spacing.small,
            theme.spacing.small,
        ]));
        let wanted = (self.focus_requests < MAX_FOCUS_REQUESTS).then_some(self.focus);
        let problem = normalized_address(&self.address).err();
        let error = problem
            .filter(|problem| {
                self.confirm_attempted
                    || (*problem == AddressProblem::Unsupported && !self.is_address_active)
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

        let buttons = self.draw_buttons(ui, language, theme, width, problem);
        let confirmed = text.submitted || address.submitted || buttons.confirmed;
        if buttons.cancelled {
            LinkPopupOutcome::Cancelled
        } else if buttons.removed {
            LinkPopupOutcome::Removed
        } else if confirmed {
            self.confirm()
        } else {
            LinkPopupOutcome::Open
        }
    }

    /// Draws the button row: "remove link" at the left of an existing link,
    /// "Cancel" and the primary button at the right.
    fn draw_buttons(
        &self,
        ui: &Ui,
        language: Language,
        theme: &Theme,
        width: f32,
        problem: Option<AddressProblem>,
    ) -> ButtonClicks {
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
        let state = problem.map_or(ButtonState::Ready, |problem| ButtonState::Disabled {
            reason: problem_text(problem).get(language),
        });
        let confirm = Button::new(confirm_label)
            .kind(ButtonKind::Primary)
            .state(state);
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
    fn confirm(&mut self) -> LinkPopupOutcome {
        match normalized_address(&self.address) {
            Ok(address) => LinkPopupOutcome::Submitted {
                text: self.text.clone(),
                address,
            },
            Err(_) => {
                self.confirm_attempted = true;
                self.focus = LinkField::Address;
                self.focus_requests = 0;
                LinkPopupOutcome::Open
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
        AddressProblem::Unsupported => EditorText::LinkAddressUnsupported,
    }
}
