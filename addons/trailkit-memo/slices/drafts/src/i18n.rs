//! The drafts slice's user-facing strings in English and German (ROADMAP
//! R76: every slice owns its texts; nexus-addon-dev skill §1).

use trailkit_memo_shared::i18n::{Language, LocalizedText};

/// A translatable string of the drafts slice.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DraftText {
    /// Draft hint, followed by the date and time of the draft (M54).
    DraftFound,
    /// Draft hint addition when the note changed after the draft (M54).
    DraftNoteChangedSince,
    /// Restore a draft into the editor (M54).
    RestoreDraft,
}

impl LocalizedText for DraftText {
    const ALL: &'static [Self] = &[
        Self::DraftFound,
        Self::DraftNoteChangedSince,
        Self::RestoreDraft,
    ];

    fn get(self, language: Language) -> &'static str {
        match (self, language) {
            (Self::DraftFound, Language::English) => "Unsaved changes from",
            (Self::DraftFound, Language::German) => "Ungespeicherte Änderungen vom",
            (Self::DraftNoteChangedSince, Language::English) => "The note was changed since.",
            (Self::DraftNoteChangedSince, Language::German) => "Die Notiz wurde seitdem geändert.",
            (Self::RestoreDraft, Language::English) => "Restore",
            (Self::RestoreDraft, Language::German) => "Wiederherstellen",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_text_is_translated_and_registered() {
        trailkit_memo_shared::i18n::assert_all_translated::<DraftText>();
    }
}
