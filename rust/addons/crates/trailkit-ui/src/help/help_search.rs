//! Searching a help window's topics (BL30, Q575–Q583, mock
//! https://claude.ai/artifact/DQRhxQtb4zyUxKB1hBtEVy v9): which topics and
//! sections the topic list shows, the hit counts at their rows, and where
//! each section's hits start, so a click on a section jumps to its first hit
//! and the arrows step through all hits.
//!
//! ImGui-free so it is tested on any platform. The index is built once per
//! help text, the results once per query.

use crate::{
    markdown::{Block, MarkdownDocument, plain_text},
    text::search_steps::HitStop,
    text::text_search::SearchQuery,
};

/// The heading level that starts a topic's section (`###`).
pub const SECTION_HEADING_LEVEL: u8 = 3;

/// The searchable text of one topic.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IndexedTopic {
    /// The title in lower case.
    title: String,
    /// The lead sentence in lower case: it makes the topic match, but its
    /// hits are not marked, because it is drawn as the page's subtitle.
    lead: String,
    /// The drawn text before the first section, then each section with its
    /// heading, in original case (hits are counted like the view marks
    /// them).
    parts: Vec<String>,
}

impl IndexedTopic {
    /// Indexes a topic from its `title`, `lead` and drawn `document`.
    #[must_use]
    pub fn new(title: &str, lead: Option<&str>, document: &MarkdownDocument) -> Self {
        let mut parts = vec![MarkdownDocument::default()];
        for block in &document.blocks {
            if matches!(block, Block::Heading { level, .. } if *level == SECTION_HEADING_LEVEL) {
                parts.push(MarkdownDocument::default());
            }
            if let Some(part) = parts.last_mut() {
                part.blocks.push(block.clone());
            }
        }
        Self {
            title: title.to_lowercase(),
            lead: lead.unwrap_or_default().to_lowercase(),
            parts: parts.iter().map(plain_text).collect(),
        }
    }
}

/// What a search shows of one topic.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct TopicResult {
    /// Whether the topic matches: every term occurs in its title, lead or
    /// text (like memo M119).
    pub matches: bool,
    /// The marked hits in the topic's text.
    pub hits: usize,
    /// The sections in order: whether each shows below the topic, and
    /// where its hits start among the topic's hits.
    pub sections: Vec<SectionResult>,
}

/// What a search shows of one section.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct SectionResult {
    /// Whether the row shows: it holds a hit, or the topic's title matches,
    /// which keeps all its sections (Q577 A).
    pub visible: bool,
    /// The section's hits.
    pub hits: usize,
    /// The index of its first hit among the topic's hits.
    pub first_hit: usize,
}

/// The results of one query over all topics.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct HelpSearchResults {
    /// One per topic, in order.
    pub topics: Vec<TopicResult>,
    /// The matching topics with their hits, for the arrows.
    stops: Vec<HitStop<usize>>,
}

impl HelpSearchResults {
    /// Searches `topics` for `query`.
    #[must_use]
    pub fn compute(topics: &[IndexedTopic], query: &SearchQuery) -> Self {
        let mut results = Self::default();
        for (index, topic) in topics.iter().enumerate() {
            let result = search_topic(topic, query);
            if result.matches {
                results.stops.push(HitStop {
                    page: index,
                    hits: result.hits.max(1),
                });
            }
            results.topics.push(result);
        }
        results
    }

    /// Whether any topic matches.
    #[must_use]
    pub fn has_hits(&self) -> bool {
        !self.stops.is_empty()
    }

    /// The matching topics in order, for stepping through the hits.
    #[must_use]
    pub fn hit_stops(&self) -> &[HitStop<usize>] {
        &self.stops
    }
}

