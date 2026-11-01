//! The editable text: content, cursor, selection and undo history (memo plan
//! M59). Knows nothing about fonts or screen positions; vertical movement and
//! visual line starts/ends come from [`super::layout`].
//!
//! All positions are byte offsets into the UTF-8 text and always lie on
//! character boundaries.

use std::ops::Range;

use crate::chat_link::chat_code_around;

/// At most this many undo steps are kept.
const MAXIMUM_UNDO_STEPS: usize = 200;

/// The kind of the last edit, to group consecutive typing or deleting into
/// one undo step.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum EditKind {
    /// Nothing that can be grouped (or a cursor move happened since).
    Other,
    /// Typing ordinary characters.
    Typing,
    /// Deleting single characters with Backspace or Delete.
    Deleting,
}

/// A restorable state for undo/redo.
#[derive(Debug, Clone, PartialEq, Eq)]
struct Snapshot {
    /// The full text.
    text: String,
    /// Cursor position.
    cursor: usize,
    /// Selection anchor.
    anchor: usize,
}

/// Editable text with cursor, selection and undo history.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TextBuffer {
    /// The text.
    text: String,
    /// Cursor position (the moving end of the selection).
    cursor: usize,
    /// The fixed end of the selection; equal to `cursor` when nothing is
    /// selected.
    anchor: usize,
    /// States to go back to, oldest first.
    undo: Vec<Snapshot>,
    /// States undone, to go forward again.
    redo: Vec<Snapshot>,
    /// What the last edit was, for grouping.
    last_edit: EditKind,
}

/// A horizontal direction.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Direction {
    /// Towards the start of the text.
    Backward,
    /// Towards the end of the text.
    Forward,
}

/// How far a horizontal move or delete reaches.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Step {
    /// One character.
    Character,
    /// To the next word boundary.
    Word,
}

impl TextBuffer {
    /// A buffer with `text` and the cursor at its end.
    #[must_use]
    pub fn new(text: impl Into<String>) -> Self {
        let text = text.into();
        let end = text.len();
        Self {
            text,
            cursor: end,
            anchor: end,
            undo: Vec::new(),
            redo: Vec::new(),
            last_edit: EditKind::Other,
        }
    }

    /// The text.
    #[must_use]
    pub fn text(&self) -> &str {
        &self.text
    }

    /// The cursor position.
    #[must_use]
    pub const fn cursor(&self) -> usize {
        self.cursor
    }

    /// The selected range (start ≤ end); empty when nothing is selected.
    #[must_use]
    pub fn selection(&self) -> Range<usize> {
        self.cursor.min(self.anchor)..self.cursor.max(self.anchor)
    }

    /// Whether some text is selected.
    #[must_use]
    pub const fn has_selection(&self) -> bool {
        self.cursor != self.anchor
    }

    /// The selected text.
    #[must_use]
    pub fn selected_text(&self) -> &str {
        &self.text[self.selection()]
    }

    /// Moves the cursor to `position` (clamped to a character boundary);
    /// with `extend` the selection grows from its anchor, otherwise it is
    /// cleared.
    pub fn set_cursor(&mut self, position: usize, extend: bool) {
        self.cursor = floor_char_boundary(&self.text, position);
        if !extend {
            self.anchor = self.cursor;
        }
        self.last_edit = EditKind::Other;
    }

    /// Selects from `anchor` to `cursor` (clamped); the cursor may lie
    /// before the anchor.
    pub fn select(&mut self, anchor: usize, cursor: usize) {
        self.anchor = floor_char_boundary(&self.text, anchor);
        self.cursor = floor_char_boundary(&self.text, cursor);
        self.last_edit = EditKind::Other;
    }

    /// Selects the whole text.
    pub fn select_all(&mut self) {
        self.select(0, self.text.len());
    }

