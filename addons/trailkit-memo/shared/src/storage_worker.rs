//! The addon's worker thread for disk access (memo plan M18, M30, M180).
//!
//! The render thread owns the note tree. On every change it sends a copy to
//! this worker, which waits until no change arrived for the debounce delay,
//! then writes the file through `trailkit-storage`. A failed write is
//! repeated at most three times, 5 s apart, then the worker gives up until a
//! new change or a manual write ([`pending_save`]); it reports how the shown
//! documents fare as [`SaveReport`]s. At start the worker loads
//! `notes.json`, `ui_state.json` and `drafts.json` and hands them back over a
//! channel. An empty draft list deletes `drafts.json` instead of writing it.
//! While nothing is pending the worker blocks in `recv()` and uses no CPU
//! (nexus-addon-dev skill §2). On shutdown every pending save is written
//! once more before the thread ends. A worker that died can be restarted
//! without loading (Q151).
//!
//! Uses only `std`, so it is unit-tested on any host (ROADMAP R23).

mod pending_save;

use std::{
    io,
    sync::{
        Arc, OnceLock,
        mpsc::{self, Receiver, RecvTimeoutError, Sender},
    },
    thread::{self, JoinHandle},
    time::{Duration, Instant},
};

use jiff::Timestamp;
use trailkit_log::ErrorChain;
use trailkit_storage::{DocumentStore, FileName, LoadOutcome, ReadOnlyReason, VersionedDocument};

pub use self::pending_save::{MAX_ATTEMPTS, RETRY_DELAY, SAVE_DEBOUNCE, SaveHealth, WorkerTiming};
use self::pending_save::{Slot, WriteMode, WriteResult};
use crate::{
    drafts::DraftsDocument,
    note_tree::{NotesDocument, TreeNode},
    settings::{Settings, TextScale},
    ui_state::UiState,
};

/// Name of the worker thread (visible in debuggers and crash dumps).
const THREAD_NAME: &str = "trailkit-memo storage";

/// A request from the render thread to the worker.
enum WorkerCommand {
    /// Save this version of the notes once no newer one arrives in time.
    SaveNotes(Arc<NotesDocument>),
    /// Save these settings once no newer ones arrive in time.
    SaveSettings(Settings),
    /// Save this window state once no newer one arrives in time.
    SaveUiState(UiState),
    /// Save (or, if empty, delete) the drafts once no newer ones arrive in time.
    SaveDrafts(DraftsDocument),
    /// Write everything pending now, once, and report
    /// [`SaveReport::WriteFinished`].
    WriteNow,
    /// Write everything pending and end the thread.
    Shutdown,
}

/// What the worker tells the UI about saving (M180).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SaveReport {
    /// The state of the shown documents changed.
    Health(SaveHealth),
    /// A [`StorageSaver::write_now`] request is done; sent after the
    /// [`SaveReport::Health`] it caused.
    WriteFinished {
        /// Whether every shown document that was pending is on disk now.
        succeeded: bool,
    },
}

/// Handle to the running worker thread. Dropping it shuts the worker down and
/// waits until pending saves are written.
#[derive(Debug)]
pub struct StorageWorker {
    /// Channel to the worker thread.
    commands: Sender<WorkerCommand>,
    /// The worker thread; `None` once it has been joined.
    thread: Option<JoinHandle<()>>,
    /// The files the worker writes, kept for a restart.
    store: DocumentStore,
    /// The worker's delays, kept for a restart.
    timing: WorkerTiming,
    /// Which documents may be written; set once loading has ended.
    permissions: Arc<OnceLock<Permissions>>,
}

/// Cloneable handle for queueing saves, held by the render thread. Sending
/// never blocks.
#[derive(Debug, Clone)]
pub struct StorageSaver {
    /// Channel to the worker thread.
    commands: Sender<WorkerCommand>,
}

/// The render thread's link to a running worker: where to send saves and
/// where the worker reports back.
#[derive(Debug)]
pub struct SaveConnection {
    /// Queues saves on the worker.
    pub saver: StorageSaver,
    /// Reports from the worker; disconnected once the worker has ended.
    pub reports: Receiver<SaveReport>,
}

