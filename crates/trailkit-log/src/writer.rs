//! The background thread that owns the log file (nexus-addon-dev skill §2:
//! no file I/O on the render thread).
//!
//! At start it removes old entries ([`retain`]), notes an unclean previous
//! session, then appends every line it receives until it is told to stop.
//! Lines are written unbuffered, so the operating system has them even if
//! the game is killed a moment later.

use std::{
    borrow::Cow,
    fs::{self, File, OpenOptions},
    io::{self, Write as _},
    path::Path,
    sync::{Mutex, PoisonError, mpsc::Receiver},
};

use jiff::{SignedDuration, Timestamp};
use trailkit_storage::write_atomically;

use crate::{
    events::{EVENT_LOG_FULL, EVENT_LOG_UNAVAILABLE, EVENT_PREVIOUS_SESSION_UNCLEAN},
    line::{LineContent, LineIdentity, format_body, format_line},
    retention::retain,
};

/// Largest size of a log file; reaching it ends the session's log with
/// `log_full` (ROADMAP R92).
pub(crate) const MAX_FILE_BYTES: usize = 1024 * 1024;

/// What is kept of an existing file at start, so a session has room to log.
const START_BUDGET_BYTES: usize = MAX_FILE_BYTES / 2;

/// Room kept free below [`MAX_FILE_BYTES`] for the `log_full` line itself.
const LOG_FULL_RESERVE_BYTES: usize = 256;

/// How long entries are kept (ROADMAP R92: 30 days).
const RETENTION: SignedDuration = SignedDuration::from_hours(30 * 24);

/// Sends a WARN or ERROR text to `Nexus.log`.
pub(crate) type Forward = fn(log::Level, &str);

/// A message for the writer thread.
pub(crate) enum Command {
    /// Append this complete line (with newline).
    Line(String),
    /// Stop after everything sent before.
    Shutdown,
}

/// State shared by the logger, the writer thread and the panic hook.
pub(crate) struct Shared {
    /// Addon, version and clock of every line.
    pub(crate) identity: LineIdentity,
    /// Sends WARN and ERROR texts to `Nexus.log`.
    pub(crate) forward: Forward,
    /// The open file; the panic hook writes here directly, because the
    /// process may end before the writer thread runs again.
    pub(crate) file: Mutex<LogFile>,
}

/// The log file while it is open, with its size.
pub(crate) struct LogFile {
    /// `None` before the writer opened it, after a write error and after
    /// shutdown: lines are dropped then.
    file: Option<File>,
    /// Bytes in the file.
    size: usize,
    /// Whether `log_full` was written.
    full: bool,
}

impl LogFile {
    /// No file open yet.
    pub(crate) const CLOSED: Self = Self {
        file: None,
        size: 0,
        full: false,
    };

    /// Whether lines can be written right now.
    pub(crate) const fn is_open(&self) -> bool {
        self.file.is_some()
    }

    /// Appends `line`, or `log_full` instead once the cap is near. A write
    /// error closes the file and is reported to `Nexus.log`.
    pub(crate) fn write_line(
        &mut self,
        line: &str,
        shared_identity: &LineIdentity,
        forward: Forward,
    ) {
        if self.full || self.file.is_none() {
            return;
        }
        let line: Cow<'_, str> = if self.size + line.len() > MAX_FILE_BYTES - LOG_FULL_RESERVE_BYTES
        {
            self.full = true;
            let fields = [(
                Cow::Borrowed("max_bytes"),
                Cow::Owned(MAX_FILE_BYTES.to_string()),
            )];
            Cow::Owned(format_line(
                shared_identity,
                Timestamp::now(),
                &LineContent {
                    level: log::Level::Warn,
                    event: EVENT_LOG_FULL,
                    at: concat!(module_path!(), ":", line!()),
                    fields: &fields,
                },
            ))
        } else {
            Cow::Borrowed(line)
        };
        let Some(file) = self.file.as_mut() else {
            return;
        };
        match file.write_all(line.as_bytes()) {
            Ok(()) => self.size += line.len(),
            Err(error) => {
                self.file = None;
                report_unavailable(shared_identity, forward, "write", &error);
            }
        }
    }

    /// Closes the file; later lines are dropped.
    pub(crate) fn close(&mut self) {
        self.file = None;
    }
}

/// Body of the writer thread: prepares `path`, then appends every received
/// line until [`Command::Shutdown`] or until every sender is gone.
pub(crate) fn run(shared: &Shared, path: &Path, commands: &Receiver<Command>) {
    match prepare(path, Timestamp::now()) {
        Ok((file, size, previous_session_unclean)) => {
            let mut log_file = lock(&shared.file);
            *log_file = LogFile {
                file: Some(file),
                size,
                full: false,
            };
            if previous_session_unclean {
                let line = format_line(
                    &shared.identity,
                    Timestamp::now(),
                    &LineContent {
                        level: log::Level::Warn,
                        event: EVENT_PREVIOUS_SESSION_UNCLEAN,
                        at: concat!(module_path!(), ":", line!()),
                        fields: &[],
                    },
                );
                log_file.write_line(&line, &shared.identity, shared.forward);
            }
        }
        Err(error) => report_unavailable(&shared.identity, shared.forward, "prepare", &error),
    }

    while let Ok(Command::Line(line)) = commands.recv() {
        lock(&shared.file).write_line(&line, &shared.identity, shared.forward);
    }
    lock(&shared.file).close();
}

