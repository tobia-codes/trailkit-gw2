//! A failed save ends up in memo's log file as `save_failed` with the file
//! name and the error, but without the note's content (ROADMAP R92, Q194).
//! Runs the real `trailkit-log` logger, so it lives in its own test binary.

use std::{fs, sync::Arc, time::Duration};

use jiff::{Timestamp, tz::Offset};
use trailkit_log::{LogConfig, start};
use trailkit_memo_shared::{
    note_tree::{Note, NoteKind, NotesDocument, TreeNode},
    storage_worker::{SavingAllowed, StorageWorker},
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
        Duration::from_secs(3600),
        SavingAllowed { settings: true },
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

    let log = fs::read_to_string(directory.logs().join("memo.log"))?;
    let failure = log
        .lines()
        .find(|line| line.contains(" save_failed "))
        .ok_or_else(|| format!("no save_failed line in:\n{log}"))?;
    assert!(failure.contains(" ERROR "), "{failure}");
    assert!(failure.contains(" file=notes.json "), "{failure}");
    assert!(failure.contains(" error="), "{failure}");
    assert!(failure.contains(" ms="), "{failure}");
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
