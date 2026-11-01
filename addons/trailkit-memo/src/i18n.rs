//! Localization for trailkit-memo (English + German, AGENTS.md rule 5).
//!
//! Strings are addressed by the typed [`Text`] key, never as inline literals in
//! UI code. Moves into a shared `trailkit-i18n` crate once a second addon needs
//! it (AGENTS.md rule 7).

use trailkit_ui::chat_link::{CHIP_DETAIL_CHARACTERS, ChatLinkKind, ChipText, Profession};

/// A user interface language supported by the addon.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[repr(u8)]
pub enum Language {
    /// English — the fallback language.
    #[default]
    English = 0,
    /// German.
    German = 1,
}

impl Language {
    /// Every supported language, in a stable order.
    pub const ALL: [Self; 2] = [Self::English, Self::German];

    /// Returns the language code Nexus uses for this language (`"en"`, `"de"`).
    #[must_use]
    pub const fn code(self) -> &'static str {
        match self {
            Self::English => "en",
            Self::German => "de",
        }
    }

    /// Parses a Nexus language code; returns `None` for unsupported languages.
    #[must_use]
    pub fn from_code(code: &str) -> Option<Self> {
        Self::ALL
            .into_iter()
            .find(|language| language.code() == code)
    }

    /// Converts the `repr(u8)` discriminant back into a language, falling back
    /// to English for unknown values (used for lock-free atomic storage).
    #[must_use]
    pub const fn from_u8(value: u8) -> Self {
        match value {
            1 => Self::German,
            _ => Self::English,
        }
    }

    /// Resolves the effective language (nexus-addon-dev skill §4): the language
    /// chosen in the addon settings, else the active Nexus language, else English.
    #[must_use]
    pub fn resolve(setting: Option<Self>, nexus_code: Option<&str>) -> Self {
        setting
            .or_else(|| nexus_code.and_then(Self::from_code))
            .unwrap_or_default()
    }
}

/// The translation of a chat-link chip text (memo plan M75).
#[must_use]
pub const fn chip_text(text: ChipText, language: Language) -> &'static str {
    let german = matches!(language, Language::German);
    match text {
        ChipText::Kind(kind) => match (kind, german) {
            (ChatLinkKind::Coins, false) => "Coins",
            (ChatLinkKind::Coins, true) => "Münzen",
            (ChatLinkKind::Item, false) => "Item",
            (ChatLinkKind::Item, true) => "Gegenstand",
            (ChatLinkKind::NpcText, false) => "NPC text",
            (ChatLinkKind::NpcText, true) => "NPC-Text",
            (ChatLinkKind::MapPoint, false) => "Map point",
            (ChatLinkKind::MapPoint, true) => "Kartenpunkt",
            (ChatLinkKind::Skill, false) => "Skill",
            (ChatLinkKind::Skill, true) => "Fertigkeit",
            (ChatLinkKind::Trait, false) => "Trait",
            (ChatLinkKind::Trait, true) => "Eigenschaft",
            (ChatLinkKind::Player, false) => "Player",
            (ChatLinkKind::Player, true) => "Spieler",
            (ChatLinkKind::Recipe, false) => "Recipe",
            (ChatLinkKind::Recipe, true) => "Rezept",
            (ChatLinkKind::Skin, _) => "Skin",
            (ChatLinkKind::Outfit, _) => "Outfit",
            (ChatLinkKind::WvwObjective, false) => "WvW objective",
            (ChatLinkKind::WvwObjective, true) => "WvW-Ziel",
            (ChatLinkKind::BuildTemplate, _) => "Build",
            (ChatLinkKind::Achievement, false) => "Achievement",
            (ChatLinkKind::Achievement, true) => "Erfolg",
        },
        ChipText::Profession(profession) => match (profession, german) {
            (Profession::Guardian, false) => "Guardian",
            (Profession::Guardian, true) => "Wächter",
            (Profession::Warrior, false) => "Warrior",
            (Profession::Warrior, true) => "Krieger",
            (Profession::Engineer, false) => "Engineer",
            (Profession::Engineer, true) => "Ingenieur",
            (Profession::Ranger, false) => "Ranger",
            (Profession::Ranger, true) => "Waldläufer",
            (Profession::Thief, false) => "Thief",
            (Profession::Thief, true) => "Dieb",
            (Profession::Elementalist, false) => "Elementalist",
            (Profession::Elementalist, true) => "Elementarmagier",
            (Profession::Mesmer, _) => "Mesmer",
            (Profession::Necromancer, false) => "Necromancer",
            (Profession::Necromancer, true) => "Nekromant",
            (Profession::Revenant, false) => "Revenant",
            (Profession::Revenant, true) => "Widergänger",
        },
        ChipText::InvalidChatLink if german => "Ungültiger Chat-Link",
        ChipText::InvalidChatLink => "Invalid chat link",
        ChipText::UnknownLinkType if german => "Unbekannter Link-Typ",
        ChipText::UnknownLinkType => "Unknown link type",
        ChipText::DamagedCode if german => "Der Code ist beschädigt",
        ChipText::DamagedCode => "The code is damaged",
        ChipText::ClickToCopy if german => "Klicken zum Kopieren",
        ChipText::ClickToCopy => "Click to copy",
        ChipText::Copied if german => "Kopiert",
        ChipText::Copied => "Copied",
    }
}

