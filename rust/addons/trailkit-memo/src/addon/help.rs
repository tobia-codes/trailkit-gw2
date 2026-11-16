//! Memo's help window (ROADMAP R95, R100): the shared `trailkit-ui` window
//! with memo's texts and help pages, the title bar `?`, and the two steps
//! of the problem page. The log export runs on a short-lived worker thread
//! per click (nexus-addon-dev skill §2: no I/O on the render thread); so does
//! the check whether a log exists, once each time the window opens. Both
//! threads end when their reply is sent, so nothing runs while memo idles.

use std::{
    ffi::OsString,
    os::windows::{ffi::OsStringExt as _, process::CommandExt as _},
    path::{Path, PathBuf},
    sync::mpsc::{self, Receiver, TryRecvError},
    thread,
};

use jiff::Timestamp;
use nexus::imgui::Ui;
use windows::Win32::{
    System::Com::CoTaskMemFree,
    UI::Shell::{FOLDERID_Downloads, KF_FLAG_DEFAULT, SHGetKnownFolderPath},
};

use trailkit_log::{ErrorChain, has_logs};
use trailkit_memo_shared::i18n::{
    Language, LocalizedText as _, Text, chip_context, local_utc_offset,
};
use trailkit_ui::{
    busy::BusyState,
    button::ButtonState,
    help_window::{
        HelpAction, HelpTab, HelpWindow, HelpWindowContent, HelpWindowState, HelpWindowTexts,
        ProblemNotice, ProblemPage,
    },
    markdown_view::open_link,
    status_line::Status,
    theme::Theme,
    title_bar::{TitleBarTexts, TitleBarToggle},
};

use crate::{
    help_document::help_document,
    help_text::HelpText,
    log_export::{ExportError, ExportOutcome, ExportStage, problem_report_url, write_log_export},
    main_window::help_title_bar_toggle,
};

/// The ImGui id of the help window, stable across languages; also its name
/// for Nexus' close on Escape.
pub(super) const WINDOW_ID: &str = "trailkit.memo.help";

/// The addon name in the issue form (its `addon` dropdown).
const ISSUE_FORM_ADDON: &str = "trailkit-memo";

/// The name of the export thread, as shown in a `panic` line.
const EXPORT_THREAD_NAME: &str = "trailkit-memo-log-export";

/// The name of the thread that checks for a log, as shown in a `panic` line.
const LOG_CHECK_THREAD_NAME: &str = "trailkit-memo-log-check";

/// What an export thread reports.
type ExportReply = Result<ExportOutcome, ExportError>;

/// The help window and the state of its problem page; render thread only.
pub(super) struct MemoHelp {
    /// The shared window: open, tab and topic.
    window: HelpWindow,
    /// `trailkit/logs/`; `None` without the `trailkit/` folder.
    logs: Option<PathBuf>,
    /// Whether any trailkit addon has a log; `None` until checked.
    has_logs: Option<bool>,
    /// The pending check for a log.
    log_check: Option<Receiver<bool>>,
    /// The running export.
    export: Option<Receiver<ExportReply>>,
    /// The export button's spinner.
    busy: BusyState,
    /// The status line of the last export, until the window closes or a
    /// new export starts (Q302).
    status: Option<(Status, String)>,
    /// Whether the saved tab, topic and list width were applied (R101).
    restored: bool,
}

impl MemoHelp {
    /// A closed help window for the logs in `logs`.
    pub(super) fn new(logs: Option<PathBuf>) -> Self {
        Self {
            window: HelpWindow::new(),
            logs,
            has_logs: None,
            log_check: None,
            export: None,
            busy: BusyState::default(),
            status: None,
            restored: false,
        }
    }

    /// Applies the saved tab, topic and list width once, as soon as memo's
    /// UI state is loaded (R101 Q314, Q318).
    pub(super) fn restore_once(&mut self, state: HelpWindowState) {
        if !std::mem::replace(&mut self.restored, true) {
            self.window.restore(state);
        }
    }

    /// The state to save after the user changed tab or topic or let go of
    /// the splitter; `None` otherwise.
    pub(super) fn take_state_to_save(&mut self) -> Option<HelpWindowState> {
        self.window.take_state_to_save()
    }

    /// Whether the window is shown.
    pub(super) const fn is_open(&self) -> bool {
        self.window.is_open()
    }