    /// Selects the word (or run of spaces or punctuation) at `position`.
    pub fn select_word_at(&mut self, position: usize) {
        let position = floor_char_boundary(&self.text, position);
        // A chat-link chip is selected whole (memo plan M75).
        if let Some(code) = chat_code_around(&self.text, position + 1) {
            self.select(code.start, code.end);
            return;
        }
        let class_at = |index: usize| self.text[index..].chars().next().map(CharacterClass::of);
        let class = class_at(position)
            .filter(|class| *class != CharacterClass::LineBreak)
            .or_else(|| previous_char_start(&self.text, position).and_then(class_at));
        let Some(class) = class else {
            self.set_cursor(position, false);
            return;
        };
        let mut start = position;
        while let Some(previous) = previous_char_start(&self.text, start) {
            if class_at(previous) != Some(class) {
                break;
            }
            start = previous;
        }
        let mut end = position;
        while class_at(end) == Some(class) {
            end = next_char_end(&self.text, end);
        }
        self.select(start, end);
    }

    /// The position one step from the cursor in `direction`. A step never
    /// ends inside a chat code: it skips the whole chip (memo plan M75).
    #[must_use]
    pub fn position_after(&self, direction: Direction, step: Step) -> usize {
        let target = match (direction, step) {
            (Direction::Backward, Step::Character) => {
                previous_char_start(&self.text, self.cursor).unwrap_or(0)
            }
            (Direction::Forward, Step::Character) => next_char_end(&self.text, self.cursor),
            (Direction::Backward, Step::Word) => previous_word_start(&self.text, self.cursor),
            (Direction::Forward, Step::Word) => next_word_end(&self.text, self.cursor),
        };
        match (chat_code_around(&self.text, target), direction) {
            (Some(code), Direction::Backward) => code.start,
            (Some(code), Direction::Forward) => code.end,
            (None, _) => target,
        }
    }

    /// Moves the cursor horizontally. Without `extend` and with a selection,
    /// a character step collapses the selection to its side instead.
    pub fn move_horizontally(&mut self, direction: Direction, step: Step, extend: bool) {
        if !extend && self.has_selection() && step == Step::Character {
            let selection = self.selection();
            let target = match direction {
                Direction::Backward => selection.start,
                Direction::Forward => selection.end,
            };
            self.set_cursor(target, false);
            return;
        }
        let target = self.position_after(direction, step);
        self.set_cursor(target, extend);
    }

    /// Replaces the selection with `inserted` (typing or pasting).
    pub fn insert(&mut self, inserted: &str) {
        // Typing over a selection starts a typing group; the following
        // characters join it.
        let is_typing =
            !inserted.is_empty() && inserted.chars().count() == 1 && !inserted.contains('\n');
        let kind = if is_typing {
            EditKind::Typing
        } else {
            EditKind::Other
        };
        self.replace(self.selection(), inserted, kind);
    }

    /// Deletes the selection, or one step in `direction` if nothing is
    /// selected (Backspace / Delete, with Ctrl a word).
    pub fn delete(&mut self, direction: Direction, step: Step) {
        if self.has_selection() {
            self.replace(self.selection(), "", EditKind::Other);
            return;
        }
        let other_end = self.position_after(direction, step);
        let range = self.cursor.min(other_end)..self.cursor.max(other_end);
        if range.is_empty() {
            return;
        }
        let kind = if step == Step::Character {
            EditKind::Deleting
        } else {
            EditKind::Other
        };
        self.replace(range, "", kind);
    }

    /// Enter: inserts a line break and continues a markdown list item
    /// (`- `, `* `, `- [ ] `, `1. `). On an empty list item it ends the list
    /// by removing the marker instead (M59).
    pub fn insert_line_break(&mut self) {
        let selection = self.selection();
        let line_start = line_start(&self.text, selection.start);
        let line = &self.text[line_start..line_end(&self.text, selection.start)];
        let Some(marker) = list_marker(line) else {
            self.replace(selection, "\n", EditKind::Other);
            return;
        };
        if line.trim_end() == marker.text.trim_end() && !self.has_selection() {
            let range = line_start..line_start + line.len();
            self.replace(range, "", EditKind::Other);
            return;
        }
        let continued = format!("\n{}", marker.next);
        self.replace(selection, &continued, EditKind::Other);
    }

    /// Replaces `range` with `replacement` as one undoable edit, grouped with
    /// the previous one when both are of the same groupable `kind` and
    /// directly connected. The cursor ends after the replacement.
    pub fn replace_range(&mut self, range: Range<usize>, replacement: &str) {
        self.replace(range, replacement, EditKind::Other);
    }

