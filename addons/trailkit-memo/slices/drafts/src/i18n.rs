//! The drafts slice's user-facing strings in English and German (ROADMAP
//! R76: every slice owns its texts; nexus-addon-dev skill §1).

use trailkit_memo_shared::i18n::Language;

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

impl DraftText {
    /// Every string of the slice.
    pub const ALL: [Self; 3] = [
        Self::DraftFound,
        Self::DraftNoteChangedSince,
        Self::RestoreDraft,
    ];

    /// All strings in all languages concatenated, so the app can register
    /// every character with Nexus' font atlas.
    #[must_use]
    pub fn all_texts() -> String {
        Self::ALL
            .iter()
            .flat_map(|text| Language::ALL.map(|language| text.get(language)))
            .collect()
    }

    /// Returns the translation of this string in `language`.
    #[must_use]
    pub const fn get(self, language: Language) -> &'static str {
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
        let all = DraftText::all_texts();
        for text in DraftText::ALL {
            for language in Language::ALL {
                assert!(!text.get(language).is_empty(), "{text:?} {language:?}");
                assert!(all.contains(text.get(language)));
            }
        }
    }
}
