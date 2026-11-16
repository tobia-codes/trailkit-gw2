//! The background thread that owns the log files (nexus-addon-dev skill §2:
//! no file I/O on the render thread).
//!
//! At start it deletes expired day files ([`day_files::cleanup`]), notes an
//! unclean previous session, then appends every line it receives to the file
//! of the current day until it is told to stop. Lines are written
//! unbuffered, so the operating system has them even if the game is killed a
//! moment later.

use std::{
    borrow::Cow,
    fs::{self, File, OpenOptions},
    io::{self, Write as _},
    path::{Path, PathBuf},
    sync::{Mutex, MutexGuard, PoisonError, mpsc::Receiver},
};

use jiff::{Timestamp, civil::Date, tz::TimeZone};
use trailkit_storage::AddonName;

use crate::{
    day_files::{self, DayFile, MAX_PARTS_PER_DAY},
    events::{
        EVENT_LOG_FULL, EVENT_LOG_MAINTENANCE_FAILED, EVENT_LOG_UNAVAILABLE,
        EVENT_PREVIOUS_SESSION_UNCLEAN, EVENT_SESSION_CONTINUES,
    },
    line::{Field, LineContent, LineIdentity, format_body, format_line},
};

/// Largest size of one log file (ROADMAP R92); a full file continues in the
/// next part of the day.
pub(crate) const MAX_PART_BYTES: usize = 1024 * 1024;

/// Room at the end of a day's last file that only critical events may use
/// (and the `log_full` line itself).
pub(crate) const CRITICAL_RESERVE_BYTES: usize = 8 * 1024;

/// Sends a WARN or ERROR text to `Nexus.log`.
pub(crate) type Forward = fn(log::Level, &str);

/// A message for the writer thread.
pub(crate) enum Command {
    /// Append this complete line (with newline).
    Line {
        /// The formatted line.
        text: String,
        /// Whether the event may use the reserve of a full day.
        critical: bool,
    },
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

/// The file a line is appended to, with its size.
struct OpenPart {
    /// The open file.
    file: File,
    /// Which day and part it is.
    id: DayFile,
    /// Bytes in the file.
    size: usize,
}

impl OpenPart {
    /// Appends `line` unbuffered.
    fn write(&mut self, line: &str) -> io::Result<()> {
        self.file.write_all(line.as_bytes())?;
        self.size += line.len();
        Ok(())
    }
}

/// An addon's log: the day file lines go to, switched at midnight and when a
/// file is full.
pub(crate) struct LogFile {
    /// The `trailkit/logs/` folder.
    logs: PathBuf,
    /// The addon whose folder inside `logs` is used.
    addon: AddonName,
    /// `false` before the writer prepared the log, if preparing failed and
    /// after shutdown: lines are dropped then.
    usable: bool,
    /// The file of the current day, once the first line was written.
    part: Option<OpenPart>,
    /// The day for which `log_full` was written; only critical events are
    /// written for it.
    full_day: Option<Date>,
    /// The `session_start` fields, repeated when a new day begins.
    session_fields: Vec<Field<'static>>,
    /// Whether a write error was already sent to `Nexus.log`.
    failure_reported: bool,
}

impl LogFile {
    /// A log that is not usable until the writer thread prepared it.
    pub(crate) const fn new(
        logs: PathBuf,
        addon: AddonName,
        session_fields: Vec<Field<'static>>,
    ) -> Self {
        Self {
            logs,
            addon,
            usable: false,
            part: None,
            full_day: None,
            session_fields,
            failure_reported: false,
        }
    }

    /// The addon's folder, `trailkit/logs/<addon>/`.
    fn folder(&self) -> PathBuf {
        self.logs.join(self.addon.as_str())
    }

    /// Whether lines can be written right now.
    pub(crate) const fn is_open(&self) -> bool {
        self.usable
    }

