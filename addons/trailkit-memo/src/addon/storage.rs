//! The addon's side of the storage worker: starting it at load, handing the
//! loaded notes to the window, and restarting a dead worker on "Try again"
//! (memo M18, M30, Q151). Never blocks the render thread.

use super::*;

/// Reads `settings.json` and starts the storage worker, which loads the
/// notes in the background. Returns the settings to use (defaults if the file
/// is missing or unusable). Problems are logged; the addon keeps working.
pub(super) fn start_storage(trailkit_directory: Option<TrailkitDirectory>) -> Settings {
    // Without the folder `load` already reported the problem.
    let Some(trailkit_directory) = trailkit_directory else {
        return *SETTINGS.get_or_init(|| Settings::DEFAULT);
    };
    let directory = trailkit_directory.addon_data(ADDON);
    let store = DocumentStore::new(directory.clone());

    let started = Instant::now();
    let settings_outcome = store.load::<Settings>(Timestamp::now());
    let settings_elapsed = started.elapsed();
    log_load_outcome(&settings_outcome, settings_elapsed);
    let settings_missing = matches!(settings_outcome, LoadOutcome::Missing);
    let saving_allowed = SavingAllowed {
        settings: settings_outcome.allows_saving(),
    };
    let settings = settings_outcome.into_document().unwrap_or_default();
    let settings = *SETTINGS.get_or_init(|| settings);

    let settings_summary = SettingsSummary {
        missing: settings_missing,
        text_scale: settings.text_scale,
        elapsed: settings_elapsed,
    };
    match StorageWorker::spawn(
        store,
        WorkerTiming::DEFAULT,
        saving_allowed,
        settings_summary,
    ) {
        Ok((worker, loaded)) => {
            let saver = worker.saver();
            if let Ok(mut slot) = STORAGE_WORKER.lock() {
                *slot = Some(worker);
            }
            if let Ok(mut notes) = NOTES.lock() {
                *notes = NotesState::Loading {
                    loaded,
                    saver,
                    directory,
                };
            }
        }
        Err(error) => log::error!(error:% = ErrorChain(&error); "worker_start_failed"),
    }
    settings
}

/// Restarts the stopped storage worker when the window asks for it ("Try
/// again", memo Q151). Never waits: if `unload` holds the worker, nothing
/// happens and the user may ask again.
pub(super) fn restart_storage_worker_if_asked(window: &mut MemoWindow) {
    if !window.take_worker_restart_request() {
        return;
    }
    let Ok(mut slot) = STORAGE_WORKER.try_lock() else {
        window.storage_restart_failed();
        return;
    };
    let Some(worker) = slot.as_mut() else {
        window.storage_restart_failed();
        return;
    };
    match worker.restart() {
        Ok(connection) => window.reconnect_storage(connection),
        Err(error) => {
            log::warn!(error:% = ErrorChain(&error); "worker_restart_failed");
            window.storage_restart_failed();
        }
    }
}

/// Moves the notes from [`NotesState::Loading`] to [`NotesState::Ready`] once
/// the storage worker has delivered them. Never blocks.
pub(super) fn receive_loaded_notes(notes: &mut NotesState) {
    let NotesState::Loading {
        loaded,
        saver,
        directory,
    } = notes
    else {
        return;
    };
    *notes = match loaded.try_recv() {
        Ok(LoadedDocuments {
            notes: outcome,
            ui_state,
            drafts,
            reports,
        }) => {
            let (document, problem) = document_and_problem(outcome);
            NotesState::Ready(Box::new(MemoWindow::new(
                document,
                problem,
                ui_state,
                SaveConnection {
                    saver: saver.clone(),
                    reports,
                },
                directory.clone(),
                drafts,
                settings(),
            )))
        }
        Err(TryRecvError::Empty) => return,
        Err(TryRecvError::Disconnected) => NotesState::Unavailable,
    };
}

/// Splits the worker's load result into the notes to show and a problem hint.
fn document_and_problem(
    outcome: LoadOutcome<NotesDocument>,
) -> (NotesDocument, Option<NotesProblem>) {
    match outcome {
        LoadOutcome::Missing => (NotesDocument::default(), None),
        LoadOutcome::Loaded { document, .. } => (document, None),
        LoadOutcome::Recovered { preserved_as, .. } => (
            NotesDocument::default(),
            Some(NotesProblem::Recovered(display_path_of_safety_copy(
                &preserved_as,
            ))),
        ),
        LoadOutcome::ReadOnly(ReadOnlyReason::NewerVersion { .. }) => {
            (NotesDocument::default(), Some(NotesProblem::NewerVersion))
        }
        LoadOutcome::ReadOnly(ReadOnlyReason::Io(_)) => {
            (NotesDocument::default(), Some(NotesProblem::Inaccessible))
        }
    }
}

/// Formats a safety copy's path relative to the addon folder
/// (`backups/notes.corrupt-….json`), computed once instead of every frame.
fn display_path_of_safety_copy(path: &Path) -> String {
    path.file_name().map_or_else(
        || path.display().to_string(),
        |name| format!("{BACKUPS_DIRECTORY_NAME}/{}", name.to_string_lossy()),
    )
}
