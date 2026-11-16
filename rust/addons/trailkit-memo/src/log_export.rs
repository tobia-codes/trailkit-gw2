//! The two steps of the problem page that leave the help window (ROADMAP
//! R95, R100): writing the log export into a folder (Downloads) and the
//! address of the GitHub issue form. Platform independent; finding the
//! Downloads folder, starting Explorer and the worker thread live in the
//! Windows glue.

use std::{
    fmt, fs,
    io::{self, Write as _},
    path::{Path, PathBuf},
};

use jiff::{
    Timestamp,
    tz::{Offset, TimeZone},
};
use trailkit_log::{build_log_export, export_file_name};

/// The file name of the repository's issue form for problems
/// (`.github/ISSUE_TEMPLATE/`).
const ISSUE_FORM: &str = "report-a-problem.yml";

/// What an export did.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ExportOutcome {
    /// The export was written.
    Saved {
        /// The whole path of the new file, for Explorer.
        path: PathBuf,
        /// The bare file name, for the status line.
        file_name: String,
        /// Whether older days or lines were left out (5 MiB limit).
        trimmed: bool,
    },
    /// No trailkit addon has a log yet; nothing was written.
    NoLogs,
}

/// The step of an export that failed, for the `log_export_failed` line.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExportStage {
    /// Listing or reading the log files.
    Read,
    /// Finding the Downloads folder.
    Downloads,
    /// Writing the export file.
    Write,
}

impl ExportStage {
    /// The value of the `stage` field.
    #[must_use]
    pub const fn code(self) -> &'static str {
        match self {
            Self::Read => "read",
            Self::Downloads => "downloads",
            Self::Write => "write",
        }
    }
}

/// A failed export: the step and its I/O error.
#[derive(Debug)]
pub struct ExportError {
    /// The step that failed.
    pub stage: ExportStage,
    /// The cause.
    pub source: io::Error,
}

impl fmt::Display for ExportError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "log export failed at {}", self.stage.code())
    }
}

impl std::error::Error for ExportError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        Some(&self.source)
    }
}

/// Builds the export of the logs in `logs` (`trailkit/logs/`) at
/// `exported_at` in the player's `offset` and writes it into `folder` as
/// `trailkit-logs-<yyyymmdd>.txt`, or ` (2)` and so on when that name is
/// taken; an existing file is never overwritten, even one created after the
/// name was chosen.
///
/// # Errors
///
/// Returns the step that failed with its I/O error: reading the logs or
/// writing the file.
pub fn write_log_export(
    logs: &Path,
    folder: &Path,
    exported_at: Timestamp,
    offset: Offset,
) -> Result<ExportOutcome, ExportError> {
    let failed_at = |stage| move |source| ExportError { stage, source };
    let Some(export) =
        build_log_export(logs, exported_at, offset).map_err(failed_at(ExportStage::Read))?
    else {
        return Ok(ExportOutcome::NoLogs);
    };
    let day = exported_at.to_zoned(TimeZone::fixed(offset)).date();
    // Another program may create the chosen name in between: then take the next.
    loop {
        let file_name = export_file_name(day, |name| folder.join(name).exists());
        let path = folder.join(&file_name);
        match fs::File::create_new(&path) {
            Ok(mut file) => {
                file.write_all(export.text.as_bytes())
                    .and_then(|()| file.sync_all())
                    .map_err(failed_at(ExportStage::Write))?;
                return Ok(ExportOutcome::Saved {
                    path,
                    file_name,
                    trimmed: export.trimmed,
                });
            }
            Err(error) if error.kind() == io::ErrorKind::AlreadyExists => {}
            Err(error) => return Err(failed_at(ExportStage::Write)(error)),
        }
    }
}

/// The address of the repository's issue form "Report a problem" with
/// `addon` and `version` filled in (memo Q170, R95). Only these two values
/// go into the address; nothing else is sent.
#[must_use]
pub fn problem_report_url(repository: &str, addon: &str, version: &str) -> String {
    format!(
        "{}/issues/new?template={ISSUE_FORM}&addon={}&version={}",
        repository.trim_end_matches('/'),
        query_value(addon),
        query_value(version),
    )
}

