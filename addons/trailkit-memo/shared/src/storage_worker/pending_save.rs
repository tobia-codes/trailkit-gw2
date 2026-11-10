//! The save cycle of one document (memo M180, Q149, Q162, Q166): after the
//! debounce the worker writes; a failed write is repeated at most
//! [`MAX_ATTEMPTS`] times per cycle, [`RETRY_DELAY`] apart, then the worker
//! gives up and keeps the document until a new change or a manual write.

use std::{
    sync::Arc,
    time::{Duration, Instant},
};

use trailkit_storage::{DocumentStore, StorageError, VersionedDocument};

use super::document_file_name;
use crate::{
    drafts::DraftsDocument, note_tree::NotesDocument, settings::Settings, ui_state::UiState,
};

/// Delay after the last change before a document is written (M18).
pub const SAVE_DEBOUNCE: Duration = Duration::from_secs(2);

/// Delay between two automatic writes after a failure (Q149).
pub const RETRY_DELAY: Duration = Duration::from_secs(5);

/// Automatic writes per cycle before the worker gives up (Q149).
pub const MAX_ATTEMPTS: u8 = 3;

/// The worker's delays; [`WorkerTiming::DEFAULT`] outside of tests.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct WorkerTiming {
    /// Delay after the last change before the first write.
    pub debounce: Duration,
    /// Delay between two automatic writes after a failure.
    pub retry_delay: Duration,
}

impl WorkerTiming {
    /// [`SAVE_DEBOUNCE`] and [`RETRY_DELAY`].
    pub const DEFAULT: Self = Self {
        debounce: SAVE_DEBOUNCE,
        retry_delay: RETRY_DELAY,
    };
}

/// Whether the documents the user sees are on disk, as the worker reports it
/// to the UI (M180). Only notes and drafts with content count (Q148); other
/// documents fail silently into the log.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum SaveHealth {
    /// Everything shown is saved or waiting for its first write.
    #[default]
    Healthy,
    /// A write failed; the worker tries again automatically.
    Retrying {
        /// Failed writes in the current cycle (0 right after a new change
        /// started a fresh cycle once the worker had given up).
        failed_attempts: u8,
    },
    /// Every automatic write of the cycle failed; only a new change or a
    /// manual write tries again.
    GaveUp,
}

impl SaveHealth {
    /// The more severe of `self` and `other`.
    #[must_use]
    pub fn worse(self, other: Self) -> Self {
        match (self, other) {
            (Self::GaveUp, _) | (_, Self::GaveUp) => Self::GaveUp,
            (
                Self::Retrying { failed_attempts },
                Self::Retrying {
                    failed_attempts: other_attempts,
                },
            ) => Self::Retrying {
                failed_attempts: failed_attempts.max(other_attempts),
            },
            (retrying @ Self::Retrying { .. }, Self::Healthy)
            | (Self::Healthy, retrying @ Self::Retrying { .. }) => retrying,
            (Self::Healthy, Self::Healthy) => Self::Healthy,
        }
    }
}

/// A document the worker writes.
pub(super) trait WorkerDocument {
    /// The document type on disk, for its file name.
    type Stored: VersionedDocument;

    /// Writes the document (or deletes its file, see [`DraftsDocument`]).
    ///
    /// # Errors
    ///
    /// Returns the [`StorageError`] of the failed write.
    fn write(&self, store: &DocumentStore) -> Result<(), StorageError>;

    /// Whether a failed write is shown to the user (Q148).
    fn failures_shown(&self) -> bool;
}

impl WorkerDocument for Arc<NotesDocument> {
    type Stored = NotesDocument;

    /// Saves `notes.json`.
    fn write(&self, store: &DocumentStore) -> Result<(), StorageError> {
        store.save(self.as_ref())
    }

    /// Always: lost notes are the worst case.
    fn failures_shown(&self) -> bool {
        true
    }
}

impl WorkerDocument for Settings {
    type Stored = Self;

    /// Saves `settings.json`.
    fn write(&self, store: &DocumentStore) -> Result<(), StorageError> {
        store.save(self)
    }

    /// Never: settings fail into the log only (Q147).
    fn failures_shown(&self) -> bool {
        false
    }
}

impl WorkerDocument for UiState {
    type Stored = Self;

    /// Saves `ui_state.json`.
    fn write(&self, store: &DocumentStore) -> Result<(), StorageError> {
        store.save(self)
    }

    /// Never: window state fails into the log only (Q147).
    fn failures_shown(&self) -> bool {
        false
    }
}