/// Creates the logs folder, rewrites `path` without old entries (atomically)
/// and opens it for appending. Returns the file, its size and whether the
/// previous session ended uncleanly.
///
/// # Errors
///
/// Returns the I/O error of any of these steps.
fn prepare(path: &Path, now: Timestamp) -> io::Result<(File, usize, bool)> {
    if let Some(folder) = path.parent() {
        fs::create_dir_all(folder)?;
    }
    let existing = match fs::read(path) {
        Ok(bytes) => String::from_utf8_lossy(&bytes).into_owned(),
        Err(error) if error.kind() == io::ErrorKind::NotFound => String::new(),
        Err(error) => return Err(error),
    };
    let retained = retain(&existing, now - RETENTION, START_BUDGET_BYTES);
    if retained.changed {
        write_atomically(path, retained.content.as_bytes())?;
    }
    let file = OpenOptions::new().create(true).append(true).open(path)?;
    Ok((
        file,
        retained.content.len(),
        retained.previous_session_unclean,
    ))
}

/// Tells `Nexus.log` that the log file cannot be used.
fn report_unavailable(identity: &LineIdentity, forward: Forward, stage: &str, error: &io::Error) {
    let error = error.to_string();
    let fields = [
        (Cow::Borrowed("stage"), Cow::Borrowed(stage)),
        (Cow::Borrowed("error"), Cow::Borrowed(error.as_str())),
    ];
    forward(
        log::Level::Warn,
        &format_body(
            identity,
            &LineContent {
                level: log::Level::Warn,
                event: EVENT_LOG_UNAVAILABLE,
                at: concat!(module_path!(), ":", line!()),
                fields: &fields,
            },
        ),
    );
}

/// Locks `file`, recovering it if a panic poisoned the lock.
pub(crate) fn lock(file: &Mutex<LogFile>) -> std::sync::MutexGuard<'_, LogFile> {
    file.lock().unwrap_or_else(PoisonError::into_inner)
}

/// The name of the log file of `identity`'s addon, e.g. `memo.log`.
pub(crate) fn log_file_name(identity: &LineIdentity) -> String {
    format!("{}.log", identity.addon)
}

#[cfg(test)]
mod tests {
    use std::sync::mpsc;

    use jiff::tz::Offset;
    use trailkit_storage::AddonName;

    use super::*;

    /// UTC as the local offset.
    fn utc(_time: Timestamp) -> Offset {
        Offset::UTC
    }

    /// Ignores forwarded texts.
    fn ignore(_level: log::Level, _text: &str) {}

    /// Shared state for a test addon called `memo`.
    fn shared() -> Shared {
        Shared {
            identity: LineIdentity {
                addon: AddonName::new("memo"),
                version: "0.3.0",
                local_offset: utc,
            },
            forward: ignore,
            file: Mutex::new(LogFile::CLOSED),
        }
    }

    /// A sample line at `time` with `event`.
    fn line(shared: &Shared, time: Timestamp, event: &str) -> String {
        format_line(
            &shared.identity,
            time,
            &LineContent {
                level: log::Level::Info,
                event,
                at: "x:1",
                fields: &[],
            },
        )
    }

    #[test]
    fn appends_lines_and_notes_an_unclean_previous_session() -> io::Result<()> {
        let folder = tempfile::tempdir()?;
        let path = folder.path().join("logs").join("memo.log");
        let shared = shared();
        fs::create_dir_all(folder.path().join("logs"))?;
        let old = line(
            &shared,
            Timestamp::now() - SignedDuration::from_hours(40 * 24),
            "session_end",
        );
        let recent = line(&shared, Timestamp::now(), "session_start");
        fs::write(&path, format!("{old}{recent}"))?;

        let (sender, receiver) = mpsc::channel();
        sender
            .send(Command::Line(line(
                &shared,
                Timestamp::now(),
                "load_finished",
            )))
            .map_err(io::Error::other)?;
        sender.send(Command::Shutdown).map_err(io::Error::other)?;
        run(&shared, &path, &receiver);

        let content = fs::read_to_string(&path)?;
        let events: Vec<_> = content
            .lines()
            .filter_map(crate::line::parse_event)
            .collect();
        assert_eq!(
            events,
            ["session_start", "previous_session_unclean", "load_finished"]
        );
        assert!(!lock(&shared.file).is_open());
        Ok(())
    }

    #[test]
    fn ends_with_log_full_at_the_cap() -> io::Result<()> {
        let folder = tempfile::tempdir()?;
        let path = folder.path().join("memo.log");
        let shared = shared();
        let (file, size, _) = prepare(&path, Timestamp::now())?;
        let mut log_file = LogFile {
            file: Some(file),
            size,
            full: false,
        };
        let sample = line(&shared, Timestamp::now(), "save_finished");
        for _ in 0..(MAX_FILE_BYTES / sample.len() + 10) {
            log_file.write_line(&sample, &shared.identity, ignore);
        }
        drop(log_file);

        let content = fs::read_to_string(&path)?;
        assert!(content.len() <= MAX_FILE_BYTES, "{}", content.len());
        assert_eq!(
            content.lines().last().and_then(crate::line::parse_event),
            Some("log_full")
        );
        Ok(())
    }
}
