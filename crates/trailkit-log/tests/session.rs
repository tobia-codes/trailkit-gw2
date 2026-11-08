//! Runs one complete log session through the real global logger. Only one
//! test lives here, because a process can install a logger only once.

use std::{fs, io, sync::Mutex};

use jiff::{Timestamp, tz::Offset};
use trailkit_log::{ErrorChain, LogConfig, start};
use trailkit_storage::{AddonName, FileName, TrailkitDirectory};

/// Target of the lines that stand for our own crates (module paths start
/// with `trailkit`; this test binary's do not).
const OWN_TARGET: &str = "trailkit_memo_test";

/// Texts forwarded to `Nexus.log`.
static FORWARDED: Mutex<Vec<String>> = Mutex::new(Vec::new());

/// Records a forwarded text.
fn record_forwarded(level: log::Level, text: &str) {
    if let Ok(mut forwarded) = FORWARDED.lock() {
        forwarded.push(format!("{level} {text}"));
    }
}

/// UTC as the local offset.
fn utc(_time: Timestamp) -> Offset {
    Offset::UTC
}

/// The event (fifth word) of every line.
fn events(content: &str) -> Vec<&str> {
    content
        .lines()
        .filter_map(|line| line.split(' ').nth(4))
        .collect()
}

#[test]
fn writes_a_whole_session_without_paths_or_content() -> Result<(), Box<dyn std::error::Error>> {
    let folder = tempfile::tempdir()?;
    let directory = TrailkitDirectory::new(folder.path().join("trailkit"));
    let session = start(LogConfig {
        directory: directory.clone(),
        addon: AddonName::new("memo"),
        version: "0.3.0",
        local_offset: utc,
        forward: record_forwarded,
        session_fields: vec![("language", "de".to_owned()), ("theme", "dark".to_owned())],
    })?;

    let note_path = folder
        .path()
        .join("trailkit")
        .join("memo")
        .join("notes.json");
    let error = io::Error::other(format!("cannot open {}", r"C:\Users\Jo\notes.json"));
    log::info!(target: OWN_TARGET, file:% = FileName::of(&note_path), ms = 4; "load_finished");
    log::warn!(target: OWN_TARGET, file:% = FileName::of(&note_path), error:% = ErrorChain(&error); "save_failed");
    log::info!(target: "some_dependency", "dropped below WARN");
    log::warn!(target: "some_dependency", "free text from a dependency");
    log::debug!(target: OWN_TARGET, file = "notes.json"; "save_finished");
    drop(session);

    let content = fs::read_to_string(directory.logs().join("memo.log"))?;
    let expected_debug: &[&str] = if cfg!(debug_assertions) {
        &["save_finished"]
    } else {
        &[]
    };
    let expected: Vec<&str> = ["session_start", "load_finished", "save_failed", "message"]
        .into_iter()
        .chain(expected_debug.iter().copied())
        .chain(["session_end"])
        .collect();
    assert_eq!(events(&content), expected, "{content}");

    let first = content.lines().next().unwrap_or_default();
    assert!(first.contains(" build="), "{first}");
    assert!(first.contains(" commit="), "{first}");
    assert!(first.ends_with(" language=de theme=dark"), "{first}");
    assert!(content.contains(" file=notes.json "), "{content}");
    assert!(
        content.contains(r#"text="free text from a dependency""#),
        "{content}"
    );
    let temporary_folder = folder.path().to_string_lossy().into_owned();
    assert!(!content.contains(&temporary_folder), "{content}");
    assert!(!content.contains("Users"), "{content}");

    let forwarded = FORWARDED
        .lock()
        .map(|forwarded| forwarded.clone())
        .unwrap_or_default();
    assert_eq!(forwarded.len(), 2, "{forwarded:?}");
    assert!(
        forwarded[0].starts_with("WARN addon=memo version=0.3.0 save_failed at="),
        "{forwarded:?}"
    );
    Ok(())
}
