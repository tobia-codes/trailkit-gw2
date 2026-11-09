//! The day files of a log (ROADMAP R92): names, listing, cleanup at start and
//! the check whether the previous session ended cleanly.
//!
//! An addon's log lives in `trailkit/logs/<addon>/`. Each day has its own
//! file `<addon>-<yyyymmdd>.log`; when a file is full the day continues in
//! `<addon>-<yyyymmdd>_1.log`, `_2`, … (at most [`MAX_PARTS_PER_DAY`] files).
//! Old files are deleted whole, so nothing is ever read into memory or
//! rewritten to clean up.

use std::{
    fs::{self, File},
    io::{self, Read as _, Seek as _, SeekFrom},
    path::{Path, PathBuf},
};

use jiff::{Span, civil::Date};
use trailkit_storage::AddonName;

use crate::{events::EVENT_SESSION_END, line::parse_event};

/// How many days with a log file are kept per addon (ROADMAP R92).
pub(crate) const KEPT_DAYS: usize = 14;

/// Files of older days are deleted even among the newest [`KEPT_DAYS`].
pub(crate) const MAX_AGE_DAYS: i64 = 30;

/// Most files of one day: the base file and the parts `_1` to `_4`.
pub(crate) const MAX_PARTS_PER_DAY: u8 = 5;

/// How much of a file's end is read to find its last line.
const TAIL_BYTES: u64 = 8 * 1024;

/// One log file of an addon: a day and its part number (0 is the base file).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) struct DayFile {
    /// The local day the file belongs to.
    pub(crate) day: Date,
    /// 0 for `<addon>-<yyyymmdd>.log`, n for `<addon>-<yyyymmdd>_n.log`.
    pub(crate) part: u8,
}

impl DayFile {
    /// The file name for `addon`, e.g. `memo-20261109.log` or
    /// `memo-20261109_1.log`.
    pub(crate) fn file_name(self, addon: &str) -> String {
        let (year, month, day) = (self.day.year(), self.day.month(), self.day.day());
        if self.part == 0 {
            format!("{addon}-{year:04}{month:02}{day:02}.log")
        } else {
            format!("{addon}-{year:04}{month:02}{day:02}_{}.log", self.part)
        }
    }

    /// Reads a file name of `addon`'s folder; `None` for anything that is not
    /// one of our day files, so foreign files are never touched.
    pub(crate) fn parse(addon: &str, name: &str) -> Option<Self> {
        let stem = name
            .strip_prefix(addon)?
            .strip_prefix('-')?
            .strip_suffix(".log")?;
        let (date, part) = match stem.split_once('_') {
            Some((date, part)) => {
                if !is_digits(part) || part.starts_with('0') {
                    return None;
                }
                (date, part.parse::<u8>().ok().filter(|part| *part > 0)?)
            }
            None => (stem, 0),
        };
        if date.len() != 8 || !is_digits(date) {
            return None;
        }
        let day = Date::new(
            date[..4].parse().ok()?,
            date[4..6].parse().ok()?,
            date[6..].parse().ok()?,
        )
        .ok()?;
        Some(Self { day, part })
    }
}

/// Whether `text` is not empty and consists of ASCII digits only.
fn is_digits(text: &str) -> bool {
    !text.is_empty() && text.bytes().all(|byte| byte.is_ascii_digit())
}

/// The day files of `addon` in `folder`, oldest first. A missing folder has
/// none.
///
/// # Errors
///
/// Returns the I/O error of listing the folder.
pub(crate) fn list(folder: &Path, addon: &str) -> io::Result<Vec<DayFile>> {
    let entries = match fs::read_dir(folder) {
        Ok(entries) => entries,
        Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(Vec::new()),
        Err(error) => return Err(error),
    };
    let mut files = Vec::new();
    for entry in entries {
        let name = entry?.file_name();
        if let Some(file) = name.to_str().and_then(|name| DayFile::parse(addon, name)) {
            files.push(file);
        }
    }
    files.sort_unstable();
    Ok(files)
}

/// The newest day file of `addon` in `folder`.
///
/// # Errors
///
/// Returns the I/O error of listing the folder.
pub(crate) fn latest(folder: &Path, addon: &str) -> io::Result<Option<DayFile>> {
    Ok(list(folder, addon)?.last().copied())
}

