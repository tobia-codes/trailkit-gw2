//! The note-list slice's user-facing strings in English and German
//! (ROADMAP R76: every slice owns its texts; nexus-addon-dev skill §1).
//! "New note" and "New group" stay in the shared texts: the app names new
//! entries with them.

use trailkit_memo_shared::i18n::{Language, LocalizedText};

/// A translatable string of the note-list slice.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NoteListText {
    /// Context menu entry to rename a note or group.
    Rename,
    /// Context menu submenu that moves a note or group (M112).
    MoveTo,
    /// "Move to" target: the root level of the note list (M112).
    TopLevel,
    /// Context menu entry that moves a note or group to the trash (M133).
    Delete,
    /// Tooltip of "Delete" on the note open in the editor (M141).
    SaveOrCancelFirst,
}

impl LocalizedText for NoteListText {
    const ALL: &'static [Self] = &[
        Self::Rename,
        Self::MoveTo,
        Self::TopLevel,
        Self::Delete,
        Self::SaveOrCancelFirst,
    ];

    fn get(self, language: Language) -> &'static str {
        match (self, language) {
            (Self::Rename, Language::English) => "Rename",
            (Self::Rename, Language::German) => "Umbenennen",
            (Self::MoveTo, Language::English) => "Move to",
            (Self::MoveTo, Language::German) => "Verschieben nach",
            (Self::TopLevel, Language::English) => "Top level",
            (Self::TopLevel, Language::German) => "Oberste Ebene",
            (Self::Delete, Language::English) => "Delete",
            (Self::Delete, Language::German) => "Löschen",
            (Self::SaveOrCancelFirst, Language::English) => "Save or cancel the edit first",
            (Self::SaveOrCancelFirst, Language::German) => {
                "Erst speichern oder die Bearbeitung abbrechen"
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_text_is_translated_and_registered() {
        trailkit_memo_shared::i18n::assert_all_translated::<NoteListText>();
    }
}