    /// Selects `range` after an external edit (e.g. a formatting command).
    pub fn select_after_edit(&mut self, range: Range<usize>) {
        self.select(range.start, range.end);
    }

    /// Goes back one undo step. Returns whether anything changed.
    pub fn undo(&mut self) -> bool {
        let Some(snapshot) = self.undo.pop() else {
            return false;
        };
        let current = self.snapshot();
        self.redo.push(current);
        self.restore(snapshot);
        true
    }

    /// Goes forward one undone step. Returns whether anything changed.
    pub fn redo(&mut self) -> bool {
        let Some(snapshot) = self.redo.pop() else {
            return false;
        };
        let current = self.snapshot();
        self.undo.push(current);
        self.restore(snapshot);
        true
    }

    /// Performs an edit and records the undo step.
    fn replace(&mut self, range: Range<usize>, replacement: &str, kind: EditKind) {
        let start = floor_char_boundary(&self.text, range.start);
        let end = floor_char_boundary(&self.text, range.end).max(start);
        let groups_with_last = kind != EditKind::Other
            && kind == self.last_edit
            && match kind {
                EditKind::Typing => start == self.cursor,
                _ => start == self.cursor || end == self.cursor,
            };
        if !groups_with_last {
            let snapshot = self.snapshot();
            self.undo.push(snapshot);
            if self.undo.len() > MAXIMUM_UNDO_STEPS {
                self.undo.remove(0);
            }
        }
        self.redo.clear();
        self.text.replace_range(start..end, replacement);
        self.cursor = start + replacement.len();
        self.anchor = self.cursor;
        self.last_edit = kind;
    }

    /// The current state for the undo history.
    fn snapshot(&self) -> Snapshot {
        Snapshot {
            text: self.text.clone(),
            cursor: self.cursor,
            anchor: self.anchor,
        }
    }

    /// Restores a state from the undo history.
    fn restore(&mut self, snapshot: Snapshot) {
        self.text = snapshot.text;
        self.cursor = snapshot.cursor;
        self.anchor = snapshot.anchor;
        self.last_edit = EditKind::Other;
    }
}

/// A list marker at the start of a line and the marker for the next item.
#[derive(Debug, Clone, PartialEq, Eq)]
struct ListMarker {
    /// The marker as written, including indentation and the trailing space.
    text: String,
    /// The marker for the continued item.
    next: String,
}

/// Recognizes a markdown list marker at the start of `line`: indentation,
/// then `- `, `* ` or `+ ` (optionally followed by a task box) or `N. `.
fn list_marker(line: &str) -> Option<ListMarker> {
    let indentation = &line[..line.len() - line.trim_start().len()];
    let rest = &line[indentation.len()..];
    for bullet in ["- ", "* ", "+ "] {
        if let Some(after) = rest.strip_prefix(bullet) {
            let task = ["[ ] ", "[x] ", "[X] "]
                .into_iter()
                .find(|task| after.starts_with(task));
            let written = format!("{indentation}{bullet}{}", task.unwrap_or(""));
            let next = format!("{indentation}{bullet}{}", task.map_or("", |_| "[ ] "));
            return Some(ListMarker {
                text: written,
                next,
            });
        }
    }
    let digits = rest.chars().take_while(char::is_ascii_digit).count();
    if digits > 0 && rest[digits..].starts_with(". ") {
        let number: u64 = rest[..digits].parse().ok()?;
        return Some(ListMarker {
            text: format!("{indentation}{number}. "),
            next: format!("{indentation}{}. ", number.saturating_add(1)),
        });
    }
    None
}

/// Kinds of characters for word movement and double-click selection.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum CharacterClass {
    /// Letters, digits and `_`.
    Word,
    /// Spaces and tabs.
    Space,
    /// A line break.
    LineBreak,
    /// Everything else.
    Punctuation,
}

