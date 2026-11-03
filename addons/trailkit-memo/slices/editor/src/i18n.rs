//! The editor slice's user-facing strings in English and German
//! (ROADMAP R76: every slice owns its texts; nexus-addon-dev skill §1).

use trailkit_memo_shared::i18n::{Language, LocalizedText};

/// A translatable string of the editor slice.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EditorText {
    /// Placeholder of the title input (M51).
    TitleHint,
    /// Tooltip of the heading 1 button (M100).
    FormatHeading1,
    /// Tooltip of the heading 2 button (M100).
    FormatHeading2,
    /// Tooltip of the bold button (M53).
    FormatBold,
    /// Tooltip of the italic button (M53).
    FormatItalic,
    /// Tooltip of the bulleted list button (M53).
    FormatBulletList,
    /// Tooltip of the checkbox button (M53).
    FormatCheckbox,
    /// Tooltip of the link button (M53).
    FormatLink,
    /// Tooltip of the disabled event button without trailkit-core (M53).
    InsertEventRequiresCore,
    /// Tooltip of the preview button while the preview is hidden (M60).
    ShowPreview,
    /// Tooltip of the preview button while the preview is shown (M60).
    HidePreview,
}

impl LocalizedText for EditorText {
    const ALL: &'static [Self] = &[
        Self::TitleHint,
        Self::FormatHeading1,
        Self::FormatHeading2,
        Self::FormatBold,
        Self::FormatItalic,
        Self::FormatBulletList,
        Self::FormatCheckbox,
        Self::FormatLink,
        Self::InsertEventRequiresCore,
        Self::ShowPreview,
        Self::HidePreview,
    ];

    fn get(self, language: Language) -> &'static str {
        match (self, language) {
            (Self::TitleHint, Language::English) => "Title",
            (Self::TitleHint, Language::German) => "Titel",
            (Self::FormatHeading1, Language::English) => "Heading 1",
            (Self::FormatHeading1, Language::German) => "Überschrift 1",
            (Self::FormatHeading2, Language::English) => "Heading 2",
            (Self::FormatHeading2, Language::German) => "Überschrift 2",
            (Self::FormatBold, Language::English) => "Bold (Ctrl+B)",
            (Self::FormatBold, Language::German) => "Fett (Strg+B)",
            (Self::FormatItalic, Language::English) => "Italic (Ctrl+I)",
            (Self::FormatItalic, Language::German) => "Kursiv (Strg+I)",
            (Self::FormatBulletList, Language::English) => "Bulleted list",
            (Self::FormatBulletList, Language::German) => "Aufzählung",
            (Self::FormatCheckbox, Language::English) => "Checkbox",
            (Self::FormatCheckbox, Language::German) => "Checkbox",
            (Self::FormatLink, Language::English) => "Link",
            (Self::FormatLink, Language::German) => "Link",
            (Self::InsertEventRequiresCore, Language::English) => {
                "Insert event — requires trailkit-core"
            }
            (Self::InsertEventRequiresCore, Language::German) => {
                "Event einfügen – benötigt trailkit-core"
            }
            (Self::ShowPreview, Language::English) => "Show preview",
            (Self::ShowPreview, Language::German) => "Vorschau einblenden",
            (Self::HidePreview, Language::English) => "Hide preview",
            (Self::HidePreview, Language::German) => "Vorschau ausblenden",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_text_is_translated_and_registered() {
        trailkit_memo_shared::i18n::assert_all_translated::<EditorText>();
    }
}
