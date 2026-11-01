//! Editing a note (memo plan M51–M54): the edit session that holds an
//! uncommitted copy of title and body, and the markdown formatting commands
//! of the editor toolbar.
//!
//! Platform-independent so it can be unit-tested on any host (ROADMAP R23).
//! All text positions are byte offsets into UTF-8 text, as ImGui's input
//! callbacks report them.

use std::ops::Range;

use jiff::Timestamp;

use crate::{
    drafts::Draft,
    note_tree::{Note, NoteId},
};

/// An edit in progress: a copy of the note's title and body that is only
/// written back on "Done" (M52).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EditSession {
    /// The note being edited.
    pub note: NoteId,
    /// The edited title.
    pub title: String,
    /// The edited markdown body.
    pub body: String,
    /// The title when editing started, to detect changes.
    original_title: String,
    /// The body when editing started, to detect changes.
    original_body: String,
}

impl EditSession {
    /// Starts editing `note` with its current title and body.
    #[must_use]
    pub fn start(note: &Note) -> Self {
        Self {
            note: note.id,
            title: note.title.clone(),
            body: note.body.clone(),
            original_title: note.title.clone(),
            original_body: note.body.clone(),
        }
    }

    /// Resumes editing `note` with the content of a restored `draft` (M54).
    /// The session counts as changed whenever the draft differs from the note.
    #[must_use]
    pub fn from_draft(note: &Note, draft: Draft) -> Self {
        Self {
            note: note.id,
            title: draft.title,
            body: draft.body,
            original_title: note.title.clone(),
            original_body: note.body.clone(),
        }
    }

    /// Whether title or body differ from the note as it was when editing
    /// started.
    #[must_use]
    pub fn has_changes(&self) -> bool {
        self.title != self.original_title || self.body != self.original_body
    }

    /// Turns the uncommitted content into a draft to keep across an unload
    /// (M54). `note_updated_at` is the note's last change, so a later restore
    /// can tell whether the note changed in between.
    #[must_use]
    pub fn to_draft(&self, saved_at: Timestamp, note_updated_at: Timestamp) -> Draft {
        Draft {
            note: self.note,
            title: self.title.clone(),
            body: self.body.clone(),
            saved_at,
            note_updated_at,
        }
    }
}

/// A formatting button of the editor toolbar (M53).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FormatCommand {
    /// `## ` at the start of the line (toggles).
    Heading,
    /// `**…**` around the selection.
    Bold,
    /// `*…*` around the selection.
    Italic,
    /// `- ` at the start of the line (toggles).
    BulletList,
    /// `- [ ] ` at the start of the line (toggles).
    Checkbox,
    /// `[…](https://)` around the selection, with the URL part selected.
    Link,
}

/// One replacement in the text, plus the selection to show afterwards.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TextEdit {
    /// The byte range of the original text that is replaced.
    pub range: Range<usize>,
    /// The text inserted instead.
    pub replacement: String,
    /// The selection (byte range) in the resulting text; empty = cursor.
    pub selection: Range<usize>,
}

impl TextEdit {
    /// Applies the edit to `text` (for callers without an input callback,
    /// and for tests).
    pub fn apply_to(&self, text: &mut String) {
        text.replace_range(self.range.clone(), &self.replacement);
    }
}

/// URL placeholder inserted by [`FormatCommand::Link`]; selected afterwards
/// so typing replaces it.
const LINK_URL_PLACEHOLDER: &str = "https://";