/// What the worker loaded at start, delivered once.
#[derive(Debug)]
pub struct LoadedDocuments {
    /// How `notes.json` was loaded.
    pub notes: LoadOutcome<NotesDocument>,
    /// The restored window state (defaults if missing or unusable).
    pub ui_state: UiState,
    /// Drafts kept from an earlier unload (empty if missing or unusable).
    pub drafts: DraftsDocument,
    /// Reports from the worker about saving.
    pub reports: Receiver<SaveReport>,
}

/// Which documents the worker may write (from their load outcomes).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SavingAllowed {
    /// Whether `settings.json` may be written.
    pub settings: bool,
}

/// Why [`StorageWorker::restart`] did not start a new worker.
#[derive(Debug, thiserror::Error)]
pub enum RestartError {
    /// The worker thread is still running.
    #[error("the storage worker is still running")]
    StillRunning,
    /// The worker ended before it finished loading, so it is unknown which
    /// documents may be written.
    #[error("the storage worker ended before loading finished")]
    NotLoaded,
    /// The operating system cannot start the thread.
    #[error("cannot start the storage worker: {0}")]
    Spawn(#[from] io::Error),
}

impl StorageWorker {
    /// Starts the worker thread for the files in `store`.
    ///
    /// The worker first loads `notes.json`, `ui_state.json` and `drafts.json`;
    /// the returned receiver yields them exactly once as [`LoadedDocuments`].
    /// Settings are loaded by the caller (they are needed synchronously at
    /// addon load), so only whether they may be saved and what the
    /// `load_finished` line reports about them is passed in. `timing` is
    /// [`WorkerTiming::DEFAULT`] outside of tests.
    ///
    /// # Errors
    ///
    /// Returns the I/O error if the operating system cannot start the thread.
    pub fn spawn(
        store: DocumentStore,
        timing: WorkerTiming,
        saving_allowed: SavingAllowed,
        settings_summary: SettingsSummary,
    ) -> io::Result<(Self, Receiver<LoadedDocuments>)> {
        let (commands, command_receiver) = mpsc::channel();
        let (loaded_sender, loaded) = mpsc::channel();
        let permissions = Arc::new(OnceLock::new());
        let thread = {
            let store = store.clone();
            let permissions = Arc::clone(&permissions);
            thread::Builder::new()
                .name(THREAD_NAME.to_owned())
                .spawn(move || {
                    let (report_sender, reports) = mpsc::channel();
                    let (documents, loaded_permissions) =
                        load_documents(&store, saving_allowed, settings_summary, reports);
                    let _ = permissions.set(loaded_permissions);
                    // The render side may already be gone during a quick unload; that is fine.
                    let _ = loaded_sender.send(documents);
                    serve(
                        &store,
                        timing,
                        loaded_permissions,
                        &command_receiver,
                        &report_sender,
                    );
                })?
        };
        let worker = Self {
            commands,
            thread: Some(thread),
            store,
            timing,
            permissions,
        };
        Ok((worker, loaded))
    }

    /// A handle for queueing saves from another thread.
    #[must_use]
    pub fn saver(&self) -> StorageSaver {
        StorageSaver {
            commands: self.commands.clone(),
        }
    }

    /// Starts a new worker after the old one died (Q151): it loads nothing
    /// and keeps the old worker's write permissions. The caller then sends
    /// its current documents and [`StorageSaver::write_now`]. Logs
    /// `worker_died` for the old thread and `worker_restarted`.
    ///
    /// # Errors
    ///
    /// Returns [`RestartError::StillRunning`] if the old thread has not
    /// ended (nothing changes then), [`RestartError::NotLoaded`] if it died
    /// while loading, or [`RestartError::Spawn`] if no thread can be started.
    pub fn restart(&mut self) -> Result<SaveConnection, RestartError> {
        if self
            .thread
            .as_ref()
            .is_some_and(|thread| !thread.is_finished())
        {
            return Err(RestartError::StillRunning);
        }
        self.join();
        let permissions = *self.permissions.get().ok_or(RestartError::NotLoaded)?;
        let (commands, command_receiver) = mpsc::channel();
        let (report_sender, reports) = mpsc::channel();
        let store = self.store.clone();
        let timing = self.timing;
        let thread = thread::Builder::new()
            .name(THREAD_NAME.to_owned())
            .spawn(move || {
                serve(
                    &store,
                    timing,
                    permissions,
                    &command_receiver,
                    &report_sender,
                )
            })?;
        self.commands = commands;
        self.thread = Some(thread);
        log::info!("worker_restarted");
        Ok(SaveConnection {
            saver: self.saver(),
            reports,
        })
    }

