//! The trash slice's user-facing strings in English and German (ROADMAP
//! R76: every slice owns its texts; nexus-addon-dev skill §1). Placeholders
//! in braces are replaced by the formatting functions in `labels`.

use trailkit_memo_shared::i18n::{Language, LocalizedText};

/// A translatable string of the trash slice.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TrashText {
    /// Heading of the trash view and name of its tool (M136).
    Trash,
    /// Tooltip of the trash tool while the notes are shown.
    ShowTrash,
    /// Tooltip of the trash tool while the trash is shown.
    HideTrash,
    /// Button that empties the trash (asks first, M146).
    EmptyTrash,
    /// Empty state of the trash.
    TrashIsEmpty,
    /// Second line of the empty state (M12).
    TrashKeepsEntries,
    /// Hint in the note pane while no trash entry is selected.
    SelectTrashEntry,
    /// Puts an entry back (M140, M145).
    Restore,
    /// Tooltip and menu entry that deletes an entry for good (M145, M146).
    DeletePermanently,
    /// Days until an entry is purged; `{count}` is replaced (M150).
    DaysLeft,
    /// One day until an entry is purged (M150).
    OneDayLeft,
    /// Info line start of a trashed entry (M145).
    InTrash,
    /// Deleted within the last day.
    DeletedToday,
    /// Deleted one day ago.
    DeletedYesterday,
    /// Deleted `{count}` days ago.
    DeletedDaysAgo,
    /// Number of notes in a trashed group; `{count}` is replaced.
    NoteCount,
    /// One note in a trashed group.
    OneNote,
    /// Question of the permanent delete dialog; `{title}` is replaced.
    DeletePermanentlyTitle,
    /// Explanation of the permanent delete dialog for a note.
    DeleteNotePermanentlyText,
    /// Explanation of the permanent delete dialog for a group; `{count}`
    /// is replaced.
    DeleteGroupPermanentlyText,
    /// Question of the empty trash dialog.
    EmptyTrashTitle,
    /// Explanation of the empty trash dialog; `{entries}` and `{notes}`
    /// are replaced.
    EmptyTrashText,
    /// Question when deleting a group with notes (M134); `{title}` is
    /// replaced.
    DeleteGroupTitle,
    /// Explanation when deleting a group with notes; `{count}` is replaced.
    DeleteGroupText,
    /// Confirm button of the group delete dialog.
    Delete,
    /// Toast after deleting (M148); `{title}` is replaced.
    Deleted,
    /// Action of the toast and the undo step (M135).
    Undo,
}

impl LocalizedText for TrashText {
    const ALL: &'static [Self] = &[
        Self::Trash,
        Self::ShowTrash,
        Self::HideTrash,
        Self::EmptyTrash,
        Self::TrashIsEmpty,
        Self::TrashKeepsEntries,
        Self::SelectTrashEntry,
        Self::Restore,
        Self::DeletePermanently,
        Self::DaysLeft,
        Self::OneDayLeft,
        Self::InTrash,
        Self::DeletedToday,
        Self::DeletedYesterday,
        Self::DeletedDaysAgo,
        Self::NoteCount,
        Self::OneNote,
        Self::DeletePermanentlyTitle,
        Self::DeleteNotePermanentlyText,
        Self::DeleteGroupPermanentlyText,
        Self::EmptyTrashTitle,
        Self::EmptyTrashText,
        Self::DeleteGroupTitle,
        Self::DeleteGroupText,
        Self::Delete,
        Self::Deleted,
        Self::Undo,
    ];

    fn get(self, language: Language) -> &'static str {
        match language {
            Language::English => english(self),
            Language::German => german(self),
        }
    }
}