/// Computes the edit that applies `command` to `text` with the current
/// `selection` (byte range; empty = cursor position). Out-of-range or
/// non-character-boundary positions are clamped to valid ones.
#[must_use]
pub fn format_edit(text: &str, selection: Range<usize>, command: FormatCommand) -> TextEdit {
    let start = floor_char_boundary(text, selection.start.min(selection.end));
    let end = floor_char_boundary(text, selection.start.max(selection.end));
    match command {
        FormatCommand::Bold => toggle_emphasis(text, start..end, Emphasis::Bold),
        FormatCommand::Italic => toggle_emphasis(text, start..end, Emphasis::Italic),
        FormatCommand::Heading => toggle_line_prefix(text, start..end, "## "),
        FormatCommand::BulletList => toggle_line_prefix(text, start..end, "- "),
        FormatCommand::Checkbox => toggle_line_prefix(text, start..end, "- [ ] "),
        FormatCommand::Link => {
            let label = &text[start..end];
            let replacement = format!("[{label}]({LINK_URL_PLACEHOLDER})");
            let selection = if label.is_empty() {
                // Cursor inside the brackets to type the label first.
                start + 1..start + 1
            } else {
                let url_start = start + label.len() + 3;
                url_start..url_start + LINK_URL_PLACEHOLDER.len()
            };
            TextEdit {
                range: start..end,
                replacement,
                selection,
            }
        }
    }
}

/// Bold (`**`) or italic (`*`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Emphasis {
    /// `**…**`.
    Bold,
    /// `*…*`.
    Italic,
}

impl Emphasis {
    /// The marker written on both sides.
    const fn marker(self) -> &'static str {
        match self {
            Self::Bold => "**",
            Self::Italic => "*",
        }
    }

    /// Whether `before` (stars directly left) and `after` (stars directly
    /// right) contain this emphasis: bold needs two stars on each side,
    /// italic an odd number (`*x*` or `***x***`), so italic never eats half
    /// of a bold marker.
    const fn is_applied(self, before: usize, after: usize) -> bool {
        match self {
            Self::Bold => before >= 2 && after >= 2,
            Self::Italic => before % 2 == 1 && after % 2 == 1,
        }
    }
}

/// Toggles bold or italic around `selection` (M53, M68): removes the markers
/// if they already surround the selection or the cursor, or if the selection
/// itself starts and ends with them; otherwise wraps the selection (or puts
/// the cursor between new markers). The selection stays on the text.
fn toggle_emphasis(text: &str, selection: Range<usize>, emphasis: Emphasis) -> TextEdit {
    let marker = emphasis.marker();
    let stars_before = text[..selection.start]
        .bytes()
        .rev()
        .take_while(|byte| *byte == b'*')
        .count();
    let stars_after = text[selection.end..]
        .bytes()
        .take_while(|byte| *byte == b'*')
        .count();
    let inner = &text[selection.clone()];
    if emphasis.is_applied(stars_before, stars_after) {
        // Remove the innermost markers on both sides.
        let start = selection.start - marker.len();
        let end = selection.end + marker.len();
        return TextEdit {
            range: start..end,
            replacement: inner.to_owned(),
            selection: start..start + inner.len(),
        };
    }
    let inner_before = inner.bytes().take_while(|byte| *byte == b'*').count();
    let inner_after = inner.bytes().rev().take_while(|byte| *byte == b'*').count();
    if inner.len() > 2 * marker.len() && emphasis.is_applied(inner_before, inner_after) {
        let unwrapped = &inner[marker.len()..inner.len() - marker.len()];
        return TextEdit {
            range: selection.clone(),
            replacement: unwrapped.to_owned(),
            selection: selection.start..selection.start + unwrapped.len(),
        };
    }
    let inner_start = selection.start + marker.len();
    TextEdit {
        range: selection.clone(),
        replacement: format!("{marker}{inner}{marker}"),
        selection: inner_start..inner_start + inner.len(),
    }
}

/// Adds `prefix` at the start of the line containing `selection.start`, or
/// removes it if the line already starts with it; the selection moves along.
fn toggle_line_prefix(text: &str, selection: Range<usize>, prefix: &str) -> TextEdit {
    let line_start = text[..selection.start]
        .rfind('\n')
        .map_or(0, |index| index + 1);
    if text[line_start..].starts_with(prefix) {
        let shift = |position: usize| position.saturating_sub(prefix.len()).max(line_start);
        TextEdit {
            range: line_start..line_start + prefix.len(),
            replacement: String::new(),
            selection: shift(selection.start)..shift(selection.end),
        }
    } else {
        TextEdit {
            range: line_start..line_start,
            replacement: prefix.to_owned(),
            selection: selection.start + prefix.len()..selection.end + prefix.len(),
        }
    }
}