    /// Appends `line` to the file of `now`'s local day. After an I/O error
    /// the line is lost and the next one tries again (a virus scanner or
    /// cloud sync may hold a file only briefly); `Nexus.log` hears of the
    /// first error of a session.
    ///
    /// A file that cannot take the line continues in the next part of the
    /// day. In the last part a normal line stops at the reserve: `log_full`
    /// is written once and only `critical` lines follow until midnight.
    pub(crate) fn write_line(
        &mut self,
        line: &str,
        critical: bool,
        now: Timestamp,
        identity: &LineIdentity,
        forward: Forward,
    ) {
        let day = local_day(identity, now);
        if !self.usable || (self.full_day == Some(day) && !critical) {
            return;
        }
        if let Err(error) = self.append(line, critical, day, identity, now) {
            self.part = None;
            if !self.failure_reported {
                self.failure_reported = true;
                report_unavailable(identity, forward, "write", &error);
            }
        }
    }

    /// Appends `line` to `day`'s files (see [`Self::write_line`]).
    fn append(
        &mut self,
        line: &str,
        critical: bool,
        day: Date,
        identity: &LineIdentity,
        now: Timestamp,
    ) -> io::Result<()> {
        let open_day = self.part.as_ref().map(|open| open.id.day);
        if open_day != Some(day) {
            let latest = day_files::latest_part(&self.folder(), self.addon.as_str(), day)?;
            self.open_part(latest.unwrap_or(DayFile { day, part: 0 }))?;
            if open_day.is_some() {
                let header = session_continues_line(identity, now, &self.session_fields);
                if let Some(open) = self.part.as_mut() {
                    open.write(&header)?;
                }
            }
        }
        let limit = if critical {
            MAX_PART_BYTES
        } else {
            MAX_PART_BYTES - CRITICAL_RESERVE_BYTES
        };
        loop {
            let Some(open) = self.part.as_mut() else {
                return Ok(());
            };
            // A fresh file takes any line, so an oversized one cannot loop.
            if open.size == 0 || open.size + line.len() <= limit {
                return open.write(line);
            }
            if open.id.part + 1 < MAX_PARTS_PER_DAY {
                let next = DayFile {
                    day,
                    part: open.id.part + 1,
                };
                self.open_part(next)?;
                continue;
            }
            if critical {
                // Not even the reserve has room: dropped.
                return Ok(());
            }
            self.full_day = Some(day);
            // After a restart on a full day the reserve may be used up.
            let full = log_full_line(identity, now);
            if open.size + full.len() > MAX_PART_BYTES {
                return Ok(());
            }
            return open.write(&full);
        }
    }

    /// Opens `id` for appending and makes it the current file. A last line
    /// cut off by a crash gets its newline first.
    fn open_part(&mut self, id: DayFile) -> io::Result<()> {
        let folder = self.folder();
        fs::create_dir_all(&folder)?;
        let path = folder.join(id.file_name(self.addon.as_str()));
        let mut file = OpenOptions::new().create(true).append(true).open(&path)?;
        let mut size = usize::try_from(file.metadata()?.len()).unwrap_or(usize::MAX);
        if size > 0 && !day_files::ends_with_newline(&path)? {
            file.write_all(b"\n")?;
            size += 1;
        }
        self.part = Some(OpenPart { file, id, size });
        Ok(())
    }