    /// The title bar `?` (Q171): active while the window is open.
    pub(super) fn title_bar_toggle(&self, language: Language) -> TitleBarToggle<'static> {
        help_title_bar_toggle(language, self.window.is_open())
    }

    /// The title bar `?` was clicked: opens or closes the window (Q296).
    pub(super) fn toggle(&mut self) {
        let was_open = self.window.is_open();
        self.window.toggle();
        self.after_open_change(was_open);
    }

    /// "Open help" in the Nexus options was clicked: opens the window or
    /// brings it to the front; never closes it (Q296).
    pub(super) fn open(&mut self) {
        let was_open = self.window.is_open();
        self.window.open();
        self.after_open_change(was_open);
    }

    /// memo stopped on an error (R104, Q366): opens the window on "Report a
    /// problem" or brings it to the front there.
    pub(super) fn open_on_problem_tab(&mut self) {
        self.window.show_tab(HelpTab::ReportProblem);
        self.open();
    }

    /// Closes the window if it is open: Nexus cleared its flag on Escape.
    pub(super) fn close(&mut self) {
        if self.window.is_open() {
            self.toggle();
        }
    }

    /// Takes the replies of the worker threads and draws the window if it
    /// is open; carries out what the user asked for.
    pub(super) fn draw(&mut self, ui: &Ui, theme: &Theme, language: Language) {
        self.draw_with_notice(ui, theme, language, None);
    }

    /// Like [`Self::draw`] after memo stopped on an error: the problem page
    /// shows the warning card (R104, Q367), with the draft sentence when
    /// `draft_kept`.
    pub(super) fn draw_stopped(
        &mut self,
        ui: &Ui,
        theme: &Theme,
        language: Language,
        draft_kept: bool,
    ) {
        let detail = if draft_kept {
            HelpText::StoppedDetailWithDraft
        } else {
            HelpText::StoppedDetail
        };
        let notice = ProblemNotice {
            text: HelpText::StoppedNotice.get(language),
            detail: detail.get(language),
        };
        self.draw_with_notice(ui, theme, language, Some(notice));
    }

    /// Draws the window as [`Self::draw`] does, with `notice` on the problem
    /// page.
    fn draw_with_notice(
        &mut self,
        ui: &Ui,
        theme: &Theme,
        language: Language,
        notice: Option<ProblemNotice<'_>>,
    ) {
        if !self.window.is_open() {
            return;
        }
        self.receive_replies(language);
        let export_button = match self.has_logs {
            Some(false) => ButtonState::Disabled {
                reason: HelpText::NoLogReason.get(language),
            },
            _ => self.busy.button_state(),
        };
        let status = if self.has_logs == Some(false) {
            Some((Status::Info, HelpText::NoLogYet.get(language)))
        } else if self.busy.is_busy() {
            None
        } else {
            self.status
                .as_ref()
                .map(|(status, text)| (*status, text.as_str()))
        };
        let content = HelpWindowContent {
            id: WINDOW_ID,
            help_text: help_document(language),
            texts: window_texts(language),
            problem: ProblemPage {
                export_button,
                status,
                notice,
            },
            chips: chip_context(language),
        };
        let action = self.window.draw(ui, theme, &content);
        if !self.window.is_open() {
            self.after_open_change(true);
        }
        match action {
            Some(HelpAction::ExportLog) => self.start_export(language),
            Some(HelpAction::ReportProblem) => open_link(&problem_report_url(
                env!("CARGO_PKG_REPOSITORY"),
                ISSUE_FORM_ADDON,
                env!("CARGO_PKG_VERSION"),
            )),
            None => {}
        }
    }

    /// Follows an open or close: a closed window forgets the status line
    /// (Q302); an opened one checks again whether a log exists.
    fn after_open_change(&mut self, was_open: bool) {
        match (was_open, self.window.is_open()) {
            (true, false) => self.status = None,
            (false, true) => self.start_log_check(),
            _ => {}
        }
    }

    /// Starts the check whether any trailkit addon has a log.
    fn start_log_check(&mut self) {
        let Some(logs) = self.logs.clone() else {
            return;
        };
        let (sender, receiver) = mpsc::channel();
        let spawned = thread::Builder::new()
            .name(LOG_CHECK_THREAD_NAME.to_owned())
            .spawn(move || {
                // A folder that cannot be listed counts as "has logs": the
                // export then reports the real error.
                let found = has_logs(&logs).unwrap_or(true);
                let _ = sender.send(found);
            });
        match spawned {
            Ok(_) => self.log_check = Some(receiver),
            Err(error) => log::warn!(error:% = error; "log_check_failed"),
        }
    }

    /// Starts an export unless one runs; clears the last status (Q302).
    fn start_export(&mut self, language: Language) {
        if self.export.is_some() {
            return;
        }
        self.status = None;
        self.busy.start();
        let Some(logs) = self.logs.clone() else {
            self.busy.finish();
            self.has_logs = Some(false);
            return;
        };
        let (sender, receiver) = mpsc::channel();
        let spawned = thread::Builder::new()
            .name(EXPORT_THREAD_NAME.to_owned())
            .spawn(move || {
                let _ = sender.send(export_to_downloads(&logs));
            });
        match spawned {
            Ok(_) => self.export = Some(receiver),
            Err(error) => {
                log::warn!(stage = "thread", error:% = error; "log_export_failed");
                self.busy.finish();
                self.status = Some((
                    Status::Danger,
                    HelpText::ExportFailed.get(language).to_owned(),
                ));
            }
        }
    }

    /// Takes the replies of the log check and the export, if they arrived.
    fn receive_replies(&mut self, language: Language) {
        if let Some(receiver) = &self.log_check {
            match receiver.try_recv() {
                Ok(found) => {
                    self.has_logs = Some(found);
                    self.log_check = None;
                }
                Err(TryRecvError::Disconnected) => self.log_check = None,
                Err(TryRecvError::Empty) => {}
            }
        }
        let Some(receiver) = &self.export else {
            return;
        };
        let reply = match receiver.try_recv() {
            Ok(reply) => reply,
            Err(TryRecvError::Empty) => return,
            // The thread panicked; its `panic` line is in the log.
            Err(TryRecvError::Disconnected) => Err(ExportError {
                stage: ExportStage::Write,
                source: std::io::Error::other("export thread ended without a reply"),
            }),
        };
        self.export = None;
        self.busy.finish();
        self.status = match reply {
            Ok(ExportOutcome::Saved {
                file_name, trimmed, ..
            }) => {
                self.has_logs = Some(true);
                let text = if trimmed {
                    HelpText::ExportSavedTrimmed
                } else {
                    HelpText::ExportSaved
                };
                Some((Status::Success, text.with_file(language, &file_name)))
            }
            Ok(ExportOutcome::NoLogs) => {
                self.has_logs = Some(false);
                None
            }
            Err(_) => Some((
                Status::Danger,
                HelpText::ExportFailed.get(language).to_owned(),
            )),
        };
    }
}

