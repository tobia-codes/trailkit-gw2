//! Finds web and e-mail addresses written as plain text, so the markdown view
//! can show them as links (memo plan M209, M211).

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

/// A web or e-mail address found in plain text.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BareLink {
    /// Where the address stands in the text.
    pub range: Range<usize>,
    /// The link target: the address, with `mailto:` for an e-mail address.
    pub target: String,
}

/// Every bare web and e-mail address of `text`, in text order. An `@` inside
/// a web address does not start an e-mail address.
#[must_use]
pub fn find_bare_links(text: &str) -> Vec<BareLink> {
    let urls = find_bare_urls(text);
    let mut links: Vec<BareLink> = urls
        .iter()
        .map(|range| BareLink {
            range: range.clone(),
            target: text[range.clone()].to_owned(),
        })
        .collect();
    links.extend(
        find_bare_emails(text)
            .into_iter()
            .filter(|email| {
                !urls
                    .iter()
                    .any(|url| url.start < email.end && email.start < url.end)
            })
            .map(|range| BareLink {
                target: format!("mailto:{}", &text[range.clone()]),
                range,
            }),
    );
    links.sort_by_key(|link| link.range.start);
    links
}

/// The byte ranges of every e-mail address in `text`: a name, `@` and a domain
/// with a top-level domain of at least two letters.
fn find_bare_emails(text: &str) -> Vec<Range<usize>> {
    let is_name = |c: char| c.is_alphanumeric() || "._%+-".contains(c);
    let is_domain = |c: char| c.is_alphanumeric() || ".-".contains(c);
    let mut emails = Vec::new();
    for (at, _) in text.match_indices('@') {
        let before = &text[..at];
        let name_start = before
            .char_indices()
            .rev()
            .take_while(|(_, c)| is_name(*c))
            .last()
            .map_or(at, |(index, _)| index);
        let name = before[name_start..].trim_start_matches(['.', '-', '+', '%']);
        let start = at - name.len();
        let after = &text[at + 1..];
        let length = after.find(|c| !is_domain(c)).unwrap_or(after.len());
        let domain = after[..length].trim_end_matches(['.', '-']);
        let has_top_level_domain = domain.rsplit_once('.').is_some_and(|(host, top)| {
            !host.is_empty() && top.chars().count() >= 2 && top.chars().all(char::is_alphabetic)
        });
        let is_clean = !domain.starts_with(['.', '-']) && !domain.contains("..");
        if !name.is_empty() && has_top_level_domain && is_clean {
            emails.push(start..at + 1 + domain.len());
        }
    }
    emails
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

    /// The links found in `text`, as (shown text, target).
    fn links(text: &str) -> Vec<(&str, String)> {
        find_bare_links(text)
            .into_iter()
            .map(|link| (&text[link.range], link.target))
            .collect()
    }

    #[test]
    fn finds_email_addresses() {
        assert_eq!(
            links("Mail bogus@example.com, or (a.b+c@sub.example.de)."),
            [
                ("bogus@example.com", "mailto:bogus@example.com".to_owned()),
                (
                    "a.b+c@sub.example.de",
                    "mailto:a.b+c@sub.example.de".to_owned()
                ),
            ]
        );
    }

    #[test]
    fn ignores_what_only_looks_like_an_email_address() {
        for text in [
            "a@b",
            "@example.com",
            "a@.de",
            "a@b.c",
            "a@b..de",
            "1 @ 2.de",
        ] {
            assert!(links(text).is_empty(), "in {text:?}");
        }
    }

    #[test]
    fn an_at_sign_in_a_web_address_stays_part_of_it() {
        assert_eq!(
            links("https://a.de/u@b.de"),
            [("https://a.de/u@b.de", "https://a.de/u@b.de".to_owned())]
        );
    }

    #[test]
    fn keeps_positions_valid_around_umlauts() {
        assert_eq!(urls("Für https://ä.de/ü."), ["https://ä.de/ü"]);
    }
}
