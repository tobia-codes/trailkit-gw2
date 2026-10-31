//! Localization for trailkit-memo (English + German, AGENTS.md rule 5).
//!
//! Strings are addressed by the typed [`Text`] key, never as inline literals in
//! UI code. Moves into a shared `trailkit-i18n` crate once a second addon needs
//! it (AGENTS.md rule 7).

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
        Self::ALL.into_iter().find(|lang| lang.code() == code)
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

/// A translatable user-facing string.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Text {
    /// Title of the main window.
    MainWindowTitle,
    /// Greeting shown by the phase 0 skeleton window.
    Hello,
    /// Display name of the open/close keybind in the Nexus keybind list.
    KeybindToggle,
}

impl Text {
    /// Returns the translation of this string in `lang`.
    #[must_use]
    pub const fn get(self, lang: Language) -> &'static str {
        match (self, lang) {
            (Self::MainWindowTitle, Language::English) => "Notes",
            (Self::MainWindowTitle, Language::German) => "Notizen",
            (Self::Hello, Language::English) => "Hello from trailkit-memo!",
            (Self::Hello, Language::German) => "Hallo von trailkit-memo!",
            (Self::KeybindToggle, Language::English) => "Open/close notes",
            (Self::KeybindToggle, Language::German) => "Notizen öffnen/schließen",
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
    fn code_round_trips() {
        for lang in Language::ALL {
            assert_eq!(Language::from_code(lang.code()), Some(lang));
            assert_eq!(Language::from_u8(lang as u8), lang);
        }
    }
}
