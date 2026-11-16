//! Snapshots of the help window (ROADMAP R95, R100) in every state of both
//! tabs, Dark and Light, at the minimum width, and in German: reviewed
//! against the design system (`tokens.md` §10) and the approved mockup.
//!
//! Run from WSL with `cargo xtask ui-test` (Windows only).
#![cfg(windows)]

use nexus::imgui::{Condition, Ui, Window};
use trailkit_ui::{
    button::ButtonState,
    chat_link::ChipText,
    chat_link_chip::ChipContext,
    help_window::{
        HelpTab, HelpWindow, HelpWindowContent, HelpWindowTexts, ProblemNotice, ProblemPage,
    },
    status_line::Status,
    style_scope::StyleScope,
    theme::ThemeKind,
    title_bar::TitleBarTexts,
};
use trailkit_ui_testing::{
    harness::{FrameInput, UiHarness},
    snapshot::assert_snapshot,
};

/// Size of the rendered images.
const IMAGE_WIDTH: f32 = 640.0;

/// Height of the rendered images.
const HEIGHT: f32 = 520.0;

/// The frames a state is held so that hover and layout settle.
const SETTLE_FRAMES: usize = 3;

/// The English help text of the mockup (first topic in full).
const HELP_EN: &str = "## Getting started\nOpen the window, move it and find the buttons.\n\n### Open the window\nPress the keybind you set in Nexus, or choose the addon in the Nexus menu. Drag the title bar to move the window.\n\n### The title bar\nThe menu button opens the window menu. The arrow collapses the window, the cross closes it.\n\n## Notes and groups\nx\n\n## Editing notes\nx\n\n## Daily and weekly notes\nx\n\n## Searching\nx\n\n## Trash and undo\nx\n\n## Keyboard shortcuts\nx\n\n## Where your notes are stored\nx\n\n## Privacy\nx\n";

/// The German help text of the mockup.
const HELP_DE: &str = "## Erste Schritte\nÖffne das Fenster, verschiebe es und finde die Schaltflächen.\n\n### Das Fenster öffnen\nDrücke die Tastenkombination, die du in Nexus festgelegt hast, oder wähle das Addon im Nexus-Menü. Zieh die Titelleiste, um das Fenster zu verschieben.\n\n### Die Titelleiste\nDie Menü-Schaltfläche öffnet das Fenstermenü. Der Pfeil klappt das Fenster ein, das Kreuz schließt es.\n\n## Notizen und Gruppen\nx\n\n## Notizen bearbeiten\nx\n\n## Tägliche und wöchentliche Notizen\nx\n\n## Suchen\nx\n\n## Papierkorb und Rückgängig\nx\n\n## Tastenkürzel\nx\n\n## Wo deine Notizen liegen\nx\n\n## Datenschutz\nx\n";

/// The English texts of the mockup.
const TEXTS_EN: HelpWindowTexts<'static> = HelpWindowTexts {
    title_bar: TitleBarTexts {
        title: "Notes · Help",
        collapse: "Collapse",
        expand: "Expand",
        close: "Close",
    },
    help_tab: "Help",
    problem_tab: "Report a problem",
    problem_intro: "Nothing is sent automatically. You decide what to share.",
    save_log_step: "Save the log",
    export_log: "Export log",
    export_hint: "Saves the newest 3 days of every trailkit addon as one text file in your Downloads folder. It holds no notes. Look through it before you attach it.",
    report_step: "Report on GitHub",
    report_button: "Report a problem",
    report_hint: "Opens a form in your browser. Reports are public and need a GitHub account.",
    footer: "Logs are in addons/trailkit/logs. Nexus keeps its own Nexus.log in addons/Nexus.",
};

/// The German texts of the mockup.
const TEXTS_DE: HelpWindowTexts<'static> = HelpWindowTexts {
    title_bar: TitleBarTexts {
        title: "Notizen · Hilfe",
        collapse: "Einklappen",
        expand: "Ausklappen",
        close: "Schließen",
    },
    help_tab: "Hilfe",
    problem_tab: "Problem melden",
    problem_intro: "Es wird nichts automatisch gesendet. Du entscheidest, was du teilst.",
    save_log_step: "Log speichern",
    export_log: "Log exportieren",
    export_hint: "Speichert die neuesten 3 Tage aller trailkit-Addons als eine Textdatei in deinem Download-Ordner. Sie enthält keine Notizen. Sieh sie dir an, bevor du sie anhängst.",
    report_step: "Auf GitHub melden",
    report_button: "Problem melden",
    report_hint: "Öffnet ein Formular im Browser. Berichte sind öffentlich und brauchen ein GitHub-Konto.",
    footer: "Logs liegen in addons/trailkit/logs. Nexus führt eine eigene Nexus.log in addons/Nexus.",
};

