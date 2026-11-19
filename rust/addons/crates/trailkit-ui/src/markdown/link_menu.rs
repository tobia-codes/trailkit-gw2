//! What the right-click menu of links and chat-link chips offers (memo plan
//! M210). Every addon gets the same menu: the target alone decides its
//! entries, so this part is independent of ImGui and testable on any host.

use crate::{
    chat_link::{ChatLinkChip, ChatLinkKind, ChipText},
    markdown::{LinkKind, link_target_label},
    text::icons::Icon,
};

/// What the menu was opened on.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LinkMenuTarget {
    /// A web or e-mail link to this address.
    Link(String),
    /// A chat-link chip.
    Chip {
        /// The whole chat code.
        code: String,
        /// Whether the wiki can have a page for it.
        has_wiki_page: bool,
    },
}

/// What a menu entry does.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LinkMenuCommand {
    /// Opens the link.
    Open,
    /// Copies the address of the link.
    CopyAddress,
    /// Copies the chat code.
    CopyChatLink,
    /// Searches the wiki for the chat code.
    OpenInWiki,
}

/// One entry of the menu.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LinkMenuEntry {
    /// The icon before the label.
    pub icon: Icon,
    /// The label, translated by the addon.
    pub label: ChipText,
    /// What it does.
    pub command: LinkMenuCommand,
}

/// The entry that copies a chat code.
const COPY_CHAT_LINK: LinkMenuEntry = LinkMenuEntry {
    icon: Icon::COPY,
    label: ChipText::CopyChatLink,
    command: LinkMenuCommand::CopyChatLink,
};

impl LinkMenuTarget {
    /// The target for `chip`. Coins, players, builds and damaged codes have
    /// no wiki page to look up.
    #[must_use]
    pub fn chip(chip: &ChatLinkChip) -> Self {
        let has_wiki_page = chip.link.as_ref().is_ok_and(|link| {
            matches!(
                link.kind(),
                ChatLinkKind::Item
                    | ChatLinkKind::MapPoint
                    | ChatLinkKind::Skill
                    | ChatLinkKind::Trait
                    | ChatLinkKind::Recipe
                    | ChatLinkKind::Skin
                    | ChatLinkKind::Outfit
                    | ChatLinkKind::Achievement
            )
        });
        Self::Chip {
            code: chip.code.clone(),
            has_wiki_page,
        }
    }

    /// The menu entries, the action of a left click first.
    #[must_use]
    pub fn entries(&self) -> Vec<LinkMenuEntry> {
        match self {
            Self::Link(url) => {
                let kind = LinkKind::of(url);
                let (open, copy) = match kind {
                    LinkKind::Web => (ChipText::OpenLink, ChipText::CopyLinkAddress),
                    LinkKind::Mail => (ChipText::WriteEmail, ChipText::CopyEmailAddress),
                };
                vec![
                    LinkMenuEntry {
                        icon: kind.icon(),
                        label: open,
                        command: LinkMenuCommand::Open,
                    },
                    LinkMenuEntry {
                        icon: Icon::COPY,
                        label: copy,
                        command: LinkMenuCommand::CopyAddress,
                    },
                ]
            }
            Self::Chip { has_wiki_page, .. } => {
                let mut entries = vec![COPY_CHAT_LINK];
                if *has_wiki_page {
                    entries.push(LinkMenuEntry {
                        icon: LinkKind::Web.icon(),
                        label: ChipText::OpenInWiki,
                        command: LinkMenuCommand::OpenInWiki,
                    });
                }
                entries
            }
        }
    }

    /// The text a copy command puts on the clipboard.
    #[must_use]
    pub fn copied_text(&self) -> &str {
        match self {
            Self::Link(url) => link_target_label(url),
            Self::Chip { code, .. } => code,
        }
    }
}

/// The address that searches the wiki at `search_base` for the chat `code`.
#[must_use]
pub fn wiki_search_url(search_base: &str, code: &str) -> String {
    let mut url = String::with_capacity(search_base.len() + 3 * code.len());
    url.push_str(search_base);
    for byte in code.bytes() {
        if byte.is_ascii_alphanumeric() {
            url.push(char::from(byte));
        } else {
            url.push_str(&format!("%{byte:02X}"));
        }
    }
    url
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The labels of the entries of `target`.
    fn labels(target: &LinkMenuTarget) -> Vec<ChipText> {
        target.entries().iter().map(|entry| entry.label).collect()
    }

    #[test]
    fn links_offer_opening_and_copying_their_address() {
        let web = LinkMenuTarget::Link("https://a.de".to_owned());
        assert_eq!(
            labels(&web),
            [ChipText::OpenLink, ChipText::CopyLinkAddress]
        );
        assert_eq!(web.entries()[0].icon, Icon::EXTERNAL_LINK);
        assert_eq!(web.copied_text(), "https://a.de");
        let mail = LinkMenuTarget::Link("mailto:a@b.de".to_owned());
        assert_eq!(
            labels(&mail),
            [ChipText::WriteEmail, ChipText::CopyEmailAddress]
        );
        assert_eq!(mail.entries()[0].icon, Icon::MAIL);
        assert_eq!(mail.copied_text(), "a@b.de");
    }

    #[test]
    fn only_chips_with_a_wiki_page_offer_the_wiki() {
        let item = LinkMenuTarget::chip(&ChatLinkChip::new("[&AgH1WQAA]"));
        assert_eq!(
            labels(&item),
            [ChipText::CopyChatLink, ChipText::OpenInWiki]
        );
        for code in ["[&AdsnAAA=]", "[&AAAA]"] {
            let target = LinkMenuTarget::chip(&ChatLinkChip::new(code));
            assert_eq!(labels(&target), [ChipText::CopyChatLink], "for {code}");
        }
    }

    #[test]
    fn the_wiki_search_encodes_the_code() {
        assert_eq!(
            wiki_search_url("https://w/?search=", "[&Ag+/=]"),
            "https://w/?search=%5B%26Ag%2B%2F%3D%5D"
        );
    }
}