    /// Waits for the worker thread, logging `worker_died` if it panicked.
    fn join(&mut self) {
        if let Some(thread) = self.thread.take()
            && thread.join().is_err()
        {
            log::error!("worker_died");
        }
    }
}

impl StorageSaver {
    /// Queues `document` to be saved after the debounce delay.
    pub fn save_notes(&self, document: Arc<NotesDocument>) {
        self.send(WorkerCommand::SaveNotes(document));
    }

    /// Queues `settings` to be saved after the debounce delay.
    pub fn save_settings(&self, settings: Settings) {
        self.send(WorkerCommand::SaveSettings(settings));
    }

    /// Queues the window state to be saved after the debounce delay.
    pub fn save_ui_state(&self, ui_state: UiState) {
        self.send(WorkerCommand::SaveUiState(ui_state));
    }

    /// Queues the drafts to be saved (or the file deleted, if there are none)
    /// after the debounce delay.
    pub fn save_drafts(&self, drafts: DraftsDocument) {
        self.send(WorkerCommand::SaveDrafts(drafts));
    }

    /// Asks the worker to write everything queued so far now, once, even
    /// after it gave up ("Übernehmen" and manual retries, Q151, Q158). The
    /// result arrives as [`SaveReport::WriteFinished`].
    pub fn write_now(&self) {
        self.send(WorkerCommand::WriteNow);
    }

    /// Sends `command` to the worker, logging if the worker is gone.
    fn send(&self, command: WorkerCommand) {
        if self.commands.send(command).is_err() {
            log::error!(reason = "worker_stopped"; "save_dropped");
        }
    }
}

impl Drop for StorageWorker {
    /// Asks the worker to write everything pending and waits for it to finish.
    fn drop(&mut self) {
        // The worker may already be gone (it ends on a panic); nothing to report then.
        let _ = self.commands.send(WorkerCommand::Shutdown);
        self.join();
    }
}

/// What the caller learned when it read `settings.json`, for the
/// `load_finished` line.
#[derive(Debug, Clone, Copy, Default)]
pub struct SettingsSummary {
    /// Whether the file did not exist.
    pub missing: bool,
    /// The text size the addon starts with.
    pub text_scale: TextScale,
    /// How long reading it took.
    pub elapsed: Duration,
}

/// Which documents the worker may write, from their load outcomes.
#[derive(Debug, Clone, Copy)]
struct Permissions {
    /// `notes.json`.
    notes: bool,
    /// `settings.json` (loaded by the caller).
    settings: bool,
    /// `ui_state.json`.
    ui_state: bool,
    /// `drafts.json`.
    drafts: bool,
}

/// Loads the worker's documents and logs the `load_finished` summary.
/// `reports` is handed to the UI with the documents.
fn load_documents(
    store: &DocumentStore,
    saving_allowed: SavingAllowed,
    settings_summary: SettingsSummary,
    reports: Receiver<SaveReport>,
) -> (LoadedDocuments, Permissions) {
    let started = Instant::now();
    let now = Timestamp::now();
    let notes = load_logged::<NotesDocument>(store, now);
    let ui_state_outcome = load_logged::<UiState>(store, now);
    let drafts_outcome = load_logged::<DraftsDocument>(store, now);
    let permissions = Permissions {
        notes: notes.allows_saving(),
        settings: saving_allowed.settings,
        ui_state: ui_state_outcome.allows_saving(),
        drafts: drafts_outcome.allows_saving(),
    };
    let mut missing = Vec::new();
    if settings_summary.missing {
        missing.push(document_file_name::<Settings>());
    }
    missing.extend(missing_file(&notes));
    missing.extend(missing_file(&ui_state_outcome));
    let ui_state = ui_state_outcome.into_document().unwrap_or_default();
    let drafts = drafts_outcome.into_document().unwrap_or_default();
    log_load_finished(
        &LoadCounts {
            notes: note_count(&notes),
            drafts: drafts.drafts.len(),
        },
        &missing,
        settings_summary,
        started.elapsed() + settings_summary.elapsed,
    );
    let documents = LoadedDocuments {
        notes,
        ui_state,
        drafts,
        reports,
    };
    (documents, permissions)
}

/// The pending save of every document.
#[derive(Default)]
struct Slots {
    /// `notes.json`.
    notes: Slot<Arc<NotesDocument>>,
    /// `settings.json`.
    settings: Slot<Settings>,
    /// `ui_state.json`.
    ui_state: Slot<UiState>,
    /// `drafts.json`.
    drafts: Slot<DraftsDocument>,
}

impl Slots {
    /// When the next automatic write is due, if any is planned.
    fn next_due(&self) -> Option<Instant> {
        [
            self.notes.due_at(),
            self.settings.due_at(),
            self.ui_state.due_at(),
            self.drafts.due_at(),
        ]
        .into_iter()
        .flatten()
        .min()
    }

