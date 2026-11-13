//! The log export of the help window's problem page (ROADMAP R95).
//!
//! One text file with the newest [`EXPORT_DAYS`] days of every trailkit
//! addon that has a log, so a user can attach it to a public issue. It holds
//! a header (export time, addons with versions, the covered days and every
//! trim), then one block per addon with one section per day file, newest
//! first. The text never exceeds [`MAX_EXPORT_BYTES`]: if it would, the
//! oldest day goes first (for all addons), then the oldest lines of the
//! remaining day, shared evenly between the addons.
//!
//! Nothing here touches the Downloads folder or Explorer: the caller writes
//! [`LogExport::text`] to the file named by [`export_file_name`].

use std::{fmt::Write as _, fs, io, path::Path};

use jiff::{
    Timestamp,
    civil::Date,
    tz::{Offset, TimeZone},
};

use crate::day_files::{DayFile, list};

/// How many days with a log are exported per addon.
pub const EXPORT_DAYS: usize = 3;

/// The largest export in bytes (5 MiB).
pub const MAX_EXPORT_BYTES: usize = 5 * 1024 * 1024;

/// Room kept free for the header when the body is trimmed to fit.
const HEADER_RESERVE: usize = 8 * 1024;

/// The text shown for an addon without any log file.
const NO_LOGS: &str = "(no logs)";

/// A finished export.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LogExport {
    /// The complete text of the export file.
    pub text: String,
    /// Whether anything was left out to stay within [`MAX_EXPORT_BYTES`];
    /// the problem page then says so.
    pub trimmed: bool,
}

/// One day file of an addon with its lines read.
struct Section {
    /// The file name, e.g. `memo-20261112.log`.
    name: String,
    /// The day the file belongs to.
    day: Date,
    /// The file's lines.
    lines: Vec<String>,
}

/// One addon of the export.
struct Block {
    /// The folder (and short addon) name, e.g. `memo`.
    addon: String,
    /// The version of its newest log line, if any.
    version: Option<String>,
    /// Its sections, newest first.
    sections: Vec<Section>,
}

/// Whether any trailkit addon has a log file in `logs`; the problem page
/// disables "Export log" and says why if not.
///
/// # Errors
///
/// Returns the I/O error of listing `logs` or an addon folder.
pub fn has_logs(logs: &Path) -> io::Result<bool> {
    for addon in addon_folders(logs)? {
        if !list(&logs.join(&addon), &addon)?.is_empty() {
            return Ok(true);
        }
    }
    Ok(false)
}

/// The name of the export file for `day`: `trailkit-logs-<yyyymmdd>.txt`,
/// or with ` (2)`, ` (3)`, … before the extension for the first name
/// `is_taken` does not report as existing, so an earlier export is never
/// overwritten.
#[must_use]
pub fn export_file_name(day: Date, is_taken: impl Fn(&str) -> bool) -> String {
    let stem = format!(
        "trailkit-logs-{:04}{:02}{:02}",
        day.year(),
        day.month(),
        day.day()
    );
    let first = format!("{stem}.txt");
    if !is_taken(&first) {
        return first;
    }
    (2_u32..)
        .map(|number| format!("{stem} ({number}).txt"))
        .find(|name| !is_taken(name))
        .unwrap_or(first)
}

/// Builds the export of the logs in `logs` (`trailkit/logs/`), exported at
/// `exported_at` and shown in `offset` (the player's local time zone).
/// `None` if no addon has a log.
///
/// # Errors
///
/// Returns the I/O error of listing or reading the log files.
pub fn build_log_export(
    logs: &Path,
    exported_at: Timestamp,
    offset: Offset,
) -> io::Result<Option<LogExport>> {
    let mut blocks = read_blocks(logs)?;
    if blocks.iter().all(|block| block.sections.is_empty()) {
        return Ok(None);
    }
    let mut trims = Vec::new();
    trim_to_fit(&mut blocks, &mut trims);
    let text = render(&blocks, &trims, exported_at, offset);
    Ok(Some(LogExport {
        trimmed: !trims.is_empty(),
        text,
    }))
}

