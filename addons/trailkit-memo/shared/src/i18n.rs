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

/// A typed set of user-facing strings (ROADMAP R76, question 42a): shared
/// i18n and every slice implement it for their text enum, so registering
/// characters and checking translations work the same everywhere.
pub trait LocalizedText: Sized + Copy + std::fmt::Debug + 'static {
    /// Every string of the set.
    const ALL: &'static [Self];

    /// Returns the translation of this string in `language`.
    fn get(self, language: Language) -> &'static str;

    /// Further characters to register besides the strings themselves, e.g.
    /// the digits and units of a number format. Empty by default.
    #[must_use]
    fn extra_characters() -> String {
        String::new()
    }

    /// All strings in all languages concatenated, plus
    /// [`extra_characters`](Self::extra_characters), so the app can register
    /// every character with Nexus' font atlas (nexus-addon-dev skill §1).
    #[must_use]
    fn all_texts() -> String {
        Self::ALL
            .iter()
            .flat_map(|text| Language::ALL.map(|language| text.get(language)))
            .collect::<String>()
            + &Self::extra_characters()
    }
}

/// Checks that every string of `T` is translated into every language and
/// part of [`LocalizedText::all_texts`]; called from each set's tests.
///
/// # Panics
///
/// Panics naming the first string that is empty or not registered.
pub fn assert_all_translated<T: LocalizedText>() {
    let all = T::all_texts();
    for text in T::ALL {
        for language in Language::ALL {
            assert!(!text.get(language).is_empty(), "{text:?} {language:?}");
            assert!(all.contains(text.get(language)), "{text:?} {language:?}");
        }
    }
}

/// What chat-link chips need from memo (M75): chip texts in `language` and
/// the code copied last (`copied`), shared by note view and editor.
#[cfg(windows)]
#[must_use]
pub fn chip_context(
    language: Language,
    copied: Option<&str>,
) -> trailkit_ui::chat_link_chip::ChipContext<'_> {
    let text: &'static dyn Fn(ChipText) -> &'static str = match language {
        Language::English => &|text| chip_text(text, Language::English),
        Language::German => &|text| chip_text(text, Language::German),
    };
    trailkit_ui::chat_link_chip::ChipContext { text, copied }
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
    /// Tooltip of the title bar chevron while the window is expanded (R71).
    CollapseWindow,
    /// Tooltip of the title bar chevron while the window is collapsed.
    ExpandWindow,
    /// Tooltip of the title bar close button.
    CloseWindow,
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
    /// Label of the primary edit button (M51; a brass text button, R71).
    EditNote,
    /// Button committing an edit (M51).
    Save,
    /// Tooltip of the "Save" button with its shortcuts (M51, M100).
    SaveTooltip,
    /// Button cancelling an edit (M51).
    Cancel,
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
    /// Tooltip of the preview button while the preview is hidden (M60).
    ShowPreview,
    /// Tooltip of the preview button while the preview is shown (M60).
    HidePreview,
    /// Label of the text size shown while changing it (M80).
    TextSize,
    /// Context menu submenu that moves a note or group (M112).
    MoveTo,
    /// "Move to" target: the root level of the note list (M112).
    TopLevel,
}

impl LocalizedText for Text {
    const ALL: &'static [Self] = &[
        Self::MainWindowTitle,
        Self::CollapseWindow,
        Self::ExpandWindow,
        Self::CloseWindow,
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
        Self::Save,
        Self::SaveTooltip,
        Self::Cancel,
        Self::TitleHint,
        Self::FormatHeading1,
        Self::FormatHeading2,
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
        Self::ShowPreview,
        Self::HidePreview,
        Self::TextSize,
        Self::MoveTo,
        Self::TopLevel,
    ];

    /// Chat-link chip texts and the characters of the number and time
    /// formats.
    fn extra_characters() -> String {
        ChipText::all()
            .flat_map(|text| Language::ALL.map(|language| chip_text(text, language)))
            .chain([
                CHIP_DETAIL_CHARACTERS,
                PERCENT_CHARACTERS,
                COUNTDOWN_CHARACTERS,
            ])
            .collect()
    }

    fn get(self, language: Language) -> &'static str {
        match (self, language) {
            (Self::MainWindowTitle, Language::English) => "Notes",
            (Self::MainWindowTitle, Language::German) => "Notizen",
            (Self::CollapseWindow, Language::English) => "Collapse",
            (Self::CollapseWindow, Language::German) => "Einklappen",
            (Self::ExpandWindow, Language::English) => "Expand",
            (Self::ExpandWindow, Language::German) => "Ausklappen",
            (Self::CloseWindow, Language::English) => "Close",
            (Self::CloseWindow, Language::German) => "Schließen",
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
            (Self::EditNote, Language::English) => "Edit",
            (Self::EditNote, Language::German) => "Bearbeiten",
            (Self::Save, Language::English) => "Save",
            (Self::Save, Language::German) => "Speichern",
            (Self::SaveTooltip, Language::English) => "Save changes (Ctrl+Enter, Ctrl+S)",
            (Self::SaveTooltip, Language::German) => "Änderungen speichern (Strg+Enter, Strg+S)",
            (Self::Cancel, Language::English) => "Cancel",
            (Self::Cancel, Language::German) => "Abbrechen",
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
            (Self::ShowPreview, Language::English) => "Show preview",
            (Self::ShowPreview, Language::German) => "Vorschau einblenden",
            (Self::HidePreview, Language::English) => "Hide preview",
            (Self::HidePreview, Language::German) => "Vorschau ausblenden",
            (Self::TextSize, Language::English) => "Text size",
            (Self::TextSize, Language::German) => "Textgröße",
            (Self::MoveTo, Language::English) => "Move to",
            (Self::MoveTo, Language::German) => "Verschieben nach",
            (Self::TopLevel, Language::English) => "Top level",
            (Self::TopLevel, Language::German) => "Oberste Ebene",
        }
    }
}

