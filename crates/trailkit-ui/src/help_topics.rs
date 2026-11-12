//! The topics of an addon's help text (ROADMAP R95): every `##` heading of
//! `help/<language>.md` starts a topic, `###` headings structure it, and a
//! level 1 heading before the first topic (the document title) is ignored.
//! The help window lists the topics on the left and shows the selected one,
//! rendered with the markdown view, on the right.

/// One topic of a help text.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HelpTopic {
    /// The `##` heading text, shown in the topic list.
    pub title: String,
    /// The topic's markdown below its heading, with the `###` headings in
    /// place; empty lines at both ends are removed.
    pub body: String,
}

/// Splits `markdown` into its topics, in order. Text before the first `##`
/// heading is dropped. Lines inside fenced code blocks are never headings.
#[must_use]
pub fn parse_help_topics(markdown: &str) -> Vec<HelpTopic> {
    let mut topics: Vec<(String, Vec<&str>)> = Vec::new();
    let mut in_fence = false;
    for line in markdown.lines() {
        if line.trim_start().starts_with("```") {
            in_fence = !in_fence;
        }
        if !in_fence && let Some(title) = topic_title(line) {
            topics.push((title.to_owned(), Vec::new()));
        } else if let Some((_, lines)) = topics.last_mut() {
            lines.push(line);
        }
    }
    topics
        .into_iter()
        .map(|(title, lines)| HelpTopic {
            title,
            body: lines.join("\n").trim_matches('\n').to_owned(),
        })
        .collect()
}

/// The text of a `## title` line; `None` for any other line, including
/// `###` headings.
fn topic_title(line: &str) -> Option<&str> {
    let title = line.strip_prefix("## ")?.trim();
    (!title.is_empty()).then_some(title)
}

/// How many `###` headings each topic of `markdown` has, in order: what the
/// language parity tests compare, since both languages must structure every
/// topic the same way.
#[must_use]
pub fn subsection_counts(markdown: &str) -> Vec<usize> {
    parse_help_topics(markdown)
        .iter()
        .map(|topic| {
            topic
                .body
                .lines()
                .filter(|line| line.starts_with("### "))
                .count()
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    const HELP: &str = "# Help\nIntro that is dropped.\n\n## Getting started\nOpen the window.\n\n### Keybind\nPress it.\n\n## Search\n```\n## not a heading\n```\nType.\n";

    #[test]
    fn every_level_two_heading_starts_a_topic() {
        let topics = parse_help_topics(HELP);
        let titles: Vec<_> = topics.iter().map(|topic| topic.title.as_str()).collect();
        assert_eq!(titles, ["Getting started", "Search"]);
    }

    #[test]
    fn level_three_headings_stay_inside_their_topic() {
        let topics = parse_help_topics(HELP);
        assert_eq!(topics[0].body, "Open the window.\n\n### Keybind\nPress it.");
    }

    #[test]
    fn text_before_the_first_topic_and_fenced_headings_are_not_topics() {
        let topics = parse_help_topics(HELP);
        assert!(!topics.iter().any(|topic| topic.body.contains("Intro")));
        assert!(topics[1].body.contains("## not a heading"));
    }

    #[test]
    fn subsections_are_counted_per_topic() {
        assert_eq!(subsection_counts(HELP), [1, 0]);
    }

    #[test]
    fn a_text_without_topics_has_none() {
        assert!(parse_help_topics("just text\n### only a section\n").is_empty());
    }
}
