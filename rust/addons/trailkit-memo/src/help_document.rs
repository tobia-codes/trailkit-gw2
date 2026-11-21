//! memo's help text (ROADMAP R95, memo M181): the build script assembles it
//! from one file per topic in `help/<language>/`, the last topic completed
//! with the privacy policy `PRIVACY.md` / `PRIVACY.de.md` from the repository
//! root, so the help never drifts from the published policy (R99, R157).

use trailkit_memo_shared::i18n::Language;

/// The help text in `language`, embedded at build time; the help window
/// recognizes an unchanged text by its address.
pub fn help_document(language: Language) -> &'static str {
    match language {
        Language::English => include_str!(concat!(env!("OUT_DIR"), "/help-en.md")),
        Language::German => include_str!(concat!(env!("OUT_DIR"), "/help-de.md")),
    }
}

#[cfg(test)]
mod tests {
    use trailkit_ui::help::help_topics::{parse_help_topics, subsection_counts};

    use super::*;

    #[test]
    fn both_languages_have_the_same_topics_and_structure() {
        let english = help_document(Language::English);
        let german = help_document(Language::German);
        assert_eq!(parse_help_topics(english).len(), 9);
        assert_eq!(subsection_counts(english), subsection_counts(german));
    }

    #[test]
    fn every_topic_opens_with_one_lead_sentence() {
        for language in Language::ALL {
            for topic in parse_help_topics(help_document(language)) {
                let (lead, _) = topic.split_lead();
                let lead = lead.unwrap_or_default();
                assert_eq!(
                    lead.matches(". ").count(),
                    0,
                    "{}: lead is not one sentence: {lead}",
                    topic.title
                );
                assert!(lead.ends_with('.'), "{}: no lead sentence", topic.title);
            }
        }
    }

    #[test]
    fn the_last_topic_holds_the_privacy_policy() {
        for language in Language::ALL {
            let topics = parse_help_topics(help_document(language));
            let privacy = topics.last().map(|topic| topic.body.as_str());
            let body = privacy.unwrap_or_default();
            assert!(body.contains("### "), "{language:?}: no policy sections");
            assert!(
                !body.contains("PRIVACY"),
                "{language:?}: link to the other language"
            );
            assert!(!body.lines().any(|line| line.starts_with("# ")));
        }
    }
}