/// A translatable user-facing string.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Text {
    /// Title of the main window.
    MainWindowTitle,
    /// Display name of the open/close keybind in the Nexus keybind list.
    KeybindToggle,
    /// Tooltip of the quick access icon.
    QuickAccessTooltip,
    /// Shown while the notes are being loaded from disk.
    Loading,
    /// `notes.json` was unreadable and moved away; followed by the new location.
    NotesRecovered,
    /// `notes.json` was written by a newer addon version.
    NotesNewerVersion,
    /// `notes.json` exists but cannot be accessed.
    NotesInaccessible,
    /// Button opening the addon folder in the file explorer.
    OpenFolder,
    /// Tooltip of the "new note" button and title of a new note.
    NewNote,
    /// Tooltip of the "new group" button, empty-state button and title of a new group.
    NewGroup,
    /// Context menu entry to rename a note or group.
    Rename,
    /// Empty state headline when there are no notes.
    NoNotesYet,
    /// Primary empty-state button.
    CreateFirstNote,
    /// Tooltip of the menu button while the note list is hidden (M48).
    ShowNoteList,
    /// Tooltip of the menu button while the note list is shown (M48).
    HideNoteList,
    /// Tooltip of the edit button (M51).
    EditNote,
    /// Button committing an edit (M51).
    Done,
    /// Tooltip of the "Done" button with its shortcut (M51).
    DoneTooltip,
    /// Button cancelling an edit (M51).
    Cancel,
    /// Placeholder of the title input (M51).
    TitleHint,
    /// Tooltip of the heading button (M53).
    FormatHeading,
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
    /// Title of the unsaved changes dialog (M54).
    UnsavedChangesTitle,
    /// Text of the unsaved changes dialog (M54).
    UnsavedChangesQuestion,
    /// Keep the changes (unsaved changes dialog).
    KeepChanges,
    /// Throw the changes away (unsaved changes dialog, draft hint).
    DiscardChanges,
    /// Stay in the editor (unsaved changes dialog).
    KeepEditing,
    /// Draft hint, followed by the date and time of the draft (M54).
    DraftFound,
    /// Draft hint addition when the note changed after the draft (M54).
    DraftNoteChangedSince,
    /// Restore a draft into the editor (M54).
    RestoreDraft,
    /// Tooltip of the preview button while the preview is hidden (M60).
    ShowPreview,
    /// Tooltip of the preview button while the preview is shown (M60).
    HidePreview,
    /// Shown instead of an empty note's content.
    EmptyNote,
    /// Shown in the note pane when no note is selected.
    SelectNote,
    /// Letter on the badge of daily reset notes.
    DailyBadge,
    /// Letter on the badge of weekly reset notes.
    WeeklyBadge,
    /// Tooltip of the daily badge.
    DailyTooltip,
    /// Tooltip of the weekly badge.
    WeeklyTooltip,
}

impl Text {
    /// Every string, for registering their characters with Nexus' font atlas.
    pub const ALL: [Self; 43] = [
        Self::MainWindowTitle,
        Self::KeybindToggle,
        Self::QuickAccessTooltip,
        Self::Loading,
        Self::NotesRecovered,
        Self::NotesNewerVersion,
        Self::NotesInaccessible,
        Self::OpenFolder,
        Self::NewNote,
        Self::NewGroup,
        Self::Rename,
        Self::NoNotesYet,
        Self::CreateFirstNote,
        Self::ShowNoteList,
        Self::HideNoteList,
        Self::EditNote,
        Self::Done,
        Self::DoneTooltip,
        Self::Cancel,
        Self::TitleHint,
        Self::FormatHeading,
        Self::FormatBold,
        Self::FormatItalic,
        Self::FormatBulletList,
        Self::FormatCheckbox,
        Self::FormatLink,
        Self::InsertEventRequiresCore,
        Self::UnsavedChangesTitle,
        Self::UnsavedChangesQuestion,
        Self::KeepChanges,
        Self::DiscardChanges,
        Self::KeepEditing,
        Self::DraftFound,
        Self::DraftNoteChangedSince,
        Self::RestoreDraft,
        Self::ShowPreview,
        Self::HidePreview,
        Self::EmptyNote,
        Self::SelectNote,
        Self::DailyBadge,
        Self::WeeklyBadge,
        Self::DailyTooltip,
        Self::WeeklyTooltip,
    ];

