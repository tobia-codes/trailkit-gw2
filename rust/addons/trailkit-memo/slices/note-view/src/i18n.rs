//! The note-view slice's user-facing strings in English and German
//! (ROADMAP R76: every slice owns its texts; nexus-addon-dev skill §1).

use trailkit_memo_shared::i18n::{Language, LocalizedText};

/// A translatable string of the note-view slice.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NoteViewText {
    /// Shown in the note pane when no note is selected.
    SelectNote,
    /// Shown instead of an empty note's content.
    EmptyNote,
}

impl LocalizedText for NoteViewText {
    const ALL: &'static [Self] = &[Self::SelectNote, Self::EmptyNote];

    fn get(self, language: Language) -> &'static str {
        match (self, language) {
            (Self::SelectNote, Language::English) => "Select a note in the list.",
            (Self::SelectNote, Language::German) => "Wähle eine Notiz in der Liste aus.",
            (Self::EmptyNote, Language::English) => "This note is empty.",
            (Self::EmptyNote, Language::German) => "Diese Notiz ist leer.",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_text_is_translated_and_registered() {
        trailkit_memo_shared::i18n::assert_all_translated::<NoteViewText>();
    }
}
