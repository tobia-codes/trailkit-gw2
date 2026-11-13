//! memo's help text (ROADMAP R95, memo M181): the topics of
//! `help/<language>.md`, whose last topic "Privacy" is completed with the
//! privacy policy `PRIVACY.md` / `PRIVACY.de.md` from the repository root,
//! so the help never drifts from the published policy (R99).

use std::sync::OnceLock;

use trailkit_memo_shared::i18n::Language;

/// The help text in `language`, built once and then shared; the help window
/// recognizes an unchanged text by its address.
pub fn help_document(language: Language) -> &'static str {
    static ENGLISH: OnceLock<String> = OnceLock::new();
    static GERMAN: OnceLock<String> = OnceLock::new();
    let (cell, help, privacy) = match language {
        Language::English => (
            &ENGLISH,
            include_str!("../help/en.md"),
            include_str!("../../../PRIVACY.md"),
        ),
        Language::German => (
            &GERMAN,
            include_str!("../help/de.md"),
            include_str!("../../../PRIVACY.de.md"),
        ),
    };
    cell.get_or_init(|| with_privacy_topic(help, privacy))
}

/// `help` with the body of `privacy` appended to its last topic: the policy's
/// `#` title is dropped, its `##` sections become `###` headings, and the
/// link to the other language on its date line is cut, since both languages
/// have their own help.
fn with_privacy_topic(help: &str, privacy: &str) -> String {
    let mut document = help.trim_end().to_owned();
    document.push_str("\n\n");
    for line in privacy
        .lines()
        .skip_while(|line| !line.starts_with("# "))
        .skip(1)
    {
        if let Some(section) = line.strip_prefix("## ") {
            document.push_str("### ");
            document.push_str(section);
        } else if let Some((date, _other_language)) = line.split_once(" · [") {
            document.push_str(date);
        } else {
            document.push_str(line);
        }
        document.push('\n');
    }
    document
}

#[cfg(test)]
mod tests {
    use trailkit_ui::help_topics::{parse_help_topics, subsection_counts};

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

    #[test]
    fn the_policy_sections_become_subsections() {
        let document = with_privacy_topic(
            "## Privacy\n\nLead.\n",
            "# Privacy\n\nLast updated: 2026-11-11 · [Deutsche Fassung](PRIVACY.de.md)\n\n## Data\n\nText.\n",
        );
        assert_eq!(
            document,
            "## Privacy\n\nLead.\n\n\nLast updated: 2026-11-11\n\n### Data\n\nText.\n"
        );
    }
}