impl WorkerDocument for DraftsDocument {
    type Stored = Self;

    /// Saves `drafts.json`, or deletes it when there are no drafts, so no
    /// empty file lingers on disk (M55).
    fn write(&self, store: &DocumentStore) -> Result<(), StorageError> {
        if self.drafts.is_empty() {
            store.delete::<Self>()
        } else {
            store.save(self)
        }
    }

    /// Only with content; failing to delete the empty file is log-only (Q148).
    fn failures_shown(&self) -> bool {
        !self.drafts.is_empty()
    }
}

/// When [`Slot::write`] writes.
#[derive(Debug, Clone, Copy)]
pub(super) enum WriteMode {
    /// Only if the next automatic write is due at this time.
    Due(Instant),
    /// Now, even after giving up (a manual write or "Übernehmen", Q158).
    Now,
    /// Now, for the last time: the worker ends (Q150).
    Final,
}

/// The result of one write of a slot.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) struct WriteResult {
    /// Whether the document is on disk now.
    pub succeeded: bool,
    /// Whether the document's failures are shown to the user.
    pub shown: bool,
}

/// A document that is not on disk yet.
struct PendingSave<T> {
    /// The newest version of the document.
    document: T,
    /// When the next automatic write happens; `None` after giving up.
    due_at: Option<Instant>,
    /// Failed writes in the current cycle.
    cycle_failures: u8,
    /// Failed writes since the document was last on disk.
    total_failures: u32,
    /// When the oldest change that is not on disk was made.
    unsaved_since: Instant,
}

/// One document's pending save, if any.
pub(super) struct Slot<T>(Option<PendingSave<T>>);

impl<T> Default for Slot<T> {
    /// Nothing pending.
    fn default() -> Self {
        Self(None)
    }
}

impl<T: WorkerDocument> Slot<T> {
    /// Takes a new version of the document. A first change or a change in a
    /// cycle without failures (re)starts the debounce; a change while the
    /// worker retries only replaces the document; a change after giving up
    /// starts a fresh cycle (Q162).
    pub fn replace(&mut self, document: T, now: Instant, timing: WorkerTiming) {
        let debounced = Some(now + timing.debounce);
        match &mut self.0 {
            None => {
                self.0 = Some(PendingSave {
                    document,
                    due_at: debounced,
                    cycle_failures: 0,
                    total_failures: 0,
                    unsaved_since: now,
                });
            }
            Some(pending) => {
                pending.document = document;
                if pending.due_at.is_none() {
                    pending.cycle_failures = 0;
                    pending.due_at = debounced;
                } else if pending.cycle_failures == 0 {
                    pending.due_at = debounced;
                }
            }
        }
    }

    /// When the next automatic write is due, if one is planned.
    pub fn due_at(&self) -> Option<Instant> {
        self.0.as_ref().and_then(|pending| pending.due_at)
    }

    /// How the user should see this document's state.
    pub fn health(&self) -> SaveHealth {
        let Some(pending) = &self.0 else {
            return SaveHealth::Healthy;
        };
        if !pending.document.failures_shown() || pending.total_failures == 0 {
            return SaveHealth::Healthy;
        }
        if pending.due_at.is_none() {
            SaveHealth::GaveUp
        } else {
            SaveHealth::Retrying {
                failed_attempts: pending.cycle_failures,
            }
        }
    }

    /// Writes the document if `mode` asks for it and logs the outcome:
    /// `save_failed` (WARN) per failed write, `save_gave_up` (ERROR) when the
    /// cycle or the worker ends without success, `save_recovered` (INFO) when
    /// a write succeeds after failures. Returns `None` if nothing was written.
    pub fn write(
        &mut self,
        store: &DocumentStore,
        mode: WriteMode,
        timing: WorkerTiming,
    ) -> Option<WriteResult> {
        let pending = self.0.as_mut()?;
        if let WriteMode::Due(now) = mode
            && pending.due_at.is_none_or(|due_at| due_at > now)
        {
            return None;
        }
        let shown = pending.document.failures_shown();
        let started = Instant::now();
        let result = pending.document.write(store);
        let ms = started.elapsed().as_millis();
        let file = document_file_name::<T::Stored>();
        let file = file.as_str();
        let unsaved_for = pending.unsaved_since.elapsed().as_secs();
        let attempts = pending.total_failures + 1;
        let Err(error) = result else {
            if pending.total_failures > 0 {
                log::info!(file, attempts, unsaved_for; "save_recovered");
            }
            self.0 = None;
            return Some(WriteResult {
                succeeded: true,
                shown,
            });
        };
        log_save_failed(file, attempts, &error, ms, unsaved_for);
        pending.total_failures = attempts;
        pending.cycle_failures = pending.cycle_failures.saturating_add(1);
        let gave_up_before = pending.due_at.is_none();
        match mode {
            WriteMode::Final => {
                log::error!(file, attempts, unsaved_for; "save_gave_up");
                self.0 = None;
            }
            // One write per manual click; no new automatic cycle.
            _ if gave_up_before => {}
            _ if pending.cycle_failures >= MAX_ATTEMPTS => {
                log::error!(file, attempts, unsaved_for; "save_gave_up");
                pending.due_at = None;
            }
            _ => pending.due_at = Some(Instant::now() + timing.retry_delay),
        }
        Some(WriteResult {
            succeeded: false,
            shown,
        })
    }
}