/// The newest part of `day` in `folder`.
///
/// # Errors
///
/// Returns the I/O error of listing the folder.
pub(crate) fn latest_part(folder: &Path, addon: &str, day: Date) -> io::Result<Option<DayFile>> {
    Ok(list(folder, addon)?
        .into_iter()
        .rfind(|file| file.day == day))
}

/// The paths of all of `addon`'s day files in `logs`, oldest first: what an
/// export or a test reads to see the whole log.
///
/// # Errors
///
/// Returns the I/O error of listing the addon's folder.
pub fn log_files(logs: &Path, addon: AddonName) -> io::Result<Vec<PathBuf>> {
    let folder = logs.join(addon.as_str());
    Ok(list(&folder, addon.as_str())?
        .into_iter()
        .map(|file| folder.join(file.file_name(addon.as_str())))
        .collect())
}

/// Deletes the day files of every addon folder in `logs` that are beyond the
/// newest [`KEPT_DAYS`] days or older than [`MAX_AGE_DAYS`] days before
/// `today`, and removes folders left empty. Other files and folders are left
/// alone. Goes on after an error and returns the first one.
///
/// # Errors
///
/// Returns the first I/O error of listing or deleting.
pub(crate) fn cleanup(logs: &Path, today: Date) -> io::Result<()> {
    let entries = match fs::read_dir(logs) {
        Ok(entries) => entries,
        Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(()),
        Err(error) => return Err(error),
    };
    let oldest = today
        .checked_sub(Span::new().days(MAX_AGE_DAYS))
        .map_err(io::Error::other)?;
    let mut first_error = None;
    for entry in entries {
        let entry = entry?;
        if !entry.file_type()?.is_dir() {
            continue;
        }
        let Some(addon) = entry.file_name().to_str().map(str::to_owned) else {
            continue;
        };
        if let Err(error) = clean_folder(&entry.path(), &addon, oldest) {
            first_error.get_or_insert(error);
        }
    }
    first_error.map_or(Ok(()), Err)
}

/// Deletes the expired day files of one addon folder (see [`cleanup`]);
/// `oldest` is the oldest day that may stay.
fn clean_folder(folder: &Path, addon: &str, oldest: Date) -> io::Result<()> {
    let files = list(folder, addon)?;
    let mut days: Vec<Date> = files.iter().map(|file| file.day).collect();
    days.dedup();
    let kept: Vec<Date> = days
        .into_iter()
        .rev()
        .filter(|day| *day >= oldest)
        .take(KEPT_DAYS)
        .collect();
    let mut first_error = None;
    for file in files.iter().filter(|file| !kept.contains(&file.day)) {
        if let Err(error) = fs::remove_file(folder.join(file.file_name(addon))) {
            first_error.get_or_insert(error);
        }
    }
    if kept.is_empty() {
        // Fails harmlessly while foreign files or a failed delete remain.
        let _ = fs::remove_dir(folder);
    }
    first_error.map_or(Ok(()), Err)
}

/// Whether the file at `path` ends with a newline (an empty file does).
///
/// # Errors
///
/// Returns the I/O error of opening or reading the file.
pub(crate) fn ends_with_newline(path: &Path) -> io::Result<bool> {
    let mut file = File::open(path)?;
    let length = file.metadata()?.len();
    if length == 0 {
        return Ok(true);
    }
    file.seek(SeekFrom::Start(length - 1))?;
    let mut last = [0_u8; 1];
    file.read_exact(&mut last)?;
    Ok(last[0] == b'\n')
}

/// Whether the session that wrote the file at `path` ended uncleanly: its
/// last line is not `session_end`. Reads only the end of the file. A file
/// without lines counts as clean.
///
/// # Errors
///
/// Returns the I/O error of opening or reading the file.
pub(crate) fn previous_session_unclean(path: &Path) -> io::Result<bool> {
    let mut file = File::open(path)?;
    let length = file.metadata()?.len();
    file.seek(SeekFrom::Start(length.saturating_sub(TAIL_BYTES)))?;
    let mut tail = Vec::new();
    file.read_to_end(&mut tail)?;
    let tail = String::from_utf8_lossy(&tail);
    let last_line = tail.lines().rev().find(|line| !line.trim().is_empty());
    Ok(last_line.is_some_and(|line| parse_event(line) != Some(EVENT_SESSION_END)))
}

