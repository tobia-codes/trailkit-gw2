//! The texts of memo's help window (ROADMAP R100, mockup
//! https://claude.ai/artifact/82dwWQcEtFeTfxpQ2ex1cy) in English and German.
//! The help topics themselves are `help/{en,de}.md`.

use trailkit_memo_shared::i18n::{Language, LocalizedText};

/// A string of memo's help window.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HelpText {
    /// The help window's title: `<window title> · <Help>` (R95).
    WindowTitle,
    /// The left tab and the tooltip of the title bar `?`.
    Help,
    /// The right tab, the problem page's title and the GitHub button.
    ReportProblem,
    /// The sentence below the problem page's title.
    ProblemIntro,
    /// Step 1 heading.
    SaveLogStep,
    /// The button of step 1.
    ExportLog,
    /// What the export holds; names [`trailkit_log::EXPORT_DAYS`].
    ExportHint,
    /// Step 2 heading.
    ReportStep,
    /// That reports are public and need a GitHub account.
    ReportHint,
    /// Where the logs and `Nexus.log` are (Q259).
    Footer,
    /// Status after an export; `{file}` is the bare file name.
    ExportSaved,
    /// Status after an export that left older days out; `{file}` is the
    /// bare file name.
    ExportSavedTrimmed,
    /// Status after a failed export (the cause goes to the log).
    ExportFailed,
    /// Tooltip of the disabled export button while there is no log.
    NoLogReason,
    /// Status while there is no log.
    NoLogYet,
    /// The button in memo's Nexus options tab (R53, Q307).
    OpenHelp,
    /// The warning card on the problem page after memo stopped (R104, Q367).
    StoppedNotice,
    /// The line under [`Self::StoppedNotice`] when no draft was kept.
    StoppedDetail,
    /// The line under [`Self::StoppedNotice`] when unsaved text was kept as
    /// a draft (Q360).
    StoppedDetailWithDraft,
    /// The Nexus alert after memo stopped (Q364, Q366).
    StoppedAlert,
    /// The Nexus alert after memo stopped while loading, when there is no
    /// help window to point to (R104).
    StoppedWhileLoadingAlert,
}

impl HelpText {
    /// The text of `self` in `language` with `{file}` replaced by `file`.
    #[must_use]
    pub fn with_file(self, language: Language, file: &str) -> String {
        self.get(language).replace("{file}", file)
    }
}

