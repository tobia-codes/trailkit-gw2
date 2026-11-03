//! The reset-schedule slice's user-facing strings in English and German
//! (ROADMAP R76: every slice owns its texts; nexus-addon-dev skill §1).

use trailkit_memo_shared::i18n::Language;

/// Words of [`reset_countdown`] in every language, for the font atlas.
const COUNTDOWN_WORDS: &str = "Resets in Reset in d T h min";

/// A translatable string of the reset-schedule slice.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ResetText {
    /// Letter on the badge of daily reset notes.
    DailyBadge,
    /// Letter on the badge of weekly reset notes.
    WeeklyBadge,
    /// Context menu submenu and dropdown tooltip for the note kind (M86).
    ResetType,
    /// Note kind option: no reset.
    KindNormal,
    /// Note kind option: daily reset.
    KindDaily,
    /// Note kind option: weekly reset.
    KindWeekly,
}

impl ResetText {
    /// Every string of the slice.
    pub const ALL: [Self; 6] = [
        Self::DailyBadge,
        Self::WeeklyBadge,
        Self::ResetType,
        Self::KindNormal,
        Self::KindDaily,
        Self::KindWeekly,
    ];

    /// All strings in all languages concatenated, so the app can register
    /// every character with Nexus' font atlas.
    #[must_use]
    pub fn all_texts() -> String {
        Self::ALL
            .iter()
            .flat_map(|text| Language::ALL.map(|language| text.get(language)))
            .chain([COUNTDOWN_WORDS])
            .collect()
    }

    /// Returns the translation of this string in `language`.
    #[must_use]
    pub const fn get(self, language: Language) -> &'static str {
        match (self, language) {
            (Self::DailyBadge, Language::English) => "D",
            (Self::DailyBadge, Language::German) => "T",
            (Self::WeeklyBadge, Language::English | Language::German) => "W",
            (Self::ResetType, Language::English) => "Reset type",
            (Self::ResetType, Language::German) => "Reset-Art",
            (Self::KindNormal, Language::English | Language::German) => "Normal",
            (Self::KindDaily, Language::English) => "Daily reset",
            (Self::KindDaily, Language::German) => "Täglicher Reset",
            (Self::KindWeekly, Language::English) => "Weekly reset",
            (Self::KindWeekly, Language::German) => "Wöchentlicher Reset",
        }
    }
}

/// The time until the next reset, in whole minutes rounded up (M91), e.g.
/// "Resets in 5 h 12 min", "Resets in 12 min", "Resets in 3 d 4 h";
/// German "Reset in …" with "T" for days.
#[must_use]
pub fn reset_countdown(minutes: i64, language: Language) -> String {
    let (prefix, day_unit) = match language {
        Language::English => ("Resets in", "d"),
        Language::German => ("Reset in", "T"),
    };
    let minutes = minutes.max(1);
    let (days, hours, minutes) = (minutes / 1_440, minutes % 1_440 / 60, minutes % 60);
    if days > 0 {
        format!("{prefix} {days} {day_unit} {hours} h")
    } else if hours > 0 {
        format!("{prefix} {hours} h {minutes} min")
    } else {
        format!("{prefix} {minutes} min")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn countdown_uses_the_largest_two_units() {
        assert_eq!(
            reset_countdown(312, Language::English),
            "Resets in 5 h 12 min"
        );
        assert_eq!(reset_countdown(12, Language::German), "Reset in 12 min");
        assert_eq!(reset_countdown(4_560, Language::German), "Reset in 3 T 4 h");
        assert_eq!(reset_countdown(0, Language::English), "Resets in 1 min");
    }

    #[test]
    fn every_text_is_translated_and_registered() {
        let all = ResetText::all_texts();
        for text in ResetText::ALL {
            for language in Language::ALL {
                assert!(!text.get(language).is_empty(), "{text:?} {language:?}");
                assert!(all.contains(text.get(language)));
            }
        }
    }
}