/// The names of the addon folders in `logs`, sorted. A missing `logs`
/// folder has none; files and folders with other names are ignored.
fn addon_folders(logs: &Path) -> io::Result<Vec<String>> {
    let entries = match fs::read_dir(logs) {
        Ok(entries) => entries,
        Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(Vec::new()),
        Err(error) => return Err(error),
    };
    let mut names = Vec::new();
    for entry in entries {
        let entry = entry?;
        if entry.file_type()?.is_dir()
            && let Some(name) = entry.file_name().to_str()
            && is_addon_folder_name(name)
        {
            names.push(name.to_owned());
        }
    }
    names.sort_unstable();
    Ok(names)
}

/// Whether `name` looks like an addon folder: lowercase ASCII letters,
/// digits and inner hyphens.
fn is_addon_folder_name(name: &str) -> bool {
    !name.is_empty()
        && !name.starts_with('-')
        && !name.ends_with('-')
        && name
            .bytes()
            .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'-')
}

/// Reads the newest [`EXPORT_DAYS`] days of every addon, newest section first.
fn read_blocks(logs: &Path) -> io::Result<Vec<Block>> {
    let mut blocks = Vec::new();
    for addon in addon_folders(logs)? {
        let folder = logs.join(&addon);
        let mut files = list(&folder, &addon)?;
        files.reverse();
        let mut days: Vec<Date> = files.iter().map(|file| file.day).collect();
        days.dedup();
        days.truncate(EXPORT_DAYS);
        files.retain(|file| days.contains(&file.day));
        let mut sections = Vec::new();
        for file in files {
            sections.push(read_section(&folder, &addon, file)?);
        }
        let version = sections.first().and_then(|section| {
            section
                .lines
                .iter()
                .rev()
                .find_map(|line| version_of(line).map(str::to_owned))
        });
        blocks.push(Block {
            addon,
            version,
            sections,
        });
    }
    Ok(blocks)
}

/// Reads one day file; invalid UTF-8 is replaced, not an error.
fn read_section(folder: &Path, addon: &str, file: DayFile) -> io::Result<Section> {
    let name = file.file_name(addon);
    let bytes = fs::read(folder.join(&name))?;
    Ok(Section {
        name,
        day: file.day,
        lines: String::from_utf8_lossy(&bytes)
            .lines()
            .map(str::to_owned)
            .collect(),
    })
}

/// The `version=` value of a log line (its fourth field), if it has one.
fn version_of(line: &str) -> Option<&str> {
    line.split_whitespace().nth(3)?.strip_prefix("version=")
}

/// The size of the body in bytes: every line with its newline plus the
/// section and block titles.
fn body_size(blocks: &[Block]) -> usize {
    blocks
        .iter()
        .map(|block| {
            title_size(block)
                + block.sections.iter().map(section_size).sum::<usize>()
                + if block.sections.is_empty() {
                    NO_LOGS.len() + 1
                } else {
                    0
                }
        })
        .sum()
}

/// The bytes of a block's title line and the blank line after the block.
fn title_size(block: &Block) -> usize {
    block_title(block).len() + 2
}

/// The bytes of a section's title line and lines.
fn section_size(section: &Section) -> usize {
    section_title(section).len()
        + 1
        + section
            .lines
            .iter()
            .map(|line| line.len() + 1)
            .sum::<usize>()
}

/// `===== <addon> <version> =====`.
fn block_title(block: &Block) -> String {
    match &block.version {
        Some(version) => format!("===== {} {version} =====", block.addon),
        None => format!("===== {} =====", block.addon),
    }
}

/// `--- <file> ---`.
fn section_title(section: &Section) -> String {
    format!("--- {} ---", section.name)
}

/// Leaves out the oldest day (all addons), then the oldest lines, until the
/// body fits; records what it left out in `trims`.
fn trim_to_fit(blocks: &mut [Block], trims: &mut Vec<String>) {
    let budget = MAX_EXPORT_BYTES - HEADER_RESERVE;
    while body_size(blocks) > budget {
        let days: Vec<Date> = {
            let mut days: Vec<Date> = blocks
                .iter()
                .flat_map(|block| block.sections.iter().map(|section| section.day))
                .collect();
            days.sort_unstable();
            days.dedup();
            days
        };
        if days.len() < 2 {
            break;
        }
        let oldest = days[0];
        for block in blocks.iter_mut() {
            block.sections.retain(|section| section.day != oldest);
        }
        trims.push(format!(
            "{} left out for all addons (size limit)",
            iso_day(oldest)
        ));
    }
    if body_size(blocks) > budget {
        trim_lines(blocks, budget, trims);
    }
}

