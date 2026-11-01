//! Markdown → a small layout model for the read-only markdown view (memo plan
//! M3, M19, M37, M38).
//!
//! Supported (M3): headings, paragraphs, bold, italic, bullet and numbered
//! lists, task list checkboxes, links and horizontal rules. Everything else
//! (code, quotes, images, HTML) is shown as plain text. Parsing happens once
//! per content change, never per frame; the Windows-only `markdown_view`
//! draws the result.

use pulldown_cmark::{Event, HeadingLevel, Options, Parser, Tag, TagEnd};

/// A parsed markdown document, ready to draw.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct MarkdownDocument {
    /// The blocks in display order.
    pub blocks: Vec<Block>,
}

/// One block-level element.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Block {
    /// A heading; `level` 1 is the largest.
    Heading {
        /// Heading level 1–6.
        level: u8,
        /// The heading text.
        spans: Vec<Span>,
    },
    /// A paragraph, indented by `depth` list levels (0 = not in a list).
    Paragraph {
        /// List nesting depth the paragraph belongs to.
        depth: u8,
        /// The paragraph text.
        spans: Vec<Span>,
    },
    /// A list item at nesting `depth` (1 = top-level list).
    ListItem {
        /// List nesting depth, starting at 1.
        depth: u8,
        /// Bullet, number or task checkbox.
        marker: ListMarker,
        /// The item text.
        spans: Vec<Span>,
    },
    /// A horizontal rule.
    Rule,
}

/// What precedes a list item.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ListMarker {
    /// A bullet point.
    Bullet,
    /// A number in an ordered list.
    Number(u64),
    /// A task checkbox.
    Task {
        /// Whether the box is checked.
        checked: bool,
        /// Where the `[ ]` / `[x]` is in the source, for toggling.
        marker: TaskMarker,
    },
}

/// Position of a task checkbox (`[ ]` or `[x]`) in the markdown source.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TaskMarker {
    /// Byte offset of the opening `[`.
    byte_offset: usize,
}

/// A run of text with one style.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Span {
    /// The text; `"\n"` alone is a hard line break.
    pub text: String,
    /// How the text is drawn.
    pub style: SpanStyle,
}

/// Inline style of a [`Span`].
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct SpanStyle {
    /// Bold text.
    pub bold: bool,
    /// Italic text.
    pub italic: bool,
    /// Target of a clickable link; only `http(s)` links are kept (M37).
    pub link: Option<String>,
}

impl Span {
    /// Whether this span is a hard line break.
    #[must_use]
    pub fn is_line_break(&self) -> bool {
        self.text == "\n"
    }
}

/// Parses `source` into a [`MarkdownDocument`].
#[must_use]
pub fn parse(source: &str) -> MarkdownDocument {
    let mut builder = Builder::default();
    let parser = Parser::new_ext(source, Options::ENABLE_TASKLISTS).into_offset_iter();
    for (event, range) in parser {
        builder.handle(event, range.start);
    }
    builder.finish()
}

/// Returns `source` with the task checkbox at `marker` toggled, or `None` if
/// the source no longer has a checkbox there (it changed since parsing).
#[must_use]
pub fn toggle_task(source: &str, marker: TaskMarker) -> Option<String> {
    let offset = marker.byte_offset;
    let bytes = source.as_bytes();
    let (open, state, close) = (
        *bytes.get(offset)?,
        *bytes.get(offset + 1)?,
        *bytes.get(offset + 2)?,
    );
    if open != b'[' || close != b']' {
        return None;
    }
    let replacement = match state {
        b' ' => "x",
        b'x' | b'X' => " ",
        _ => return None,
    };
    let mut toggled = source.to_owned();
    toggled.replace_range(offset + 1..offset + 2, replacement);
    Some(toggled)
}

/// Whether `url` may be opened in the browser (M37: `http(s)` only).
#[must_use]
pub fn is_web_link(url: &str) -> bool {
    let lower = url.trim_start().to_ascii_lowercase();
    lower.starts_with("https://") || lower.starts_with("http://")
}

/// Collects pulldown-cmark events into blocks.
#[derive(Default)]
struct Builder {
    /// Finished blocks.
    blocks: Vec<Block>,
    /// Spans of the block being built.
    spans: Vec<Span>,
    /// What the current block becomes when it ends.
    pending: Option<PendingBlock>,
    /// Open lists, innermost last: `None` = bullet list, `Some(n)` = next number.
    lists: Vec<Option<u64>>,
    /// Depth of bold nesting.
    bold: u32,
    /// Depth of italic nesting.
    italic: u32,
    /// Target of the link being read, if it is a web link.
    link: Option<String>,
}