    /// Closes the file; later lines are dropped.
    pub(crate) fn close(&mut self) {
        self.part = None;
        self.usable = false;
    }
}

/// The local day of `now` by the player's UTC offset.
fn local_day(identity: &LineIdentity, now: Timestamp) -> Date {
    now.to_zoned(TimeZone::fixed((identity.local_offset)(now)))
        .date()
}

/// The first line of a new day's file in a running session.
fn session_continues_line(
    identity: &LineIdentity,
    now: Timestamp,
    fields: &[Field<'static>],
) -> String {
    format_line(
        identity,
        now,
        &LineContent {
            level: log::Level::Info,
            event: EVENT_SESSION_CONTINUES,
            at: concat!(module_path!(), ":", line!()),
            fields,
        },
    )
}

/// The `log_full` line.
fn log_full_line(identity: &LineIdentity, now: Timestamp) -> String {
    let fields = [(
        Cow::Borrowed("max_bytes"),
        Cow::Owned((MAX_PART_BYTES * usize::from(MAX_PARTS_PER_DAY)).to_string()),
    )];
    format_line(
        identity,
        now,
        &LineContent {
            level: log::Level::Warn,
            event: EVENT_LOG_FULL,
            at: concat!(module_path!(), ":", line!()),
            fields: &fields,
        },
    )
}

/// What [`prepare`] found out.
struct Prepared {
    /// Whether the previous session ended without `session_end`.
    previous_session_unclean: bool,
    /// Housekeeping steps that failed, as stage and error; logging goes on.
    failures: Vec<(&'static str, io::Error)>,
}

/// Body of the writer thread: prepares the log, then appends every received
/// line until [`Command::Shutdown`] or until every sender is gone.
pub(crate) fn run(shared: &Shared, commands: &Receiver<Command>) {
    let (logs, addon) = {
        let file = lock(&shared.file);
        (file.logs.clone(), file.addon)
    };
    let now = Timestamp::now();
    match prepare(&logs, addon, local_day(&shared.identity, now)) {
        Ok(prepared) => {
            let mut log_file = lock(&shared.file);
            log_file.usable = true;
            for (stage, error) in &prepared.failures {
                log_maintenance_failure(&mut log_file, shared, now, stage, error);
            }
            if prepared.previous_session_unclean {
                let line = format_line(
                    &shared.identity,
                    now,
                    &LineContent {
                        level: log::Level::Warn,
                        event: EVENT_PREVIOUS_SESSION_UNCLEAN,
                        at: concat!(module_path!(), ":", line!()),
                        fields: &[],
                    },
                );
                log_file.write_line(&line, true, now, &shared.identity, shared.forward);
            }
        }
        Err(error) => report_unavailable(&shared.identity, shared.forward, "prepare", &error),
    }

    while let Ok(Command::Line { text, critical }) = commands.recv() {
        lock(&shared.file).write_line(
            &text,
            critical,
            Timestamp::now(),
            &shared.identity,
            shared.forward,
        );
    }
    lock(&shared.file).close();
}

/// Deletes expired day files, creates `addon`'s folder and checks how the
/// previous session ended.
///
/// # Errors
///
/// Returns the I/O error of creating the folder. Failing to delete old files
/// or to read the newest one does not stop logging; it is returned inside
/// [`Prepared`], and an unknown previous session counts as clean.
fn prepare(logs: &Path, addon: AddonName, today: Date) -> io::Result<Prepared> {
    let mut failures = Vec::new();
    if let Err(error) = day_files::cleanup(logs, today) {
        failures.push(("cleanup", error));
    }
    let folder = logs.join(addon.as_str());
    fs::create_dir_all(&folder)?;
    let previous_session_unclean =
        newest_session_unclean(&folder, addon.as_str()).unwrap_or_else(|error| {
            failures.push(("session_check", error));
            false
        });
    Ok(Prepared {
        previous_session_unclean,
        failures,
    })
}

/// Whether the newest day file in `folder` ends without `session_end`.
///
/// # Errors
///
/// Returns the I/O error of listing the folder or reading the file.
fn newest_session_unclean(folder: &Path, addon: &str) -> io::Result<bool> {
    match day_files::latest(folder, addon)? {
        Some(newest) => day_files::previous_session_unclean(&folder.join(newest.file_name(addon))),
        None => Ok(false),
    }
}

/// Writes that the housekeeping step `stage` failed into the log and tells
/// `Nexus.log` (it is a warning like any other).
fn log_maintenance_failure(
    log_file: &mut LogFile,
    shared: &Shared,
    now: Timestamp,
    stage: &'static str,
    error: &io::Error,
) {
    let error = error.to_string();
    let fields = [
        (Cow::Borrowed("stage"), Cow::Borrowed(stage)),
        (Cow::Borrowed("error"), Cow::Borrowed(error.as_str())),
    ];
    let content = LineContent {
        level: log::Level::Warn,
        event: EVENT_LOG_MAINTENANCE_FAILED,
        at: concat!(module_path!(), ":", line!()),
        fields: &fields,
    };
    let line = format_line(&shared.identity, now, &content);
    log_file.write_line(&line, false, now, &shared.identity, shared.forward);
    (shared.forward)(log::Level::Warn, &format_body(&shared.identity, &content));
}

/// Tells `Nexus.log` that the log files cannot be used.
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
pub(crate) fn lock(file: &Mutex<LogFile>) -> MutexGuard<'_, LogFile> {
    file.lock().unwrap_or_else(PoisonError::into_inner)
}

#[cfg(test)]
mod tests {
    use std::sync::mpsc;

    use jiff::{SignedDuration, tz::Offset};

    use super::*;
    use crate::line::parse_event;

    /// UTC as the local offset.
    fn utc(_time: Timestamp) -> Offset {
        Offset::UTC
    }

    /// Ignores forwarded texts.
    fn ignore(_level: log::Level, _text: &str) {}

    /// Shared state for a test addon called `memo`, logging into `logs`.
    fn shared(logs: &Path) -> Shared {
        Shared {
            identity: LineIdentity {
                addon: AddonName::new("memo"),
                version: "0.3.0",
                local_offset: utc,
            },
            forward: ignore,
            file: Mutex::new(LogFile::new(
                logs.to_path_buf(),
                AddonName::new("memo"),
                vec![(Cow::Borrowed("commit"), Cow::Borrowed("abc"))],
            )),
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

    /// The events of every line of `path`.
    fn events(path: &Path) -> io::Result<Vec<String>> {
        Ok(fs::read_to_string(path)?
            .lines()
            .filter_map(parse_event)
            .map(str::to_owned)
            .collect())
    }

    /// The memo day file `name` in `logs`.
    fn file(logs: &Path, name: &str) -> PathBuf {
        logs.join("memo").join(name)
    }

    /// The name of today's base file (UTC).
    fn today_name() -> String {
        DayFile {
            day: Timestamp::now().to_zoned(TimeZone::UTC).date(),
            part: 0,
        }
        .file_name("memo")
    }

    #[test]
    fn appends_lines_and_notes_an_unclean_previous_session() -> io::Result<()> {
        let logs = tempfile::tempdir()?;
        let shared = shared(logs.path());
        fs::create_dir_all(logs.path().join("memo"))?;
        let path = file(logs.path(), &today_name());
        fs::write(&path, line(&shared, Timestamp::now(), "session_start"))?;

        let (sender, receiver) = mpsc::channel();
        sender
            .send(Command::Line {
                text: line(&shared, Timestamp::now(), "load_finished"),
                critical: false,
            })
            .map_err(io::Error::other)?;
        sender.send(Command::Shutdown).map_err(io::Error::other)?;
        run(&shared, &receiver);

        assert_eq!(
            events(&path)?,
            ["session_start", "previous_session_unclean", "load_finished"]
        );
        assert!(!lock(&shared.file).is_open());
        Ok(())
    }

    #[test]
    fn a_clean_previous_session_is_not_reported() -> io::Result<()> {
        let logs = tempfile::tempdir()?;
        let shared = shared(logs.path());
        fs::create_dir_all(logs.path().join("memo"))?;
        let path = file(logs.path(), &today_name());
        fs::write(
            &path,
            format!(
                "{}{}",
                line(&shared, Timestamp::now(), "session_start"),
                line(&shared, Timestamp::now(), "session_end")
            ),
        )?;

        let (sender, receiver) = mpsc::channel();
        sender
            .send(Command::Line {
                text: line(&shared, Timestamp::now(), "session_start"),
                critical: false,
            })
            .map_err(io::Error::other)?;
        sender.send(Command::Shutdown).map_err(io::Error::other)?;
        run(&shared, &receiver);

        assert_eq!(
            events(&path)?,
            ["session_start", "session_end", "session_start"]
        );
        Ok(())
    }

    /// Texts forwarded by [`unavailable_log_is_reported_to_nexus`].
    static FORWARDED: Mutex<Vec<String>> = Mutex::new(Vec::new());

    /// Records a forwarded text.
    fn record(level: log::Level, text: &str) {
        FORWARDED
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .push(format!("{level} {text}"));
    }

    #[test]
    fn unavailable_log_is_reported_to_nexus() -> io::Result<()> {
        let logs = tempfile::tempdir()?;
        // A file in place of the addon's folder: it can be neither listed nor created.
        fs::write(logs.path().join("memo"), "not a folder")?;
        let shared = Shared {
            forward: record,
            ..shared(logs.path())
        };

        let (sender, receiver) = mpsc::channel();
        sender
            .send(Command::Line {
                text: line(&shared, Timestamp::now(), "load_finished"),
                critical: false,
            })
            .map_err(io::Error::other)?;
        sender.send(Command::Shutdown).map_err(io::Error::other)?;
        run(&shared, &receiver);

        let forwarded = FORWARDED
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .clone();
        assert_eq!(forwarded.len(), 1, "{forwarded:?}");
        assert!(
            forwarded[0].starts_with("WARN addon=memo version=0.3.0 log_unavailable at="),
            "{forwarded:?}"
        );
        assert!(forwarded[0].contains(" stage=prepare "), "{forwarded:?}");
        assert!(logs.path().join("memo").is_file(), "left alone");
        Ok(())
    }

    /// A usable log of `shared` for the tests below.
    fn open_log(shared: &Shared) -> MutexGuard<'_, LogFile> {
        let mut log_file = lock(&shared.file);
        log_file.usable = true;
        log_file
    }

    #[test]
    fn a_new_day_starts_a_new_file() -> io::Result<()> {
        let logs = tempfile::tempdir()?;
        let shared = shared(logs.path());
        let mut log_file = open_log(&shared);
        let first: Timestamp = "2026-11-09T23:59:50Z".parse().map_err(io::Error::other)?;
        let second: Timestamp = "2026-11-10T00:00:10Z".parse().map_err(io::Error::other)?;
        for (time, event) in [(first, "session_start"), (second, "load_finished")] {
            log_file.write_line(
                &line(&shared, time, event),
                false,
                time,
                &shared.identity,
                ignore,
            );
        }
        drop(log_file);

        assert_eq!(
            events(&file(logs.path(), "memo-20261109.log"))?,
            ["session_start"]
        );
        assert_eq!(
            events(&file(logs.path(), "memo-20261110.log"))?,
            ["session_continues", "load_finished"]
        );
        Ok(())
    }

    #[test]
    fn the_day_follows_the_players_utc_offset() -> io::Result<()> {
        /// One hour ahead of UTC.
        fn plus_one(_time: Timestamp) -> Offset {
            Offset::constant(1)
        }
        let logs = tempfile::tempdir()?;
        let mut shared = shared(logs.path());
        shared.identity.local_offset = plus_one;
        let mut log_file = open_log(&shared);
        let time: Timestamp = "2026-11-09T23:30:00Z".parse().map_err(io::Error::other)?;
        log_file.write_line(
            &line(&shared, time, "session_start"),
            false,
            time,
            &shared.identity,
            ignore,
        );
        drop(log_file);

        assert!(file(logs.path(), "memo-20261110.log").is_file());
        Ok(())
    }

    #[test]
    fn a_full_file_continues_in_the_next_part_and_the_day_ends_with_log_full() -> io::Result<()> {
        let logs = tempfile::tempdir()?;
        let shared = shared(logs.path());
        let mut log_file = open_log(&shared);
        let now: Timestamp = "2026-11-09T12:00:00Z".parse().map_err(io::Error::other)?;
        let sample = line(&shared, now, "save_finished");
        let writes = usize::from(MAX_PARTS_PER_DAY) * MAX_PART_BYTES / sample.len() + 50;
        for _ in 0..writes {
            log_file.write_line(&sample, false, now, &shared.identity, ignore);
        }
        // Critical events still fit into the reserve, normal ones do not.
        let panic = line(&shared, now, "panic");
        log_file.write_line(&panic, true, now, &shared.identity, ignore);
        log_file.write_line(&sample, false, now, &shared.identity, ignore);
        drop(log_file);

        let names = [
            "memo-20261109.log",
            "memo-20261109_1.log",
            "memo-20261109_2.log",
            "memo-20261109_3.log",
            "memo-20261109_4.log",
        ];
        for name in names {
            let size = fs::metadata(file(logs.path(), name))?.len();
            assert!(size <= MAX_PART_BYTES as u64, "{name}: {size}");
        }
        assert!(
            !file(logs.path(), "memo-20261109_5.log").exists(),
            "at most five files a day"
        );
        let last = events(&file(logs.path(), names[4]))?;
        assert_eq!(last.last().map(String::as_str), Some("panic"));
        assert_eq!(
            last.iter().rev().nth(1).map(String::as_str),
            Some("log_full")
        );
        assert_eq!(last.iter().filter(|event| *event == "log_full").count(), 1);
        Ok(())
    }

    #[test]
    fn a_full_day_is_open_again_the_next_day() -> io::Result<()> {
        let logs = tempfile::tempdir()?;
        let shared = shared(logs.path());
        let mut log_file = open_log(&shared);
        let now: Timestamp = "2026-11-09T12:00:00Z".parse().map_err(io::Error::other)?;
        let sample = line(&shared, now, "save_finished");
        for _ in 0..(usize::from(MAX_PARTS_PER_DAY) * MAX_PART_BYTES / sample.len() + 50) {
            log_file.write_line(&sample, false, now, &shared.identity, ignore);
        }
        let tomorrow = now + SignedDuration::from_hours(24);
        log_file.write_line(
            &line(&shared, tomorrow, "session_start"),
            false,
            tomorrow,
            &shared.identity,
            ignore,
        );
        drop(log_file);

        assert_eq!(
            events(&file(logs.path(), "memo-20261110.log"))?,
            ["session_continues", "session_start"]
        );
        Ok(())
    }

    #[test]
    fn a_new_day_repeats_the_session_fields() -> io::Result<()> {
        let logs = tempfile::tempdir()?;
        let shared = shared(logs.path());
        let mut log_file = open_log(&shared);
        let first: Timestamp = "2026-11-09T23:59:50Z".parse().map_err(io::Error::other)?;
        let second: Timestamp = "2026-11-10T00:00:10Z".parse().map_err(io::Error::other)?;
        for time in [first, second] {
            log_file.write_line(
                &line(&shared, time, "load_finished"),
                false,
                time,
                &shared.identity,
                ignore,
            );
        }
        drop(log_file);

        let first_file = fs::read_to_string(file(logs.path(), "memo-20261109.log"))?;
        assert!(!first_file.contains("session_continues"), "{first_file}");
        let second_file = fs::read_to_string(file(logs.path(), "memo-20261110.log"))?;
        let header = second_file.lines().next().unwrap_or_default();
        assert!(header.contains(" session_continues "), "{header}");
        assert!(header.ends_with(" commit=abc"), "{header}");
        Ok(())
    }

    /// Writes a last part of the day 2026-11-09 with `size` bytes.
    fn fill_last_part(logs: &Path, size: usize) -> io::Result<PathBuf> {
        fs::create_dir_all(logs.join("memo"))?;
        let path = file(logs, "memo-20261109_4.log");
        fs::write(&path, format!("{}\n", "x".repeat(size - 1)))?;
        Ok(path)
    }

    #[test]
    fn a_restart_on_a_full_day_keeps_session_start_and_stays_below_the_cap() -> io::Result<()> {
        let logs = tempfile::tempdir()?;
        let shared = shared(logs.path());
        let path = fill_last_part(logs.path(), MAX_PART_BYTES - CRITICAL_RESERVE_BYTES + 1)?;
        let now: Timestamp = "2026-11-09T12:00:00Z".parse().map_err(io::Error::other)?;
        let mut log_file = open_log(&shared);
        let start = line(&shared, now, "session_start");
        log_file.write_line(&start, true, now, &shared.identity, ignore);
        let sample = line(&shared, now, "save_failed");
        log_file.write_line(&sample, false, now, &shared.identity, ignore);
        log_file.write_line(&sample, false, now, &shared.identity, ignore);
        drop(log_file);

        assert_eq!(events(&path)?, ["session_start", "log_full"]);
        Ok(())
    }

    #[test]
    fn log_full_never_exceeds_the_cap() -> io::Result<()> {
        let logs = tempfile::tempdir()?;
        let shared = shared(logs.path());
        let path = fill_last_part(logs.path(), MAX_PART_BYTES - 10)?;
        let now: Timestamp = "2026-11-09T12:00:00Z".parse().map_err(io::Error::other)?;
        let mut log_file = open_log(&shared);
        let sample = line(&shared, now, "save_failed");
        log_file.write_line(&sample, false, now, &shared.identity, ignore);
        drop(log_file);

        assert_eq!(fs::metadata(&path)?.len(), MAX_PART_BYTES as u64 - 10);
        Ok(())
    }

    /// Texts forwarded by [`a_failed_write_is_reported_once_and_the_next_line_retries`].
    static RETRY_FORWARDED: Mutex<Vec<String>> = Mutex::new(Vec::new());

    /// Records a forwarded text for the retry test.
    fn record_retry(level: log::Level, text: &str) {
        RETRY_FORWARDED
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .push(format!("{level} {text}"));
    }

    #[test]
    fn a_failed_write_is_reported_once_and_the_next_line_retries() -> io::Result<()> {
        let logs = tempfile::tempdir()?;
        let shared = shared(logs.path());
        // A file in place of the addon's folder makes every open fail.
        fs::write(logs.path().join("memo"), "in the way")?;
        let now: Timestamp = "2026-11-09T12:00:00Z".parse().map_err(io::Error::other)?;
        let sample = line(&shared, now, "save_failed");
        let mut log_file = open_log(&shared);
        for _ in 0..2 {
            log_file.write_line(&sample, false, now, &shared.identity, record_retry);
        }
        assert!(log_file.is_open(), "the log stays usable");
        fs::remove_file(logs.path().join("memo"))?;
        log_file.write_line(&sample, false, now, &shared.identity, record_retry);
        drop(log_file);

        let forwarded = RETRY_FORWARDED
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .clone();
        assert_eq!(forwarded.len(), 1, "{forwarded:?}");
        assert!(forwarded[0].contains(" stage=write "), "{forwarded:?}");
        assert_eq!(
            events(&file(logs.path(), "memo-20261109.log"))?,
            ["save_failed"]
        );
        Ok(())
    }

    #[test]
    fn an_unreadable_newest_file_does_not_stop_the_log() -> io::Result<()> {
        let logs = tempfile::tempdir()?;
        let shared = shared(logs.path());
        // A folder with the name of yesterday's file cannot be read as a file.
        let yesterday = DayFile {
            day: Timestamp::now()
                .to_zoned(TimeZone::UTC)
                .date()
                .yesterday()
                .map_err(io::Error::other)?,
            part: 0,
        };
        fs::create_dir_all(file(logs.path(), &yesterday.file_name("memo")))?;

        let (sender, receiver) = mpsc::channel();
        sender
            .send(Command::Line {
                text: line(&shared, Timestamp::now(), "session_start"),
                critical: true,
            })
            .map_err(io::Error::other)?;
        sender.send(Command::Shutdown).map_err(io::Error::other)?;
        run(&shared, &receiver);

        let path = file(logs.path(), &today_name());
        assert_eq!(events(&path)?, ["log_maintenance_failed", "session_start"]);
        assert!(fs::read_to_string(&path)?.contains(" stage=session_check "));
        Ok(())
    }

    #[test]
    fn a_line_cut_off_by_a_crash_gets_its_newline_before_the_next_one() -> io::Result<()> {
        let logs = tempfile::tempdir()?;
        let shared = shared(logs.path());
        fs::create_dir_all(logs.path().join("memo"))?;
        let now: Timestamp = "2026-11-09T12:00:00Z".parse().map_err(io::Error::other)?;
        let path = file(logs.path(), "memo-20261109.log");
        fs::write(&path, "2026-11-09T12:00:00+00:00 INFO addon=memo vers")?;
        let mut log_file = open_log(&shared);
        log_file.write_line(
            &line(&shared, now, "session_start"),
            false,
            now,
            &shared.identity,
            ignore,
        );
        drop(log_file);

        let content = fs::read_to_string(&path)?;
        assert_eq!(content.lines().count(), 2, "{content}");
        assert_eq!(events(&path)?, ["session_start"]);
        Ok(())
    }
}