/// Searches one topic.
fn search_topic(topic: &IndexedTopic, query: &SearchQuery) -> TopicResult {
    let matches = query.terms().iter().all(|term| {
        topic.title.contains(term.as_str())
            || topic.lead.contains(term.as_str())
            || topic
                .parts
                .iter()
                .any(|part| part.to_lowercase().contains(term.as_str()))
    });
    if !matches {
        return TopicResult {
            sections: vec![SectionResult::default(); topic.parts.len() - 1],
            ..TopicResult::default()
        };
    }
    let title_matches = query.matches_lowered(&topic.title);
    let part_hits: Vec<usize> = topic
        .parts
        .iter()
        .map(|part| query.hit_ranges(part).len())
        .collect();
    let mut first_hit = part_hits.first().copied().unwrap_or(0);
    let sections = part_hits
        .iter()
        .skip(1)
        .map(|&hits| {
            let section = SectionResult {
                visible: title_matches || hits > 0,
                hits,
                first_hit,
            };
            first_hit += hits;
            section
        })
        .collect();
    TopicResult {
        matches,
        hits: part_hits.iter().sum(),
        sections,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::markdown::parse;

    /// Indexes topics `(title, lead, text)`.
    fn index(topics: &[(&str, Option<&str>, &str)]) -> Vec<IndexedTopic> {
        topics
            .iter()
            .map(|(title, lead, text)| IndexedTopic::new(title, *lead, &parse(text)))
            .collect()
    }

    /// Runs `query` over `topics`.
    fn search(topics: &[IndexedTopic], query: &str) -> HelpSearchResults {
        HelpSearchResults::compute(topics, &SearchQuery::parse(query).expect("terms"))
    }

    /// Three topics: one with two sections, a privacy topic, and one whose
    /// word only appears in its lead.
    fn sample() -> Vec<IndexedTopic> {
        index(&[
            (
                "Trash and undo",
                None,
                "The trash keeps notes.\n\n### Undo\nCtrl+Z.\n\n### The trash\nOpen the trash, empty the trash.",
            ),
            (
                "Privacy",
                None,
                "Nothing leaves your PC.\n\n### Logs\nThe trash log.",
            ),
            ("Keys", Some("Every trash key."), "### Window\nShift twice."),
        ])
    }

    #[test]
    fn sections_with_hits_show_and_know_where_their_hits_start() {
        let results = search(&sample(), "trash");
        let trash = &results.topics[0];
        assert!(trash.matches);
        // "trash" in the intro once, the second section's heading and text.
        assert_eq!(trash.hits, 1 + 3);
        assert_eq!(
            trash.sections,
            vec![
                SectionResult {
                    visible: true,
                    hits: 0,
                    first_hit: 1
                },
                SectionResult {
                    visible: true,
                    hits: 3,
                    first_hit: 1
                },
            ],
            "the topic title matches, so every section shows"
        );
    }

    #[test]
    fn only_sections_with_hits_show_unless_the_title_matches() {
        let results = search(&sample(), "log");
        let privacy = &results.topics[1];
        assert!(privacy.matches, "the privacy topic is searched (Q579)");
        assert_eq!(
            privacy.sections,
            vec![SectionResult {
                visible: true,
                hits: 2,
                first_hit: 0
            }]
        );
        let results = search(&sample(), "undo");
        assert!(results.topics[0].sections[1].visible, "title match");
        let results = search(&sample(), "ctrl");
        assert!(!results.topics[0].sections[1].visible);
        assert!(results.topics[0].sections[0].visible);
    }

    #[test]
    fn a_lead_only_match_is_one_stop_without_marks() {
        let results = search(&sample(), "every");
        assert_eq!(results.topics[2].hits, 0);
        assert_eq!(results.hit_stops(), &[HitStop { page: 2, hits: 1 }]);
    }

    #[test]
    fn every_term_must_occur_somewhere_in_the_topic() {
        let results = search(&sample(), "trash shift");
        assert!(
            results.topics[2].matches,
            "trash in the lead, shift in the text"
        );
        assert!(!results.topics[1].matches);
        assert!(!search(&sample(), "zzz").has_hits());
    }
}