/// Logs one failed write as `save_failed` (WARN, Q166): the file, the
/// attempt since the last successful write, the stage, the OS error kind
/// and raw code (0 if the OS gave none), the document size (0 when
/// deleting), how long the change waits (seconds), the duration and the
/// whole error. A document that cannot be serialized has no disk details.
/// Never content or paths.
fn log_save_failed(file: &str, attempt: u32, error: &StorageError, ms: u128, unsaved_for: u64) {
    let error_chain = trailkit_log::ErrorChain(error);
    let (Some(stage), Some(io_error)) = (error.stage(), error.io_error()) else {
        log::warn!(file, attempt, stage = "serialize", unsaved_for, ms, error:% = error_chain; "save_failed");
        return;
    };
    let stage = stage.as_str();
    let kind = format!("{:?}", io_error.kind());
    let os_error = io_error.raw_os_error().unwrap_or_default();
    let bytes = error.bytes().unwrap_or_default();
    log::warn!(
        file,
        attempt,
        stage,
        kind = kind.as_str(),
        os_error,
        bytes,
        unsaved_for,
        ms,
        error:% = error_chain;
        "save_failed"
    );
}

#[cfg(test)]
mod tests {
    use std::{
        io,
        sync::atomic::{AtomicBool, AtomicUsize, Ordering},
    };

    use trailkit_storage::{WriteError, WriteStage};

    use super::*;

    /// Test delays: the debounce and retry delay never pass by themselves.
    const TIMING: WorkerTiming = WorkerTiming {
        debounce: Duration::from_secs(2),
        retry_delay: Duration::from_secs(5),
    };

    /// The disk as the test document sees it.
    #[derive(Default)]
    struct FakeDisk {
        /// Writes so far.
        writes: AtomicUsize,
        /// Whether writes fail.
        failing: AtomicBool,
    }

    /// A document that counts its writes and fails while the disk fails.
    struct CountingDocument(Arc<FakeDisk>);

    impl WorkerDocument for CountingDocument {
        type Stored = UiState;

        /// Counts the write; fails at [`WriteStage::Rename`] while failing.
        fn write(&self, _store: &DocumentStore) -> Result<(), StorageError> {
            self.0.writes.fetch_add(1, Ordering::SeqCst);
            if self.0.failing.load(Ordering::SeqCst) {
                Err(StorageError::Write {
                    bytes: 1,
                    source: WriteError {
                        stage: WriteStage::Rename,
                        source: io::Error::from(io::ErrorKind::PermissionDenied),
                    },
                })
            } else {
                Ok(())
            }
        }

        /// Shown, like the notes.
        fn failures_shown(&self) -> bool {
            true
        }
    }

    /// A slot with a pending document on a failing disk, and the disk.
    fn failing_slot(now: Instant) -> (Slot<CountingDocument>, Arc<FakeDisk>) {
        let disk = Arc::new(FakeDisk::default());
        disk.failing.store(true, Ordering::SeqCst);
        let mut slot = Slot::default();
        slot.replace(CountingDocument(Arc::clone(&disk)), now, TIMING);
        (slot, disk)
    }

    /// Runs every automatic write until the slot gives up.
    fn run_until_given_up(slot: &mut Slot<CountingDocument>, store: &DocumentStore) {
        while let Some(due_at) = slot.due_at() {
            slot.write(store, WriteMode::Due(due_at + TIMING.retry_delay), TIMING);
        }
    }

    #[test]
    fn nothing_is_written_before_the_debounce() {
        let store = DocumentStore::new("unused");
        let now = Instant::now();
        let (mut slot, disk) = failing_slot(now);
        assert_eq!(slot.write(&store, WriteMode::Due(now), TIMING), None);
        assert_eq!(disk.writes.load(Ordering::SeqCst), 0);
    }

