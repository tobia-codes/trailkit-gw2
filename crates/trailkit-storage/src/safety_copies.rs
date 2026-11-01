//! Safety copies in the `backups/` folder, each kind capped (memo plan M33).
//!
//! - Migration copy `<stem>.v<N>.bak.json`: the original file before it was
//!   migrated from version `N`. Only the newest one per document is kept, and
//!   it is deleted 30 days after it was written.
//! - Copy of an unreadable file `<stem>.corrupt-<YYYYMMDD-HHMMSS>.json`: at
//!   most [`MAX_UNREADABLE_COPIES`] per document, the oldest is deleted first.
//!
//! Pruning is best effort: failures are logged and never block loading.

use std::{
    fs, io,
    path::{Path, PathBuf},
    time::{Duration, SystemTime},
};

use jiff::Timestamp;

use crate::{SchemaVersion, write_atomically};

/// How many copies of unreadable files are kept per document.
pub const MAX_UNREADABLE_COPIES: usize = 3;

/// How long a migration copy is kept after it was written.
pub const MIGRATION_COPY_RETENTION: Duration = Duration::from_secs(30 * 24 * 60 * 60);

/// Writes `original_content` as the migration copy for `file_stem` at
/// `from_version`, replacing any older migration copy of the same document.
///
/// # Errors
///
/// Returns the I/O error if the backups folder cannot be created or the copy
/// cannot be written. Removing older copies is best effort.
pub fn write_migration_copy(
    backups_directory: &Path,
    file_stem: &str,
    from_version: SchemaVersion,
    original_content: &[u8],
) -> io::Result<PathBuf> {
    fs::create_dir_all(backups_directory)?;
    let copy_path = backups_directory.join(format!("{file_stem}.v{from_version}.bak.json"));
    // Written fresh (not `fs::copy`) so the file's modification time is "now",
    // which the 30-day retention is measured from.
    write_atomically(&copy_path, original_content)?;
    for path in files_matching(backups_directory, |name| is_migration_copy(name, file_stem)) {
        if path != copy_path {
            remove_logged(&path);
        }
    }
    Ok(copy_path)
}

/// Moves the unreadable file at `source` into the backups folder as
/// `<stem>.corrupt-<time>.json` and deletes the oldest such copies beyond
/// [`MAX_UNREADABLE_COPIES`].
///
/// # Errors
///
/// Returns the I/O error if the backups folder cannot be created or the file
/// cannot be moved; the file then stays where it was.
pub fn preserve_unreadable_file(
    source: &Path,
    backups_directory: &Path,
    file_stem: &str,
    now: Timestamp,
) -> io::Result<PathBuf> {
    fs::create_dir_all(backups_directory)?;
    let time = now.strftime("%Y%m%d-%H%M%S").to_string();
    let mut target = backups_directory.join(format!("{file_stem}.corrupt-{time}.json"));
    let mut collision_counter = 1;
    while target.exists() {
        collision_counter += 1;
        target = backups_directory.join(format!(
            "{file_stem}.corrupt-{time}-{collision_counter}.json"
        ));
    }
    fs::rename(source, &target)?;

    // The time stamp in the name sorts chronologically, so the oldest come first.
    let mut copies = files_matching(backups_directory, |name| {
        is_unreadable_copy(name, file_stem)
    });
    copies.sort();
    let excess = copies.len().saturating_sub(MAX_UNREADABLE_COPIES);
    for path in &copies[..excess] {
        remove_logged(path);
    }
    Ok(target)
}

/// Deletes migration copies of `file_stem` that were written more than
/// [`MIGRATION_COPY_RETENTION`] before `now`.
pub fn prune_expired_migration_copies(backups_directory: &Path, file_stem: &str, now: Timestamp) {
    let now = SystemTime::from(now);
    for path in files_matching(backups_directory, |name| is_migration_copy(name, file_stem)) {
        let written_at = fs::metadata(&path).and_then(|metadata| metadata.modified());
        match written_at {
            Ok(written_at) => {
                let is_expired = now
                    .duration_since(written_at)
                    .is_ok_and(|age| age > MIGRATION_COPY_RETENTION);
                if is_expired {
                    remove_logged(&path);
                }
            }
            Err(error) => log::warn!("cannot read age of {}: {error}", path.display()),
        }
    }
}

/// Whether `file_name` is a migration copy of `file_stem` (`<stem>.v<N>.bak.json`).
fn is_migration_copy(file_name: &str, file_stem: &str) -> bool {
    file_name
        .strip_prefix(file_stem)
        .and_then(|rest| rest.strip_prefix(".v"))
        .and_then(|rest| rest.strip_suffix(".bak.json"))
        .is_some_and(|version| {
            !version.is_empty() && version.bytes().all(|byte| byte.is_ascii_digit())
        })
}