impl CharacterClass {
    /// The class of `character`.
    fn of(character: char) -> Self {
        if character == '\n' {
            Self::LineBreak
        } else if character.is_whitespace() {
            Self::Space
        } else if character.is_alphanumeric() || character == '_' {
            Self::Word
        } else {
            Self::Punctuation
        }
    }
}

/// Start of the line containing `position`.
#[must_use]
pub fn line_start(text: &str, position: usize) -> usize {
    text[..position].rfind('\n').map_or(0, |index| index + 1)
}

/// End of the line containing `position` (before its line break).
#[must_use]
pub fn line_end(text: &str, position: usize) -> usize {
    text[position..]
        .find('\n')
        .map_or(text.len(), |index| position + index)
}

/// The start of the character before `position`, if any.
fn previous_char_start(text: &str, position: usize) -> Option<usize> {
    text[..position]
        .char_indices()
        .next_back()
        .map(|(index, _)| index)
}

/// The position after the character at `position` (or the text end).
fn next_char_end(text: &str, position: usize) -> usize {
    text[position..]
        .chars()
        .next()
        .map_or(text.len(), |character| position + character.len_utf8())
}

/// Ctrl+Left: skips spaces backwards, then the run of same-class characters.
fn previous_word_start(text: &str, position: usize) -> usize {
    let mut index = position;
    let class_before = |index: usize| {
        previous_char_start(text, index).and_then(|start| text[start..].chars().next())
    };
    while let Some(character) = class_before(index) {
        if CharacterClass::of(character) != CharacterClass::Space {
            break;
        }
        index = previous_char_start(text, index).unwrap_or(0);
    }
    let Some(first) = class_before(index) else {
        return index;
    };
    let class = CharacterClass::of(first);
    if class == CharacterClass::LineBreak {
        return previous_char_start(text, index).unwrap_or(0);
    }
    while let Some(character) = class_before(index) {
        if CharacterClass::of(character) != class {
            break;
        }
        index = previous_char_start(text, index).unwrap_or(0);
    }
    index
}

/// Ctrl+Right: skips the run of same-class characters, then spaces.
fn next_word_end(text: &str, position: usize) -> usize {
    let class_at = |index: usize| text[index..].chars().next().map(CharacterClass::of);
    let Some(class) = class_at(position) else {
        return position;
    };
    if class == CharacterClass::LineBreak {
        return next_char_end(text, position);
    }
    let mut index = position;
    if class != CharacterClass::Space {
        while class_at(index) == Some(class) {
            index = next_char_end(text, index);
        }
    }
    while class_at(index) == Some(CharacterClass::Space) {
        index = next_char_end(text, index);
    }
    index
}

