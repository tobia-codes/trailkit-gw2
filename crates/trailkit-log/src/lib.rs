//! One local log per trailkit addon, one file per day (ROADMAP R89, R92).
//!
//! [`start`] installs the DLL's `log` logger. From then on every `log::`
//! call of the addon and its crates becomes one line in
//! `trailkit/logs/<addon>/<addon>-<yyyymmdd>.log`; WARN and ERROR also go to
//! `Nexus.log`.
//!
//! - **Threads:** the logger only formats and queues; a background thread
//!   owns the file (nexus-addon-dev skill §2). Dropping the returned
//!   [`LogSession`] writes `session_end` and joins that thread (hot reload).
//! - **Retention:** at start, the files of the addons' older days are deleted:
//!   the newest 14 days stay, none older than 30 days. A day has at most five
//!   files of 1 MiB (`_1` … `_4`), then `log_full`.
//! - **Events:** a message is an event name in snake_case, details go into
//!   `key = value` pairs:
//!   `log::warn!(file:% = FileName::of(&path), error:% = ErrorChain(&error); "save_failed")`.
//!   The format and every event are listed in the README.
//! - **Privacy:** never log user content or paths. Values pass through a
//!   path scrubber as a safety net only.

mod day_files;
mod error_chain;
mod events;
mod line;
mod logger;
mod panic_hook;
mod system;
mod writer;

use std::{
    borrow::Cow,
    io,
    sync::{Arc, Mutex, mpsc},
    thread::{self, JoinHandle},
};

use jiff::{Timestamp, tz::Offset};
use trailkit_storage::{AddonName, TrailkitDirectory};

pub use day_files::log_files;
pub use error_chain::ErrorChain;
pub use events::BUILT_IN_EVENTS;

use crate::{
    events::{EVENT_SESSION_END, EVENT_SESSION_START, is_critical},
    line::{Field, LineContent, LineIdentity, format_line},
    logger::{MAX_LEVEL, TrailkitLogger},
    writer::{Command, LogFile, Shared},
};

/// Name of the writer thread, as shown in a `panic` line.
const WRITER_THREAD_NAME: &str = "trailkit-log-writer";

/// What [`start`] needs to know about the addon.
pub struct LogConfig {
    /// The `trailkit/` folder; the log goes to `logs/<addon>/` below it.
    pub directory: TrailkitDirectory,
    /// The addon, e.g. `memo` (files `memo-20261109.log`, …).
    pub addon: AddonName,
    /// The addon's version, e.g. `env!("CARGO_PKG_VERSION")`.
    pub version: &'static str,
    /// The player's UTC offset at a given time (from Windows).
    pub local_offset: fn(Timestamp) -> Offset,
    /// Sends a WARN or ERROR text to `Nexus.log`.
    pub forward: fn(log::Level, &str),
    /// Extra `session_start` fields, e.g. `("language", "de")`.
    pub session_fields: Vec<(&'static str, String)>,
}

/// Why logging could not be started.
#[derive(Debug)]
pub enum StartError {
    /// This DLL already has a logger.
    LoggerAlreadySet(log::SetLoggerError),
    /// The writer thread could not be spawned.
    Spawn(io::Error),
}

impl std::fmt::Display for StartError {
    /// Describes the problem in one sentence.
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::LoggerAlreadySet(_) => formatter.write_str("a logger is already installed"),
            Self::Spawn(error) => write!(formatter, "cannot start the log writer: {error}"),
        }
    }
}

impl std::error::Error for StartError {
    /// The underlying error.
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::LoggerAlreadySet(error) => Some(error),
            Self::Spawn(error) => Some(error),
        }
    }
}

/// A running log. Dropping it writes `session_end` and joins the writer
/// thread; later `log::` calls only reach `Nexus.log` (WARN and up).
pub struct LogSession {
    /// Identity and file, shared with logger and panic hook.
    shared: Arc<Shared>,
    /// Queue of the writer thread.
    commands: mpsc::Sender<Command>,
    /// The writer thread, until it is joined.
    writer: Option<JoinHandle<()>>,
}

/// Starts logging for one addon: spawns the writer thread, installs the
/// logger and the panic hook and queues the `session_start` line. Call it
/// first in `load`, so nothing logged later is lost. Never touches the file
/// on the calling thread.
///
/// # Errors
///
/// Returns [`StartError`] if this DLL already has a logger or the writer
/// thread cannot be spawned. The addon keeps working without a log file.
pub fn start(config: LogConfig) -> Result<LogSession, StartError> {
    let identity = LineIdentity {
        addon: config.addon,
        version: config.version,
        local_offset: config.local_offset,
    };
    let shared = Arc::new(Shared {
        identity,
        forward: config.forward,
        file: Mutex::new(LogFile::new(config.directory.logs(), config.addon)),
    });
    let (commands, receiver) = mpsc::channel();

    let logger = TrailkitLogger {
        shared: Arc::clone(&shared),
        commands: commands.clone(),
    };
    log::set_boxed_logger(Box::new(logger)).map_err(StartError::LoggerAlreadySet)?;
    log::set_max_level(MAX_LEVEL);

    let writer_shared = Arc::clone(&shared);
    let writer = thread::Builder::new()
        .name(WRITER_THREAD_NAME.to_owned())
        .spawn(move || writer::run(&writer_shared, &receiver))
        .map_err(StartError::Spawn)?;
    panic_hook::install(Arc::clone(&shared), commands.clone());

    let mut fields: Vec<Field<'_>> = vec![
        (Cow::Borrowed("build"), Cow::Borrowed(system::BUILD_KIND)),
        (Cow::Borrowed("commit"), Cow::Borrowed(system::COMMIT)),
        (
            Cow::Borrowed("windows"),
            Cow::Owned(system::windows_version()),
        ),
    ];
    fields.extend(
        config
            .session_fields
            .into_iter()
            .map(|(key, value)| (Cow::Borrowed(key), Cow::Owned(value))),
    );
    let session = LogSession {
        shared,
        commands,
        writer: Some(writer),
    };
    session.queue(
        EVENT_SESSION_START,
        concat!(module_path!(), ":", line!()),
        &fields,
    );
    Ok(session)
}

impl LogSession {
    /// Queues one INFO line written by this crate.
    fn queue(&self, event: &str, at: &str, fields: &[Field<'_>]) {
        let line = format_line(
            &self.shared.identity,
            Timestamp::now(),
            &LineContent {
                level: log::Level::Info,
                event,
                at,
                fields,
            },
        );
        let _ = self.commands.send(Command::Line {
            text: line,
            critical: is_critical(event),
        });
    }
}

impl Drop for LogSession {
    /// Writes `session_end`, stops the writer thread and waits for it.
    fn drop(&mut self) {
        self.queue(
            EVENT_SESSION_END,
            concat!(module_path!(), ":", line!()),
            &[],
        );
        let _ = self.commands.send(Command::Shutdown);
        if let Some(writer) = self.writer.take() {
            // A panicking writer already logged its panic; nothing else to do.
            let _ = writer.join();
        }
    }
}
