//! What the link popup edits (memo plan M208): the link at the selection, the
//! check of its address and the markdown it becomes. Platform-independent, so
//! it can be unit-tested on any host (ROADMAP R23).

use std::ops::Range;

use trailkit_ui::markdown::{inline_links, is_openable_link};

use crate::note_editing::TextEdit;

/// The scheme added to an address without one.
const DEFAULT_SCHEME: &str = "https://";

/// The scheme added to an e-mail address.
const MAIL_SCHEME: &str = "mailto:";

/// The field of the link popup that gets the keyboard focus first.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LinkField {
    /// The link text.
    Text,
    /// The address.
    Address,
}

/// The starting values of the link popup.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LinkDraft {
    /// The part of the body the link replaces.
    pub range: Range<usize>,
    /// The link text.
    pub text: String,
    /// The address.
    pub address: String,
    /// Whether `range` already is a link.
    pub is_existing: bool,
    /// The field to focus.
    pub focus: LinkField,
}

/// Why an address cannot be used.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AddressProblem {
    /// Nothing was entered.
    Missing,
    /// It has spaces or a scheme memo does not open.
    Unsupported,
}

/// The starting values for `selection` in `body`: the link around it, or a new
/// link with the selected text and an address from `clipboard`, if it has one.
#[must_use]
pub fn link_draft(body: &str, selection: Range<usize>, clipboard: Option<&str>) -> LinkDraft {
    let existing = inline_links(body)
        .into_iter()
        .find(|link| link.range.start <= selection.start && selection.end <= link.range.end);
    if let Some(link) = existing {
        let target = &body[link.target_range()];
        return LinkDraft {
            text: body[link.text_range()].to_owned(),
            address: target.split_whitespace().next().unwrap_or("").to_owned(),
            range: link.range,
            is_existing: true,
            focus: LinkField::Address,
        };
    }
    let text = body[selection.clone()].replace('\n', " ");
    let address = clipboard
        .map(str::trim)
        .filter(|candidate| is_openable_link(candidate) && !candidate.contains(char::is_whitespace))
        .unwrap_or("")
        .to_owned();
    let focus = if text.is_empty() {
        LinkField::Text
    } else {
        LinkField::Address
    };
    LinkDraft {
        range: selection,
        text,
        address,
        is_existing: false,
        focus,
    }
}

/// The address to store for what the user typed: `https://` is added when the
/// scheme is missing, `mailto:` for an e-mail address.
///
/// # Errors
///
/// Returns the [`AddressProblem`] if `address` is empty or cannot be opened.
pub fn normalized_address(address: &str) -> Result<String, AddressProblem> {
    let address = address.trim();
    if address.is_empty() {
        return Err(AddressProblem::Missing);
    }
    if address.contains(|character: char| character.is_whitespace() || "<>".contains(character)) {
        return Err(AddressProblem::Unsupported);
    }
    let has_scheme =
        address.contains("://") || address.to_ascii_lowercase().starts_with(MAIL_SCHEME);
    if has_scheme {
        return if is_openable_link(address) {
            Ok(address.to_owned())
        } else {
            Err(AddressProblem::Unsupported)
        };
    }
    let scheme = if address.contains('@') && !address.contains('/') {
        MAIL_SCHEME
    } else {
        DEFAULT_SCHEME
    };
    Ok(format!("{scheme}{address}"))
}

/// The edit that writes the link over `draft.range`, with the cursor behind
/// it. Without a text a web address stays bare: the view shows it as a link
/// (M209).
#[must_use]
pub fn link_edit(draft: &LinkDraft, text: &str, address: &str) -> TextEdit {
    let text = text.trim();
    let replacement = if text.is_empty() && !address.starts_with(MAIL_SCHEME) {
        address.to_owned()
    } else {
        let label = if text.is_empty() { address } else { text };
        format!("[{}]({})", escaped_label(label), escaped_target(address))
    };
    let end = draft.range.start + replacement.len();
    TextEdit {
        range: draft.range.clone(),
        replacement,
        selection: end..end,
    }
}