/// One snapshot: language, tab and the problem page.
struct State {
    /// Snapshot name between `help-window-` and the theme.
    name: &'static str,
    /// German instead of English.
    german: bool,
    /// The shown tab.
    tab: HelpTab,
    /// The export button.
    export_button: ButtonState<'static>,
    /// The status line.
    status: Option<(Status, &'static str)>,
    /// The warning card above the steps (R104).
    notice: Option<ProblemNotice<'static>>,
    /// Where the mouse is.
    mouse: [f32; 2],
}

/// Somewhere outside every control.
const AWAY: [f32; 2] = [2.0, HEIGHT - 2.0];

/// The gap between topic list and topic at the minimum width (R101).
const SPLITTER: [f32; 2] = [144.0, 300.0];

/// Every state of both tabs (R100 Q302) in English, the long German texts,
/// a hovered tab, the hovered splitter of the topic list (R101) and the
/// warning card after the addon stopped, with the long German draft text (R104).
const STATES: [State; 12] = [
    State {
        name: "help-splitter-hover",
        german: false,
        tab: HelpTab::Help,
        export_button: ButtonState::Ready,
        status: None,
        notice: None,
        mouse: SPLITTER,
    },
    State {
        name: "help",
        german: false,
        tab: HelpTab::Help,
        export_button: ButtonState::Ready,
        status: None,
        notice: None,
        mouse: AWAY,
    },
    State {
        name: "help-de",
        german: true,
        tab: HelpTab::Help,
        export_button: ButtonState::Ready,
        status: None,
        notice: None,
        mouse: AWAY,
    },
    State {
        name: "problem",
        german: false,
        tab: HelpTab::ReportProblem,
        export_button: ButtonState::Ready,
        status: None,
        notice: None,
        mouse: AWAY,
    },
    State {
        name: "problem-success",
        german: false,
        tab: HelpTab::ReportProblem,
        export_button: ButtonState::Ready,
        status: Some((
            Status::Success,
            "Saved trailkit-logs-20261112.txt in Downloads.",
        )),
        notice: None,
        mouse: AWAY,
    },
    State {
        name: "problem-success-trimmed",
        german: false,
        tab: HelpTab::ReportProblem,
        export_button: ButtonState::Ready,
        status: Some((
            Status::Success,
            "Saved trailkit-logs-20261112.txt in Downloads. Older days were left out to stay under 5 MiB.",
        )),
        notice: None,
        mouse: AWAY,
    },
    State {
        name: "problem-failure",
        german: false,
        tab: HelpTab::ReportProblem,
        export_button: ButtonState::Ready,
        status: Some((
            Status::Danger,
            "Could not save the log. Check that your Downloads folder can be written to, then try again.",
        )),
        notice: None,
        mouse: AWAY,
    },
    State {
        name: "problem-no-log",
        german: false,
        tab: HelpTab::ReportProblem,
        export_button: ButtonState::Disabled {
            reason: "There is no log yet.",
        },
        status: Some((
            Status::Info,
            "There is no log yet. Use the addon for a moment, then try again.",
        )),
        notice: None,
        mouse: AWAY,
    },
    State {
        name: "problem-de-failure",
        german: true,
        tab: HelpTab::ReportProblem,
        export_button: ButtonState::Ready,
        status: Some((
            Status::Danger,
            "Das Log konnte nicht gespeichert werden. Prüfe, ob dein Download-Ordner beschreibbar ist, und versuche es erneut.",
        )),
        notice: None,
        mouse: AWAY,
    },
    State {
        name: "problem-stopped",
        german: false,
        tab: HelpTab::ReportProblem,
        export_button: ButtonState::Ready,
        status: None,
        notice: Some(ProblemNotice {
            text: "memo stopped after an error.",
            detail: "Report the problem here, then reload memo in the Nexus addon list or restart the game.",
        }),
        mouse: AWAY,
    },
    State {
        name: "problem-de-stopped-draft",
        german: true,
        tab: HelpTab::ReportProblem,
        export_button: ButtonState::Ready,
        status: None,
        notice: Some(ProblemNotice {
            text: "memo wurde nach einem Fehler angehalten.",
            detail: "Dein ungespeicherter Text ist als Entwurf gesichert. Melde das Problem hier und lade memo danach in der Addon-Liste von Nexus neu oder starte das Spiel neu.",
        }),
        mouse: AWAY,
    },
    State {
        name: "tab-hover",
        german: false,
        tab: HelpTab::Help,
        export_button: ButtonState::Ready,
        status: None,
        notice: None,
        // Over "Report a problem" in the tab row.
        mouse: [110.0, 62.0],
    },
];

/// Chip texts are not used by the help text.
fn chip_text(_text: ChipText) -> &'static str {
    ""
}