/// Whether `file_name` is a preserved unreadable copy of `file_stem`.
fn is_unreadable_copy(file_name: &str, file_stem: &str) -> bool {
    file_name
        .strip_prefix(file_stem)
        .and_then(|rest| rest.strip_prefix(".corrupt-"))
        .is_some_and(|rest| rest.ends_with(".json"))
}

/// Lists the files in `directory` whose name satisfies `matches`. A missing or
/// unreadable directory yields an empty list (logged unless missing).
fn files_matching(directory: &Path, matches: impl Fn(&str) -> bool) -> Vec<PathBuf> {
    let entries = match fs::read_dir(directory) {
        Ok(entries) => entries,
        Err(error) => {
            if error.kind() != io::ErrorKind::NotFound {
                log::warn!("cannot list {}: {error}", directory.display());
            }
            return Vec::new();
        }
    };
    entries
        .filter_map(Result::ok)
        .filter(|entry| entry.file_type().is_ok_and(|kind| kind.is_file()))
        .filter(|entry| entry.file_name().to_str().is_some_and(&matches))
        .map(|entry| entry.path())
        .collect()
}

/// Deletes `path`, logging (not returning) a failure.
fn remove_logged(path: &Path) {
    if let Err(error) = fs::remove_file(path) {
        log::warn!("cannot delete {}: {error}", path.display());
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A fixed point in time for deterministic file names.
    fn noon() -> Timestamp {
        "2026-11-01T12:00:00Z".parse().unwrap_or_default()
    }

    #[test]
    fn recognizes_copy_names() {
        assert!(is_migration_copy("notes.v1.bak.json", "notes"));
        assert!(is_migration_copy("notes.v12.bak.json", "notes"));
        assert!(!is_migration_copy("notes.v.bak.json", "notes"));
        assert!(!is_migration_copy("settings.v1.bak.json", "notes"));
        assert!(is_unreadable_copy(
            "notes.corrupt-20261101-120000.json",
            "notes"
        ));
        assert!(!is_unreadable_copy("notes.json", "notes"));
    }

    #[test]
    fn keeps_only_the_newest_migration_copy() -> io::Result<()> {
        let directory = tempfile::tempdir()?;
        write_migration_copy(directory.path(), "notes", SchemaVersion(1), b"v1")?;
        let newest = write_migration_copy(directory.path(), "notes", SchemaVersion(2), b"v2")?;

        let copies = files_matching(directory.path(), |name| is_migration_copy(name, "notes"));
        assert_eq!(copies, vec![newest]);
        Ok(())
    }

    #[test]
    fn migration_copy_expires_after_retention() -> io::Result<()> {
        let directory = tempfile::tempdir()?;
        let copy = write_migration_copy(directory.path(), "notes", SchemaVersion(1), b"v1")?;
        let now = Timestamp::now();

        prune_expired_migration_copies(directory.path(), "notes", now);
        assert!(copy.exists(), "a fresh copy is kept");

        let later = now
            .checked_add(MIGRATION_COPY_RETENTION + Duration::from_secs(60))
            .unwrap_or(now);
        prune_expired_migration_copies(directory.path(), "notes", later);
        assert!(!copy.exists(), "an expired copy is deleted");
        Ok(())
    }

    #[test]
    fn keeps_at_most_three_unreadable_copies() -> io::Result<()> {
        let directory = tempfile::tempdir()?;
        let backups = directory.path().join("backups");
        let live = directory.path().join("notes.json");
        let mut kept = Vec::new();
        for minute in 0..5 {
            fs::write(&live, b"garbage")?;
            let now = noon()
                .checked_add(Duration::from_secs(minute * 60))
                .unwrap_or_else(|_| noon());
            kept.push(preserve_unreadable_file(&live, &backups, "notes", now)?);
        }

        let mut copies = files_matching(&backups, |name| is_unreadable_copy(name, "notes"));
        copies.sort();
        assert_eq!(copies, kept[2..].to_vec());
        assert!(!live.exists());
        Ok(())
    }

    #[test]
    fn same_second_does_not_overwrite_a_copy() -> io::Result<()> {
        let directory = tempfile::tempdir()?;
        let backups = directory.path().join("backups");
        let live = directory.path().join("notes.json");
        fs::write(&live, b"first")?;
        let first = preserve_unreadable_file(&live, &backups, "notes", noon())?;
        fs::write(&live, b"second")?;
        let second = preserve_unreadable_file(&live, &backups, "notes", noon())?;

        assert_ne!(first, second);
        assert_eq!(fs::read(first)?, b"first");
        assert_eq!(fs::read(second)?, b"second");
        Ok(())
    }
}