impl LocalizedText for HelpText {
    const ALL: &'static [Self] = &[
        Self::WindowTitle,
        Self::Help,
        Self::ReportProblem,
        Self::ProblemIntro,
        Self::SaveLogStep,
        Self::ExportLog,
        Self::ExportHint,
        Self::ReportStep,
        Self::ReportHint,
        Self::Footer,
        Self::ExportSaved,
        Self::ExportSavedTrimmed,
        Self::ExportFailed,
        Self::NoLogReason,
        Self::NoLogYet,
        Self::OpenHelp,
        Self::StoppedNotice,
        Self::StoppedDetail,
        Self::StoppedDetailWithDraft,
        Self::StoppedAlert,
        Self::StoppedWhileLoadingAlert,
    ];

    fn get(self, language: Language) -> &'static str {
        match (self, language) {
            (Self::WindowTitle, Language::English) => "Notes · Help",
            (Self::WindowTitle, Language::German) => "Notizen · Hilfe",
            (Self::Help, Language::English) => "Help",
            (Self::Help, Language::German) => "Hilfe",
            (Self::ReportProblem, Language::English) => "Report a problem",
            (Self::ReportProblem, Language::German) => "Problem melden",
            (Self::ProblemIntro, Language::English) => {
                "Nothing is sent automatically. You decide what to share."
            }
            (Self::ProblemIntro, Language::German) => {
                "Es wird nichts automatisch gesendet. Du entscheidest, was du teilst."
            }
            (Self::SaveLogStep, Language::English) => "Save the log",
            (Self::SaveLogStep, Language::German) => "Log speichern",
            (Self::ExportLog, Language::English) => "Export log",
            (Self::ExportLog, Language::German) => "Log exportieren",
            (Self::ExportHint, Language::English) => {
                "Saves the newest 3 days of every trailkit addon as one text file in your \
                 Downloads folder. It holds no notes. Look through it before you attach it."
            }
            (Self::ExportHint, Language::German) => {
                "Speichert die neuesten 3 Tage aller trailkit-Addons als eine Textdatei in \
                 deinem Download-Ordner. Sie enthält keine Notizen. Sieh sie dir an, bevor du \
                 sie anhängst."
            }
            (Self::ReportStep, Language::English) => "Report on GitHub",
            (Self::ReportStep, Language::German) => "Auf GitHub melden",
            (Self::ReportHint, Language::English) => {
                "Opens a form in your browser. Reports are public and need a GitHub account."
            }
            (Self::ReportHint, Language::German) => {
                "Öffnet ein Formular im Browser. Berichte sind öffentlich und brauchen ein \
                 GitHub-Konto."
            }
            (Self::Footer, Language::English) => {
                "Logs are in addons/trailkit/logs. Nexus keeps its own Nexus.log in addons/Nexus."
            }
            (Self::Footer, Language::German) => {
                "Logs liegen in addons/trailkit/logs. Nexus führt eine eigene Nexus.log in \
                 addons/Nexus."
            }
            (Self::ExportSaved, Language::English) => "Saved {file} in Downloads.",
            (Self::ExportSaved, Language::German) => "{file} im Download-Ordner gespeichert.",
            (Self::ExportSavedTrimmed, Language::English) => {
                "Saved {file} in Downloads. Older days were left out to stay under 5 MiB."
            }
            (Self::ExportSavedTrimmed, Language::German) => {
                "{file} im Download-Ordner gespeichert. Ältere Tage wurden weggelassen, damit \
                 die Datei unter 5 MiB bleibt."
            }
            (Self::ExportFailed, Language::English) => {
                "Could not save the log. Check that your Downloads folder can be written to, \
                 then try again."
            }
            (Self::ExportFailed, Language::German) => {
                "Das Log konnte nicht gespeichert werden. Prüfe, ob dein Download-Ordner \
                 beschreibbar ist, und versuche es erneut."
            }
            (Self::NoLogReason, Language::English) => "There is no log yet.",
            (Self::NoLogReason, Language::German) => "Es gibt noch kein Log.",
            (Self::NoLogYet, Language::English) => {
                "There is no log yet. Use the addon for a moment, then try again."
            }
            (Self::NoLogYet, Language::German) => {
                "Es gibt noch kein Log. Nutze das Addon einen Moment und versuche es dann erneut."
            }
            (Self::OpenHelp, Language::English) => "Open help",
            (Self::OpenHelp, Language::German) => "Hilfe öffnen",
            (Self::StoppedNotice, Language::English) => "memo stopped after an error.",
            (Self::StoppedNotice, Language::German) => "memo wurde nach einem Fehler angehalten.",
            (Self::StoppedDetail, Language::English) => {
                "Report the problem here, then reload memo in the Nexus addon list or restart \
                 the game."
            }
            (Self::StoppedDetail, Language::German) => {
                "Melde das Problem hier und lade memo danach in der Addon-Liste von Nexus neu \
                 oder starte das Spiel neu."
            }
            (Self::StoppedDetailWithDraft, Language::English) => {
                "Your unsaved text was kept as a draft. Report the problem here, then reload \
                 memo in the Nexus addon list or restart the game."
            }
            (Self::StoppedDetailWithDraft, Language::German) => {
                "Dein ungespeicherter Text ist als Entwurf gesichert. Melde das Problem hier \
                 und lade memo danach in der Addon-Liste von Nexus neu oder starte das Spiel neu."
            }
            (Self::StoppedAlert, Language::English) => {
                "trailkit memo stopped after an error – details in the help window."
            }
            (Self::StoppedAlert, Language::German) => {
                "trailkit memo wurde nach einem Fehler angehalten – Details im Hilfefenster."
            }
            (Self::StoppedWhileLoadingAlert, Language::English) => {
                "trailkit memo stopped after an error while loading. Reload memo in the Nexus \
                 addon list or restart the game."
            }
            (Self::StoppedWhileLoadingAlert, Language::German) => {
                "trailkit memo wurde beim Laden nach einem Fehler angehalten. Lade memo in der \
                 Addon-Liste von Nexus neu oder starte das Spiel neu."
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use trailkit_log::{EXPORT_DAYS, MAX_EXPORT_BYTES};
    use trailkit_memo_shared::i18n::assert_all_translated;

    use super::*;

    #[test]
    fn every_text_is_translated() {
        assert_all_translated::<HelpText>();
    }

    #[test]
    fn the_hints_name_the_real_export_limits() {
        for language in Language::ALL {
            assert!(
                HelpText::ExportHint
                    .get(language)
                    .contains(&format!(" {EXPORT_DAYS} "))
            );
            let mebibytes = MAX_EXPORT_BYTES / (1024 * 1024);
            assert!(
                HelpText::ExportSavedTrimmed
                    .get(language)
                    .contains(&format!("{mebibytes} MiB"))
            );
        }
    }

    #[test]
    fn status_texts_name_the_file() {
        for language in Language::ALL {
            for text in [HelpText::ExportSaved, HelpText::ExportSavedTrimmed] {
                let status = text.with_file(language, "trailkit-logs-20261112.txt");
                assert!(status.contains("trailkit-logs-20261112.txt"));
                assert!(!status.contains("{file}"));
            }
        }
    }
}