#[test]
fn help_window_states_match_snapshots_at_the_minimum_width() {
    // The window asks for a tiny width, so ImGui widens it to the minimum
    // of 60 characters; the image is a little wider than that.
    let size = [IMAGE_WIDTH, HEIGHT];
    for state in &STATES {
        for (kind, theme_name) in [(ThemeKind::Dark, "dark"), (ThemeKind::Light, "light")] {
            let theme = kind.theme();
            let mut window = HelpWindow::new();
            window.open();
            window.show_tab(state.tab);
            let (texts, help_text) = if state.german {
                (TEXTS_DE, HELP_DE)
            } else {
                (TEXTS_EN, HELP_EN)
            };
            let content = HelpWindowContent {
                id: "help",
                help_text,
                texts,
                problem: ProblemPage {
                    export_button: state.export_button,
                    status: state.status,
                    notice: state.notice,
                },
                chips: ChipContext { text: &chip_text },
            };
            let mut harness = UiHarness::new(size);
            let input = FrameInput {
                mouse_position: state.mouse,
                ..FrameInput::default()
            };
            let image = harness.settle(SETTLE_FRAMES, &input, |ui: &Ui| {
                // The default tooltip offset, not the real cursor (it differs per run).
                let _style = StyleScope::push_with_cursor_extent(ui, &theme, None);
                window.draw_in(
                    ui,
                    &theme,
                    Window::new("help")
                        .position([0.0, 0.0], Condition::Always)
                        .size([10.0, HEIGHT], Condition::Always),
                    &content,
                );
            });
            assert_snapshot(
                env!("CARGO_MANIFEST_DIR"),
                &format!("help-window-{}-{theme_name}", state.name),
                &image,
            );
        }
    }
}

/// A topic with an e-mail link (memo M183), as in the privacy topic.
const HELP_MAIL: &str = "## Privacy\nWhat stays on your PC and what leaves it.\n\n### Contact\nWrite to <privacy@example.com> if you have questions.\n";

/// Over the e-mail address in [`HELP_MAIL`] at the minimum width.
const MAIL_LINK: [f32; 2] = [270.0, 212.0];

#[test]
fn a_hovered_email_link_shows_its_address() {
    for (kind, theme_name) in [(ThemeKind::Dark, "dark"), (ThemeKind::Light, "light")] {
        let theme = kind.theme();
        let mut window = HelpWindow::new();
        window.open();
        let content = HelpWindowContent {
            id: "help",
            help_text: HELP_MAIL,
            texts: TEXTS_EN,
            problem: ProblemPage {
                export_button: ButtonState::Ready,
                status: None,
                notice: None,
            },
            chips: ChipContext { text: &chip_text },
        };
        let mut harness = UiHarness::new([IMAGE_WIDTH, HEIGHT]);
        let input = FrameInput {
            mouse_position: MAIL_LINK,
            ..FrameInput::default()
        };
        let image = harness.settle(SETTLE_FRAMES, &input, |ui: &Ui| {
            // The default tooltip offset, not the real cursor (it differs per run).
            let _style = StyleScope::push_with_cursor_extent(ui, &theme, None);
            window.draw_in(
                ui,
                &theme,
                Window::new("help")
                    .position([0.0, 0.0], Condition::Always)
                    .size([10.0, HEIGHT], Condition::Always),
                &content,
            );
        });
        assert_snapshot(
            env!("CARGO_MANIFEST_DIR"),
            &format!("help-window-mail-link-hover-{theme_name}"),
            &image,
        );
    }
}