    #[test]
    fn gives_up_after_three_writes_and_then_writes_no_more() {
        let store = DocumentStore::new("unused");
        let (mut slot, disk) = failing_slot(Instant::now());
        run_until_given_up(&mut slot, &store);
        assert_eq!(
            disk.writes.load(Ordering::SeqCst),
            usize::from(MAX_ATTEMPTS)
        );
        assert_eq!(slot.health(), SaveHealth::GaveUp);

        let much_later = Instant::now() + Duration::from_secs(3600);
        assert_eq!(slot.write(&store, WriteMode::Due(much_later), TIMING), None);
        assert_eq!(
            disk.writes.load(Ordering::SeqCst),
            usize::from(MAX_ATTEMPTS)
        );
    }

    #[test]
    fn a_manual_write_after_giving_up_writes_once_and_starts_no_cycle() {
        let store = DocumentStore::new("unused");
        let (mut slot, disk) = failing_slot(Instant::now());
        run_until_given_up(&mut slot, &store);

        let result = slot.write(&store, WriteMode::Now, TIMING);
        assert_eq!(
            result,
            Some(WriteResult {
                succeeded: false,
                shown: true
            })
        );
        assert_eq!(
            disk.writes.load(Ordering::SeqCst),
            usize::from(MAX_ATTEMPTS) + 1
        );
        assert_eq!(slot.due_at(), None);
        assert_eq!(slot.health(), SaveHealth::GaveUp);
    }

    #[test]
    fn a_new_change_after_giving_up_starts_a_fresh_cycle() {
        let store = DocumentStore::new("unused");
        let (mut slot, disk) = failing_slot(Instant::now());
        run_until_given_up(&mut slot, &store);

        let now = Instant::now();
        slot.replace(CountingDocument(Arc::clone(&disk)), now, TIMING);
        assert_eq!(slot.due_at(), Some(now + TIMING.debounce));
        assert_eq!(slot.health(), SaveHealth::Retrying { failed_attempts: 0 });
        run_until_given_up(&mut slot, &store);
        assert_eq!(
            disk.writes.load(Ordering::SeqCst),
            2 * usize::from(MAX_ATTEMPTS)
        );
    }

    #[test]
    fn a_change_while_retrying_keeps_the_retry_time() {
        let store = DocumentStore::new("unused");
        let now = Instant::now();
        let (mut slot, disk) = failing_slot(now);
        slot.write(&store, WriteMode::Now, TIMING);
        let retry_at = slot.due_at();

        slot.replace(CountingDocument(disk), Instant::now(), TIMING);
        assert_eq!(slot.due_at(), retry_at);
        assert_eq!(slot.health(), SaveHealth::Retrying { failed_attempts: 1 });
    }

    #[test]
    fn a_successful_write_after_failures_clears_the_slot() {
        let store = DocumentStore::new("unused");
        let (mut slot, disk) = failing_slot(Instant::now());
        slot.write(&store, WriteMode::Now, TIMING);
        disk.failing.store(false, Ordering::SeqCst);
        let result = slot.write(&store, WriteMode::Now, TIMING);
        assert_eq!(
            result,
            Some(WriteResult {
                succeeded: true,
                shown: true
            })
        );
        assert_eq!(slot.health(), SaveHealth::Healthy);
        assert_eq!(slot.due_at(), None);
    }

    #[test]
    fn the_final_write_ends_the_slot_even_on_failure() {
        let store = DocumentStore::new("unused");
        let (mut slot, _disk) = failing_slot(Instant::now());
        slot.write(&store, WriteMode::Final, TIMING);
        assert_eq!(slot.health(), SaveHealth::Healthy);
        assert_eq!(slot.write(&store, WriteMode::Now, TIMING), None);
    }

    #[test]
    fn empty_drafts_fail_silently() {
        assert!(!DraftsDocument::default().failures_shown());
        assert!(!Settings::DEFAULT.failures_shown());
        assert!(!UiState::default().failures_shown());
    }

    #[test]
    fn the_worse_health_wins() {
        let retrying = SaveHealth::Retrying { failed_attempts: 1 };
        assert_eq!(SaveHealth::Healthy.worse(retrying), retrying);
        assert_eq!(retrying.worse(SaveHealth::GaveUp), SaveHealth::GaveUp);
        assert_eq!(
            retrying.worse(SaveHealth::Retrying { failed_attempts: 2 }),
            SaveHealth::Retrying { failed_attempts: 2 }
        );
    }
}