    /// Writes every slot as `mode` asks. Returns whether every shown
    /// document that was written is on disk now.
    fn write(&mut self, store: &DocumentStore, mode: WriteMode, timing: WorkerTiming) -> bool {
        [
            self.notes.write(store, mode, timing),
            self.settings.write(store, mode, timing),
            self.ui_state.write(store, mode, timing),
            self.drafts.write(store, mode, timing),
        ]
        .into_iter()
        .flatten()
        .all(|WriteResult { succeeded, shown }| succeeded || !shown)
    }

    /// How the user should see the shown documents.
    fn health(&self) -> SaveHealth {
        self.notes.health().worse(self.drafts.health())
    }
}

/// Serves save requests until shutdown: writes each document once it was
/// unchanged for the debounce delay, retries failed writes and reports
/// changes of [`SaveHealth`] and finished [`WorkerCommand::WriteNow`]s.
fn serve(
    store: &DocumentStore,
    timing: WorkerTiming,
    permissions: Permissions,
    commands: &Receiver<WorkerCommand>,
    reports: &Sender<SaveReport>,
) {
    let mut slots = Slots::default();
    let mut reported_health = SaveHealth::Healthy;
    loop {
        let received = match slots.next_due() {
            Some(due_at) => commands.recv_timeout(due_at.saturating_duration_since(Instant::now())),
            None => commands.recv().map_err(|_| RecvTimeoutError::Disconnected),
        };

        let now = Instant::now();
        let mut finished = None;
        match received {
            Ok(WorkerCommand::SaveNotes(document)) if permissions.notes => {
                slots.notes.replace(document, now, timing);
            }
            Ok(WorkerCommand::SaveSettings(settings)) if permissions.settings => {
                slots.settings.replace(settings, now, timing);
            }
            Ok(WorkerCommand::SaveUiState(ui_state)) if permissions.ui_state => {
                slots.ui_state.replace(ui_state, now, timing);
            }
            Ok(WorkerCommand::SaveDrafts(drafts)) if permissions.drafts => {
                slots.drafts.replace(drafts, now, timing);
            }
            Ok(
                WorkerCommand::SaveNotes(_)
                | WorkerCommand::SaveSettings(_)
                | WorkerCommand::SaveUiState(_)
                | WorkerCommand::SaveDrafts(_),
            ) => {
                log::warn!(reason = "read_only"; "save_dropped");
            }
            Ok(WorkerCommand::WriteNow) => {
                finished = Some(slots.write(store, WriteMode::Now, timing));
            }
            Ok(WorkerCommand::Shutdown) | Err(RecvTimeoutError::Disconnected) => {
                slots.write(store, WriteMode::Final, timing);
                return;
            }
            Err(RecvTimeoutError::Timeout) => {}
        }

        slots.write(store, WriteMode::Due(Instant::now()), timing);
        // The render side may be gone during unload; reports are then moot.
        let health = slots.health();
        if health != reported_health {
            reported_health = health;
            let _ = reports.send(SaveReport::Health(health));
        }
        if let Some(succeeded) = finished {
            let _ = reports.send(SaveReport::WriteFinished { succeeded });
        }
    }
}

/// Loads `D` from `store` and logs the outcome with its duration.
fn load_logged<D: VersionedDocument>(store: &DocumentStore, now: Timestamp) -> LoadOutcome<D> {
    let started = Instant::now();
    let outcome = store.load::<D>(now);
    log_load_outcome(&outcome, started.elapsed());
    outcome
}

/// The file name of document `D`, e.g. `notes.json`.
fn document_file_name<D: VersionedDocument>() -> String {
    format!("{}.json", D::FILE_STEM)
}

/// The file name of `D` if `outcome` found no file.
fn missing_file<D: VersionedDocument>(outcome: &LoadOutcome<D>) -> Option<String> {
    matches!(outcome, LoadOutcome::Missing).then(document_file_name::<D>)
}

/// How many notes `outcome` holds (0 without a document).
fn note_count(outcome: &LoadOutcome<NotesDocument>) -> usize {
    match outcome {
        LoadOutcome::Loaded { document, .. } => document
            .root
            .iter()
            .map(|node| match node {
                TreeNode::Note(_) => 1,
                TreeNode::Group(group) => group.note_count(),
            })
            .sum(),
        _ => 0,
    }
}

/// What the documents held when they were loaded.
struct LoadCounts {
    /// Notes in the tree, without the trash.
    notes: usize,
    /// Saved drafts.
    drafts: usize,
}

/// Logs the one `load_finished` line of a session (ROADMAP R92): that loading
/// ended, and how much there was. `missing` names the files that do not exist
/// (drafts are normally absent and not listed); outside a first start a
/// missing `notes.json` explains "my notes are gone".
fn log_load_finished(
    counts: &LoadCounts,
    missing: &[String],
    settings: SettingsSummary,
    elapsed: Duration,
) {
    let LoadCounts { notes, drafts } = *counts;
    let text_scale = f64::from(settings.text_scale.factor());
    let ms = elapsed.as_millis();
    if missing.is_empty() {
        log::info!(notes, drafts, text_scale, ms; "load_finished");
    } else {
        let missing = missing.join(",");
        log::info!(notes, drafts, missing = missing.as_str(), text_scale, ms; "load_finished");
    }
}

/// Logs a load that was not routine (ROADMAP R92): `load_migrated` (INFO),
/// `load_recovered` (ERROR, the unreadable file was moved to `backups/`) or
/// `load_read_only` (newer version: WARN, I/O error: ERROR), each with the
/// file name and the duration. A found or missing file logs nothing here;
/// [`log_load_finished`] sums those up.
pub fn log_load_outcome<D: VersionedDocument>(outcome: &LoadOutcome<D>, elapsed: Duration) {
    let file = document_file_name::<D>();
    let file = file.as_str();
    let ms = elapsed.as_millis();
    match outcome {
        LoadOutcome::Missing
        | LoadOutcome::Loaded {
            migrated_from: None,
            ..
        } => {}
        LoadOutcome::Loaded {
            migrated_from: Some(version),
            ..
        } => log::info!(file, from_version = version.0, ms; "load_migrated"),
        LoadOutcome::Recovered {
            reason,
            preserved_as,
        } => log::error!(
            file,
            error = reason.as_str(),
            preserved_as:% = FileName::of(preserved_as),
            ms;
            "load_recovered"
        ),
        LoadOutcome::ReadOnly(ReadOnlyReason::NewerVersion { found }) => {
            log::warn!(file, reason = "newer_version", found = found.0, ms; "load_read_only");
        }
        LoadOutcome::ReadOnly(ReadOnlyReason::Io(error)) => {
            log::error!(file, reason = "io", error:% = ErrorChain(error), ms; "load_read_only");
        }
    }
}

#[cfg(test)]
mod tests {
    use std::{fs, path::Path};

