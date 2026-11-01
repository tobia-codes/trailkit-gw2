//! The addon's worker thread for disk access (memo plan M18, M30).
//!
//! The render thread owns the note tree. On every change it sends a copy to
//! this worker, which waits until no change arrived for the debounce delay,
//! then writes the file through `trailkit-storage`. At start the worker loads
//! `notes.json` and hands the result back over a channel. While nothing is
//! pending the worker blocks in `recv()` and uses no CPU (nexus-addon-dev
//! skill §2). On shutdown every pending save is written before the thread ends.
//!
//! Uses only `std`, so it is unit-tested on any host (ROADMAP R23).

use std::{
    borrow::Borrow,
    io,
    sync::{
        Arc,
        mpsc::{self, Receiver, RecvTimeoutError, Sender},
    },
    thread::{self, JoinHandle},
    time::{Duration, Instant},
};

use jiff::Timestamp;
use trailkit_storage::{DocumentStore, LoadOutcome, ReadOnlyReason, VersionedDocument};

use crate::{note_tree::NotesDocument, settings::Settings};

/// Delay after the last change before a document is written (M18).
pub const SAVE_DEBOUNCE: Duration = Duration::from_secs(2);

/// Name of the worker thread (visible in debuggers and crash dumps).
const THREAD_NAME: &str = "trailkit-memo storage";

/// A request from the render thread to the worker.
enum WorkerCommand {
    /// Save this version of the notes once no newer one arrives in time.
    SaveNotes(Arc<NotesDocument>),
    /// Save these settings once no newer ones arrive in time.
    SaveSettings(Settings),
    /// Write everything pending and end the thread.
    Shutdown,
}

/// Handle to the running worker thread. Dropping it shuts the worker down and
/// waits until pending saves are written.
#[derive(Debug)]
pub struct StorageWorker {
    /// Channel to the worker thread.
    commands: Sender<WorkerCommand>,
    /// The worker thread; `None` once it has been joined.
    thread: Option<JoinHandle<()>>,
}

/// Which documents the worker may write (from their load outcomes).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SavingAllowed {
    /// Whether `settings.json` may be written.
    pub settings: bool,
}

/// A document waiting for its debounce delay to pass.
struct PendingSave<T> {
    /// The newest version of the document.
    document: T,
    /// When it is written unless a newer version arrives first.
    due_at: Instant,
}

impl StorageWorker {
    /// Starts the worker thread for the files in `store`.
    ///
    /// The worker first loads `notes.json`; the returned receiver yields that
    /// [`LoadOutcome`] exactly once. Settings are loaded by the caller (they are
    /// needed synchronously at addon load), so only whether they may be saved
    /// is passed in. `debounce` is [`SAVE_DEBOUNCE`] outside of tests.
    ///
    /// # Errors
    ///
    /// Returns the I/O error if the operating system cannot start the thread.
    pub fn spawn(
        store: DocumentStore,
        debounce: Duration,
        saving_allowed: SavingAllowed,
    ) -> io::Result<(Self, Receiver<LoadOutcome<NotesDocument>>)> {
        let (commands, command_receiver) = mpsc::channel();
        let (loaded_sender, loaded) = mpsc::channel();
        let thread = thread::Builder::new()
            .name(THREAD_NAME.to_owned())
            .spawn(move || {
                run_worker(
                    &store,
                    debounce,
                    saving_allowed,
                    &command_receiver,
                    &loaded_sender,
                );
            })?;
        let worker = Self {
            commands,
            thread: Some(thread),
        };
        Ok((worker, loaded))
    }

    /// Queues `document` to be saved after the debounce delay. Never blocks.
    pub fn save_notes(&self, document: Arc<NotesDocument>) {
        self.send(WorkerCommand::SaveNotes(document));
    }

    /// Queues `settings` to be saved after the debounce delay. Never blocks.
    pub fn save_settings(&self, settings: Settings) {
        self.send(WorkerCommand::SaveSettings(settings));
    }

    /// Sends `command` to the worker, logging if the worker is gone.
    fn send(&self, command: WorkerCommand) {
        if self.commands.send(command).is_err() {
            log::error!("storage worker is not running; change not saved");
        }
    }
}

impl Drop for StorageWorker {
    /// Asks the worker to write everything pending and waits for it to finish.
    fn drop(&mut self) {
        // The worker may already be gone (it ends on a panic); nothing to report then.
        let _ = self.commands.send(WorkerCommand::Shutdown);
        if let Some(thread) = self.thread.take()
            && thread.join().is_err()
        {
            log::error!("storage worker panicked");
        }
    }
}