/// The kind of block currently being collected.
#[derive(Debug, Clone, Copy)]
enum PendingBlock {
    /// A heading of this level.
    Heading(u8),
    /// A paragraph or other text outside list items.
    Paragraph,
    /// A list item with this marker.
    ListItem(ListMarker),
}

impl Builder {
    /// Processes one parser event that started at byte `offset`.
    fn handle(&mut self, event: Event<'_>, offset: usize) {
        match event {
            Event::Start(tag) => self.start(tag),
            Event::End(tag) => self.end(tag),
            Event::Text(text) | Event::Code(text) | Event::Html(text) | Event::InlineHtml(text) => {
                self.push_text(&text);
            }
            Event::SoftBreak => self.push_text(" "),
            Event::HardBreak => self.push_text("\n"),
            Event::Rule => {
                self.flush();
                self.blocks.push(Block::Rule);
            }
            Event::TaskListMarker(checked) => {
                if let Some(PendingBlock::ListItem(marker)) = &mut self.pending {
                    *marker = ListMarker::Task {
                        checked,
                        marker: TaskMarker {
                            byte_offset: offset,
                        },
                    };
                }
            }
            Event::FootnoteReference(text) | Event::InlineMath(text) | Event::DisplayMath(text) => {
                self.push_text(&text)
            }
        }
    }

    /// Handles the start of a tag.
    fn start(&mut self, tag: Tag<'_>) {
        match tag {
            Tag::Heading { level, .. } => {
                self.flush();
                self.pending = Some(PendingBlock::Heading(heading_level(level)));
            }
            Tag::Paragraph | Tag::CodeBlock(_) | Tag::BlockQuote(_) | Tag::HtmlBlock => {
                // Paragraphs inside a list item continue that item's text.
                if matches!(self.pending, Some(PendingBlock::ListItem(_))) {
                    if !self.spans.is_empty() {
                        self.push_text("\n");
                    }
                } else {
                    self.flush();
                    self.pending = Some(PendingBlock::Paragraph);
                }
            }
            Tag::List(first_number) => {
                // A nested list ends the text of the enclosing item.
                self.flush();
                self.lists.push(first_number);
            }
            Tag::Item => {
                self.flush();
                let marker = match self.lists.last_mut() {
                    Some(Some(number)) => {
                        let marker = ListMarker::Number(*number);
                        *number += 1;
                        marker
                    }
                    _ => ListMarker::Bullet,
                };
                self.pending = Some(PendingBlock::ListItem(marker));
            }
            Tag::Emphasis => self.italic += 1,
            Tag::Strong => self.bold += 1,
            Tag::Link { dest_url, .. } => {
                self.link = is_web_link(&dest_url).then(|| dest_url.into_string());
            }
            _ => {}
        }
    }

    /// Handles the end of a tag.
    fn end(&mut self, tag: TagEnd) {
        match tag {
            TagEnd::Heading(_) | TagEnd::Item => self.flush(),
            TagEnd::Paragraph | TagEnd::CodeBlock | TagEnd::BlockQuote(_) | TagEnd::HtmlBlock => {
                if !matches!(self.pending, Some(PendingBlock::ListItem(_))) {
                    self.flush();
                }
            }
            TagEnd::List(_) => {
                self.flush();
                self.lists.pop();
            }
            TagEnd::Emphasis => self.italic = self.italic.saturating_sub(1),
            TagEnd::Strong => self.bold = self.bold.saturating_sub(1),
            TagEnd::Link => self.link = None,
            _ => {}
        }
    }

    /// Appends text in the current style, merging with the previous span if
    /// the style is the same.
    fn push_text(&mut self, text: &str) {
        if self.pending.is_none() {
            // Text after a nested list inside an item continues as a paragraph.
            self.pending = Some(PendingBlock::Paragraph);
        }
        let style = SpanStyle {
            bold: self.bold > 0,
            italic: self.italic > 0,
            link: self.link.clone(),
        };
        let is_break = text == "\n";
        if let Some(last) = self.spans.last_mut()
            && last.style == style
            && !is_break
            && !last.is_line_break()
        {
            last.text.push_str(text);
            return;
        }
        self.spans.push(Span {
            text: text.to_owned(),
            style,
        });
    }

    /// Ends the current block, if any, and stores it.
    fn flush(&mut self) {
        let Some(pending) = self.pending.take() else {
            return;
        };
        let spans = std::mem::take(&mut self.spans);
        let depth = u8::try_from(self.lists.len()).unwrap_or(u8::MAX);
        let block = match pending {
            PendingBlock::Heading(level) => Block::Heading { level, spans },
            PendingBlock::Paragraph => {
                if spans.is_empty() {
                    return;
                }
                Block::Paragraph { depth, spans }
            }
            PendingBlock::ListItem(marker) => Block::ListItem {
                depth,
                marker,
                spans,
            },
        };
        self.blocks.push(block);
    }