    use super::*;
    use crate::{
        drafts::Draft,
        note_tree::{Note, NoteId, NoteKind, TreeNode},
        settings::QuickAccessIcon,
    };

    /// Short debounce so tests stay fast.
    const TEST_DEBOUNCE: Duration = Duration::from_millis(50);

    /// Short delays so tests stay fast.
    const TEST_TIMING: WorkerTiming = WorkerTiming {
        debounce: TEST_DEBOUNCE,
        retry_delay: TEST_DEBOUNCE,
    };

    /// Delays no test waits for: only shutdown or `write_now` write.
    const SLOW_TIMING: WorkerTiming = WorkerTiming {
        debounce: Duration::from_secs(3600),
        retry_delay: Duration::from_secs(3600),
    };

    /// Upper bound for waiting on the worker in tests.
    const TEST_TIMEOUT: Duration = Duration::from_secs(5);

    /// Settings may be saved.
    const ALL_ALLOWED: SavingAllowed = SavingAllowed { settings: true };

    /// A document with one note.
    fn document_with_note(title: &str) -> Arc<NotesDocument> {
        Arc::new(NotesDocument {
            root: vec![TreeNode::Note(Note::new(
                title.to_owned(),
                NoteKind::Normal,
                Timestamp::now(),
            ))],
            trash: Vec::new(),
        })
    }

