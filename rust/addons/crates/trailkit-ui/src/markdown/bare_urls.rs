//! Finds web addresses written as plain text, so the markdown view can show
//! them as links (memo plan M209).

use std::ops::Range;

/// The schemes a bare address starts with.
const SCHEMES: [&str; 2] = ["https://", "http://"];

/// Characters that end an address.
const fn ends_address(character: char) -> bool {
    character.is_whitespace() || matches!(character, '<' | '>' | '"' | '`')
}

/// The byte ranges of every bare `http(s)` address in `text`, in text order.
/// Punctuation after an address, like the full stop of a sentence or the
/// closing bracket around it, is left out.
#[must_use]
pub fn find_bare_urls(text: &str) -> Vec<Range<usize>> {
    let mut urls = Vec::new();
    let mut position = 0;
    while let Some((start, scheme)) = next_scheme(text, position) {
        let rest = &text[start..];
        let length = rest.find(ends_address).unwrap_or(rest.len());
        let address = trim_trailing_punctuation(&rest[..length]);
        let starts_a_word = !text[..start]
            .chars()
            .next_back()
            .is_some_and(char::is_alphanumeric);
        if starts_a_word && address.len() > scheme.len() {
            urls.push(start..start + address.len());
        }
        position = start + length.max(scheme.len());
    }
    urls
}

/// The first scheme at or after `position`, with its start.
fn next_scheme(text: &str, position: usize) -> Option<(usize, &'static str)> {
    SCHEMES
        .into_iter()
        .filter_map(|scheme| {
            text[position..]
                .find(scheme)
                .map(|index| (position + index, scheme))
        })
        .min_by_key(|(start, _)| *start)
}

/// `address` without the punctuation that belongs to the sentence around it.
fn trim_trailing_punctuation(mut address: &str) -> &str {
    loop {
        let Some(last) = address.chars().next_back() else {
            return address;
        };
        let belongs_to_sentence = match last {
            '.' | ',' | ';' | ':' | '!' | '?' | '\'' | '*' | '_' | '~' => true,
            ')' => has_more_closing(address, '(', ')'),
            ']' => has_more_closing(address, '[', ']'),
            '}' => has_more_closing(address, '{', '}'),
            _ => false,
        };
        if !belongs_to_sentence {
            return address;
        }
        address = &address[..address.len() - last.len_utf8()];
    }
}

/// Whether `text` closes more brackets than it opens.
fn has_more_closing(text: &str, open: char, close: char) -> bool {
    text.matches(close).count() > text.matches(open).count()
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The addresses found in `text`.
    fn urls(text: &str) -> Vec<&str> {
        find_bare_urls(text)
            .into_iter()
            .map(|range| &text[range])
            .collect()
    }

    #[test]
    fn finds_addresses_in_running_text() {
        assert_eq!(
            urls("See https://a.de/b and http://c.de."),
            ["https://a.de/b", "http://c.de"]
        );
    }

    #[test]
    fn leaves_out_the_punctuation_around_an_address() {
        assert_eq!(urls("(https://a.de)"), ["https://a.de"]);
        assert_eq!(urls("https://a.de/b_(c)!"), ["https://a.de/b_(c)"]);
        assert_eq!(urls("[https://a.de], ok"), ["https://a.de"]);
    }

    #[test]
    fn ignores_text_that_only_looks_like_an_address() {
        assert!(urls("https://").is_empty());
        assert!(urls("xhttps://a.de").is_empty());
        assert!(urls("ftp://a.de www.a.de").is_empty());
    }

    #[test]
    fn keeps_positions_valid_around_umlauts() {
        assert_eq!(urls("Für https://ä.de/ü."), ["https://ä.de/ü"]);
    }
}
