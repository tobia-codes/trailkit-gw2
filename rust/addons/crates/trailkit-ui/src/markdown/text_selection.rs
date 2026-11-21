//! Text selection in a rendered [`MarkdownDocument`] (memo plan M217): where
//! a selection starts and ends, and the text it copies (Q554). Independent of
//! ImGui, so every rule is testable on any host; the Windows-only
//! `markdown_view` draws the selection and moves it with the mouse.

use std::ops::Range;

use super::{Block, ListMarker, MarkdownDocument, Span};

/// A place between two characters of a document: before byte `offset` of the
/// text of span `span` in block `block`. A chat-link chip has only the places
/// before (0) and after it (its code's length). Positions order as they read.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, PartialOrd, Ord)]
pub struct TextPosition {
    /// Index of the block in the document.
    pub block: usize,
    /// Index of the span in the block.
    pub span: usize,
    /// Byte offset in the span's text, on a character boundary.
    pub offset: usize,
}

impl TextPosition {
    /// The position before byte `offset` of span `span` in block `block`.
    #[must_use]
    pub const fn new(block: usize, span: usize, offset: usize) -> Self {
        Self {
            block,
            span,
            offset,
        }
    }

    /// The start of block `block`.
    #[must_use]
    pub const fn block_start(block: usize) -> Self {
        Self::new(block, 0, 0)
    }

    /// The end of block `block` of `document`: after its last span.
    #[must_use]
    pub fn block_end(document: &MarkdownDocument, block: usize) -> Self {
        let spans = document.blocks.get(block).map_or(&[][..], block_spans);
        spans.last().map_or_else(
            || Self::block_start(block),
            |last| Self::new(block, spans.len() - 1, last.text.len()),
        )
    }

    /// The nearest valid position of `document`: inside its blocks and
    /// spans, on a character boundary, before or after a chip.
    #[must_use]
    pub fn clamped(self, document: &MarkdownDocument) -> Self {
        let Some(last_block) = document.blocks.len().checked_sub(1) else {
            return Self::default();
        };
        if self.block > last_block {
            return Self::block_end(document, last_block);
        }
        let spans = block_spans(&document.blocks[self.block]);
        let Some(span) = spans.get(self.span) else {
            return Self::block_end(document, self.block);
        };
        let mut offset = self.offset.min(span.text.len());
        if span.chip.is_some() && offset > 0 {
            offset = span.text.len();
        }
        while !span.text.is_char_boundary(offset) {
            offset -= 1;
        }
        Self { offset, ..self }
    }
}

/// The spans of `block`; a rule has none.
#[must_use]
pub fn block_spans(block: &Block) -> &[Span] {
    match block {
        Block::Heading { spans, .. }
        | Block::Paragraph { spans, .. }
        | Block::ListItem { spans, .. } => spans,
        Block::Rule => &[],
    }
}

/// What the left mouse button is doing with a selection.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum SelectionGesture {
    /// Nothing; the button is up or was pressed elsewhere.
    #[default]
    Idle,
    /// Pressed in the text, not yet moved past the drag threshold: a release
    /// now is a click (Q551).
    Pressed,
    /// Moved past the drag threshold while held: the focus follows the mouse
    /// and the release is no click.
    Dragging,
}

/// A selection in one document that outlives a frame: where it was started
/// (`anchor`) and where it ends now (`focus`), in either order. Empty when
/// both are equal.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct TextSelection {
    /// Where the selection was started.
    pub anchor: TextPosition,
    /// Where the selection ends now; Shift+click and dragging move it.
    pub focus: TextPosition,
    /// What the mouse is doing with it.
    pub gesture: SelectionGesture,
}

impl TextSelection {
    /// The selected range from its first to its last position, `None` while
    /// nothing is selected.
    #[must_use]
    pub fn range(&self) -> Option<Range<TextPosition>> {
        let (start, end) = if self.anchor <= self.focus {
            (self.anchor, self.focus)
        } else {
            (self.focus, self.anchor)
        };
        (start < end).then_some(start..end)
    }

    /// Whether some text is selected.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.range().is_none()
    }

    /// Selects nothing and forgets a gesture in progress (Q553).
    pub fn clear(&mut self) {
        *self = Self::default();
    }

    /// Selects the whole `document` (Ctrl+A, Q555).
    pub fn select_all(&mut self, document: &MarkdownDocument) {
        let end = document
            .blocks
            .len()
            .checked_sub(1)
            .map_or_else(TextPosition::default, |last| {
                TextPosition::block_end(document, last)
            });
        *self = Self {
            anchor: TextPosition::default(),
            focus: end,
            gesture: SelectionGesture::Idle,
        };
    }

    /// Keeps both ends valid after `document` changed.
    pub fn clamp(&mut self, document: &MarkdownDocument) {
        self.anchor = self.anchor.clamped(document);
        self.focus = self.focus.clamped(document);
    }

    /// The selected text of `document` as [`selected_text`] gives it.
    #[must_use]
    pub fn text(&self, document: &MarkdownDocument) -> Option<String> {
        self.range().map(|range| selected_text(document, range))
    }
}

