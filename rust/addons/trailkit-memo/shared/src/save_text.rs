//! The user-facing strings of save errors in English and German (memo
//! M180, Q165, mockup V4; nexus-addon-dev skill §1).

use crate::{
    i18n::{Language, LocalizedText},
    storage_worker::MAX_ATTEMPTS,
};

/// A translatable string of save errors.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SaveText {
    /// The error toast and the title of the dialog when leaving (Q165).
    SaveFailed,
    /// The toast's button after every automatic write failed.
    TryAgain,
    /// A button while its write runs.
    Saving,
    /// The toast after a shown error was resolved (Q160).
    ChangesSaved,
    /// Tooltip of the toast's close button (Q164).
    Close,
    /// Text of the dialog when leaving the editor after a failed save (Q157).
    DiscardDeletesChanges,
    /// The dialog's primary button.
    SaveAgain,
    /// The dialog's status line after "Save again" failed (Q157).
    FailedAgain,
    /// Why the dialog's other answers wait while "Save again" runs (R93).
    AvailableAfterSaving,
}

impl LocalizedText for SaveText {
    const ALL: &'static [Self] = &[
        Self::SaveFailed,
        Self::TryAgain,
        Self::Saving,
        Self::ChangesSaved,
        Self::Close,
        Self::DiscardDeletesChanges,
        Self::SaveAgain,
        Self::FailedAgain,
        Self::AvailableAfterSaving,
    ];

    fn get(self, language: Language) -> &'static str {
        match (self, language) {
            (Self::SaveFailed, Language::English) => "Saving failed",
            (Self::SaveFailed, Language::German) => "Speichern fehlgeschlagen",
            (Self::TryAgain, Language::English) => "Try again",
            (Self::TryAgain, Language::German) => "Erneut versuchen",
            (Self::Saving, Language::English) => "Saving…",
            (Self::Saving, Language::German) => "Speichert…",
            (Self::ChangesSaved, Language::English) => "Changes saved",
            (Self::ChangesSaved, Language::German) => "Änderungen gespeichert",
            (Self::Close, Language::English) => "Close",
            (Self::Close, Language::German) => "Schließen",
            (Self::DiscardDeletesChanges, Language::English) => "Discarding deletes your changes.",
            (Self::DiscardDeletesChanges, Language::German) => "Verwerfen löscht deine Änderungen.",
            (Self::SaveAgain, Language::English) => "Save again",
            (Self::SaveAgain, Language::German) => "Erneut speichern",
            (Self::FailedAgain, Language::English) => "Failed again",
            (Self::FailedAgain, Language::German) => "Erneut fehlgeschlagen",
            (Self::AvailableAfterSaving, Language::English) => "Available once saving has finished",
            (Self::AvailableAfterSaving, Language::German) => {
                "Möglich, sobald das Speichern fertig ist"
            }
        }
    }

    /// The characters of the attempt counter ("2/3").
    fn extra_characters() -> String {
        ATTEMPT_COUNTERS.concat()
    }
}

/// The attempt counters after one and two failed writes; the third failure
/// ends the cycle and shows a button instead (Q149).
const ATTEMPT_COUNTERS: [&str; MAX_ATTEMPTS as usize - 1] = ["1/3", "2/3"];

/// The counter shown after `failed_attempts` failed writes of a cycle
/// ("2/3"), or `None` while none failed yet or after the last one.
#[must_use]
pub fn attempt_counter(failed_attempts: u8) -> Option<&'static str> {
    ATTEMPT_COUNTERS
        .get(usize::from(failed_attempts).checked_sub(1)?)
        .copied()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::i18n::assert_all_translated;

    #[test]
    fn every_text_is_translated_and_registered() {
        assert_all_translated::<SaveText>();
    }

    #[test]
    fn the_counter_names_the_failed_attempts_of_three() {
        assert_eq!(attempt_counter(0), None);
        assert_eq!(attempt_counter(1), Some("1/3"));
        assert_eq!(attempt_counter(2), Some("2/3"));
        assert_eq!(attempt_counter(3), None);
        assert_eq!(MAX_ATTEMPTS, 3, "the counters spell out three attempts");
    }
}
