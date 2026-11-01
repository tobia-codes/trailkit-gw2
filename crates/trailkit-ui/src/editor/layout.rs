//! Word wrapping and position mapping for the editor (memo plan M59).
//!
//! The layout splits the text into visual lines that fit a width, using a
//! caller-provided measure function (ImGui text widths in the view, a fixed
//! width per character in tests). ImGui has no kerning, so the width of a
//! text is the sum of its characters' advances; the layout measures each
//! character once and keeps the x position of every character boundary.
//! It is rebuilt only when the text or the width changes.

use std::ops::Range;

use super::highlight::{SpanStyle, StyledSpan, highlight_line};

/// One line on screen.
#[derive(Debug, Clone, PartialEq)]
pub struct VisualLine {
    /// The text range shown on this line (without a trailing line break).
    pub range: Range<usize>,
    /// Whether this is the last visual line of its logical line.
    pub ends_logical_line: bool,
    /// Every character boundary of the line: byte position and x offset
    /// from the line start; starts with `(range.start, 0.0)`.
    boundaries: Vec<(usize, f32)>,
}

impl VisualLine {
    /// The x offset of `position` (clamped to the line).
    #[must_use]
    pub fn x_of(&self, position: usize) -> f32 {
        let index = self
            .boundaries
            .partition_point(|(boundary, _)| *boundary <= position);
        self.boundaries
            .get(index.saturating_sub(1))
            .map_or(0.0, |(_, x)| *x)
    }

    /// The character boundary closest to `x`.
    #[must_use]
    pub fn position_at(&self, x: f32) -> usize {
        let index = self
            .boundaries
            .partition_point(|(_, boundary_x)| *boundary_x < x);
        let after = self.boundaries.get(index);
        let before = index
            .checked_sub(1)
            .and_then(|index| self.boundaries.get(index));
        match (before, after) {
            (Some(before), Some(after)) if x - before.1 < after.1 - x => before.0,
            (_, Some(after)) => after.0,
            (Some(before), None) => before.0,
            (None, None) => self.range.start,
        }
    }

    /// The width of the line.
    #[must_use]
    pub fn width(&self) -> f32 {
        self.boundaries.last().map_or(0.0, |(_, x)| *x)
    }
}

/// The wrapped text: visual lines plus highlighting spans.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct Layout {
    /// Visual lines, top to bottom; at least one.
    lines: Vec<VisualLine>,
    /// Highlighting spans covering the text, in order.
    spans: Vec<StyledSpan>,
}

impl Layout {
    /// Wraps `text` at `width` (a line holds at least one character).
    /// `measure` returns the width of a single character in a style.
    pub fn build(text: &str, width: f32, mut measure: impl FnMut(&str, SpanStyle) -> f32) -> Self {
        let mut spans = Vec::new();
        let mut lines = Vec::new();
        let mut line_start = 0;
        loop {
            let line_end = text[line_start..]
                .find('\n')
                .map_or(text.len(), |index| line_start + index);
            let first_span = spans.len();
            highlight_line(&text[line_start..line_end], line_start, &mut spans);
            wrap_logical_line(
                text,
                line_start..line_end,
                &spans[first_span..],
                width,
                &mut measure,
                &mut lines,
            );
            if line_end == text.len() {
                break;
            }
            line_start = line_end + 1;
        }
        Self { lines, spans }
    }

    /// The visual lines, top to bottom (at least one).
    #[must_use]
    pub fn lines(&self) -> &[VisualLine] {
        &self.lines
    }

    /// The highlighting spans that intersect `range`, clipped to it.
    pub fn spans_in(&self, range: Range<usize>) -> impl Iterator<Item = StyledSpan> + '_ {
        let first = self
            .spans
            .partition_point(|span| span.range.end <= range.start);
        self.spans[first..]
            .iter()
            .take_while(move |span| span.range.start < range.end)
            .map(move |span| StyledSpan {
                range: span.range.start.max(range.start)..span.range.end.min(range.end),
                style: span.style,
            })
    }

    /// The index of the visual line that shows `position`. A position at a
    /// wrap point belongs to the following line.
    #[must_use]
    pub fn line_index_of(&self, position: usize) -> usize {
        self.lines
            .partition_point(|line| line.range.start <= position)
            .saturating_sub(1)
    }

    /// The visual line that shows `position`.
    #[must_use]
    pub fn line_of(&self, position: usize) -> &VisualLine {
        &self.lines[self.line_index_of(position)]
    }

    /// Home: the start of the visual line of `position`.
    #[must_use]
    pub fn visual_line_start(&self, position: usize) -> usize {
        self.line_of(position).range.start
    }

    /// End: the end of the visual line of `position`. On a wrapped line this
    /// is before its last character (the space it wrapped at), so the cursor
    /// stays on that line.
    #[must_use]
    pub fn visual_line_end(&self, position: usize) -> usize {
        let line = self.line_of(position);
        if line.ends_logical_line {
            line.range.end
        } else {
            line.boundaries
                .iter()
                .rev()
                .nth(1)
                .map_or(line.range.start, |(boundary, _)| *boundary)
        }
    }

    /// The position `lines` visual lines above (negative) or below
    /// (positive) `position`, at horizontal offset `x`.
    #[must_use]
    pub fn position_vertically(&self, position: usize, lines: isize, x: f32) -> usize {
        let current = self.line_index_of(position);
        let target = current
            .saturating_add_signed(lines)
            .min(self.lines.len() - 1);
        if lines < 0 && current == 0 {
            return 0;
        }
        if lines > 0 && current == self.lines.len() - 1 {
            return self.lines[current].range.end;
        }
        let line = &self.lines[target];
        let found = line.position_at(x);
        if line.ends_logical_line {
            found
        } else {
            found.min(self.visual_line_end(line.range.start))
        }
    }
}