/// The largest character boundary of `text` that is at most `index`.
fn floor_char_boundary(text: &str, index: usize) -> usize {
    let mut index = index.min(text.len());
    while !text.is_char_boundary(index) {
        index -= 1;
    }
    index
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::note_tree::NoteKind;

    /// Applies `command` to `text` with `selection` and returns the new text
    /// and selection.
    fn run(text: &str, selection: Range<usize>, command: FormatCommand) -> (String, Range<usize>) {
        let edit = format_edit(text, selection, command);
        let mut result = text.to_owned();
        edit.apply_to(&mut result);
        (result, edit.selection)
    }

    #[test]
    fn bold_and_italic_wrap_the_selection_or_insert_at_the_cursor() {
        assert_eq!(
            run("a big word", 2..5, FormatCommand::Bold),
            ("a **big** word".to_owned(), 4..7)
        );
        assert_eq!(
            run("ab", 1..1, FormatCommand::Italic),
            ("a**b".to_owned(), 2..2)
        );
    }

    #[test]
    fn bold_and_italic_toggle_off_again() {
        // Cursor between empty markers (the reported case).
        assert_eq!(
            run("sdasd****", 7..7, FormatCommand::Bold),
            ("sdasd".to_owned(), 5..5)
        );
        // Selection inside markers, and selection including them.
        assert_eq!(
            run("a **b** c", 4..5, FormatCommand::Bold),
            ("a b c".to_owned(), 2..3)
        );
        assert_eq!(
            run("a **b** c", 2..7, FormatCommand::Bold),
            ("a b c".to_owned(), 2..3)
        );
        assert_eq!(
            run("*x*", 1..2, FormatCommand::Italic),
            ("x".to_owned(), 0..1)
        );
        // Italic does not take half of a bold marker; it adds its own.
        assert_eq!(
            run("**x**", 2..3, FormatCommand::Italic),
            ("***x***".to_owned(), 3..4)
        );
        assert_eq!(
            run("***x***", 3..4, FormatCommand::Italic),
            ("**x**".to_owned(), 2..3)
        );
        assert_eq!(
            run("***x***", 3..4, FormatCommand::Bold),
            ("*x*".to_owned(), 1..2)
        );
    }

    #[test]
    fn line_prefixes_toggle_on_the_current_line() {
        let text = "first\nsecond";
        let (with_prefix, selection) = run(text, 8..8, FormatCommand::Checkbox);
        assert_eq!(with_prefix, "first\n- [ ] second");
        assert_eq!(selection, 14..14);
        let (removed, selection) = run(&with_prefix, selection, FormatCommand::Checkbox);
        assert_eq!(removed, text);
        assert_eq!(selection, 8..8);
        assert_eq!(
            run("", 0..0, FormatCommand::Heading),
            ("## ".to_owned(), 3..3)
        );
    }

    #[test]
    fn link_selects_the_url_or_puts_the_cursor_into_the_label() {
        let (text, selection) = run("see wiki", 4..8, FormatCommand::Link);
        assert_eq!(text, "see [wiki](https://)");
        assert_eq!(&text[selection], "https://");
        assert_eq!(
            run("", 0..0, FormatCommand::Link),
            ("[](https://)".to_owned(), 1..1)
        );
    }

    #[test]
    fn positions_inside_characters_are_clamped() {
        // "ä" is two bytes; position 1 lies inside it.
        let (text, _) = run("ä", 1..1, FormatCommand::Bold);
        assert_eq!(text, "****ä");
        let (text, _) = run("ab", 7..9, FormatCommand::Bold);
        assert_eq!(text, "ab****");
    }

    #[test]
    fn session_detects_changes() {
        let note = Note::new("Title".to_owned(), NoteKind::Normal, Timestamp::UNIX_EPOCH);
        let mut session = EditSession::start(&note);
        assert!(!session.has_changes());
        session.body.push('x');
        assert!(session.has_changes());
        session.body.clear();
        assert!(!session.has_changes());
        let draft = session.to_draft(Timestamp::UNIX_EPOCH, note.updated_at);
        assert!(!EditSession::from_draft(&note, draft).has_changes());
    }
}
