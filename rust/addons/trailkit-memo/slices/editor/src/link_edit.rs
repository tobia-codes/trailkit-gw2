//! What the link dialog edits (memo plan M208): the link at the selection, the
//! check of its address and the markdown it becomes. Platform-independent, so
//! it can be unit-tested on any host (ROADMAP R23).

use std::ops::Range;

use trailkit_ui::markdown::{inline_links, is_openable_link};

use crate::note_editing::TextEdit;

/// The scheme added to an address without one.
const DEFAULT_SCHEME: &str = "https://";

/// The scheme added to an e-mail address.
const MAIL_SCHEME: &str = "mailto:";

/// The field of the link dialog that gets the keyboard focus first.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LinkField {
    /// The link text.
    Text,
    /// The address.
    Address,
}

/// The starting values of the link dialog.
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
    /// It is no valid address, or memo does not open its scheme.
    Invalid,
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
        .filter(|candidate| {
            candidate.contains("://") || candidate.to_ascii_lowercase().starts_with(MAIL_SCHEME)
        })
        .filter(|candidate| normalized_address(candidate).is_ok())
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
/// Returns the [`AddressProblem`] if `address` is empty, is no valid address
/// or cannot be opened.
pub fn normalized_address(address: &str) -> Result<String, AddressProblem> {
    let address = address.trim();
    if address.is_empty() {
        return Err(AddressProblem::Missing);
    }
    let has_scheme =
        address.contains("://") || address.to_ascii_lowercase().starts_with(MAIL_SCHEME);
    let complete = if has_scheme {
        address.to_owned()
    } else if address.contains('@') && !address.contains('/') {
        format!("{MAIL_SCHEME}{address}")
    } else {
        format!("{DEFAULT_SCHEME}{address}")
    };
    if is_openable_link(&complete) && has_valid_destination(&complete) {
        Ok(complete)
    } else {
        Err(AddressProblem::Invalid)
    }
}

/// Whether the complete `address` names a mail box or a web host.
fn has_valid_destination(address: &str) -> bool {
    if address.contains(|character: char| character.is_whitespace() || "<>".contains(character)) {
        return false;
    }
    if address.len() >= MAIL_SCHEME.len()
        && address[..MAIL_SCHEME.len()].eq_ignore_ascii_case(MAIL_SCHEME)
    {
        let mailbox = address[MAIL_SCHEME.len()..]
            .split('?')
            .next()
            .unwrap_or_default();
        return mailbox
            .split_once('@')
            .is_some_and(|(name, host)| !name.is_empty() && is_valid_host(host));
    }
    let Some((_, rest)) = address.split_once("://") else {
        return false;
    };
    let authority = rest.split(['/', '?', '#']).next().unwrap_or_default();
    let host_and_port = authority.rsplit('@').next().unwrap_or_default();
    match host_and_port.split_once(':') {
        Some((host, port)) => {
            is_valid_host(host)
                && !port.is_empty()
                && port.chars().all(|character| character.is_ascii_digit())
        }
        None => is_valid_host(host_and_port),
    }
}

/// Whether `host` is `localhost`, an IPv4 address or a name with a top-level
/// domain, like `wiki.guildwars2.com`.
fn is_valid_host(host: &str) -> bool {
    if host.eq_ignore_ascii_case("localhost") {
        return true;
    }
    let labels: Vec<&str> = host.split('.').collect();
    if labels.len() == 4 && labels.iter().all(|label| label.parse::<u8>().is_ok()) {
        return true;
    }
    let is_valid_label = |label: &&str| {
        !label.is_empty()
            && !label.starts_with('-')
            && !label.ends_with('-')
            && label
                .chars()
                .all(|character| character.is_alphanumeric() || character == '-')
    };
    let has_top_level_domain = labels
        .last()
        .is_some_and(|label| label.chars().count() >= 2 && label.chars().all(char::is_alphabetic));
    labels.len() >= 2 && has_top_level_domain && labels.iter().all(is_valid_label)
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
        assert_eq!(address("a.de"), "");
        assert_eq!(address("https://nonsense"), "");
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
            ("https://127.0.0.1:3000/a", "https://127.0.0.1:3000/a"),
            (
                "wiki.guildwars2.com/wiki/Main?x=1#y",
                "https://wiki.guildwars2.com/wiki/Main?x=1#y",
            ),
            ("müller.de", "https://müller.de"),
        ] {
            assert_eq!(normalized_address(typed).as_deref(), Ok(expected));
        }
    }

    #[test]
    fn unusable_addresses_are_refused() {
        assert_eq!(normalized_address("  "), Err(AddressProblem::Missing));
        for typed in [
            "ftp://a.de",
            "a b.de",
            "<https://a.de>",
            "sdsdsdsdsd",
            "https://sdsdsd",
            "https://",
            "https://.de",
            "https://a..de",
            "https://a.d",
            "https://-a.de",
            "https://a.de:port",
            "a.1",
            "a@b",
            "mailto:a",
            "mailto:@b.de",
        ] {
            assert_eq!(
                normalized_address(typed),
                Err(AddressProblem::Invalid),
                "for {typed:?}"
            );
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