/// `label` with its square brackets escaped if they do not pair up, which
/// would end the link text early.
fn escaped_label(label: &str) -> String {
    let mut depth = 0_i32;
    let mut is_balanced = true;
    for character in label.chars() {
        match character {
            '[' => depth += 1,
            ']' => depth -= 1,
            _ => {}
        }
        is_balanced &= depth >= 0;
    }
    if is_balanced && depth == 0 {
        label.to_owned()
    } else {
        label.replace('[', "\\[").replace(']', "\\]")
    }
}

/// `address` with its parentheses encoded if they do not pair up, which would
/// end the link target early.
fn escaped_target(address: &str) -> String {
    let mut depth = 0_i32;
    let mut is_balanced = true;
    for character in address.chars() {
        match character {
            '(' => depth += 1,
            ')' => depth -= 1,
            _ => {}
        }
        is_balanced &= depth >= 0;
    }
    if is_balanced && depth == 0 {
        address.to_owned()
    } else {
        address.replace('(', "%28").replace(')', "%29")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const BODY: &str = "See [wiki](https://a.de \"Title\") and more";

    #[test]
    fn a_cursor_in_a_link_edits_that_link() {
        let draft = link_draft(BODY, 6..6, Some("https://other.de"));
        assert!(draft.is_existing);
        assert_eq!(draft.range, 4..32);
        assert_eq!(draft.text, "wiki");
        assert_eq!(draft.address, "https://a.de");
        assert_eq!(draft.focus, LinkField::Address);
    }

    #[test]
    fn a_selection_becomes_the_link_text() {
        let draft = link_draft("one two\nthree", 4..13, None);
        assert!(!draft.is_existing);
        assert_eq!(draft.range, 4..13);
        assert_eq!(draft.text, "two three");
        assert_eq!(draft.focus, LinkField::Address);
    }

    #[test]
    fn only_an_address_is_taken_from_the_clipboard() {
        let address = |clipboard| link_draft("", 0..0, Some(clipboard)).address;
        assert_eq!(address(" https://a.de\n"), "https://a.de");
        assert_eq!(address("mailto:a@b.de"), "mailto:a@b.de");
        assert_eq!(address("some words"), "");
        assert_eq!(address("https://a.de and more"), "");
        assert_eq!(link_draft("", 0..0, None).focus, LinkField::Text);
    }

    #[test]
    fn addresses_get_the_missing_scheme() {
        for (typed, expected) in [
            ("https://a.de", "https://a.de"),
            ("HTTP://a.de", "HTTP://a.de"),
            (" a.de/b ", "https://a.de/b"),
            ("localhost:8080", "https://localhost:8080"),
            ("a@b.de", "mailto:a@b.de"),
            ("mailto:a@b.de", "mailto:a@b.de"),
        ] {
            assert_eq!(normalized_address(typed).as_deref(), Ok(expected));
        }
    }

    #[test]
    fn unusable_addresses_are_refused() {
        assert_eq!(normalized_address("  "), Err(AddressProblem::Missing));
        for typed in ["ftp://a.de", "a b.de", "<https://a.de>"] {
            assert_eq!(normalized_address(typed), Err(AddressProblem::Unsupported));
        }
    }

    #[test]
    fn the_edit_writes_the_link_and_ends_behind_it() {
        let draft = link_draft("see more", 4..8, None);
        let edit = link_edit(&draft, "more", "https://a.de");
        assert_eq!(edit.range, 4..8);
        assert_eq!(edit.replacement, "[more](https://a.de)");
        assert_eq!(edit.selection, 24..24);
    }

    #[test]
    fn without_a_text_a_web_address_stays_bare() {
        let draft = link_draft("", 0..0, None);
        assert_eq!(
            link_edit(&draft, " ", "https://a.de").replacement,
            "https://a.de"
        );
        assert_eq!(
            link_edit(&draft, "", "mailto:a@b.de").replacement,
            "[mailto:a@b.de](mailto:a@b.de)"
        );
    }

    #[test]
    fn unpaired_brackets_cannot_end_the_link_early() {
        let draft = link_draft("", 0..0, None);
        assert_eq!(
            link_edit(&draft, "a]b", "https://a.de/(c").replacement,
            "[a\\]b](https://a.de/%28c)"
        );
        assert_eq!(
            link_edit(&draft, "[a]", "https://a.de/(c)").replacement,
            "[[a]](https://a.de/(c))"
        );
    }
}
