//! A panic writes a `panic` line into the log file and still runs the hook
//! that was installed before (nexus-rs' hook in an addon). Its own test
//! binary, because the logger and the panic hook are process-wide.

use std::{
    fs, panic,
    sync::atomic::{AtomicBool, Ordering},
    thread,
};

use jiff::{Timestamp, tz::Offset};
use trailkit_log::{LogConfig, start};
use trailkit_storage::{AddonName, TrailkitDirectory};

/// Set by the hook installed before `start`.
static PREVIOUS_HOOK_RAN: AtomicBool = AtomicBool::new(false);

/// UTC as the local offset.
fn utc(_time: Timestamp) -> Offset {
    Offset::UTC
}

/// Drops texts meant for `Nexus.log`.
fn ignore(_level: log::Level, _text: &str) {}

#[test]
fn a_panic_is_logged_and_the_previous_hook_still_runs() -> Result<(), Box<dyn std::error::Error>> {
    panic::set_hook(Box::new(|_| {
        PREVIOUS_HOOK_RAN.store(true, Ordering::SeqCst)
    }));
    let folder = tempfile::tempdir()?;
    let directory = TrailkitDirectory::new(folder.path().join("trailkit"));
    let session = start(LogConfig {
        directory: directory.clone(),
        addon: AddonName::new("memo"),
        version: "0.0.0",
        local_offset: utc,
        forward: ignore,
        session_fields: Vec::new(),
    })?;
    let log_path = directory.logs().join("memo.log");
    // Wait until the writer has opened the file, so the hook writes directly.
    for _ in 0..500 {
        if fs::read_to_string(&log_path).is_ok_and(|log| log.contains("session_start")) {
            break;
        }
        thread::sleep(std::time::Duration::from_millis(10));
    }

    let result = thread::Builder::new()
        .name("storage-worker".to_owned())
        .spawn(|| panic!("index out of range"))?
        .join();
    assert!(result.is_err(), "the thread should have panicked");
    drop(session);

    assert!(
        PREVIOUS_HOOK_RAN.load(Ordering::SeqCst),
        "previous hook did not run"
    );
    let log = fs::read_to_string(&log_path)?;
    let line = log
        .lines()
        .find(|line| line.contains(" panic "))
        .ok_or_else(|| format!("no panic line in:\n{log}"))?;
    assert!(line.contains(" ERROR "), "{line}");
    assert!(line.contains(" at=panic_hook.rs:"), "{line}");
    assert!(line.contains(" thread=storage-worker "), "{line}");
    assert!(line.ends_with(r#"message="index out of range""#), "{line}");
    Ok(())
}