#[cfg(test)]
mod tests {
    use jiff::civil::date;

    use super::*;

    /// A day file of `memo` on 2026-11-`day`.
    fn november(day: i8, part: u8) -> DayFile {
        DayFile {
            day: date(2026, 11, day),
            part,
        }
    }

    #[test]
    fn names_round_trip() {
        assert_eq!(november(9, 0).file_name("memo"), "memo-20261109.log");
        assert_eq!(november(9, 2).file_name("memo"), "memo-20261109_2.log");
        for file in [november(9, 0), november(9, 4), november(30, 1)] {
            assert_eq!(DayFile::parse("memo", &file.file_name("memo")), Some(file));
        }
    }

    #[test]
    fn foreign_names_are_not_ours() {
        for name in [
            "memo.log",
            "memo-20261109.txt",
            "event-20261109.log",
            "memo-2026-11-09.log",
            "memo-20261309.log",
            "memo-20261109_0.log",
            "memo-20261109_01.log",
            "memo-20261109_+1.log",
            "memo-20261109_.log",
            "memo-20261109_1.log.bak",
            "notes.json",
        ] {
            assert_eq!(DayFile::parse("memo", name), None, "{name}");
        }
    }

    #[test]
    fn files_sort_by_day_then_part() {
        let mut files = [
            november(10, 0),
            november(9, 2),
            november(9, 1),
            november(9, 0),
        ];
        files.sort_unstable();
        assert_eq!(
            files,
            [
                november(9, 0),
                november(9, 1),
                november(9, 2),
                november(10, 0)
            ]
        );
    }

    /// Creates the day files `days` (with part 0 and 1) in `logs/memo`.
    fn create(logs: &Path, addon: &str, days: &[Date]) -> io::Result<()> {
        let folder = logs.join(addon);
        fs::create_dir_all(&folder)?;
        for day in days {
            for part in [0, 1] {
                fs::write(
                    folder.join(DayFile { day: *day, part }.file_name(addon)),
                    "x\n",
                )?;
            }
        }
        Ok(())
    }

    #[test]
    fn cleanup_keeps_the_newest_days_and_nothing_older_than_thirty_days() -> io::Result<()> {
        let logs = tempfile::tempdir()?;
        let today = date(2026, 12, 20);
        // 20 consecutive days, the newest being today.
        let days: Vec<Date> = (0..20)
            .map(|offset| {
                today
                    .checked_sub(Span::new().days(offset))
                    .map_err(io::Error::other)
            })
            .collect::<Result<_, _>>()?;
        create(logs.path(), "memo", &days)?;
        fs::write(logs.path().join("memo").join("notes.txt"), "mine")?;

        cleanup(logs.path(), today)?;

        let kept = list(&logs.path().join("memo"), "memo")?;
        let kept_days: Vec<Date> = kept.iter().map(|file| file.day).collect();
        assert_eq!(kept.len(), 2 * KEPT_DAYS, "both parts of 14 days stay");
        assert_eq!(kept_days.first(), days.get(KEPT_DAYS - 1));
        assert_eq!(kept_days.last(), days.first());
        assert!(logs.path().join("memo").join("notes.txt").is_file());
        Ok(())
    }

    #[test]
    fn a_pause_does_not_delete_the_last_days_but_the_age_limit_does() -> io::Result<()> {
        let logs = tempfile::tempdir()?;
        // The last session was 10 days ago: all stay. A session 40 days ago goes.
        let today = date(2026, 12, 20);
        create(
            logs.path(),
            "memo",
            &[date(2026, 12, 10), date(2026, 11, 10)],
        )?;

        cleanup(logs.path(), today)?;

        let days: Vec<Date> = list(&logs.path().join("memo"), "memo")?
            .iter()
            .map(|file| file.day)
            .collect();
        assert_eq!(days, [date(2026, 12, 10), date(2026, 12, 10)]);
        Ok(())
    }

