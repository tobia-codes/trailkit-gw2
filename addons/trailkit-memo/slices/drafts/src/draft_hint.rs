//! The draft hint above a note (M54): when the draft was saved, whether the
//! note changed since, and "Restore" (primary) / "Discard".

use jiff::Timestamp;
use nexus::imgui::Ui;
use trailkit_memo_shared::{
    drafts::Draft,
    i18n::{Language, LocalizedText, Text, format_local_time},
    note_tree::NoteId,
};
use trailkit_ui::{
    button::{draw_button, draw_primary_button},
    separator::draw_separator,
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

/// State of the hint that outlives a frame: the formatted label, so the
/// local time is not formatted every frame.
#[derive(Debug, Default)]
pub struct DraftHint {
    /// Cached label for (note, language).
    label: Option<(NoteId, Language, String)>,
}

impl DraftHint {
    /// Draws the hint for `draft` of a note last changed at
    /// `note_updated_at`, followed by a separator. Returns the user's choice.
    pub fn draw(
        &mut self,
        ui: &Ui,
        language: Language,
        theme: &Theme,
        draft: &Draft,
        note_updated_at: Timestamp,
    ) -> Option<DraftHintAction> {
        let is_cached = self
            .label
            .as_ref()
            .is_some_and(|(note, cached_language, _)| {
                *note == draft.note && *cached_language == language
            });
        if !is_cached {
            let time = format_local_time(draft.saved_at, language);
            self.label = Some((
                draft.note,
                language,
                format!("{} {time}", DraftText::DraftFound.get(language)),
            ));
        }
        if let Some((_, _, label)) = self.label.as_ref() {
            ui.text_colored(theme.status.warning.to_array(), label);
        }
        if draft.note_updated_at != note_updated_at {
            ui.text_colored(
                theme.text.secondary.to_array(),
                DraftText::DraftNoteChangedSince.get(language),
            );
        }
        let mut action = None;
        if draw_primary_button(ui, theme, DraftText::RestoreDraft.get(language)) {
            action = Some(DraftHintAction::Restore);
        }
        ui.same_line();
        // "Discard" is shared with the unsaved-changes question.
        if draw_button(ui, theme, Text::DiscardChanges.get(language)) {
            action = Some(DraftHintAction::Discard);
        }
        draw_separator(ui, theme);
        action
    }
}