/// The largest character boundary of `text` that is at most `index`.
#[must_use]
pub fn floor_char_boundary(text: &str, index: usize) -> usize {
    let mut index = index.min(text.len());
    while !text.is_char_boundary(index) {
        index -= 1;
    }
    index
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Types `text` character by character, like the keyboard does.
    fn type_text(buffer: &mut TextBuffer, text: &str) {
        let mut encoded = [0; 4];
        for character in text.chars() {
            if character == '\n' {
                buffer.insert_line_break();
            } else {
                buffer.insert(character.encode_utf8(&mut encoded));
            }
        }
    }

    #[test]
    fn typing_replaces_the_selection_and_groups_into_one_undo_step() {
        let mut buffer = TextBuffer::new("Hello world");
        buffer.select(6, 11);
        type_text(&mut buffer, "Tyria");
        assert_eq!(buffer.text(), "Hello Tyria");
        assert!(buffer.undo());
        assert_eq!(buffer.text(), "Hello world");
        assert_eq!(buffer.selection(), 6..11);
        assert!(buffer.redo());
        assert_eq!(buffer.text(), "Hello Tyria");
        assert!(!buffer.redo());
    }

    #[test]
    fn a_cursor_move_starts_a_new_undo_step() {
        let mut buffer = TextBuffer::new("");
        type_text(&mut buffer, "ab");
        buffer.move_horizontally(Direction::Backward, Step::Character, false);
        type_text(&mut buffer, "x");
        assert_eq!(buffer.text(), "axb");
        assert!(buffer.undo());
        assert_eq!(buffer.text(), "ab");
        assert!(buffer.undo());
        assert_eq!(buffer.text(), "");
        assert!(!buffer.undo());
    }

    #[test]
    fn umlauts_move_and_delete_as_one_character() {
        let mut buffer = TextBuffer::new("Grüße");
        buffer.delete(Direction::Backward, Step::Character);
        assert_eq!(buffer.text(), "Grüß");
        buffer.move_horizontally(Direction::Backward, Step::Character, false);
        buffer.delete(Direction::Backward, Step::Character);
        assert_eq!(buffer.text(), "Grß");
        buffer.set_cursor(3, false);
        assert_eq!(buffer.cursor(), 2, "inside ß is clamped to its start");
    }

    #[test]
    fn word_movement_and_deletion() {
        let mut buffer = TextBuffer::new("one two, three");
        buffer.set_cursor(0, false);
        buffer.move_horizontally(Direction::Forward, Step::Word, false);
        assert_eq!(buffer.cursor(), 4);
        buffer.move_horizontally(Direction::Forward, Step::Word, true);
        assert_eq!(buffer.selected_text(), "two");
        buffer.set_cursor(buffer.text().len(), false);
        buffer.delete(Direction::Backward, Step::Word);
        assert_eq!(buffer.text(), "one two, ");
        buffer.move_horizontally(Direction::Backward, Step::Word, false);
        assert_eq!(buffer.cursor(), 7, "punctuation is its own word");
    }

    #[test]
    fn double_click_selects_a_word() {
        let mut buffer = TextBuffer::new("daily: Grüße an alle");
        buffer.select_word_at(9);
        assert_eq!(buffer.selected_text(), "Grüße");
        buffer.select_word_at(2);
        assert_eq!(buffer.selected_text(), "daily");
    }

    #[test]
    fn enter_continues_lists_and_ends_them_on_an_empty_item() {
        let mut buffer = TextBuffer::new("");
        type_text(&mut buffer, "- [x] done\nnext\n\n");
        assert_eq!(buffer.text(), "- [x] done\n- [ ] next\n");
        let mut numbered = TextBuffer::new("");
        type_text(&mut numbered, "  9. a\nb");
        assert_eq!(numbered.text(), "  9. a\n  10. b");
        let mut plain = TextBuffer::new("");
        type_text(&mut plain, "text\nmore");
        assert_eq!(plain.text(), "text\nmore");
    }

    #[test]
    fn backspace_deletes_the_selection() {
        let mut buffer = TextBuffer::new("abcdef");
        buffer.select(1, 4);
        buffer.delete(Direction::Backward, Step::Character);
        assert_eq!(buffer.text(), "aef");
        assert_eq!(buffer.cursor(), 1);
    }

    #[test]
    fn character_move_collapses_a_selection() {
        let mut buffer = TextBuffer::new("abcdef");
        buffer.select(4, 1);
        buffer.move_horizontally(Direction::Forward, Step::Character, false);
        assert_eq!(buffer.cursor(), 4);
        assert!(!buffer.has_selection());
    }

    #[test]
    fn undo_history_is_capped() {
        let mut buffer = TextBuffer::new("");
        for _ in 0..MAXIMUM_UNDO_STEPS + 10 {
            buffer.insert("x\n");
        }
        let mut steps = 0;
        while buffer.undo() {
            steps += 1;
        }
        assert_eq!(steps, MAXIMUM_UNDO_STEPS);
    }

    #[test]
    fn chat_codes_move_and_delete_as_one_unit() {
        let text = "a [&BDgAAAA=] b";
        let mut buffer = TextBuffer::new(text);
        buffer.set_cursor(2, false);
        buffer.move_horizontally(Direction::Forward, Step::Character, false);
        assert_eq!(buffer.cursor(), 13);
        buffer.move_horizontally(Direction::Backward, Step::Character, false);
        assert_eq!(buffer.cursor(), 2);
        buffer.set_cursor(13, false);
        buffer.delete(Direction::Backward, Step::Character);
        assert_eq!(buffer.text(), "a  b");
        buffer.undo();
        buffer.select_word_at(2);
        assert_eq!(buffer.selected_text(), "[&BDgAAAA=]");
    }
}
