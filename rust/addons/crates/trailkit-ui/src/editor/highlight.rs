//! Light markdown highlighting for the editor (memo plan M60): heading text
//! is bold, markdown markers (`#`, list bullets, task boxes, `**`, `*`,
//! backticks, link brackets and targets) are dimmed. Everything else is
//! plain text. Chat codes (`[&…]`) are chips (memo plan M75). Works line by
//! line and never fails: unknown syntax is text.

use std::ops::Range;

use crate::chat_link::find_chat_codes;

/// How a piece of text is drawn.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SpanStyle {
    /// Normal text.
    Text,
    /// A markdown marker, drawn in `text.secondary`.
    Marker,
    /// Heading text, drawn bold.
    Strong,
    /// A chat code, drawn as one chip; never merged with a neighbour.
    ChatLink,
}

/// A styled range of the text (byte offsets into the whole text).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StyledSpan {
    /// The range.
    pub range: Range<usize>,
    /// How it is drawn.
    pub style: SpanStyle,
}

/// Appends the styled spans of one logical `line` (without its line break)
/// that starts at byte `offset` of the text. Spans cover the whole line,
/// in order, and adjacent spans have different styles.
pub fn highlight_line(line: &str, offset: usize, spans: &mut Vec<StyledSpan>) {
    let first = spans.len();
    let mut push = |range: Range<usize>, style: SpanStyle| {
        if range.is_empty() {
            return;
        }
        let range = offset + range.start..offset + range.end;
        if let Some(last) = spans[first..].last_mut()
            && last.style == style
            && style != SpanStyle::ChatLink
            && last.range.end == range.start
        {
            last.range.end = range.end;
        } else {
            spans.push(StyledSpan { range, style });
        }
    };

    let prefix = line_prefix_length(line);
    push(0..prefix.length, SpanStyle::Marker);
    let base = if prefix.is_heading {
        SpanStyle::Strong
    } else {
        SpanStyle::Text
    };

    let bytes = line.as_bytes();
    let mut index = prefix.length;
    let mut text_start = index;
    let mut codes = find_chat_codes(line)
        .skip_while(|code| code.start < prefix.length)
        .peekable();
    while index < bytes.len() {
        // Codes swallowed by a marker (e.g. inside a link target) are text.
        while codes.next_if(|code| code.start < index).is_some() {}
        if let Some(code) = codes.next_if(|code| code.start == index) {
            push(text_start..index, base);
            push(code.clone(), SpanStyle::ChatLink);
            index = code.end;
            text_start = index;
            continue;
        }
        let marker_length = inline_marker_length(line, index);
        if marker_length > 0 {
            push(text_start..index, base);
            push(index..index + marker_length, SpanStyle::Marker);
            index += marker_length;
            text_start = index;
        } else {
            index += line[index..].chars().next().map_or(1, char::len_utf8);
        }
    }
    push(text_start..line.len(), base);
}

/// What a line starts with.
struct LinePrefix {
    /// Length of the marker prefix (heading hashes or list marker).
    length: usize,
    /// Whether the line is a heading.
    is_heading: bool,
}

/// Recognizes a heading (`#`…`######` plus a space) or a list marker
/// (`- `, `* `, `+ `, `N. `, optionally followed by a task box) at the start
/// of `line`, including indentation.
fn line_prefix_length(line: &str) -> LinePrefix {
    let indentation = line.len() - line.trim_start_matches([' ', '\t']).len();
    let rest = &line[indentation..];
    let hashes = rest.bytes().take_while(|byte| *byte == b'#').count();
    if (1..=6).contains(&hashes) && (rest.len() == hashes || rest[hashes..].starts_with(' ')) {
        let length = indentation + hashes + usize::from(rest.len() > hashes);
        return LinePrefix {
            length,
            is_heading: true,
        };
    }
    let bullet = if ["- ", "* ", "+ "]
        .iter()
        .any(|bullet| rest.starts_with(bullet))
    {
        2
    } else {
        let digits = rest.bytes().take_while(u8::is_ascii_digit).count();
        if digits > 0 && rest[digits..].starts_with(". ") {
            digits + 2
        } else {
            0
        }
    };
    if bullet == 0 {
        return LinePrefix {
            length: 0,
            is_heading: false,
        };
    }
    let task = ["[ ] ", "[x] ", "[X] "]
        .iter()
        .find(|task| rest[bullet..].starts_with(*task))
        .map_or(0, |task| task.len());
    LinePrefix {
        length: indentation + bullet + task,
        is_heading: false,
    }
}

/// Length of an inline marker starting at byte `index` of `line`, or 0:
/// `**`, `*`, a backtick, `[`, or a link target `](…)`.
fn inline_marker_length(line: &str, index: usize) -> usize {
    let rest = &line[index..];
    if rest.starts_with("**") {
        2
    } else if rest.starts_with('*') || rest.starts_with('`') || rest.starts_with('[') {
        1
    } else if rest.starts_with("](") {
        rest.find(')').map_or(2, |close| close + 1)
    } else {
        0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Highlights `line` and renders the spans as `style:text` pieces.
    fn render(line: &str) -> Vec<String> {
        let mut spans = Vec::new();
        highlight_line(line, 0, &mut spans);
        spans
            .iter()
            .map(|span| {
                let tag = match span.style {
                    SpanStyle::Text => "t",
                    SpanStyle::Marker => "m",
                    SpanStyle::Strong => "s",
                    SpanStyle::ChatLink => "c",
                };
                format!("{tag}:{}", &line[span.range.clone()])
            })
            .collect()
    }

    #[test]
    fn headings_are_strong_with_dimmed_hashes() {
        assert_eq!(render("## Raids"), ["m:## ", "s:Raids"]);
        assert_eq!(render("#hashtag"), ["t:#hashtag"]);
    }

    #[test]
    fn list_markers_and_task_boxes_are_dimmed() {
        assert_eq!(render("  - [x] done"), ["m:  - [x] ", "t:done"]);
        assert_eq!(render("12. step"), ["m:12. ", "t:step"]);
    }

    #[test]
    fn inline_markers_and_links_are_dimmed() {
        assert_eq!(
            render("a **b** [w](https://x) `c`"),
            [
                "t:a ",
                "m:**",
                "t:b",
                "m:**",
                "t: ",
                "m:[",
                "t:w",
                "m:](https://x)",
                "t: ",
                "m:`",
                "t:c",
                "m:`"
            ]
        );
    }

    #[test]
    fn spans_cover_the_line_with_offsets() {
        let mut spans = Vec::new();
        highlight_line("- ä", 10, &mut spans);
        assert_eq!(spans.first().map(|span| span.range.start), Some(10));
        assert_eq!(
            spans.last().map(|span| span.range.end),
            Some(10 + "- ä".len())
        );
    }

    #[test]
    fn chat_codes_are_separate_chip_spans() {
        assert_eq!(
            render("- go [&BDgAAAA=][&BDgAAAA=] now"),
            ["m:- ", "t:go ", "c:[&BDgAAAA=]", "c:[&BDgAAAA=]", "t: now"]
        );
    }
}