/// `value` percent-encoded for a URL query: every byte outside the
/// unreserved characters (RFC 3986) is escaped.
fn query_value(value: &str) -> String {
    let mut encoded = String::with_capacity(value.len());
    for byte in value.bytes() {
        if byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'.' | b'_' | b'~') {
            encoded.push(char::from(byte));
        } else {
            encoded.push_str(&format!("%{byte:02X}"));
        }
    }
    encoded
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 2026-11-12 10:15 UTC.
    fn exported_at() -> Timestamp {
        "2026-11-12T10:15:00Z".parse().expect("valid time")
    }

    /// A `logs` folder with one memo day file.
    fn logs_with_one_file(root: &Path) -> PathBuf {
        let logs = root.join("logs");
        fs::create_dir_all(logs.join("memo")).expect("create log folder");
        fs::write(
            logs.join("memo").join("memo-20261112.log"),
            "10:00:00.000 INFO  session_start addon=trailkit-memo version=0.3.0\n",
        )
        .expect("write log");
        logs
    }

    #[test]
    fn writes_the_export_and_never_overwrites() {
        let root = tempfile::tempdir().expect("temp dir");
        let logs = logs_with_one_file(root.path());
        let downloads = root.path().join("Downloads");
        fs::create_dir(&downloads).expect("create Downloads");
        fs::write(downloads.join("trailkit-logs-20261112.txt"), "older").expect("write");

        let outcome =
            write_log_export(&logs, &downloads, exported_at(), Offset::UTC).expect("export");

        let ExportOutcome::Saved {
            path,
            file_name,
            trimmed,
        } = outcome
        else {
            panic!("expected a saved export, got {outcome:?}");
        };
        assert_eq!(file_name, "trailkit-logs-20261112 (2).txt");
        assert_eq!(path, downloads.join(&file_name));
        assert!(!trimmed);
        let older = fs::read_to_string(downloads.join("trailkit-logs-20261112.txt"));
        assert_eq!(older.expect("read"), "older");
        assert!(
            fs::read_to_string(path)
                .expect("read")
                .contains("session_start")
        );
    }

    #[test]
    fn the_file_is_named_after_the_local_day() {
        let root = tempfile::tempdir().expect("temp dir");
        let logs = logs_with_one_file(root.path());
        let late = "2026-11-12T23:30:00Z".parse().expect("valid time");
        let berlin = Offset::from_hours(1).expect("valid offset");

        let outcome = write_log_export(&logs, root.path(), late, berlin).expect("export");

        assert!(matches!(
            outcome,
            ExportOutcome::Saved { file_name, .. } if file_name == "trailkit-logs-20261113.txt"
        ));
    }

    #[test]
    fn nothing_is_written_without_logs() {
        let root = tempfile::tempdir().expect("temp dir");
        let downloads = root.path().join("Downloads");
        fs::create_dir(&downloads).expect("create Downloads");

        let outcome = write_log_export(
            &root.path().join("logs"),
            &downloads,
            exported_at(),
            Offset::UTC,
        )
        .expect("export");

        assert_eq!(outcome, ExportOutcome::NoLogs);
        assert_eq!(fs::read_dir(&downloads).expect("list").count(), 0);
    }

    #[test]
    fn a_missing_folder_fails_at_writing() {
        let root = tempfile::tempdir().expect("temp dir");
        let logs = logs_with_one_file(root.path());

        let error = write_log_export(
            &logs,
            &root.path().join("missing"),
            exported_at(),
            Offset::UTC,
        )
        .expect_err("no folder");

        assert_eq!(error.stage, ExportStage::Write);
    }

    #[test]
    fn the_report_url_fills_in_addon_and_version() {
        assert_eq!(
            problem_report_url(
                "https://github.com/tobia-codes/trailkit-gw2",
                "trailkit-memo",
                "0.3.0+dev 1"
            ),
            "https://github.com/tobia-codes/trailkit-gw2/issues/new?template=report-a-problem.yml\
             &addon=trailkit-memo&version=0.3.0%2Bdev%201"
        );
    }
}