    #[test]
    fn cleanup_removes_empty_folders_of_other_addons_but_not_foreign_content() -> io::Result<()> {
        let logs = tempfile::tempdir()?;
        let today = date(2026, 12, 20);
        create(logs.path(), "event", &[date(2026, 10, 1)])?;
        create(logs.path(), "memo", &[today])?;
        fs::create_dir_all(logs.path().join("other"))?;
        fs::write(logs.path().join("other").join("keep.txt"), "mine")?;
        fs::write(logs.path().join("memo.log"), "old flat log")?;

        cleanup(logs.path(), today)?;

        assert!(
            !logs.path().join("event").exists(),
            "expired folder is removed"
        );
        assert!(logs.path().join("memo").is_dir());
        assert!(logs.path().join("other").join("keep.txt").is_file());
        assert!(
            logs.path().join("memo.log").is_file(),
            "a loose file is not ours"
        );
        Ok(())
    }

    #[test]
    fn cleanup_without_a_logs_folder_is_fine() -> io::Result<()> {
        let folder = tempfile::tempdir()?;
        cleanup(&folder.path().join("missing"), date(2026, 12, 20))
    }

    #[test]
    fn an_unclean_session_is_found_from_the_last_line() -> io::Result<()> {
        let folder = tempfile::tempdir()?;
        let path = folder.path().join("memo-20261109.log");
        let line = |event: &str| {
            format!("2026-11-09T12:00:00+01:00 INFO addon=memo version=0.3.0 {event} at=x:1\n")
        };

        fs::write(
            &path,
            format!("{}{}", line("session_start"), line("session_end")),
        )?;
        assert!(!previous_session_unclean(&path)?);
        fs::write(
            &path,
            format!("{}{}", line("session_start"), line("save_failed")),
        )?;
        assert!(previous_session_unclean(&path)?);
        // A write cut off by a crash.
        fs::write(
            &path,
            format!("{}2026-11-09T12:00:01+01", line("session_start")),
        )?;
        assert!(previous_session_unclean(&path)?);
        fs::write(&path, "")?;
        assert!(!previous_session_unclean(&path)?);
        Ok(())
    }

    #[test]
    fn a_long_session_is_judged_by_the_end_of_its_file() -> io::Result<()> {
        let folder = tempfile::tempdir()?;
        let path = folder.path().join("memo-20261109.log");
        let filler =
            "2026-11-09T12:00:00+01:00 INFO addon=memo version=0.3.0 save_finished at=x:1\n"
                .repeat(1000);
        let end = "2026-11-09T13:00:00+01:00 INFO addon=memo version=0.3.0 session_end at=x:1\n";
        fs::write(&path, format!("{filler}{end}"))?;
        assert!(!previous_session_unclean(&path)?);
        fs::write(&path, filler)?;
        assert!(previous_session_unclean(&path)?);
        Ok(())
    }

    #[test]
    fn a_missing_newline_is_detected() -> io::Result<()> {
        let folder = tempfile::tempdir()?;
        let path = folder.path().join("x.log");
        fs::write(&path, "abc\n")?;
        assert!(ends_with_newline(&path)?);
        fs::write(&path, "abc")?;
        assert!(!ends_with_newline(&path)?);
        fs::write(&path, "")?;
        assert!(ends_with_newline(&path)?);
        Ok(())
    }

    #[test]
    fn log_files_lists_one_addon_oldest_first() -> io::Result<()> {
        let logs = tempfile::tempdir()?;
        create(
            logs.path(),
            "memo",
            &[date(2026, 11, 10), date(2026, 11, 9)],
        )?;
        create(logs.path(), "event", &[date(2026, 11, 8)])?;
        let names: Vec<String> = log_files(logs.path(), AddonName::new("memo"))?
            .iter()
            .filter_map(|path| path.file_name()?.to_str().map(str::to_owned))
            .collect();
        assert_eq!(
            names,
            [
                "memo-20261109.log",
                "memo-20261109_1.log",
                "memo-20261110.log",
                "memo-20261110_1.log"
            ]
        );
        Ok(())
    }
}