/// Drops the oldest lines of every addon so each keeps an equal share of
/// `budget`; only reached when a single day alone is too large.
fn trim_lines(blocks: &mut [Block], budget: usize, trims: &mut Vec<String>) {
    let share = budget / blocks.len().max(1);
    for block in blocks.iter_mut() {
        let mut dropped = 0_usize;
        while block_size(block) > share {
            let Some(section) = block
                .sections
                .iter_mut()
                .rev()
                .find(|section| !section.lines.is_empty())
            else {
                break;
            };
            section.lines.remove(0);
            dropped += 1;
        }
        if dropped > 0 {
            trims.push(format!(
                "{}: the oldest {dropped} lines left out (size limit)",
                block.addon
            ));
        }
    }
}

/// The size of one block's title, sections and lines.
fn block_size(block: &Block) -> usize {
    title_size(block) + block.sections.iter().map(section_size).sum::<usize>()
}

/// `yyyy-mm-dd`.
fn iso_day(day: Date) -> String {
    format!("{:04}-{:02}-{:02}", day.year(), day.month(), day.day())
}

/// Writes the header and the blocks.
fn render(blocks: &[Block], trims: &[String], exported_at: Timestamp, offset: Offset) -> String {
    let mut text = String::with_capacity(body_size(blocks) + HEADER_RESERVE);
    let local = exported_at.to_zoned(TimeZone::fixed(offset));
    let days = blocks
        .iter()
        .flat_map(|block| block.sections.iter().map(|section| section.day));
    let range = match (days.clone().min(), days.max()) {
        (Some(first), Some(last)) if first != last => {
            format!("{} to {}", iso_day(first), iso_day(last))
        }
        (Some(day), _) => iso_day(day),
        (None, _) => "none".to_owned(),
    };
    let addons = blocks
        .iter()
        .map(|block| match &block.version {
            Some(version) => format!("{} {version}", block.addon),
            None => block.addon.clone(),
        })
        .collect::<Vec<_>>()
        .join(", ");
    // Writing to a `String` cannot fail.
    let _ = writeln!(text, "trailkit log export");
    let _ = writeln!(text, "exported: {}", local.strftime("%Y-%m-%dT%H:%M:%S%:z"));
    let _ = writeln!(text, "days: {range}");
    let _ = writeln!(text, "addons: {addons}");
    if trims.is_empty() {
        let _ = writeln!(text, "trimmed: no");
    } else {
        for trim in trims {
            let _ = writeln!(text, "trimmed: {trim}");
        }
    }
    text.push('\n');
    for block in blocks {
        let _ = writeln!(text, "{}", block_title(block));
        if block.sections.is_empty() {
            let _ = writeln!(text, "{NO_LOGS}");
        }
        for section in &block.sections {
            let _ = writeln!(text, "{}", section_title(section));
            for line in &section.lines {
                text.push_str(line);
                text.push('\n');
            }
        }
        text.push('\n');
    }
    text
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A day file with `lines` lines of `addon` version 1.2.3.
    fn write_log(logs: &Path, addon: &str, name: &str, lines: usize) {
        let folder = logs.join(addon);
        fs::create_dir_all(&folder).expect("folder");
        let content: String = (0..lines)
            .map(|index| {
                format!(
                    "2026-11-12T10:00:{:02}+01:00 INFO addon={addon} version=1.2.3 event_{index}\n",
                    index % 60
                )
            })
            .collect();
        fs::write(folder.join(name), content).expect("log file");
    }

    fn at(logs: &Path) -> LogExport {
        let when: Timestamp = "2026-11-12T09:15:00Z".parse().expect("time");
        let offset = Offset::from_seconds(3600).expect("offset");
        build_log_export(logs, when, offset)
            .expect("export")
            .expect("some logs")
    }

    #[test]
    fn no_logs_means_no_export() {
        let directory = tempfile::tempdir().expect("directory");
        assert!(!has_logs(directory.path()).expect("listing"));
        let when: Timestamp = "2026-11-12T09:15:00Z".parse().expect("time");
        let offset = Offset::from_seconds(0).expect("offset");
        assert_eq!(
            build_log_export(directory.path(), when, offset).expect("export"),
            None
        );
    }

    #[test]
    fn the_export_has_a_header_and_a_block_per_addon_newest_file_first() {
        let directory = tempfile::tempdir().expect("directory");
        write_log(directory.path(), "memo", "memo-20261111.log", 2);
        write_log(directory.path(), "memo", "memo-20261112.log", 2);
        write_log(directory.path(), "event", "event-20261112.log", 1);
        assert!(has_logs(directory.path()).expect("listing"));
        let export = at(directory.path());
        assert!(!export.trimmed);
        let text = export.text;
        assert!(text.starts_with("trailkit log export\nexported: 2026-11-12T10:15:00+01:00\n"));
        assert!(text.contains("days: 2026-11-11 to 2026-11-12\n"));
        assert!(text.contains("addons: event 1.2.3, memo 1.2.3\n"));
        assert!(text.contains("trimmed: no\n"));
        let newest = text.find("--- memo-20261112.log ---").expect("newest");
        let older = text.find("--- memo-20261111.log ---").expect("older");
        assert!(newest < older, "newest first");
        assert!(text.contains("===== event 1.2.3 ====="));
    }

    #[test]
    fn only_the_newest_three_days_are_exported() {
        let directory = tempfile::tempdir().expect("directory");
        for day in ["20261108", "20261109", "20261110", "20261111"] {
            write_log(directory.path(), "memo", &format!("memo-{day}.log"), 1);
        }
        let text = at(directory.path()).text;
        assert!(!text.contains("memo-20261108.log"));
        for day in ["20261109", "20261110", "20261111"] {
            assert!(text.contains(&format!("memo-{day}.log")), "{day}");
        }
    }

    #[test]
    fn the_parts_of_a_day_are_sections_and_foreign_files_are_ignored() {
        let directory = tempfile::tempdir().expect("directory");
        write_log(directory.path(), "memo", "memo-20261112.log", 1);
        write_log(directory.path(), "memo", "memo-20261112_1.log", 1);
        fs::write(directory.path().join("memo").join("notes.txt"), "secret").expect("foreign");
        fs::write(directory.path().join("readme.txt"), "x").expect("file");
        let text = at(directory.path()).text;
        let part = text.find("--- memo-20261112_1.log ---").expect("part");
        let base = text.find("--- memo-20261112.log ---").expect("base");
        assert!(part < base, "the newest part comes first");
        assert!(!text.contains("secret"));
    }

    #[test]
    fn an_addon_folder_without_log_files_says_so() {
        let directory = tempfile::tempdir().expect("directory");
        write_log(directory.path(), "memo", "memo-20261112.log", 1);
        fs::create_dir(directory.path().join("event")).expect("empty folder");
        let text = at(directory.path()).text;
        assert!(text.contains("===== event =====\n(no logs)\n"));
    }

    #[test]
    fn too_large_exports_lose_the_oldest_day_first() {
        let directory = tempfile::tempdir().expect("directory");
        // Each day is about 2 MiB (30,000 lines of about 70 bytes), so three days exceed the 5 MiB limit.
        for day in ["20261110", "20261111", "20261112"] {
            write_log(directory.path(), "memo", &format!("memo-{day}.log"), 30_000);
        }
        let export = at(directory.path());
        assert!(export.trimmed);
        assert!(export.text.len() <= MAX_EXPORT_BYTES);
        assert!(
            export
                .text
                .contains("trimmed: 2026-11-10 left out for all addons")
        );
        assert!(!export.text.contains("memo-20261110.log"));
        assert!(export.text.contains("memo-20261112.log"));
    }

    #[test]
    fn a_single_oversized_day_loses_its_oldest_lines() {
        let directory = tempfile::tempdir().expect("directory");
        write_log(directory.path(), "memo", "memo-20261112.log", 100_000);
        let export = at(directory.path());
        assert!(export.trimmed);
        assert!(export.text.len() <= MAX_EXPORT_BYTES);
        assert!(export.text.contains("trimmed: memo: the oldest "));
        assert!(export.text.contains("event_99999"), "the newest line stays");
        assert!(!export.text.contains(" event_0\n"), "the oldest line goes");
    }

    #[test]
    fn the_file_name_counts_up_instead_of_overwriting() {
        let day = Date::new(2026, 11, 2).expect("date");
        assert_eq!(
            export_file_name(day, |_| false),
            "trailkit-logs-20261102.txt"
        );
        let taken = [
            "trailkit-logs-20261102.txt",
            "trailkit-logs-20261102 (2).txt",
        ];
        assert_eq!(
            export_file_name(day, |name| taken.contains(&name)),
            "trailkit-logs-20261102 (3).txt"
        );
    }
}