    /// All strings in all languages concatenated, so Nexus puts every
    /// character they use into the font atlas (nexus-addon-dev skill §1).
    #[must_use]
    pub fn all_texts() -> String {
        let chip_texts = ChipText::all()
            .flat_map(|text| Language::ALL.map(|language| chip_text(text, language)));
        Self::ALL
            .iter()
            .flat_map(|text| Language::ALL.map(|language| text.get(language)))
            .chain(chip_texts)
            .chain([CHIP_DETAIL_CHARACTERS])
            .collect()
    }

    /// Returns the translation of this string in `language`.
    #[must_use]
    pub const fn get(self, language: Language) -> &'static str {
        match (self, language) {
            (Self::MainWindowTitle, Language::English) => "Notes",
            (Self::MainWindowTitle, Language::German) => "Notizen",
            (Self::KeybindToggle, Language::English) => "Open/close notes",
            (Self::KeybindToggle, Language::German) => "Notizen öffnen/schließen",
            (Self::QuickAccessTooltip, Language::English) => "Notes",
            (Self::QuickAccessTooltip, Language::German) => "Notizen",
            (Self::Loading, Language::English) => "Loading…",
            (Self::Loading, Language::German) => "Lädt…",
            (Self::NotesRecovered, Language::English) => {
                "Your notes could not be loaded. The damaged file was kept in the addon folder as:"
            }
            (Self::NotesRecovered, Language::German) => {
                "Deine Notizen konnten nicht geladen werden. Die beschädigte Datei wurde im Addon-Ordner gesichert als:"
            }
            (Self::NotesNewerVersion, Language::English) => {
                "Your notes were saved by a newer version of trailkit-memo. Please update the addon — until then, changes are not saved."
            }
            (Self::NotesNewerVersion, Language::German) => {
                "Deine Notizen wurden von einer neueren Version von trailkit-memo gespeichert. Bitte aktualisiere das Addon — bis dahin werden Änderungen nicht gespeichert."
            }
            (Self::NotesInaccessible, Language::English) => {
                "Your notes could not be read, so changes are not saved. Details are in the Nexus log."
            }
            (Self::NotesInaccessible, Language::German) => {
                "Deine Notizen konnten nicht gelesen werden, daher werden Änderungen nicht gespeichert. Details stehen im Nexus-Log."
            }
            (Self::OpenFolder, Language::English) => "Open folder",
            (Self::OpenFolder, Language::German) => "Ordner öffnen",
            (Self::NewNote, Language::English) => "New note",
            (Self::NewNote, Language::German) => "Neue Notiz",
            (Self::NewGroup, Language::English) => "New group",
            (Self::NewGroup, Language::German) => "Neue Gruppe",
            (Self::Rename, Language::English) => "Rename",
            (Self::Rename, Language::German) => "Umbenennen",
            (Self::NoNotesYet, Language::English) => "No notes yet",
            (Self::NoNotesYet, Language::German) => "Noch keine Notizen",
            (Self::CreateFirstNote, Language::English) => "Create your first note",
            (Self::CreateFirstNote, Language::German) => "Erste Notiz anlegen",
            (Self::ShowNoteList, Language::English) => "Show note list",
            (Self::ShowNoteList, Language::German) => "Notizliste einblenden",
            (Self::HideNoteList, Language::English) => "Hide note list",
            (Self::HideNoteList, Language::German) => "Notizliste ausblenden",
            (Self::EditNote, Language::English) => "Edit note",
            (Self::EditNote, Language::German) => "Notiz bearbeiten",
            (Self::Done, Language::English) => "Done",
            (Self::Done, Language::German) => "Fertig",
            (Self::DoneTooltip, Language::English) => "Save changes (Ctrl+Enter)",
            (Self::DoneTooltip, Language::German) => "Änderungen übernehmen (Strg+Enter)",
            (Self::Cancel, Language::English) => "Cancel",
            (Self::Cancel, Language::German) => "Abbrechen",
            (Self::TitleHint, Language::English) => "Title",
            (Self::TitleHint, Language::German) => "Titel",
            (Self::FormatHeading, Language::English) => "Heading",
            (Self::FormatHeading, Language::German) => "Überschrift",
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
            (Self::UnsavedChangesTitle, Language::English) => "Unsaved changes",
            (Self::UnsavedChangesTitle, Language::German) => "Ungespeicherte Änderungen",
            (Self::UnsavedChangesQuestion, Language::English) => {
                "This note has changes that are not saved yet."
            }
            (Self::UnsavedChangesQuestion, Language::German) => {
                "Diese Notiz hat Änderungen, die noch nicht übernommen wurden."
            }
            (Self::KeepChanges, Language::English) => "Save",
            (Self::KeepChanges, Language::German) => "Übernehmen",
            (Self::DiscardChanges, Language::English) => "Discard",
            (Self::DiscardChanges, Language::German) => "Verwerfen",
            (Self::KeepEditing, Language::English) => "Keep editing",
            (Self::KeepEditing, Language::German) => "Weiter bearbeiten",
            (Self::DraftFound, Language::English) => "Unsaved changes from",
            (Self::DraftFound, Language::German) => "Ungespeicherte Änderungen vom",
            (Self::DraftNoteChangedSince, Language::English) => "The note was changed since.",
            (Self::DraftNoteChangedSince, Language::German) => "Die Notiz wurde seitdem geändert.",
            (Self::RestoreDraft, Language::English) => "Restore",
            (Self::RestoreDraft, Language::German) => "Wiederherstellen",
            (Self::ShowPreview, Language::English) => "Show preview",
            (Self::ShowPreview, Language::German) => "Vorschau einblenden",
            (Self::HidePreview, Language::English) => "Hide preview",
            (Self::HidePreview, Language::German) => "Vorschau ausblenden",
            (Self::EmptyNote, Language::English) => "This note is empty.",
            (Self::EmptyNote, Language::German) => "Diese Notiz ist leer.",
            (Self::SelectNote, Language::English) => "Select a note in the list.",
            (Self::SelectNote, Language::German) => "Wähle eine Notiz in der Liste aus.",
            (Self::DailyBadge, Language::English) => "D",
            (Self::DailyBadge, Language::German) => "T",
            (Self::WeeklyBadge, Language::English | Language::German) => "W",
            (Self::DailyTooltip, Language::English) => "Daily note — resets at 00:00 UTC",
            (Self::DailyTooltip, Language::German) => {
                "Tägliche Notiz – setzt sich um 00:00 UTC zurück"
            }
            (Self::WeeklyTooltip, Language::English) => {
                "Weekly note — resets on Mondays at 07:30 UTC"
            }
            (Self::WeeklyTooltip, Language::German) => {
                "Wöchentliche Notiz – setzt sich montags um 07:30 UTC zurück"
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn setting_wins_over_nexus_language() {
        assert_eq!(
            Language::resolve(Some(Language::German), Some("en")),
            Language::German
        );
    }

    #[test]
    fn nexus_language_used_without_setting() {
        assert_eq!(Language::resolve(None, Some("de")), Language::German);
    }

    #[test]
    fn unsupported_or_missing_language_falls_back_to_english() {
        assert_eq!(Language::resolve(None, Some("fr")), Language::English);
        assert_eq!(Language::resolve(None, None), Language::English);
    }

    #[test]
    fn every_character_is_in_the_subset_fonts() {
        use trailkit_ui::fonts::{
            OPEN_SANS_BOLD, OPEN_SANS_BOLD_ITALIC, OPEN_SANS_ITALIC, OPEN_SANS_REGULAR, covers,
        };
        // UI strings plus the help pages (rendered with the same fonts).
        let texts =
            Text::all_texts() + include_str!("../help/en.md") + include_str!("../help/de.md");
        for font in [
            OPEN_SANS_REGULAR,
            OPEN_SANS_BOLD,
            OPEN_SANS_ITALIC,
            OPEN_SANS_BOLD_ITALIC,
        ] {
            let missing: String = texts
                .chars()
                .filter(|character| !character.is_control() && !covers(font, *character))
                .collect();
            assert!(
                missing.is_empty(),
                "missing in the subset fonts (run `cargo xtask subset-fonts`): {missing}"
            );
        }
    }

    #[test]
    fn every_text_is_translated_and_registered() {
        let all = Text::all_texts();
        for text in Text::ALL {
            for language in Language::ALL {
                assert!(!text.get(language).is_empty(), "{text:?} {language:?}");
                assert!(all.contains(text.get(language)));
            }
        }
    }

    #[test]
    fn code_round_trips() {
        for language in Language::ALL {
            assert_eq!(Language::from_code(language.code()), Some(language));
            assert_eq!(Language::from_u8(language as u8), language);
        }
    }
}
