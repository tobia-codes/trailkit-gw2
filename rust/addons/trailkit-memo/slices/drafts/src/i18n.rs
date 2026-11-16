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
    /// Card above the panes with one note with unsaved changes (M185).
    OverviewOne,
    /// The same card for several notes; `{count}` is their number.
    OverviewMany,
    /// The card's button: show the next note with unsaved changes.
    GoToNote,
    /// Tooltip of the mark in the note list.
    UnsavedChangesMark,
}

impl LocalizedText for DraftText {
    const ALL: &'static [Self] = &[
        Self::DraftFound,
        Self::DraftNoteChangedSince,
        Self::RestoreDraft,
        Self::OverviewOne,
        Self::OverviewMany,
        Self::GoToNote,
        Self::UnsavedChangesMark,
    ];

    fn get(self, language: Language) -> &'static str {
        match (self, language) {
            (Self::DraftFound, Language::English) => "Unsaved changes from",
            (Self::DraftFound, Language::German) => "Ungespeicherte Änderungen vom",
            (Self::DraftNoteChangedSince, Language::English) => "The note was changed since.",
            (Self::DraftNoteChangedSince, Language::German) => "Die Notiz wurde seitdem geändert.",
            (Self::RestoreDraft, Language::English) => "Restore",
            (Self::RestoreDraft, Language::German) => "Wiederherstellen",
            (Self::OverviewOne, Language::English) => {
                "1 note has unsaved changes. It is marked with the same sign in the list."
            }
            (Self::OverviewOne, Language::German) => {
                "1 Notiz hat ungespeicherte Änderungen. Sie ist in der Liste mit demselben Zeichen markiert."
            }
            (Self::OverviewMany, Language::English) => {
                "{count} notes have unsaved changes. They are marked with the same sign in the list."
            }
            (Self::OverviewMany, Language::German) => {
                "{count} Notizen haben ungespeicherte Änderungen. Sie sind in der Liste mit demselben Zeichen markiert."
            }
            (Self::GoToNote, Language::English) => "Go to note",
            (Self::GoToNote, Language::German) => "Zur Notiz springen",
            (Self::UnsavedChangesMark, Language::English) => "Unsaved changes",
            (Self::UnsavedChangesMark, Language::German) => "Ungespeicherte Änderungen",
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
