//! Plain-text search shared by list filtering and hit highlighting (memo
//! plan M118–M120).
//!
//! A [`SearchQuery`] is split at whitespace into terms; a text matches when
//! it contains **every** term, ignoring case. Accents are not folded: "ä"
//! matches "Ä" but never "a" (M119). Terms never contain whitespace, so a
//! hit never spans a word boundary.

use std::ops::Range;

/// A parsed, non-empty search query: its terms in lower case.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SearchQuery {
    /// The lower-cased terms, in input order, without duplicates.
    terms: Vec<String>,
}

impl SearchQuery {
    /// Parses what the user typed; `None` if it holds no term (empty or only
    /// whitespace), i.e. there is nothing to search for.
    #[must_use]
    pub fn parse(input: &str) -> Option<Self> {
        let mut terms: Vec<String> = Vec::new();
        for term in input.split_whitespace().map(str::to_lowercase) {
            if !terms.contains(&term) {
                terms.push(term);
            }
        }
        (!terms.is_empty()).then_some(Self { terms })
    }

    /// The lower-cased terms.
    #[must_use]
    pub fn terms(&self) -> &[String] {
        &self.terms
    }

    /// Whether `lowered` (text already lower-cased with [`str::to_lowercase`])
    /// contains every term.
    #[must_use]
    pub fn matches_lowered(&self, lowered: &str) -> bool {
        self.terms
            .iter()
            .all(|term| lowered.contains(term.as_str()))
    }

    /// How many times any term occurs in `lowered` (already lower-cased),
    /// counting non-overlapping occurrences per term.
    #[must_use]
    pub fn count_in_lowered(&self, lowered: &str) -> usize {
        self.terms
            .iter()
            .map(|term| lowered.matches(term.as_str()).count())
            .sum()
    }

    /// Byte ranges of every term occurrence in `text` (original case),
    /// sorted and merged where they overlap or touch. Ranges always lie on
    /// character boundaries of `text`.
    #[must_use]
    pub fn hit_ranges(&self, text: &str) -> Vec<Range<usize>> {
        let mut ranges = Vec::new();
        self.collect_hit_ranges(text, &mut ranges);
        ranges
    }

    /// Like [`Self::hit_ranges`], but fills `ranges` (cleared first) so a
    /// caller drawing every frame can reuse one allocation.
    pub fn collect_hit_ranges(&self, text: &str, ranges: &mut Vec<Range<usize>>) {
        ranges.clear();
        if text.is_empty() {
            return;
        }
        // Lower-case per character and remember where each lowered byte came
        // from, so hits map back to `text` even when lower-casing changes a
        // character's length (e.g. "İ").
        let mut lowered = String::with_capacity(text.len());
        let mut origin = Vec::with_capacity(text.len() + 1);
        for (offset, character) in text.char_indices() {
            for lower in character.to_lowercase() {
                for _ in 0..lower.len_utf8() {
                    origin.push(offset);
                }
                lowered.push(lower);
            }
        }
        origin.push(text.len());
        for term in &self.terms {
            for (start, matched) in lowered.match_indices(term.as_str()) {
                let end = start + matched.len();
                // The end of the original character holding the hit's last
                // lowered byte (a hit may end inside a multi-character
                // lower-case expansion).
                let last_character = origin[end - 1];
                let original_end = origin[end..]
                    .iter()
                    .copied()
                    .find(|&offset| offset > last_character)
                    .unwrap_or(text.len());
                ranges.push(origin[start]..original_end);
            }
        }
        ranges.sort_by_key(|range| range.start);
        let mut merged: Vec<Range<usize>> = Vec::with_capacity(ranges.len());
        for range in ranges.drain(..) {
            match merged.last_mut() {
                Some(last) if range.start <= last.end => last.end = last.end.max(range.end),
                _ => merged.push(range),
            }
        }
        *ranges = merged;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Parses a query that is known to contain terms.
    fn query(input: &str) -> SearchQuery {
        SearchQuery::parse(input).expect("query has terms")
    }

    #[test]
    fn empty_or_blank_input_is_no_query() {
        assert_eq!(SearchQuery::parse(""), None);
        assert_eq!(SearchQuery::parse("   \t "), None);
    }

    #[test]
    fn terms_are_lower_cased_and_deduplicated() {
        assert_eq!(query("Mithril  ORE mithril").terms(), ["mithril", "ore"]);
    }

    #[test]
    fn every_term_must_occur() {
        let query = query("ore mith");
        assert!(query.matches_lowered("mithril ore farm"));
        assert!(!query.matches_lowered("mithril farm"));
    }

    #[test]
    fn umlauts_ignore_case_but_not_accents() {
        assert!(query("ÄRGER").matches_lowered(&"ärger".to_lowercase()));
        assert!(!query("bar").matches_lowered("bär"));
        assert!(!query("bär").matches_lowered("bar"));
    }

    #[test]
    fn counts_occurrences_of_all_terms() {
        assert_eq!(query("a b").count_in_lowered("a b a c"), 3);
    }

    #[test]
    fn hit_ranges_are_in_original_case_and_merged() {
        let query = query("ore re");
        assert_eq!(query.hit_ranges("Mithril Ore, more"), vec![8..11, 14..17]);
    }

    #[test]
    fn hit_ranges_survive_length_changing_lowercase() {
        // "İ" (2 bytes) lower-cases to "i̇" (3 bytes).
        let text = "İstanbul and istanbul";
        let ranges = query("stan").hit_ranges(text);
        assert_eq!(ranges.len(), 2);
        for range in ranges {
            assert_eq!(text[range].to_lowercase(), "stan");
        }
    }

    #[test]
    fn no_hits_give_no_ranges() {
        assert!(query("zzz").hit_ranges("abc").is_empty());
    }
}
