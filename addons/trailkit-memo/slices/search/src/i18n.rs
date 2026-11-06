//! The search slice's user-facing strings in English and German (ROADMAP
//! R76: every slice owns its texts; nexus-addon-dev skill §1).

use trailkit_memo_shared::i18n::{Language, LocalizedText};

/// A translatable string of the search slice.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SearchText {
    /// Placeholder of the search field (M123).
    SearchHint,
    /// Button of the empty search result (M126).
    ClearSearch,
    /// Tooltip of the `x` that closes the search field (M169).
    CloseSearch,
    /// Empty search result; `{query}` is replaced by the search text (M126).
    NoSearchResults,
    /// Tooltip while dragging during a search, which cannot move (M125).
    ClearSearchToMove,
    /// Tooltip of a row's content hit count; `{count}` is replaced (M122).
    ContentHits,
}

impl LocalizedText for SearchText {
    const ALL: &'static [Self] = &[
        Self::SearchHint,
        Self::ClearSearch,
        Self::CloseSearch,
        Self::NoSearchResults,
        Self::ClearSearchToMove,
        Self::ContentHits,
    ];

    fn get(self, language: Language) -> &'static str {
        match (self, language) {
            (Self::SearchHint, Language::English) => "Search …",
            (Self::SearchHint, Language::German) => "Suchen …",
            (Self::ClearSearch, Language::English) => "Clear search",
            (Self::ClearSearch, Language::German) => "Suche leeren",
            (Self::CloseSearch, Language::English) => "Close search",
            (Self::CloseSearch, Language::German) => "Suche schließen",
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
        trailkit_memo_shared::i18n::assert_all_translated::<SearchText>();
    }
}
