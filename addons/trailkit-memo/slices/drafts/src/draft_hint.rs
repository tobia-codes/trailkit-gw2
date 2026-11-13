//! The draft hint above a note (M54, M184): a `warning` status card that
//! says when the draft was saved and whether the note changed since, with
//! "Discard" / "Restore" (primary, last). It cannot be closed: the two buttons
//! answer it (Q336).

use jiff::Timestamp;
use nexus::imgui::Ui;
use trailkit_memo_shared::{
    drafts::Draft,
    i18n::{Language, LocalizedText, Text, format_local_time},
    note_tree::NoteId,
};
use trailkit_ui::{
    button::{Button, ButtonKind},
    status_card::{CardDetail, CardStatus, StatusCard, draw_status_card, fits_status_card_line},
    text::TextStyle,
    theme::Theme,
};

use crate::i18n::DraftText;

/// What the user chose in the hint.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DraftHintAction {
    /// Continue editing the note from its draft.
    Restore,
    /// Delete the draft.
    Discard,
}

/// The hint's sentence in its two forms.
#[derive(Debug)]
struct DraftLabel {
    /// The note and language the label was formatted for.
    key: (NoteId, Language),
    /// The sentence on one line.
    one_line: String,
    /// The sentence with the date on a line of its own, for a card too
    /// narrow for one line, so the date never breaks at its dots (Q339).
    date_apart: String,
}

/// State of the hint that outlives a frame: the formatted label, so the
/// local time is not formatted every frame.
#[derive(Debug, Default)]
pub struct DraftHint {
    /// Cached label.
    label: Option<DraftLabel>,
}

impl DraftHint {
    /// Draws the hint for `draft` of a note last changed at
    /// `note_updated_at` as a status card. Returns the user's choice.
    pub fn draw(
        &mut self,
        ui: &Ui,
        language: Language,
        theme: &Theme,
        draft: &Draft,
        note_updated_at: Timestamp,
    ) -> Option<DraftHintAction> {
        let key = (draft.note, language);
        let label = match &mut self.label {
            Some(label) if label.key == key => label,
            label => {
                let found = DraftText::DraftFound.get(language);
                let time = format_local_time(draft.saved_at, language);
                label.insert(DraftLabel {
                    key,
                    one_line: format!("{found} {time}"),
                    date_apart: format!("{found}\n{time}"),
                })
            }
        };
        let text = if fits_status_card_line(ui, theme, CardStatus::Warning, false, &label.one_line)
        {
            label.one_line.as_str()
        } else {
            label.date_apart.as_str()
        };
        let detail = (draft.note_updated_at != note_updated_at).then(|| CardDetail {
            text: DraftText::DraftNoteChangedSince.get(language),
            style: TextStyle::Secondary,
        });
        // "Discard" is shared with the unsaved-changes question.
        // The brass button last (R103, Q349).
        let buttons = [
            Button::new(Text::DiscardChanges.get(language)),
            Button::new(DraftText::RestoreDraft.get(language)).kind(ButtonKind::Primary),
        ];
        let outcome = draw_status_card(
            ui,
            theme,
            &StatusCard {
                status: CardStatus::Warning,
                text,
                detail,
                buttons: &buttons,
                close_tooltip: None,
            },
        );
        match outcome.clicked_button {
            Some(0) => Some(DraftHintAction::Discard),
            Some(1) => Some(DraftHintAction::Restore),
            _ => None,
        }
    }
}
