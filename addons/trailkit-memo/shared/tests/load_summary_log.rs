//! A normal session logs three lines: `session_start`, one `load_finished`
//! summary and `session_end` (ROADMAP R92, Q219). Runs the real `trailkit-log`
//! logger, so it lives in its own test binary.

use std::{fs, sync::Arc, time::Duration};

use jiff::{Timestamp, tz::Offset};
use trailkit_log::{LogConfig, log_files, start};
use trailkit_memo_shared::{
    note_tree::{Note, NoteKind, NotesDocument, TreeNode},
    settings::TextScale,
    storage_worker::{SavingAllowed, SettingsSummary, StorageWorker},
};
use trailkit_storage::{AddonName, DocumentStore, TrailkitDirectory};

/// Text that must never appear in the log.
const SECRET_TITLE: &str = "Secret raid plan 7f3a";

/// UTC as the local offset.
fn utc(_time: Timestamp) -> Offset {
    Offset::UTC
}

/// Drops texts meant for `Nexus.log`.
fn ignore(_level: log::Level, _text: &str) {}

#[test]
fn a_normal_session_is_three_lines() -> Result<(), Box<dyn std::error::Error>> {
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

    let data = directory.addon_data(memo);
    fs::create_dir_all(&data)?;
    let note = || Note::new(SECRET_TITLE.to_owned(), NoteKind::Normal, Timestamp::now());
    DocumentStore::new(&data).save(&NotesDocument {
        root: vec![TreeNode::Note(note()), TreeNode::Note(note())],
        trash: Vec::new(),
    })?;

    let (worker, loaded) = StorageWorker::spawn(
        DocumentStore::new(&data),
        Duration::from_millis(1),
        SavingAllowed { settings: true },
        SettingsSummary {
            missing: true,
            text_scale: TextScale::default(),
            elapsed: Duration::ZERO,
        },
    )?;
    loaded.recv_timeout(Duration::from_secs(5))?;
    // A successful save is routine and adds no line.
    worker.saver().save_notes(Arc::new(NotesDocument {
        root: vec![TreeNode::Note(note())],
        trash: Vec::new(),
    }));
    drop(worker);
    drop(session);

    let log = log_files(&directory.logs(), memo)?
        .iter()
        .map(fs::read_to_string)
        .collect::<Result<String, _>>()?;
    let events: Vec<&str> = log
        .lines()
        .filter_map(|line| line.split(' ').nth(4))
        .collect();
    assert_eq!(
        events,
        ["session_start", "load_finished", "session_end"],
        "{log}"
    );
    let summary = log.lines().nth(1).unwrap_or_default();
    assert!(
        summary
            .contains(" notes=2 drafts=0 missing=settings.json,ui_state.json text_scale=1.125 ms="),
        "{summary}"
    );
    assert!(!log.contains(SECRET_TITLE), "{log}");
    Ok(())
}