    /// Reads the saved notes, if the file exists and is current.
    fn saved_notes(directory: &Path) -> Option<NotesDocument> {
        DocumentStore::new(directory)
            .load::<NotesDocument>(Timestamp::now())
            .into_document()
    }

    #[test]
    fn loads_notes_and_reports_them_once() -> Result<(), Box<dyn std::error::Error>> {
        let directory = tempfile::tempdir()?;
        let store = DocumentStore::new(directory.path());
        let document = document_with_note("Existing");
        store.save(document.as_ref())?;

        let (_worker, loaded) =
            StorageWorker::spawn(store, TEST_TIMING, ALL_ALLOWED, SettingsSummary::default())?;
        let loaded = loaded.recv_timeout(TEST_TIMEOUT)?;
        assert_eq!(
            loaded.notes.into_document().as_ref(),
            Some(document.as_ref())
        );
        assert_eq!(loaded.ui_state, UiState::default());
        Ok(())
    }

    #[test]
    fn saves_only_the_latest_version_after_the_debounce() -> Result<(), Box<dyn std::error::Error>>
    {
        let directory = tempfile::tempdir()?;
        let (worker, loaded) = StorageWorker::spawn(
            DocumentStore::new(directory.path()),
            TEST_TIMING,
            ALL_ALLOWED,
            SettingsSummary::default(),
        )?;
        loaded.recv_timeout(TEST_TIMEOUT)?;

        let saver = worker.saver();
        saver.save_notes(document_with_note("first"));
        let latest = document_with_note("second");
        saver.save_notes(Arc::clone(&latest));
        assert!(
            saved_notes(directory.path()).is_none(),
            "nothing written before the debounce"
        );

        let deadline = Instant::now() + TEST_TIMEOUT;
        while saved_notes(directory.path()).is_none() && Instant::now() < deadline {
            thread::sleep(TEST_DEBOUNCE);
        }
        assert_eq!(
            saved_notes(directory.path()).as_ref(),
            Some(latest.as_ref())
        );
        Ok(())
    }

    #[test]
    fn shutdown_writes_pending_saves_immediately() -> Result<(), Box<dyn std::error::Error>> {
        let directory = tempfile::tempdir()?;
        let (worker, loaded) = StorageWorker::spawn(
            DocumentStore::new(directory.path()),
            SLOW_TIMING,
            ALL_ALLOWED,
            SettingsSummary::default(),
        )?;
        loaded.recv_timeout(TEST_TIMEOUT)?;
        let saver = worker.saver();
        let document = document_with_note("unsaved");
        saver.save_notes(Arc::clone(&document));
        let settings = Settings {
            quick_access_icon: QuickAccessIcon::Hidden,
            ..Settings::DEFAULT
        };
        saver.save_settings(settings);
        let ui_state = UiState {
            note_scroll: 42.0,
            ..UiState::default()
        };
        saver.save_ui_state(ui_state);

        drop(worker);

        let saved_ui_state = DocumentStore::new(directory.path())
            .load::<UiState>(Timestamp::now())
            .into_document();
        assert_eq!(saved_ui_state, Some(ui_state));

        assert_eq!(
            saved_notes(directory.path()).as_ref(),
            Some(document.as_ref())
        );
        let saved_settings = DocumentStore::new(directory.path())
            .load::<Settings>(Timestamp::now())
            .into_document();
        assert_eq!(saved_settings, Some(settings));
        Ok(())
    }