/// Body of the worker thread: loads the notes, then serves save requests until
/// shutdown, writing each document once it was unchanged for `debounce`.
fn run_worker(
    store: &DocumentStore,
    debounce: Duration,
    saving_allowed: SavingAllowed,
    commands: &Receiver<WorkerCommand>,
    loaded: &Sender<LoadOutcome<NotesDocument>>,
) {
    let outcome = store.load::<NotesDocument>(Timestamp::now());
    log_load_outcome(&outcome);
    let notes_saving_allowed = outcome.allows_saving();
    // The render side may already be gone during a quick unload; that is fine.
    let _ = loaded.send(outcome);

    let mut pending_notes: Option<PendingSave<Arc<NotesDocument>>> = None;
    let mut pending_settings: Option<PendingSave<Settings>> = None;
    loop {
        let next_due = [
            pending_notes.as_ref().map(|pending| pending.due_at),
            pending_settings.as_ref().map(|pending| pending.due_at),
        ]
        .into_iter()
        .flatten()
        .min();
        let received = match next_due {
            Some(due_at) => commands.recv_timeout(due_at.saturating_duration_since(Instant::now())),
            None => commands.recv().map_err(|_| RecvTimeoutError::Disconnected),
        };

        match received {
            Ok(WorkerCommand::SaveNotes(document)) if notes_saving_allowed => {
                pending_notes = Some(PendingSave {
                    document,
                    due_at: Instant::now() + debounce,
                });
            }
            Ok(WorkerCommand::SaveSettings(settings)) if saving_allowed.settings => {
                pending_settings = Some(PendingSave {
                    document: settings,
                    due_at: Instant::now() + debounce,
                });
            }
            Ok(WorkerCommand::SaveNotes(_) | WorkerCommand::SaveSettings(_)) => {
                log::warn!("document is read-only; change not saved");
            }
            Ok(WorkerCommand::Shutdown) | Err(RecvTimeoutError::Disconnected) => {
                save_pending::<_, NotesDocument>(store, pending_notes.take(), None);
                save_pending::<_, Settings>(store, pending_settings.take(), None);
                return;
            }
            Err(RecvTimeoutError::Timeout) => {}
        }

        let now = Instant::now();
        pending_notes = save_pending::<_, NotesDocument>(store, pending_notes, Some(now));
        pending_settings = save_pending::<_, Settings>(store, pending_settings, Some(now));
    }
}

/// Writes `pending` if it is due at `now` (or unconditionally if `now` is
/// `None`) and returns what is still pending. `T` is the document itself or a
/// shared copy of it.
fn save_pending<T, D>(
    store: &DocumentStore,
    pending: Option<PendingSave<T>>,
    now: Option<Instant>,
) -> Option<PendingSave<T>>
where
    T: Borrow<D>,
    D: VersionedDocument,
{
    let pending = pending?;
    if now.is_some_and(|now| pending.due_at > now) {
        return Some(pending);
    }
    if let Err(error) = store.save::<D>(pending.document.borrow()) {
        log::error!("cannot save {}.json: {error}", D::FILE_STEM);
    }
    None
}

/// Writes a log line describing how `notes.json` was loaded.
fn log_load_outcome(outcome: &LoadOutcome<NotesDocument>) {
    match outcome {
        LoadOutcome::Missing => log::info!("no notes.json yet; starting empty"),
        LoadOutcome::Loaded {
            migrated_from: None,
            ..
        } => log::info!("notes loaded"),
        LoadOutcome::Loaded {
            migrated_from: Some(version),
            ..
        } => log::info!("notes migrated from schema version {version}"),
        LoadOutcome::Recovered {
            reason,
            preserved_as,
        } => log::error!(
            "notes.json was unreadable ({reason}); moved to {}",
            preserved_as.display()
        ),
        LoadOutcome::ReadOnly(ReadOnlyReason::NewerVersion { found }) => log::warn!(
            "notes.json has newer schema version {found}; read-only until the addon is updated"
        ),
        LoadOutcome::ReadOnly(ReadOnlyReason::Io(error)) => {
            log::error!("cannot access notes.json ({error}); read-only");
        }
    }
}

#[cfg(test)]
mod tests {
    use std::{fs, path::Path};

    use super::*;
    use crate::{
        note_tree::{Note, NoteKind, TreeNode},
        settings::QuickAccessIcon,
    };

    /// Short debounce so tests stay fast.
    const TEST_DEBOUNCE: Duration = Duration::from_millis(50);

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

        let (_worker, loaded) = StorageWorker::spawn(store, TEST_DEBOUNCE, ALL_ALLOWED)?;
        let outcome = loaded.recv_timeout(TEST_TIMEOUT)?;
        assert_eq!(outcome.into_document().as_ref(), Some(document.as_ref()));
        Ok(())
    }

    #[test]
    fn saves_only_the_latest_version_after_the_debounce() -> Result<(), Box<dyn std::error::Error>>
    {
        let directory = tempfile::tempdir()?;
        let (worker, loaded) = StorageWorker::spawn(
            DocumentStore::new(directory.path()),
            TEST_DEBOUNCE,
            ALL_ALLOWED,
        )?;
        loaded.recv_timeout(TEST_TIMEOUT)?;

        worker.save_notes(document_with_note("first"));
        let latest = document_with_note("second");
        worker.save_notes(Arc::clone(&latest));
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
            Duration::from_secs(3600),
            ALL_ALLOWED,
        )?;
        loaded.recv_timeout(TEST_TIMEOUT)?;
        let document = document_with_note("unsaved");
        worker.save_notes(Arc::clone(&document));
        let settings = Settings {
            quick_access_icon: QuickAccessIcon::Hidden,
            ..Settings::DEFAULT
        };
        worker.save_settings(settings);

        drop(worker);

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
    fn read_only_documents_are_never_written() -> Result<(), Box<dyn std::error::Error>> {
        let directory = tempfile::tempdir()?;
        let store = DocumentStore::new(directory.path());
        let newer = r#"{"schema_version": 999}"#;
        fs::write(store.path_of::<NotesDocument>(), newer)?;
        let (worker, loaded) = StorageWorker::spawn(
            store.clone(),
            TEST_DEBOUNCE,
            SavingAllowed { settings: false },
        )?;
        assert!(!loaded.recv_timeout(TEST_TIMEOUT)?.allows_saving());

        worker.save_notes(document_with_note("ignored"));
        worker.save_settings(Settings::DEFAULT);
        drop(worker);

        assert_eq!(fs::read_to_string(store.path_of::<NotesDocument>())?, newer);
        assert!(!store.path_of::<Settings>().exists());
        Ok(())
    }
}
