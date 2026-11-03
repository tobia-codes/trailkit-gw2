//! The search slice's user-facing strings in English and German (ROADMAP
//! R76: every slice owns its texts; nexus-addon-dev skill §1).

use trailkit_memo_shared::i18n::Language;

/// A translatable string of the search slice.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SearchText {
    /// Placeholder of the search field (M123).
    SearchHint,
    /// Clear button of the search field and of the empty search result
    /// (M118, M126).
    ClearSearch,
    /// Empty search result; `{query}` is replaced by the search text (M126).
    NoSearchResults,
    /// Tooltip while dragging during a search, which cannot move (M125).
    ClearSearchToMove,
    /// Tooltip of a row's content hit count; `{count}` is replaced (M122).
    ContentHits,
}

impl SearchText {
    /// Every string of the slice.
    pub const ALL: [Self; 5] = [
        Self::SearchHint,
        Self::ClearSearch,
        Self::NoSearchResults,
        Self::ClearSearchToMove,
        Self::ContentHits,
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
            (Self::SearchHint, Language::English) => "Search …",
            (Self::SearchHint, Language::German) => "Suchen …",
            (Self::ClearSearch, Language::English) => "Clear search",
            (Self::ClearSearch, Language::German) => "Suche leeren",
            (Self::NoSearchResults, Language::English) => "No notes for “{query}”",
            (Self::NoSearchResults, Language::German) => "Keine Notizen für „{query}“",
            (Self::ClearSearchToMove, Language::English) => "Clear the search to move",
            (Self::ClearSearchToMove, Language::German) => "Suche leeren zum Verschieben",
            (Self::ContentHits, Language::English) => "Matches in the text: {count}",
            (Self::ContentHits, Language::German) => "Treffer im Text: {count}",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_text_is_translated_and_registered() {
        let all = SearchText::all_texts();
        for text in SearchText::ALL {
            for language in Language::ALL {
                assert!(!text.get(language).is_empty(), "{text:?} {language:?}");
                assert!(all.contains(text.get(language)));
            }
        }
    }
}