/// Wraps one logical line (`range`, without line break) into visual lines.
/// Spaces may hang past the width; a word longer than the width breaks hard.
fn wrap_logical_line(
    text: &str,
    range: Range<usize>,
    spans: &[StyledSpan],
    width: f32,
    measure: &mut impl FnMut(&str, SpanStyle) -> f32,
    lines: &mut Vec<VisualLine>,
) {
    let mut boundaries = vec![(range.start, 0.0)];
    let mut line_start = range.start;
    // Index into `boundaries` right after the last space: a wrap opportunity.
    let mut last_break: Option<usize> = None;
    let mut span_index = 0;
    for (offset, character) in text[range.clone()].char_indices() {
        let position = range.start + offset;
        while spans
            .get(span_index)
            .is_some_and(|span| span.range.end <= position)
        {
            span_index += 1;
        }
        let style = spans
            .get(span_index)
            .map_or(SpanStyle::Text, |span| span.style);
        let mut encoded = [0; 4];
        let advance = measure(character.encode_utf8(&mut encoded), style);
        let x = boundaries.last().map_or(0.0, |(_, x)| *x);
        let is_space = character == ' ' || character == '\t';
        if !is_space && x + advance > width && position > line_start {
            let split = last_break
                .filter(|index| *index > 1)
                .unwrap_or(boundaries.len() - 1);
            let rest: Vec<(usize, f32)> = boundaries[split..].to_vec();
            boundaries.truncate(split + 1);
            let wrap_at = boundaries
                .last()
                .map_or(position, |(boundary, _)| *boundary);
            lines.push(VisualLine {
                range: line_start..wrap_at,
                ends_logical_line: false,
                boundaries: std::mem::take(&mut boundaries),
            });
            let shift = rest.first().map_or(0.0, |(_, x)| *x);
            boundaries = rest
                .iter()
                .map(|(boundary, x)| (*boundary, x - shift))
                .collect();
            line_start = wrap_at;
            last_break = None;
        }
        let x = boundaries.last().map_or(0.0, |(_, x)| *x);
        boundaries.push((position + character.len_utf8(), x + advance));
        if is_space {
            last_break = Some(boundaries.len() - 1);
        }
    }
    lines.push(VisualLine {
        range: line_start..range.end,
        ends_logical_line: true,
        boundaries,
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Every character is one unit wide; bold is two.
    fn monospace(_: &str, style: SpanStyle) -> f32 {
        if style == SpanStyle::Strong { 2.0 } else { 1.0 }
    }

    /// The visual lines of `text` wrapped at `width`, as strings.
    fn wrapped(text: &str, width: f32) -> Vec<String> {
        Layout::build(text, width, monospace)
            .lines()
            .iter()
            .map(|line| text[line.range.clone()].to_owned())
            .collect()
    }

    #[test]
    fn wraps_at_spaces_and_breaks_long_words() {
        assert_eq!(wrapped("one two three", 8.0), ["one two ", "three"]);
        assert_eq!(wrapped("abcdefghij", 4.0), ["abcd", "efgh", "ij"]);
        assert_eq!(wrapped("a\n\nb", 10.0), ["a", "", "b"]);
        assert_eq!(wrapped("", 10.0), [""]);
    }

    #[test]
    fn heading_text_is_measured_bold() {
        // "# " is a marker (1 unit each), "abcd" is bold (2 units each): the
        // word moves to the next line whole, then breaks hard after "abc".
        assert_eq!(wrapped("# abcd", 6.0), ["# ", "abc", "d"]);
    }

    #[test]
    fn positions_map_to_lines_and_back() {
        let text = "one two three";
        let layout = Layout::build(text, 8.0, monospace);
        assert_eq!(layout.line_index_of(0), 0);
        assert_eq!(
            layout.line_index_of(8),
            1,
            "the wrap point starts the next line"
        );
        assert_eq!(layout.line_index_of(text.len()), 1);
        let line = layout.line_of(2);
        assert!((line.x_of(2) - 2.0).abs() < f32::EPSILON);
        assert_eq!(line.position_at(2.4), 2);
        assert_eq!(line.position_at(2.6), 3);
        assert_eq!(layout.visual_line_end(0), 7, "before the hanging space");
        assert_eq!(layout.visual_line_start(10), 8);
    }

    #[test]
    fn vertical_moves_keep_the_column() {
        let text = "abcdef\nab\nabcdef";
        let layout = Layout::build(text, 100.0, monospace);
        assert_eq!(
            layout.position_vertically(5, 1, 5.0),
            9,
            "clamped to the short line"
        );
        assert_eq!(layout.position_vertically(9, 1, 5.0), 15);
        assert_eq!(
            layout.position_vertically(3, -1, 3.0),
            0,
            "top line goes to start"
        );
        assert_eq!(layout.position_vertically(12, 1, 2.0), text.len());
    }

    #[test]
    fn spans_are_clipped_to_a_range() {
        let layout = Layout::build("a **b**", 100.0, monospace);
        let styles: Vec<SpanStyle> = layout.spans_in(1..5).map(|span| span.style).collect();
        assert_eq!(
            styles,
            [SpanStyle::Text, SpanStyle::Marker, SpanStyle::Text]
        );
    }
}