    #[test]
    fn drafts_are_written_on_shutdown_and_deleted_when_empty()
    -> Result<(), Box<dyn std::error::Error>> {
        let directory = tempfile::tempdir()?;
        let store = DocumentStore::new(directory.path());
        let spawn = || {
            StorageWorker::spawn(
                DocumentStore::new(directory.path()),
                SLOW_TIMING,
                ALL_ALLOWED,
                SettingsSummary::default(),
            )
        };
        let draft = Draft {
            note: NoteId::new_unique(),
            title: "Title".to_owned(),
            body: "Unsaved".to_owned(),
            saved_at: Timestamp::now(),
            note_updated_at: Timestamp::now(),
        };

        let (worker, loaded) = spawn()?;
        assert!(loaded.recv_timeout(TEST_TIMEOUT)?.drafts.drafts.is_empty());
        worker.saver().save_drafts(DraftsDocument {
            drafts: vec![draft.clone()],
        });
        drop(worker);

        let (worker, loaded) = spawn()?;
        assert_eq!(
            loaded.recv_timeout(TEST_TIMEOUT)?.drafts.drafts,
            vec![draft]
        );
        worker.saver().save_drafts(DraftsDocument::default());
        drop(worker);
        assert!(!store.path_of::<DraftsDocument>().exists());
        Ok(())
    }

    #[test]
    fn read_only_documents_are_never_written() -> Result<(), Box<dyn std::error::Error>> {
        let directory = tempfile::tempdir()?;
        let store = DocumentStore::new(directory.path());
        let newer = r#"{"schema_version": 999}"#;
        fs::write(store.path_of::<NotesDocument>(), newer)?;
        let (worker, loaded) = StorageWorker::spawn(
            store.clone(),
            TEST_TIMING,
            SavingAllowed { settings: false },
            SettingsSummary::default(),
        )?;
        assert!(!loaded.recv_timeout(TEST_TIMEOUT)?.notes.allows_saving());

        let saver = worker.saver();
        saver.save_notes(document_with_note("ignored"));
        saver.save_settings(Settings::DEFAULT);
        drop(worker);

        assert_eq!(fs::read_to_string(store.path_of::<NotesDocument>())?, newer);
        assert!(!store.path_of::<Settings>().exists());
        Ok(())
    }

    /// Waits for the next report from the worker.
    fn next_report(reports: &Receiver<SaveReport>) -> Result<SaveReport, RecvTimeoutError> {
        reports.recv_timeout(TEST_TIMEOUT)
    }

    #[test]
    fn failed_notes_are_retried_three_times_then_given_up() -> Result<(), Box<dyn std::error::Error>>
    {
        let directory = tempfile::tempdir()?;
        let store = DocumentStore::new(directory.path());
        // A folder where the temporary file must go makes every notes save fail.
        let blocker = directory.path().join("notes.json.tmp");
        fs::create_dir(&blocker)?;
        let (worker, loaded) =
            StorageWorker::spawn(store, TEST_TIMING, ALL_ALLOWED, SettingsSummary::default())?;
        let reports = loaded.recv_timeout(TEST_TIMEOUT)?.reports;
        let document = document_with_note("blocked");
        worker.saver().save_notes(Arc::clone(&document));

        for failed_attempts in 1..MAX_ATTEMPTS {
            assert_eq!(
                next_report(&reports)?,
                SaveReport::Health(SaveHealth::Retrying { failed_attempts })
            );
        }
        assert_eq!(
            next_report(&reports)?,
            SaveReport::Health(SaveHealth::GaveUp)
        );
        // No more automatic writes: nothing changes without input.
        assert!(reports.recv_timeout(TEST_DEBOUNCE * 6).is_err());

        // A manual write once the disk works again succeeds and reports it.
        fs::remove_dir(&blocker)?;
        worker.saver().write_now();
        assert_eq!(
            next_report(&reports)?,
            SaveReport::Health(SaveHealth::Healthy)
        );
        assert_eq!(
            next_report(&reports)?,
            SaveReport::WriteFinished { succeeded: true }
        );
        assert_eq!(
            saved_notes(directory.path()).as_ref(),
            Some(document.as_ref())
        );
        Ok(())
    }

