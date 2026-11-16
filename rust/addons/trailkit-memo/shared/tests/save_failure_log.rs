//! A failed save ends up in memo's log file as `save_failed` (WARN) with the
//! file name, stage and error, and the lost change as `save_gave_up` (ERROR),
//! but without the note's content (ROADMAP R92, Q166, Q194).
//! Runs the real `trailkit-log` logger, so it lives in its own test binary.

use std::{fs, sync::Arc, time::Duration};

use jiff::{Timestamp, tz::Offset};
use trailkit_log::{LogConfig, log_files, start};
use trailkit_memo_shared::{
    note_tree::{Note, NoteKind, NotesDocument, TreeNode},
    storage_worker::{SavingAllowed, SettingsSummary, StorageWorker, WorkerTiming},
};
use trailkit_storage::{AddonName, DocumentStore, TrailkitDirectory};

/// Text that must never appear in the log.
const SECRET_TITLE: &str = "Secret raid plan 7f3a";

/// Text that must never appear in the log.
const SECRET_BODY: &str = "Meet at Lion's Arch 9b2c";

/// UTC as the local offset.
fn utc(_time: Timestamp) -> Offset {
    Offset::UTC
}

/// Drops texts meant for `Nexus.log`.
fn ignore(_level: log::Level, _text: &str) {}

#[test]
fn a_failed_save_is_logged_without_note_content() -> Result<(), Box<dyn std::error::Error>> {
    let folder = tempfile::tempdir()?;
    let directory = TrailkitDirectory::new(folder.path().join("trailkit"));
    let memo = AddonName::new("memo");
    let session = start(LogConfig {
        directory: directory.clone(),
        addon: memo,
        version: "0.0.0",
        local_offset: utc,
        forward: ignore,
        session_fields: Vec::new(),
    })?;

    // A folder where the temporary file must go makes every notes save fail.
    let data = directory.addon_data(memo);
    fs::create_dir_all(data.join("notes.json.tmp"))?;
    let (worker, loaded) = StorageWorker::spawn(
        DocumentStore::new(&data),
        WorkerTiming {
            debounce: Duration::from_secs(3600),
            retry_delay: Duration::from_secs(3600),
        },
        SavingAllowed { settings: true },
        SettingsSummary::default(),
    )?;
    loaded.recv_timeout(Duration::from_secs(5))?;
    let mut note = Note::new(SECRET_TITLE.to_owned(), NoteKind::Normal, Timestamp::now());
    note.body = SECRET_BODY.to_owned();
    worker.saver().save_notes(Arc::new(NotesDocument {
        root: vec![TreeNode::Note(note)],
        trash: Vec::new(),
    }));
    // Shutdown writes the pending save, which fails.
    drop(worker);
    drop(session);

    let log = log_files(&directory.logs(), AddonName::new("memo"))?
        .iter()
        .map(fs::read_to_string)
        .collect::<Result<String, _>>()?;
    let failure = log
        .lines()
        .find(|line| line.contains(" save_failed "))
        .ok_or_else(|| format!("no save_failed line in:\n{log}"))?;
    assert!(failure.contains(" WARN "), "{failure}");
    for field in [
        " file=notes.json ",
        " attempt=1 ",
        " stage=write_temp ",
        " kind=",
        " os_error=",
        " bytes=",
        " unsaved_for=",
        " ms=",
        " error=",
    ] {
        assert!(failure.contains(field), "{field} missing: {failure}");
    }
    let gave_up = log
        .lines()
        .find(|line| line.contains(" save_gave_up "))
        .ok_or_else(|| format!("no save_gave_up line in:\n{log}"))?;
    assert!(gave_up.contains(" ERROR "), "{gave_up}");
    assert!(gave_up.contains(" file=notes.json "), "{gave_up}");
    for secret in [SECRET_TITLE, SECRET_BODY, "7f3a", "9b2c"] {
        assert!(!log.contains(secret), "log contains {secret:?}:\n{log}");
    }
    let temporary_folder = folder.path().to_string_lossy().into_owned();
    assert!(
        !log.contains(&temporary_folder),
        "log contains a path:\n{log}"
    );
    Ok(())
}