/// The text between `range` in `document` as it is shown (Q554): link labels,
/// chips as their chat codes, list items with `- `, `1. `, `[ ] ` or `[x] `
/// in front (indented two spaces per nesting level) when the item's start is
/// selected, a rule as `---`, list items one per line and every other block
/// boundary as a blank line. No markdown markers.
#[must_use]
pub fn selected_text(document: &MarkdownDocument, range: Range<TextPosition>) -> String {
    let mut text = String::new();
    let last_block = range.end.block.min(document.blocks.len().saturating_sub(1));
    for index in range.start.block..=last_block {
        let Some(block) = document.blocks.get(index) else {
            break;
        };
        if index > range.start.block {
            let is_list_continuation = matches!(block, Block::ListItem { .. })
                && matches!(document.blocks[index - 1], Block::ListItem { .. });
            text.push_str(if is_list_continuation { "\n" } else { "\n\n" });
        }
        let from = if index == range.start.block {
            range.start
        } else {
            TextPosition::block_start(index)
        };
        if from == TextPosition::block_start(index) {
            push_block_prefix(&mut text, block);
        }
        let spans = block_spans(block);
        for (span_index, span) in spans.iter().enumerate() {
            let span_range = span_byte_range(span_index, span, index, &range);
            if span_range.is_empty() {
                continue;
            }
            text.push_str(&span.text[span_range]);
        }
    }
    text
}

/// Writes what stands in front of `block`'s text when copied: the list
/// marker of a list item, the whole `---` of a rule.
fn push_block_prefix(text: &mut String, block: &Block) {
    match block {
        Block::ListItem { depth, marker, .. } => {
            for _ in 1..*depth {
                text.push_str("  ");
            }
            match marker {
                ListMarker::Bullet => text.push_str("- "),
                ListMarker::Number(number) => text.push_str(&format!("{number}. ")),
                ListMarker::Task { checked: true, .. } => text.push_str("[x] "),
                ListMarker::Task { checked: false, .. } => text.push_str("[ ] "),
            }
        }
        Block::Rule => text.push_str("---"),
        Block::Heading { .. } | Block::Paragraph { .. } => {}
    }
}

/// The bytes of `span` (span `span_index` of block `block`) inside `range`;
/// a chip is taken whole or not at all.
fn span_byte_range(
    span_index: usize,
    span: &Span,
    block: usize,
    range: &Range<TextPosition>,
) -> Range<usize> {
    let span_start = TextPosition::new(block, span_index, 0);
    let span_end = TextPosition::new(block, span_index, span.text.len());
    if range.end <= span_start || range.start >= span_end {
        return 0..0;
    }
    let start = if range.start > span_start {
        range.start.offset
    } else {
        0
    };
    let end = if range.end < span_end {
        range.end.offset
    } else {
        span.text.len()
    };
    if span.chip.is_some() {
        return 0..span.text.len();
    }
    let clamp = |offset: usize| {
        let mut offset = offset.min(span.text.len());
        while !span.text.is_char_boundary(offset) {
            offset -= 1;
        }
        offset
    };
    clamp(start)..clamp(end).max(clamp(start))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::markdown::parse;

    /// The positions of the whole `document`.
    fn everything(document: &MarkdownDocument) -> Range<TextPosition> {
        let last = document.blocks.len() - 1;
        TextPosition::default()..TextPosition::block_end(document, last)
    }

    #[test]
    fn copies_the_text_as_shown_with_list_markers() {
        let document = parse(
            "## Ectos\n\n- [x] **250** Globs\n- [ ] Mystic Coins\n\nCheck the [wiki](https://w.de) first.\n\n1. Buy at [&AvoJTQAA]\n   - nested\n\n---\n",
        );
        assert_eq!(
            selected_text(&document, everything(&document)),
            "Ectos\n\n[x] 250 Globs\n[ ] Mystic Coins\n\nCheck the wiki first.\n\n1. Buy at [&AvoJTQAA]\n  - nested\n\n---"
        );
    }

    #[test]
    fn a_list_marker_comes_only_with_the_items_start() {
        let document = parse("- [x] 250 Globs of Ectoplasm\n- [ ] Mystic Coins\n");
        let middle = TextPosition::new(0, 0, "250 Globs ".len());
        let end = TextPosition::block_end(&document, 1);
        assert_eq!(
            selected_text(&document, middle..end),
            "of Ectoplasm\n[ ] Mystic Coins"
        );
    }

    #[test]
    fn a_chip_is_copied_whole_and_a_line_break_kept() {
        let document = parse("Buy at [&AvoJTQAA] now  \nthen go");
        let spans = block_spans(&document.blocks[0]);
        let chip = spans.iter().position(|span| span.chip.is_some()).unwrap();
        let from = TextPosition::new(0, chip, 0);
        let to = TextPosition::block_end(&document, 0);
        assert_eq!(
            selected_text(&document, from..to),
            "[&AvoJTQAA] now\nthen go"
        );
    }

    #[test]
    fn the_range_is_ordered_and_empty_selects_nothing() {
        let document = parse("one two");
        let mut selection = TextSelection {
            anchor: TextPosition::new(0, 0, 7),
            focus: TextPosition::new(0, 0, 4),
            ..TextSelection::default()
        };
        assert_eq!(selection.text(&document).as_deref(), Some("two"));
        selection.focus = selection.anchor;
        assert!(selection.is_empty());
        assert_eq!(selection.text(&document), None);
    }

    #[test]
    fn select_all_and_clamping_follow_the_document() {
        let mut selection = TextSelection::default();
        selection.select_all(&parse("Grüße\n\nzwei"));
        assert_eq!(
            selection.text(&parse("Grüße\n\nzwei")).as_deref(),
            Some("Grüße\n\nzwei")
        );
        // The note got shorter: both ends move to valid places.
        let shorter = parse("Grü");
        selection.anchor = TextPosition::new(0, 0, 3);
        selection.clamp(&shorter);
        assert_eq!(selection.anchor, TextPosition::new(0, 0, 2), "inside the ü");
        assert_eq!(selection.focus, TextPosition::new(0, 0, 4));
        assert_eq!(
            TextPosition::new(3, 0, 0).clamped(&MarkdownDocument::default()),
            TextPosition::default()
        );
    }
}