    #[test]
    fn a_failed_manual_write_reports_it_and_keeps_waiting() -> Result<(), Box<dyn std::error::Error>>
    {
        let directory = tempfile::tempdir()?;
        fs::create_dir(directory.path().join("notes.json.tmp"))?;
        let (worker, loaded) = StorageWorker::spawn(
            DocumentStore::new(directory.path()),
            SLOW_TIMING,
            ALL_ALLOWED,
            SettingsSummary::default(),
        )?;
        let reports = loaded.recv_timeout(TEST_TIMEOUT)?.reports;
        let saver = worker.saver();
        saver.save_notes(document_with_note("blocked"));
        saver.write_now();
        assert_eq!(
            next_report(&reports)?,
            SaveReport::Health(SaveHealth::Retrying { failed_attempts: 1 })
        );
        assert_eq!(
            next_report(&reports)?,
            SaveReport::WriteFinished { succeeded: false }
        );
        Ok(())
    }

    #[test]
    fn hidden_documents_never_report_failures() -> Result<(), Box<dyn std::error::Error>> {
        let directory = tempfile::tempdir()?;
        fs::create_dir(directory.path().join("settings.json.tmp"))?;
        let (worker, loaded) = StorageWorker::spawn(
            DocumentStore::new(directory.path()),
            SLOW_TIMING,
            ALL_ALLOWED,
            SettingsSummary::default(),
        )?;
        let reports = loaded.recv_timeout(TEST_TIMEOUT)?.reports;
        let saver = worker.saver();
        saver.save_settings(Settings::DEFAULT);
        saver.write_now();
        assert_eq!(
            next_report(&reports)?,
            SaveReport::WriteFinished { succeeded: true }
        );
        Ok(())
    }

    #[test]
    fn a_running_worker_is_not_restarted() -> Result<(), Box<dyn std::error::Error>> {
        let directory = tempfile::tempdir()?;
        let (mut worker, loaded) = StorageWorker::spawn(
            DocumentStore::new(directory.path()),
            SLOW_TIMING,
            ALL_ALLOWED,
            SettingsSummary::default(),
        )?;
        loaded.recv_timeout(TEST_TIMEOUT)?;
        assert!(matches!(worker.restart(), Err(RestartError::StillRunning)));
        Ok(())
    }

    #[test]
    fn an_ended_worker_restarts_without_loading_and_writes()
    -> Result<(), Box<dyn std::error::Error>> {
        let directory = tempfile::tempdir()?;
        let (mut worker, loaded) = StorageWorker::spawn(
            DocumentStore::new(directory.path()),
            SLOW_TIMING,
            ALL_ALLOWED,
            SettingsSummary::default(),
        )?;
        let old_reports = loaded.recv_timeout(TEST_TIMEOUT)?.reports;
        // Stands in for a panic: the old thread ends and its reports disconnect.
        worker.saver().send(WorkerCommand::Shutdown);
        assert!(matches!(
            old_reports.recv_timeout(TEST_TIMEOUT),
            Err(RecvTimeoutError::Disconnected)
        ));

        // The thread ends just after its channels close; a restart before that is refused.
        let deadline = Instant::now() + TEST_TIMEOUT;
        let SaveConnection { saver, reports } = loop {
            match worker.restart() {
                Err(RestartError::StillRunning) if Instant::now() < deadline => {
                    thread::sleep(Duration::from_millis(1));
                }
                result => break result?,
            }
        };
        let document = document_with_note("after restart");
        saver.save_notes(Arc::clone(&document));
        saver.write_now();
        assert_eq!(
            next_report(&reports)?,
            SaveReport::WriteFinished { succeeded: true }
        );
        assert_eq!(
            saved_notes(directory.path()).as_ref(),
            Some(document.as_ref())
        );
        Ok(())
    }
}