/// Writes the export into the Downloads folder and shows it in Explorer;
/// runs on the export thread. A failure is logged here.
fn export_to_downloads(logs: &Path) -> ExportReply {
    let result = downloads_folder()
        .map_err(|source| ExportError {
            stage: ExportStage::Downloads,
            source,
        })
        .and_then(|downloads| {
            let now = Timestamp::now();
            write_log_export(logs, &downloads, now, local_utc_offset(now))
        });
    match &result {
        Ok(ExportOutcome::Saved { path, .. }) => show_in_explorer(path),
        Ok(ExportOutcome::NoLogs) => {}
        Err(error) => log::warn!(
            stage = error.stage.code(),
            error:% = ErrorChain(&error.source);
            "log_export_failed"
        ),
    }
    result
}

/// Opens Explorer with `file` selected. A failure is only logged: the file
/// is saved and the status line names it.
fn show_in_explorer(file: &Path) {
    // `raw_arg`: Explorer wants `/select,"<path>"` as one argument with the
    // quotes inside; Rust's own quoting would wrap the whole argument.
    let result = std::process::Command::new("explorer")
        .raw_arg(format!("/select,\"{}\"", file.display()))
        .spawn();
    if let Err(error) = result {
        log::warn!(error:% = error; "folder_open_failed");
    }
}

/// The player's Downloads folder, wherever they moved it.
///
/// # Errors
///
/// Returns the Windows error if the folder is not known.
fn downloads_folder() -> std::io::Result<PathBuf> {
    // SAFETY: `FOLDERID_Downloads` is a valid known folder id; without a
    // token the current user's folder is returned. On success the returned
    // string is a valid, NUL-terminated wide string owned by us, read once
    // and then freed with `CoTaskMemFree` as the API requires.
    unsafe {
        let path = SHGetKnownFolderPath(&FOLDERID_Downloads, KF_FLAG_DEFAULT, None)
            .map_err(std::io::Error::other)?;
        let folder = PathBuf::from(OsString::from_wide(path.as_wide()));
        CoTaskMemFree(Some(path.0.cast_const().cast()));
        Ok(folder)
    }
}

/// The texts of the help window in `language`.
fn window_texts(language: Language) -> HelpWindowTexts<'static> {
    HelpWindowTexts {
        title_bar: TitleBarTexts {
            title: HelpText::WindowTitle.get(language),
            collapse: Text::CollapseWindow.get(language),
            expand: Text::ExpandWindow.get(language),
            close: Text::CloseWindow.get(language),
        },
        help_tab: HelpText::Help.get(language),
        problem_tab: HelpText::ReportProblem.get(language),
        problem_intro: HelpText::ProblemIntro.get(language),
        save_log_step: HelpText::SaveLogStep.get(language),
        export_log: HelpText::ExportLog.get(language),
        export_hint: HelpText::ExportHint.get(language),
        report_step: HelpText::ReportStep.get(language),
        report_button: HelpText::ReportProblem.get(language),
        report_hint: HelpText::ReportHint.get(language),
        footer: HelpText::Footer.get(language),
    }
}