    /// Finishes parsing and returns the document.
    fn finish(mut self) -> MarkdownDocument {
        self.flush();
        MarkdownDocument {
            blocks: self.blocks,
        }
    }
}

/// Converts a pulldown-cmark heading level to 1–6.
const fn heading_level(level: HeadingLevel) -> u8 {
    match level {
        HeadingLevel::H1 => 1,
        HeadingLevel::H2 => 2,
        HeadingLevel::H3 => 3,
        HeadingLevel::H4 => 4,
        HeadingLevel::H5 => 5,
        HeadingLevel::H6 => 6,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A plain span.
    fn plain(text: &str) -> Span {
        Span {
            text: text.to_owned(),
            style: SpanStyle::default(),
        }
    }

    #[test]
    fn headings_paragraphs_and_rules() {
        let document = parse("# Title\n\nSome text\nmore\n\n---\n");
        assert_eq!(
            document.blocks,
            vec![
                Block::Heading {
                    level: 1,
                    spans: vec![plain("Title")]
                },
                Block::Paragraph {
                    depth: 0,
                    spans: vec![plain("Some text more")]
                },
                Block::Rule,
            ]
        );
    }

    #[test]
    fn bold_italic_and_links() {
        let document = parse("a **b** *c* ***d*** [w](https://wiki.guildwars2.com) [f](file:///c)");
        let Some(Block::Paragraph { spans, .. }) = document.blocks.first() else {
            panic!("expected a paragraph: {document:?}");
        };
        let bold = spans.iter().find(|span| span.text == "b");
        assert!(bold.is_some_and(|span| span.style.bold && !span.style.italic));
        let both = spans.iter().find(|span| span.text == "d");
        assert!(both.is_some_and(|span| span.style.bold && span.style.italic));
        let web = spans.iter().find(|span| span.text == "w");
        assert_eq!(
            web.and_then(|span| span.style.link.as_deref()),
            Some("https://wiki.guildwars2.com")
        );
        // The non-web link is plain text, merged with the space before it.
        let file = spans.iter().find(|span| span.text.ends_with('f'));
        assert!(file.is_some_and(|span| span.style.link.is_none()));
        assert_eq!(
            spans
                .iter()
                .filter(|span| span.style.link.is_some())
                .count(),
            1
        );
    }

    #[test]
    fn nested_and_numbered_lists() {
        let document = parse("- one\n  - inner\n- two\n\n3. three\n4. four\n");
        let markers: Vec<(u8, ListMarker)> = document
            .blocks
            .iter()
            .filter_map(|block| match block {
                Block::ListItem { depth, marker, .. } => Some((*depth, *marker)),
                _ => None,
            })
            .collect();
        assert_eq!(
            markers,
            vec![
                (1, ListMarker::Bullet),
                (2, ListMarker::Bullet),
                (1, ListMarker::Bullet),
                (1, ListMarker::Number(3)),
                (1, ListMarker::Number(4)),
            ]
        );
    }

    #[test]
    fn task_checkboxes_toggle_in_the_source() {
        let source = "- [ ] daily fractals\n- [x] wvw\n";
        let tasks: Vec<(bool, TaskMarker)> = parse(source)
            .blocks
            .into_iter()
            .filter_map(|block| match block {
                Block::ListItem {
                    marker: ListMarker::Task { checked, marker },
                    ..
                } => Some((checked, marker)),
                _ => None,
            })
            .collect();
        assert_eq!(tasks.len(), 2);
        assert!(!tasks[0].0 && tasks[1].0);

        let once = toggle_task(source, tasks[0].1);
        assert_eq!(once.as_deref(), Some("- [x] daily fractals\n- [x] wvw\n"));
        let twice = toggle_task(source, tasks[1].1);
        assert_eq!(twice.as_deref(), Some("- [ ] daily fractals\n- [ ] wvw\n"));
    }

    #[test]
    fn stale_task_marker_is_rejected() {
        let source = "- [ ] task\n";
        let Some(Block::ListItem {
            marker: ListMarker::Task { marker, .. },
            ..
        }) = parse(source).blocks.into_iter().next()
        else {
            panic!("expected a task");
        };
        assert_eq!(toggle_task("changed", marker), None);
    }

    #[test]
    fn hard_breaks_are_separate_spans() {
        let document = parse("line one  \nline two");
        let Some(Block::Paragraph { spans, .. }) = document.blocks.first() else {
            panic!("expected a paragraph");
        };
        assert_eq!(spans.iter().filter(|span| span.is_line_break()).count(), 1);
    }

    #[test]
    fn web_link_detection() {
        assert!(is_web_link("https://example.com"));
        assert!(is_web_link("HTTP://example.com"));
        assert!(!is_web_link("javascript:alert(1)"));
        assert!(!is_web_link("file:///C:/x"));
    }
}
