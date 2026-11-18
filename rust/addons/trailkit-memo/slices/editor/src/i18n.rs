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
    /// Label of the link text field (M208).
    LinkText,
    /// Label of the link address field (M208).
    LinkAddress,
    /// Button that inserts a new link (M208).
    LinkInsert,
    /// Button that saves a changed link (M208).
    LinkSave,
    /// Button that closes the link popup (M208).
    LinkCancel,
    /// Tooltip of the button that turns a link into plain text (M208).
    LinkRemove,
    /// Why a link cannot be inserted without an address (M208).
    LinkAddressMissing,
    /// Why an address cannot be used (M208).
    LinkAddressUnsupported,
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
        Self::LinkText,
        Self::LinkAddress,
        Self::LinkInsert,
        Self::LinkSave,
        Self::LinkCancel,
        Self::LinkRemove,
        Self::LinkAddressMissing,
        Self::LinkAddressUnsupported,
    ];

    fn get(self, language: Language) -> &'static str {
        match (self, language) {
            (Self::LinkText, Language::English | Language::German) => "Text",
            (Self::LinkAddress, Language::English) => "Address",
            (Self::LinkAddress, Language::German) => "Adresse",
            (Self::LinkInsert, Language::English) => "Insert",
            (Self::LinkInsert, Language::German) => "Einfügen",
            (Self::LinkSave, Language::English) => "Save",
            (Self::LinkSave, Language::German) => "Speichern",
            (Self::LinkCancel, Language::English) => "Cancel",
            (Self::LinkCancel, Language::German) => "Abbrechen",
            (Self::LinkRemove, Language::English) => "Remove link, keep the text",
            (Self::LinkRemove, Language::German) => "Link entfernen, Text behalten",
            (Self::LinkAddressMissing, Language::English) => "Enter an address.",
            (Self::LinkAddressMissing, Language::German) => "Gib eine Adresse ein.",
            (Self::LinkAddressUnsupported, Language::English) => {
                "Use an http, https or mailto address."
            }
            (Self::LinkAddressUnsupported, Language::German) => {
                "Nutze eine http-, https- oder mailto-Adresse."
            }
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