/// Characters of [`label_with_percent`] besides the label and the digits.
const PERCENT_CHARACTERS: &str = ":.,% ";

/// `label` and a percentage in the conventions of `language`, e.g.
/// "Text size: 112.5%" or "Textgröße: 112,5 %" (M80).
#[must_use]
pub fn label_with_percent(label: &str, percent: f32, language: Language) -> String {
    let mut number = format!("{percent:.1}");
    if number.ends_with(".0") {
        number.truncate(number.len() - 2);
    }
    match language {
        Language::English => format!("{label}: {number}%"),
        Language::German => format!("{label}: {} %", number.replace('.', ",")),
    }
}

/// Characters of [`format_local_time`] and of the slices' number formats
/// (e.g. the reset countdown) besides words that are already in a text.
const COUNTDOWN_CHARACTERS: &str = "0123456789 hminTd.,:-";

/// `time` as a date and time in the player's local time zone, formatted for
/// `language` (`data-formatting`), e.g. "2026-11-03 01:00" or
/// "03.11.2026, 01:00".
///
/// The local offset comes from the operating system (ROADMAP R72: jiff ships
/// without a time-zone database); under Wine/Proton, Wine derives it from the
/// Linux host's zone.
#[must_use]
pub fn format_local_time(time: jiff::Timestamp, language: Language) -> String {
    let pattern = match language {
        Language::German => "%d.%m.%Y, %H:%M",
        Language::English => "%Y-%m-%d %H:%M",
    };
    time.to_zoned(jiff::tz::TimeZone::fixed(local_utc_offset(time)))
        .strftime(pattern)
        .to_string()
}

/// The player's UTC offset at `time`, daylight saving time included, as
/// Windows' current time-zone settings define it. Falls back to UTC (and logs
/// a warning) if Windows cannot convert the time.
#[cfg(windows)]
fn local_utc_offset(time: jiff::Timestamp) -> jiff::tz::Offset {
    use windows::Win32::{Foundation::SYSTEMTIME, System::Time::SystemTimeToTzSpecificLocalTime};

    let utc = time.to_zoned(jiff::tz::TimeZone::UTC).datetime();
    let Some(utc_system_time) = system_time_from_civil(utc) else {
        return jiff::tz::Offset::UTC;
    };
    let mut local_system_time = SYSTEMTIME::default();
    // SAFETY: both pointers refer to live, properly aligned `SYSTEMTIME`
    // values for the duration of the call; `None` selects the current
    // time-zone settings.
    let conversion = unsafe {
        SystemTimeToTzSpecificLocalTime(
            None,
            &raw const utc_system_time,
            &raw mut local_system_time,
        )
    };
    if let Err(error) = conversion {
        log::warn!("local time conversion failed, showing UTC: {error}");
        return jiff::tz::Offset::UTC;
    }
    civil_from_system_time(local_system_time)
        .and_then(|local| local.duration_since(utc).as_secs().try_into().ok())
        .and_then(|seconds| jiff::tz::Offset::from_seconds(seconds).ok())
        .unwrap_or(jiff::tz::Offset::UTC)
}

/// Without Windows (unit tests on Linux) there are no time-zone settings to
/// ask: the offset is always UTC.
#[cfg(not(windows))]
fn local_utc_offset(_time: jiff::Timestamp) -> jiff::tz::Offset {
    jiff::tz::Offset::UTC
}

/// `date_time` as a Windows `SYSTEMTIME` (milliseconds dropped), or `None`
/// for years Windows cannot represent.
#[cfg(windows)]
fn system_time_from_civil(
    date_time: jiff::civil::DateTime,
) -> Option<windows::Win32::Foundation::SYSTEMTIME> {
    Some(windows::Win32::Foundation::SYSTEMTIME {
        wYear: u16::try_from(date_time.year()).ok()?,
        wMonth: u16::try_from(date_time.month()).ok()?,
        wDayOfWeek: u16::try_from(date_time.weekday().to_sunday_zero_offset()).ok()?,
        wDay: u16::try_from(date_time.day()).ok()?,
        wHour: u16::try_from(date_time.hour()).ok()?,
        wMinute: u16::try_from(date_time.minute()).ok()?,
        wSecond: u16::try_from(date_time.second()).ok()?,
        wMilliseconds: 0,
    })
}

/// A Windows `SYSTEMTIME` as a civil date and time, or `None` if its fields
/// are out of range.
#[cfg(windows)]
fn civil_from_system_time(
    system_time: windows::Win32::Foundation::SYSTEMTIME,
) -> Option<jiff::civil::DateTime> {
    jiff::civil::DateTime::new(
        i16::try_from(system_time.wYear).ok()?,
        i8::try_from(system_time.wMonth).ok()?,
        i8::try_from(system_time.wDay).ok()?,
        i8::try_from(system_time.wHour).ok()?,
        i8::try_from(system_time.wMinute).ok()?,
        i8::try_from(system_time.wSecond).ok()?,
        0,
    )
    .ok()
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
    fn every_text_is_translated_and_registered() {
        assert_all_translated::<Text>();
    }

    #[test]
    fn code_round_trips() {
        for language in Language::ALL {
            assert_eq!(Language::from_code(language.code()), Some(language));
            assert_eq!(Language::from_u8(language as u8), language);
        }
    }

    #[test]
    fn percentages_follow_the_language() {
        assert_eq!(
            label_with_percent("Text size", 112.5, Language::English),
            "Text size: 112.5%"
        );
        assert_eq!(
            label_with_percent("Textgröße", 125.0, Language::German),
            "Textgröße: 125 %"
        );
    }
}