/// The English text of `text`.
const fn english(text: TrashText) -> &'static str {
    match text {
        TrashText::Trash => "Trash",
        TrashText::ShowTrash => "Show trash",
        TrashText::HideTrash => "Hide trash",
        TrashText::EmptyTrash => "Empty",
        TrashText::TrashIsEmpty => "The trash is empty.",
        TrashText::TrashKeepsEntries => "Deleted entries stay here for 30 days.",
        TrashText::SelectTrashEntry => "Select an entry to view it.",
        TrashText::Restore => "Restore",
        TrashText::DeletePermanently => "Delete permanently",
        TrashText::DaysLeft => "{count} days left",
        TrashText::OneDayLeft => "1 day left",
        TrashText::InTrash => "In the trash",
        TrashText::DeletedToday => "deleted today",
        TrashText::DeletedYesterday => "deleted yesterday",
        TrashText::DeletedDaysAgo => "deleted {count} days ago",
        TrashText::NoteCount => "{count} notes",
        TrashText::OneNote => "1 note",
        TrashText::DeletePermanentlyTitle => "Delete \u{201c}{title}\u{201d} permanently?",
        TrashText::DeleteNotePermanentlyText => {
            "The note will be deleted permanently. This cannot be undone."
        }
        TrashText::DeleteGroupPermanentlyText => {
            "The group and the {count} notes in it will be deleted permanently. This cannot be undone."
        }
        TrashText::EmptyTrashTitle => "Empty the trash?",
        TrashText::EmptyTrashText => {
            "{entries} entries ({notes} notes) will be deleted permanently. This cannot be undone."
        }
        TrashText::DeleteGroupTitle => "Delete \u{201c}{title}\u{201d}?",
        TrashText::DeleteGroupText => {
            "The group and the {count} notes in it move to the trash. You can restore them for 30 days."
        }
        TrashText::Delete => "Delete",
        TrashText::Deleted => "\u{201c}{title}\u{201d} deleted",
        TrashText::Undo => "Undo",
    }
}

/// The German text of `text`.
const fn german(text: TrashText) -> &'static str {
    match text {
        TrashText::Trash => "Papierkorb",
        TrashText::ShowTrash => "Papierkorb anzeigen",
        TrashText::HideTrash => "Papierkorb ausblenden",
        TrashText::EmptyTrash => "Leeren",
        TrashText::TrashIsEmpty => "Der Papierkorb ist leer.",
        TrashText::TrashKeepsEntries => "Gelöschtes bleibt hier 30 Tage.",
        TrashText::SelectTrashEntry => "Wähle einen Eintrag, um ihn anzusehen.",
        TrashText::Restore => "Wiederherstellen",
        TrashText::DeletePermanently => "Endgültig löschen",
        TrashText::DaysLeft => "noch {count} Tage",
        TrashText::OneDayLeft => "noch 1 Tag",
        TrashText::InTrash => "Im Papierkorb",
        TrashText::DeletedToday => "gelöscht heute",
        TrashText::DeletedYesterday => "gelöscht gestern",
        TrashText::DeletedDaysAgo => "gelöscht vor {count} Tagen",
        TrashText::NoteCount => "{count} Notizen",
        TrashText::OneNote => "1 Notiz",
        TrashText::DeletePermanentlyTitle => "\u{201e}{title}\u{201c} endgültig löschen?",
        TrashText::DeleteNotePermanentlyText => {
            "Die Notiz wird endgültig gelöscht. Das lässt sich nicht rückgängig machen."
        }
        TrashText::DeleteGroupPermanentlyText => {
            "Die Gruppe und die {count} Notizen darin werden endgültig gelöscht. Das lässt sich nicht rückgängig machen."
        }
        TrashText::EmptyTrashTitle => "Papierkorb leeren?",
        TrashText::EmptyTrashText => {
            "{entries} Einträge ({notes} Notizen) werden endgültig gelöscht. Das lässt sich nicht rückgängig machen."
        }
        TrashText::DeleteGroupTitle => "\u{201e}{title}\u{201c} löschen?",
        TrashText::DeleteGroupText => {
            "Die Gruppe und die {count} Notizen darin kommen in den Papierkorb. Du kannst sie 30 Tage lang wiederherstellen."
        }
        TrashText::Delete => "Löschen",
        TrashText::Deleted => "\u{201e}{title}\u{201c} gelöscht",
        TrashText::Undo => "Rückgängig",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_text_is_translated_and_registered() {
        trailkit_memo_shared::i18n::assert_all_translated::<TrashText>();
    }
}
